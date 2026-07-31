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
    AuthIntent, CodexEnvironmentPresence, CodexProfileAuthMode, CodexRuntimeAuthSource,
    CodexRuntimeDiagnostic, CodexRuntimeIssue, CredentialStoreKind, OpenAiAuthMethod, Platform,
    PlatformConfig, PlatformPaths, ProfileConfig, ProviderAuthValidity, RuntimeMatchStatus,
};
use crate::services::{
    CodexAuthCacheAction, CodexOAuthTokenService, CodexRuntimeCommitPlan, CodexRuntimeService,
};
use ccr_config::CcsConfig;
use ccr_config::PlatformConfigManager;
use ccr_config::platforms::base;
use ccr_core::Secret;
use ccr_core::core::AtomicWriter;
use ccr_core::core::error::{CcrError, Result};
use ccr_core::core::logging::ColorOutput;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::path::Path;
use std::path::PathBuf;

const THIRD_PARTY_RUNTIME_PROVIDER_KEY: &str = "custom";
const OPENAI_PROVIDER_KEY: &str = "openai";
const OPENAI_DEFAULT_BASE_URL: &str = "https://api.openai.com/v1";
const PROFILE_ENTRY_AUTH_STATE_FILE: &str = "profile_entry_auth_state.json";
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

#[derive(Clone, PartialEq, Eq)]
enum AuthSelection {
    EnsureChatgpt,
    WriteOpenAiApiKey(String),
    WriteProviderBearerToken(Secret),
    ClearOpenAi,
}

#[derive(Clone, PartialEq, Eq)]
struct SwitchSpec {
    route: RouteSelection,
    auth: Option<AuthSelection>,
    auth_mode: CodexProfileAuthMode,
    model: Option<String>,
    model_catalog_json: Option<String>,
    preferred_auth_method: Option<String>,
    approval_policy: Option<String>,
    sandbox_mode: Option<String>,
    reasoning_effort: Option<String>,
    network_access: Option<bool>,
    disable_response_storage: Option<bool>,
    forced_login_method: Option<String>,
    credential_store_override: Option<CredentialStoreKind>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProfileEntryAuthState {
    exists: bool,
    content: Option<String>,
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
    fn registry_lock_dir(&self) -> PathBuf {
        self.paths.root.join(".locks")
    }

    fn codex_dir(&self) -> PathBuf {
        self.config_manager
            .auth_path()
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf()
    }

    fn profile_entry_auth_state_path(&self) -> PathBuf {
        self.paths.platform_dir.join(PROFILE_ENTRY_AUTH_STATE_FILE)
    }

    pub fn editable_fields() -> &'static [&'static str] {
        CODEX_EDITABLE_FIELDS
    }

