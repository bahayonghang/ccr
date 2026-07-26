// 🔐 OpenCode Auth 服务层
// 管理 OpenCode 的 openai provider 多账号手动切换

use crate::models::{
    CodexAuthJson, CodexAuthRegistry, CodexToOpenCodeMigrationItem, CodexToOpenCodeMigrationReport,
    CodexToOpenCodeMigrationStatus, OpenAiAuthMethod, OpenCodeAuthAccount, OpenCodeAuthItem,
    OpenCodeAuthRegistry, OpenCodeCurrentAuthInfo, OpenCodeLoginState, OpenCodeOpenAiAuth,
    OpenCodeReadSnapshot,
};
use crate::utils::{CodexPaths, OpenCodePaths, decode_base64url, ensure_private_permissions};
use ccr_core::core::atomic_writer::AtomicWriter;
use ccr_core::core::error::{CcrError, Result};
use chrono::{DateTime, TimeZone, Utc};
use serde_json::{Map as JsonMap, Value as JsonValue};
use std::fs;
use std::path::{Path, PathBuf};

use super::openai_quota_core::normalize_openai_plan;

/// OpenCode Auth 服务
pub struct OpenCodeAuthService {
    /// CCR 平台数据目录 (~/.ccr/platforms/opencode/)
    ccr_opencode_dir: PathBuf,
    /// OpenCode 数据目录（官方默认：`$HOME/.local/share/opencode/`）
    opencode_dir: PathBuf,
    /// 测试注入的 Codex CCR 数据目录
    codex_ccr_dir_override: Option<PathBuf>,
}

impl OpenCodeAuthService {
    /// 创建新的 OpenCodeAuthService 实例
    pub fn new() -> Result<Self> {
        let paths = OpenCodePaths::resolve()?;
        Ok(Self {
            ccr_opencode_dir: paths.ccr_opencode_dir,
            opencode_dir: paths.opencode_dir,
            codex_ccr_dir_override: None,
        })
    }

    pub fn from_dirs(ccr_opencode_dir: PathBuf, opencode_dir: PathBuf) -> Self {
        Self {
            ccr_opencode_dir,
            opencode_dir,
            codex_ccr_dir_override: None,
        }
    }

    pub fn from_dirs_with_codex(
        ccr_opencode_dir: PathBuf,
        opencode_dir: PathBuf,
        codex_ccr_dir: PathBuf,
    ) -> Self {
        Self {
            ccr_opencode_dir,
            opencode_dir,
            codex_ccr_dir_override: Some(codex_ccr_dir),
        }
    }

    pub fn opencode_dir(&self) -> &Path {
        &self.opencode_dir
    }

    fn auth_json_path(&self) -> PathBuf {
        self.opencode_dir.join("auth.json")
    }

    fn auth_storage_dir(&self) -> PathBuf {
        self.ccr_opencode_dir.join("auth")
    }

    fn registry_path(&self) -> PathBuf {
        self.ccr_opencode_dir.join("auth_registry.toml")
    }

    fn account_auth_path(&self, name: &str) -> PathBuf {
        self.auth_storage_dir().join(format!("{name}.json"))
    }

    fn ensure_storage_dirs(&self) -> Result<()> {
        fs::create_dir_all(&self.ccr_opencode_dir)
            .map_err(|e| CcrError::ConfigError(format!("创建 OpenCode 平台目录失败: {e}")))?;
        fs::create_dir_all(self.auth_storage_dir())
            .map_err(|e| CcrError::ConfigError(format!("创建 OpenCode auth 目录失败: {e}")))?;
        Ok(())
    }

    fn load_auth_root_map(&self) -> Result<JsonMap<String, JsonValue>> {
        let auth_path = self.auth_json_path();
        let content = fs::read_to_string(&auth_path)
            .map_err(|e| CcrError::ConfigError(format!("读取 OpenCode auth.json 失败: {e}")))?;
        serde_json::from_str::<JsonMap<String, JsonValue>>(&content)
            .map_err(|e| CcrError::ConfigError(format!("解析 OpenCode auth.json 失败: {e}")))
    }

    fn load_auth_root_map_or_default(&self) -> Result<JsonMap<String, JsonValue>> {
        let auth_path = self.auth_json_path();
        if !auth_path.exists() {
            return Ok(JsonMap::new());
        }
        self.load_auth_root_map()
    }

    fn current_openai_value(&self) -> Result<Option<JsonValue>> {
        let auth_path = self.auth_json_path();
        if !auth_path.exists() {
            return Ok(None);
        }

        let root = self.load_auth_root_map()?;
        Ok(root.get("openai").cloned().filter(|value| !value.is_null()))
    }

    /// 读取当前 OpenCode 登录快照
    pub fn read_auth_snapshot(&self) -> Result<OpenCodeReadSnapshot> {
        let registry = self.load_registry()?;
        let current_value = self.current_openai_value()?;
        let current_info = match current_value {
            Some(ref openai) => Some(self.extract_current_auth_info(openai)?),
            None => None,
        };
        let current_account_name =
            Self::matched_saved_account_name(&registry, current_info.as_ref());
        let login_state = match (current_info.as_ref(), current_account_name.as_ref()) {
            (Some(_), Some(name)) => OpenCodeLoginState::LoggedInSaved(name.clone()),
            (Some(_), None) => OpenCodeLoginState::LoggedInUnsaved,
            (None, _) => OpenCodeLoginState::NotLoggedIn,
        };

        Ok(OpenCodeReadSnapshot {
            login_state,
            current_info,
            registry,
            current_account_name,
        })
    }

