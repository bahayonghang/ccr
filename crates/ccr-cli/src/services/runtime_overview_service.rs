use super::claude_auth_service::ClaudeAuthService;
use crate::managers::ConfigManager;
use crate::models::{
    AuthIntent, AuthStateStatus, ClaudeLoginState, ClaudeProfileAuthMode, ClaudeRuntimeMode,
    ClaudeRuntimeSummary, CodexProfileAuthMode, CodexRuntimeMode, CodexRuntimeSummary, LoginState,
    OpenAiAuthMethod, Platform, ProfileConfig,
};
use crate::platforms::create_platform;
use ccr_codex::CodexAuthService;
use ccr_core::core::error::Result;
use chrono::{DateTime, Utc};
use serde::Serialize;

pub const RUNTIME_OVERVIEW_SCHEMA_VERSION: u32 = 2;

#[derive(Debug, Serialize)]
pub struct RuntimeOverview {
    pub schema_version: u32,
    pub generated_at: DateTime<Utc>,
    pub claude: PlatformStatusCard,
    pub codex: PlatformStatusCard,
}

#[derive(Debug, Serialize)]
pub struct PlatformStatusCard {
    pub platform: String,
    pub display_name: String,
    pub profile: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub small_fast_model: Option<String>,
    pub auth: String,
    pub auth_kind: StatusAuthKind,
    pub health: StatusHealth,
    pub note: String,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StatusAuthKind {
    OfficialAuth,
    ThirdPartyApi,
    ProviderKey,
    NoAuth,
    Missing,
    Unknown,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StatusHealth {
    Ready,
    NeedsLogin,
    Invalid,
    Unsupported,
    Error,
}

pub struct RuntimeOverviewService;

impl RuntimeOverviewService {
    pub fn load() -> Result<RuntimeOverview> {
        Ok(RuntimeOverview {
            schema_version: RUNTIME_OVERVIEW_SCHEMA_VERSION,
            generated_at: Utc::now(),
            claude: build_claude_status_card(),
            codex: build_codex_status_card(),
        })
    }
}

fn build_claude_status_card() -> PlatformStatusCard {
    let Ok(service) = ClaudeAuthService::new() else {
        return PlatformStatusCard::error(
            Platform::Claude,
            "无法访问 Claude 配置或认证文件".to_string(),
        );
    };

    match service.get_runtime_summary() {
        Ok(summary) => {
            let profile = load_profile(Platform::Claude, summary.current_profile_name.as_deref());
            status_card_from_claude_summary(summary, profile.as_ref())
        }
        Err(error) => PlatformStatusCard::error(Platform::Claude, error.to_string()),
    }
}

fn build_codex_status_card() -> PlatformStatusCard {
    let Ok(service) = CodexAuthService::new() else {
        return PlatformStatusCard::error(
            Platform::Codex,
            "无法访问 Codex 配置或认证文件".to_string(),
        );
    };

    match service.get_runtime_summary() {
        Ok(summary) => {
            let (fallback_name, fallback_profile) = current_profile_from_file(Platform::Codex);
            let profile = load_profile(Platform::Codex, summary.current_profile_name.as_deref())
                .or(fallback_profile);
            status_card_from_codex_summary(summary, profile.as_ref(), fallback_name.as_deref())
        }
        Err(error) => PlatformStatusCard::error(Platform::Codex, error.to_string()),
    }
}

fn load_profile(platform: Platform, profile_name: Option<&str>) -> Option<ProfileConfig> {
    let profile_name = profile_name?;
    let platform_config = create_platform(platform).ok()?;
    let profiles = platform_config.load_profiles().ok()?;
    profiles.get(profile_name).cloned()
}

pub(crate) fn current_profile_from_file(
    platform: Platform,
) -> (Option<String>, Option<ProfileConfig>) {
    let Ok(manager) = ConfigManager::for_platform(platform.short_name()) else {
        return (None, None);
    };
    let Ok(config) = manager.load_with_autofix() else {
        return (None, None);
    };

    let name = config.current_config.trim();
    if name.is_empty() {
        return (None, None);
    }

    let profile = config
        .sections
        .get(name)
        .map(ccr_config::section_to_profile);

    (Some(name.to_string()), profile)
}

fn status_card_from_claude_summary(
    summary: ClaudeRuntimeSummary,
    profile: Option<&ProfileConfig>,
) -> PlatformStatusCard {
    let (auth, auth_kind) = claude_auth_display(&summary);
    let health = claude_health(&summary);
    let note = claude_note(&summary);

    PlatformStatusCard {
        platform: Platform::Claude.short_name().to_string(),
        display_name: Platform::Claude.display_name().to_string(),
        profile: summary
            .current_profile_name
            .clone()
            .unwrap_or_else(|| "未绑定".to_string()),
        provider: profile
            .and_then(|profile| profile.provider.clone())
            .or(summary.current_profile_provider.clone()),
        base_url: profile.and_then(|profile| profile.base_url.clone()),
        model: profile.and_then(|profile| profile.model.clone()),
        small_fast_model: profile.and_then(|profile| profile.small_fast_model.clone()),
        auth,
        auth_kind,
        health,
        note,
    }
}

fn status_card_from_codex_summary(
    summary: CodexRuntimeSummary,
    profile: Option<&ProfileConfig>,
    fallback_profile_name: Option<&str>,
) -> PlatformStatusCard {
    let (auth, auth_kind) = codex_auth_display(&summary, profile);
    let health = codex_health(&summary);
    let used_profile_fallback =
        summary.current_profile_name.is_none() && fallback_profile_name.is_some();
    let note = codex_note(
        &summary,
        used_profile_fallback,
        profile_has_auth_token(profile),
    );

    PlatformStatusCard {
        platform: Platform::Codex.short_name().to_string(),
        display_name: Platform::Codex.display_name().to_string(),
        profile: summary
            .current_profile_name
            .clone()
            .or_else(|| fallback_profile_name.map(str::to_string))
            .unwrap_or_else(|| "未绑定".to_string()),
        provider: profile
            .and_then(|profile| profile.provider.clone())
            .or(summary.current_profile_provider.clone()),
        base_url: profile.and_then(|profile| profile.base_url.clone()),
        model: profile.and_then(|profile| profile.model.clone()),
        small_fast_model: profile.and_then(|profile| profile.small_fast_model.clone()),
        auth,
        auth_kind,
        health,
        note,
    }
}

impl PlatformStatusCard {
    fn error(platform: Platform, note: String) -> Self {
        Self {
            platform: platform.short_name().to_string(),
            display_name: platform.display_name().to_string(),
            profile: "未解析".to_string(),
            provider: None,
            base_url: None,
            model: None,
            small_fast_model: None,
            auth: "未解析".to_string(),
            auth_kind: StatusAuthKind::Unknown,
            health: StatusHealth::Error,
            note,
        }
    }
}

fn claude_auth_display(summary: &ClaudeRuntimeSummary) -> (String, StatusAuthKind) {
    match summary.mode {
        ClaudeRuntimeMode::ProfileOnly => match summary.current_profile_auth_mode {
            Some(ClaudeProfileAuthMode::ApiKey) => {
                let source = summary
                    .current_profile_auth_source
                    .as_deref()
                    .and_then(provider_suffix)
                    .unwrap_or("ANTHROPIC_AUTH_TOKEN");
                (
                    format!("第三方 API: {source}"),
                    StatusAuthKind::ThirdPartyApi,
                )
            }
            Some(ClaudeProfileAuthMode::Subscription) => {
                ("官方 Auth: 等待登录".to_string(), StatusAuthKind::Missing)
            }
            None => ("未解析".to_string(), StatusAuthKind::Unknown),
        },
        ClaudeRuntimeMode::ProfileWithAuth | ClaudeRuntimeMode::RuntimeOnly => {
            let name = summary
                .current_auth_name
                .as_deref()
                .or(summary.current_login_name.as_deref())
                .unwrap_or("已登录账号");
            (format!("官方 Auth: {name}"), StatusAuthKind::OfficialAuth)
        }
        ClaudeRuntimeMode::ProfilePendingAuth => (
            "官方 Auth: 需要 claude login".to_string(),
            StatusAuthKind::Missing,
        ),
        ClaudeRuntimeMode::Unresolved => ("未解析".to_string(), StatusAuthKind::Unknown),
    }
}

fn codex_auth_display(
    summary: &CodexRuntimeSummary,
    profile: Option<&ProfileConfig>,
) -> (String, StatusAuthKind) {
    let provider = summary
        .current_profile_provider
        .as_deref()
        .or_else(|| profile.and_then(|profile| profile.provider.as_deref()))
        .map(str::trim)
        .filter(|value| !value.is_empty());

    match &summary.login_state {
        LoginState::LoggedInSaved(name) => {
            return (
                format!("官方 Auth: {name}"),
                codex_openai_kind(&summary.auth_state.intent),
            );
        }
        LoginState::LoggedInUnsaved => {
            return (
                "官方 Auth: 已登录未保存".to_string(),
                codex_openai_kind(&summary.auth_state.intent),
            );
        }
        LoginState::ApiKeyActive => {
            let source = provider.unwrap_or("OPENAI_API_KEY");
            return (
                format!("第三方 API: {source}"),
                StatusAuthKind::ThirdPartyApi,
            );
        }
        LoginState::ProviderKeyActive { env_key } => {
            return (
                format!("Provider Key: {env_key}"),
                StatusAuthKind::ProviderKey,
            );
        }
        LoginState::Unknown { type_name, .. } => {
            return (
                format!("未知认证状态: {type_name}"),
                StatusAuthKind::Unknown,
            );
        }
        LoginState::NotLoggedIn => {}
    }

    if profile_has_auth_token(profile) {
        return (
            format!("第三方 API: {}", provider.unwrap_or("OPENAI_API_KEY")),
            StatusAuthKind::ThirdPartyApi,
        );
    }

    match summary.current_profile_auth_mode {
        Some(CodexProfileAuthMode::OpenAiChatgpt) => (
            "官方 Auth: 需要 codex login".to_string(),
            StatusAuthKind::Missing,
        ),
        Some(CodexProfileAuthMode::OpenAiApiKey) => (
            format!("第三方 API: {}", provider.unwrap_or("OPENAI_API_KEY")),
            StatusAuthKind::ThirdPartyApi,
        ),
        Some(CodexProfileAuthMode::ProviderEnvKey) => {
            let source = summary
                .current_profile_auth_source
                .as_deref()
                .and_then(provider_suffix)
                .unwrap_or("provider env");
            (
                format!("Provider Key: {source}"),
                StatusAuthKind::ProviderKey,
            )
        }
        Some(CodexProfileAuthMode::ProviderBearerToken) => (
            "Provider Bearer Token".to_string(),
            StatusAuthKind::ProviderKey,
        ),
        Some(CodexProfileAuthMode::NoAuth) => ("No Auth".to_string(), StatusAuthKind::NoAuth),
        None => ("未解析".to_string(), StatusAuthKind::Unknown),
    }
}

fn codex_openai_kind(intent: &AuthIntent) -> StatusAuthKind {
    match intent {
        AuthIntent::OpenAiAuth {
            method: OpenAiAuthMethod::Chatgpt,
        } => StatusAuthKind::OfficialAuth,
        AuthIntent::OpenAiAuth {
            method: OpenAiAuthMethod::Api,
        } => StatusAuthKind::ThirdPartyApi,
        AuthIntent::ProviderEnvKey { .. } => StatusAuthKind::ProviderKey,
        AuthIntent::ProviderBearerToken => StatusAuthKind::ProviderKey,
        AuthIntent::NoAuth => StatusAuthKind::NoAuth,
    }
}

fn claude_health(summary: &ClaudeRuntimeSummary) -> StatusHealth {
    match summary.mode {
        ClaudeRuntimeMode::ProfileOnly | ClaudeRuntimeMode::ProfileWithAuth => StatusHealth::Ready,
        ClaudeRuntimeMode::RuntimeOnly => match summary.login_state {
            ClaudeLoginState::NotLoggedIn => StatusHealth::NeedsLogin,
            _ => StatusHealth::Ready,
        },
        ClaudeRuntimeMode::ProfilePendingAuth => StatusHealth::NeedsLogin,
        ClaudeRuntimeMode::Unresolved => StatusHealth::Invalid,
    }
}

fn codex_health(summary: &CodexRuntimeSummary) -> StatusHealth {
    if matches!(summary.auth_state.status, AuthStateStatus::Unsupported) {
        return StatusHealth::Unsupported;
    }

    match summary.mode {
        CodexRuntimeMode::ProfileOnly | CodexRuntimeMode::ProfileWithAuth => StatusHealth::Ready,
        CodexRuntimeMode::RuntimeOnly => match summary.auth_state.status {
            AuthStateStatus::Valid => StatusHealth::Ready,
            AuthStateStatus::Missing => StatusHealth::NeedsLogin,
            AuthStateStatus::Invalid => StatusHealth::Invalid,
            AuthStateStatus::Unsupported => StatusHealth::Unsupported,
        },
        CodexRuntimeMode::ProfilePendingAuth => StatusHealth::NeedsLogin,
        CodexRuntimeMode::Unresolved => StatusHealth::Invalid,
    }
}

fn claude_note(summary: &ClaudeRuntimeSummary) -> String {
    match summary.mode {
        ClaudeRuntimeMode::ProfileOnly => {
            "当前由 profile/API key 控制，不使用官方订阅凭据".to_string()
        }
        ClaudeRuntimeMode::ProfileWithAuth => "Profile 控制路由，官方 Auth 控制身份".to_string(),
        ClaudeRuntimeMode::ProfilePendingAuth => {
            "Profile 需要官方订阅凭据，但当前未检测到登录".to_string()
        }
        ClaudeRuntimeMode::RuntimeOnly => {
            "当前仅检测到官方 Auth 运行时，未绑定 profile".to_string()
        }
        ClaudeRuntimeMode::Unresolved => "无法解析当前 Claude profile 或认证状态".to_string(),
    }
}

fn codex_note(
    summary: &CodexRuntimeSummary,
    used_profile_fallback: bool,
    profile_has_auth_token: bool,
) -> String {
    if profile_has_auth_token {
        return "当前 profile 使用第三方 API key".to_string();
    }

    if used_profile_fallback {
        return "Profile 来自 CCR 当前配置；runtime/auth 已生效".to_string();
    }

    match summary.mode {
        CodexRuntimeMode::ProfileOnly => "当前由 profile/provider key 控制".to_string(),
        CodexRuntimeMode::ProfileWithAuth => "Profile 控制路由，Auth 控制身份".to_string(),
        CodexRuntimeMode::ProfilePendingAuth => {
            "Profile 需要 OpenAI Auth，但当前未检测到有效凭据".to_string()
        }
        CodexRuntimeMode::RuntimeOnly => {
            "当前仅检测到 runtime/auth 生效，未绑定 profile".to_string()
        }
        CodexRuntimeMode::Unresolved => "无法解析当前 Codex profile 或认证状态".to_string(),
    }
}

fn provider_suffix(source: &str) -> Option<&str> {
    source
        .strip_prefix("provider:")
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn profile_has_auth_token(profile: Option<&ProfileConfig>) -> bool {
    profile
        .and_then(|profile| profile.auth_token.as_ref())
        .map(|token| token.expose().trim())
        .is_some_and(|value| !value.is_empty())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::models::{
        AuthIntent, AuthState, ClaudeLoginState, ClaudeProfileAuthMode, ClaudeRuntimeMode,
        ClaudeRuntimeSummary, CodexProfileAuthMode, CodexRuntimeMode, CodexRuntimeSummary,
        LoginState, OpenAiAuthMethod,
    };
    use chrono::Utc;
    use serde_json::json;

    fn codex_auth_state(intent: AuthIntent, status: AuthStateStatus) -> AuthState {
        AuthState {
            intent,
            store: crate::models::CredentialStoreKind::File,
            status,
            reason: "test".to_string(),
        }
    }

    fn codex_summary(
        mode: CodexRuntimeMode,
        auth_mode: Option<CodexProfileAuthMode>,
        login_state: LoginState,
        auth_state: AuthState,
    ) -> CodexRuntimeSummary {
        CodexRuntimeSummary {
            mode,
            current_profile_name: Some("codex-work".to_string()),
            current_profile_provider: Some("9m8m".to_string()),
            current_profile_auth_mode: auth_mode,
            current_profile_auth_source: Some("openai_api_key".to_string()),
            current_auth_name: Some("work".to_string()),
            login_state,
            auth_state,
        }
    }

    fn claude_summary(
        mode: ClaudeRuntimeMode,
        auth_mode: Option<ClaudeProfileAuthMode>,
        login_state: ClaudeLoginState,
    ) -> ClaudeRuntimeSummary {
        ClaudeRuntimeSummary {
            mode,
            current_profile_name: Some("claude-work".to_string()),
            current_profile_provider: Some("anthropic".to_string()),
            current_profile_auth_mode: auth_mode,
            current_profile_auth_source: Some("provider:anthropic".to_string()),
            current_login_name: Some("official-work".to_string()),
            official_login_state: login_state.clone(),
            current_auth_name: Some("official-work".to_string()),
            login_state,
            auth_diagnosis: Default::default(),
        }
    }

    #[test]
    fn codex_chatgpt_login_renders_official_auth() {
        let summary = codex_summary(
            CodexRuntimeMode::ProfileWithAuth,
            Some(CodexProfileAuthMode::OpenAiChatgpt),
            LoginState::LoggedInSaved("work".to_string()),
            codex_auth_state(
                AuthIntent::OpenAiAuth {
                    method: OpenAiAuthMethod::Chatgpt,
                },
                AuthStateStatus::Valid,
            ),
        );

        let (label, kind) = codex_auth_display(&summary, None);

        assert_eq!(label, "官方 Auth: work");
        assert_eq!(kind, StatusAuthKind::OfficialAuth);
    }

    #[test]
    fn codex_api_key_renders_third_party_api() {
        let summary = codex_summary(
            CodexRuntimeMode::ProfileWithAuth,
            Some(CodexProfileAuthMode::OpenAiApiKey),
            LoginState::ApiKeyActive,
            codex_auth_state(
                AuthIntent::OpenAiAuth {
                    method: OpenAiAuthMethod::Api,
                },
                AuthStateStatus::Valid,
            ),
        );

        let (label, kind) = codex_auth_display(&summary, None);

        assert_eq!(label, "第三方 API: 9m8m");
        assert_eq!(kind, StatusAuthKind::ThirdPartyApi);
    }

    #[test]
    fn codex_provider_key_renders_provider_key() {
        let mut summary = codex_summary(
            CodexRuntimeMode::ProfileOnly,
            Some(CodexProfileAuthMode::ProviderEnvKey),
            LoginState::ProviderKeyActive {
                env_key: "MISTRAL_API_KEY".to_string(),
            },
            codex_auth_state(
                AuthIntent::ProviderEnvKey {
                    env_key: "MISTRAL_API_KEY".to_string(),
                },
                AuthStateStatus::Valid,
            ),
        );
        summary.current_profile_auth_source = Some("provider:MISTRAL_API_KEY".to_string());

        let (label, kind) = codex_auth_display(&summary, None);

        assert_eq!(label, "Provider Key: MISTRAL_API_KEY");
        assert_eq!(kind, StatusAuthKind::ProviderKey);
    }

    #[test]
    fn codex_missing_chatgpt_auth_renders_login_hint() {
        let summary = codex_summary(
            CodexRuntimeMode::ProfilePendingAuth,
            Some(CodexProfileAuthMode::OpenAiChatgpt),
            LoginState::NotLoggedIn,
            codex_auth_state(
                AuthIntent::OpenAiAuth {
                    method: OpenAiAuthMethod::Chatgpt,
                },
                AuthStateStatus::Missing,
            ),
        );

        let (label, kind) = codex_auth_display(&summary, None);

        assert_eq!(label, "官方 Auth: 需要 codex login");
        assert_eq!(kind, StatusAuthKind::Missing);
    }

    #[test]
    fn claude_subscription_login_renders_official_auth() {
        let summary = claude_summary(
            ClaudeRuntimeMode::ProfileWithAuth,
            Some(ClaudeProfileAuthMode::Subscription),
            ClaudeLoginState::LoggedInSaved {
                account_name: "official-work".to_string(),
            },
        );

        let (label, kind) = claude_auth_display(&summary);

        assert_eq!(label, "官方 Auth: official-work");
        assert_eq!(kind, StatusAuthKind::OfficialAuth);
    }

    #[test]
    fn claude_api_key_profile_renders_third_party_api() {
        let summary = claude_summary(
            ClaudeRuntimeMode::ProfileOnly,
            Some(ClaudeProfileAuthMode::ApiKey),
            ClaudeLoginState::ApiKeyActive,
        );

        let (label, kind) = claude_auth_display(&summary);

        assert_eq!(label, "第三方 API: anthropic");
        assert_eq!(kind, StatusAuthKind::ThirdPartyApi);
    }

    #[test]
    fn json_overview_contains_schema_and_both_platforms_without_legacy_key() {
        let overview = RuntimeOverview {
            schema_version: RUNTIME_OVERVIEW_SCHEMA_VERSION,
            generated_at: Utc::now(),
            claude: PlatformStatusCard {
                platform: "claude".to_string(),
                display_name: "Claude Code".to_string(),
                profile: "claude-work".to_string(),
                provider: Some("anthropic".to_string()),
                base_url: None,
                model: Some("claude-sonnet".to_string()),
                small_fast_model: None,
                auth: "官方 Auth: work".to_string(),
                auth_kind: StatusAuthKind::OfficialAuth,
                health: StatusHealth::Ready,
                note: "ok".to_string(),
            },
            codex: PlatformStatusCard {
                platform: "codex".to_string(),
                display_name: "Codex".to_string(),
                profile: "codex-work".to_string(),
                provider: Some("9m8m".to_string()),
                base_url: Some("https://9m8m.com".to_string()),
                model: Some("gpt-5.4".to_string()),
                small_fast_model: None,
                auth: "第三方 API: OPENAI_API_KEY".to_string(),
                auth_kind: StatusAuthKind::ThirdPartyApi,
                health: StatusHealth::Ready,
                note: "ok".to_string(),
            },
        };

        let value = serde_json::to_value(&overview).unwrap();

        assert_eq!(
            value["schema_version"],
            json!(RUNTIME_OVERVIEW_SCHEMA_VERSION)
        );
        assert!(value["generated_at"].is_string());
        assert_eq!(value["claude"]["platform"], json!("claude"));
        assert_eq!(value["codex"]["platform"], json!("codex"));
        assert!(value.get("current_platform").is_none());
        let serialized = serde_json::to_string(&value).unwrap();
        assert!(!serialized.contains("sk-"));
        assert!(!serialized.contains("auth_token"));
    }
}
