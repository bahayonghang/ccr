// 🔐 Codex Auth 服务层
// 管理 Codex CLI 的多账号登录状态
//
// 核心职责:
// - 📋 检测登录状态
// - 💾 保存/切换/删除账号
// - 🔍 解析 JWT 提取账号信息
// - ⏰ 计算 Token 新鲜度
// - 🔄 进程检测与备份管理

use super::codex_runtime_service::{
    CodexAuthCacheAction, CodexRuntimeCommitPlan, CodexRuntimeService,
};
use super::openai_quota_core::normalize_openai_plan;
use crate::managers::codex_config::CodexConfigManager;
use crate::models::PlatformConfig;
use crate::models::{
    AuthIntent, AuthState, AuthStateStatus, CodexAuthAccount, CodexAuthEncryptedExport,
    CodexAuthExport, CodexAuthExportAccount, CodexAuthItem, CodexAuthJson, CodexAuthRegistry,
    CodexRuntimeMode, CodexRuntimeSummary, CredentialStoreKind, CurrentAuthInfo, ImportFormat,
    ImportMode, ImportResult, LoginState, OpenAiAuthMethod, PlatformPaths,
    normalize_auth_map_for_intent,
};
use crate::platforms::codex::CodexPlatform;
use crate::utils::CodexPaths;
use ccr_core::core::error::{CcrError, Result};
use ccr_core::core::lock::LockManager;
use chrono::{DateTime, Utc};
use std::path::PathBuf;
use std::{env, fs};
use tracing::{debug, warn};

/// 备份保留数量
#[allow(dead_code)]
const MAX_BACKUPS: usize = 10;
const OPENAI_DEFAULT_BASE_URL: &str = "https://api.openai.com/v1";
const THIRD_PARTY_RUNTIME_PROVIDER_KEY: &str = "custom";

/// Codex Auth 服务
///
/// 提供 Codex 多账号管理的所有业务逻辑
pub struct CodexAuthService {
    /// CCR 平台数据目录 (~/.ccr/platforms/codex/)
    ccr_codex_dir: PathBuf,
    /// Codex CLI 配置目录 (~/.codex/)
    codex_dir: PathBuf,
    /// Lock directory captured at construction so operations cannot drift with process env.
    lock_dir: PathBuf,
}

struct CurrentAuthDocuments {
    raw: serde_json::Map<String, serde_json::Value>,
    auth: CodexAuthJson,
}

#[allow(dead_code)]
pub struct AuthReadSnapshot {
    pub auth_state: AuthState,
    pub login_state: LoginState,
    pub current_info: Option<CurrentAuthInfo>,
    pub registry: CodexAuthRegistry,
    pub current_account_name: Option<String>,
}

impl CodexAuthService {
    /// 创建新的 CodexAuthService 实例
    pub fn new() -> Result<Self> {
        let paths = CodexPaths::resolve()?;
        let lock_dir = env::var_os("CCR_LOCK_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| paths.codex_dir.join(".locks"));
        Ok(Self {
            ccr_codex_dir: paths.ccr_codex_dir,
            codex_dir: paths.codex_dir,
            lock_dir,
        })
    }

    /// 从显式路径构造（用于测试注入与非标准工作目录场景）
    pub fn from_dirs(ccr_codex_dir: PathBuf, codex_dir: PathBuf) -> Self {
        let lock_dir = codex_dir.join(".locks");
        Self {
            ccr_codex_dir,
            codex_dir,
            lock_dir,
        }
    }

    fn ccr_root_dir(&self) -> PathBuf {
        let is_standard_platform_dir = self
            .ccr_codex_dir
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.eq_ignore_ascii_case("codex"))
            && self
                .ccr_codex_dir
                .parent()
                .and_then(|parent| parent.file_name())
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.eq_ignore_ascii_case("platforms"));