    /// 构建 TUI 展示列表
    pub fn build_account_items(
        &self,
        snapshot: &OpenCodeReadSnapshot,
    ) -> Result<Vec<OpenCodeAuthItem>> {
        let mut items = Vec::new();

        if let Some(info) = snapshot.current_info.as_ref()
            && snapshot.current_account_name.is_none()
        {
            items.push(OpenCodeAuthItem {
                name: "current-login".to_string(),
                account_id: Some(info.account_id.clone()),
                email: info.email.as_ref().map(|email| self.mask_email(email)),
                plan_type: info.plan_type.clone(),
                is_current: true,
                is_virtual: true,
                saved_at: None,
                last_used: None,
                expires_at: info.expires_at,
            });
        }

        let mut saved_items: Vec<_> = snapshot
            .registry
            .accounts
            .iter()
            .map(|(name, account)| {
                let is_current = snapshot.current_account_name.as_deref() == Some(name.as_str());
                OpenCodeAuthItem {
                    name: name.clone(),
                    account_id: Some(account.account_id.clone()),
                    email: account.email.clone(),
                    plan_type: account
                        .plan_type
                        .clone()
                        .or_else(|| self.load_saved_account_plan_type(name)),
                    is_current,
                    is_virtual: false,
                    saved_at: Some(account.saved_at),
                    last_used: account.last_used,
                    expires_at: account.expires_at,
                }
            })
            .collect();

        saved_items.sort_by(|left, right| {
            right
                .is_current
                .cmp(&left.is_current)
                .then_with(|| left.name.cmp(&right.name))
        });

        items.extend(saved_items);
        Ok(items)
    }

    /// 保存当前 openai 登录为一个可切换账号
    pub fn save_current(&self, name: &str, force: bool) -> Result<()> {
        self.validate_account_name(name)?;
        self.ensure_storage_dirs()?;

        let current_value = self
            .current_openai_value()?
            .ok_or_else(|| CcrError::ConfigError("当前 OpenCode 未检测到 openai 登录".into()))?;
        let current_info = self.extract_current_auth_info(&current_value)?;

        let mut registry = self.load_registry()?;
        if registry.accounts.contains_key(name) && !force {
            return Err(CcrError::ResourceAlreadyExists(format!(
                "OpenCode auth account '{name}'"
            )));
        }

        let now = Utc::now();
        let saved_at = registry
            .accounts
            .get(name)
            .map(|account| account.saved_at)
            .unwrap_or(now);

        registry.accounts.insert(
            name.to_string(),
            OpenCodeAuthAccount {
                account_id: current_info.account_id.clone(),
                email: current_info
                    .email
                    .as_ref()
                    .map(|email| self.mask_email(email)),
                plan_type: current_info.plan_type.clone(),
                saved_at,
                last_used: Some(now),
                expires_at: current_info.expires_at,
            },
        );
        registry.current_auth = Some(name.to_string());

        self.write_account_snapshot(name, &current_value)?;
        self.save_registry(&registry)
    }

    /// 从已保存的 Codex Auth 账号导入可兼容的 OpenAI OAuth 账号
    pub fn import_saved_codex_accounts(
        &self,
        dry_run: bool,
    ) -> Result<CodexToOpenCodeMigrationReport> {
        let codex_ccr_dir = self.resolve_codex_ccr_dir()?;
        self.import_saved_codex_accounts_from_dir(&codex_ccr_dir, dry_run)
    }

