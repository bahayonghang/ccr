// 💻 Codex Platform 实现
// 📦 Codex CLI 平台配置管理
//
// 核心职责:
// - 📋 管理 Codex profiles
// - ⚙️ 操作 Codex config.toml / auth.json
// - 🔄 两路分发: Official (完全重置) / ThirdParty (read-modify-write)
// - 💾 仅支持 Unified 模式

use crate::managers::codex_config::CodexConfigManager;
use crate::models::{
    AuthIntent, CodexProfileAuthMode, CredentialStoreKind, OpenAiAuthMethod, Platform,
    PlatformConfig, PlatformPaths, ProfileConfig,
};
use crate::services::{
    CodexAuthCacheAction, CodexOAuthTokenService, CodexRuntimeCommitPlan, CodexRuntimeService,
};
use ccr_config::CcsConfig;
use ccr_config::PlatformConfigManager;
use ccr_config::platforms::base;
use ccr_core::core::error::{CcrError, Result};
use indexmap::IndexMap;
use serde_json::Value as JsonValue;
use std::path::PathBuf;

const THIRD_PARTY_RUNTIME_PROVIDER_KEY: &str = "custom";
const OPENAI_PROVIDER_KEY: &str = "openai";
const OPENAI_DEFAULT_BASE_URL: &str = "https://api.openai.com/v1";
const CODEX_EDITABLE_FIELDS: &[&str] = &[
    "description",
    "model",
    "small_fast_model",
    "provider",
    "provider_type",
    "account",
    "tags",
];

#[derive(Debug, Clone, PartialEq, Eq)]
enum RouteSelection {
    Official {
        relay_base_url: Option<String>,
    },
    ThirdPartyCustom {
        provider_name: String,
        base_url: String,
        wire_api: String,
        requires_openai_auth: bool,
        env_key: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum AuthSelection {
    EnsureChatgpt,
    WriteOpenAiApiKey(String),
    ClearOpenAi,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SwitchSpec {
    route: RouteSelection,
    auth: Option<AuthSelection>,
    auth_mode: CodexProfileAuthMode,
    model: Option<String>,
    approval_policy: Option<String>,
    sandbox_mode: Option<String>,
    reasoning_effort: Option<String>,
    network_access: Option<bool>,
    disable_response_storage: Option<bool>,
    forced_login_method: Option<String>,
    credential_store_override: Option<CredentialStoreKind>,
}

/// 💻 Codex Platform 实现
///
/// ## 配置文件
/// - Profiles: `~/.ccr/platforms/codex/profiles.toml`
/// - Config: `~/.codex/config.toml`
/// - Auth: `~/.codex/auth.json`
///
/// ## 分发模式
/// - **Official**: 显式切到 openai provider，并按 profile 覆盖 CCR 管理字段
/// - **ThirdParty**: 固定写入 model_provider = "custom"
pub struct CodexPlatform {
    paths: PlatformPaths,
    config_manager: CodexConfigManager,
    runtime_service: CodexRuntimeService,
}

impl CodexPlatform {
    pub fn editable_fields() -> &'static [&'static str] {
        CODEX_EDITABLE_FIELDS
    }

    /// 🏗️ 创建新的 Codex Platform 实例
    pub fn new() -> Result<Self> {
        let paths = PlatformPaths::new(Platform::Codex)?;
        let config_manager = CodexConfigManager::with_default()?;
        let runtime_service = CodexRuntimeService::new()?;
        Ok(Self {
            paths,
            config_manager,
            runtime_service,
        })
    }
    pub fn from_parts(
        paths: PlatformPaths,
        config_manager: CodexConfigManager,
        runtime_service: CodexRuntimeService,
    ) -> Self {
        Self {
            paths,
            config_manager,
            runtime_service,
        }
    }

    // ═══════════════════════════════════════════════════════════
    // 🔍 Profile 分类方法
    // ═══════════════════════════════════════════════════════════

    fn canonical_provider_type(value: Option<&str>) -> Option<&'static str> {
        match value?.trim().to_ascii_lowercase().as_str() {
            "official_relay" => Some("official_relay"),
            "third_party" | "third_party_model" => Some("third_party_model"),
            _ => None,
        }
    }

    fn canonical_auth_mode(value: Option<&str>) -> Option<CodexProfileAuthMode> {
        match value?.trim().to_ascii_lowercase().as_str() {
            "openai_chatgpt" => Some(CodexProfileAuthMode::OpenAiChatgpt),
            "openai_api_key" => Some(CodexProfileAuthMode::OpenAiApiKey),
            "provider_env_key" => Some(CodexProfileAuthMode::ProviderEnvKey),
            "no_auth" => Some(CodexProfileAuthMode::NoAuth),
            _ => None,
        }
    }

    fn normalize_profile_for_storage(profile: &ProfileConfig) -> ProfileConfig {
        let mut normalized = profile.clone();
        normalized.provider_type =
            Self::canonical_provider_type(profile.provider_type.as_deref()).map(str::to_string);
        Self::normalize_auth_fields(&mut normalized);
        normalized
    }

    fn trimmed(value: Option<&String>) -> Option<String> {
        value
            .map(|text| text.trim())
            .filter(|text| !text.is_empty())
            .map(str::to_string)
    }

    /// 🔍 判断是否为官方配置
    ///
    /// 优先检查 provider_type 字段，回退检查 base_url
    pub fn is_official_profile(profile: &ProfileConfig) -> bool {
        if let Some(kind) = Self::canonical_provider_type(profile.provider_type.as_deref()) {
            return kind == "official_relay";
        }
        profile
            .base_url
            .as_ref()
            .is_none_or(|url| url.trim().is_empty())
    }

    /// 🔍 检查是否为已弃用的 GitHub 模式
    fn is_legacy_github_profile(profile: &ProfileConfig) -> bool {
        Self::platform_string(profile, "api_mode")
            .is_some_and(|mode| mode.eq_ignore_ascii_case("github"))
    }

    // ═══════════════════════════════════════════════════════════
    // 🔧 平台数据辅助方法
    // ═══════════════════════════════════════════════════════════

    fn platform_string(profile: &ProfileConfig, key: &str) -> Option<String> {
        profile
            .platform_data
            .get(key)
            .and_then(|value| match value {
                JsonValue::String(text) if !text.trim().is_empty() => Some(text.trim().to_string()),
                JsonValue::Number(num) => Some(num.to_string()),
                JsonValue::Bool(flag) => Some(flag.to_string()),
                _ => None,
            })
    }

    fn platform_bool(profile: &ProfileConfig, key: &str) -> Option<bool> {
        profile
            .platform_data
            .get(key)
            .and_then(|value| match value {
                JsonValue::Bool(flag) => Some(*flag),
                JsonValue::String(text) => {
                    let trimmed = text.trim().to_lowercase();
                    match trimmed.as_str() {
                        "true" | "1" => Some(true),
                        "false" | "0" => Some(false),
                        _ => None,
                    }
                }
                _ => None,
            })
    }

    fn set_platform_string(profile: &mut ProfileConfig, key: &str, value: Option<&str>) {
        match value.map(str::trim).filter(|value| !value.is_empty()) {
            Some(value) => {
                profile
                    .platform_data
                    .insert(key.to_string(), JsonValue::String(value.to_string()));
            }
            None => {
                profile.platform_data.shift_remove(key);
            }
        }
    }

    fn set_platform_bool(profile: &mut ProfileConfig, key: &str, value: Option<bool>) {
        match value {
            Some(value) => {
                profile
                    .platform_data
                    .insert(key.to_string(), JsonValue::Bool(value));
            }
            None => {
                profile.platform_data.shift_remove(key);
            }
        }
    }

    fn sanitize_identifier(raw: &str) -> String {
        let mut sanitized = raw
            .trim()
            .to_lowercase()
            .chars()
            .map(|ch| {
                if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                    ch
                } else {
                    '-'
                }
            })
            .collect::<String>();

        sanitized = sanitized.trim_matches('-').to_string();
        if sanitized.is_empty() {
            "custom-provider".into()
        } else {
            sanitized
        }
    }

    fn resolve_wire_api(profile: &ProfileConfig) -> Result<String> {
        let protocol = Self::platform_string(profile, "wire_api")
            .or_else(|| Self::platform_string(profile, "api_protocol"))
            .unwrap_or_else(|| "responses".into());

        let normalized = protocol.to_lowercase();
        if normalized == "responses" {
            Ok(normalized)
        } else if normalized == "chat" {
            // Codex CLI 已废弃 wire_api=chat，自动迁移为 responses
            tracing::warn!("wire_api=\"chat\" 已被 Codex CLI 废弃，自动迁移为 \"responses\"");
            Ok("responses".into())
        } else {
            Err(CcrError::ValidationError(format!(
                "wire_api 必须为 responses，当前值: {protocol}"
            )))
        }
    }

    fn resolve_model_reasoning_effort(profile: &ProfileConfig) -> Result<Option<String>> {
        let Some(effort) = Self::platform_string(profile, "model_reasoning_effort") else {
            return Ok(None);
        };

        let normalized = effort.to_ascii_lowercase();
        match normalized.as_str() {
            "minimal" | "low" | "medium" | "high" | "xhigh" => Ok(Some(normalized)),
            _ => Err(CcrError::ValidationError(format!(
                "model_reasoning_effort 必须为 minimal/low/medium/high/xhigh，当前值: {effort}"
            ))),
        }
    }