        if is_standard_platform_dir {
            self.ccr_codex_dir
                .parent()
                .and_then(|parent| parent.parent())
                .map(|root| root.to_path_buf())
                .unwrap_or_else(|| self.ccr_codex_dir.clone())
        } else {
            self.ccr_codex_dir.clone()
        }
    }

    fn platform_paths(&self) -> PlatformPaths {
        let root = self.ccr_root_dir();
        PlatformPaths {
            registry_file: root.join("config.toml"),
            platform_dir: self.ccr_codex_dir.clone(),
            profiles_file: self.ccr_codex_dir.join("profiles.toml"),
            settings_file: self.ccr_codex_dir.join("settings.json"),
            history_file: root.join("history").join("codex.json"),
            backups_dir: root.join("backups").join("codex"),
            root,
        }
    }

    fn platform(&self) -> Result<CodexPlatform> {
        Ok(CodexPlatform::from_parts(
            self.platform_paths(),
            self.codex_config_manager()?,
            self.runtime_service()?,
        ))
    }

    fn current_profile_name(&self) -> Result<Option<String>> {
        let registry_path = self.platform_paths().registry_file;
        let manager = ccr_config::PlatformConfigManager::new(registry_path);
        let unified = manager.load_or_create_default()?;

        match unified.get_platform_profile("codex") {
            Ok(profile) => Ok(profile.map(str::to_string)),
            Err(CcrError::PlatformNotFound(_)) => Ok(None),
            Err(err) => Err(err),
        }
    }

    // ==================== 路径辅助方法 ====================

    /// 获取 Codex auth.json 路径
    fn auth_json_path(&self) -> PathBuf {
        self.codex_dir.join("auth.json")
    }

    /// 获取 CCR auth 存储目录
    fn auth_storage_dir(&self) -> PathBuf {
        self.ccr_codex_dir.join("auth")
    }

    /// 获取 auth_registry.toml 路径
    fn registry_path(&self) -> PathBuf {
        self.ccr_codex_dir.join("auth_registry.toml")
    }

    /// 获取备份目录
    fn backup_dir(&self) -> PathBuf {
        self.auth_storage_dir().join("backups")
    }

    /// 获取指定账号的 auth 文件路径
    fn account_auth_path(&self, name: &str) -> PathBuf {
        self.auth_storage_dir().join(format!("{}.json", name))
    }

    /// 创建使用当前 service 本地路径的 CodexConfigManager
    fn codex_config_manager(&self) -> Result<CodexConfigManager> {
        fs::create_dir_all(&self.lock_dir)?;

        Ok(CodexConfigManager::new(
            self.codex_dir.join("config.toml"),
            self.auth_json_path(),
            self.codex_dir.join("backups"),
            LockManager::new(self.lock_dir.clone()),
        ))
    }

    /// 读取 auth.json 原始 JSON Map
    fn load_auth_raw_map(
        &self,
        path: &PathBuf,
    ) -> Result<serde_json::Map<String, serde_json::Value>> {
        let content = fs::read_to_string(path)
            .map_err(|e| CcrError::ConfigError(format!("读取 auth.json 失败: {}", e)))?;

        serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(&content)
            .map_err(|e| CcrError::ConfigError(format!("解析 auth.json 失败: {}", e)))
    }

    fn load_current_auth_documents(&self) -> Result<CurrentAuthDocuments> {
        let auth_path = self.auth_json_path();
        if !auth_path.exists() {
            return Err(CcrError::ConfigError(
                "未登录 Codex，请先运行 `codex login`".into(),
            ));
        }

        let raw = self.load_auth_raw_map(&auth_path)?;
        let auth: CodexAuthJson = serde_json::from_value(serde_json::Value::Object(raw.clone()))
            .map_err(|e| CcrError::ConfigError(format!("解析 auth.json 失败: {}", e)))?;

        Ok(CurrentAuthDocuments { raw, auth })
    }

    /// 检测 Codex 凭据存储模式
    fn detect_credential_store(&self) -> CredentialStoreKind {
        let config_path = self.codex_dir.join("config.toml");
        if !config_path.exists() {
            return CredentialStoreKind::Auto;
        }

        let content = match fs::read_to_string(&config_path) {
            Ok(content) => content,
            Err(_) => return CredentialStoreKind::Auto,
        };

        let parsed: toml::Value = match toml::from_str(&content) {
            Ok(value) => value,
            Err(_) => return CredentialStoreKind::Auto,
        };

        let store = parsed
            .as_table()
            .and_then(|table| table.get("cli_auth_credentials_store"))
            .and_then(|value| value.as_str());

        CredentialStoreKind::from_config_value(store)
    }

    fn supports_managed_auth_accounts(store: CredentialStoreKind) -> bool {
        matches!(store, CredentialStoreKind::File)
    }

    fn unsupported_store_error(&self, operation: &str, store: CredentialStoreKind) -> CcrError {
        CcrError::ConfigError(format!(
            "当前 Codex 凭据存储为 {}，CCR 暂不支持{}；请使用 `codex login` / `codex logout`，或将 cli_auth_credentials_store 切换为 file",
            store.as_str(),
            operation
        ))
    }

    fn ensure_managed_auth_supported(&self, operation: &str) -> Result<CredentialStoreKind> {
        let store = self.detect_credential_store();
        if Self::supports_managed_auth_accounts(store) {
            Ok(store)
        } else {
            Err(self.unsupported_store_error(operation, store))
        }
    }

    fn find_provider_api_key(
        raw: &serde_json::Map<String, serde_json::Value>,
    ) -> Option<(&str, &str)> {
        raw.iter().find_map(|(key, value)| {
            if key == "OPENAI_API_KEY" || !key.ends_with("_API_KEY") {
                return None;
            }

            value
                .as_str()
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .map(|v| (key.as_str(), v))
        })
    }

    fn build_auth_state_from_raw(
        store: CredentialStoreKind,
        raw: &serde_json::Map<String, serde_json::Value>,
    ) -> AuthState {
        let (intent, status, reason) = Self::infer_auth_intent(raw);
        AuthState {
            intent,
            store,
            status,
            reason,
        }
    }

    fn infer_auth_intent(
        raw: &serde_json::Map<String, serde_json::Value>,
    ) -> (AuthIntent, AuthStateStatus, String) {
        if raw.is_empty() {
            return (
                AuthIntent::NoAuth,
                AuthStateStatus::Missing,
                "auth.json 为空".to_string(),
            );
        }

        let tokens = raw.get("tokens").and_then(|v| v.as_object());
        let has_tokens = tokens.is_some_and(|t| {
            t.get("id_token")
                .and_then(|v| v.as_str())
                .is_some_and(|s| !s.trim().is_empty())
                || t.get("access_token")
                    .and_then(|v| v.as_str())
                    .is_some_and(|s| !s.trim().is_empty())
                || t.get("refresh_token")
                    .and_then(|v| v.as_str())
                    .is_some_and(|s| !s.trim().is_empty())
        });

        if has_tokens {
            return (
                AuthIntent::OpenAiAuth {
                    method: OpenAiAuthMethod::Chatgpt,
                },
                AuthStateStatus::Valid,
                "检测到 OpenAI ChatGPT 会话令牌".to_string(),
            );
        }

        if raw
            .get("OPENAI_API_KEY")
            .and_then(|v| v.as_str())
            .is_some_and(|s| !s.trim().is_empty())
        {
            return (
                AuthIntent::OpenAiAuth {
                    method: OpenAiAuthMethod::Api,
                },
                AuthStateStatus::Valid,
                "检测到 OPENAI_API_KEY".to_string(),
            );
        }

        if let Some((env_key, _)) = Self::find_provider_api_key(raw) {
            return (
                AuthIntent::ProviderEnvKey {
                    env_key: env_key.to_string(),
                },
                AuthStateStatus::Valid,
                format!("检测到 provider API key: {env_key}"),
            );
        }

        (
            AuthIntent::NoAuth,
            AuthStateStatus::Invalid,
            "auth.json 存在但缺少可识别凭据".to_string(),
        )
    }

    fn extract_jwt_claim(auth: &CodexAuthJson, claim: &str) -> Option<String> {
        let id_token = auth.tokens.as_ref()?.id_token.as_ref()?;
        let parts: Vec<&str> = id_token.split('.').collect();
        if parts.len() != 3 {
            return None;
        }

        let payload = parts[1];
        let decoded = crate::utils::decode_base64url(payload)?;
        let payload_str = String::from_utf8(decoded).ok()?;
        let payload_json: serde_json::Value = serde_json::from_str(&payload_str).ok()?;

        payload_json
            .get(claim)
            .and_then(|value| value.as_str())
            .map(str::to_string)
    }

    fn decode_jwt_claims(token: &str) -> Option<serde_json::Value> {
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return None;
        }

        let decoded = crate::utils::decode_base64url(parts[1])?;
        let payload = String::from_utf8(decoded).ok()?;
        serde_json::from_str(&payload).ok()
    }

    fn claim_string(claims: Option<&serde_json::Value>, claim: &str) -> Option<String> {
        claims
            .and_then(|payload| payload.get(claim))
            .and_then(|value| value.as_str())
            .map(str::to_string)
    }

    fn extract_plan_type_from_auth(&self, auth: &CodexAuthJson) -> Option<String> {
        let tokens = auth.tokens.as_ref()?;
        let access_claims = tokens
            .access_token
            .as_deref()
            .and_then(Self::decode_jwt_claims);
        let id_claims = tokens.id_token.as_deref().and_then(Self::decode_jwt_claims);

        Self::claim_string(access_claims.as_ref(), "chatgpt_plan_type")
            .or_else(|| Self::claim_string(access_claims.as_ref(), "plan"))
            .or_else(|| Self::claim_string(id_claims.as_ref(), "chatgpt_plan_type"))
            .or_else(|| Self::claim_string(id_claims.as_ref(), "plan"))
            .map(|value| normalize_openai_plan(&value))
            .filter(|value| !value.is_empty())
    }

    fn auth_json_to_raw_map(
        auth: &CodexAuthJson,
    ) -> Result<serde_json::Map<String, serde_json::Value>> {
        serde_json::to_value(auth)
            .map_err(|e| CcrError::ConfigError(format!("序列化 auth.json 失败: {}", e)))?
            .as_object()
            .cloned()
            .ok_or_else(|| CcrError::ConfigError("auth.json 必须为对象".into()))
    }

    fn resolve_account_id_from_auth(
        &self,
        auth: &CodexAuthJson,
        raw: &serde_json::Map<String, serde_json::Value>,
    ) -> String {
        if let Some(account_id) = auth
            .tokens
            .as_ref()
            .and_then(|tokens| tokens.account_id.as_ref())
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
        {
            return account_id.to_string();
        }

        if let Some(openai_key) = raw
            .get("OPENAI_API_KEY")
            .and_then(|value| value.as_str())
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            return format!("api:{}", Self::key_fingerprint(openai_key));
        }

        if let Some((env_key, provider_key)) = Self::find_provider_api_key(raw) {
            return format!("provider:{env_key}:{}", Self::key_fingerprint(provider_key));
        }

        if let Some(subject) = Self::extract_jwt_claim(auth, "sub")
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
        {
            return format!("oauth:{subject}");
        }

        if let Some(access_token) = auth
            .tokens
            .as_ref()
            .and_then(|tokens| tokens.access_token.as_ref())
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
        {
            return format!("oauth-token:{}", Self::key_fingerprint(access_token));
        }

        "unknown".to_string()
    }

    fn key_fingerprint(value: &str) -> String {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return "empty".to_string();
        }

        let len = trimmed.len();
        if len <= 6 {
            return format!("len{len}");
        }

        let prefix = &trimmed[..3];
        let suffix = &trimmed[len - 3..];
        format!("{prefix}..{suffix}:len{len}")
    }

    fn runtime_service(&self) -> Result<CodexRuntimeService> {
        Ok(CodexRuntimeService::from_parts(
            self.platform_paths(),
            self.codex_dir.clone(),
            self.codex_config_manager()?,
        ))
    }

    /// 获取当前认证状态快照
    pub fn get_auth_state(&self) -> AuthState {
        let store = self.detect_credential_store();
        if !Self::supports_managed_auth_accounts(store) {
            return AuthState {
                intent: AuthIntent::NoAuth,
                store,
                status: AuthStateStatus::Unsupported,
                reason: format!(
                    "当前凭据存储为 {}，CCR 不读取系统钥匙串/自动存储中的实际凭据",
                    store.as_str()
                ),
            };
        }

        let auth_path = self.auth_json_path();

        if !auth_path.exists() {
            return AuthState {
                intent: AuthIntent::NoAuth,
                store,
                status: AuthStateStatus::Missing,
                reason: format!("{} 模式下未找到 auth.json", store.as_str()),
            };
        }

        let raw = match self.load_auth_raw_map(&auth_path) {
            Ok(raw) => raw,
            Err(e) => {
                return AuthState {
                    intent: AuthIntent::NoAuth,
                    store,
                    status: AuthStateStatus::Invalid,
                    reason: format!("auth.json 无法解析: {e}"),
                };
            }
        };

        Self::build_auth_state_from_raw(store, &raw)
    }

    pub fn read_auth_snapshot(&self) -> Result<AuthReadSnapshot> {
        let auth_state = self.get_auth_state();
        let registry = self.load_registry()?;

        let current_info = if auth_state.status == AuthStateStatus::Valid {
            let docs = self.load_current_auth_documents()?;
            Some(self.build_current_auth_info_from_documents(&auth_state, &docs)?)
        } else {
            None
        };

        let current_account_name =
            Self::matched_saved_account_name(&registry, current_info.as_ref());
        let login_state = Self::compute_login_state(
            &auth_state,
            current_info.as_ref(),
            current_account_name.as_deref(),
        );

        Ok(AuthReadSnapshot {
            auth_state,
            login_state,
            current_info,
            registry,
            current_account_name,
        })
    }

    pub fn get_runtime_summary(&self) -> Result<CodexRuntimeSummary> {
        let snapshot = self.read_auth_snapshot()?;
        let platform = self.platform()?;
        let current_profile_name = platform.get_current_profile()?;

        let mut current_profile_provider = None;
        let mut current_profile_auth_mode = None;
        let mut current_profile_auth_source = None;

        if let Some(profile_name) = current_profile_name.as_ref() {
            let profiles = platform.load_profiles()?;
            if let Some(profile) = profiles.get(profile_name) {
                current_profile_provider = profile.provider.clone();
                let auth_mode = CodexPlatform::profile_auth_mode(profile);
                current_profile_auth_mode = Some(auth_mode);
                current_profile_auth_source = Some(CodexPlatform::profile_auth_source(profile));
            }
        }

        let mode = match current_profile_auth_mode {
            Some(auth_mode) if auth_mode.uses_openai_auth() => {
                if matches!(snapshot.auth_state.status, AuthStateStatus::Valid) {
                    CodexRuntimeMode::ProfileWithAuth
                } else {
                    CodexRuntimeMode::ProfilePendingAuth
                }
            }
            Some(_) => CodexRuntimeMode::ProfileOnly,
            None if matches!(snapshot.auth_state.status, AuthStateStatus::Valid) => {
                CodexRuntimeMode::RuntimeOnly
            }
            None => CodexRuntimeMode::Unresolved,
        };

        let (current_auth_name, login_state) = match mode {
            CodexRuntimeMode::ProfileWithAuth | CodexRuntimeMode::RuntimeOnly => {
                (snapshot.current_account_name, snapshot.login_state)
            }
            CodexRuntimeMode::ProfileOnly => match current_profile_auth_mode {
                Some(crate::models::CodexProfileAuthMode::ProviderEnvKey) => {
                    let env_key = current_profile_auth_source
                        .as_deref()
                        .and_then(|source| source.strip_prefix("provider:"))
                        .map(str::to_string);
                    (
                        None,
                        env_key
                            .map(|env_key| LoginState::ProviderKeyActive { env_key })
                            .unwrap_or(LoginState::NotLoggedIn),
                    )
                }
                _ => (None, LoginState::NotLoggedIn),
            },
            CodexRuntimeMode::ProfilePendingAuth | CodexRuntimeMode::Unresolved => {
                (None, LoginState::NotLoggedIn)
            }
        };

        Ok(CodexRuntimeSummary {
            mode,
            current_profile_name,
            current_profile_provider,
            current_profile_auth_mode,
            current_profile_auth_source,
            current_auth_name,
            login_state,
            auth_state: snapshot.auth_state,
        })
    }

    fn build_current_auth_info_from_documents(
        &self,
        auth_state: &AuthState,
        docs: &CurrentAuthDocuments,
    ) -> Result<CurrentAuthInfo> {
        if auth_state.status != AuthStateStatus::Valid {
            return Err(CcrError::ConfigError(format!(
                "未检测到有效登录状态: {}",
                auth_state.reason
            )));
        }

        let account_id = self.resolve_account_id_from_auth(&docs.auth, &docs.raw);

        let email = self.extract_email_from_jwt(&docs.auth);
        let plan_type = self.extract_plan_type_from_auth(&docs.auth);

        let last_refresh = docs
            .auth
            .last_refresh
            .as_ref()
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        let auth_method = match &auth_state.intent {
            AuthIntent::OpenAiAuth { method } => Some(*method),
            AuthIntent::ProviderEnvKey { .. }
            | AuthIntent::ProviderBearerToken
            | AuthIntent::NoAuth => None,
        };

        Ok(CurrentAuthInfo {
            account_id,
            auth_method,
            email,
            plan_type,
            last_refresh,
        })
    }

    fn matched_saved_account_name(
        registry: &CodexAuthRegistry,
        current_info: Option<&CurrentAuthInfo>,
    ) -> Option<String> {
        let info = current_info?;
        registry.accounts.iter().find_map(|(name, account)| {
            (account.account_id == info.account_id).then(|| name.clone())
        })
    }

    fn compute_login_state(
        auth_state: &AuthState,
        current_info: Option<&CurrentAuthInfo>,
        matched_account_name: Option<&str>,
    ) -> LoginState {
        if auth_state.status != AuthStateStatus::Valid {
            return LoginState::NotLoggedIn;
        }

        match &auth_state.intent {
            AuthIntent::OpenAiAuth {
                method: OpenAiAuthMethod::Api,
            } => LoginState::ApiKeyActive,
            AuthIntent::ProviderEnvKey { env_key } => LoginState::ProviderKeyActive {
                env_key: env_key.clone(),
            },
            AuthIntent::ProviderBearerToken => LoginState::ProviderKeyActive {
                env_key: "experimental_bearer_token".to_string(),
            },
            AuthIntent::NoAuth => LoginState::NotLoggedIn,
            AuthIntent::OpenAiAuth {
                method: OpenAiAuthMethod::Chatgpt,
            } => {
                if current_info.is_none() {
                    return LoginState::NotLoggedIn;
                }

                matched_account_name
                    .map(|name| LoginState::LoggedInSaved(name.to_string()))
                    .unwrap_or(LoginState::LoggedInUnsaved)
            }
        }
    }

    /// 根据当前 runtime auth 对账 current_auth 指针
    pub fn sync_current_auth_registry(&self) -> Result<Option<String>> {
        let mut registry = self.load_registry()?;
        let state = self.get_auth_state();
        let new_current = match state.intent {
            AuthIntent::OpenAiAuth { .. } if matches!(state.status, AuthStateStatus::Valid) => {
                let info = self.get_current_auth_info()?;
                registry.accounts.iter().find_map(|(name, account)| {
                    (account.account_id == info.account_id).then(|| name.clone())
                })
            }
            _ => None,
        };

        if registry.current_auth != new_current {
            registry.current_auth = new_current.clone();
            if let Some(name) = &new_current
                && let Some(account) = registry.accounts.get(name)
            {
                registry.record_usage_activation(
                    name.clone(),
                    account.account_id.clone(),
                    Utc::now(),
                );
            }
            self.save_registry(&registry)?;
        }

        Ok(new_current)
    }

    // ==================== 登录状态检测 ====================

    /// 检查用户是否已登录 Codex
    pub fn is_logged_in(&self) -> bool {
        matches!(self.get_auth_state().status, AuthStateStatus::Valid)
    }

    /// 获取当前登录状态
    pub fn get_login_state(&self) -> Result<LoginState> {
        Ok(self.read_auth_snapshot()?.login_state)
    }

    /// 获取当前 auth.json 的解析信息
    pub fn get_current_auth_info(&self) -> Result<CurrentAuthInfo> {
        self.read_auth_snapshot()?
            .current_info
            .ok_or_else(|| CcrError::ConfigError("未检测到有效登录状态".into()))
    }

    fn ensure_current_runtime_supports_openai_switch(&self) -> Result<()> {
        let Some(current_profile) = self.current_profile_name()? else {
            return Ok(());
        };
        let platform = self.platform()?;
        let profiles = platform.load_profiles()?;
        let Some(profile) = profiles.get(&current_profile) else {
            return Ok(());
        };

        let auth_mode = CodexPlatform::profile_auth_mode(profile);

        if auth_mode.uses_openai_auth() {
            return Ok(());
        }

        // Profile 配置层未识别为 OpenAI 认证（可能缺少 auth_mode 元数据）；
        // 回退检查运行时 auth.json 的实际凭据状态
        let auth_state = self.get_auth_state();
        if matches!(auth_state.intent, AuthIntent::OpenAiAuth { .. }) {
            debug!(
                "Profile '{}' 配置层 auth_mode={:?} 未标记 OpenAI，但运行时 auth.json 包含 OpenAI 凭据，允许切换",
                current_profile, auth_mode
            );
            return Ok(());
        }

        Err(CcrError::ValidationError(
            "当前 Profile 不使用 OpenAI 认证；请改用 Profile 切换来切换 URL + Key / Login 模式"
                .into(),
        ))
    }

    fn sync_current_profile_openai_mode(
        &self,
        auth_method: OpenAiAuthMethod,
        auth_data: &serde_json::Map<String, serde_json::Value>,
    ) -> Result<()> {
        let Some(current_profile) = self.current_profile_name()? else {
            return Ok(());
        };

        let platform = self.platform()?;
        let profiles = platform.load_profiles()?;
        let Some(profile) = profiles.get(&current_profile) else {
            return Ok(());
        };

        let current_auth_mode = CodexPlatform::profile_auth_mode(profile);

        // 非 OpenAI 认证的 Profile 不需要同步元数据
        if !current_auth_mode.uses_openai_auth() {
            return Ok(());
        }

        let mut updated = profile.clone();
        let auth_mode = match auth_method {
            OpenAiAuthMethod::Chatgpt => "openai_chatgpt",
            OpenAiAuthMethod::Api => "openai_api_key",
        };
        updated.platform_data.insert(
            "auth_mode".to_string(),
            serde_json::Value::String(auth_mode.to_string()),
        );
        updated.platform_data.insert(
            "openai_login_method".to_string(),
            serde_json::Value::String(match auth_method {
                OpenAiAuthMethod::Chatgpt => "chatgpt".to_string(),
                OpenAiAuthMethod::Api => "api".to_string(),
            }),
        );
        updated.platform_data.insert(
            "forced_login_method".to_string(),
            serde_json::Value::String(match auth_method {
                OpenAiAuthMethod::Chatgpt => "chatgpt".to_string(),
                OpenAiAuthMethod::Api => "api".to_string(),
            }),
        );

        // 仅官方 Profile 同步 auth_token（即 OpenAI API Key）。
        // 第三方 Profile 的 auth_token 是 Provider 自身的密钥，
        // 与 OpenAI 认证无关，切换 Auth 账号时不应被覆盖或清除。
        if CodexPlatform::is_official_profile(profile) {
            if matches!(auth_method, OpenAiAuthMethod::Api) {
                updated.auth_token = auth_data
                    .get("OPENAI_API_KEY")
                    .and_then(serde_json::Value::as_str)
                    .map(ccr_core::Secret::from);
            } else {
                updated.auth_token = None;
            }
        }

        platform.save_profile(&current_profile, &updated)
    }

    // ==================== 账号管理操作 ====================

    /// 保存当前登录到指定名称
    pub fn save_current(&self, name: &str, description: Option<String>, force: bool) -> Result<()> {
        self.ensure_managed_auth_supported("保存账号")?;
        let auth_state = self.get_auth_state();

        // 检查是否已登录
        if auth_state.status != AuthStateStatus::Valid {
            return Err(CcrError::ConfigError(
                "未登录 Codex，请先运行 `codex login`".into(),
            ));
        }

        let auth_method = match auth_state.intent {
            AuthIntent::OpenAiAuth { method } => method,
            AuthIntent::ProviderEnvKey { .. }
            | AuthIntent::ProviderBearerToken
            | AuthIntent::NoAuth => {
                return Err(CcrError::ValidationError(
                    "当前 runtime 不是 OpenAI 登录态，不能保存为 Codex Auth 账号".into(),
                ));
            }
        };

        // 验证名称
        self.validate_account_name(name)?;

        // 检查是否已存在
        let mut registry = self.load_registry()?;
        if registry.accounts.contains_key(name) && !force {
            return Err(CcrError::ConfigError(format!(
                "账号 '{}' 已存在，使用 --force 覆盖",
                name
            )));
        }

        // 确保目录存在
        let auth_storage = self.auth_storage_dir();
        fs::create_dir_all(&auth_storage)
            .map_err(|e| CcrError::ConfigError(format!("创建存储目录失败: {}", e)))?;

        // 复制 auth.json
        let src = self.auth_json_path();
        let dst = self.account_auth_path(name);
        fs::copy(&src, &dst)
            .map_err(|e| CcrError::ConfigError(format!("复制 auth.json 失败: {}", e)))?;

        // 设置文件权限（仅当前用户可读写）
        crate::utils::ensure_private_permissions(&dst);

        // 获取当前账号信息
        let current_info = self.get_current_auth_info()?;

        // 更新注册表
        let account = CodexAuthAccount {
            description,
            account_id: current_info.account_id,
            auth_method: Some(auth_method),
            api_base_url: None,
            api_provider_name: None,
            email: current_info.email.map(|e| self.mask_email(&e)),
            plan_type: current_info.plan_type,
            saved_at: Utc::now(),
            last_used: Some(Utc::now()),
            last_refresh: current_info.last_refresh,
            expires_at: None,
        };

        registry.accounts.insert(name.to_string(), account);
        registry.current_auth = Some(name.to_string());
        if let Some(saved) = registry.accounts.get(name) {
            registry.record_usage_activation(
                name.to_string(),
                saved.account_id.clone(),
                Utc::now(),
            );
        }
        self.save_registry(&registry)?;

        debug!("已保存账号: {}", name);
        Ok(())
    }

    /// 列出所有账号
    pub fn list_accounts(&self) -> Result<Vec<CodexAuthItem>> {
        let snapshot = self.read_auth_snapshot()?;
        self.build_account_items(&snapshot)
    }

    pub fn build_export_account_from_auth_json(
        &self,
        auth: CodexAuthJson,
        description: Option<String>,
        expires_at: Option<DateTime<Utc>>,
        api_base_url: Option<String>,
        api_provider_name: Option<String>,
    ) -> Result<CodexAuthExportAccount> {
        let raw = Self::auth_json_to_raw_map(&auth)?;
        let auth_state = Self::build_auth_state_from_raw(CredentialStoreKind::File, &raw);

        let auth_method = match auth_state.intent {
            AuthIntent::OpenAiAuth { method } => Some(method),
            AuthIntent::ProviderEnvKey { .. }
            | AuthIntent::ProviderBearerToken
            | AuthIntent::NoAuth => None,
        };

        let account_id = self.resolve_account_id_from_auth(&auth, &raw);
        let email = self
            .extract_email_from_jwt(&auth)
            .map(|value| self.mask_email(&value));
        let plan_type = self.extract_plan_type_from_auth(&auth);
        let last_refresh = auth
            .last_refresh
            .as_deref()
            .and_then(|value| DateTime::parse_from_rfc3339(value).ok())
            .map(|value| value.with_timezone(&Utc));

        Ok(CodexAuthExportAccount {
            description,
            account_id,
            auth_method,
            api_base_url: api_base_url
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty()),
            api_provider_name: api_provider_name
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty()),
            email,
            plan_type,
            saved_at: Utc::now(),
            last_used: Some(Utc::now()),
            last_refresh,
            expires_at,
            auth_data: Some(auth),
        })
    }

    pub fn load_current_auth_json(&self) -> Result<CodexAuthJson> {
        Ok(self.load_current_auth_documents()?.auth)
    }

    pub fn suggest_account_name(&self, hint: Option<&str>) -> Result<String> {
        let registry = self.load_registry()?;
        let mut base = hint
            .map(Self::sanitize_account_name_hint)
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "account".to_string());

        if base == "default" {
            base = "account".to_string();
        }

        if !registry.accounts.contains_key(&base) {
            self.validate_account_name(&base)?;
            return Ok(base);
        }

        for index in 2..=999 {
            let suffix = format!("-{index}");
            let max_base_len = 32usize.saturating_sub(suffix.len());
            let trimmed = if base.len() > max_base_len {
                base[..max_base_len]
                    .trim_end_matches(['-', '_'])
                    .to_string()
            } else {
                base.clone()
            };
            let candidate = format!("{}{}", trimmed, suffix);
            if !registry.accounts.contains_key(&candidate) {
                self.validate_account_name(&candidate)?;
                return Ok(candidate);
            }
        }

        Err(CcrError::ValidationError(
            "无法为账号生成唯一名称，请手动指定".into(),
        ))
    }

    pub fn reserve_explicit_account_name(&self, name: &str) -> Result<String> {
        let explicit = name.trim();
        self.validate_account_name(explicit)?;

        let registry = self.load_registry()?;
        if registry.accounts.contains_key(explicit) {
            return Err(CcrError::ValidationError(format!(
                "账号 '{}' 已存在，请更换名称",
                explicit
            )));
        }

        Ok(explicit.to_string())
    }

    pub fn build_account_items(&self, snapshot: &AuthReadSnapshot) -> Result<Vec<CodexAuthItem>> {
        let mut items = Vec::new();

        // 如果已登录但未保存，添加虚拟 "default" 项
        if let LoginState::LoggedInUnsaved = snapshot.login_state
            && let Some(info) = &snapshot.current_info
        {
            items.push(CodexAuthItem {
                name: "default".to_string(),
                description: Some("(未保存的当前登录)".to_string()),
                email: info.email.clone().map(|e| self.mask_email(&e)),
                plan_type: info.plan_type.clone(),
                is_current: true,
                is_virtual: true,
                saved_at: None,
                last_used: None,
                last_refresh: info.last_refresh,
            });
        }

        // 添加所有已保存的账号
        for (name, account) in &snapshot.registry.accounts {
            if account.auth_method.is_none() && account.account_id.starts_with("provider:") {
                continue;
            }

            let is_current = match &snapshot.login_state {
                LoginState::LoggedInSaved(current_name) => current_name == name,
                _ => false,
            };

            items.push(CodexAuthItem {
                name: name.clone(),
                description: account.description.clone(),
                email: account.email.clone(),
                plan_type: account
                    .plan_type
                    .clone()
                    .or_else(|| self.load_saved_account_plan_type(name)),
                is_current,
                is_virtual: false,
                saved_at: Some(account.saved_at),
                last_used: account.last_used,
                last_refresh: account.last_refresh,
            });
        }

        Ok(items)
    }

    /// 切换到指定账号
    pub fn switch_account(&self, name: &str) -> Result<()> {
        self.ensure_managed_auth_supported("切换账号")?;
        self.ensure_current_runtime_supports_openai_switch()?;

        // 检查账号是否存在
        let registry = self.load_registry()?;
        if !registry.accounts.contains_key(name) {
            let available: Vec<_> = registry.accounts.keys().collect();
            return Err(CcrError::ConfigError(format!(
                "账号 '{}' 不存在。可用账号: {:?}",
                name, available
            )));
        }

        let account = registry
            .accounts
            .get(name)
            .cloned()
            .ok_or_else(|| CcrError::ConfigError(format!("账号 '{}' 不存在", name)))?;

        let src = self.account_auth_path(name);
        let incoming = self.load_auth_raw_map(&src)?;
        let (target_intent, _, _) = Self::infer_auth_intent(&incoming);
        let auth_method = match target_intent {
            AuthIntent::OpenAiAuth { method } => method,
            AuthIntent::ProviderEnvKey { .. }
            | AuthIntent::ProviderBearerToken
            | AuthIntent::NoAuth => {
                return Err(CcrError::ValidationError(
                    "Codex Auth 账号只支持 OpenAI 登录态".into(),
                ));
            }
        };
        let normalized = normalize_auth_map_for_intent(&target_intent, &incoming);
        let mut config = self.codex_config_manager()?.load_config()?;
        if !matches!(config, toml::Value::Table(_)) {
            config = toml::Value::Table(toml::map::Map::new());
        }
        let root = config
            .as_table_mut()
            .ok_or_else(|| CcrError::ConfigError("Codex config.toml 应为 table".into()))?;
        root.insert(
            "forced_login_method".into(),
            toml::Value::String(match auth_method {
                OpenAiAuthMethod::Chatgpt => "chatgpt".to_string(),
                OpenAiAuthMethod::Api => "api".to_string(),
            }),
        );
        Self::apply_account_route_config(root, &account, auth_method);

        let runtime_service = self.runtime_service()?;
        runtime_service.commit_plan(CodexRuntimeCommitPlan {
            config: Some(config),
            auth_cache: if normalized.is_empty() {
                CodexAuthCacheAction::Delete
            } else {
                CodexAuthCacheAction::Write(normalized.clone())
            },
        })?;
        self.sync_current_profile_openai_mode(auth_method, &normalized)?;

        // 更新注册表
        let mut registry = self.load_registry()?;
        registry.current_auth = Some(name.to_string());
        if let Some(account) = registry.accounts.get_mut(name) {
            account.last_used = Some(Utc::now());
        }
        if let Some(account) = registry.accounts.get(name) {
            registry.record_usage_activation(
                name.to_string(),
                account.account_id.clone(),
                Utc::now(),
            );
        }
        self.save_registry(&registry)?;
        let _ = self.sync_current_auth_registry();

        debug!("已切换到账号: {}", name);
        Ok(())
    }

    /// 删除指定账号
    pub fn delete_account(&self, name: &str) -> Result<()> {
        self.ensure_managed_auth_supported("删除账号")?;

        let mut registry = self.load_registry()?;

        // 检查账号是否存在
        if !registry.accounts.contains_key(name) {
            return Err(CcrError::ConfigError(format!("账号 '{}' 不存在", name)));
        }

        // 删除 auth 文件
        let auth_path = self.account_auth_path(name);
        if auth_path.exists() {
            fs::remove_file(&auth_path)
                .map_err(|e| CcrError::ConfigError(format!("删除 auth 文件失败: {}", e)))?;
        }

        // 从注册表移除
        registry.accounts.shift_remove(name);

        // 如果删除的是当前账号，清除 current_auth
        if registry.current_auth.as_deref() == Some(name) {
            registry.current_auth = None;
        }

        self.save_registry(&registry)?;

        debug!("已删除账号: {}", name);
        Ok(())
    }

    // ==================== 备份管理 ====================

    /// 备份当前 auth.json
    #[allow(dead_code)]
    pub fn backup_current_auth(&self) -> Result<PathBuf> {
        let auth_path = self.auth_json_path();
        if !auth_path.exists() {
            return Err(CcrError::ConfigError("没有可备份的 auth.json".into()));
        }

        // 确保备份目录存在
        let backup_dir = self.backup_dir();
        fs::create_dir_all(&backup_dir)
            .map_err(|e| CcrError::ConfigError(format!("创建备份目录失败: {}", e)))?;

        // 生成带时间戳的备份文件名
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let backup_name = format!("auth_{}.json", timestamp);
        let backup_path = backup_dir.join(&backup_name);

        // 复制文件
        fs::copy(&auth_path, &backup_path)
            .map_err(|e| CcrError::ConfigError(format!("备份失败: {}", e)))?;

        // 清理旧备份
        self.cleanup_old_backups()?;

        debug!("已备份到: {}", backup_path.display());
        Ok(backup_path)
    }

    fn backup_registry(&self) -> Result<Option<PathBuf>> {
        let registry_path = self.registry_path();
        if !registry_path.exists() {
            return Ok(None);
        }

        let backup_dir = self.backup_dir();
        fs::create_dir_all(&backup_dir)
            .map_err(|e| CcrError::ConfigError(format!("创建备份目录失败: {}", e)))?;

        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let backup_name = format!("auth_registry_{}.toml", timestamp);
        let backup_path = backup_dir.join(&backup_name);

        fs::copy(&registry_path, &backup_path)
            .map_err(|e| CcrError::ConfigError(format!("备份注册表失败: {}", e)))?;

        Ok(Some(backup_path))
    }

    fn backup_account_auth(&self, name: &str) -> Result<Option<PathBuf>> {
        let auth_path = self.account_auth_path(name);
        if !auth_path.exists() {
            return Ok(None);
        }

        let backup_dir = self.backup_dir();
        fs::create_dir_all(&backup_dir)
            .map_err(|e| CcrError::ConfigError(format!("创建备份目录失败: {}", e)))?;

        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let backup_name = format!("auth_account_{}_{}.json", name, timestamp);
        let backup_path = backup_dir.join(&backup_name);

        fs::copy(&auth_path, &backup_path)
            .map_err(|e| CcrError::ConfigError(format!("备份 auth 文件失败: {}", e)))?;

        Ok(Some(backup_path))
    }

    /// 清理旧备份，保留最新的 MAX_BACKUPS 个
    #[allow(dead_code)]
    fn cleanup_old_backups(&self) -> Result<()> {
        let backup_dir = self.backup_dir();
        if !backup_dir.exists() {
            return Ok(());
        }

        let mut backups: Vec<_> = fs::read_dir(&backup_dir)
            .map_err(|e| CcrError::ConfigError(format!("读取备份目录失败: {}", e)))?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with("auth_") && n.ends_with(".json"))
            })
            .collect();

        // 按修改时间排序 (最新的在前)
        backups.sort_by(|a, b| {
            let time_a = a.metadata().and_then(|m| m.modified()).ok();
            let time_b = b.metadata().and_then(|m| m.modified()).ok();
            time_b.cmp(&time_a)
        });

        // 删除超出限制的旧备份
        for backup in backups.iter().skip(MAX_BACKUPS) {
            if let Err(e) = fs::remove_file(backup.path()) {
                warn!("删除旧备份失败: {}", e);
            }
        }

        Ok(())
    }

    // ==================== 进程检测 ====================

    /// 进程检测缓存（5 秒节流，避免 TUI 场景下频繁扫描进程表）
    fn cached_codex_processes() -> Vec<u32> {
        use std::sync::{LazyLock, Mutex};
        use std::time::Instant;

        static CACHE: LazyLock<Mutex<(Instant, Vec<u32>)>> = LazyLock::new(|| {
            Mutex::new((
                Instant::now() - std::time::Duration::from_secs(10),
                Vec::new(),
            ))
        });

        const THROTTLE: std::time::Duration = std::time::Duration::from_secs(5);

        let mut cache = CACHE
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if cache.0.elapsed() < THROTTLE {
            return cache.1.clone();
        }

        // 缓存过期，重新扫描
        use sysinfo::System;
        let mut sys = System::new();
        sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

        let pids: Vec<u32> = sys
            .processes()
            .iter()
            .filter(|(_, process)| {
                let name = process.name().to_string_lossy().to_lowercase();
                name.contains("codex") && !name.contains("ccr")
            })
            .map(|(pid, _)| pid.as_u32())
            .collect();

        *cache = (Instant::now(), pids.clone());
        pids
    }

    /// 检测是否有 Codex 进程正在运行（5 秒缓存节流）
    pub fn detect_codex_process(&self) -> Vec<u32> {
        Self::cached_codex_processes()
    }

    // ==================== JWT 解析 ====================

    /// 从 JWT 中提取邮箱
    fn extract_email_from_jwt(&self, auth: &CodexAuthJson) -> Option<String> {
        Self::extract_jwt_claim(auth, "email")
    }

    // ==================== 注册表管理 ====================

    /// 加载注册表
    pub fn load_registry(&self) -> Result<CodexAuthRegistry> {
        self.registry_store().load()
    }

    /// 保存注册表
    fn save_registry(&self, registry: &CodexAuthRegistry) -> Result<()> {
        self.registry_store().save(registry)
    }

    fn registry_store(&self) -> super::codex_registry_store::CodexRegistryStore {
        super::codex_registry_store::CodexRegistryStore::new(&self.ccr_codex_dir)
    }

    fn load_saved_account_plan_type(&self, name: &str) -> Option<String> {
        let auth_path = self.account_auth_path(name);
        let raw = self.load_auth_raw_map(&auth_path).ok()?;
        let auth: CodexAuthJson = serde_json::from_value(serde_json::Value::Object(raw)).ok()?;
        self.extract_plan_type_from_auth(&auth)
    }

    pub fn update_account_description(
        &self,
        name: &str,
        description: Option<String>,
    ) -> Result<CodexAuthAccount> {
        self.ensure_managed_auth_supported("更新账号描述")?;

        let mut registry = self.load_registry()?;
        let account = registry
            .accounts
            .get_mut(name)
            .ok_or_else(|| CcrError::ResourceNotFound(format!("Codex auth account '{}'", name)))?;
        account.description = description;
        let updated = account.clone();
        self.save_registry(&registry)?;
        Ok(updated)
    }

    /// 重命名已保存的 Codex 账号
    ///
    /// 原子性地迁移 auth 文件、registry 键顺序以及 usage_ledger 归因记录，
    /// 保证 auth.json 本身不变（因为账号身份由 account_id 决定，而非名称）。
    ///
    /// # 参数
    /// * `old_name` - 当前账号名称（必须已存在）
    /// * `new_name` - 目标新名称（通过 validate_account_name 校验）
    /// * `force`    - 当 `new_name` 已被其他账号占用时是否强制覆盖
    ///
    /// # 行为
    /// - `old_name == new_name` 时视为空操作并返回原账号信息
    /// - `new_name` 冲突且 `force=false` 时返回 ConfigError
    /// - `force=true` 时对被覆盖目标执行备份 + 删除，再执行重命名
    /// - 迁移完成后同步：registry.accounts 键、current_auth 指针、usage_ledger 中的 account_name
    pub fn rename_account(
        &self,
        old_name: &str,
        new_name: &str,
        force: bool,
    ) -> Result<CodexAuthAccount> {
        self.ensure_managed_auth_supported("重命名账号")?;

        // 空操作：同名直接返回
        if old_name == new_name {
            let registry = self.load_registry()?;
            return registry.accounts.get(old_name).cloned().ok_or_else(|| {
                CcrError::ResourceNotFound(format!("Codex auth account '{}'", old_name))
            });
        }

        // 校验目标名称
        self.validate_account_name(new_name)?;

        let mut registry = self.load_registry()?;

        // 源账号必须存在
        if !registry.accounts.contains_key(old_name) {
            return Err(CcrError::ResourceNotFound(format!(
                "Codex auth account '{}'",
                old_name
            )));
        }

        // 处理目标名称冲突
        let needs_conflict_cleanup = registry.accounts.contains_key(new_name);
        if needs_conflict_cleanup {
            if !force {
                return Err(CcrError::ConfigError(format!(
                    "账号 '{}' 已存在，使用 force 覆盖或先删除",
                    new_name
                )));
            }

            // 备份即将被覆盖的目标，留下可恢复痕迹
            let _ = self.backup_account_auth(new_name);

            let conflict_auth = self.account_auth_path(new_name);
            if conflict_auth.exists() {
                fs::remove_file(&conflict_auth)
                    .map_err(|e| CcrError::ConfigError(format!("删除冲突 auth 文件失败: {}", e)))?;
            }

            registry.accounts.shift_remove(new_name);
        }

        // 备份源 auth 与 registry，再执行文件搬迁
        let _ = self.backup_account_auth(old_name);
        let _ = self.backup_registry();

        let src_path = self.account_auth_path(old_name);
        let dst_path = self.account_auth_path(new_name);
        if src_path.exists() {
            if let Some(parent) = dst_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| CcrError::ConfigError(format!("创建存储目录失败: {}", e)))?;
            }
            if let Err(rename_err) = fs::rename(&src_path, &dst_path) {
                // 跨卷或锁定场景下的兼容回退：copy + remove
                fs::copy(&src_path, &dst_path).map_err(|copy_err| {
                    CcrError::ConfigError(format!(
                        "移动 auth 文件失败 (rename: {}, copy: {})",
                        rename_err, copy_err
                    ))
                })?;
                fs::remove_file(&src_path)
                    .map_err(|e| CcrError::ConfigError(format!("清理旧 auth 文件失败: {}", e)))?;
            }
            crate::utils::ensure_private_permissions(&dst_path);
        }

        // 重建 accounts IndexMap，保持原插入顺序
        let original_accounts = std::mem::take(&mut registry.accounts);
        let mut rebuilt: indexmap::IndexMap<String, CodexAuthAccount> =
            indexmap::IndexMap::with_capacity(original_accounts.len());
        let mut renamed_account: Option<CodexAuthAccount> = None;
        for (key, value) in original_accounts {
            if key == old_name {
                renamed_account = Some(value.clone());
                rebuilt.insert(new_name.to_string(), value);
            } else {
                rebuilt.insert(key, value);
            }
        }
        registry.accounts = rebuilt;

        // 更新 current_auth 指针
        if registry.current_auth.as_deref() == Some(old_name) {
            registry.current_auth = Some(new_name.to_string());
        }

        // 同步 usage_ledger 中的 account_name，避免归因断层
        for entry in registry.usage_ledger.iter_mut() {
            if entry.account_name == old_name {
                entry.account_name = new_name.to_string();
            }
        }

        self.save_registry(&registry)?;

        let updated = renamed_account
            .ok_or_else(|| CcrError::ConfigError("rename 内部错误：未找到重命名后的账号".into()))?;

        debug!("已重命名 Codex 账号: {} -> {}", old_name, new_name);
        Ok(updated)
    }

    // ==================== 辅助方法 ====================

    /// 验证账号名称
    fn validate_account_name(&self, name: &str) -> Result<()> {
        if name.is_empty() {
            return Err(CcrError::ValidationError("账号名称不能为空".into()));
        }

        if name == "default" {
            return Err(CcrError::ValidationError(
                "'default' 是保留名称，请使用其他名称".into(),
            ));
        }

        // 只允许字母、数字、下划线、连字符
        if !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            return Err(CcrError::ValidationError(
                "账号名称只能包含字母、数字、下划线和连字符".into(),
            ));
        }

        if name.len() > 32 {
            return Err(CcrError::ValidationError(
                "账号名称不能超过 32 个字符".into(),
            ));
        }

        Ok(())
    }

    fn sanitize_account_name_hint(input: &str) -> String {
        let mut output = String::with_capacity(input.len());
        let mut previous_is_separator = false;

        for ch in input.chars() {
            let normalized = ch.to_ascii_lowercase();
            if normalized.is_ascii_alphanumeric() {
                output.push(normalized);
                previous_is_separator = false;
                continue;
            }

            if matches!(normalized, '-' | '_' | '.' | '@' | ' ') && !previous_is_separator {
                output.push('-');
                previous_is_separator = true;
            }
        }

        let trimmed = output.trim_matches('-').trim_matches('_').to_string();
        let truncated = if trimmed.len() > 32 {
            trimmed[..32].trim_end_matches(['-', '_']).to_string()
        } else {
            trimmed
        };

        if truncated.is_empty() {
            "account".to_string()
        } else {
            truncated
        }
    }

    fn apply_account_route_config(
        root: &mut toml::map::Map<String, toml::Value>,
        account: &CodexAuthAccount,
        auth_method: OpenAiAuthMethod,
    ) {
        root.remove("model_provider");
        root.remove("model_providers");

        let explicit_base_url = account
            .api_base_url
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());

        if explicit_base_url.is_none() {
            return;
        }

        root.insert(
            "model_provider".into(),
            toml::Value::String(THIRD_PARTY_RUNTIME_PROVIDER_KEY.to_string()),
        );

        let provider_name = account
            .api_provider_name
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| "custom".to_string());

        let base_url = explicit_base_url
            .unwrap_or(OPENAI_DEFAULT_BASE_URL)
            .to_string();

        let requires_openai_auth = match auth_method {
            OpenAiAuthMethod::Chatgpt => true,
            OpenAiAuthMethod::Api => account.api_base_url.is_none(),
        };

        let mut provider_table = toml::map::Map::new();
        provider_table.insert("name".into(), toml::Value::String(provider_name));
        provider_table.insert("base_url".into(), toml::Value::String(base_url));
        provider_table.insert("wire_api".into(), toml::Value::String("responses".into()));
        provider_table.insert(
            "requires_openai_auth".into(),
            toml::Value::Boolean(requires_openai_auth),
        );

        let mut providers = toml::map::Map::new();
        providers.insert(
            THIRD_PARTY_RUNTIME_PROVIDER_KEY.to_string(),
            toml::Value::Table(provider_table),
        );
        root.insert("model_providers".into(), toml::Value::Table(providers));
    }

    /// 邮箱脱敏
    pub fn mask_email(&self, email: &str) -> String {
        if let Some(at_pos) = email.find('@') {
            let local = &email[..at_pos];
            let domain = &email[at_pos..];

            if local.len() <= 3 {
                format!("{}***{}", local, domain)
            } else {
                let visible = &local[..3];
                format!("{}***{}", visible, domain)
            }
        } else {
            // 不是有效邮箱格式，直接返回
            email.to_string()
        }
    }

    // ==================== 导入/导出 ====================

    /// 导出所有账号到 JSON
    ///
    /// # 参数
    ///
    /// * `include_secrets` - 是否包含完整的 auth.json 数据（Token 等敏感信息）
    ///
    /// # 返回
    ///
    /// * `Ok(String)` - JSON 格式的导出数据
    /// * `Err(CcrError)` - 导出失败
    pub fn export_accounts(&self, include_secrets: bool) -> Result<String> {
        self.ensure_managed_auth_supported("导出账号")?;

        let registry = self.load_registry()?;

        let mut export_accounts = indexmap::IndexMap::new();

        for (name, account) in &registry.accounts {
            if account.auth_method.is_none() && account.account_id.starts_with("provider:") {
                continue;
            }

            let auth_data = if include_secrets {
                // 读取完整的 auth.json
                let auth_path = self.account_auth_path(name);
                if auth_path.exists() {
                    let content = fs::read_to_string(&auth_path)
                        .map_err(|e| CcrError::ConfigError(format!("读取账号文件失败: {}", e)))?;
                    Some(
                        serde_json::from_str(&content).map_err(|e| {
                            CcrError::ConfigError(format!("解析账号文件失败: {}", e))
                        })?,
                    )
                } else {
                    warn!("账号 {} 的 auth 文件不存在", name);
                    None
                }
            } else {
                None
            };

            export_accounts.insert(
                name.clone(),
                CodexAuthExportAccount {
                    description: account.description.clone(),
                    account_id: account.account_id.clone(),
                    auth_method: account.auth_method,
                    api_base_url: account.api_base_url.clone(),
                    api_provider_name: account.api_provider_name.clone(),
                    email: account.email.clone(),
                    plan_type: account
                        .plan_type
                        .clone()
                        .or_else(|| self.load_saved_account_plan_type(name)),
                    saved_at: account.saved_at,
                    last_used: account.last_used,
                    last_refresh: account.last_refresh,
                    expires_at: account.expires_at,
                    auth_data,
                },
            );
        }

        let export = CodexAuthExport {
            version: "1.0".to_string(),
            exported_at: Utc::now(),
            accounts: export_accounts,
        };

        serde_json::to_string_pretty(&export)
            .map_err(|e| CcrError::ConfigError(format!("序列化导出数据失败: {}", e)))
    }

    /// 导出账号（加密版本）
    ///
    /// 内部先调用 `export_accounts(true)` 获取完整明文 JSON，
    /// 再从中解析 accounts 字段以获取账号数量，最后加密 accounts 部分。
    ///
    /// # 参数
    ///
    /// * `password` - 用户设置的导出密码
    ///
    /// # 返回
    ///
    /// * `Ok(String)` - 加密信封格式的 JSON 字符串
    /// * `Err(CcrError)` - 导出失败
    pub fn export_accounts_encrypted(&self, password: &str) -> Result<String> {
        let full_json = self.export_accounts(true)?;
        let export_data: CodexAuthExport = serde_json::from_str(&full_json)
            .map_err(|e| CcrError::ConfigError(format!("解析导出数据失败: {}", e)))?;
        let account_count = export_data.accounts.len();
        let accounts_json = serde_json::to_string(&export_data.accounts)
            .map_err(|e| CcrError::ConfigError(format!("序列化账号数据失败: {}", e)))?;

        let encrypted = super::codex_auth_crypto::ExportCrypto::encrypt_export(
            &accounts_json,
            password,
            export_data.exported_at,
            account_count,
        )?;

        serde_json::to_string_pretty(&encrypted)
            .map_err(|e| CcrError::ConfigError(format!("序列化加密导出数据失败: {}", e)))
    }

    /// 检测导入数据格式
    ///
    /// 根据 JSON 内容判断是加密信封 (v2.0) 还是明文导出 (v1.0)。
    pub fn detect_import_format(content: &str) -> ImportFormat {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(content) {
            if v.get("format").and_then(|f| f.as_str()) == Some("encrypted") {
                return ImportFormat::EncryptedV2;
            }
            if v.get("accounts").is_some() {
                return ImportFormat::PlaintextV1;
            }
        }
        ImportFormat::Unknown
    }

    /// 解密并导入账号数据
    ///
    /// 从加密信封中恢复 accounts JSON，然后执行常规导入流程。
    ///
    /// # 参数
    ///
    /// * `encrypted_content` - 加密信封 JSON 字符串
    /// * `password` - 用户输入的密码
    /// * `mode` - 导入模式
    /// * `force` - 是否强制覆盖
    pub fn import_accounts_encrypted(
        &self,
        encrypted_content: &str,
        password: &str,
        mode: ImportMode,
        force: bool,
    ) -> Result<ImportResult> {
        let encrypted: CodexAuthEncryptedExport = serde_json::from_str(encrypted_content)
            .map_err(|e| CcrError::ConfigError(format!("解析加密导出数据失败: {}", e)))?;

        let accounts_json =
            super::codex_auth_crypto::ExportCrypto::decrypt_export(&encrypted, password)?;

        // 重建完整的 v1.0 明文结构以复用现有 import_accounts
        let full_export = format!(
            r#"{{"version":"1.0","exported_at":"{}","accounts":{}}}"#,
            encrypted.exported_at.to_rfc3339(),
            accounts_json
        );

        self.import_accounts(&full_export, mode, force)
    }

    /// 导入账号数据
    ///
    /// # 参数
    ///
    /// * `content` - JSON 格式的导入数据
    /// * `mode` - 导入模式 (Merge/Replace)
    /// * `force` - 是否强制覆盖（仅在 Merge 模式下有效）
    ///
    /// # 返回
    ///
    /// * `Ok(ImportResult)` - 导入结果统计
    /// * `Err(CcrError)` - 导入失败
    pub fn import_accounts(
        &self,
        content: &str,
        mode: ImportMode,
        force: bool,
    ) -> Result<ImportResult> {
        self.ensure_managed_auth_supported("导入账号")?;

        // 解析导入数据
        let import_data: CodexAuthExport = serde_json::from_str(content)
            .map_err(|e| CcrError::ConfigError(format!("解析导入数据失败: {}", e)))?;

        let mut registry = self.load_registry()?;
        let mut result = ImportResult::default();

        // 确保存储目录存在
        let auth_storage = self.auth_storage_dir();
        fs::create_dir_all(&auth_storage)
            .map_err(|e| CcrError::ConfigError(format!("创建存储目录失败: {}", e)))?;

        let mut registry_backed_up = false;

        for (name, import_account) in import_data.accounts {
            // 验证账号名称
            self.validate_account_name(&name)?;

            if import_account.auth_method.is_none()
                && import_account.account_id.starts_with("provider:")
            {
                debug!("跳过旧版 provider auth 账号: {}", name);
                result.skipped += 1;
                continue;
            }

            let exists = registry.accounts.contains_key(&name);

            if force && exists && !registry_backed_up {
                if let Some(path) = self.backup_registry()? {
                    debug!("已备份注册表: {}", path.display());
                }
                registry_backed_up = true;
            }

            match mode {
                ImportMode::Merge => {
                    if exists && !force {
                        debug!("跳过已存在的账号: {}", name);
                        result.skipped += 1;
                        continue;
                    }
                    if exists && force {
                        debug!("强制覆盖已存在的账号: {}", name);
                        result.overwritten.push(name.clone());
                    }
                }
                ImportMode::Replace => {
                    if exists {
                        debug!("替换模式覆盖账号: {}", name);
                        result.overwritten.push(name.clone());
                    }
                }
            }

            if force && exists {
                if let Some(path) = self.backup_account_auth(&name)? {
                    debug!("已备份账号 {} 的 auth 文件: {}", name, path.display());
                }

                let auth_path = self.account_auth_path(&name);
                if auth_path.exists() {
                    let metadata = fs::metadata(&auth_path)
                        .map_err(|e| CcrError::ConfigError(format!("无法读取文件元数据: {}", e)))?;
                    if metadata.permissions().readonly() {
                        return Err(CcrError::ConfigError(format!(
                            "无法覆盖账号 '{}': 文件为只读",
                            name
                        )));
                    }

                    fs::remove_file(&auth_path)
                        .map_err(|e| CcrError::ConfigError(format!("删除 auth 文件失败: {}", e)))?;
                }

                registry.accounts.shift_remove(&name);
            }

            // 保存 auth 文件（如果有）
            if let Some(auth_data) = &import_account.auth_data {
                let auth_path = self.account_auth_path(&name);

                // 检查文件写入权限
                if auth_path.exists() {
                    let metadata = fs::metadata(&auth_path)
                        .map_err(|e| CcrError::ConfigError(format!("无法读取文件元数据: {}", e)))?;
                    if metadata.permissions().readonly() {
                        return Err(CcrError::ConfigError(format!(
                            "无法覆盖账号 '{}': 文件为只读",
                            name
                        )));
                    }
                }

                let auth_content = serde_json::to_string_pretty(auth_data)
                    .map_err(|e| CcrError::ConfigError(format!("序列化 auth 数据失败: {}", e)))?;
                fs::write(&auth_path, auth_content).map_err(|e| {
                    CcrError::ConfigError(format!("写入 auth 文件失败 (账号: {}): {}", name, e))
                })?;

                // 设置文件权限（仅当前用户可读写）
                crate::utils::ensure_private_permissions(&auth_path);

                debug!("已写入账号 {} 的 auth 文件", name);
            }

            // 更新注册表
            let account = CodexAuthAccount {
                description: import_account.description,
                account_id: import_account.account_id,
                auth_method: import_account.auth_method,
                api_base_url: import_account.api_base_url,
                api_provider_name: import_account.api_provider_name,
                email: import_account.email,
                plan_type: import_account.plan_type,
                saved_at: import_account.saved_at,
                last_used: import_account.last_used,
                last_refresh: import_account.last_refresh,
                expires_at: import_account.expires_at,
            };

            registry.accounts.insert(name.clone(), account);

            if exists {
                debug!("已更新账号: {}", name);
                result.updated += 1;
            } else {
                debug!("已添加账号: {}", name);
                result.added += 1;
            }
        }

        self.save_registry(&registry)?;

        Ok(result)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::models::{CodexAuthTokens, CodexRuntimeMode, ProfileConfig};
    use crate::test_support::TestCodexEnv;
    use chrono::Duration;
    use serde_json::json;
    use tempfile::TempDir;

    /// 创建测试用的 service 实例
    fn create_test_service() -> (CodexAuthService, TempDir, TempDir) {
        let ccr_dir = TempDir::new().unwrap();
        let codex_dir = TempDir::new().unwrap();
        fs::write(
            codex_dir.path().join("config.toml"),
            "cli_auth_credentials_store = \"file\"\n",
        )
        .unwrap();

        let service = CodexAuthService::from_dirs(
            ccr_dir.path().to_path_buf(),
            codex_dir.path().to_path_buf(),
        );

        (service, ccr_dir, codex_dir)
    }

    /// 创建测试用的 auth.json 内容
    fn create_test_auth_json(account_id: &str, last_refresh: &str) -> String {
        format!(
            r#"{{
                "OPENAI_API_KEY": null,
                "tokens": {{
                    "id_token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJlbWFpbCI6InRlc3RAZXhhbXBsZS5jb20iLCJzdWIiOiIxMjM0NTY3ODkwIn0.signature",
                    "access_token": "eyJ...",
                    "refresh_token": "rt_test",
                    "account_id": "{}"
                }},
                "last_refresh": "{}"
            }}"#,
            account_id, last_refresh
        )
    }

    fn fake_jwt(payload: serde_json::Value) -> String {
        let header = r#"{"alg":"none","typ":"JWT"}"#;
        format!(
            "{}.{}.signature",
            base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, header),
            base64::Engine::encode(
                &base64::engine::general_purpose::URL_SAFE_NO_PAD,
                payload.to_string()
            )
        )
    }

    fn official_profile() -> ProfileConfig {
        ProfileConfig {
            description: Some("Official".to_string()),
            provider: Some("openai".to_string()),
            provider_type: Some("official_relay".to_string()),
            ..ProfileConfig::default()
        }
    }

    fn provider_env_profile() -> ProfileConfig {
        let mut profile = ProfileConfig {
            description: Some("Duck".to_string()),
            base_url: Some("https://api.example.com/v1".to_string()),
            auth_token: Some(ccr_core::Secret::from("duck-key")),
            model: Some("gpt-5-codex".to_string()),
            provider: Some("duck".to_string()),
            provider_type: Some("third_party_model".to_string()),
            ..ProfileConfig::default()
        };
        profile
            .platform_data
            .insert("wire_api".to_string(), json!("responses"));
        profile
            .platform_data
            .insert("env_key".to_string(), json!("DUCK_API_KEY"));
        profile
            .platform_data
            .insert("requires_openai_auth".to_string(), json!(false));
        profile
    }

    fn runtime_fallback_profile() -> ProfileConfig {
        ProfileConfig {
            description: Some("ICE".to_string()),
            base_url: Some("https://api.example.com/v1".to_string()),
            model: Some("gpt-5.4".to_string()),
            provider: Some("ice".to_string()),
            provider_type: Some("ice".to_string()),
            ..ProfileConfig::default()
        }
    }

    #[test]
    fn test_get_runtime_summary_prefers_profile_only_for_non_openai_profile() {
        let (service, _ccr, codex) = create_test_service();

        let platform = service.platform().unwrap();
        platform
            .save_profile("ice", &runtime_fallback_profile())
            .unwrap();
        platform.apply_profile("ice").unwrap();

        let auth_path = codex.path().join("auth.json");
        fs::write(
            &auth_path,
            create_test_auth_json("test-id", "2026-01-08T03:09:53.894843900Z"),
        )
        .unwrap();

        let summary = service.get_runtime_summary().unwrap();
        assert_eq!(summary.mode, CodexRuntimeMode::ProfileOnly);
        assert_eq!(summary.current_profile_name.as_deref(), Some("ice"));
        assert_eq!(summary.current_auth_name, None);
        assert_eq!(
            summary.current_profile_auth_mode,
            Some(crate::models::CodexProfileAuthMode::NoAuth)
        );
        assert_eq!(summary.auth_label(), "No Auth");
    }

    #[test]
    fn test_get_runtime_summary_keeps_explicit_openai_api_profile_mode() {
        let (service, _ccr, codex) = create_test_service();
        let auth_path = codex.path().join("auth.json");
        fs::write(
            &auth_path,
            r#"{
                "OPENAI_API_KEY": "sk-test-1234567890",
                "tokens": null,
                "last_refresh": null
            }"#,
        )
        .unwrap();

        let mut profile = official_profile();
        profile.auth_token = Some(ccr_core::Secret::from("sk-profile-token"));

        let platform = service.platform().unwrap();
        platform.save_profile("official-api", &profile).unwrap();
        platform.apply_profile("official-api").unwrap();

        let summary = service.get_runtime_summary().unwrap();
        assert_eq!(summary.mode, CodexRuntimeMode::ProfileWithAuth);
        assert_eq!(
            summary.current_profile_auth_mode,
            Some(crate::models::CodexProfileAuthMode::OpenAiApiKey)
        );
        assert_eq!(summary.auth_label(), "OpenAI / API Key");
    }

    #[test]
    fn test_get_runtime_summary_preserves_explicit_openai_pending_state() {
        let (service, _ccr, _codex) = create_test_service();
        let mut profile = runtime_fallback_profile();
        profile
            .platform_data
            .insert("requires_openai_auth".to_string(), json!(true));
        profile
            .platform_data
            .insert("openai_login_method".to_string(), json!("chatgpt"));

        let platform = service.platform().unwrap();
        platform.save_profile("ice-openai", &profile).unwrap();
        platform.apply_profile("ice-openai").unwrap();

        let summary = service.get_runtime_summary().unwrap();
        assert_eq!(summary.mode, CodexRuntimeMode::ProfilePendingAuth);
        assert_eq!(
            summary.current_profile_auth_mode,
            Some(crate::models::CodexProfileAuthMode::OpenAiChatgpt)
        );
    }

    #[test]
    fn test_get_runtime_summary_for_profile_with_auth() {
        let (service, _ccr, codex) = create_test_service();
        let auth_path = codex.path().join("auth.json");
        fs::write(
            &auth_path,
            create_test_auth_json("test-id", "2026-01-08T03:09:53.894843900Z"),
        )
        .unwrap();

        let platform = service.platform().unwrap();
        platform
            .save_profile("official", &official_profile())
            .unwrap();
        platform.apply_profile("official").unwrap();
        service.save_current("work", None, false).unwrap();

        let summary = service.get_runtime_summary().unwrap();
        assert_eq!(summary.mode, CodexRuntimeMode::ProfileWithAuth);
        assert_eq!(summary.current_profile_name.as_deref(), Some("official"));
        assert_eq!(summary.current_auth_name.as_deref(), Some("work"));
        assert_eq!(
            summary.current_profile_auth_mode,
            Some(crate::models::CodexProfileAuthMode::OpenAiChatgpt)
        );
        assert_eq!(
            summary.profile_label(),
            "official · openai · openai_chatgpt"
        );
        assert_eq!(summary.auth_label(), "work · OpenAI / ChatGPT");
    }

    #[test]
    fn test_get_runtime_summary_for_profile_only_provider_key() {
        let (service, _ccr, _codex) = create_test_service();
        let platform = service.platform().unwrap();
        platform
            .save_profile("duck", &provider_env_profile())
            .unwrap();
        platform.apply_profile("duck").unwrap();

        let summary = service.get_runtime_summary().unwrap();
        assert_eq!(summary.mode, CodexRuntimeMode::ProfileOnly);
        assert_eq!(summary.current_profile_name.as_deref(), Some("duck"));
        assert_eq!(
            summary.current_profile_auth_mode,
            Some(crate::models::CodexProfileAuthMode::ProviderEnvKey)
        );
        assert_eq!(
            summary.current_profile_auth_source.as_deref(),
            Some("provider:DUCK_API_KEY")
        );
        assert_eq!(summary.auth_label(), "Provider / DUCK_API_KEY");
    }

    #[test]
    fn test_get_runtime_summary_ignores_global_registry_current_profile() {
        let env = TestCodexEnv::new();

        let manager = ccr_config::PlatformConfigManager::with_default().unwrap();
        let mut global_registry = manager.load_or_create_default().unwrap();
        if global_registry.get_platform("codex").is_err() {
            global_registry
                .register_platform(
                    "codex".to_string(),
                    ccr_config::PlatformConfigEntry::default(),
                )
                .unwrap();
        }
        global_registry
            .set_platform_profile("codex", "stale-global-profile")
            .unwrap();
        manager.save(&global_registry).unwrap();

        let (service, _ccr, _codex) = create_test_service();
        let platform = service.platform().unwrap();
        platform
            .save_profile("duck", &provider_env_profile())
            .unwrap();
        platform.apply_profile("duck").unwrap();

        let summary = service.get_runtime_summary().unwrap();
        assert_eq!(summary.mode, CodexRuntimeMode::ProfileOnly);
        assert_eq!(summary.current_profile_name.as_deref(), Some("duck"));
        assert_eq!(
            summary.current_profile_auth_mode,
            Some(crate::models::CodexProfileAuthMode::ProviderEnvKey)
        );
        assert!(
            !service
                .platform_paths()
                .registry_file
                .starts_with(env.root())
        );
    }

    #[test]
    fn test_get_runtime_summary_for_runtime_only_auth() {
        let (service, _ccr, codex) = create_test_service();
        let auth_path = codex.path().join("auth.json");
        fs::write(
            &auth_path,
            create_test_auth_json("test-id", "2026-01-08T03:09:53.894843900Z"),
        )
        .unwrap();
        service.save_current("work", None, false).unwrap();

        let summary = service.get_runtime_summary().unwrap();
        assert_eq!(summary.mode, CodexRuntimeMode::RuntimeOnly);
        assert_eq!(summary.current_profile_name, None);
        assert_eq!(summary.current_auth_name.as_deref(), Some("work"));
        assert_eq!(summary.auth_label(), "work · OpenAI / ChatGPT");
    }

    #[test]
    fn test_get_runtime_summary_ignores_stale_profile_pointer_after_auth_switch() {
        let (service, _ccr, codex) = create_test_service();
        let platform = service.platform().unwrap();
        platform
            .save_profile("ice", &runtime_fallback_profile())
            .unwrap();
        platform.apply_profile("ice").unwrap();

        let auth_path = codex.path().join("auth.json");
        fs::write(
            &auth_path,
            create_test_auth_json("test-id", "2026-01-08T03:09:53.894843900Z"),
        )
        .unwrap();

        service.save_current("plus", None, false).unwrap();
        service.switch_account("plus").unwrap();

        let summary = service.get_runtime_summary().unwrap();
        assert_eq!(summary.mode, CodexRuntimeMode::RuntimeOnly);
        assert_eq!(summary.current_profile_name, None);
        assert_eq!(summary.current_auth_name.as_deref(), Some("plus"));
        assert_eq!(summary.auth_label(), "plus · OpenAI / ChatGPT");
    }

    #[test]
    fn test_get_runtime_summary_for_profile_pending_auth() {
        let (service, _ccr, _codex) = create_test_service();
        let platform = service.platform().unwrap();
        platform
            .save_profile("official", &official_profile())
            .unwrap();
        platform.apply_profile("official").unwrap();

        let summary = service.get_runtime_summary().unwrap();
        assert_eq!(summary.mode, CodexRuntimeMode::ProfilePendingAuth);
        assert_eq!(summary.current_profile_name.as_deref(), Some("official"));
        assert_eq!(summary.auth_label(), "未登录 · OpenAI / ChatGPT");
    }

    // ==================== 邮箱脱敏测试 ====================

    #[test]
    fn test_mask_email() {
        let (service, _ccr, _codex) = create_test_service();

        assert_eq!(service.mask_email("user@example.com"), "use***@example.com");
        assert_eq!(service.mask_email("ab@example.com"), "ab***@example.com");
        assert_eq!(service.mask_email("a@example.com"), "a***@example.com");
        assert_eq!(service.mask_email("invalid"), "invalid");
    }

    #[test]
    fn test_mask_email_edge_cases() {
        let (service, _ccr, _codex) = create_test_service();

        // 空邮箱
        assert_eq!(service.mask_email(""), "");
        // 只有 @
        assert_eq!(service.mask_email("@domain.com"), "***@domain.com");
        // 多个 @
        assert_eq!(
            service.mask_email("user@sub@domain.com"),
            "use***@sub@domain.com"
        );
    }

    // ==================== 账号名称验证测试 ====================

    #[test]
    fn test_validate_account_name() {
        let (service, _ccr, _codex) = create_test_service();

        // 有效名称
        assert!(service.validate_account_name("my-account").is_ok());
        assert!(service.validate_account_name("account_1").is_ok());
        assert!(service.validate_account_name("Account123").is_ok());
        assert!(service.validate_account_name("a").is_ok());
        assert!(service.validate_account_name("A1_b2-c3").is_ok());

        // 无效名称
        assert!(service.validate_account_name("").is_err());
        assert!(service.validate_account_name("default").is_err());
        assert!(service.validate_account_name("invalid name").is_err());
        assert!(service.validate_account_name("名称").is_err());
        assert!(service.validate_account_name("user@email").is_err());
        assert!(service.validate_account_name("path/name").is_err());
    }

    #[test]
    fn test_validate_account_name_length() {
        let (service, _ccr, _codex) = create_test_service();

        // 32 字符 - 有效
        let valid_name = "a".repeat(32);
        assert!(service.validate_account_name(&valid_name).is_ok());

        // 33 字符 - 无效
        let invalid_name = "a".repeat(33);
        assert!(service.validate_account_name(&invalid_name).is_err());
    }

    #[test]
    fn test_reserve_explicit_account_name_rejects_duplicates() {
        let (service, _ccr, codex) = create_test_service();

        let auth_path = codex.path().join("auth.json");
        let auth_content = create_test_auth_json("test-id", "2026-01-08T03:09:53.894843900Z");
        fs::write(&auth_path, auth_content).unwrap();
        service.save_current("existing", None, false).unwrap();

        let duplicate = service.reserve_explicit_account_name("existing");
        assert!(duplicate.is_err());
        assert!(duplicate.unwrap_err().to_string().contains("已存在"));

        assert_eq!(
            service.reserve_explicit_account_name("fresh-name").unwrap(),
            "fresh-name"
        );
    }

    // ==================== 注册表测试 ====================

    #[test]
    fn test_registry_default() {
        let registry = CodexAuthRegistry::default();
        assert_eq!(registry.version, "1.0");
        assert!(registry.current_auth.is_none());
        assert!(registry.accounts.is_empty());
    }

    #[test]
    fn test_registry_serialization() {
        let mut registry = CodexAuthRegistry {
            current_auth: Some("test-account".to_string()),
            ..Default::default()
        };
        registry.accounts.insert(
            "test-account".to_string(),
            CodexAuthAccount {
                description: Some("Test".to_string()),
                account_id: "acc-123".to_string(),
                auth_method: Some(OpenAiAuthMethod::Chatgpt),
                api_base_url: None,
                api_provider_name: None,
                email: Some("tes***@example.com".to_string()),
                plan_type: None,
                saved_at: Utc::now(),
                last_used: None,
                last_refresh: None,
                expires_at: None,
            },
        );

        // 序列化
        let toml_str = toml::to_string_pretty(&registry).unwrap();
        assert!(toml_str.contains("test-account"));
        assert!(toml_str.contains("acc-123"));

        // 反序列化
        let parsed: CodexAuthRegistry = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.current_auth, Some("test-account".to_string()));
        assert!(parsed.accounts.contains_key("test-account"));
    }

    // ==================== 登录状态测试 ====================

    #[test]
    fn test_is_logged_in_no_file() {
        let (service, _ccr, _codex) = create_test_service();
        assert!(!service.is_logged_in());
    }

    #[test]
    fn test_is_logged_in_with_valid_auth() {
        let (service, _ccr, codex) = create_test_service();

        // 创建有效的 auth.json
        let auth_path = codex.path().join("auth.json");
        let auth_content = create_test_auth_json("test-id", "2026-01-08T03:09:53.894843900Z");
        fs::write(&auth_path, auth_content).unwrap();

        assert!(service.is_logged_in());
    }

    #[test]
    fn test_is_logged_in_with_invalid_json() {
        let (service, _ccr, codex) = create_test_service();

        // 创建无效的 auth.json
        let auth_path = codex.path().join("auth.json");
        fs::write(&auth_path, "invalid json content").unwrap();

        assert!(!service.is_logged_in());
    }

    #[test]
    fn test_is_logged_in_with_empty_object() {
        let (service, _ccr, codex) = create_test_service();

        let auth_path = codex.path().join("auth.json");
        fs::write(&auth_path, "{}").unwrap();

        assert!(!service.is_logged_in());
        let state = service.get_auth_state();
        assert_eq!(state.status, AuthStateStatus::Missing);
    }

    #[test]
    fn test_get_login_state_not_logged_in() {
        let (service, _ccr, _codex) = create_test_service();
        let state = service.get_login_state().unwrap();
        assert_eq!(state, LoginState::NotLoggedIn);
    }

    #[test]
    fn test_get_login_state_logged_in_unsaved() {
        let (service, _ccr, codex) = create_test_service();

        // 创建 auth.json 但不保存到注册表
        let auth_path = codex.path().join("auth.json");
        let auth_content = create_test_auth_json("test-id", "2026-01-08T03:09:53.894843900Z");
        fs::write(&auth_path, auth_content).unwrap();

        let state = service.get_login_state().unwrap();
        assert_eq!(state, LoginState::LoggedInUnsaved);
    }

    #[test]
    fn test_get_login_state_does_not_create_registry_side_effects() {
        let (service, _ccr, codex) = create_test_service();

        let auth_path = codex.path().join("auth.json");
        let auth_content = create_test_auth_json("test-id", "2026-01-08T03:09:53.894843900Z");
        fs::write(&auth_path, auth_content).unwrap();

        let registry_path = service.registry_path();
        assert!(!registry_path.exists());

        let state = service.get_login_state().unwrap();
        assert_eq!(state, LoginState::LoggedInUnsaved);
        assert!(!registry_path.exists());
    }

    #[test]
    fn test_get_current_auth_info_api_key_identity() {
        let (service, _ccr, codex) = create_test_service();

        let auth_path = codex.path().join("auth.json");
        fs::write(
            &auth_path,
            r#"{
                "OPENAI_API_KEY": "sk-test-api-key-123456",
                "last_refresh": "2026-01-08T03:09:53.894843900Z"
            }"#,
        )
        .unwrap();

        let info = service.get_current_auth_info().unwrap();
        assert!(info.account_id.starts_with("api:"));
        assert_ne!(info.account_id, "unknown");
    }

    #[test]
    fn test_keyring_store_is_reported_as_unsupported() {
        let (service, _ccr, codex) = create_test_service();
        fs::write(
            codex.path().join("config.toml"),
            "cli_auth_credentials_store = \"keyring\"\n",
        )
        .unwrap();
        fs::write(
            codex.path().join("auth.json"),
            r#"{"OPENAI_API_KEY":"sk-test-api-key"}"#,
        )
        .unwrap();

        let state = service.get_auth_state();
        assert_eq!(state.store, CredentialStoreKind::Keyring);
        assert_eq!(state.status, AuthStateStatus::Unsupported);

        let err = service
            .save_current("work", Some("unsupported".to_string()), false)
            .unwrap_err()
            .to_string();
        assert!(err.contains("cli_auth_credentials_store"));
    }

    // ==================== 账号管理工作流测试 ====================

    #[test]
    fn test_save_switch_delete_workflow() {
        let (service, _ccr, codex) = create_test_service();

        // 1. 创建 auth.json
        let auth_path = codex.path().join("auth.json");
        let auth_content = create_test_auth_json("test-id-1", "2026-01-08T03:09:53.894843900Z");
        fs::write(&auth_path, auth_content).unwrap();

        // 2. 保存账号
        service
            .save_current("account1", Some("First account".to_string()), false)
            .unwrap();

        // 验证保存成功
        let accounts = service.list_accounts().unwrap();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].name, "account1");
        assert!(!accounts[0].is_virtual);

        // 3. 创建第二个 auth.json 并保存
        let auth_content2 = create_test_auth_json("test-id-2", "2026-01-09T03:09:53.894843900Z");
        fs::write(&auth_path, auth_content2).unwrap();

        service
            .save_current("account2", Some("Second account".to_string()), false)
            .unwrap();

        // 验证两个账号
        let accounts = service.list_accounts().unwrap();
        assert_eq!(accounts.len(), 2);

        // 4. 切换到 account1
        service.switch_account("account1").unwrap();

        // 验证切换成功
        let state = service.get_login_state().unwrap();
        assert_eq!(state, LoginState::LoggedInSaved("account1".to_string()));

        // 5. 删除 account2
        service.delete_account("account2").unwrap();

        // 验证删除成功
        let accounts = service.list_accounts().unwrap();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].name, "account1");
    }

    #[test]
    fn test_save_duplicate_without_force() {
        let (service, _ccr, codex) = create_test_service();

        // 创建 auth.json
        let auth_path = codex.path().join("auth.json");
        let auth_content = create_test_auth_json("test-id", "2026-01-08T03:09:53.894843900Z");
        fs::write(&auth_path, auth_content).unwrap();

        // 第一次保存
        service.save_current("myaccount", None, false).unwrap();

        // 第二次保存同名 - 应该失败
        let result = service.save_current("myaccount", None, false);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("已存在"));
    }

    #[test]
    fn test_save_duplicate_with_force() {
        let (service, _ccr, codex) = create_test_service();

        // 创建 auth.json
        let auth_path = codex.path().join("auth.json");
        let auth_content = create_test_auth_json("test-id", "2026-01-08T03:09:53.894843900Z");
        fs::write(&auth_path, auth_content).unwrap();

        // 第一次保存
        service.save_current("myaccount", None, false).unwrap();

        // 第二次保存同名 with force - 应该成功
        let result = service.save_current("myaccount", Some("Updated".to_string()), true);
        assert!(result.is_ok());

        // 验证描述已更新
        let accounts = service.list_accounts().unwrap();
        assert_eq!(accounts[0].description, Some("Updated".to_string()));
    }

    #[test]
    fn test_switch_nonexistent_account() {
        let (service, _ccr, _codex) = create_test_service();

        let result = service.switch_account("nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("不存在"));
    }

    #[test]
    fn test_switch_account_rewrites_auth_with_only_normalized_target_fields() {
        let (service, _ccr, codex) = create_test_service();

        let auth_path = codex.path().join("auth.json");
        fs::write(
            &auth_path,
            r#"{
                "OPENAI_API_KEY": "sk-old",
                "custom_meta": "keep-me"
            }"#,
        )
        .unwrap();

        service
            .save_current("merged", Some("merge test".to_string()), false)
            .unwrap();

        let account_path = service.account_auth_path("merged");
        fs::write(
            &account_path,
            r#"{
                "OPENAI_API_KEY": "sk-new",
                "last_refresh": "2026-01-08T03:09:53.894843900Z",
                "custom_meta": "drop-me"
            }"#,
        )
        .unwrap();

        service.switch_account("merged").unwrap();

        let merged: serde_json::Map<String, serde_json::Value> =
            serde_json::from_str(&fs::read_to_string(&auth_path).unwrap()).unwrap();
        assert_eq!(
            merged.get("OPENAI_API_KEY").and_then(|v| v.as_str()),
            Some("sk-new")
        );
        assert!(!merged.contains_key("last_refresh"));
        assert!(!merged.contains_key("custom_meta"));
        assert_eq!(merged.len(), 1);
    }

    #[test]
    fn test_switch_account_clears_custom_route_for_plain_openai_accounts() {
        let (service, _ccr, codex) = create_test_service();

        fs::write(
            codex.path().join("config.toml"),
            r#"
cli_auth_credentials_store = "file"
model_provider = "custom"

[model_providers.custom]
name = "legacy"
base_url = "https://legacy.example.com/v1"
wire_api = "responses"
requires_openai_auth = true
"#,
        )
        .unwrap();

        let auth_path = codex.path().join("auth.json");
        fs::write(
            &auth_path,
            create_test_auth_json("test-id", "2026-01-08T03:09:53.894843900Z"),
        )
        .unwrap();

        let platform = service.platform().unwrap();
        platform
            .save_profile("official", &official_profile())
            .unwrap();
        platform.apply_profile("official").unwrap();

        service.save_current("plain-openai", None, false).unwrap();
        service.switch_account("plain-openai").unwrap();

        let config = service
            .codex_config_manager()
            .unwrap()
            .load_config()
            .unwrap();
        let root = config.as_table().unwrap();
        assert_eq!(
            root.get("forced_login_method")
                .and_then(|value| value.as_str()),
            Some("chatgpt")
        );
        assert!(
            root.get("model_provider").is_none(),
            "plain OpenAI auth switch should not write custom model_provider"
        );
        assert!(
            root.get("model_providers").is_none(),
            "plain OpenAI auth switch should clear legacy custom providers"
        );
    }

    #[test]
    fn test_switch_account_keeps_custom_route_for_explicit_base_url_accounts() {
        let (service, _ccr, codex) = create_test_service();

        let platform = service.platform().unwrap();
        platform
            .save_profile("official-api", &official_profile())
            .unwrap();
        platform.apply_profile("official-api").unwrap();

        let auth_path = codex.path().join("auth.json");
        fs::write(
            &auth_path,
            r#"{
                "OPENAI_API_KEY": "sk-explicit-base-url",
                "last_refresh": "2026-01-08T03:09:53.894843900Z"
            }"#,
        )
        .unwrap();

        service.save_current("relay-account", None, false).unwrap();

        let mut registry = service.load_registry().unwrap();
        let account = registry.accounts.get_mut("relay-account").unwrap();
        account.api_base_url = Some("https://relay.example.com/v1".to_string());
        account.api_provider_name = Some("relay-edge".to_string());
        account.auth_method = Some(OpenAiAuthMethod::Api);
        service.save_registry(&registry).unwrap();

        service.switch_account("relay-account").unwrap();

        let config = service
            .codex_config_manager()
            .unwrap()
            .load_config()
            .unwrap();
        let root = config.as_table().unwrap();
        assert_eq!(
            root.get("model_provider").and_then(|value| value.as_str()),
            Some("custom")
        );
        let providers = root
            .get("model_providers")
            .and_then(|value| value.as_table())
            .unwrap();
        let custom = providers
            .get("custom")
            .and_then(|value| value.as_table())
            .unwrap();
        assert_eq!(
            custom.get("name").and_then(|value| value.as_str()),
            Some("relay-edge")
        );
        assert_eq!(
            custom.get("base_url").and_then(|value| value.as_str()),
            Some("https://relay.example.com/v1")
        );
    }

    #[test]
    fn test_delete_nonexistent_account() {
        let (service, _ccr, _codex) = create_test_service();

        let result = service.delete_account("nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("不存在"));
    }

    // ==================== 重命名测试 ====================

    #[test]
    fn test_rename_account_moves_file_and_updates_registry() {
        let (service, _ccr, codex) = create_test_service();

        let auth_path = codex.path().join("auth.json");
        fs::write(
            &auth_path,
            create_test_auth_json("acc-rename", "2026-04-10T10:00:00Z"),
        )
        .unwrap();

        service
            .save_current("old_name", Some("rename me".to_string()), false)
            .unwrap();
        service
            .rename_account("old_name", "new_name", false)
            .unwrap();

        let registry = service.load_registry().unwrap();
        assert!(registry.accounts.contains_key("new_name"));
        assert!(!registry.accounts.contains_key("old_name"));
        assert_eq!(registry.current_auth.as_deref(), Some("new_name"));

        // 源文件已搬迁
        let old_path = service.account_auth_path("old_name");
        let new_path = service.account_auth_path("new_name");
        assert!(!old_path.exists(), "旧 auth 文件应被清理");
        assert!(new_path.exists(), "新 auth 文件应存在");

        // 描述元数据保留
        let account = registry.accounts.get("new_name").unwrap();
        assert_eq!(account.description.as_deref(), Some("rename me"));
    }

    #[test]
    fn test_rename_account_preserves_usage_ledger_attribution() {
        let (service, _ccr, codex) = create_test_service();

        let auth_path = codex.path().join("auth.json");
        fs::write(
            &auth_path,
            create_test_auth_json("acc-ledger", "2026-04-10T10:00:00Z"),
        )
        .unwrap();

        service.save_current("src", None, false).unwrap();
        service.rename_account("src", "dst", false).unwrap();

        let registry = service.load_registry().unwrap();
        assert!(
            registry
                .usage_ledger
                .iter()
                .all(|entry| entry.account_name != "src"),
            "ledger 不应残留旧名称"
        );
        assert!(
            registry
                .usage_ledger
                .iter()
                .any(|entry| entry.account_name == "dst"),
            "ledger 应改写为新名称"
        );
    }

    #[test]
    fn test_rename_account_rejects_conflict_without_force() {
        let (service, _ccr, codex) = create_test_service();

        let auth_path = codex.path().join("auth.json");
        fs::write(
            &auth_path,
            create_test_auth_json("acc-a", "2026-04-10T10:00:00Z"),
        )
        .unwrap();

        service.save_current("alpha", None, false).unwrap();
        service.save_current("beta", None, true).unwrap();

        let result = service.rename_account("alpha", "beta", false);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("已存在"));

        let registry = service.load_registry().unwrap();
        assert!(registry.accounts.contains_key("alpha"));
        assert!(registry.accounts.contains_key("beta"));
    }

    #[test]
    fn test_rename_account_force_overwrites_conflict() {
        let (service, _ccr, codex) = create_test_service();

        let auth_path = codex.path().join("auth.json");
        fs::write(
            &auth_path,
            create_test_auth_json("acc-f", "2026-04-10T10:00:00Z"),
        )
        .unwrap();

        service.save_current("alpha", None, false).unwrap();
        service.save_current("beta", None, true).unwrap();

        service.rename_account("alpha", "beta", true).unwrap();

        let registry = service.load_registry().unwrap();
        assert!(registry.accounts.contains_key("beta"));
        assert!(!registry.accounts.contains_key("alpha"));
    }

    #[test]
    fn test_rename_account_rejects_invalid_new_name() {
        let (service, _ccr, codex) = create_test_service();

        let auth_path = codex.path().join("auth.json");
        fs::write(
            &auth_path,
            create_test_auth_json("acc-v", "2026-04-10T10:00:00Z"),
        )
        .unwrap();

        service.save_current("keep", None, false).unwrap();

        let result = service.rename_account("keep", "bad name!", false);
        assert!(result.is_err());

        // default 是保留名称
        let result_default = service.rename_account("keep", "default", false);
        assert!(result_default.is_err());
    }

    #[test]
    fn test_rename_account_same_name_is_noop() {
        let (service, _ccr, codex) = create_test_service();

        let auth_path = codex.path().join("auth.json");
        fs::write(
            &auth_path,
            create_test_auth_json("acc-same", "2026-04-10T10:00:00Z"),
        )
        .unwrap();

        service.save_current("stable", None, false).unwrap();
        let result = service.rename_account("stable", "stable", false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_rename_account_missing_source_errors() {
        let (service, _ccr, _codex) = create_test_service();

        let result = service.rename_account("ghost", "phantom", false);
        assert!(result.is_err());
    }

    // ==================== 虚拟 default 账号测试 ====================

    #[test]
    fn test_virtual_default_account() {
        let (service, _ccr, codex) = create_test_service();

        // 创建 auth.json 但不保存
        let auth_path = codex.path().join("auth.json");
        let auth_content = create_test_auth_json("test-id", "2026-01-08T03:09:53.894843900Z");
        fs::write(&auth_path, auth_content).unwrap();

        // 列出账号 - 应该有虚拟 default
        let accounts = service.list_accounts().unwrap();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].name, "default");
        assert!(accounts[0].is_virtual);
        assert!(accounts[0].is_current);
    }

    #[test]
    fn test_no_virtual_default_when_saved() {
        let (service, _ccr, codex) = create_test_service();

        // 创建 auth.json
        let auth_path = codex.path().join("auth.json");
        let auth_content = create_test_auth_json("test-id", "2026-01-08T03:09:53.894843900Z");
        fs::write(&auth_path, auth_content).unwrap();

        // 保存账号
        service.save_current("myaccount", None, false).unwrap();

        // 列出账号 - 不应该有虚拟 default
        let accounts = service.list_accounts().unwrap();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].name, "myaccount");
        assert!(!accounts[0].is_virtual);
    }

    // ==================== 备份测试 ====================

    #[test]
    fn test_backup_current_auth() {
        let (service, _ccr, codex) = create_test_service();

        // 创建 auth.json
        let auth_path = codex.path().join("auth.json");
        let auth_content = create_test_auth_json("test-id", "2026-01-08T03:09:53.894843900Z");
        fs::write(&auth_path, auth_content).unwrap();

        // 备份
        let backup_path = service.backup_current_auth().unwrap();
        assert!(backup_path.exists());
        assert!(backup_path.to_string_lossy().contains("auth_"));
    }

    #[test]
    fn test_backup_rotation() {
        let (service, ccr, codex) = create_test_service();

        // 创建 auth.json
        let auth_path = codex.path().join("auth.json");
        let auth_content = create_test_auth_json("test-id", "2026-01-08T03:09:53.894843900Z");
        fs::write(&auth_path, auth_content).unwrap();

        // 创建备份目录和 15 个旧备份
        let backup_dir = ccr.path().join("auth/backups");
        fs::create_dir_all(&backup_dir).unwrap();

        for i in 0..15 {
            let backup_name = format!("auth_20260101_{:06}.json", i);
            fs::write(backup_dir.join(&backup_name), "{}").unwrap();
        }

        // 执行新备份 (会触发清理)
        service.backup_current_auth().unwrap();

        // 验证只保留 MAX_BACKUPS 个
        let backups: Vec<_> = fs::read_dir(&backup_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        assert!(backups.len() <= MAX_BACKUPS + 1); // +1 for the new backup
    }

    // ==================== JWT 解析测试 ====================

    #[test]
    fn test_base64url_decode() {
        let (_service, _ccr, _codex) = create_test_service();

        // 标准 base64url 编码的 "test"
        let decoded = crate::utils::decode_base64url("dGVzdA").unwrap();
        assert_eq!(decoded, b"test");

        // 带 padding 的情况
        let decoded2 = crate::utils::decode_base64url("dGVzdA==").unwrap();
        assert_eq!(decoded2, b"test");
    }

    #[test]
    fn test_extract_email_from_jwt() {
        let (service, _ccr, _codex) = create_test_service();

        // 创建包含 email 的 JWT payload
        // {"email":"test@example.com","sub":"1234567890"}
        // Base64URL: eyJlbWFpbCI6InRlc3RAZXhhbXBsZS5jb20iLCJzdWIiOiIxMjM0NTY3ODkwIn0
        let auth = CodexAuthJson {
            openai_api_key: None,
            tokens: Some(CodexAuthTokens {
                id_token: Some("eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJlbWFpbCI6InRlc3RAZXhhbXBsZS5jb20iLCJzdWIiOiIxMjM0NTY3ODkwIn0.signature".to_string()),
                access_token: None,
                refresh_token: None,
                account_id: Some("test-id".to_string()),
            }),
            last_refresh: None,
        };

        let email = service.extract_email_from_jwt(&auth);
        assert_eq!(email, Some("test@example.com".to_string()));
    }

    #[test]
    fn test_extract_email_no_token() {
        let (service, _ccr, _codex) = create_test_service();

        let auth = CodexAuthJson {
            openai_api_key: None,
            tokens: None,
            last_refresh: None,
        };

        let email = service.extract_email_from_jwt(&auth);
        assert!(email.is_none());
    }

    #[test]
    fn test_extract_plan_type_prefers_access_token_claims() {
        let (service, _ccr, _codex) = create_test_service();

        let auth = CodexAuthJson {
            openai_api_key: None,
            tokens: Some(CodexAuthTokens {
                id_token: Some(fake_jwt(json!({
                    "plan": "TEAM"
                }))),
                access_token: Some(fake_jwt(json!({
                    "chatgpt_plan_type": "PRO_20X"
                }))),
                refresh_token: None,
                account_id: Some("test-id".to_string()),
            }),
            last_refresh: None,
        };

        let plan_type = service.extract_plan_type_from_auth(&auth);
        assert_eq!(plan_type.as_deref(), Some("pro 20x"));
    }

    #[test]
    fn test_save_current_persists_plan_type_from_runtime_tokens() {
        let (service, _ccr, codex) = create_test_service();

        let auth_path = codex.path().join("auth.json");
        let auth_json = json!({
            "tokens": {
                "id_token": fake_jwt(json!({
                    "email": "test@example.com",
                    "sub": "subject-123"
                })),
                "access_token": fake_jwt(json!({
                    "chatgpt_plan_type": "TEAM"
                })),
                "refresh_token": "rt_test",
                "account_id": "acc-team"
            },
            "last_refresh": "2026-01-08T03:09:53.894843900Z"
        });
        fs::write(
            &auth_path,
            serde_json::to_string_pretty(&auth_json).unwrap(),
        )
        .unwrap();

        service.save_current("team", None, false).unwrap();

        let registry = service.load_registry().unwrap();
        assert_eq!(registry.accounts["team"].plan_type.as_deref(), Some("team"));

        let items = service.list_accounts().unwrap();
        assert_eq!(items[0].plan_type.as_deref(), Some("team"));
    }

    // ==================== 进程检测测试 ====================

    #[test]
    fn test_detect_codex_process() {
        let (service, _ccr, _codex) = create_test_service();

        // 这个测试主要验证函数不会 panic
        // 实际检测结果取决于系统状态
        let pids = service.detect_codex_process();
        // 返回类型正确即可
        assert!(pids.is_empty() || !pids.is_empty());
    }

    #[test]
    fn test_registry_with_expiry_serialization() {
        let mut registry = CodexAuthRegistry {
            current_auth: Some("test-account".to_string()),
            ..Default::default()
        };

        let expires_at = Utc::now() + Duration::days(30);
        registry.accounts.insert(
            "test-account".to_string(),
            CodexAuthAccount {
                description: Some("Test".to_string()),
                account_id: "acc-123".to_string(),
                auth_method: Some(OpenAiAuthMethod::Chatgpt),
                api_base_url: None,
                api_provider_name: None,
                email: Some("tes***@example.com".to_string()),
                plan_type: None,
                saved_at: Utc::now(),
                last_used: None,
                last_refresh: None,
                expires_at: Some(expires_at),
            },
        );

        // 序列化
        let toml_str = toml::to_string_pretty(&registry).unwrap();
        assert!(toml_str.contains("expires_at"));

        // 反序列化
        let parsed: CodexAuthRegistry = toml::from_str(&toml_str).unwrap();
        let account = parsed.accounts.get("test-account").unwrap();
        assert!(account.expires_at.is_some());
    }

    #[test]
    fn test_registry_without_expiry_serialization() {
        let mut registry = CodexAuthRegistry {
            current_auth: Some("test-account".to_string()),
            ..Default::default()
        };

        registry.accounts.insert(
            "test-account".to_string(),
            CodexAuthAccount {
                description: Some("Test".to_string()),
                account_id: "acc-123".to_string(),
                auth_method: Some(OpenAiAuthMethod::Chatgpt),
                api_base_url: None,
                api_provider_name: None,
                email: Some("tes***@example.com".to_string()),
                plan_type: None,
                saved_at: Utc::now(),
                last_used: None,
                last_refresh: None,
                expires_at: None,
            },
        );

        // 序列化时 None 应该被跳过
        let toml_str = toml::to_string_pretty(&registry).unwrap();
        assert!(!toml_str.contains("expires_at"));

        // 反序列化
        let parsed: CodexAuthRegistry = toml::from_str(&toml_str).unwrap();
        let account = parsed.accounts.get("test-account").unwrap();
        assert!(account.expires_at.is_none());
    }

    #[test]
    fn test_switch_to_legacy_expired_account_still_succeeds() {
        let (service, _ccr, codex) = create_test_service();

        // 创建 auth.json
        let auth_path = codex.path().join("auth.json");
        let auth_content = create_test_auth_json("test-id", "2026-01-08T03:09:53.894843900Z");
        fs::write(&auth_path, auth_content).unwrap();

        // 保存账号，再注入旧版过期字段
        let past = Utc::now() - Duration::days(1);
        service
            .save_current("expired-account", Some("Expired".to_string()), false)
            .unwrap();
        let mut registry = service.load_registry().unwrap();
        registry
            .accounts
            .get_mut("expired-account")
            .unwrap()
            .expires_at = Some(past);
        service.save_registry(&registry).unwrap();

        // 创建另一个 auth.json 以便切换
        let auth_content2 = create_test_auth_json("test-id-2", "2026-01-09T03:09:53.894843900Z");
        fs::write(&auth_path, auth_content2).unwrap();

        service.switch_account("expired-account").unwrap();

        let registry = service.load_registry().unwrap();
        assert_eq!(registry.current_auth.as_deref(), Some("expired-account"));
    }

    #[test]
    fn test_switch_to_non_expired_account_allowed() {
        let (service, _ccr, codex) = create_test_service();

        // 创建 auth.json
        let auth_path = codex.path().join("auth.json");
        let auth_content = create_test_auth_json("test-id", "2026-01-08T03:09:53.894843900Z");
        fs::write(&auth_path, auth_content).unwrap();

        // 保存账号，再注入旧版过期字段
        let future = Utc::now() + Duration::days(30);
        service
            .save_current("valid-account", Some("Valid".to_string()), false)
            .unwrap();
        let mut registry = service.load_registry().unwrap();
        registry
            .accounts
            .get_mut("valid-account")
            .unwrap()
            .expires_at = Some(future);
        service.save_registry(&registry).unwrap();

        // 创建另一个 auth.json 以便切换
        let auth_content2 = create_test_auth_json("test-id-2", "2026-01-09T03:09:53.894843900Z");
        fs::write(&auth_path, auth_content2).unwrap();

        // 切换到未过期账号 - 应该成功
        let result = service.switch_account("valid-account");
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_accounts_ignores_legacy_expiry_metadata() {
        let (service, _ccr, codex) = create_test_service();

        // 创建 auth.json
        let auth_path = codex.path().join("auth.json");
        let auth_content = create_test_auth_json("test-id", "2026-01-08T03:09:53.894843900Z");
        fs::write(&auth_path, auth_content).unwrap();

        // 保存账号并注入旧版过期字段
        let future = Utc::now() + Duration::days(30);
        service.save_current("with-expiry", None, false).unwrap();
        let mut registry = service.load_registry().unwrap();
        registry.accounts.get_mut("with-expiry").unwrap().expires_at = Some(future);
        service.save_registry(&registry).unwrap();

        // 列出账号
        let accounts = service.list_accounts().unwrap();
        let account = accounts.iter().find(|a| a.name == "with-expiry").unwrap();
        assert!(account.last_refresh.is_some());
    }

    // ==================== 导入账号测试 ====================

    #[test]
    fn test_import_accounts_merge_without_force() {
        let (service, _ccr, _codex) = create_test_service();

        // 先保存一个账号
        let auth_path = service.codex_dir.join("auth.json");
        let auth_content = create_test_auth_json("existing-id", "2026-01-08T03:09:53.894843900Z");
        fs::write(&auth_path, auth_content).unwrap();
        service
            .save_current("existing", Some("Existing account".to_string()), false)
            .unwrap();

        // 准备导入数据（包含同名账号和新账号）
        let import_json = r#"{
            "version": "1.0",
            "exported_at": "2026-01-22T00:00:00Z",
            "accounts": {
                "existing": {
                    "description": "Updated description",
                    "account_id": "new-id",
                    "email": "new***@example.com",
                    "saved_at": "2026-01-22T00:00:00Z",
                    "auth_data": {
                        "tokens": {
                            "id_token": "new_token",
                            "access_token": "new_access",
                            "refresh_token": "new_refresh",
                            "account_id": "new-id"
                        },
                        "last_refresh": "2026-01-22T00:00:00Z"
                    }
                },
                "new-account": {
                    "description": "New account",
                    "account_id": "new-account-id",
                    "email": "new***@example.com",
                    "saved_at": "2026-01-22T00:00:00Z"
                }
            }
        }"#;

        // 合并模式，不强制覆盖
        let result = service
            .import_accounts(import_json, ImportMode::Merge, false)
            .unwrap();

        // 验证结果
        assert_eq!(result.added, 1); // new-account
        assert_eq!(result.updated, 0);
        assert_eq!(result.skipped, 1); // existing
        assert_eq!(result.overwritten.len(), 0);

        // 验证 registry 中的账号数量
        let registry = service.load_registry().unwrap();
        assert_eq!(registry.accounts.len(), 2);

        // 验证 existing 账号没有被更新
        let existing = registry.accounts.get("existing").unwrap();
        assert_eq!(existing.account_id, "existing-id");
        assert_eq!(existing.description, Some("Existing account".to_string()));
    }

    #[test]
    fn test_import_accounts_merge_with_force() {
        let (service, _ccr, _codex) = create_test_service();

        // 先保存一个账号
        let auth_path = service.codex_dir.join("auth.json");
        let auth_content = create_test_auth_json("existing-id", "2026-01-08T03:09:53.894843900Z");
        fs::write(&auth_path, auth_content).unwrap();
        service
            .save_current("existing", Some("Existing account".to_string()), false)
            .unwrap();

        // 准备导入数据
        let import_json = r#"{
            "version": "1.0",
            "exported_at": "2026-01-22T00:00:00Z",
            "accounts": {
                "existing": {
                    "description": "Updated description",
                    "account_id": "new-id",
                    "email": "new***@example.com",
                    "saved_at": "2026-01-22T00:00:00Z",
                    "auth_data": {
                        "tokens": {
                            "id_token": "new_token",
                            "access_token": "new_access",
                            "refresh_token": "new_refresh",
                            "account_id": "new-id"
                        },
                        "last_refresh": "2026-01-22T00:00:00Z"
                    }
                },
                "new-account": {
                    "description": "New account",
                    "account_id": "new-account-id",
                    "email": "new***@example.com",
                    "saved_at": "2026-01-22T00:00:00Z"
                }
            }
        }"#;

        // 合并模式，强制覆盖
        let result = service
            .import_accounts(import_json, ImportMode::Merge, true)
            .unwrap();

        // 验证结果
        assert_eq!(result.added, 1); // new-account
        assert_eq!(result.updated, 1); // existing 被更新
        assert_eq!(result.skipped, 0);
        assert_eq!(result.overwritten.len(), 1);
        assert_eq!(result.overwritten[0], "existing");

        // 验证 registry 中的账号数量
        let registry = service.load_registry().unwrap();
        assert_eq!(registry.accounts.len(), 2);

        // 验证 existing 账号已被更新
        let existing = registry.accounts.get("existing").unwrap();
        assert_eq!(existing.account_id, "new-id");
        assert_eq!(
            existing.description,
            Some("Updated description".to_string())
        );
    }

    #[test]
    fn test_import_accounts_force_creates_backups() {
        let (service, _ccr, _codex) = create_test_service();

        let auth_path = service.codex_dir.join("auth.json");
        let auth_content = create_test_auth_json("existing-id", "2026-01-08T03:09:53.894843900Z");
        fs::write(&auth_path, auth_content).unwrap();
        service
            .save_current("existing", Some("Existing account".to_string()), false)
            .unwrap();

        let import_json = r#"{
            "version": "1.0",
            "exported_at": "2026-01-22T00:00:00Z",
            "accounts": {
                "existing": {
                    "description": "Updated description",
                    "account_id": "new-id",
                    "email": "new***@example.com",
                    "saved_at": "2026-01-22T00:00:00Z",
                    "auth_data": {
                        "tokens": {
                            "id_token": "new_token",
                            "access_token": "new_access",
                            "refresh_token": "new_refresh",
                            "account_id": "new-id"
                        },
                        "last_refresh": "2026-01-22T00:00:00Z"
                    }
                }
            }
        }"#;

        service
            .import_accounts(import_json, ImportMode::Merge, true)
            .unwrap();

        let stored_auth_path = service.account_auth_path("existing");
        let stored_auth = fs::read_to_string(&stored_auth_path).unwrap();
        assert!(stored_auth.contains("new_access"));

        let backup_dir = service.backup_dir();
        let backups: Vec<_> = fs::read_dir(&backup_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();

        let has_registry_backup = backups.iter().any(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with("auth_registry_")
        });
        let has_account_backup = backups.iter().any(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with("auth_account_existing_")
        });

        assert!(has_registry_backup);
        assert!(has_account_backup);
    }

    #[test]
    fn test_import_accounts_replace_mode() {
        let (service, _ccr, _codex) = create_test_service();

        // 先保存一个账号
        let auth_path = service.codex_dir.join("auth.json");
        let auth_content = create_test_auth_json("existing-id", "2026-01-08T03:09:53.894843900Z");
        fs::write(&auth_path, auth_content).unwrap();
        service
            .save_current("existing", Some("Existing account".to_string()), false)
            .unwrap();

        // 准备导入数据
        let import_json = r#"{
            "version": "1.0",
            "exported_at": "2026-01-22T00:00:00Z",
            "accounts": {
                "existing": {
                    "description": "Replaced description",
                    "account_id": "replaced-id",
                    "email": "rep***@example.com",
                    "saved_at": "2026-01-22T00:00:00Z"
                }
            }
        }"#;

        // 替换模式（force 参数在 Replace 模式下被忽略）
        let result = service
            .import_accounts(import_json, ImportMode::Replace, false)
            .unwrap();

        // 验证结果
        assert_eq!(result.added, 0);
        assert_eq!(result.updated, 1);
        assert_eq!(result.skipped, 0);
        assert_eq!(result.overwritten.len(), 1);
        assert_eq!(result.overwritten[0], "existing");

        // 验证账号已被替换 - 从 registry 读取
        let registry = service.load_registry().unwrap();
        let existing = registry.accounts.get("existing").unwrap();
        assert_eq!(existing.account_id, "replaced-id");
        assert_eq!(
            existing.description,
            Some("Replaced description".to_string())
        );
    }

    #[test]
    fn test_import_accounts_invalid_name() {
        let (service, _ccr, _codex) = create_test_service();

        // 准备包含无效账号名的导入数据
        let import_json = r#"{
            "version": "1.0",
            "exported_at": "2026-01-22T00:00:00Z",
            "accounts": {
                "invalid name": {
                    "description": "Invalid",
                    "account_id": "test-id",
                    "email": "test***@example.com",
                    "saved_at": "2026-01-22T00:00:00Z"
                }
            }
        }"#;

        // 应该返回错误
        let result = service.import_accounts(import_json, ImportMode::Merge, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_import_accounts_invalid_json() {
        let (service, _ccr, _codex) = create_test_service();

        // 无效的 JSON
        let invalid_json = "{ invalid json }";

        // 应该返回错误
        let result = service.import_accounts(invalid_json, ImportMode::Merge, false);
        assert!(result.is_err());
    }
}