    fn import_saved_codex_accounts_from_dir(
        &self,
        codex_ccr_dir: &Path,
        dry_run: bool,
    ) -> Result<CodexToOpenCodeMigrationReport> {
        let source_registry = Self::load_codex_registry(codex_ccr_dir)?;
        let mut target_registry = self.load_registry()?;
        let preserved_current_auth = target_registry.current_auth.clone();
        let mut report = CodexToOpenCodeMigrationReport {
            dry_run,
            ..Default::default()
        };
        let mut pending_snapshots: Vec<(String, JsonValue)> = Vec::new();

        for (name, account) in &source_registry.accounts {
            if matches!(account.auth_method, Some(OpenAiAuthMethod::Api))
                || account.account_id.starts_with("provider:")
            {
                report.skipped_incompatible_auth += 1;
                report.outcomes.push(CodexToOpenCodeMigrationItem {
                    name: name.clone(),
                    status: CodexToOpenCodeMigrationStatus::SkippedIncompatibleAuth,
                    account_id: Some(account.account_id.clone()),
                    message: "仅支持 ChatGPT OAuth 账号，API key / provider 账号不会迁移"
                        .to_string(),
                });
                continue;
            }

            let snapshot_path = Self::codex_account_auth_path(codex_ccr_dir, name);
            if !snapshot_path.exists() {
                report.skipped_missing_snapshot += 1;
                report.outcomes.push(CodexToOpenCodeMigrationItem {
                    name: name.clone(),
                    status: CodexToOpenCodeMigrationStatus::SkippedMissingSnapshot,
                    account_id: Some(account.account_id.clone()),
                    message: format!("缺少 Codex 账号快照: {}", snapshot_path.display()),
                });
                continue;
            }

            let raw_content = match fs::read_to_string(&snapshot_path) {
                Ok(content) => content,
                Err(err) => {
                    report.skipped_invalid_snapshot += 1;
                    report.outcomes.push(CodexToOpenCodeMigrationItem {
                        name: name.clone(),
                        status: CodexToOpenCodeMigrationStatus::SkippedInvalidSnapshot,
                        account_id: Some(account.account_id.clone()),
                        message: format!("读取 Codex 账号快照失败: {err}"),
                    });
                    continue;
                }
            };

            let auth: CodexAuthJson = match serde_json::from_str(&raw_content) {
                Ok(auth) => auth,
                Err(err) => {
                    report.skipped_invalid_snapshot += 1;
                    report.outcomes.push(CodexToOpenCodeMigrationItem {
                        name: name.clone(),
                        status: CodexToOpenCodeMigrationStatus::SkippedInvalidSnapshot,
                        account_id: Some(account.account_id.clone()),
                        message: format!("解析 Codex 账号快照失败: {err}"),
                    });
                    continue;
                }
            };

            let (openai_value, info) = match self.build_openai_snapshot_from_codex(account, &auth) {
                Ok(result) => result,
                Err(err) => {
                    let status = if auth
                        .openai_api_key
                        .as_deref()
                        .map(str::trim)
                        .is_some_and(|value| !value.is_empty())
                        && auth.tokens.is_none()
                    {
                        report.skipped_incompatible_auth += 1;
                        CodexToOpenCodeMigrationStatus::SkippedIncompatibleAuth
                    } else {
                        report.skipped_invalid_snapshot += 1;
                        CodexToOpenCodeMigrationStatus::SkippedInvalidSnapshot
                    };
                    report.outcomes.push(CodexToOpenCodeMigrationItem {
                        name: name.clone(),
                        status,
                        account_id: Some(account.account_id.clone()),
                        message: err.to_string(),
                    });
                    continue;
                }
            };

            if target_registry.accounts.contains_key(name) {
                report.skipped_existing_name += 1;
                report.outcomes.push(CodexToOpenCodeMigrationItem {
                    name: name.clone(),
                    status: CodexToOpenCodeMigrationStatus::SkippedExistingName,
                    account_id: Some(info.account_id.clone()),
                    message: format!("OpenCode 已存在同名账号 '{name}'"),
                });
                continue;
            }

            if let Some((existing_name, _)) = target_registry
                .accounts
                .iter()
                .find(|(_, existing)| existing.account_id == info.account_id)
            {
                report.skipped_existing_account_id += 1;
                report.outcomes.push(CodexToOpenCodeMigrationItem {
                    name: name.clone(),
                    status: CodexToOpenCodeMigrationStatus::SkippedExistingAccountId,
                    account_id: Some(info.account_id.clone()),
                    message: format!("OpenCode 已存在相同 account_id 的账号 '{}'", existing_name),
                });
                continue;
            }

            report.imported += 1;
            report.outcomes.push(CodexToOpenCodeMigrationItem {
                name: name.clone(),
                status: CodexToOpenCodeMigrationStatus::Imported,
                account_id: Some(info.account_id.clone()),
                message: if dry_run {
                    "可导入到 OpenCode".to_string()
                } else {
                    "已导入到 OpenCode".to_string()
                },
            });

            target_registry.accounts.insert(
                name.clone(),
                OpenCodeAuthAccount {
                    account_id: info.account_id.clone(),
                    email: info
                        .email
                        .as_deref()
                        .map(|email| self.mask_email(email))
                        .or_else(|| account.email.clone()),
                    plan_type: info.plan_type.clone().or_else(|| account.plan_type.clone()),
                    saved_at: account.saved_at,
                    last_used: account.last_used,
                    expires_at: info.expires_at.or(account.expires_at),
                },
            );
            pending_snapshots.push((name.clone(), openai_value));
        }

        debug_assert_eq!(target_registry.current_auth, preserved_current_auth);

        if dry_run || pending_snapshots.is_empty() {
            return Ok(report);
        }

        let mut written_paths = Vec::new();
        for (name, snapshot) in &pending_snapshots {
            if let Err(err) = self.write_account_snapshot(name, snapshot) {
                for path in written_paths {
                    let _ = fs::remove_file(path);
                }
                return Err(err);
            }
            written_paths.push(self.account_auth_path(name));
        }

        if let Err(err) = self.save_registry(&target_registry) {
            for path in written_paths {
                let _ = fs::remove_file(path);
            }
            return Err(err);
        }

        Ok(report)
    }

    /// 切换到指定已保存账号
    pub fn switch_account(&self, name: &str) -> Result<()> {
        let mut registry = self.load_registry()?;
        let _account =
            registry.accounts.get(name).cloned().ok_or_else(|| {
                CcrError::ResourceNotFound(format!("OpenCode auth account '{name}'"))
            })?;

        let snapshot_path = self.account_auth_path(name);
        if !snapshot_path.exists() {
            return Err(CcrError::ResourceNotFound(format!(
                "OpenCode auth snapshot '{}'",
                snapshot_path.display()
            )));
        }

        let snapshot_content = fs::read_to_string(&snapshot_path)
            .map_err(|e| CcrError::ConfigError(format!("读取 OpenCode 账号快照失败: {e}")))?;
        let saved_openai: JsonValue = serde_json::from_str(&snapshot_content)
            .map_err(|e| CcrError::ConfigError(format!("解析 OpenCode 账号快照失败: {e}")))?;

        let mut root = self.load_auth_root_map_or_default()?;
        root.insert("openai".to_string(), saved_openai);
        self.write_auth_root_map(&root)?;

        registry.current_auth = Some(name.to_string());
        if let Some(account) = registry.accounts.get_mut(name) {
            account.last_used = Some(Utc::now());
        }
        self.save_registry(&registry)
    }

    /// 删除指定已保存账号
    pub fn delete_account(&self, name: &str) -> Result<()> {
        let mut registry = self.load_registry()?;
        if registry.accounts.shift_remove(name).is_none() {
            return Err(CcrError::ResourceNotFound(format!(
                "OpenCode auth account '{name}'"
            )));
        }

        if registry.current_auth.as_deref() == Some(name) {
            registry.current_auth = None;
        }

        let snapshot_path = self.account_auth_path(name);
        if snapshot_path.exists() {
            fs::remove_file(&snapshot_path)
                .map_err(|e| CcrError::ConfigError(format!("删除 OpenCode 账号快照失败: {e}")))?;
        }

        self.save_registry(&registry)
    }

    /// 加载注册表
    pub fn load_registry(&self) -> Result<OpenCodeAuthRegistry> {
        let registry_path = self.registry_path();
        if !registry_path.exists() {
            return Ok(OpenCodeAuthRegistry::default());
        }

        let content = fs::read_to_string(&registry_path)
            .map_err(|e| CcrError::ConfigError(format!("读取 OpenCode 注册表失败: {e}")))?;
        toml::from_str(&content)
            .map_err(|e| CcrError::ConfigError(format!("解析 OpenCode 注册表失败: {e}")))
    }