    fn resolve_provider_name(name: &str, profile: &ProfileConfig) -> String {
        profile
            .description
            .clone()
            .or_else(|| profile.provider.clone())
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| Self::sanitize_identifier(name))
    }

    fn ensure_toml_table(
        value: &mut toml::Value,
    ) -> Result<&mut toml::map::Map<String, toml::Value>> {
        if !matches!(value, toml::Value::Table(_)) {
            *value = toml::Value::Table(toml::map::Map::new());
        }
        value
            .as_table_mut()
            .ok_or_else(|| CcrError::ConfigError("TOML table expected".into()))
    }

    fn resolve_requires_openai_auth(profile: &ProfileConfig) -> bool {
        Self::platform_bool(profile, "requires_openai_auth").unwrap_or(false)
    }

    fn resolve_openai_login_method(
        profile: &ProfileConfig,
        auth_mode: CodexProfileAuthMode,
    ) -> Option<OpenAiAuthMethod> {
        if let Some(explicit) = Self::platform_string(profile, "openai_login_method")
            .or_else(|| Self::platform_string(profile, "forced_login_method"))
            .or_else(|| Self::platform_string(profile, "login_method"))
            .or_else(|| Self::platform_string(profile, "openai_auth_method"))
        {
            return match explicit.to_ascii_lowercase().as_str() {
                "api" | "api_key" => Some(OpenAiAuthMethod::Api),
                "chatgpt" => Some(OpenAiAuthMethod::Chatgpt),
                _ => auth_mode.openai_login_method(),
            };
        }

        auth_mode.openai_login_method()
    }

    fn resolve_profile_auth_mode(profile: &ProfileConfig) -> CodexProfileAuthMode {
        if let Some(explicit) =
            Self::canonical_auth_mode(Self::platform_string(profile, "auth_mode").as_deref())
        {
            return explicit;
        }

        if Self::is_official_profile(profile) {
            return if Self::trimmed(profile.auth_token.as_ref()).is_some() {
                CodexProfileAuthMode::OpenAiApiKey
            } else {
                CodexProfileAuthMode::OpenAiChatgpt
            };
        }

        if Self::resolve_requires_openai_auth(profile) {
            return match Self::resolve_openai_login_method(
                profile,
                CodexProfileAuthMode::OpenAiChatgpt,
            ) {
                Some(OpenAiAuthMethod::Api) => CodexProfileAuthMode::OpenAiApiKey,
                _ => CodexProfileAuthMode::OpenAiChatgpt,
            };
        }

        if Self::platform_string(profile, "env_key").is_some() {
            return CodexProfileAuthMode::ProviderEnvKey;
        }

        CodexProfileAuthMode::NoAuth
    }

    fn normalize_auth_fields(profile: &mut ProfileConfig) {
        let auth_mode = Self::resolve_profile_auth_mode(profile);
        Self::set_platform_string(profile, "auth_mode", Some(auth_mode.as_str()));

        match auth_mode {
            CodexProfileAuthMode::OpenAiChatgpt | CodexProfileAuthMode::OpenAiApiKey => {
                let method = Self::resolve_openai_login_method(profile, auth_mode)
                    .unwrap_or(OpenAiAuthMethod::Chatgpt);
                let method_value = match method {
                    OpenAiAuthMethod::Chatgpt => "chatgpt",
                    OpenAiAuthMethod::Api => "api",
                };
                Self::set_platform_string(profile, "openai_login_method", Some(method_value));
                Self::set_platform_string(profile, "forced_login_method", Some(method_value));

                if !Self::is_official_profile(profile) {
                    Self::set_platform_bool(profile, "requires_openai_auth", Some(true));
                } else {
                    Self::set_platform_bool(profile, "requires_openai_auth", None);
                }
                Self::set_platform_string(profile, "env_key", None);
            }
            CodexProfileAuthMode::ProviderEnvKey => {
                Self::set_platform_string(profile, "openai_login_method", None);
                Self::set_platform_string(profile, "forced_login_method", None);
                Self::set_platform_bool(profile, "requires_openai_auth", Some(false));
            }
            CodexProfileAuthMode::NoAuth => {
                Self::set_platform_string(profile, "openai_login_method", None);
                Self::set_platform_string(profile, "forced_login_method", None);
                Self::set_platform_bool(profile, "requires_openai_auth", Some(false));
                Self::set_platform_string(profile, "env_key", None);
            }
        }
    }

    fn resolve_network_access(profile: &ProfileConfig) -> Result<Option<bool>> {
        let Some(value) = profile.platform_data.get("network_access") else {
            return Ok(None);
        };

        match value {
            JsonValue::Bool(flag) => Ok(Some(*flag)),
            JsonValue::String(text) => match text.trim().to_ascii_lowercase().as_str() {
                "true" | "1" => Ok(Some(true)),
                "false" | "0" => Ok(Some(false)),
                invalid => Err(CcrError::ValidationError(format!(
                    "network_access 必须为 true/false，当前值: {invalid}"
                ))),
            },
            _ => Err(CcrError::ValidationError(
                "network_access 必须为布尔值".into(),
            )),
        }
    }

    fn resolve_credential_store_override(
        profile: &ProfileConfig,
    ) -> Result<Option<CredentialStoreKind>> {
        let Some(store) = Self::platform_string(profile, "cli_auth_credentials_store") else {
            return Ok(None);
        };

        match store.to_ascii_lowercase().as_str() {
            "file" => Ok(Some(CredentialStoreKind::File)),
            "keyring" => Ok(Some(CredentialStoreKind::Keyring)),
            "auto" => Ok(Some(CredentialStoreKind::Auto)),
            _ => Err(CcrError::ValidationError(format!(
                "cli_auth_credentials_store 必须为 file/keyring/auto，当前值: {store}"
            ))),
        }
    }

    fn parse_current_auth_intent(config: &toml::Value) -> AuthIntent {
        let root = match config.as_table() {
            Some(root) => root,
            None => {
                return AuthIntent::OpenAiAuth {
                    method: OpenAiAuthMethod::Chatgpt,
                };
            }
        };

        let provider_id = match root.get("model_provider").and_then(|v| v.as_str()) {
            Some(id) if !id.trim().is_empty() => id,
            _ => {
                return AuthIntent::OpenAiAuth {
                    method: OpenAiAuthMethod::Chatgpt,
                };
            }
        };

        if provider_id == OPENAI_PROVIDER_KEY {
            let forced = root
                .get("forced_login_method")
                .and_then(|v| v.as_str())
                .map(str::to_string)
                .unwrap_or_else(|| "chatgpt".to_string());
            let method = match forced.to_ascii_lowercase().as_str() {
                "api" => OpenAiAuthMethod::Api,
                _ => OpenAiAuthMethod::Chatgpt,
            };
            return AuthIntent::OpenAiAuth { method };
        }

        let providers = match root.get("model_providers").and_then(|v| v.as_table()) {
            Some(p) => p,
            None => {
                return AuthIntent::OpenAiAuth {
                    method: OpenAiAuthMethod::Chatgpt,
                };
            }
        };

        let provider = providers
            .get(provider_id)
            .and_then(|v| v.as_table())
            .or_else(|| {
                (provider_id != THIRD_PARTY_RUNTIME_PROVIDER_KEY)
                    .then(|| providers.get(THIRD_PARTY_RUNTIME_PROVIDER_KEY))
                    .flatten()
                    .and_then(|v| v.as_table())
            });

        let provider = match provider {
            Some(provider) => provider,
            None => {
                return AuthIntent::OpenAiAuth {
                    method: OpenAiAuthMethod::Chatgpt,
                };
            }
        };

        let requires_openai_auth = provider
            .get("requires_openai_auth")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if requires_openai_auth {
            let forced = root
                .get("forced_login_method")
                .and_then(|v| v.as_str())
                .map(str::to_string)
                .unwrap_or_else(|| "chatgpt".to_string());
            let method = match forced.to_ascii_lowercase().as_str() {
                "api" => OpenAiAuthMethod::Api,
                _ => OpenAiAuthMethod::Chatgpt,
            };
            return AuthIntent::OpenAiAuth { method };
        }

        if let Some(env_key) = provider.get("env_key").and_then(|v| v.as_str()) {
            return AuthIntent::ProviderEnvKey {
                env_key: env_key.to_string(),
            };
        }

        AuthIntent::NoAuth
    }

    fn parse_auth_intent_from_auth_map(
        auth: &serde_json::Map<String, serde_json::Value>,
    ) -> Option<AuthIntent> {
        let has_oauth_tokens =
            auth.get("tokens")
                .and_then(|v| v.as_object())
                .is_some_and(|tokens| {
                    tokens
                        .get("id_token")
                        .and_then(|v| v.as_str())
                        .is_some_and(|s| !s.trim().is_empty())
                        || tokens
                            .get("access_token")
                            .and_then(|v| v.as_str())
                            .is_some_and(|s| !s.trim().is_empty())
                        || tokens
                            .get("refresh_token")
                            .and_then(|v| v.as_str())
                            .is_some_and(|s| !s.trim().is_empty())
                });

        if has_oauth_tokens {
            return Some(AuthIntent::OpenAiAuth {
                method: OpenAiAuthMethod::Chatgpt,
            });
        }

        if auth
            .get("OPENAI_API_KEY")
            .and_then(|v| v.as_str())
            .is_some_and(|v| !v.trim().is_empty())
        {
            return Some(AuthIntent::OpenAiAuth {
                method: OpenAiAuthMethod::Api,
            });
        }

        auth.iter().find_map(|(key, value)| {
            if !Self::is_provider_api_key_field(key) {
                return None;
            }
            value
                .as_str()
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .map(|_| AuthIntent::ProviderEnvKey {
                    env_key: key.clone(),
                })
        })
    }

    fn resolve_current_auth_intent(
        config: &toml::Value,
        auth: &serde_json::Map<String, serde_json::Value>,
    ) -> AuthIntent {
        Self::parse_auth_intent_from_auth_map(auth)
            .unwrap_or_else(|| Self::parse_current_auth_intent(config))
    }

    fn is_provider_api_key_field(key: &str) -> bool {
        key.ends_with("_API_KEY") && key != "OPENAI_API_KEY"
    }

    fn remove_openai_tokens(auth: &mut serde_json::Map<String, serde_json::Value>) {
        auth.remove("tokens");
        auth.remove("last_refresh");
    }

    fn remove_openai_api_key(auth: &mut serde_json::Map<String, serde_json::Value>) {
        auth.remove("OPENAI_API_KEY");
    }

    fn remove_auth_mode_metadata(auth: &mut serde_json::Map<String, serde_json::Value>) {
        auth.remove("auth_mode");
    }

    fn remove_provider_keys(auth: &mut serde_json::Map<String, serde_json::Value>) {
        auth.retain(|key, _| !Self::is_provider_api_key_field(key));
    }

    fn apply_auth_selection(
        auth: &mut serde_json::Map<String, serde_json::Value>,
        selection: &AuthSelection,
    ) {
        Self::remove_auth_mode_metadata(auth);
        match selection {
            AuthSelection::EnsureChatgpt => {
                Self::remove_provider_keys(auth);
                Self::remove_openai_api_key(auth);
            }
            AuthSelection::WriteOpenAiApiKey(token) => {
                Self::remove_provider_keys(auth);
                Self::remove_openai_tokens(auth);
                auth.insert(
                    "OPENAI_API_KEY".to_string(),
                    JsonValue::String(token.clone()),
                );
            }
            AuthSelection::ClearOpenAi => {
                Self::remove_provider_keys(auth);
                Self::remove_openai_tokens(auth);
                Self::remove_openai_api_key(auth);
            }
        }
    }

    fn set_optional_root_string(
        root: &mut toml::map::Map<String, toml::Value>,
        key: &str,
        value: Option<&String>,
    ) {
        if let Some(value) = value {
            root.insert(key.to_string(), toml::Value::String(value.clone()));
        } else {
            root.remove(key);
        }
    }

    fn set_optional_root_bool(
        root: &mut toml::map::Map<String, toml::Value>,
        key: &str,
        value: Option<bool>,
    ) {
        if let Some(value) = value {
            root.insert(key.to_string(), toml::Value::Boolean(value));
        } else {
            root.remove(key);
        }
    }

    fn providers_table_mut(
        root: &mut toml::map::Map<String, toml::Value>,
        create: bool,
    ) -> Result<Option<&mut toml::map::Map<String, toml::Value>>> {
        if create {
            let providers = root
                .entry("model_providers")
                .or_insert_with(|| toml::Value::Table(toml::map::Map::new()));
            return Self::ensure_toml_table(providers).map(Some);
        }

        root.get_mut("model_providers")
            .map(Self::ensure_toml_table)
            .transpose()
    }

    fn cleanup_model_providers(root: &mut toml::map::Map<String, toml::Value>) {
        let should_remove = root
            .get("model_providers")
            .and_then(|value| value.as_table())
            .is_some_and(|table| table.is_empty());
        if should_remove {
            root.remove("model_providers");
        }
    }

    fn set_network_access(
        root: &mut toml::map::Map<String, toml::Value>,
        value: Option<bool>,
    ) -> Result<()> {
        match value {
            Some(flag) => {
                let workspace = root
                    .entry("sandbox_workspace_write")
                    .or_insert_with(|| toml::Value::Table(toml::map::Map::new()));
                let workspace = Self::ensure_toml_table(workspace)?;
                workspace.insert("network_access".into(), toml::Value::Boolean(flag));
            }
            None => {
                if let Some(workspace) = root.get_mut("sandbox_workspace_write") {
                    let workspace = Self::ensure_toml_table(workspace)?;
                    workspace.remove("network_access");
                }
            }
        }

        let remove_workspace = root
            .get("sandbox_workspace_write")
            .and_then(|value| value.as_table())
            .is_some_and(|table| table.is_empty());
        if remove_workspace {
            root.remove("sandbox_workspace_write");
        }

        Ok(())
    }

    fn current_runtime_auth_intent(&self) -> Result<AuthIntent> {
        let config = self.config_manager.load_config()?;
        let auth = self.config_manager.load_auth()?;
        Ok(Self::resolve_current_auth_intent(&config, &auth))
    }

    fn resolve_auth_selection(
        auth_mode: CodexProfileAuthMode,
        auth_token: Option<String>,
        current_auth_intent: &AuthIntent,
    ) -> Result<Option<AuthSelection>> {
        match auth_mode {
            CodexProfileAuthMode::OpenAiChatgpt => Ok(Some(AuthSelection::EnsureChatgpt)),
            CodexProfileAuthMode::OpenAiApiKey => {
                let token = auth_token.ok_or_else(|| {
                    CcrError::ValidationError("当前 Profile 需要 OpenAI API Key".into())
                })?;
                Ok(Some(AuthSelection::WriteOpenAiApiKey(token)))
            }
            CodexProfileAuthMode::ProviderEnvKey | CodexProfileAuthMode::NoAuth => {
                if matches!(current_auth_intent, AuthIntent::OpenAiAuth { .. }) {
                    Ok(Some(AuthSelection::ClearOpenAi))
                } else {
                    Ok(None)
                }
            }
        }
    }

    fn build_switch_spec(
        name: &str,
        profile: &ProfileConfig,
        current_auth_intent: &AuthIntent,
    ) -> Result<SwitchSpec> {
        let auth_token = Self::trimmed(profile.auth_token.as_ref());
        let model = Self::trimmed(profile.model.as_ref());
        let approval_policy = Self::platform_string(profile, "approval_policy");
        let sandbox_mode = Self::platform_string(profile, "sandbox_mode");
        let reasoning_effort = Self::resolve_model_reasoning_effort(profile)?;
        let network_access = Self::resolve_network_access(profile)?;
        let disable_response_storage = Self::platform_bool(profile, "disable_response_storage");
        let explicit_credential_store = Self::resolve_credential_store_override(profile)?;
        let auth_mode = Self::resolve_profile_auth_mode(profile);
        let (route, auto_promote_api_key) = if Self::is_official_profile(profile) {
            let relay_base_url = Self::trimmed(profile.base_url.as_ref());
            (RouteSelection::Official { relay_base_url }, false)
        } else {
            let base_url = Self::trimmed(profile.base_url.as_ref()).ok_or_else(|| {
                CcrError::ValidationError("Codex profile 缺少 base_url (api_endpoint)".into())
            })?;
            let wire_api = Self::resolve_wire_api(profile)?;
            let mut requires_openai_auth = Self::resolve_requires_openai_auth(profile);
            let env_key = Self::platform_string(profile, "env_key");

            // 自动修正：第三方 profile 配置了 auth_token 但没有任何令牌传递路径
            // (requires_openai_auth=false 且无 env_key)，自动启用 requires_openai_auth
            // 确保令牌通过 auth.json OPENAI_API_KEY 传递给 Codex CLI
            let auto_promote = auth_token.is_some()
                && !requires_openai_auth
                && env_key.is_none()
                && matches!(auth_mode, CodexProfileAuthMode::NoAuth);
            if auto_promote {
                tracing::info!(
                    "第三方 profile 配置了 auth_token 但未指定传递方式，自动启用 requires_openai_auth"
                );
                requires_openai_auth = true;
            }

            (
                RouteSelection::ThirdPartyCustom {
                    provider_name: Self::resolve_provider_name(name, profile),
                    base_url,
                    wire_api,
                    requires_openai_auth,
                    env_key,
                },
                auto_promote,
            )
        };

        // 若自动启用了 requires_openai_auth，同步提升 auth_mode 为 OpenAiApiKey
        // 使 resolve_auth_selection 将 auth_token 写入 auth.json 而非忽略
        let effective_auth_mode = if auto_promote_api_key {
            CodexProfileAuthMode::OpenAiApiKey
        } else {
            auth_mode
        };

        let auth =
            Self::resolve_auth_selection(effective_auth_mode, auth_token, current_auth_intent)?;
        let forced_login_method =
            Self::platform_string(profile, "forced_login_method").or_else(|| {
                matches!(effective_auth_mode, CodexProfileAuthMode::OpenAiApiKey)
                    .then(|| "api".to_string())
            });

        // 未显式配置凭据存储时，仅 API key 模式需要强制 file 存储
        // （确保 CCR 写入的 auth.json 被 Codex 正确读取）
        // ChatGPT 等模式尊重当前 config.toml 中已有的凭据存储设置
        let credential_store_override = explicit_credential_store.or(match effective_auth_mode {
            CodexProfileAuthMode::OpenAiApiKey => Some(CredentialStoreKind::File),
            _ => None,
        });

        Ok(SwitchSpec {
            route,
            auth,
            auth_mode: effective_auth_mode,
            model,
            approval_policy,
            sandbox_mode,
            reasoning_effort,
            network_access,
            disable_response_storage,
            forced_login_method,
            credential_store_override,
        })
    }

    fn apply_common_settings(
        root: &mut toml::map::Map<String, toml::Value>,
        spec: &SwitchSpec,
    ) -> Result<()> {
        Self::set_optional_root_string(root, "model", spec.model.as_ref());
        Self::set_optional_root_string(root, "approval_policy", spec.approval_policy.as_ref());
        Self::set_optional_root_string(root, "sandbox_mode", spec.sandbox_mode.as_ref());
        Self::set_optional_root_string(
            root,
            "model_reasoning_effort",
            spec.reasoning_effort.as_ref(),
        );
        Self::set_optional_root_string(
            root,
            "forced_login_method",
            spec.forced_login_method.as_ref(),
        );
        Self::set_optional_root_bool(
            root,
            "disable_response_storage",
            spec.disable_response_storage,
        );
        Self::set_network_access(root, spec.network_access)?;

        if let Some(store) = spec.credential_store_override {
            root.insert(
                "cli_auth_credentials_store".into(),
                toml::Value::String(store.as_str().to_string()),
            );
        }

        Ok(())
    }

    fn apply_switch_spec(&self, spec: &SwitchSpec) -> Result<()> {
        let mut config = self.config_manager.load_config()?;
        let root = Self::ensure_toml_table(&mut config)?;

        Self::apply_common_settings(root, spec)?;

        match &spec.route {
            RouteSelection::Official { relay_base_url } => {
                root.insert(
                    "model_provider".into(),
                    toml::Value::String(THIRD_PARTY_RUNTIME_PROVIDER_KEY.into()),
                );

                let providers = Self::providers_table_mut(root, true)?.expect("providers table");
                providers.remove(OPENAI_PROVIDER_KEY);

                let mut provider_table = toml::map::Map::new();
                provider_table.insert(
                    "name".into(),
                    toml::Value::String(OPENAI_PROVIDER_KEY.into()),
                );
                provider_table.insert(
                    "base_url".into(),
                    toml::Value::String(
                        relay_base_url
                            .clone()
                            .unwrap_or_else(|| OPENAI_DEFAULT_BASE_URL.to_string()),
                    ),
                );
                provider_table.insert("wire_api".into(), toml::Value::String("responses".into()));
                provider_table.insert("requires_openai_auth".into(), toml::Value::Boolean(true));
                providers.insert(
                    THIRD_PARTY_RUNTIME_PROVIDER_KEY.to_string(),
                    toml::Value::Table(provider_table),
                );
            }
            RouteSelection::ThirdPartyCustom {
                provider_name,
                base_url,
                wire_api,
                requires_openai_auth,
                env_key,
            } => {
                root.insert(
                    "model_provider".into(),
                    toml::Value::String(THIRD_PARTY_RUNTIME_PROVIDER_KEY.into()),
                );

                let providers = Self::providers_table_mut(root, true)?.expect("providers table");
                providers.remove(OPENAI_PROVIDER_KEY);

                let mut provider_table = toml::map::Map::new();
                provider_table.insert("name".into(), toml::Value::String(provider_name.clone()));
                provider_table.insert("base_url".into(), toml::Value::String(base_url.clone()));
                provider_table.insert("wire_api".into(), toml::Value::String(wire_api.clone()));
                provider_table.insert(
                    "requires_openai_auth".into(),
                    toml::Value::Boolean(*requires_openai_auth),
                );
                if !requires_openai_auth && let Some(env_key) = env_key {
                    provider_table.insert("env_key".into(), toml::Value::String(env_key.clone()));
                }
                providers.insert(
                    THIRD_PARTY_RUNTIME_PROVIDER_KEY.to_string(),
                    toml::Value::Table(provider_table),
                );
            }
        }

        Self::cleanup_model_providers(root);
        let auth_cache = match &spec.auth {
            Some(selection) => match selection {
                AuthSelection::EnsureChatgpt => {
                    // ChatGPT 登录模式：根据凭据存储策略决定 auth.json 处理方式
                    // 优先使用显式覆盖值，否则读取当前 config.toml 的设置
                    let effective_store = spec.credential_store_override.unwrap_or_else(|| {
                        CredentialStoreKind::from_config_value(
                            root.get("cli_auth_credentials_store")
                                .and_then(|v| v.as_str()),
                        )
                    });
                    match effective_store {
                        CredentialStoreKind::File => {
                            // file 模式: 原地修改 auth.json（移除 API key，保留 OAuth tokens）
                            let mut auth = self.config_manager.load_auth()?;
                            let original = auth.clone();
                            Self::apply_auth_selection(&mut auth, selection);
                            if auth == original {
                                CodexAuthCacheAction::Preserve
                            } else if auth.is_empty() {
                                CodexAuthCacheAction::Delete
                            } else {
                                CodexAuthCacheAction::Write(auth)
                            }
                        }
                        _ => {
                            // auto/keyring 模式: 删除 auth.json
                            // OAuth 凭据由 `codex login` 通过系统钥匙链管理
                            let auth = self.config_manager.load_auth()?;
                            if auth.is_empty() {
                                CodexAuthCacheAction::Preserve
                            } else {
                                CodexAuthCacheAction::Delete
                            }
                        }
                    }
                }
                _ => {
                    // 在清理 OpenAI tokens 前，尽量把当前 runtime OAuth tokens 回写到已保存账号快照，
                    // 避免 refresh_token 轮换导致 CCR 快照持有旧 refresh_token（refresh_token_reused）。
                    if let Ok(oauth) = CodexOAuthTokenService::new()
                        && let Err(err) = oauth.sync_runtime_tokens_to_saved_account()
                    {
                        tracing::warn!(
                            "Failed to sync runtime OAuth tokens before clearing: {}",
                            err
                        );
                    }
                    let mut auth = self.config_manager.load_auth()?;
                    let original = auth.clone();
                    Self::apply_auth_selection(&mut auth, selection);
                    if auth == original {
                        // auth 无实际变更，跳过写入
                        CodexAuthCacheAction::Preserve
                    } else if auth.is_empty() {
                        // 清理旧 OAuth tokens，防止 Codex 尝试刷新过期 refresh_token
                        CodexAuthCacheAction::Delete
                    } else {
                        CodexAuthCacheAction::Write(auth)
                    }
                }
            },
            None => CodexAuthCacheAction::Preserve,
        };

        self.runtime_service.commit_plan(CodexRuntimeCommitPlan {
            config: Some(config),
            auth_cache,
        })
    }

    // ═══════════════════════════════════════════════════════════
    // 🏛️ 官方模式
    // ═══════════════════════════════════════════════════════════

    /// 🏛️ 应用官方配置（按 profile 覆盖 CCR 管理字段）
    fn apply_official_profile(&self, profile: &ProfileConfig) -> Result<()> {
        let current_auth_intent = self.current_runtime_auth_intent()?;
        let spec = Self::build_switch_spec(OPENAI_PROVIDER_KEY, profile, &current_auth_intent)?;
        self.apply_switch_spec(&spec)
    }

    // ═══════════════════════════════════════════════════════════
    // 🔧 第三方模式 - read-modify-write
    // ═══════════════════════════════════════════════════════════

    /// 🔧 应用第三方配置（固定 model_provider = custom）
    fn apply_third_party_profile(&self, name: &str, profile: &ProfileConfig) -> Result<()> {
        let current_auth_intent = self.current_runtime_auth_intent()?;
        let spec = Self::build_switch_spec(name, profile, &current_auth_intent)?;
        self.apply_switch_spec(&spec)
    }

    // ═══════════════════════════════════════════════════════════
    // 📋 Profile 文件操作
    // ═══════════════════════════════════════════════════════════

    /// 📋 从 TOML 文件加载 profiles
    fn load_profiles_from_file(&self) -> Result<IndexMap<String, ProfileConfig>> {
        base::load_profiles_from_toml(&self.paths.profiles_file)
    }

    /// 💾 保存 profiles 到 TOML 文件
    fn save_profiles_to_file(&self, profiles: &IndexMap<String, ProfileConfig>) -> Result<()> {
        base::save_profiles_to_toml(&self.paths.profiles_file, profiles, "codex", &self.paths)
    }

    pub fn profile_auth_mode(profile: &ProfileConfig) -> CodexProfileAuthMode {
        Self::resolve_profile_auth_mode(profile)
    }

    pub fn profile_openai_login_method(profile: &ProfileConfig) -> Option<OpenAiAuthMethod> {
        Self::resolve_openai_login_method(profile, Self::resolve_profile_auth_mode(profile))
    }

    pub fn profile_auth_source(profile: &ProfileConfig) -> String {
        match Self::resolve_profile_auth_mode(profile) {
            CodexProfileAuthMode::OpenAiChatgpt => "openai_chatgpt".to_string(),
            CodexProfileAuthMode::OpenAiApiKey => "openai_api_key".to_string(),
            CodexProfileAuthMode::ProviderEnvKey => Self::platform_string(profile, "env_key")
                .map(|env_key| format!("provider:{env_key}"))
                .unwrap_or_else(|| "provider".to_string()),
            CodexProfileAuthMode::NoAuth => "none".to_string(),
        }
    }

    #[allow(dead_code)]
    pub fn export_profile_env(&self, name: &str) -> Result<IndexMap<String, String>> {
        let profiles = self.load_profiles()?;
        let profile = profiles
            .get(name)
            .ok_or_else(|| CcrError::ProfileNotFound(name.to_string()))?;
        let auth_mode = Self::resolve_profile_auth_mode(profile);
        let env_key = Self::platform_string(profile, "env_key");
        self.runtime_service
            .build_env_export(name, auth_mode, env_key.as_deref())
    }

    pub fn export_profile_shell_script(&self, name: &str) -> Result<String> {
        let profiles = self.load_profiles()?;
        let profile = profiles
            .get(name)
            .ok_or_else(|| CcrError::ProfileNotFound(name.to_string()))?;
        let auth_mode = Self::resolve_profile_auth_mode(profile);
        let env_key = Self::platform_string(profile, "env_key");
        self.runtime_service
            .shell_export_script(name, auth_mode, env_key.as_deref())
    }

    fn auth_mode_from_intent(intent: &AuthIntent) -> CodexProfileAuthMode {
        match intent {
            AuthIntent::OpenAiAuth { method } => match method {
                OpenAiAuthMethod::Chatgpt => CodexProfileAuthMode::OpenAiChatgpt,
                OpenAiAuthMethod::Api => CodexProfileAuthMode::OpenAiApiKey,
            },
            AuthIntent::ProviderEnvKey { .. } => CodexProfileAuthMode::ProviderEnvKey,
            AuthIntent::NoAuth => CodexProfileAuthMode::NoAuth,
        }
    }

    fn current_custom_provider(
        config: &toml::Value,
    ) -> Option<toml::map::Map<String, toml::Value>> {
        config
            .as_table()
            .and_then(|root| root.get("model_providers"))
            .and_then(|value| value.as_table())
            .and_then(|providers| providers.get(THIRD_PARTY_RUNTIME_PROVIDER_KEY))
            .and_then(|value| value.as_table())
            .cloned()
    }

    fn spec_matches_runtime(
        spec: &SwitchSpec,
        config: &toml::Value,
        auth_intent: &AuthIntent,
    ) -> bool {
        let Some(root) = config.as_table() else {
            return false;
        };

        let matches_common = root
            .get("model")
            .and_then(|v| v.as_str())
            .map(str::to_string)
            == spec.model
            && root
                .get("approval_policy")
                .and_then(|v| v.as_str())
                .map(str::to_string)
                == spec.approval_policy
            && root
                .get("sandbox_mode")
                .and_then(|v| v.as_str())
                .map(str::to_string)
                == spec.sandbox_mode
            && root
                .get("model_reasoning_effort")
                .and_then(|v| v.as_str())
                .map(str::to_string)
                == spec.reasoning_effort
            && root
                .get("forced_login_method")
                .and_then(|v| v.as_str())
                .map(str::to_string)
                == spec.forced_login_method
            && root
                .get("disable_response_storage")
                .and_then(|v| v.as_bool())
                == spec.disable_response_storage
            && root
                .get("sandbox_workspace_write")
                .and_then(|v| v.as_table())
                .and_then(|workspace| workspace.get("network_access"))
                .and_then(|v| v.as_bool())
                == spec.network_access;

        if !matches_common {
            return false;
        }

        let matches_route = match &spec.route {
            RouteSelection::Official { relay_base_url } => {
                if root.get("model_provider").and_then(|v| v.as_str())
                    != Some(THIRD_PARTY_RUNTIME_PROVIDER_KEY)
                {
                    return false;
                }
                let Some(provider) = Self::current_custom_provider(config) else {
                    return false;
                };
                let expected_url = relay_base_url.as_deref().unwrap_or(OPENAI_DEFAULT_BASE_URL);
                provider.get("base_url").and_then(|v| v.as_str()) == Some(expected_url)
                    && provider
                        .get("requires_openai_auth")
                        .and_then(|v| v.as_bool())
                        == Some(true)
            }
            RouteSelection::ThirdPartyCustom {
                provider_name,
                base_url,
                wire_api,
                requires_openai_auth,
                env_key,
            } => {
                if root.get("model_provider").and_then(|v| v.as_str())
                    != Some(THIRD_PARTY_RUNTIME_PROVIDER_KEY)
                {
                    return false;
                }
                let Some(provider) = Self::current_custom_provider(config) else {
                    return false;
                };
                provider.get("name").and_then(|v| v.as_str()) == Some(provider_name.as_str())
                    && provider.get("base_url").and_then(|v| v.as_str()) == Some(base_url.as_str())
                    && provider.get("wire_api").and_then(|v| v.as_str()) == Some(wire_api.as_str())
                    && provider
                        .get("requires_openai_auth")
                        .and_then(|v| v.as_bool())
                        == Some(*requires_openai_auth)
                    && provider
                        .get("env_key")
                        .and_then(|v| v.as_str())
                        .map(str::to_string)
                        == *env_key
            }
        };

        matches_route && Self::auth_mode_from_intent(auth_intent) == spec.auth_mode
    }

    fn clear_current_profile_registry(&self) -> Result<()> {
        let manager = PlatformConfigManager::with_default()?;
        let mut unified = manager.load()?;
        if let Ok(entry) = unified.get_platform_mut("codex") {
            entry.current_profile = None;
            entry.last_used = Some(chrono::Utc::now().to_rfc3339());
        }
        manager.save(&unified)
    }

    fn fallback_current_profile_from_file(&self) -> Result<Option<String>> {
        if !self.paths.profiles_file.exists() {
            return Ok(None);
        }

        let content = match std::fs::read_to_string(&self.paths.profiles_file) {
            Ok(content) => content,
            Err(_) => return Ok(None),
        };

        let parsed = match toml::from_str::<CcsConfig>(&content) {
            Ok(parsed) => parsed,
            Err(_) => return Ok(None),
        };

        let current = parsed.current_config.trim();
        if current.is_empty() || !parsed.sections.contains_key(current) {
            return Ok(None);
        }

        Ok(Some(current.to_string()))
    }

    fn stable_current_profile(&self) -> Result<Option<String>> {
        match base::get_current_profile_from_registry("codex")? {
            Some(current) => {
                let profiles = self.load_profiles()?;
                let Some(profile) = profiles.get(&current) else {
                    self.clear_current_profile_registry()?;
                    return self.fallback_current_profile_from_file();
                };

                let config = self.config_manager.load_config()?;
                let auth = self.config_manager.load_auth()?;
                let auth_intent = Self::resolve_current_auth_intent(&config, &auth);
                let spec = Self::build_switch_spec(&current, profile, &auth_intent)?;

                if Self::spec_matches_runtime(&spec, &config, &auth_intent) {
                    Ok(Some(current))
                } else {
                    self.clear_current_profile_registry()?;
                    self.fallback_current_profile_from_file()
                }
            }
            None => self.fallback_current_profile_from_file(),
        }
    }
}