    pub fn has_profile_entry_auth_backup(&self) -> bool {
        self.profile_entry_auth_state_path().exists()
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
            "provider_bearer_token" => Some(CodexProfileAuthMode::ProviderBearerToken),
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

    /// Secret 版 trimmed：取原文（auth_token 的合法明文消费点——
    /// 用于 auth.json 持久化与 env 注入），空白视为未设置
    fn trimmed_secret(value: Option<&ccr_core::Secret>) -> Option<String> {
        value
            .map(|secret| secret.expose().trim())
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

    fn resolve_model_catalog_json(profile: &ProfileConfig) -> Option<String> {
        Self::platform_string(profile, "model_catalog_json")
    }

    fn resolve_preferred_auth_method(profile: &ProfileConfig) -> Result<Option<String>> {
        let Some(method) = Self::platform_string(profile, "preferred_auth_method") else {
            return Ok(None);
        };

        let normalized = method.to_ascii_lowercase();
        match normalized.as_str() {
            "apikey" | "chatgpt" => Ok(Some(normalized)),
            _ => Err(CcrError::ValidationError(format!(
                "preferred_auth_method 必须为 apikey/chatgpt，当前值: {method}"
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
            return if Self::trimmed_secret(profile.auth_token.as_ref()).is_some() {
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

                // `forced_login_method` is a persistent Codex policy restriction. Preserve an
                // explicit value for compatibility, but never infer or overwrite it from the
                // active authentication method.

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
            CodexProfileAuthMode::ProviderBearerToken => {
                let preferred_auth_method = Self::platform_string(profile, "preferred_auth_method")
                    .unwrap_or_else(|| "apikey".to_string());
                let forced_login_method = Self::platform_string(profile, "forced_login_method")
                    .unwrap_or_else(|| "api".to_string());
                Self::set_platform_string(
                    profile,
                    "preferred_auth_method",
                    Some(&preferred_auth_method),
                );
                Self::set_platform_string(
                    profile,
                    "forced_login_method",
                    Some(&forced_login_method),
                );
                Self::set_platform_string(profile, "openai_login_method", None);
                Self::set_platform_bool(profile, "requires_openai_auth", Some(false));
                Self::set_platform_string(profile, "env_key", None);
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

        if provider
            .get("experimental_bearer_token")
            .and_then(toml::Value::as_str)
            .is_some_and(|token| !token.trim().is_empty())
        {
            return AuthIntent::ProviderBearerToken;
        }

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
        let config_intent = Self::parse_current_auth_intent(config);
        if matches!(config_intent, AuthIntent::ProviderBearerToken) {
            return config_intent;
        }
        Self::parse_auth_intent_from_auth_map(auth).unwrap_or(config_intent)
    }

    fn is_provider_api_key_field(key: &str) -> bool {
        key.ends_with("_API_KEY") && key != "OPENAI_API_KEY"
    }

    fn is_valid_environment_variable(value: &str) -> bool {
        let mut characters = value.chars();
        let Some(first) = characters.next() else {
            return false;
        };
        (first == '_' || first.is_ascii_alphabetic())
            && characters.all(|character| character == '_' || character.is_ascii_alphanumeric())
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
            AuthSelection::WriteProviderBearerToken(_) | AuthSelection::ClearOpenAi => {
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

    fn warn_if_model_catalog_missing(model_catalog_json: Option<&String>) {
        let Some(raw_path) = model_catalog_json else {
            return;
        };
        let path = if let Some(relative) = raw_path
            .strip_prefix("~/")
            .or_else(|| raw_path.strip_prefix("~\\"))
        {
            let Some(home) = dirs::home_dir() else {
                return;
            };
            home.join(relative)
        } else {
            let path = PathBuf::from(raw_path);
            if !path.is_absolute() {
                return;
            }
            path
        };

        if !path.exists() {
            let message = format!(
                "Codex model catalog 不存在: {}；请先运行 DeepSeek 官方配置脚本生成 models.json",
                path.display()
            );
            ColorOutput::warning(&message);
            tracing::warn!("{}", message);
        }
    }

    fn ensure_providers_table_mut(
        root: &mut toml::map::Map<String, toml::Value>,
    ) -> Result<&mut toml::map::Map<String, toml::Value>> {
        let providers = root
            .entry("model_providers")
            .or_insert_with(|| toml::Value::Table(toml::map::Map::new()));
        Self::ensure_toml_table(providers)
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
            CodexProfileAuthMode::ProviderBearerToken => {
                let token = auth_token.ok_or_else(|| {
                    CcrError::ValidationError("provider_bearer_token 模式需要 auth_token".into())
                })?;
                Ok(Some(AuthSelection::WriteProviderBearerToken(Secret::new(
                    token,
                ))))
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
        let auth_token = Self::trimmed_secret(profile.auth_token.as_ref());
        let model = Self::trimmed(profile.model.as_ref());
        let model_catalog_json = Self::resolve_model_catalog_json(profile);
        let approval_policy = Self::platform_string(profile, "approval_policy");
        let sandbox_mode = Self::platform_string(profile, "sandbox_mode");
        let reasoning_effort = Self::resolve_model_reasoning_effort(profile)?;
        let network_access = Self::resolve_network_access(profile)?;
        let disable_response_storage = Self::platform_bool(profile, "disable_response_storage");
        let explicit_credential_store = Self::resolve_credential_store_override(profile)?;
        let auth_mode = Self::resolve_profile_auth_mode(profile);
        let preferred_auth_method = Self::resolve_preferred_auth_method(profile)?.or_else(|| {
            matches!(auth_mode, CodexProfileAuthMode::ProviderBearerToken)
                .then(|| "apikey".to_string())
        });
        let (route, effective_auth_mode) = if Self::is_official_profile(profile) {
            if matches!(auth_mode, CodexProfileAuthMode::ProviderBearerToken) {
                return Err(CcrError::ValidationError(
                    "官方 OpenAI profile 不支持 provider_bearer_token".into(),
                ));
            }
            let relay_base_url = Self::trimmed(profile.base_url.as_ref());
            (RouteSelection::Official { relay_base_url }, auth_mode)
        } else {
            let base_url = Self::trimmed(profile.base_url.as_ref()).ok_or_else(|| {
                CcrError::ValidationError("Codex profile 缺少 base_url (api_endpoint)".into())
            })?;
            let wire_api = Self::resolve_wire_api(profile)?;
            let mut requires_openai_auth = Self::resolve_requires_openai_auth(profile);
            let mut env_key = Self::platform_string(profile, "env_key");
            if matches!(auth_mode, CodexProfileAuthMode::ProviderBearerToken) {
                requires_openai_auth = false;
                env_key = None;
            }
            if matches!(auth_mode, CodexProfileAuthMode::ProviderEnvKey)
                && !env_key
                    .as_deref()
                    .is_some_and(Self::is_valid_environment_variable)
            {
                return Err(CcrError::ValidationError(
                    "provider_env_key 模式需要合法的 env_key 变量名".into(),
                ));
            }

            // 向后兼容：第三方 profile 配置了 auth_token 但未显式声明传递方式时，
            // resolve_profile_auth_mode 会把它解释为 OpenAI API key 模式。这里同步
            // 路由层的 requires_openai_auth，确保 status/switch/validate 使用同一语义。
            let auto_promote_api_key = auth_token.is_some()
                && !requires_openai_auth
                && env_key.is_none()
                && matches!(auth_mode, CodexProfileAuthMode::NoAuth);
            if auto_promote_api_key
                || (matches!(auth_mode, CodexProfileAuthMode::OpenAiApiKey)
                    && !requires_openai_auth
                    && env_key.is_none())
            {
                tracing::debug!(
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
                if auto_promote_api_key {
                    CodexProfileAuthMode::OpenAiApiKey
                } else {
                    auth_mode
                },
            )
        };

        let auth =
            Self::resolve_auth_selection(effective_auth_mode, auth_token, current_auth_intent)?;
        let forced_login_method = match effective_auth_mode {
            CodexProfileAuthMode::OpenAiChatgpt | CodexProfileAuthMode::OpenAiApiKey => {
                Self::platform_string(profile, "forced_login_method")
            }
            CodexProfileAuthMode::ProviderBearerToken => Some(
                Self::platform_string(profile, "forced_login_method")
                    .unwrap_or_else(|| "api".to_string()),
            ),
            CodexProfileAuthMode::ProviderEnvKey | CodexProfileAuthMode::NoAuth => None,
        };

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
            model_catalog_json,
            preferred_auth_method,
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
        Self::set_optional_root_string(
            root,
            "model_catalog_json",
            spec.model_catalog_json.as_ref(),
        );
        Self::set_optional_root_string(
            root,
            "preferred_auth_method",
            spec.preferred_auth_method.as_ref(),
        );
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

                let providers = Self::ensure_providers_table_mut(root)?;
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

                let providers = Self::ensure_providers_table_mut(root)?;
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
                if let Some(AuthSelection::WriteProviderBearerToken(token)) = spec.auth.as_ref() {
                    provider_table.insert(
                        "experimental_bearer_token".into(),
                        toml::Value::String(token.expose().to_string()),
                    );
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
        })?;
        Self::warn_if_model_catalog_missing(spec.model_catalog_json.as_ref());
        Ok(())
    }

    fn capture_profile_entry_auth_state(&self) -> Result<()> {
        if self.has_profile_entry_auth_backup() {
            return Ok(());
        }

        let auth_path = self.config_manager.auth_path();
        let state = if auth_path.exists() {
            let content = std::fs::read_to_string(auth_path).map_err(|error| {
                CcrError::ConfigError(format!(
                    "读取 Codex 入口 auth.json 失败 {}: {}",
                    auth_path.display(),
                    error
                ))
            })?;
            ProfileEntryAuthState {
                exists: true,
                content: Some(content),
            }
        } else {
            ProfileEntryAuthState {
                exists: false,
                content: None,
            }
        };

        let serialized = serde_json::to_string_pretty(&state).map_err(|error| {
            CcrError::ConfigError(format!("序列化 Codex 入口 auth 快照失败: {}", error))
        })?;
        let backup_path = self.profile_entry_auth_state_path();
        AtomicWriter::new(&backup_path)
            .secret(true)
            .write_string(&serialized)?;
        crate::utils::ensure_private_permissions(&backup_path);
        Ok(())
    }

    fn restore_profile_entry_auth_state(&self) -> Result<bool> {
        let backup_path = self.profile_entry_auth_state_path();
        if !backup_path.exists() {
            return Ok(false);
        }

        let raw = std::fs::read_to_string(&backup_path).map_err(|error| {
            CcrError::ConfigError(format!(
                "读取 Codex 入口 auth 快照失败 {}: {}",
                backup_path.display(),
                error
            ))
        })?;
        let state: ProfileEntryAuthState = serde_json::from_str(&raw).map_err(|error| {
            CcrError::ConfigError(format!(
                "解析 Codex 入口 auth 快照失败 {}: {}",
                backup_path.display(),
                error
            ))
        })?;

        let auth_path = self.config_manager.auth_path();
        if state.exists {
            let content = state
                .content
                .ok_or_else(|| CcrError::ConfigError("Codex 入口 auth 快照缺少内容".into()))?;
            AtomicWriter::new(auth_path)
                .secret(true)
                .write_string(&content)?;
            crate::utils::ensure_private_permissions(auth_path);
        } else if auth_path.exists() {
            std::fs::remove_file(auth_path).map_err(|error| {
                CcrError::ConfigError(format!(
                    "删除 Codex auth.json 失败 {}: {}",
                    auth_path.display(),
                    error
                ))
            })?;
        }

        std::fs::remove_file(&backup_path).map_err(|error| {
            CcrError::ConfigError(format!(
                "清理 Codex 入口 auth 快照失败 {}: {}",
                backup_path.display(),
                error
            ))
        })?;
        Ok(true)
    }

    fn apply_runtime_route_without_auth(
        &self,
        route: &RouteSelection,
        credential_store_override: Option<CredentialStoreKind>,
    ) -> Result<()> {
        let mut config = self.config_manager.load_config()?;
        let root = Self::ensure_toml_table(&mut config)?;

        root.remove("model");
        root.remove("model_catalog_json");
        root.remove("preferred_auth_method");
        root.remove("approval_policy");
        root.remove("sandbox_mode");
        root.remove("model_reasoning_effort");
        root.remove("disable_response_storage");
        root.remove("forced_login_method");
        root.remove("sandbox_workspace_write");

        if let Some(store) = credential_store_override {
            root.insert(
                "cli_auth_credentials_store".into(),
                toml::Value::String(store.as_str().to_string()),
            );
        }

        match route {
            RouteSelection::Official { relay_base_url } => {
                root.insert(
                    "model_provider".into(),
                    toml::Value::String(THIRD_PARTY_RUNTIME_PROVIDER_KEY.into()),
                );

                let providers = Self::ensure_providers_table_mut(root)?;
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
            RouteSelection::ThirdPartyCustom { .. } => {
                return Err(CcrError::ValidationError(
                    "Codex official runtime restore does not support third-party routes".into(),
                ));
            }
        }

        Self::cleanup_model_providers(root);
        self.runtime_service.commit_plan(CodexRuntimeCommitPlan {
            config: Some(config),
            auth_cache: CodexAuthCacheAction::Preserve,
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

    pub fn clear_active_profile_runtime(&self) -> Result<()> {
        self.apply_runtime_route_without_auth(
            &RouteSelection::Official {
                relay_base_url: None,
            },
            Some(CredentialStoreKind::File),
        )?;
        self.restore_profile_entry_auth_state()?;
        self.clear_current_profile_registry()?;
        Ok(())
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
            CodexProfileAuthMode::ProviderBearerToken => {
                "config:experimental_bearer_token".to_string()
            }
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

    /// Build a read-only snapshot of the CCR profile and Codex runtime.
    ///
    /// This intentionally avoids `stable_current_profile()`: that method repairs the registry
    /// pointer when the route differs, while diagnostics must preserve the evidence it observes.
    pub fn inspect_runtime(&self) -> Result<CodexRuntimeDiagnostic> {
        self.inspect_runtime_with_env(|name| std::env::var(name).ok())
    }

    /// Replay the resolved profile through the existing atomic apply path.
    pub fn repair_runtime(&self, snapshot: &CodexRuntimeDiagnostic) -> Result<()> {
        self.repair_runtime_with_env(snapshot, |name| std::env::var(name).ok())
    }

    fn repair_runtime_with_env<F>(
        &self,
        snapshot: &CodexRuntimeDiagnostic,
        read_env: F,
    ) -> Result<()>
    where
        F: Fn(&str) -> Option<String>,
    {
        if !snapshot.repairable {
            return Err(CcrError::ValidationError(
                "当前 Codex runtime 状态不能通过重放 profile 安全修复".into(),
            ));
        }

        let current = self.inspect_runtime_with_env(read_env)?;
        if !current.repairable || current.resolved_profile != snapshot.resolved_profile {
            return Err(CcrError::ValidationError(
                "诊断后 Codex profile 状态已变化，请重新运行 ccr codex fix".into(),
            ));
        }

        let profile = current
            .resolved_profile
            .as_deref()
            .ok_or_else(|| CcrError::ValidationError("没有可重放的 Codex profile".into()))?;
        <Self as PlatformConfig>::apply_profile(self, profile)
    }

    fn inspect_runtime_with_env<F>(&self, read_env: F) -> Result<CodexRuntimeDiagnostic>
    where
        F: Fn(&str) -> Option<String>,
    {
        let registry_profile = self.current_profile_from_registry()?;
        let profiles_file_profile = self.current_profile_from_profiles_file()?;
        let mut profiles = self.load_profiles_from_file()?;
        self.runtime_service
            .overlay_profile_secrets(&mut profiles)?;

        let mut issues = Vec::new();
        let (resolved_profile, profile_status) = Self::resolve_diagnostic_profile(
            registry_profile.as_deref(),
            profiles_file_profile.as_deref(),
            &profiles,
            &mut issues,
        );

        let config = self.config_manager.load_config()?;
        let auth = self.config_manager.load_auth()?;
        let root = config.as_table();
        let credential_store = CredentialStoreKind::from_config_value(
            root.and_then(|table| table.get("cli_auth_credentials_store"))
                .and_then(toml::Value::as_str),
        );
        let runtime_provider_id = root
            .and_then(|table| table.get("model_provider"))
            .and_then(toml::Value::as_str)
            .map(str::to_string);
        let runtime_provider = runtime_provider_id.as_deref().and_then(|provider_id| {
            root.and_then(|table| table.get("model_providers"))
                .and_then(toml::Value::as_table)
                .and_then(|providers| providers.get(provider_id))
                .and_then(toml::Value::as_table)
        });
        let runtime_provider_name = runtime_provider
            .and_then(|provider| provider.get("name"))
            .and_then(toml::Value::as_str)
            .map(str::to_string);
        let base_url = runtime_provider
            .and_then(|provider| provider.get("base_url"))
            .and_then(toml::Value::as_str)
            .map(Self::safe_base_url_for_display);
        let wire_api = runtime_provider
            .and_then(|provider| provider.get("wire_api"))
            .and_then(toml::Value::as_str)
            .map(str::to_string);
        let runtime_env_key = runtime_provider
            .and_then(|provider| provider.get("env_key"))
            .and_then(toml::Value::as_str)
            .filter(|env_key| Self::is_valid_environment_variable(env_key))
            .map(str::to_string);

        let current_auth_intent = Self::resolve_current_auth_intent(&config, &auth);
        let auth_source = Self::runtime_auth_source(&current_auth_intent, credential_store, &auth);

        let mut expected = None;
        let mut expected_env_key = None;
        if let Some(profile_name) = resolved_profile.as_deref() {
            let profile = profiles
                .get(profile_name)
                .ok_or_else(|| CcrError::ProfileNotFound(profile_name.to_string()))?;
            let spec = Self::build_switch_spec(profile_name, profile, &current_auth_intent)?;
            if let RouteSelection::ThirdPartyCustom { env_key, .. } = &spec.route {
                expected_env_key = env_key.clone();
            }
            expected = Some((spec, Self::trimmed_secret(profile.auth_token.as_ref())));
        }

        let mut env_names = vec!["CODEX_API_KEY".to_string(), "OPENAI_API_KEY".to_string()];
        for env_key in runtime_env_key.iter().chain(expected_env_key.iter()) {
            if !env_names.contains(env_key) {
                env_names.push(env_key.clone());
            }
        }
        let env_values = env_names
            .iter()
            .map(|name| {
                let value = read_env(name).filter(|value| !value.trim().is_empty());
                (name.clone(), value)
            })
            .collect::<IndexMap<_, _>>();
        let environment = env_values
            .iter()
            .map(|(variable, value)| CodexEnvironmentPresence {
                variable: variable.clone(),
                is_set: value.is_some(),
            })
            .collect::<Vec<_>>();

        let (mut route_status, mut credential_status, mut credential_repairable) = match expected
            .as_ref()
        {
            Some((spec, secret)) => {
                let route_status = Self::diagnostic_route_status(spec, &config);
                let (credential_status, credential_repairable) = Self::diagnostic_credential_status(
                    spec,
                    secret.as_deref(),
                    credential_store,
                    &config,
                    &auth,
                    &env_values,
                );
                (route_status, credential_status, credential_repairable)
            }
            None => (
                RuntimeMatchStatus::NotApplicable,
                RuntimeMatchStatus::NotApplicable,
                false,
            ),
        };

        let intended_env_key = expected_env_key.as_deref().or(runtime_env_key.as_deref());
        for variable in ["CODEX_API_KEY", "OPENAI_API_KEY"] {
            if env_values.get(variable).is_some_and(Option::is_some)
                && intended_env_key != Some(variable)
            {
                issues.push(CodexRuntimeIssue::EnvironmentOverride {
                    variable: variable.to_string(),
                });
                if matches!(
                    credential_status,
                    RuntimeMatchStatus::Match | RuntimeMatchStatus::NotApplicable
                ) {
                    credential_status = RuntimeMatchStatus::Unsupported;
                }
                credential_repairable = false;
            }
        }

        if read_env("OPENAI_BASE_URL").is_some_and(|value| !value.trim().is_empty()) {
            issues.push(CodexRuntimeIssue::EnvironmentOverride {
                variable: "OPENAI_BASE_URL".to_string(),
            });
            if route_status == RuntimeMatchStatus::Match {
                route_status = RuntimeMatchStatus::Unsupported;
            }
        }

        if let Some(codex_home) = read_env("CODEX_HOME").filter(|value| !value.trim().is_empty())
            && self.config_manager.config_path().parent() != Some(Path::new(&codex_home))
        {
            issues.push(CodexRuntimeIssue::CodexHomeMismatch);
            if route_status == RuntimeMatchStatus::Match {
                route_status = RuntimeMatchStatus::Unsupported;
            }
        }

        match route_status {
            RuntimeMatchStatus::Mismatch => issues.push(CodexRuntimeIssue::RouteMismatch),
            RuntimeMatchStatus::Missing => issues.push(CodexRuntimeIssue::RouteMismatch),
            _ => {}
        }
        match credential_status {
            RuntimeMatchStatus::Missing => issues.push(CodexRuntimeIssue::CredentialMissing),
            RuntimeMatchStatus::Mismatch => issues.push(CodexRuntimeIssue::CredentialMismatch),
            RuntimeMatchStatus::Unsupported => {
                issues.push(CodexRuntimeIssue::CredentialUnsupported)
            }
            _ => {}
        }

        let has_drift =
            profile_status.is_drift() || route_status.is_drift() || credential_status.is_drift();
        let repairable = resolved_profile.is_some()
            && profile_status != RuntimeMatchStatus::Mismatch
            && route_status != RuntimeMatchStatus::Unsupported
            && credential_repairable
            && has_drift;

        Ok(CodexRuntimeDiagnostic {
            registry_path: self.paths.registry_file.clone(),
            profiles_path: self.paths.profiles_file.clone(),
            config_path: self.config_manager.config_path().to_path_buf(),
            auth_path: self.config_manager.auth_path().to_path_buf(),
            registry_profile,
            profiles_file_profile,
            resolved_profile,
            runtime_provider_id,
            runtime_provider_name,
            base_url,
            wire_api,
            credential_store,
            auth_source,
            profile_status,
            route_status,
            credential_status,
            provider_auth_validity: ProviderAuthValidity::NotChecked,
            environment,
            issues,
            repairable,
        })
    }

    fn resolve_diagnostic_profile(
        registry_profile: Option<&str>,
        profiles_file_profile: Option<&str>,
        profiles: &IndexMap<String, ProfileConfig>,
        issues: &mut Vec<CodexRuntimeIssue>,
    ) -> (Option<String>, RuntimeMatchStatus) {
        match (registry_profile, profiles_file_profile) {
            (None, None) => (None, RuntimeMatchStatus::NotApplicable),
            (Some(registry), Some(file)) if registry == file => {
                if profiles.contains_key(registry) {
                    (Some(registry.to_string()), RuntimeMatchStatus::Match)
                } else {
                    issues.push(CodexRuntimeIssue::ProfileNotFound {
                        profile: registry.to_string(),
                    });
                    (None, RuntimeMatchStatus::Missing)
                }
            }
            (Some(_), Some(_)) => {
                issues.push(CodexRuntimeIssue::ProfilePointerMismatch);
                (None, RuntimeMatchStatus::Mismatch)
            }
            (Some(registry), None) => {
                issues.push(CodexRuntimeIssue::ProfilesPointerMissing);
                if profiles.contains_key(registry) {
                    (Some(registry.to_string()), RuntimeMatchStatus::Missing)
                } else {
                    issues.push(CodexRuntimeIssue::ProfileNotFound {
                        profile: registry.to_string(),
                    });
                    (None, RuntimeMatchStatus::Missing)
                }
            }
            (None, Some(file)) => {
                issues.push(CodexRuntimeIssue::RegistryPointerMissing);
                if profiles.contains_key(file) {
                    (Some(file.to_string()), RuntimeMatchStatus::Missing)
                } else {
                    issues.push(CodexRuntimeIssue::ProfileNotFound {
                        profile: file.to_string(),
                    });
                    (None, RuntimeMatchStatus::Missing)
                }
            }
        }
    }

    fn current_profile_from_profiles_file(&self) -> Result<Option<String>> {
        if !self.paths.profiles_file.exists() {
            return Ok(None);
        }
        let content = std::fs::read_to_string(&self.paths.profiles_file).map_err(|error| {
            CcrError::ConfigError(format!(
                "读取 Codex profiles 文件失败 {}: {}",
                self.paths.profiles_file.display(),
                error
            ))
        })?;
        let Ok(config) = toml::from_str::<CcsConfig>(&content) else {
            return Ok(None);
        };
        let current = config.current_config.trim();
        Ok((!current.is_empty()).then(|| current.to_string()))
    }

    fn diagnostic_route_status(spec: &SwitchSpec, config: &toml::Value) -> RuntimeMatchStatus {
        let Some(root) = config.as_table() else {
            return RuntimeMatchStatus::Missing;
        };
        if root
            .get("model_provider")
            .and_then(toml::Value::as_str)
            .is_none()
        {
            return RuntimeMatchStatus::Missing;
        }

        let common_matches = Self::root_string_matches(root, "model", spec.model.as_ref())
            && Self::root_string_matches(
                root,
                "model_catalog_json",
                spec.model_catalog_json.as_ref(),
            )
            && Self::root_string_matches(
                root,
                "preferred_auth_method",
                spec.preferred_auth_method.as_ref(),
            )
            && Self::root_string_matches(root, "approval_policy", spec.approval_policy.as_ref())
            && Self::root_string_matches(root, "sandbox_mode", spec.sandbox_mode.as_ref())
            && Self::root_string_matches(
                root,
                "model_reasoning_effort",
                spec.reasoning_effort.as_ref(),
            )
            && Self::root_string_matches(
                root,
                "forced_login_method",
                spec.forced_login_method.as_ref(),
            )
            && root
                .get("disable_response_storage")
                .and_then(toml::Value::as_bool)
                == spec.disable_response_storage
            && root
                .get("sandbox_workspace_write")
                .and_then(toml::Value::as_table)
                .and_then(|workspace| workspace.get("network_access"))
                .and_then(toml::Value::as_bool)
                == spec.network_access
            && spec.credential_store_override.is_none_or(|expected| {
                CredentialStoreKind::from_config_value(
                    root.get("cli_auth_credentials_store")
                        .and_then(toml::Value::as_str),
                ) == expected
            });

        if !common_matches || !Self::diagnostic_spec_matches_runtime(spec, config) {
            RuntimeMatchStatus::Mismatch
        } else {
            RuntimeMatchStatus::Match
        }
    }

    fn diagnostic_spec_matches_runtime(spec: &SwitchSpec, config: &toml::Value) -> bool {
        if !Self::spec_matches_runtime_without_auth(spec, config) {
            return false;
        }

        match &spec.route {
            RouteSelection::Official { .. } => {
                Self::current_custom_provider(config).is_some_and(|provider| {
                    provider.get("name").and_then(toml::Value::as_str) == Some(OPENAI_PROVIDER_KEY)
                        && provider.get("wire_api").and_then(toml::Value::as_str)
                            == Some("responses")
                        && provider.get("env_key").is_none()
                })
            }
            RouteSelection::ThirdPartyCustom { .. } => true,
        }
    }

    fn root_string_matches(
        root: &toml::map::Map<String, toml::Value>,
        key: &str,
        expected: Option<&String>,
    ) -> bool {
        root.get(key).and_then(toml::Value::as_str) == expected.map(String::as_str)
    }

    fn safe_base_url_for_display(value: &str) -> String {
        let without_fragment = value.split('#').next().unwrap_or(value);
        let without_query = without_fragment
            .split('?')
            .next()
            .unwrap_or(without_fragment);
        let Some((scheme, remainder)) = without_query.split_once("://") else {
            return without_query.to_string();
        };
        let host_and_path = remainder
            .rsplit_once('@')
            .map(|(_, host_and_path)| host_and_path)
            .unwrap_or(remainder);
        format!("{scheme}://{host_and_path}")
    }

    fn diagnostic_credential_status(
        spec: &SwitchSpec,
        expected_secret: Option<&str>,
        store: CredentialStoreKind,
        config: &toml::Value,
        auth: &serde_json::Map<String, serde_json::Value>,
        env: &IndexMap<String, Option<String>>,
    ) -> (RuntimeMatchStatus, bool) {
        let has_openai_key = auth
            .get("OPENAI_API_KEY")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|value| !value.trim().is_empty());
        let has_chatgpt_tokens = auth
            .get("tokens")
            .and_then(serde_json::Value::as_object)
            .is_some_and(|tokens| {
                ["id_token", "access_token", "refresh_token"]
                    .iter()
                    .any(|field| {
                        tokens
                            .get(*field)
                            .and_then(serde_json::Value::as_str)
                            .is_some_and(|value| !value.trim().is_empty())
                    })
            });
        let has_provider_key = auth.iter().any(|(key, value)| {
            Self::is_provider_api_key_field(key)
                && value.as_str().is_some_and(|value| !value.trim().is_empty())
        });

        match spec.auth_mode {
            CodexProfileAuthMode::OpenAiApiKey => {
                if store != CredentialStoreKind::File {
                    return (RuntimeMatchStatus::Unsupported, false);
                }
                let Some(expected) = expected_secret else {
                    return (RuntimeMatchStatus::Missing, false);
                };
                let actual = auth
                    .get("OPENAI_API_KEY")
                    .and_then(serde_json::Value::as_str)
                    .map(str::trim)
                    .filter(|value| !value.is_empty());
                match actual {
                    None => (RuntimeMatchStatus::Missing, true),
                    Some(actual) if actual != expected.trim() || has_chatgpt_tokens => {
                        (RuntimeMatchStatus::Mismatch, true)
                    }
                    Some(_) => (RuntimeMatchStatus::Match, true),
                }
            }
            CodexProfileAuthMode::OpenAiChatgpt => {
                if store != CredentialStoreKind::File {
                    return (RuntimeMatchStatus::Unsupported, false);
                }
                if !has_chatgpt_tokens {
                    return (RuntimeMatchStatus::Missing, false);
                }
                if has_openai_key || has_provider_key {
                    (RuntimeMatchStatus::Mismatch, true)
                } else {
                    (RuntimeMatchStatus::Match, true)
                }
            }
            CodexProfileAuthMode::ProviderEnvKey => {
                let env_key = match &spec.route {
                    RouteSelection::ThirdPartyCustom {
                        env_key: Some(env_key),
                        ..
                    } => env_key,
                    _ => return (RuntimeMatchStatus::Missing, false),
                };
                let Some(expected) = expected_secret else {
                    return (RuntimeMatchStatus::Missing, false);
                };
                let actual = env
                    .get(env_key)
                    .and_then(Option::as_deref)
                    .map(str::trim)
                    .filter(|value| !value.is_empty());
                match actual {
                    None => (RuntimeMatchStatus::Missing, false),
                    Some(actual) if actual != expected.trim() => {
                        (RuntimeMatchStatus::Mismatch, false)
                    }
                    Some(_) if has_provider_key => (RuntimeMatchStatus::Unsupported, false),
                    Some(_) if has_openai_key || has_chatgpt_tokens => {
                        (RuntimeMatchStatus::Mismatch, true)
                    }
                    Some(_) => (RuntimeMatchStatus::Match, true),
                }
            }
            CodexProfileAuthMode::ProviderBearerToken => {
                let Some(expected) = expected_secret else {
                    return (RuntimeMatchStatus::Missing, false);
                };
                let provider = Self::current_custom_provider(config);
                let actual = provider
                    .as_ref()
                    .and_then(|provider| provider.get("experimental_bearer_token"))
                    .and_then(toml::Value::as_str)
                    .map(str::trim)
                    .filter(|value| !value.is_empty());
                match actual {
                    None => (RuntimeMatchStatus::Missing, true),
                    Some(actual)
                        if actual != expected.trim()
                            || has_provider_key
                            || has_openai_key
                            || has_chatgpt_tokens =>
                    {
                        (RuntimeMatchStatus::Mismatch, true)
                    }
                    Some(_) => (RuntimeMatchStatus::Match, true),
                }
            }
            CodexProfileAuthMode::NoAuth => {
                if has_provider_key {
                    (RuntimeMatchStatus::Mismatch, false)
                } else if has_openai_key || has_chatgpt_tokens {
                    (RuntimeMatchStatus::Mismatch, true)
                } else {
                    (RuntimeMatchStatus::NotApplicable, true)
                }
            }
        }
    }

    fn runtime_auth_source(
        intent: &AuthIntent,
        store: CredentialStoreKind,
        auth: &serde_json::Map<String, serde_json::Value>,
    ) -> CodexRuntimeAuthSource {
        match intent {
            AuthIntent::ProviderEnvKey { env_key } => {
                if Self::is_valid_environment_variable(env_key) {
                    CodexRuntimeAuthSource::Environment {
                        variable: env_key.clone(),
                    }
                } else {
                    CodexRuntimeAuthSource::EnvironmentInvalid
                }
            }
            AuthIntent::ProviderBearerToken => CodexRuntimeAuthSource::ConfigBearerToken,
            AuthIntent::NoAuth => CodexRuntimeAuthSource::None,
            AuthIntent::OpenAiAuth { .. } => match store {
                CredentialStoreKind::Keyring => CodexRuntimeAuthSource::KeyringUnreadable,
                CredentialStoreKind::Auto => CodexRuntimeAuthSource::AutoUnreadable,
                CredentialStoreKind::File => {
                    if auth
                        .get("tokens")
                        .and_then(serde_json::Value::as_object)
                        .is_some_and(|tokens| !tokens.is_empty())
                    {
                        CodexRuntimeAuthSource::AuthJsonChatgptTokens
                    } else if auth
                        .get("OPENAI_API_KEY")
                        .and_then(serde_json::Value::as_str)
                        .is_some_and(|value| !value.trim().is_empty())
                    {
                        CodexRuntimeAuthSource::AuthJsonOpenAiApiKey
                    } else {
                        CodexRuntimeAuthSource::None
                    }
                }
            },
        }
    }

    fn spec_matches_runtime_without_auth(spec: &SwitchSpec, config: &toml::Value) -> bool {
        let Some(root) = config.as_table() else {
            return false;
        };

        if !Self::root_string_matches(root, "model_catalog_json", spec.model_catalog_json.as_ref())
            || !Self::root_string_matches(
                root,
                "preferred_auth_method",
                spec.preferred_auth_method.as_ref(),
            )
        {
            return false;
        }

        match &spec.route {
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
        }
    }

    fn clear_current_profile_registry(&self) -> Result<()> {
        let manager = PlatformConfigManager::new(&self.paths.registry_file);
        let mut unified = manager.load_or_create_default()?;
        if let Ok(entry) = unified.get_platform_mut("codex") {
            entry.current_profile = None;
            entry.last_used = Some(chrono::Utc::now().to_rfc3339());
        }
        manager.save(&unified)
    }

    fn current_profile_from_registry(&self) -> Result<Option<String>> {
        let manager = PlatformConfigManager::new(&self.paths.registry_file);
        let unified = manager.load()?;

        match unified.get_platform("codex") {
            Ok(entry) => Ok(entry.current_profile.clone()),
            Err(_) => Ok(None),
        }
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
        let runtime_matches_profile =
            |profile_name: &str, profile: &ProfileConfig| -> Result<bool> {
                let config = self.config_manager.load_config()?;
                let auth = self.config_manager.load_auth()?;
                let auth_intent = Self::resolve_current_auth_intent(&config, &auth);
                let spec = Self::build_switch_spec(profile_name, profile, &auth_intent)?;
                Ok(Self::spec_matches_runtime_without_auth(&spec, &config))
            };

        match self.current_profile_from_registry()? {
            Some(current) => {
                let profiles = self.load_profiles()?;
                let Some(profile) = profiles.get(&current) else {
                    self.clear_current_profile_registry()?;
                    return Ok(None);
                };

                if runtime_matches_profile(&current, profile)? {
                    Ok(Some(current))
                } else {
                    self.clear_current_profile_registry()?;
                    Ok(None)
                }
            }
            None => {
                let Some(current) = self.fallback_current_profile_from_file()? else {
                    return Ok(None);
                };

                let profiles = self.load_profiles()?;
                let Some(profile) = profiles.get(&current) else {
                    return Ok(None);
                };

                if runtime_matches_profile(&current, profile)? {
                    Ok(Some(current))
                } else {
                    Ok(None)
                }
            }
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
        let secret = Self::trimmed_secret(normalized.auth_token.as_ref());

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
        base::reconcile_registry_current_profile_after_delete_with_paths(
            &self.paths.registry_file,
            &self.registry_lock_dir(),
            "codex",
            name,
            &profiles,
        )
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
        self.capture_profile_entry_auth_state()?;

        // 两路分发: Official / ThirdParty
        if Self::is_official_profile(profile) {
            self.apply_official_profile(profile)?;
        } else {
            self.apply_third_party_profile(name, profile)?;
        }

        // 更新 profiles.toml 中的 current_config
        base::update_current_config(&self.paths.profiles_file, name)?;

        // 更新注册表 current_profile
        base::update_registry_current_profile_with_paths(
            &self.paths.registry_file,
            &self.registry_lock_dir(),
            "codex",
            name,
        )?;

        // 同步当前 OpenAI 账号指针，避免 profile/apply 与 auth registry 漂移
        let service = crate::services::CodexAuthService::from_dirs(
            self.paths.platform_dir.clone(),
            self.codex_dir(),
        );
        let _ = service.sync_current_auth_registry();

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
            && token.expose().trim().is_empty()
        {
            return Err(CcrError::ValidationError(
                "auth_token 不能为空字符串".into(),
            ));
        }

        match auth_mode {
            CodexProfileAuthMode::OpenAiApiKey => {
                if Self::trimmed_secret(profile.auth_token.as_ref()).is_none() {
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
                if Self::trimmed_secret(profile.auth_token.as_ref()).is_none() {
                    return Err(CcrError::ValidationError(
                        "provider_env_key 模式需要 auth_token".into(),
                    ));
                }
            }
            CodexProfileAuthMode::ProviderBearerToken => {
                if Self::trimmed_secret(profile.auth_token.as_ref()).is_none() {
                    return Err(CcrError::ValidationError(
                        "provider_bearer_token 模式需要 auth_token".into(),
                    ));
                }
                if Self::trimmed(profile.base_url.as_ref()).is_none() {
                    return Err(CcrError::ValidationError(
                        "provider_bearer_token 模式需要 base_url".into(),
                    ));
                }
            }
            CodexProfileAuthMode::OpenAiChatgpt | CodexProfileAuthMode::NoAuth => {}
        }

        if Self::is_official_profile(profile)
            && matches!(
                auth_mode,
                CodexProfileAuthMode::ProviderEnvKey
                    | CodexProfileAuthMode::ProviderBearerToken
                    | CodexProfileAuthMode::NoAuth
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
    use crate::test_support::TestCodexEnv;
    use serde_json::json;
    use std::path::Path;

    fn write_file_store_config(codex_dir: &Path) {
        std::fs::write(
            codex_dir.join("config.toml"),
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
            ..Default::default()
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
            auth_token: Some(ccr_core::Secret::from("sk-packy")),
            model: Some("gpt-4.1-mini".to_string()),
            small_fast_model: None,
            provider: Some("packycode".to_string()),
            provider_type: None,
            account: None,
            tags: None,
            usage_count: Some(0),
            enabled: Some(true),
            platform_data: IndexMap::new(),
            ..Default::default()
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
    fn deepseek_bearer_profile_roundtrip_is_idempotent_and_clearable() {
        let env = TestCodexEnv::new();
        write_file_store_config(env.codex_dir());
        let platform = CodexPlatform::new().unwrap();
        let bearer = "deepseek-test-secret";
        let catalog = env.codex_dir().join("models.json");
        std::fs::write(&catalog, "[]").unwrap();

        let mut profile = ProfileConfig {
            description: Some("DeepSeek".to_string()),
            base_url: Some("https://api.deepseek.com/".to_string()),
            auth_token: Some(Secret::from(bearer)),
            model: Some("deepseek-v4-flash".to_string()),
            provider: Some("deepseek".to_string()),
            provider_type: Some("third_party".to_string()),
            enabled: Some(true),
            ..Default::default()
        };
        profile.platform_data.insert(
            "auth_mode".into(),
            JsonValue::String("provider_bearer_token".into()),
        );
        profile.platform_data.insert(
            "model_catalog_json".into(),
            JsonValue::String(catalog.display().to_string()),
        );
        profile.platform_data.insert(
            "model_reasoning_effort".into(),
            JsonValue::String("high".into()),
        );

        platform.save_profile("deepseek", &profile).unwrap();
        let stored_profiles = std::fs::read_to_string(&platform.paths.profiles_file).unwrap();
        assert!(!stored_profiles.contains(bearer));

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let secret_store_mode = std::fs::metadata(platform.runtime_service.secret_store_path())
                .unwrap()
                .permissions()
                .mode()
                & 0o777;
            assert_eq!(secret_store_mode, 0o600);
        }

        platform.apply_profile("deepseek").unwrap();
        let first = std::fs::read_to_string(platform.config_manager.config_path()).unwrap();
        let config: toml::Value = toml::from_str(&first).unwrap();
        let root = config.as_table().unwrap();
        assert_eq!(
            root.get("model").and_then(toml::Value::as_str),
            Some("deepseek-v4-flash")
        );
        assert_eq!(
            root.get("model_catalog_json").and_then(toml::Value::as_str),
            Some(catalog.display().to_string().as_str())
        );
        assert_eq!(
            root.get("preferred_auth_method")
                .and_then(toml::Value::as_str),
            Some("apikey")
        );
        assert_eq!(
            root.get("forced_login_method")
                .and_then(toml::Value::as_str),
            Some("api")
        );
        let provider = CodexPlatform::current_custom_provider(&config).unwrap();
        assert_eq!(
            provider
                .get("experimental_bearer_token")
                .and_then(toml::Value::as_str),
            Some(bearer)
        );
        assert_eq!(
            provider
                .get("requires_openai_auth")
                .and_then(toml::Value::as_bool),
            Some(false)
        );
        assert!(provider.get("env_key").is_none());
        assert!(
            !platform
                .config_manager
                .load_auth()
                .unwrap()
                .contains_key("OPENAI_API_KEY")
        );

        let diagnostic = platform.inspect_runtime().unwrap();
        assert_eq!(diagnostic.route_status, RuntimeMatchStatus::Match);
        assert_eq!(diagnostic.credential_status, RuntimeMatchStatus::Match);
        assert_eq!(
            diagnostic.auth_source,
            CodexRuntimeAuthSource::ConfigBearerToken
        );

        platform.apply_profile("deepseek").unwrap();
        let second = std::fs::read_to_string(platform.config_manager.config_path()).unwrap();
        assert_eq!(first, second);

        platform.clear_active_profile_runtime().unwrap();
        let cleared = platform.config_manager.load_config().unwrap();
        let root = cleared.as_table().unwrap();
        assert!(root.get("model_catalog_json").is_none());
        assert!(root.get("preferred_auth_method").is_none());
        assert!(root.get("forced_login_method").is_none());
        assert!(
            CodexPlatform::current_custom_provider(&cleared)
                .unwrap()
                .get("experimental_bearer_token")
                .is_none()
        );
    }

    #[test]
    fn runtime_diagnostic_repairs_deepseek_fields_without_exposing_bearer() {
        const PROFILE_BEARER: &str = "deepseek-profile-bearer-must-not-leak";
        const DRIFTED_BEARER: &str = "deepseek-drifted-bearer-must-not-leak";

        let env = TestCodexEnv::new();
        write_file_store_config(env.codex_dir());
        let catalog = env.codex_dir().join("models.json");
        std::fs::write(&catalog, "[]").unwrap();
        let platform = CodexPlatform::new().unwrap();
        let profile = runtime_deepseek_profile(PROFILE_BEARER, &catalog);

        platform.save_profile("deepseek", &profile).unwrap();
        platform.apply_profile("deepseek").unwrap();

        let matched = platform.inspect_runtime_with_env(|_| None).unwrap();
        assert_eq!(matched.runtime_consistency(), RuntimeMatchStatus::Match);
        assert_diagnostic_excludes_secrets(&matched, &[PROFILE_BEARER, DRIFTED_BEARER]);

        for (field, drifted_value) in [
            ("model_catalog_json", "C:/missing/deepseek-models.json"),
            ("preferred_auth_method", "chatgpt"),
        ] {
            let mut config = platform.config_manager.load_config().unwrap();
            config
                .as_table_mut()
                .unwrap()
                .insert(field.into(), toml::Value::String(drifted_value.into()));
            platform.config_manager.save_config_atomic(&config).unwrap();

            let drifted = platform.inspect_runtime_with_env(|_| None).unwrap();
            assert_eq!(drifted.route_status, RuntimeMatchStatus::Mismatch);
            assert_eq!(drifted.credential_status, RuntimeMatchStatus::Match);
            assert!(drifted.repairable);
            assert_diagnostic_excludes_secrets(&drifted, &[PROFILE_BEARER, DRIFTED_BEARER]);

            platform
                .repair_runtime_with_env(&drifted, |_| None)
                .unwrap();
            let repaired = platform.inspect_runtime_with_env(|_| None).unwrap();
            assert_eq!(repaired.runtime_consistency(), RuntimeMatchStatus::Match);
        }

        let mut config = platform.config_manager.load_config().unwrap();
        config
            .as_table_mut()
            .unwrap()
            .get_mut("model_providers")
            .and_then(toml::Value::as_table_mut)
            .and_then(|providers| providers.get_mut(THIRD_PARTY_RUNTIME_PROVIDER_KEY))
            .and_then(toml::Value::as_table_mut)
            .unwrap()
            .insert(
                "experimental_bearer_token".into(),
                toml::Value::String(DRIFTED_BEARER.into()),
            );
        platform.config_manager.save_config_atomic(&config).unwrap();

        let drifted = platform.inspect_runtime_with_env(|_| None).unwrap();
        assert_eq!(drifted.route_status, RuntimeMatchStatus::Match);
        assert_eq!(drifted.credential_status, RuntimeMatchStatus::Mismatch);
        assert!(drifted.repairable);
        assert_diagnostic_excludes_secrets(&drifted, &[PROFILE_BEARER, DRIFTED_BEARER]);

        platform
            .repair_runtime_with_env(&drifted, |_| None)
            .unwrap();
        let repaired = platform.inspect_runtime_with_env(|_| None).unwrap();
        assert_eq!(repaired.runtime_consistency(), RuntimeMatchStatus::Match);
        assert_diagnostic_excludes_secrets(&repaired, &[PROFILE_BEARER, DRIFTED_BEARER]);
    }

    #[test]
    fn provider_bearer_mode_validates_preferred_auth_method() {
        let _env = TestCodexEnv::new();
        let platform = CodexPlatform::new().unwrap();
        let mut profile = ProfileConfig {
            base_url: Some("https://api.deepseek.com/".into()),
            auth_token: Some(Secret::from("test-secret")),
            provider_type: Some("third_party".into()),
            ..Default::default()
        };
        profile.platform_data.insert(
            "auth_mode".into(),
            JsonValue::String("provider_bearer_token".into()),
        );
        profile.platform_data.insert(
            "preferred_auth_method".into(),
            JsonValue::String("invalid".into()),
        );

        let error = platform.validate_profile(&profile).unwrap_err();
        assert!(error.to_string().contains("preferred_auth_method"));
    }

    #[test]
    fn test_legacy_github_profile_error() {
        let platform = CodexPlatform::new().unwrap();

        let mut github_profile = ProfileConfig {
            description: Some("GitHub Legacy".to_string()),
            base_url: Some("https://api.github.com".to_string()),
            auth_token: Some(ccr_core::Secret::from("ghp_1234567890abcdefghij")),
            model: Some("gpt-4".to_string()),
            small_fast_model: None,
            provider: Some("github".to_string()),
            provider_type: None,
            account: None,
            tags: None,
            usage_count: Some(0),
            enabled: Some(true),
            platform_data: IndexMap::new(),
            ..Default::default()
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
        let env = TestCodexEnv::new();
        let codex_dir = env.codex_dir();

        // CCR auth registry + snapshot
        let ccr_codex_dir = env.ccr_codex_dir();
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
                        api_base_url: None,
                        api_provider_name: None,
                        email: None,
                        plan_type: None,
                        saved_at: chrono::Utc::now(),
                        last_used: None,
                        last_refresh: None,
                        expires_at: None,
                    },
                );
                m
            },
            usage_ledger: Vec::new(),
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
        std::fs::create_dir_all(codex_dir).unwrap();
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
            auth_token: Some(ccr_core::Secret::from("sk-test")),
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
            ..Default::default()
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
    }

    #[test]
    fn test_save_profile_scrubs_secret_and_restores_on_load() {
        let env = TestCodexEnv::new();
        let ccr_root = env.root();

        let platform = CodexPlatform::new().unwrap();
        let mut profile = ProfileConfig {
            description: Some("Mistral Provider".to_string()),
            base_url: Some("https://api.mistral.ai/v1".to_string()),
            auth_token: Some(ccr_core::Secret::from("mistral-key-123")),
            model: Some("mistral-large".to_string()),
            small_fast_model: None,
            provider: Some("mistral".to_string()),
            provider_type: Some("third_party_model".to_string()),
            account: None,
            tags: None,
            usage_count: Some(0),
            enabled: Some(true),
            platform_data: IndexMap::new(),
            ..Default::default()
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
            loaded.get("mistral").unwrap().auth_token.as_ref(),
            Some(&ccr_core::Secret::from("mistral-key-123"))
        );
    }

    #[test]
    fn test_official_resets_config() {
        let _env = TestCodexEnv::new();

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
    }

    #[test]
    fn test_apply_official_profile_overlays_managed_fields() {
        let env = TestCodexEnv::new();
        write_file_store_config(env.codex_dir());

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
            ..Default::default()
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
    }

    #[test]
    fn test_apply_third_party_profile_succeeds_on_non_file_store() {
        let _env = TestCodexEnv::new();

        let platform = CodexPlatform::new().unwrap();
        let mut profile = ProfileConfig {
            description: Some("Mistral Provider".to_string()),
            base_url: Some("https://api.mistral.ai/v1".to_string()),
            auth_token: Some(ccr_core::Secret::from("mistral-key-123")),
            model: Some("mistral-large".to_string()),
            small_fast_model: None,
            provider: Some("mistral".to_string()),
            provider_type: None,
            account: None,
            tags: None,
            usage_count: Some(0),
            enabled: Some(true),
            platform_data: IndexMap::new(),
            ..Default::default()
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
    }

    #[test]
    fn test_third_party_preserves_fields() {
        let _env = TestCodexEnv::new();

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
    }

    #[test]
    fn test_apply_third_party_writes_config() {
        let env = TestCodexEnv::new();
        write_file_store_config(env.codex_dir());

        let platform = CodexPlatform::new().unwrap();
        let mut profile = ProfileConfig {
            description: Some("PackyCode Relay".to_string()),
            base_url: Some("https://api.packyapi.com/v1".to_string()),
            auth_token: Some(ccr_core::Secret::from("sk-packy")),
            model: Some("gpt-4.1-mini".to_string()),
            small_fast_model: None,
            provider: Some("packycode".to_string()),
            provider_type: None,
            account: None,
            tags: None,
            usage_count: Some(0),
            enabled: Some(true),
            platform_data: IndexMap::new(),
            ..Default::default()
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
            let config_path = env.codex_dir().join("config.toml");
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

            let auth_path = env.codex_dir().join("auth.json");
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
    }

    #[test]
    fn test_third_party_writes_env_key() {
        let env = TestCodexEnv::new();
        write_file_store_config(env.codex_dir());

        let platform = CodexPlatform::new().unwrap();
        let mut profile = ProfileConfig {
            description: Some("Mistral Provider".to_string()),
            base_url: Some("https://api.mistral.ai/v1".to_string()),
            auth_token: Some(ccr_core::Secret::from("mistral-key-123")),
            model: Some("mistral-large".to_string()),
            small_fast_model: None,
            provider: Some("mistral".to_string()),
            provider_type: None,
            account: None,
            tags: None,
            usage_count: Some(0),
            enabled: Some(true),
            platform_data: IndexMap::new(),
            ..Default::default()
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
            let config_path = env.codex_dir().join("config.toml");
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
            let auth_path = env.codex_dir().join("auth.json");
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
    }

    #[test]
    fn test_switching_from_openai_auth_to_provider_env_key_clears_openai_key() {
        let env = TestCodexEnv::new();
        write_file_store_config(env.codex_dir());

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
            auth_token: Some(ccr_core::Secret::from("mistral-key-123")),
            model: Some("mistral-large".to_string()),
            small_fast_model: None,
            provider: Some("mistral".to_string()),
            provider_type: None,
            account: None,
            tags: None,
            usage_count: Some(0),
            enabled: Some(true),
            platform_data: IndexMap::new(),
            ..Default::default()
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
    }

    #[test]
    fn test_third_party_default_auth_key() {
        let env = TestCodexEnv::new();
        write_file_store_config(env.codex_dir());

        let platform = CodexPlatform::new().unwrap();
        let mut profile = ProfileConfig {
            description: Some("OpenAI Compatible".to_string()),
            base_url: Some("https://api.openai-proxy.com/v1".to_string()),
            auth_token: Some(ccr_core::Secret::from("sk-proxy-key")),
            model: None,
            small_fast_model: None,
            provider: Some("proxy".to_string()),
            provider_type: None,
            account: None,
            tags: None,
            usage_count: Some(0),
            enabled: Some(true),
            platform_data: IndexMap::new(),
            ..Default::default()
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
            let auth_path = env.codex_dir().join("auth.json");
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
            let config_path = env.codex_dir().join("config.toml");
            if config_path.exists() {
                let content = std::fs::read_to_string(&config_path).unwrap();
                let parsed: toml::Value = toml::from_str(&content).unwrap();
                let root = parsed.as_table().unwrap();
                assert_eq!(
                    root.get("model_provider").and_then(|v| v.as_str()),
                    Some(THIRD_PARTY_RUNTIME_PROVIDER_KEY),
                    "third-party runtime provider should always be custom"
                );
                assert!(
                    root.get("forced_login_method").is_none(),
                    "auto-promote must not restrict future Codex login methods"
                );
                assert!(
                    !content.contains("env_key"),
                    "should not write env_key when not provided, got: {}",
                    content
                );
            }
        }
    }

    #[test]
    fn test_third_party_default_auth_key_clears_stale_chatgpt_auth_metadata() {
        let env = TestCodexEnv::new();
        write_file_store_config(env.codex_dir());

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
            auth_token: Some(ccr_core::Secret::from("sk-proxy-key")),
            model: None,
            small_fast_model: None,
            provider: Some("proxy".to_string()),
            provider_type: None,
            account: None,
            tags: None,
            usage_count: Some(0),
            enabled: Some(true),
            platform_data: IndexMap::new(),
            ..Default::default()
        };
        profile
            .platform_data
            .insert("wire_api".into(), json!("responses"));

        platform
            .apply_third_party_profile("proxy", &profile)
            .unwrap();

        let auth_path = env.codex_dir().join("auth.json");
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
        assert!(
            root.get("forced_login_method").is_none(),
            "API-key switching must not leave a forced login-method policy"
        );
    }

    #[test]
    fn test_apply_profile_preserves_explicit_forced_login_method() {
        let env = TestCodexEnv::new();
        write_file_store_config(env.codex_dir());

        let platform = CodexPlatform::new().unwrap();
        let mut profile = ProfileConfig {
            description: Some("Restricted Proxy".to_string()),
            base_url: Some("https://api.proxy.example/v1".to_string()),
            auth_token: Some(ccr_core::Secret::from("sk-proxy-key")),
            provider: Some("proxy".to_string()),
            enabled: Some(true),
            ..Default::default()
        };
        profile
            .platform_data
            .insert("wire_api".into(), json!("responses"));
        profile
            .platform_data
            .insert("auth_mode".into(), json!("openai_api_key"));
        profile
            .platform_data
            .insert("forced_login_method".into(), json!("api"));

        platform.save_profile("restricted-proxy", &profile).unwrap();
        platform.apply_profile("restricted-proxy").unwrap();

        let config = CodexConfigManager::with_default()
            .unwrap()
            .load_config()
            .unwrap();
        assert_eq!(
            config
                .as_table()
                .unwrap()
                .get("forced_login_method")
                .and_then(|value| value.as_str()),
            Some("api")
        );
    }

    #[test]
    fn test_provider_env_key_profile_does_not_apply_forced_login_method() {
        let env = TestCodexEnv::new();
        std::fs::write(
            env.codex_dir().join("config.toml"),
            "forced_login_method = \"chatgpt\"\n",
        )
        .unwrap();

        let platform = CodexPlatform::new().unwrap();
        let mut profile = ProfileConfig {
            description: Some("Mistral Provider".to_string()),
            base_url: Some("https://api.mistral.ai/v1".to_string()),
            auth_token: Some(ccr_core::Secret::from("mistral-key-123")),
            provider: Some("mistral".to_string()),
            enabled: Some(true),
            ..Default::default()
        };
        profile
            .platform_data
            .insert("wire_api".into(), json!("responses"));
        profile
            .platform_data
            .insert("auth_mode".into(), json!("provider_env_key"));
        profile
            .platform_data
            .insert("env_key".into(), json!("MISTRAL_API_KEY"));
        profile
            .platform_data
            .insert("forced_login_method".into(), json!("api"));

        platform
            .apply_third_party_profile("mistral", &profile)
            .unwrap();

        let config = CodexConfigManager::with_default()
            .unwrap()
            .load_config()
            .unwrap();
        assert!(
            config
                .as_table()
                .unwrap()
                .get("forced_login_method")
                .is_none(),
            "provider env-key profiles must not apply an OpenAI login-method policy"
        );
    }

    #[test]
    fn test_provider_model_explicit_is_ignored() {
        let env = TestCodexEnv::new();
        write_file_store_config(env.codex_dir());

        let platform = CodexPlatform::new().unwrap();
        let mut profile = ProfileConfig {
            description: Some("Custom Provider".to_string()),
            base_url: Some("https://api.example.com/v1".to_string()),
            auth_token: Some(ccr_core::Secret::from("sk-test")),
            model: Some("gpt-4".to_string()),
            small_fast_model: None,
            provider: Some("custom".to_string()),
            provider_type: None,
            account: None,
            tags: None,
            usage_count: Some(0),
            enabled: Some(true),
            platform_data: IndexMap::new(),
            ..Default::default()
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
            let config_path = env.codex_dir().join("config.toml");
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
    }

    #[test]
    fn test_target_gpt_5_6_models_round_trip_and_apply_without_runtime_allowlist() {
        let env = TestCodexEnv::new();
        write_file_store_config(env.codex_dir());
        let platform = CodexPlatform::new().unwrap();

        for model in ["gpt-5.6-luna", "gpt-5.6-terra", "gpt-5.6-sol"] {
            let profile_name = format!("target-{}", model.replace('.', "-"));
            let mut profile = ProfileConfig {
                description: Some("GPT-5.6 relay".to_string()),
                base_url: Some("https://api.example.com/v1".to_string()),
                model: Some(model.to_string()),
                provider: Some("custom".to_string()),
                enabled: Some(true),
                ..Default::default()
            };
            profile
                .platform_data
                .insert("wire_api".into(), json!("responses"));

            platform.save_profile(&profile_name, &profile).unwrap();
            let loaded_profiles = platform.load_profiles().unwrap();
            assert_eq!(
                loaded_profiles
                    .get(&profile_name)
                    .and_then(|saved| saved.model.as_deref()),
                Some(model)
            );

            platform.apply_profile(&profile_name).unwrap();

            let config = CodexConfigManager::with_default()
                .unwrap()
                .load_config()
                .unwrap();
            let root = config.as_table().unwrap();
            assert_eq!(
                root.get("model").and_then(|value| value.as_str()),
                Some(model)
            );

            let provider = root
                .get("model_providers")
                .and_then(|value| value.as_table())
                .and_then(|providers| providers.get(THIRD_PARTY_RUNTIME_PROVIDER_KEY))
                .and_then(|value| value.as_table())
                .unwrap();
            assert_eq!(
                provider.get("wire_api").and_then(|value| value.as_str()),
                Some("responses")
            );
        }
    }

    #[test]
    fn test_third_party_clears_stale_optional_provider_fields() {
        let env = TestCodexEnv::new();
        write_file_store_config(env.codex_dir());

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
            auth_token: Some(ccr_core::Secret::from("sk-test")),
            model: Some("new-root-model".to_string()),
            small_fast_model: None,
            provider: Some("custom".to_string()),
            provider_type: None,
            account: None,
            tags: None,
            usage_count: Some(0),
            enabled: Some(true),
            platform_data: IndexMap::new(),
            ..Default::default()
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
    }

    #[test]
    fn test_requires_openai_auth_ignores_env_key_and_clears_provider_token() {
        let env = TestCodexEnv::new();
        write_file_store_config(env.codex_dir());

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
            ..Default::default()
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
    }

    #[test]
    fn runtime_diagnostic_matches_api_key_without_exposing_secret() {
        const TEST_SECRET: &str = "diagnostic-secret-must-not-leak";
        let _env = TestCodexEnv::new();
        let platform = CodexPlatform::new().unwrap();
        let profile = runtime_api_key_profile(TEST_SECRET);

        platform.save_profile("future", &profile).unwrap();
        platform.apply_profile("future").unwrap();

        let diagnostic = platform.inspect_runtime_with_env(|_| None).unwrap();
        assert_eq!(diagnostic.resolved_profile.as_deref(), Some("future"));
        assert_eq!(diagnostic.profile_status, RuntimeMatchStatus::Match);
        assert_eq!(diagnostic.route_status, RuntimeMatchStatus::Match);
        assert_eq!(diagnostic.credential_status, RuntimeMatchStatus::Match);
        assert_eq!(
            diagnostic.provider_auth_validity,
            ProviderAuthValidity::NotChecked
        );
        assert!(!diagnostic.repairable);

        let serialized = serde_json::to_string(&diagnostic).unwrap();
        let debug = format!("{diagnostic:?}");
        assert!(!serialized.contains(TEST_SECRET));
        assert!(!debug.contains(TEST_SECRET));
    }

    #[test]
    fn runtime_diagnostic_repairs_api_key_mismatch_and_missing_key() {
        const TEST_SECRET: &str = "repair-secret-must-not-leak";
        let env = TestCodexEnv::new();
        let platform = CodexPlatform::new().unwrap();
        let profile = runtime_api_key_profile(TEST_SECRET);

        platform.save_profile("future", &profile).unwrap();
        platform.apply_profile("future").unwrap();
        std::fs::write(
            env.codex_dir().join("auth.json"),
            r#"{"OPENAI_API_KEY":"different-test-value"}"#,
        )
        .unwrap();

        let mismatch = platform.inspect_runtime_with_env(|_| None).unwrap();
        assert_eq!(mismatch.credential_status, RuntimeMatchStatus::Mismatch);
        assert!(mismatch.repairable);
        platform
            .repair_runtime_with_env(&mismatch, |_| None)
            .unwrap();

        let repaired = platform.inspect_runtime_with_env(|_| None).unwrap();
        assert_eq!(repaired.runtime_consistency(), RuntimeMatchStatus::Match);
        let auth = CodexConfigManager::with_default()
            .unwrap()
            .load_auth()
            .unwrap();
        let repaired_value_matches = auth
            .get("OPENAI_API_KEY")
            .and_then(serde_json::Value::as_str)
            == Some(TEST_SECRET);
        assert!(repaired_value_matches);

        std::fs::remove_file(env.codex_dir().join("auth.json")).unwrap();
        let missing = platform.inspect_runtime_with_env(|_| None).unwrap();
        assert_eq!(missing.credential_status, RuntimeMatchStatus::Missing);
        assert!(missing.repairable);
    }

    #[test]
    fn runtime_diagnostic_requires_provider_env_key_without_leaking_it() {
        const TEST_SECRET: &str = "provider-env-secret-must-not-leak";
        let _env = TestCodexEnv::new();
        let platform = CodexPlatform::new().unwrap();
        let mut profile = ProfileConfig {
            description: Some("Mistral Provider".to_string()),
            base_url: Some("https://api.mistral.ai/v1".to_string()),
            auth_token: Some(ccr_core::Secret::from(TEST_SECRET)),
            model: Some("mistral-large".to_string()),
            provider_type: Some("third_party_model".to_string()),
            enabled: Some(true),
            ..Default::default()
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
        platform.apply_profile("mistral").unwrap();

        let matched = platform
            .inspect_runtime_with_env(|name| {
                (name == "MISTRAL_API_KEY").then(|| TEST_SECRET.to_string())
            })
            .unwrap();
        assert_eq!(matched.credential_status, RuntimeMatchStatus::Match);

        let missing = platform.inspect_runtime_with_env(|_| None).unwrap();
        assert_eq!(missing.credential_status, RuntimeMatchStatus::Missing);
        assert!(!missing.repairable);
        let serialized = serde_json::to_string(&missing).unwrap();
        assert!(!serialized.contains(TEST_SECRET));
    }

    #[test]
    fn runtime_diagnostic_reports_runtime_and_expected_env_keys_during_route_drift() {
        const TEST_SECRET: &str = "expected-env-secret-must-not-leak";
        let _env = TestCodexEnv::new();
        let platform = CodexPlatform::new().unwrap();
        let mut profile = ProfileConfig {
            description: Some("Expected Provider".to_string()),
            base_url: Some("https://expected.example/v1".to_string()),
            auth_token: Some(ccr_core::Secret::from(TEST_SECRET)),
            provider_type: Some("third_party_model".to_string()),
            enabled: Some(true),
            ..Default::default()
        };
        profile
            .platform_data
            .insert("wire_api".into(), json!("responses"));
        profile
            .platform_data
            .insert("env_key".into(), json!("EXPECTED_API_KEY"));
        profile
            .platform_data
            .insert("auth_mode".into(), json!("provider_env_key"));

        platform.save_profile("expected", &profile).unwrap();
        platform.apply_profile("expected").unwrap();

        let manager = CodexConfigManager::with_default().unwrap();
        let mut config = manager.load_config().unwrap();
        let provider = config
            .as_table_mut()
            .and_then(|root| root.get_mut("model_providers"))
            .and_then(toml::Value::as_table_mut)
            .and_then(|providers| providers.get_mut(THIRD_PARTY_RUNTIME_PROVIDER_KEY))
            .and_then(toml::Value::as_table_mut)
            .unwrap();
        provider.insert(
            "base_url".into(),
            toml::Value::String("https://runtime.example/v1".into()),
        );
        provider.insert(
            "env_key".into(),
            toml::Value::String("RUNTIME_API_KEY".into()),
        );
        manager.save_config_atomic(&config).unwrap();

        let diagnostic = platform
            .inspect_runtime_with_env(|name| match name {
                "EXPECTED_API_KEY" => Some(TEST_SECRET.to_string()),
                "RUNTIME_API_KEY" => Some("runtime-only-value".to_string()),
                _ => None,
            })
            .unwrap();

        assert_eq!(diagnostic.route_status, RuntimeMatchStatus::Mismatch);
        assert_eq!(diagnostic.credential_status, RuntimeMatchStatus::Match);
        assert_eq!(
            diagnostic.auth_source,
            CodexRuntimeAuthSource::Environment {
                variable: "RUNTIME_API_KEY".to_string()
            }
        );
        for variable in ["EXPECTED_API_KEY", "RUNTIME_API_KEY"] {
            assert!(
                diagnostic
                    .environment
                    .iter()
                    .any(|presence| presence.variable == variable && presence.is_set)
            );
        }
        assert!(
            !serde_json::to_string(&diagnostic)
                .unwrap()
                .contains(TEST_SECRET)
        );
    }

    #[test]
    fn runtime_diagnostic_marks_keyring_credentials_as_unreadable() {
        let _env = TestCodexEnv::new();
        let platform = CodexPlatform::new().unwrap();
        platform
            .save_profile("future", &runtime_api_key_profile("keyring-secret"))
            .unwrap();
        platform.apply_profile("future").unwrap();

        let manager = CodexConfigManager::with_default().unwrap();
        let mut config = manager.load_config().unwrap();
        config.as_table_mut().unwrap().insert(
            "cli_auth_credentials_store".into(),
            toml::Value::String("keyring".into()),
        );
        manager.save_config_atomic(&config).unwrap();

        let diagnostic = platform.inspect_runtime_with_env(|_| None).unwrap();
        assert_eq!(diagnostic.credential_store, CredentialStoreKind::Keyring);
        assert_eq!(
            diagnostic.auth_source,
            CodexRuntimeAuthSource::KeyringUnreadable
        );
        assert_eq!(
            diagnostic.credential_status,
            RuntimeMatchStatus::Unsupported
        );
        assert!(!diagnostic.repairable);
    }

    #[test]
    fn runtime_diagnostic_reports_runtime_only_when_no_profile_is_current() {
        let _env = TestCodexEnv::new();
        let platform = CodexPlatform::new().unwrap();
        platform
            .save_profile("future", &runtime_api_key_profile("unused-secret"))
            .unwrap();
        platform.apply_profile("future").unwrap();

        let registry_manager = PlatformConfigManager::new(&platform.paths.registry_file);
        let mut registry = registry_manager.load().unwrap();
        registry.set_current_profile("codex", None).unwrap();
        registry_manager.save(&registry).unwrap();

        let profiles_text = std::fs::read_to_string(&platform.paths.profiles_file).unwrap();
        let mut profiles_config = toml::from_str::<CcsConfig>(&profiles_text).unwrap();
        profiles_config.current_config.clear();
        std::fs::write(
            &platform.paths.profiles_file,
            toml::to_string_pretty(&profiles_config).unwrap(),
        )
        .unwrap();

        let diagnostic = platform.inspect_runtime_with_env(|_| None).unwrap();
        assert!(diagnostic.resolved_profile.is_none());
        assert_eq!(diagnostic.profile_status, RuntimeMatchStatus::NotApplicable);
        assert_eq!(
            diagnostic.runtime_consistency(),
            RuntimeMatchStatus::NotApplicable
        );
        assert!(!diagnostic.repairable);
    }

    #[test]
    fn runtime_diagnostic_treats_no_auth_as_not_applicable_credential() {
        let _env = TestCodexEnv::new();
        let platform = CodexPlatform::new().unwrap();
        let mut profile = ProfileConfig {
            description: Some("No-auth Provider".to_string()),
            base_url: Some("https://no-auth.example/v1".to_string()),
            provider_type: Some("third_party_model".to_string()),
            enabled: Some(true),
            ..Default::default()
        };
        profile
            .platform_data
            .insert("wire_api".into(), json!("responses"));
        profile
            .platform_data
            .insert("auth_mode".into(), json!("no_auth"));

        platform.save_profile("no-auth", &profile).unwrap();
        platform.apply_profile("no-auth").unwrap();

        let diagnostic = platform.inspect_runtime_with_env(|_| None).unwrap();
        assert_eq!(diagnostic.route_status, RuntimeMatchStatus::Match);
        assert_eq!(
            diagnostic.credential_status,
            RuntimeMatchStatus::NotApplicable
        );
        assert_eq!(diagnostic.runtime_consistency(), RuntimeMatchStatus::Match);
        assert!(!diagnostic.repairable);
    }

    #[test]
    fn runtime_diagnostic_preserves_conflicting_profile_pointers() {
        let _env = TestCodexEnv::new();
        let platform = CodexPlatform::new().unwrap();
        platform
            .save_profile("future", &runtime_api_key_profile("future-secret"))
            .unwrap();
        platform.apply_profile("future").unwrap();
        platform
            .save_profile("other", &runtime_api_key_profile("other-secret"))
            .unwrap();

        let manager = PlatformConfigManager::new(&platform.paths.registry_file);
        let mut registry = manager.load().unwrap();
        registry
            .set_current_profile("codex", Some("other"))
            .unwrap();
        manager.save(&registry).unwrap();

        let diagnostic = platform.inspect_runtime_with_env(|_| None).unwrap();
        assert_eq!(diagnostic.registry_profile.as_deref(), Some("other"));
        assert_eq!(diagnostic.profiles_file_profile.as_deref(), Some("future"));
        assert_eq!(diagnostic.profile_status, RuntimeMatchStatus::Mismatch);
        assert!(diagnostic.resolved_profile.is_none());
        assert!(!diagnostic.repairable);
        assert_eq!(
            platform.current_profile_from_registry().unwrap().as_deref(),
            Some("other")
        );
    }

    #[test]
    fn runtime_diagnostic_sanitizes_url_credentials_and_rejects_invalid_env_names() {
        const URL_SECRET: &str = "url-secret-must-not-leak";
        let _env = TestCodexEnv::new();
        let platform = CodexPlatform::new().unwrap();
        let mut profile = runtime_api_key_profile("profile-secret");
        profile.base_url = Some(format!(
            "https://user:{URL_SECRET}@example.com/v1?key={URL_SECRET}"
        ));

        platform.save_profile("future", &profile).unwrap();
        platform.apply_profile("future").unwrap();
        let diagnostic = platform.inspect_runtime_with_env(|_| None).unwrap();
        assert_eq!(
            diagnostic.base_url.as_deref(),
            Some("https://example.com/v1")
        );
        let serialized = serde_json::to_string(&diagnostic).unwrap();
        assert!(!serialized.contains(URL_SECRET));

        let mut invalid_env_profile = ProfileConfig {
            description: Some("Invalid env".to_string()),
            base_url: Some("https://example.com/v1".to_string()),
            auth_token: Some(ccr_core::Secret::from("provider-secret")),
            provider_type: Some("third_party_model".to_string()),
            ..Default::default()
        };
        invalid_env_profile
            .platform_data
            .insert("wire_api".into(), json!("responses"));
        invalid_env_profile
            .platform_data
            .insert("auth_mode".into(), json!("provider_env_key"));
        invalid_env_profile
            .platform_data
            .insert("env_key".into(), json!("secret=value"));
        let error = platform.validate_profile(&invalid_env_profile).unwrap_err();
        assert!(!error.to_string().contains("secret=value"));
    }

    fn runtime_api_key_profile(secret: &str) -> ProfileConfig {
        let mut profile = ProfileConfig {
            description: Some("Future Provider".to_string()),
            base_url: Some("https://www.futureapi.cc/v1".to_string()),
            auth_token: Some(ccr_core::Secret::from(secret)),
            model: Some("gpt-5.6-sol".to_string()),
            provider_type: Some("third_party_model".to_string()),
            enabled: Some(true),
            ..Default::default()
        };
        profile
            .platform_data
            .insert("wire_api".into(), json!("responses"));
        profile
            .platform_data
            .insert("auth_mode".into(), json!("openai_api_key"));
        profile
    }

    fn runtime_deepseek_profile(secret: &str, catalog: &Path) -> ProfileConfig {
        let mut profile = ProfileConfig {
            description: Some("DeepSeek".to_string()),
            base_url: Some("https://api.deepseek.com/".to_string()),
            auth_token: Some(Secret::from(secret)),
            model: Some("deepseek-v4-flash".to_string()),
            provider: Some("deepseek".to_string()),
            provider_type: Some("third_party_model".to_string()),
            enabled: Some(true),
            ..Default::default()
        };
        profile
            .platform_data
            .insert("wire_api".into(), json!("responses"));
        profile
            .platform_data
            .insert("auth_mode".into(), json!("provider_bearer_token"));
        profile.platform_data.insert(
            "model_catalog_json".into(),
            json!(catalog.display().to_string()),
        );
        profile
            .platform_data
            .insert("model_reasoning_effort".into(), json!("high"));
        profile
    }

    fn assert_diagnostic_excludes_secrets(diagnostic: &CodexRuntimeDiagnostic, secrets: &[&str]) {
        let serialized = serde_json::to_string(diagnostic).unwrap();
        let debug = format!("{diagnostic:?}");
        for secret in secrets {
            assert!(!serialized.contains(secret));
            assert!(!debug.contains(secret));
        }
    }
}