    fn save_registry(&self, registry: &OpenCodeAuthRegistry) -> Result<()> {
        self.ensure_storage_dirs()?;
        let content = toml::to_string_pretty(registry)
            .map_err(|e| CcrError::ConfigError(format!("序列化 OpenCode 注册表失败: {e}")))?;
        let path = self.registry_path();
        AtomicWriter::new(&path)
            .secret(true)
            .write_string(&content)
            .map_err(|e| CcrError::ConfigError(format!("写入 OpenCode 注册表失败: {e}")))?;
        ensure_private_permissions(&path);
        Ok(())
    }

    fn load_saved_account_plan_type(&self, name: &str) -> Option<String> {
        let content = fs::read_to_string(self.account_auth_path(name)).ok()?;
        let value: JsonValue = serde_json::from_str(&content).ok()?;
        self.extract_current_auth_info(&value).ok()?.plan_type
    }

    fn write_account_snapshot(&self, name: &str, openai: &JsonValue) -> Result<()> {
        self.ensure_storage_dirs()?;
        let content = serde_json::to_string_pretty(openai)
            .map_err(|e| CcrError::ConfigError(format!("序列化 OpenCode 账号快照失败: {e}")))?;
        let path = self.account_auth_path(name);
        AtomicWriter::new(&path)
            .secret(true)
            .write_string(&content)
            .map_err(|e| CcrError::ConfigError(format!("写入 OpenCode 账号快照失败: {e}")))?;
        ensure_private_permissions(&path);
        Ok(())
    }

    fn write_auth_root_map(&self, root: &JsonMap<String, JsonValue>) -> Result<()> {
        if let Some(parent) = self.auth_json_path().parent() {
            fs::create_dir_all(parent)
                .map_err(|e| CcrError::ConfigError(format!("创建 OpenCode 配置目录失败: {e}")))?;
        }

        let content = serde_json::to_string_pretty(root)
            .map_err(|e| CcrError::ConfigError(format!("序列化 OpenCode auth.json 失败: {e}")))?;
        let path = self.auth_json_path();
        AtomicWriter::new(&path)
            .secret(true)
            .write_string(&content)
            .map_err(|e| CcrError::ConfigError(format!("写入 OpenCode auth.json 失败: {e}")))?;
        ensure_private_permissions(&path);
        Ok(())
    }

    fn resolve_codex_ccr_dir(&self) -> Result<PathBuf> {
        if let Some(path) = &self.codex_ccr_dir_override {
            return Ok(path.clone());
        }
        Ok(CodexPaths::resolve()?.ccr_codex_dir)
    }

    fn load_codex_registry(codex_ccr_dir: &Path) -> Result<CodexAuthRegistry> {
        let registry_path = codex_ccr_dir.join("auth_registry.toml");
        if !registry_path.exists() {
            return Ok(CodexAuthRegistry::default());
        }

        let content = fs::read_to_string(&registry_path)
            .map_err(|e| CcrError::ConfigError(format!("读取 Codex 注册表失败: {e}")))?;
        toml::from_str(&content)
            .map_err(|e| CcrError::ConfigError(format!("解析 Codex 注册表失败: {e}")))
    }

    fn codex_account_auth_path(codex_ccr_dir: &Path, name: &str) -> PathBuf {
        codex_ccr_dir.join("auth").join(format!("{name}.json"))
    }

    fn build_openai_snapshot_from_codex(
        &self,
        account: &crate::models::CodexAuthAccount,
        auth: &CodexAuthJson,
    ) -> Result<(JsonValue, OpenCodeCurrentAuthInfo)> {
        let tokens = auth.tokens.as_ref().ok_or_else(|| {
            CcrError::ConfigError("Codex 账号快照缺少 OAuth tokens，无法迁移".into())
        })?;

        let access_token = Self::trimmed_required(tokens.access_token.as_deref(), "access_token")?;
        let refresh_token =
            Self::trimmed_required(tokens.refresh_token.as_deref(), "refresh_token")?;
        let account_id = tokens
            .account_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .or_else(|| {
                let value = account.account_id.trim();
                (!value.is_empty()).then(|| value.to_string())
            })
            .ok_or_else(|| CcrError::ConfigError("Codex 账号快照缺少 account_id".into()))?;

        let access_claims = Self::decode_jwt_claims(&access_token);
        let id_claims = tokens.id_token.as_deref().and_then(Self::decode_jwt_claims);
        let email = Self::claim_string(id_claims.as_ref(), "email")
            .or_else(|| Self::claim_string(access_claims.as_ref(), "email"));
        let plan = Self::claim_string(access_claims.as_ref(), "chatgpt_plan_type")
            .or_else(|| Self::claim_string(access_claims.as_ref(), "plan"))
            .or_else(|| Self::claim_string(id_claims.as_ref(), "chatgpt_plan_type"))
            .or_else(|| Self::claim_string(id_claims.as_ref(), "plan"))
            .map(|value| normalize_openai_plan(&value))
            .filter(|value| !value.is_empty());
        let expires = Self::claim_i64(access_claims.as_ref(), "exp")
            .map(Self::normalize_unix_timestamp_millis);

        let mut raw = serde_json::Map::new();
        raw.insert("type".to_string(), JsonValue::String("oauth".to_string()));
        raw.insert("access".to_string(), JsonValue::String(access_token));
        raw.insert("refresh".to_string(), JsonValue::String(refresh_token));
        raw.insert("accountId".to_string(), JsonValue::String(account_id));

        if let Some(expires) = expires {
            raw.insert("expires".to_string(), JsonValue::Number(expires.into()));
        }
        if let Some(email) = email {
            raw.insert("email".to_string(), JsonValue::String(email));
        }
        if let Some(plan) = plan {
            raw.insert("plan".to_string(), JsonValue::String(plan));
        }

        let openai_value = JsonValue::Object(raw);
        let info = self.extract_current_auth_info(&openai_value)?;
        Ok((openai_value, info))
    }