impl PlatformConfig for CodexPlatform {
    fn platform_name(&self) -> &str {
        "codex"
    }

    fn platform_type(&self) -> Platform {
        Platform::Codex
    }

    fn load_profiles(&self) -> Result<IndexMap<String, ProfileConfig>> {
        let mut profiles = self.load_profiles_from_file()?;
        self.runtime_service
            .overlay_profile_secrets(&mut profiles)?;
        Ok(profiles)
    }

    fn save_profile(&self, name: &str, profile: &ProfileConfig) -> Result<()> {
        let normalized = Self::normalize_profile_for_storage(profile);
        self.validate_profile(&normalized)?;
        let auth_mode = Self::resolve_profile_auth_mode(&normalized);
        let env_key = Self::platform_string(&normalized, "env_key");
        let secret = Self::trimmed(normalized.auth_token.as_ref());

        self.runtime_service
            .persist_profile_secret(name, auth_mode, env_key, secret)?;

        let mut stored_profile = normalized.clone();
        CodexRuntimeService::scrub_profile_secret_fields(&mut stored_profile, auth_mode);

        // 加载现有 profiles
        let mut profiles = self.load_profiles_from_file()?;

        // 添加/更新 profile
        profiles.insert(name.to_string(), stored_profile);

        // 保存
        self.save_profiles_to_file(&profiles)
    }

    fn delete_profile(&self, name: &str) -> Result<()> {
        let mut profiles = self.load_profiles_from_file()?;

        if profiles.shift_remove(name).is_none() {
            return Err(CcrError::ProfileNotFound(name.to_string()));
        }

        self.runtime_service.delete_profile_secret(name)?;
        self.save_profiles_to_file(&profiles)?;
        base::reconcile_registry_current_profile_after_delete("codex", name, &profiles)
    }

    fn get_settings_path(&self) -> PathBuf {
        self.config_manager.config_path().to_path_buf()
    }

    fn apply_profile(&self, name: &str) -> Result<()> {
        // 加载 profile
        let profiles = self.load_profiles()?;
        let profile = profiles
            .get(name)
            .ok_or_else(|| CcrError::ProfileNotFound(name.to_string()))?;

        // 验证
        self.validate_profile(profile)?;

        // 两路分发: Official / ThirdParty
        if Self::is_official_profile(profile) {
            self.apply_official_profile(profile)?;
        } else {
            self.apply_third_party_profile(name, profile)?;
        }

        // 更新 profiles.toml 中的 current_config
        base::update_current_config(&self.paths.profiles_file, name)?;

        // 更新注册表 current_profile
        base::update_registry_current_profile("codex", name)?;

        // 同步当前 OpenAI 账号指针，避免 profile/apply 与 auth registry 漂移
        if let Ok(service) = crate::services::CodexAuthService::new() {
            let _ = service.sync_current_auth_registry();
        }

        tracing::info!("✅ 已应用 Codex profile: {}", name);
        Ok(())
    }