    fn extract_current_auth_info(&self, openai: &JsonValue) -> Result<OpenCodeCurrentAuthInfo> {
        let auth: OpenCodeOpenAiAuth = serde_json::from_value(openai.clone()).map_err(|e| {
            CcrError::ConfigError(format!("解析 OpenCode openai provider 失败: {e}"))
        })?;

        let claims = auth.access.as_deref().and_then(Self::decode_jwt_claims);
        let account_id = auth
            .account_id
            .clone()
            .or_else(|| Self::claim_string(claims.as_ref(), "chatgpt_account_id"))
            .or_else(|| Self::claim_string(claims.as_ref(), "account_id"))
            .or_else(|| Self::claim_string(claims.as_ref(), "sub"))
            .ok_or_else(|| CcrError::ConfigError("当前 OpenCode 登录缺少 account_id".into()))?;

        let email = auth
            .extra
            .get("email")
            .and_then(JsonValue::as_str)
            .map(str::to_string)
            .or_else(|| Self::claim_string(claims.as_ref(), "email"));

        let plan_type = auth
            .extra
            .get("plan")
            .and_then(JsonValue::as_str)
            .map(Self::normalize_plan)
            .or_else(|| {
                auth.extra
                    .get("planType")
                    .and_then(JsonValue::as_str)
                    .map(Self::normalize_plan)
            })
            .or_else(|| {
                Self::claim_string(claims.as_ref(), "chatgpt_plan_type")
                    .map(|value| Self::normalize_plan(value.as_str()))
            })
            .or_else(|| {
                Self::claim_string(claims.as_ref(), "plan")
                    .map(|value| Self::normalize_plan(value.as_str()))
            });

        let expires_at = auth.expires.and_then(Self::unix_millis_to_datetime);
        Ok(OpenCodeCurrentAuthInfo {
            account_id,
            email,
            plan_type,
            expires_at,
        })
    }

    fn matched_saved_account_name(
        registry: &OpenCodeAuthRegistry,
        current_info: Option<&OpenCodeCurrentAuthInfo>,
    ) -> Option<String> {
        let info = current_info?;
        registry.accounts.iter().find_map(|(name, account)| {
            (account.account_id == info.account_id).then(|| name.clone())
        })
    }

    fn decode_jwt_claims(token: &str) -> Option<JsonValue> {
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return None;
        }