    fn validate_profile(&self, profile: &ProfileConfig) -> Result<()> {
        // 向后兼容：api_mode=github 返回明确弃用错误
        if Self::is_legacy_github_profile(profile) {
            return Err(CcrError::ValidationError(
                "GitHub 模式 (api_mode=github) 已弃用，请使用第三方模式 (wire_api=responses) 替代"
                    .into(),
            ));
        }

        let spec = Self::build_switch_spec("validate", profile, &AuthIntent::NoAuth)?;
        let auth_mode = Self::resolve_profile_auth_mode(profile);

        if let Some(base_url) = profile.base_url.as_ref().map(|value| value.trim())
            && !base_url.is_empty()
            && !base_url.starts_with("http://")
            && !base_url.starts_with("https://")
        {
            return Err(CcrError::ValidationError(
                "api_endpoint 必须以 http:// 或 https:// 开头".into(),
            ));
        }

        if let Some(token) = profile.auth_token.as_ref()
            && token.trim().is_empty()
        {
            return Err(CcrError::ValidationError(
                "auth_token 不能为空字符串".into(),
            ));
        }

        match auth_mode {
            CodexProfileAuthMode::OpenAiApiKey => {
                if Self::trimmed(profile.auth_token.as_ref()).is_none() {
                    return Err(CcrError::ValidationError(
                        "openai_api_key 模式需要 auth_token".into(),
                    ));
                }
            }
            CodexProfileAuthMode::ProviderEnvKey => {
                if Self::platform_string(profile, "env_key").is_none() {
                    return Err(CcrError::ValidationError(
                        "provider_env_key 模式需要 env_key".into(),
                    ));
                }
                if Self::trimmed(profile.auth_token.as_ref()).is_none() {
                    return Err(CcrError::ValidationError(
                        "provider_env_key 模式需要 auth_token".into(),
                    ));
                }
            }
            CodexProfileAuthMode::OpenAiChatgpt | CodexProfileAuthMode::NoAuth => {}
        }

        if Self::is_official_profile(profile)
            && matches!(
                auth_mode,
                CodexProfileAuthMode::ProviderEnvKey | CodexProfileAuthMode::NoAuth
            )
        {
            return Err(CcrError::ValidationError(
                "官方 OpenAI profile 仅支持 openai_chatgpt 或 openai_api_key".into(),
            ));
        }

        match spec.route {
            RouteSelection::Official { .. } => {}
            RouteSelection::ThirdPartyCustom { .. } => {
                if Self::trimmed(profile.base_url.as_ref()).is_none() {
                    return Err(CcrError::ValidationError(
                        "Codex profile 缺少 base_url (api_endpoint)".into(),
                    ));
                }
            }
        }

        Ok(())
    }

    fn get_current_profile(&self) -> Result<Option<String>> {
        self.stable_current_profile()
    }

    fn get_env_var_names(&self) -> Vec<String> {
        vec![]
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use serde_json::json;

    fn write_file_store_config(temp_dir: &tempfile::TempDir) {
        std::fs::write(
            temp_dir.path().join("config.toml"),
            "cli_auth_credentials_store = \"file\"\n",
        )
        .unwrap();
    }

    // ═══════════════════════════════════════════════════════════
    // 🔍 Profile 分类测试
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn test_provider_type_classification() {
        // provider_type = official_relay → 官方
        let official = ProfileConfig {
            description: Some("Official".into()),
            base_url: None,
            auth_token: None,
            model: None,
            small_fast_model: None,
            provider: None,
            provider_type: Some("official_relay".into()),
            account: None,
            tags: None,
            usage_count: Some(0),
            enabled: Some(true),
            platform_data: IndexMap::new(),
        };
        assert!(CodexPlatform::is_official_profile(&official));

        // provider_type = third_party → 非官方
        let third_party = ProfileConfig {
            provider_type: Some("third_party".into()),
            base_url: Some("https://api.example.com".into()),
            ..official.clone()
        };
        assert!(!CodexPlatform::is_official_profile(&third_party));

        // 无 provider_type，无 base_url → 官方 (回退)
        let no_type_no_url = ProfileConfig {
            provider_type: None,
            base_url: None,
            ..official.clone()
        };
        assert!(CodexPlatform::is_official_profile(&no_type_no_url));

        // 无 provider_type，有 base_url → 非官方 (回退)
        let no_type_with_url = ProfileConfig {
            provider_type: None,
            base_url: Some("https://api.example.com".into()),
            ..official.clone()
        };
        assert!(!CodexPlatform::is_official_profile(&no_type_with_url));

        // 空 base_url → 官方
        let empty_url = ProfileConfig {
            provider_type: None,
            base_url: Some("  ".into()),
            ..official.clone()
        };
        assert!(CodexPlatform::is_official_profile(&empty_url));
    }

    // ═══════════════════════════════════════════════════════════
    // ✅ 验证测试
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn test_codex_platform_basic() {
        if let Ok(platform) = CodexPlatform::new() {
            assert_eq!(platform.platform_name(), "codex");
            assert_eq!(platform.platform_type(), Platform::Codex);
            let settings_path = platform.get_settings_path();
            assert!(
                settings_path
                    .file_name()
                    .map(|n| n.to_string_lossy() == "config.toml")
                    .unwrap_or(false),
                "settings path should point to config.toml, got {:?}",
                settings_path
            );
        }
    }

    #[test]
    fn test_validate_profile_modes() {
        let platform = CodexPlatform::new().unwrap();

        // 第三方 API (wire_api=responses)
        let mut custom_profile = ProfileConfig {
            description: Some("PackyCode".to_string()),
            base_url: Some("https://api.packyapi.com/v1".to_string()),
            auth_token: Some("sk-packy".to_string()),
            model: Some("gpt-4.1-mini".to_string()),
            small_fast_model: None,
            provider: Some("packycode".to_string()),
            provider_type: None,
            account: None,
            tags: None,
            usage_count: Some(0),
            enabled: Some(true),
            platform_data: IndexMap::new(),
        };
        custom_profile
            .platform_data
            .insert("wire_api".into(), json!("responses"));
        assert!(platform.validate_profile(&custom_profile).is_ok());

        // model_reasoning_effort 支持合法枚举（大小写不敏感）
        for effort in ["minimal", "low", "medium", "high", "xhigh", "HIGH"] {
            custom_profile
                .platform_data
                .insert("model_reasoning_effort".into(), json!(effort));
            assert!(
                platform.validate_profile(&custom_profile).is_ok(),
                "expected valid effort: {effort}"
            );
        }

        // model_reasoning_effort 非法值
        custom_profile
            .platform_data
            .insert("model_reasoning_effort".into(), json!("ultra"));
        let err = platform.validate_profile(&custom_profile).unwrap_err();
        let err_msg = err.to_string();
        assert!(
            err_msg.contains("model_reasoning_effort"),
            "should mention model_reasoning_effort, got: {}",
            err_msg
        );

        // wire_api 无效值
        custom_profile
            .platform_data
            .shift_remove("model_reasoning_effort");
        custom_profile
            .platform_data
            .insert("wire_api".into(), json!("invalid"));
        assert!(platform.validate_profile(&custom_profile).is_err());
    }

    #[test]
    fn test_legacy_github_profile_error() {
        let platform = CodexPlatform::new().unwrap();

        let mut github_profile = ProfileConfig {
            description: Some("GitHub Legacy".to_string()),
            base_url: Some("https://api.github.com".to_string()),
            auth_token: Some("ghp_1234567890abcdefghij".to_string()),
            model: Some("gpt-4".to_string()),
            small_fast_model: None,
            provider: Some("github".to_string()),
            provider_type: None,
            account: None,
            tags: None,
            usage_count: Some(0),
            enabled: Some(true),
            platform_data: IndexMap::new(),
        };
        github_profile
            .platform_data
            .insert("api_mode".into(), json!("github"));

        let result = platform.validate_profile(&github_profile);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("弃用"),
            "should mention deprecation, got: {}",
            err_msg
        );
    }

    #[test]
    fn test_parse_current_auth_intent_supports_legacy_and_custom_layouts() {
        let legacy: toml::Value = toml::from_str(
            r#"
model_provider = "duckcoding"

[model_providers.duckcoding]
requires_openai_auth = false
env_key = "DUCK_API_KEY"
"#,
        )
        .unwrap();
        assert_eq!(
            CodexPlatform::parse_current_auth_intent(&legacy),
            AuthIntent::ProviderEnvKey {
                env_key: "DUCK_API_KEY".to_string()
            }
        );

        let runtime_custom: toml::Value = toml::from_str(
            r#"
model_provider = "custom"

[model_providers.custom]
requires_openai_auth = false
env_key = "MISTRAL_API_KEY"
"#,
        )
        .unwrap();
        assert_eq!(
            CodexPlatform::parse_current_auth_intent(&runtime_custom),
            AuthIntent::ProviderEnvKey {
                env_key: "MISTRAL_API_KEY".to_string()
            }
        );

        let migrated: toml::Value = toml::from_str(
            r#"
model_provider = "duckcoding"

[model_providers.custom]
requires_openai_auth = false
env_key = "DUCK_API_KEY"
"#,
        )
        .unwrap();
        assert_eq!(
            CodexPlatform::parse_current_auth_intent(&migrated),
            AuthIntent::ProviderEnvKey {
                env_key: "DUCK_API_KEY".to_string()
            }
        );
    }

    #[test]
    fn test_resolve_current_auth_intent_falls_back_to_config_when_auth_missing() {
        let config: toml::Value = toml::from_str(
            r#"
model_provider = "duckcoding"
forced_login_method = "api"

[model_providers.custom]
requires_openai_auth = true
"#,
        )
        .unwrap();
        let auth = serde_json::Map::new();

        assert_eq!(
            CodexPlatform::resolve_current_auth_intent(&config, &auth),
            AuthIntent::OpenAiAuth {
                method: OpenAiAuthMethod::Api
            }
        );
    }

    // ═══════════════════════════════════════════════════════════
    // 💾 写入测试
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn test_apply_switch_spec_syncs_oauth_tokens_before_clearing() {
        let temp_dir = tempfile::tempdir().unwrap();
        let ccr_root = temp_dir.path().join("ccr");
        let codex_dir = temp_dir.path().join("codex");

        unsafe {
            std::env::set_var("CCR_ROOT", ccr_root.to_str().unwrap());
            std::env::set_var("CCR_DATA_DIR", ccr_root.to_str().unwrap());
            std::env::set_var("CCR_CODEX_DIR", codex_dir.to_str().unwrap());
        }

        // CCR auth registry + snapshot
        let ccr_codex_dir = ccr_root.join("platforms/codex");
        std::fs::create_dir_all(ccr_codex_dir.join("auth")).unwrap();

        let registry = crate::models::CodexAuthRegistry {
            version: "1.0".to_string(),
            current_auth: Some("team".to_string()),
            accounts: {
                let mut m = IndexMap::new();
                m.insert(
                    "team".to_string(),
                    crate::models::CodexAuthAccount {
                        description: None,
                        account_id: "acc-1".to_string(),
                        auth_method: Some(OpenAiAuthMethod::Chatgpt),
                        email: None,
                        saved_at: chrono::Utc::now(),
                        last_used: None,
                        last_refresh: None,
                        expires_at: None,
                    },
                );
                m
            },
        };
        let registry_path = ccr_codex_dir.join("auth_registry.toml");
        std::fs::write(&registry_path, toml::to_string_pretty(&registry).unwrap()).unwrap();

        let snapshot_path = ccr_codex_dir.join("auth/team.json");
        std::fs::write(
            &snapshot_path,
            serde_json::to_string_pretty(&json!({
                "tokens": {
                    "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJjaGF0Z3B0X2FjY291bnRfaWQiOiJhY2MtMSIsImV4cCI6MjAwMDAwMDAwMH0.sig",
                    "refresh_token": "rt_old",
                    "account_id": "acc-1"
                },
                "last_refresh": "2026-03-01T00:00:00Z"
            }))
            .unwrap(),
        )
        .unwrap();

        // Codex runtime auth.json contains OAuth tokens (will be cleared)
        std::fs::create_dir_all(&codex_dir).unwrap();
        std::fs::write(
            codex_dir.join("config.toml"),
            "cli_auth_credentials_store = \"file\"\n",
        )
        .unwrap();
        std::fs::write(
            codex_dir.join("auth.json"),
            serde_json::to_string_pretty(&json!({
                "tokens": {
                    "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJjaGF0Z3B0X2FjY291bnRfaWQiOiJhY2MtMSIsImV4cCI6MjAwMDAwMDAwMH0.sig",
                    "refresh_token": "rt_latest",
                    "account_id": "acc-1"
                },
                "last_refresh": "2026-03-26T00:00:00Z"
            }))
            .unwrap(),
        )
        .unwrap();

        let platform = CodexPlatform::new().unwrap();
        let profile = ProfileConfig {
            description: Some("API Key".to_string()),
            base_url: None,
            auth_token: Some("sk-test".to_string()),
            model: None,
            small_fast_model: None,
            provider: None,
            provider_type: Some("official_relay".to_string()),
            account: None,
            tags: None,
            usage_count: Some(0),
            enabled: Some(true),
            platform_data: {
                let mut data = IndexMap::new();
                data.insert("auth_mode".into(), json!("openai_api_key"));
                data
            },
        };

        let spec = CodexPlatform::build_switch_spec(
            "test",
            &profile,
            &AuthIntent::OpenAiAuth {
                method: OpenAiAuthMethod::Chatgpt,
            },
        )
        .unwrap();

        platform.apply_switch_spec(&spec).unwrap();

        // saved snapshot should have been synced before clearing tokens
        let updated: crate::models::CodexAuthJson =
            serde_json::from_str(&std::fs::read_to_string(&snapshot_path).unwrap()).unwrap();
        assert_eq!(
            updated.tokens.unwrap().refresh_token.as_deref().unwrap(),
            "rt_latest"
        );

        unsafe {
            std::env::remove_var("CCR_ROOT");
            std::env::remove_var("CCR_DATA_DIR");
            std::env::remove_var("CCR_CODEX_DIR");
        }
    }

    #[test]
    fn test_save_profile_scrubs_secret_and_restores_on_load() {
        let temp_dir = tempfile::tempdir().unwrap();
        let ccr_root = temp_dir.path().join("ccr");
        let codex_dir = temp_dir.path().join("codex");

        unsafe {
            std::env::set_var("CCR_ROOT", ccr_root.to_str().unwrap());
            std::env::set_var("CCR_CODEX_DIR", codex_dir.to_str().unwrap());
        }

        let platform = CodexPlatform::new().unwrap();
        let mut profile = ProfileConfig {
            description: Some("Mistral Provider".to_string()),
            base_url: Some("https://api.mistral.ai/v1".to_string()),
            auth_token: Some("mistral-key-123".to_string()),
            model: Some("mistral-large".to_string()),
            small_fast_model: None,
            provider: Some("mistral".to_string()),
            provider_type: Some("third_party_model".to_string()),
            account: None,
            tags: None,
            usage_count: Some(0),
            enabled: Some(true),
            platform_data: IndexMap::new(),
        };
        profile
            .platform_data
            .insert("wire_api".into(), json!("responses"));
        profile
            .platform_data
            .insert("env_key".into(), json!("MISTRAL_API_KEY"));
        profile
            .platform_data
            .insert("auth_mode".into(), json!("provider_env_key"));

        platform.save_profile("mistral", &profile).unwrap();

        let profiles_content =
            std::fs::read_to_string(ccr_root.join("platforms/codex/profiles.toml")).unwrap();
        assert!(
            !profiles_content.contains("mistral-key-123"),
            "profiles.toml should not persist provider secrets"
        );

        let store_content =
            std::fs::read_to_string(ccr_root.join("platforms/codex/profile_secrets.json")).unwrap();
        assert!(
            store_content.contains("mistral-key-123"),
            "secret store should persist provider secret"
        );

        let loaded = platform.load_profiles().unwrap();
        assert_eq!(
            loaded.get("mistral").unwrap().auth_token.as_deref(),
            Some("mistral-key-123")
        );

        unsafe {
            std::env::remove_var("CCR_ROOT");
            std::env::remove_var("CCR_CODEX_DIR");
        }
    }

    #[test]
    fn test_official_resets_config() {
        let temp_dir = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("CCR_CODEX_DIR", temp_dir.path().to_str().unwrap());
        }

        let config_manager = CodexConfigManager::with_default().unwrap();

        // 先写入一些数据
        let mut table = toml::map::Map::new();
        table.insert(
            "model_provider".into(),
            toml::Value::String("custom".into()),
        );
        config_manager
            .save_config_atomic(&toml::Value::Table(table))
            .unwrap();

        // 验证数据存在
        let loaded = config_manager.load_config().unwrap();
        assert!(loaded.get("model_provider").is_some());

        // 重置
        config_manager.reset_to_defaults("pre_official").unwrap();

        // 验证已重置
        let after = config_manager.load_config().unwrap();
        assert!(after.as_table().unwrap().is_empty());