        let decoded = decode_base64url(parts[1])?;
        let payload = String::from_utf8(decoded).ok()?;
        serde_json::from_str(&payload).ok()
    }

    fn claim_string(claims: Option<&JsonValue>, key: &str) -> Option<String> {
        claims?
            .get(key)
            .and_then(JsonValue::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
    }

    fn claim_i64(claims: Option<&JsonValue>, key: &str) -> Option<i64> {
        let value = claims?.get(key)?;
        value.as_i64().or_else(|| {
            value
                .as_str()
                .and_then(|raw| raw.trim().parse::<i64>().ok())
        })
    }

    fn normalize_unix_timestamp_millis(value: i64) -> i64 {
        if value >= 1_000_000_000_000 {
            value
        } else {
            value.saturating_mul(1000)
        }
    }

    fn trimmed_required(value: Option<&str>, field: &str) -> Result<String> {
        value
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .ok_or_else(|| {
                CcrError::ConfigError(format!("Codex 账号快照缺少有效的 {field}，无法迁移"))
            })
    }

    fn unix_millis_to_datetime(value: i64) -> Option<DateTime<Utc>> {
        Utc.timestamp_millis_opt(value).single()
    }

    fn normalize_plan(plan: &str) -> String {
        normalize_openai_plan(plan)
    }

    /// 验证账号名称
    fn validate_account_name(&self, name: &str) -> Result<()> {
        if name.is_empty() {
            return Err(CcrError::ValidationError("账号名称不能为空".into()));
        }

        if matches!(name, "default" | "current-login") {
            return Err(CcrError::ValidationError(format!(
                "'{name}' 是保留名称，请使用其他名称"
            )));
        }

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

    /// 邮箱脱敏
    pub fn mask_email(&self, email: &str) -> String {
        if let Some(at_pos) = email.find('@') {
            let local = &email[..at_pos];
            let domain = &email[at_pos..];

            if local.len() <= 3 {
                format!("{local}***{domain}")
            } else {
                format!("{}***{domain}", &local[..3])
            }
        } else {
            email.to_string()
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use base64::Engine;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use chrono::Duration;
    use serde_json::json;
    use tempfile::tempdir;

    fn create_test_service() -> (OpenCodeAuthService, tempfile::TempDir, tempfile::TempDir) {
        let ccr = tempdir().unwrap();
        let opencode = tempdir().unwrap();
        (
            OpenCodeAuthService::from_dirs(
                ccr.path().join("platforms").join("opencode"),
                opencode.path().to_path_buf(),
            ),
            ccr,
            opencode,
        )
    }

    fn create_migration_test_service() -> (
        OpenCodeAuthService,
        tempfile::TempDir,
        tempfile::TempDir,
        PathBuf,
    ) {
        let ccr = tempdir().unwrap();
        let opencode = tempdir().unwrap();
        let codex_ccr_dir = ccr.path().join("platforms").join("codex");
        (
            OpenCodeAuthService::from_dirs_with_codex(
                ccr.path().join("platforms").join("opencode"),
                opencode.path().to_path_buf(),
                codex_ccr_dir.clone(),
            ),
            ccr,
            opencode,
            codex_ccr_dir,
        )
    }

    fn fake_jwt(payload: JsonValue) -> String {
        let header = URL_SAFE_NO_PAD.encode(r#"{"alg":"none","typ":"JWT"}"#);
        let payload = URL_SAFE_NO_PAD.encode(payload.to_string());
        format!("{header}.{payload}.signature")
    }

    fn fake_access_token(email: &str, account_id: &str, plan: &str) -> String {
        fake_jwt(json!({
            "email": email,
            "chatgpt_account_id": account_id,
            "chatgpt_plan_type": plan
        }))
    }

    fn sample_codex_auth_json(
        email: &str,
        account_id: &str,
        plan: &str,
        expires_at: DateTime<Utc>,
    ) -> CodexAuthJson {
        CodexAuthJson {
            openai_api_key: None,
            tokens: Some(crate::models::CodexAuthTokens {
                id_token: Some(fake_jwt(json!({
                    "email": email
                }))),
                access_token: Some(fake_jwt(json!({
                    "email": email,
                    "chatgpt_account_id": account_id,
                    "chatgpt_plan_type": plan,
                    "exp": expires_at.timestamp()
                }))),
                refresh_token: Some(format!("rt_{account_id}")),
                account_id: Some(account_id.to_string()),
            }),
            last_refresh: Some(Utc::now().to_rfc3339()),
        }
    }

    fn write_codex_registry(
        codex_ccr_dir: &Path,
        name: &str,
        account: crate::models::CodexAuthAccount,
    ) {
        std::fs::create_dir_all(codex_ccr_dir.join("auth")).unwrap();
        let mut registry = CodexAuthRegistry::default();
        registry.accounts.insert(name.to_string(), account);
        std::fs::write(
            codex_ccr_dir.join("auth_registry.toml"),
            toml::to_string_pretty(&registry).unwrap(),
        )
        .unwrap();
    }

    fn write_codex_snapshot(codex_ccr_dir: &Path, name: &str, auth: &CodexAuthJson) {
        std::fs::create_dir_all(codex_ccr_dir.join("auth")).unwrap();
        std::fs::write(
            codex_ccr_dir.join("auth").join(format!("{name}.json")),
            serde_json::to_string_pretty(auth).unwrap(),
        )
        .unwrap();
    }

    fn sample_openai_provider(
        email: &str,
        account_id: &str,
        plan: &str,
        expires: i64,
    ) -> JsonValue {
        json!({
            "type": "oauth",
            "access": fake_access_token(email, account_id, plan),
            "refresh": format!("rt_{account_id}"),
            "expires": expires,
            "accountId": account_id,
            "plan": plan.to_ascii_lowercase()
        })
    }

    fn write_auth_json(service: &OpenCodeAuthService, openai: JsonValue, extra: JsonValue) {
        let mut root = JsonMap::new();
        root.insert("openai".to_string(), openai);
        if let Some(extra_map) = extra.as_object() {
            for (key, value) in extra_map {
                root.insert(key.clone(), value.clone());
            }
        }
        service.write_auth_root_map(&root).unwrap();
    }

    #[test]
    fn read_auth_snapshot_extracts_email_and_plan_from_access_token() {
        let (service, _ccr, _opencode) = create_test_service();
        let expires = (Utc::now() + Duration::days(7)).timestamp_millis();
        write_auth_json(
            &service,
            sample_openai_provider("user@example.com", "acc-1", "plus", expires),
            json!({}),
        );

        let snapshot = service.read_auth_snapshot().unwrap();
        let info = snapshot.current_info.unwrap();
        assert_eq!(info.account_id, "acc-1");
        assert_eq!(info.email.as_deref(), Some("user@example.com"));
        assert_eq!(info.plan_type.as_deref(), Some("plus"));
        assert_eq!(snapshot.login_state, OpenCodeLoginState::LoggedInUnsaved);
    }

    #[test]
    fn save_current_persists_registry_and_snapshot() {
        let (service, _ccr, _opencode) = create_test_service();
        let expires = (Utc::now() + Duration::days(7)).timestamp_millis();
        write_auth_json(
            &service,
            sample_openai_provider("user@example.com", "acc-1", "plus", expires),
            json!({}),
        );

        service.save_current("work", false).unwrap();

        let registry = service.load_registry().unwrap();
        assert_eq!(registry.current_auth.as_deref(), Some("work"));
        assert!(registry.accounts.contains_key("work"));
        assert_eq!(
            registry.accounts["work"].email.as_deref(),
            Some("use***@example.com")
        );
        assert_eq!(registry.accounts["work"].plan_type.as_deref(), Some("plus"));
        assert!(service.account_auth_path("work").exists());

        let snapshot = service.read_auth_snapshot().unwrap();
        assert_eq!(
            snapshot.login_state,
            OpenCodeLoginState::LoggedInSaved("work".to_string())
        );
    }

    #[test]
    fn switch_account_keeps_other_providers_intact() {
        let (service, _ccr, _opencode) = create_test_service();
        let expires = (Utc::now() + Duration::days(7)).timestamp_millis();
        write_auth_json(
            &service,
            sample_openai_provider("one@example.com", "acc-1", "plus", expires),
            json!({
                "github": {
                    "type": "oauth",
                    "token": "gh-token"
                }
            }),
        );
        service.save_current("primary", false).unwrap();

        write_auth_json(
            &service,
            sample_openai_provider("two@example.com", "acc-2", "pro", expires),
            json!({
                "github": {
                    "type": "oauth",
                    "token": "gh-token"
                }
            }),
        );

        service.switch_account("primary").unwrap();

        let root = service.load_auth_root_map().unwrap();
        assert_eq!(
            root.get("github")
                .and_then(|value| value.get("token"))
                .and_then(JsonValue::as_str),
            Some("gh-token")
        );
        assert_eq!(
            root.get("openai")
                .and_then(|value| value.get("accountId"))
                .and_then(JsonValue::as_str),
            Some("acc-1")
        );
    }

    #[test]
    fn switch_account_allows_saved_account_with_legacy_expiry() {
        let (service, _ccr, _opencode) = create_test_service();
        let future = (Utc::now() + Duration::days(7)).timestamp_millis();
        write_auth_json(
            &service,
            sample_openai_provider("one@example.com", "acc-1", "plus", future),
            json!({}),
        );
        service.save_current("expired", false).unwrap();

        let mut registry = service.load_registry().unwrap();
        registry.accounts.get_mut("expired").unwrap().expires_at =
            Some(Utc::now() - Duration::hours(1));
        service.save_registry(&registry).unwrap();

        service.switch_account("expired").unwrap();

        let registry = service.load_registry().unwrap();
        assert_eq!(registry.current_auth.as_deref(), Some("expired"));
    }

    #[test]
    fn delete_account_removes_registry_and_snapshot() {
        let (service, _ccr, _opencode) = create_test_service();
        let expires = (Utc::now() + Duration::days(7)).timestamp_millis();
        write_auth_json(
            &service,
            sample_openai_provider("user@example.com", "acc-1", "plus", expires),
            json!({}),
        );
        service.save_current("work", false).unwrap();

        service.delete_account("work").unwrap();

        let registry = service.load_registry().unwrap();
        assert!(!registry.accounts.contains_key("work"));
        assert!(!service.account_auth_path("work").exists());
    }

    #[test]
    fn import_saved_codex_accounts_migrates_valid_oauth_account() {
        let (service, _ccr, opencode, codex_ccr_dir) = create_migration_test_service();
        let expires_at = Utc::now() + Duration::days(10);
        let saved_at = Utc::now() - Duration::days(2);
        let last_used = Utc::now() - Duration::hours(3);
        write_codex_registry(
            &codex_ccr_dir,
            "work",
            crate::models::CodexAuthAccount {
                description: Some("Work".to_string()),
                account_id: "acc-1".to_string(),
                auth_method: Some(OpenAiAuthMethod::Chatgpt),
                api_base_url: None,
                api_provider_name: None,
                email: Some("use***@example.com".to_string()),
                plan_type: None,
                saved_at,
                last_used: Some(last_used),
                last_refresh: Some(Utc::now()),
                expires_at: Some(expires_at),
            },
        );
        write_codex_snapshot(
            &codex_ccr_dir,
            "work",
            &sample_codex_auth_json("user@example.com", "acc-1", "plus", expires_at),
        );
        let mut target_registry = OpenCodeAuthRegistry {
            current_auth: Some("existing".to_string()),
            ..Default::default()
        };
        target_registry.accounts.insert(
            "existing".to_string(),
            OpenCodeAuthAccount {
                account_id: "existing-acc".to_string(),
                email: Some("exi***@example.com".to_string()),
                plan_type: Some("plus".to_string()),
                saved_at: Utc::now(),
                last_used: None,
                expires_at: Some(expires_at),
            },
        );
        service.save_registry(&target_registry).unwrap();
        std::fs::write(
            opencode.path().join("auth.json"),
            serde_json::to_string_pretty(&json!({
                "github": { "token": "gh-token" }
            }))
            .unwrap(),
        )
        .unwrap();

        let report = service.import_saved_codex_accounts(false).unwrap();

        assert_eq!(report.imported, 1);
        assert_eq!(report.total(), 1);

        let registry = service.load_registry().unwrap();
        let imported = registry.accounts.get("work").unwrap();
        assert_eq!(imported.account_id, "acc-1");
        assert_eq!(imported.plan_type.as_deref(), Some("plus"));
        assert_eq!(imported.email.as_deref(), Some("use***@example.com"));
        assert_eq!(imported.saved_at, saved_at);
        assert_eq!(imported.last_used, Some(last_used));
        assert_eq!(registry.current_auth.as_deref(), Some("existing"));

        let snapshot: JsonValue = serde_json::from_str(
            &std::fs::read_to_string(service.account_auth_path("work")).unwrap(),
        )
        .unwrap();
        assert_eq!(
            snapshot.get("accountId").and_then(JsonValue::as_str),
            Some("acc-1")
        );
        assert_eq!(
            snapshot.get("refresh").and_then(JsonValue::as_str),
            Some("rt_acc-1")
        );
        assert_eq!(
            snapshot.get("plan").and_then(JsonValue::as_str),
            Some("plus")
        );

        let runtime_root: JsonValue = serde_json::from_str(
            &std::fs::read_to_string(opencode.path().join("auth.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(
            runtime_root
                .get("github")
                .and_then(|value| value.get("token"))
                .and_then(JsonValue::as_str),
            Some("gh-token")
        );
        assert!(runtime_root.get("openai").is_none());
    }

    #[test]
    fn import_saved_codex_accounts_skips_api_key_accounts_as_incompatible() {
        let (service, _ccr, _opencode, codex_ccr_dir) = create_migration_test_service();
        write_codex_registry(
            &codex_ccr_dir,
            "api",
            crate::models::CodexAuthAccount {
                description: None,
                account_id: "api:sk-123".to_string(),
                auth_method: Some(OpenAiAuthMethod::Api),
                api_base_url: None,
                api_provider_name: None,
                email: None,
                plan_type: None,
                saved_at: Utc::now(),
                last_used: None,
                last_refresh: None,
                expires_at: None,
            },
        );
        write_codex_snapshot(
            &codex_ccr_dir,
            "api",
            &CodexAuthJson {
                openai_api_key: Some("sk-test".to_string()),
                tokens: None,
                last_refresh: None,
            },
        );

        let report = service.import_saved_codex_accounts(false).unwrap();

        assert_eq!(report.imported, 0);
        assert_eq!(report.skipped_incompatible_auth, 1);
        assert!(service.load_registry().unwrap().accounts.is_empty());
    }

    #[test]
    fn import_saved_codex_accounts_skips_missing_snapshot() {
        let (service, _ccr, _opencode, codex_ccr_dir) = create_migration_test_service();
        write_codex_registry(
            &codex_ccr_dir,
            "missing",
            crate::models::CodexAuthAccount {
                description: None,
                account_id: "acc-missing".to_string(),
                auth_method: Some(OpenAiAuthMethod::Chatgpt),
                api_base_url: None,
                api_provider_name: None,
                email: None,
                plan_type: None,
                saved_at: Utc::now(),
                last_used: None,
                last_refresh: None,
                expires_at: None,
            },
        );

        let report = service.import_saved_codex_accounts(false).unwrap();
        assert_eq!(report.skipped_missing_snapshot, 1);
        assert!(service.load_registry().unwrap().accounts.is_empty());
    }

    #[test]
    fn import_saved_codex_accounts_skips_invalid_snapshot() {
        let (service, _ccr, _opencode, codex_ccr_dir) = create_migration_test_service();
        write_codex_registry(
            &codex_ccr_dir,
            "broken",
            crate::models::CodexAuthAccount {
                description: None,
                account_id: "acc-broken".to_string(),
                auth_method: Some(OpenAiAuthMethod::Chatgpt),
                api_base_url: None,
                api_provider_name: None,
                email: None,
                plan_type: None,
                saved_at: Utc::now(),
                last_used: None,
                last_refresh: None,
                expires_at: None,
            },
        );
        std::fs::create_dir_all(codex_ccr_dir.join("auth")).unwrap();
        std::fs::write(
            codex_ccr_dir.join("auth").join("broken.json"),
            serde_json::to_string_pretty(&json!({
                "tokens": {
                    "access_token": "only-access"
                }
            }))
            .unwrap(),
        )
        .unwrap();

        let report = service.import_saved_codex_accounts(false).unwrap();
        assert_eq!(report.skipped_invalid_snapshot, 1);
        assert!(service.load_registry().unwrap().accounts.is_empty());
    }

    #[test]
    fn import_saved_codex_accounts_skips_existing_name_conflict() {
        let (service, _ccr, _opencode, codex_ccr_dir) = create_migration_test_service();
        let expires_at = Utc::now() + Duration::days(7);
        write_codex_registry(
            &codex_ccr_dir,
            "work",
            crate::models::CodexAuthAccount {
                description: None,
                account_id: "acc-source".to_string(),
                auth_method: Some(OpenAiAuthMethod::Chatgpt),
                api_base_url: None,
                api_provider_name: None,
                email: None,
                plan_type: None,
                saved_at: Utc::now(),
                last_used: None,
                last_refresh: None,
                expires_at: Some(expires_at),
            },
        );
        write_codex_snapshot(
            &codex_ccr_dir,
            "work",
            &sample_codex_auth_json("user@example.com", "acc-source", "plus", expires_at),
        );

        let mut target_registry = OpenCodeAuthRegistry {
            current_auth: Some("work".to_string()),
            ..Default::default()
        };
        target_registry.accounts.insert(
            "work".to_string(),
            OpenCodeAuthAccount {
                account_id: "acc-target".to_string(),
                email: Some("tar***@example.com".to_string()),
                plan_type: Some("plus".to_string()),
                saved_at: Utc::now(),
                last_used: None,
                expires_at: Some(expires_at),
            },
        );
        service.save_registry(&target_registry).unwrap();

        let report = service.import_saved_codex_accounts(false).unwrap();
        let registry = service.load_registry().unwrap();
        assert_eq!(report.skipped_existing_name, 1);
        assert_eq!(registry.current_auth.as_deref(), Some("work"));
        assert_eq!(registry.accounts["work"].account_id, "acc-target");
    }

    #[test]
    fn import_saved_codex_accounts_skips_existing_account_id_conflict() {
        let (service, _ccr, _opencode, codex_ccr_dir) = create_migration_test_service();
        let expires_at = Utc::now() + Duration::days(7);
        write_codex_registry(
            &codex_ccr_dir,
            "source",
            crate::models::CodexAuthAccount {
                description: None,
                account_id: "acc-shared".to_string(),
                auth_method: Some(OpenAiAuthMethod::Chatgpt),
                api_base_url: None,
                api_provider_name: None,
                email: None,
                plan_type: None,
                saved_at: Utc::now(),
                last_used: None,
                last_refresh: None,
                expires_at: Some(expires_at),
            },
        );
        write_codex_snapshot(
            &codex_ccr_dir,
            "source",
            &sample_codex_auth_json("user@example.com", "acc-shared", "plus", expires_at),
        );

        let mut target_registry = OpenCodeAuthRegistry::default();
        target_registry.accounts.insert(
            "existing".to_string(),
            OpenCodeAuthAccount {
                account_id: "acc-shared".to_string(),
                email: Some("exi***@example.com".to_string()),
                plan_type: Some("plus".to_string()),
                saved_at: Utc::now(),
                last_used: None,
                expires_at: Some(expires_at),
            },
        );
        service.save_registry(&target_registry).unwrap();

        let report = service.import_saved_codex_accounts(false).unwrap();
        assert_eq!(report.skipped_existing_account_id, 1);
        assert!(!service.account_auth_path("source").exists());
    }

    #[test]
    fn import_saved_codex_accounts_dry_run_does_not_write_files() {
        let (service, _ccr, opencode, codex_ccr_dir) = create_migration_test_service();
        let expires_at = Utc::now() + Duration::days(7);
        write_codex_registry(
            &codex_ccr_dir,
            "preview",
            crate::models::CodexAuthAccount {
                description: None,
                account_id: "acc-preview".to_string(),
                auth_method: Some(OpenAiAuthMethod::Chatgpt),
                api_base_url: None,
                api_provider_name: None,
                email: None,
                plan_type: None,
                saved_at: Utc::now(),
                last_used: Some(Utc::now()),
                last_refresh: Some(Utc::now()),
                expires_at: Some(expires_at),
            },
        );
        write_codex_snapshot(
            &codex_ccr_dir,
            "preview",
            &sample_codex_auth_json("preview@example.com", "acc-preview", "pro", expires_at),
        );
        std::fs::write(
            opencode.path().join("auth.json"),
            serde_json::to_string_pretty(&json!({
                "openai": { "type": "oauth", "accountId": "runtime-acc" }
            }))
            .unwrap(),
        )
        .unwrap();

        let report = service.import_saved_codex_accounts(true).unwrap();

        assert!(report.dry_run);
        assert_eq!(report.imported, 1);
        assert!(service.load_registry().unwrap().accounts.is_empty());
        assert!(!service.account_auth_path("preview").exists());

        let runtime_root: JsonValue = serde_json::from_str(
            &std::fs::read_to_string(opencode.path().join("auth.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(
            runtime_root
                .get("openai")
                .and_then(|value| value.get("accountId"))
                .and_then(JsonValue::as_str),
            Some("runtime-acc")
        );
    }
}