        unsafe {
            std::env::remove_var("CCR_CODEX_DIR");
        }
    }

    #[test]
    fn test_apply_official_profile_overlays_managed_fields() {
        let temp_dir = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("CCR_CODEX_DIR", temp_dir.path().to_str().unwrap());
        }
        write_file_store_config(&temp_dir);

        let config_manager = CodexConfigManager::with_default().unwrap();
        let initial_config: toml::Value = toml::from_str(
            r#"
cli_auth_credentials_store = "file"
model_provider = "custom"
approval_policy = "never"
user_custom_field = "keep-me"

[model_providers.custom]
base_url = "https://api.example.com/v1"
wire_api = "responses"
requires_openai_auth = false
env_key = "MISTRAL_API_KEY"
"#,
        )
        .unwrap();
        config_manager.save_config_atomic(&initial_config).unwrap();

        let mut auth = serde_json::Map::new();
        auth.insert(
            "MISTRAL_API_KEY".to_string(),
            serde_json::Value::String("mistral-old-key".to_string()),
        );
        auth.insert(
            "legacy_key".to_string(),
            serde_json::Value::String("keep-me".to_string()),
        );
        config_manager.save_auth_atomic(&auth).unwrap();

        let platform = CodexPlatform::new().unwrap();
        let mut profile = ProfileConfig {
            description: Some("Official Relay".to_string()),
            base_url: Some("https://relay.openai.example/v1".to_string()),
            auth_token: None,
            model: Some("gpt-5-codex".to_string()),
            small_fast_model: None,
            provider: Some("openai".to_string()),
            provider_type: Some("official_relay".to_string()),
            account: None,
            tags: None,
            usage_count: Some(0),
            enabled: Some(true),
            platform_data: IndexMap::new(),
        };
        profile
            .platform_data
            .insert("approval_policy".into(), json!("on-request"));

        platform.apply_official_profile(&profile).unwrap();

        let config = config_manager.load_config().unwrap();
        let root = config.as_table().unwrap();
        assert_eq!(
            root.get("model_provider").and_then(|v| v.as_str()),
            Some(THIRD_PARTY_RUNTIME_PROVIDER_KEY)
        );
        assert_eq!(
            root.get("approval_policy").and_then(|v| v.as_str()),
            Some("on-request")
        );
        assert_eq!(
            root.get("user_custom_field").and_then(|v| v.as_str()),
            Some("keep-me")
        );
        let providers = root
            .get("model_providers")
            .and_then(|v| v.as_table())
            .unwrap();
        assert!(
            !providers.contains_key(OPENAI_PROVIDER_KEY),
            "official switch should remove openai provider block"
        );
        let custom_provider = providers
            .get(THIRD_PARTY_RUNTIME_PROVIDER_KEY)
            .and_then(|v| v.as_table())
            .unwrap();
        assert_eq!(
            custom_provider.get("base_url").and_then(|v| v.as_str()),
            Some("https://relay.openai.example/v1")
        );
        assert_eq!(
            custom_provider.get("name").and_then(|v| v.as_str()),
            Some(OPENAI_PROVIDER_KEY)
        );
        assert_eq!(
            custom_provider
                .get("requires_openai_auth")
                .and_then(|v| v.as_bool()),
            Some(true)
        );

        let auth = config_manager.load_auth().unwrap();
        assert!(
            !auth.contains_key("MISTRAL_API_KEY"),
            "official switch should clear provider credentials"
        );
        assert!(
            auth.contains_key("legacy_key"),
            "official switch should preserve non-auth metadata"
        );

        unsafe {
            std::env::remove_var("CCR_CODEX_DIR");
        }
    }

    #[test]
    fn test_apply_third_party_profile_succeeds_on_non_file_store() {
        let temp_dir = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("CCR_CODEX_DIR", temp_dir.path().to_str().unwrap());
        }

        let platform = CodexPlatform::new().unwrap();
        let mut profile = ProfileConfig {
            description: Some("Mistral Provider".to_string()),
            base_url: Some("https://api.mistral.ai/v1".to_string()),
            auth_token: Some("mistral-key-123".to_string()),
            model: Some("mistral-large".to_string()),
            small_fast_model: None,
            provider: Some("mistral".to_string()),
            provider_type: None,
            account: None,
            tags: None,
            usage_count: Some(0),
            enabled: Some(true),
            platform_data: IndexMap::new(),
        };
        profile
            .platform_data
            .insert("wire_api".into(), json!("responses"));
        profile
            .platform_data
            .insert("env_key".into(), json!("MISTRAL_API_KEY"));

        // 凭据存储默认为 auto，auth 无实际变更时走 Preserve，切换应成功
        let result = platform.apply_third_party_profile("mistral", &profile);
        assert!(
            result.is_ok(),
            "第三方 env_key profile 在非 file 存储下应成功切换: {:?}",
            result.err()
        );

        unsafe {
            std::env::remove_var("CCR_CODEX_DIR");
        }
    }

    #[test]
    fn test_third_party_preserves_fields() {
        let temp_dir = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("CCR_CODEX_DIR", temp_dir.path().to_str().unwrap());
        }

        let config_manager = CodexConfigManager::with_default().unwrap();

        // 先写入一些用户自定义字段
        let mut table = toml::map::Map::new();
        table.insert(
            "approval_policy".into(),
            toml::Value::String("unless-allow-listed".into()),
        );
        table.insert(
            "user_custom_field".into(),
            toml::Value::String("should_be_preserved".into()),
        );
        config_manager
            .save_config_atomic(&toml::Value::Table(table))
            .unwrap();

        // 模拟第三方 profile 应用 (read-modify-write)
        let mut config = config_manager.load_config().unwrap();
        let root = config.as_table_mut().unwrap();
        root.insert(
            "model_provider".into(),
            toml::Value::String("test-provider".into()),
        );
        config_manager.save_config_atomic(&config).unwrap();

        // 验证原有字段被保留
        let after = config_manager.load_config().unwrap();
        let after_table = after.as_table().unwrap();
        assert_eq!(
            after_table
                .get("user_custom_field")
                .and_then(|v| v.as_str()),
            Some("should_be_preserved")
        );
        assert_eq!(
            after_table.get("model_provider").and_then(|v| v.as_str()),
            Some("test-provider")
        );

        unsafe {
            std::env::remove_var("CCR_CODEX_DIR");
        }
    }

    #[test]
    fn test_apply_third_party_writes_config() {
        let temp_dir = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("CCR_CODEX_DIR", temp_dir.path().to_str().unwrap());
        }
        write_file_store_config(&temp_dir);

        let platform = CodexPlatform::new().unwrap();
        let mut profile = ProfileConfig {
            description: Some("PackyCode Relay".to_string()),
            base_url: Some("https://api.packyapi.com/v1".to_string()),
            auth_token: Some("sk-packy".to_string()),
            model: Some("gpt-4.1-mini".to_string()),
            small_fast_model: None,
            provider: Some("packycode".to_string()),
            provider_type: None,
            account: None,
            tags: None,
            usage_count: Some(0),
            enabled: Some(true),
            platform_data: IndexMap::new(),
        };
        profile
            .platform_data
            .insert("wire_api".into(), json!("responses"));
        profile
            .platform_data
            .insert("model_reasoning_effort".into(), json!("HIGH"));

        let result = platform.apply_third_party_profile("packy", &profile);
        assert!(
            result.is_ok(),
            "third-party profile should apply successfully"
        );
        if result.is_ok() {
            let config_path = temp_dir.path().join("config.toml");
            if config_path.exists() {
                let content = std::fs::read_to_string(&config_path).unwrap();

                let parsed: toml::Value = toml::from_str(&content).unwrap();
                let root = parsed.as_table().unwrap();
                assert_eq!(
                    root.get("model").and_then(|v| v.as_str()),
                    Some("gpt-4.1-mini"),
                    "model should be at root level"
                );
                assert_eq!(
                    root.get("model_provider").and_then(|v| v.as_str()),
                    Some(THIRD_PARTY_RUNTIME_PROVIDER_KEY),
                    "third-party runtime provider should always be custom"
                );
                assert_eq!(
                    root.get("model_reasoning_effort").and_then(|v| v.as_str()),
                    Some("high"),
                    "model_reasoning_effort should be normalized to lowercase"
                );

                let providers = root
                    .get("model_providers")
                    .and_then(|v| v.as_table())
                    .unwrap();
                let provider = providers
                    .get(THIRD_PARTY_RUNTIME_PROVIDER_KEY)
                    .and_then(|v| v.as_table())
                    .unwrap();
                assert_eq!(
                    provider.get("name").and_then(|v| v.as_str()),
                    Some("PackyCode Relay")
                );
                assert!(
                    provider.get("model").is_none(),
                    "model should NOT be in provider table when provider_model is not set, got: {:?}",
                    provider
                );
            }

            let auth_path = temp_dir.path().join("auth.json");
            assert!(
                auth_path.exists(),
                "auth.json should be written after auto-promote"
            );
            let content = std::fs::read_to_string(&auth_path).unwrap();
            let auth: serde_json::Map<String, serde_json::Value> =
                serde_json::from_str(&content).unwrap();
            assert_eq!(
                auth.get("OPENAI_API_KEY").and_then(|v| v.as_str()),
                Some("sk-packy"),
                "auto-promote should write auth_token as OPENAI_API_KEY, got: {auth:?}"
            );
        }

        unsafe {
            std::env::remove_var("CCR_CODEX_DIR");
        }
    }

    #[test]
    fn test_third_party_writes_env_key() {
        let temp_dir = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("CCR_CODEX_DIR", temp_dir.path().to_str().unwrap());
        }
        write_file_store_config(&temp_dir);

        let platform = CodexPlatform::new().unwrap();
        let mut profile = ProfileConfig {
            description: Some("Mistral Provider".to_string()),
            base_url: Some("https://api.mistral.ai/v1".to_string()),
            auth_token: Some("mistral-key-123".to_string()),
            model: Some("mistral-large".to_string()),
            small_fast_model: None,
            provider: Some("mistral".to_string()),
            provider_type: None,
            account: None,
            tags: None,
            usage_count: Some(0),
            enabled: Some(true),
            platform_data: IndexMap::new(),
        };
        profile
            .platform_data
            .insert("wire_api".into(), json!("responses"));
        profile
            .platform_data
            .insert("env_key".into(), json!("MISTRAL_API_KEY"));
        profile
            .platform_data
            .insert("requires_openai_auth".into(), json!(false));

        let result = platform.apply_third_party_profile("mistral", &profile);
        assert!(result.is_ok(), "provider env-key profile should apply");
        if result.is_ok() {
            // 验证 config.toml 包含 env_key
            let config_path = temp_dir.path().join("config.toml");
            if config_path.exists() {
                let content = std::fs::read_to_string(&config_path).unwrap();
                assert!(
                    content.contains("env_key"),
                    "config.toml should contain env_key, got: {}",
                    content
                );
                let parsed: toml::Value = toml::from_str(&content).unwrap();
                let root = parsed.as_table().unwrap();
                assert_eq!(
                    root.get("model_provider").and_then(|v| v.as_str()),
                    Some(THIRD_PARTY_RUNTIME_PROVIDER_KEY),
                    "third-party runtime provider should always be custom"
                );
                assert!(
                    content.contains("MISTRAL_API_KEY"),
                    "env_key should be MISTRAL_API_KEY, got: {}",
                    content
                );
            }

            // provider env key 不再写入 auth.json
            let auth_path = temp_dir.path().join("auth.json");
            if auth_path.exists() {
                let content = std::fs::read_to_string(&auth_path).unwrap();
                assert!(
                    !content.contains("OPENAI_API_KEY"),
                    "auth.json should not keep OPENAI_API_KEY when provider env key is used, got: {}",
                    content
                );
                assert!(
                    !content.contains("MISTRAL_API_KEY"),
                    "auth.json should not store provider env secrets, got: {}",
                    content
                );
            }
        }

        unsafe {
            std::env::remove_var("CCR_CODEX_DIR");
        }
    }

    #[test]
    fn test_switching_from_openai_auth_to_provider_env_key_clears_openai_key() {
        let temp_dir = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("CCR_CODEX_DIR", temp_dir.path().to_str().unwrap());
        }
        write_file_store_config(&temp_dir);

        let config_manager = CodexConfigManager::with_default().unwrap();

        let mut auth = serde_json::Map::new();
        auth.insert(
            "OPENAI_API_KEY".to_string(),
            serde_json::Value::String("openai-old-key".to_string()),
        );
        auth.insert(
            "legacy_key".to_string(),
            serde_json::Value::String("legacy".to_string()),
        );
        config_manager.save_auth_atomic(&auth).unwrap();

        let platform = CodexPlatform::new().unwrap();
        let mut profile = ProfileConfig {
            description: Some("Mistral Provider".to_string()),
            base_url: Some("https://api.mistral.ai/v1".to_string()),
            auth_token: Some("mistral-key-123".to_string()),
            model: Some("mistral-large".to_string()),
            small_fast_model: None,
            provider: Some("mistral".to_string()),
            provider_type: None,
            account: None,
            tags: None,
            usage_count: Some(0),
            enabled: Some(true),
            platform_data: IndexMap::new(),
        };
        profile
            .platform_data
            .insert("wire_api".into(), json!("responses"));
        profile
            .platform_data
            .insert("env_key".into(), json!("MISTRAL_API_KEY"));
        profile
            .platform_data
            .insert("requires_openai_auth".into(), json!(false));

        platform
            .apply_third_party_profile("mistral", &profile)
            .unwrap();

        let auth = config_manager.load_auth().unwrap();
        assert!(
            !auth.contains_key("OPENAI_API_KEY"),
            "OPENAI_API_KEY should be cleared when switching to provider env key"
        );
        assert!(
            !auth.contains_key("MISTRAL_API_KEY"),
            "provider env key should no longer be written into auth.json"
        );
        assert!(
            auth.contains_key("legacy_key"),
            "non-auth metadata should be preserved when rewriting auth.json"
        );

        let config = config_manager.load_config().unwrap();
        let root = config.as_table().unwrap();
        assert_eq!(
            root.get("model_provider").and_then(|v| v.as_str()),
            Some(THIRD_PARTY_RUNTIME_PROVIDER_KEY),
            "third-party runtime provider should always be custom"
        );
        let providers = root
            .get("model_providers")
            .and_then(|v| v.as_table())
            .unwrap();
        let provider = providers
            .get(THIRD_PARTY_RUNTIME_PROVIDER_KEY)
            .and_then(|v| v.as_table())
            .unwrap();
        assert_eq!(
            provider.get("env_key").and_then(|v| v.as_str()),
            Some("MISTRAL_API_KEY")
        );

        unsafe {
            std::env::remove_var("CCR_CODEX_DIR");
        }
    }

    #[test]
    fn test_third_party_default_auth_key() {
        let temp_dir = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("CCR_CODEX_DIR", temp_dir.path().to_str().unwrap());
        }
        write_file_store_config(&temp_dir);

        let platform = CodexPlatform::new().unwrap();
        let mut profile = ProfileConfig {
            description: Some("OpenAI Compatible".to_string()),
            base_url: Some("https://api.openai-proxy.com/v1".to_string()),
            auth_token: Some("sk-proxy-key".to_string()),
            model: None,
            small_fast_model: None,
            provider: Some("proxy".to_string()),
            provider_type: None,
            account: None,
            tags: None,
            usage_count: Some(0),
            enabled: Some(true),
            platform_data: IndexMap::new(),
        };
        profile
            .platform_data
            .insert("wire_api".into(), json!("responses"));
        // 不设置 env_key / requires_openai_auth，自动提升应启用 requires_openai_auth

        let result = platform.apply_third_party_profile("proxy", &profile);
        assert!(
            result.is_ok(),
            "third-party profile without auth hint should apply"
        );
        if result.is_ok() {
            // 自动提升：有 auth_token 但无传递路径 → requires_openai_auth=true
            // auth.json 应包含 OPENAI_API_KEY
            let auth_path = temp_dir.path().join("auth.json");
            assert!(
                auth_path.exists(),
                "auth.json should be written after auto-promote"
            );
            let content = std::fs::read_to_string(&auth_path).unwrap();
            let auth: serde_json::Map<String, serde_json::Value> =
                serde_json::from_str(&content).unwrap();
            assert_eq!(
                auth.get("OPENAI_API_KEY").and_then(|v| v.as_str()),
                Some("sk-proxy-key"),
                "auto-promote should write auth_token as OPENAI_API_KEY, got: {auth:?}"
            );

            // 验证 config.toml 不包含 env_key（因为未设置）
            let config_path = temp_dir.path().join("config.toml");
            if config_path.exists() {
                let content = std::fs::read_to_string(&config_path).unwrap();
                let parsed: toml::Value = toml::from_str(&content).unwrap();
                let root = parsed.as_table().unwrap();
                assert_eq!(
                    root.get("model_provider").and_then(|v| v.as_str()),
                    Some(THIRD_PARTY_RUNTIME_PROVIDER_KEY),
                    "third-party runtime provider should always be custom"
                );
                assert_eq!(
                    root.get("forced_login_method").and_then(|v| v.as_str()),
                    Some("api"),
                    "auto-promote should force API login method"
                );
                assert!(
                    !content.contains("env_key"),
                    "should not write env_key when not provided, got: {}",
                    content
                );
            }
        }

        unsafe {
            std::env::remove_var("CCR_CODEX_DIR");
        }
    }

    #[test]
    fn test_third_party_default_auth_key_clears_stale_chatgpt_auth_metadata() {
        let temp_dir = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("CCR_CODEX_DIR", temp_dir.path().to_str().unwrap());
        }
        write_file_store_config(&temp_dir);

        let config_manager = CodexConfigManager::with_default().unwrap();
        let mut auth = serde_json::Map::new();
        auth.insert("auth_mode".to_string(), json!("chatgpt"));
        auth.insert(
            "tokens".to_string(),
            json!({
                "id_token": "id-token",
                "access_token": "access-token",
                "refresh_token": "refresh-token",
            }),
        );
        config_manager.save_auth_atomic(&auth).unwrap();

        let platform = CodexPlatform::new().unwrap();
        let mut profile = ProfileConfig {
            description: Some("Proxy".to_string()),
            base_url: Some("https://api.proxy.example/v1".to_string()),
            auth_token: Some("sk-proxy-key".to_string()),
            model: None,
            small_fast_model: None,
            provider: Some("proxy".to_string()),
            provider_type: None,
            account: None,
            tags: None,
            usage_count: Some(0),
            enabled: Some(true),
            platform_data: IndexMap::new(),
        };
        profile
            .platform_data
            .insert("wire_api".into(), json!("responses"));

        platform
            .apply_third_party_profile("proxy", &profile)
            .unwrap();

        let auth_path = temp_dir.path().join("auth.json");
        let content = std::fs::read_to_string(&auth_path).unwrap();
        let auth: serde_json::Map<String, serde_json::Value> =
            serde_json::from_str(&content).unwrap();
        assert_eq!(
            auth.get("OPENAI_API_KEY").and_then(|v| v.as_str()),
            Some("sk-proxy-key")
        );
        assert!(
            !auth.contains_key("auth_mode"),
            "stale auth_mode should be removed after API-key switch, got: {auth:?}"
        );
        assert!(
            !auth.contains_key("tokens"),
            "chatgpt tokens should be removed after API-key switch, got: {auth:?}"
        );

        let config = config_manager.load_config().unwrap();
        let root = config.as_table().unwrap();
        assert_eq!(
            root.get("forced_login_method").and_then(|v| v.as_str()),
            Some("api")
        );

        unsafe {
            std::env::remove_var("CCR_CODEX_DIR");
        }
    }

    #[test]
    fn test_provider_model_explicit_is_ignored() {
        let temp_dir = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("CCR_CODEX_DIR", temp_dir.path().to_str().unwrap());
        }
        write_file_store_config(&temp_dir);

        let platform = CodexPlatform::new().unwrap();
        let mut profile = ProfileConfig {
            description: Some("Custom Provider".to_string()),
            base_url: Some("https://api.example.com/v1".to_string()),
            auth_token: Some("sk-test".to_string()),
            model: Some("gpt-4".to_string()),
            small_fast_model: None,
            provider: Some("custom".to_string()),
            provider_type: None,
            account: None,
            tags: None,
            usage_count: Some(0),
            enabled: Some(true),
            platform_data: IndexMap::new(),
        };
        profile
            .platform_data
            .insert("wire_api".into(), json!("responses"));
        // 历史字段 provider_model 已弃用，本次应忽略
        profile
            .platform_data
            .insert("provider_model".into(), json!("custom-gpt-4-alias"));

        let result = platform.apply_third_party_profile("custom", &profile);
        assert!(
            result.is_ok(),
            "profile with legacy provider_model should still apply"
        );
        if result.is_ok() {
            let config_path = temp_dir.path().join("config.toml");
            if config_path.exists() {
                let content = std::fs::read_to_string(&config_path).unwrap();
                let parsed: toml::Value = toml::from_str(&content).unwrap();
                let root = parsed.as_table().unwrap();

                // Root model should be profile.model
                assert_eq!(
                    root.get("model").and_then(|v| v.as_str()),
                    Some("gpt-4"),
                    "root model should be profile.model"
                );
                assert_eq!(
                    root.get("model_provider").and_then(|v| v.as_str()),
                    Some(THIRD_PARTY_RUNTIME_PROVIDER_KEY),
                    "third-party runtime provider should always be custom"
                );

                // Provider table model 不应再写入 provider_model
                let providers = root
                    .get("model_providers")
                    .and_then(|v| v.as_table())
                    .unwrap();
                let provider = providers
                    .get(THIRD_PARTY_RUNTIME_PROVIDER_KEY)
                    .and_then(|v| v.as_table())
                    .unwrap();
                assert_eq!(
                    provider.get("model").and_then(|v| v.as_str()),
                    None,
                    "provider table should ignore legacy provider_model"
                );
            }
        }

        unsafe {
            std::env::remove_var("CCR_CODEX_DIR");
        }
    }

    #[test]
    fn test_third_party_clears_stale_optional_provider_fields() {
        let temp_dir = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("CCR_CODEX_DIR", temp_dir.path().to_str().unwrap());
        }
        write_file_store_config(&temp_dir);

        let config_manager = CodexConfigManager::with_default().unwrap();

        // 先写入旧版/历史遗留字段：env_key + provider_table.model
        let mut provider_table = toml::map::Map::new();
        provider_table.insert("name".into(), toml::Value::String("custom".into()));
        provider_table.insert(
            "base_url".into(),
            toml::Value::String("https://api.example.com/v1".into()),
        );
        provider_table.insert("wire_api".into(), toml::Value::String("responses".into()));
        provider_table.insert("requires_openai_auth".into(), toml::Value::Boolean(false));
        provider_table.insert("env_key".into(), toml::Value::String("OLD_API_KEY".into()));
        provider_table.insert(
            "model".into(),
            toml::Value::String("old-provider-model".into()),
        );

        let mut providers_table = toml::map::Map::new();
        providers_table.insert("custom".into(), toml::Value::Table(provider_table));

        let mut root_table = toml::map::Map::new();
        root_table.insert(
            "model_provider".into(),
            toml::Value::String("custom".into()),
        );
        root_table.insert(
            "model_providers".into(),
            toml::Value::Table(providers_table),
        );
        // 保留 file 凭据存储设置，避免 auto-promote 时 commit_plan 拒绝写入
        root_table.insert(
            "cli_auth_credentials_store".into(),
            toml::Value::String("file".into()),
        );
        config_manager
            .save_config_atomic(&toml::Value::Table(root_table))
            .unwrap();

        let platform = CodexPlatform::new().unwrap();
        let mut profile = ProfileConfig {
            description: Some("Custom Provider".to_string()),
            base_url: Some("https://api.example.com/v1".to_string()),
            auth_token: Some("sk-test".to_string()),
            model: Some("new-root-model".to_string()),
            small_fast_model: None,
            provider: Some("custom".to_string()),
            provider_type: None,
            account: None,
            tags: None,
            usage_count: Some(0),
            enabled: Some(true),
            platform_data: IndexMap::new(),
        };
        profile
            .platform_data
            .insert("wire_api".into(), json!("responses"));
        profile
            .platform_data
            .insert("requires_openai_auth".into(), json!(false));
        // 故意不设置 env_key / provider_model，验证旧字段会被清理

        platform
            .apply_third_party_profile("custom", &profile)
            .unwrap();

        let config = config_manager.load_config().unwrap();
        let root = config.as_table().unwrap();
        assert_eq!(
            root.get("model_provider").and_then(|v| v.as_str()),
            Some(THIRD_PARTY_RUNTIME_PROVIDER_KEY),
            "third-party runtime provider should always be custom"
        );
        let providers = root
            .get("model_providers")
            .and_then(|v| v.as_table())
            .unwrap();
        let provider = providers
            .get(THIRD_PARTY_RUNTIME_PROVIDER_KEY)
            .and_then(|v| v.as_table())
            .unwrap();

        assert!(
            provider.get("env_key").is_none(),
            "stale env_key should be removed, got: {:?}",
            provider
        );
        assert!(
            provider.get("model").is_none(),
            "stale provider model should be removed, got: {:?}",
            provider
        );

        unsafe {
            std::env::remove_var("CCR_CODEX_DIR");
        }
    }

    #[test]
    fn test_requires_openai_auth_ignores_env_key_and_clears_provider_token() {
        let temp_dir = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("CCR_CODEX_DIR", temp_dir.path().to_str().unwrap());
        }
        write_file_store_config(&temp_dir);

        let config_manager = CodexConfigManager::with_default().unwrap();

        // 初始 auth.json 模拟第三方 env_key 模式
        let mut auth = serde_json::Map::new();
        auth.insert(
            "MISTRAL_API_KEY".to_string(),
            serde_json::Value::String("mistral-old-key".to_string()),
        );
        auth.insert(
            "legacy_key".to_string(),
            serde_json::Value::String("legacy".to_string()),
        );
        config_manager.save_auth_atomic(&auth).unwrap();

        let platform = CodexPlatform::new().unwrap();

        // 切换到 requires_openai_auth=true（不提供 auth_token）
        let mut profile = ProfileConfig {
            description: Some("OpenAI Auth Provider".to_string()),
            base_url: Some("https://api.proxy.example/v1".to_string()),
            auth_token: None,
            model: Some("gpt-5".to_string()),
            small_fast_model: None,
            provider: Some("proxy".to_string()),
            provider_type: None,
            account: None,
            tags: None,
            usage_count: Some(0),
            enabled: Some(true),
            platform_data: IndexMap::new(),
        };
        profile
            .platform_data
            .insert("wire_api".into(), json!("responses"));
        profile
            .platform_data
            .insert("requires_openai_auth".into(), json!(true));
        profile
            .platform_data
            .insert("env_key".into(), json!("SHOULD_BE_IGNORED"));

        platform
            .apply_third_party_profile("proxy", &profile)
            .unwrap();

        // auth.json: 第三方 key 被清理，且未写入 OPENAI_API_KEY（需要重新登录）
        let auth = config_manager.load_auth().unwrap();
        assert!(
            !auth.contains_key("MISTRAL_API_KEY"),
            "provider key should be cleared when switching to openai auth"
        );
        assert!(
            !auth.contains_key("OPENAI_API_KEY"),
            "OPENAI_API_KEY should not be auto-populated when profile has no auth_token"
        );
        assert!(
            !auth.contains_key("MISTRAL_API_KEY"),
            "provider key should be removed after switching to openai-auth provider"
        );
        assert!(
            auth.contains_key("legacy_key"),
            "non-auth metadata should be preserved after cleanup"
        );

        // config.toml: env_key 被移除
        let config = config_manager.load_config().unwrap();
        let root = config.as_table().unwrap();
        assert_eq!(
            root.get("model_provider").and_then(|v| v.as_str()),
            Some(THIRD_PARTY_RUNTIME_PROVIDER_KEY),
            "third-party runtime provider should always be custom"
        );
        let providers = root
            .get("model_providers")
            .and_then(|v| v.as_table())
            .unwrap();
        let provider = providers
            .get(THIRD_PARTY_RUNTIME_PROVIDER_KEY)
            .and_then(|v| v.as_table())
            .unwrap();
        assert_eq!(
            provider
                .get("requires_openai_auth")
                .and_then(|v| v.as_bool()),
            Some(true)
        );
        assert!(
            provider.get("env_key").is_none(),
            "env_key should be ignored/removed when requires_openai_auth=true"
        );

        unsafe {
            std::env::remove_var("CCR_CODEX_DIR");
        }
    }
}
