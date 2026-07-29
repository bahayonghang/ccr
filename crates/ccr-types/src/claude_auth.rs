//! Claude Auth Types
//!
//! Shared types for Claude subscription and API key management.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Claude profile authentication mode.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ClaudeProfileAuthMode {
    /// Use local Claude subscription credentials from `~/.claude/.credentials.json`.
    Subscription,
    /// Use `ANTHROPIC_*` environment overrides.
    ApiKey,
}

impl ClaudeProfileAuthMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            ClaudeProfileAuthMode::Subscription => "subscription",
            ClaudeProfileAuthMode::ApiKey => "api_key",
        }
    }

    pub fn uses_subscription(&self) -> bool {
        matches!(self, ClaudeProfileAuthMode::Subscription)
    }
}

/// Claude login state for official subscription credentials.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClaudeLoginState {
    NotLoggedIn,
    LoggedInUnsaved,
    LoggedInSaved { account_name: String },
    ApiKeyActive,
}

/// Saved Claude auth account metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClaudeAuthAccount {
    /// User-provided description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Stable account UUID from `~/.claude.json.oauthAccount.accountUuid`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_uuid: Option<String>,

    /// Masked email address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,

    /// Billing type from `~/.claude.json.oauthAccount.billingType`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub billing_type: Option<String>,

    /// Subscription type from `.credentials.json`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscription_type: Option<String>,

    /// Rate limit tier from `.credentials.json`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limit_tier: Option<String>,

    /// Save time.
    pub saved_at: DateTime<Utc>,

    /// Last switch/use time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_used: Option<DateTime<Utc>>,

    /// Access token expiry time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
}

/// Claude auth registry stored under `~/.ccr/platforms/claude/auth_registry.toml`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClaudeAuthRegistry {
    #[serde(default = "default_registry_version")]
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_auth: Option<String>,
    #[serde(default)]
    pub accounts: BTreeMap<String, ClaudeAuthAccount>,
}

fn default_registry_version() -> String {
    "1.0".to_string()
}

impl Default for ClaudeAuthRegistry {
    fn default() -> Self {
        Self {
            version: default_registry_version(),
            current_auth: None,
            accounts: BTreeMap::new(),
        }
    }
}

/// Current runtime auth info from local Claude subscription files.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClaudeCurrentAuthInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_uuid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub billing_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscription_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limit_tier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
}

/// Claude runtime control mode.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ClaudeRuntimeMode {
    ProfileOnly,
    ProfileWithAuth,
    ProfilePendingAuth,
    RuntimeOnly,
    Unresolved,
}

/// Credential source observed by CCR without exposing its value.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ClaudeAuthSourceKind {
    Bedrock,
    Vertex,
    Foundry,
    AnthropicAuthToken,
    AnthropicApiKey,
    ApiKeyHelper,
    ClaudeCodeOauthToken,
    SubscriptionOauth,
    PrimaryApiKey,
}

impl ClaudeAuthSourceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Bedrock => "bedrock",
            Self::Vertex => "vertex",
            Self::Foundry => "foundry",
            Self::AnthropicAuthToken => "anthropic_auth_token",
            Self::AnthropicApiKey => "anthropic_api_key",
            Self::ApiKeyHelper => "api_key_helper",
            Self::ClaudeCodeOauthToken => "claude_code_oauth_token",
            Self::SubscriptionOauth => "subscription_oauth",
            Self::PrimaryApiKey => "primary_api_key",
        }
    }
}

/// Location where CCR observed an auth source.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ClaudeAuthSourceLocation {
    ProcessEnv,
    SettingsEnv,
    SettingsRoot,
    StateFile,
    CredentialsFile,
}

impl ClaudeAuthSourceLocation {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ProcessEnv => "process_env",
            Self::SettingsEnv => "settings_env",
            Self::SettingsRoot => "settings_root",
            Self::StateFile => "state_file",
            Self::CredentialsFile => "credentials_file",
        }
    }
}

/// Strength of the conclusion CCR can draw from an observation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ClaudeAuthConfidence {
    Confirmed,
    Potential,
    Unobservable,
}

impl ClaudeAuthConfidence {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Confirmed => "confirmed",
            Self::Potential => "potential",
            Self::Unobservable => "unobservable",
        }
    }
}

/// Authority behind an auth-source rule.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ClaudeAuthEvidence {
    OfficialContract,
    IssueReport,
}

impl ClaudeAuthEvidence {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::OfficialContract => "official_contract",
            Self::IssueReport => "issue_report",
        }
    }
}

/// Ownership boundary used to decide whether CCR may clear a source.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ClaudeAuthOwnership {
    CcrManaged,
    UserOwned,
    ExternalRuntime,
}

impl ClaudeAuthOwnership {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CcrManaged => "ccr_managed",
            Self::UserOwned => "user_owned",
            Self::ExternalRuntime => "external_runtime",
        }
    }
}

/// Secret-free observation of one possible Claude Code credential source.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClaudeAuthSourceObservation {
    pub kind: ClaudeAuthSourceKind,
    pub location: ClaudeAuthSourceLocation,
    pub confidence: ClaudeAuthConfidence,
    pub evidence: ClaudeAuthEvidence,
    pub ownership: ClaudeAuthOwnership,
    pub suppresses_subscription: bool,
}

/// Read-only diagnosis shared by CLI, TUI, doctor, and desktop UI.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ClaudeAuthDiagnosis {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub observations: Vec<ClaudeAuthSourceObservation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presumed_effective_source: Option<ClaudeAuthSourceObservation>,
    #[serde(default)]
    pub custom_api_key_responses_present: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unobservable: Vec<String>,
}

impl ClaudeAuthDiagnosis {
    pub fn suppressors(&self) -> impl Iterator<Item = &ClaudeAuthSourceObservation> {
        self.observations
            .iter()
            .filter(|source| source.suppresses_subscription)
    }
}

/// Post-action diagnosis. Values are source identifiers only, never secrets.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ClaudeAuthActionOutcome {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cleared_managed_sources: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub remaining_suppressors: Vec<ClaudeAuthSourceObservation>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
}

impl ClaudeRuntimeMode {
    pub fn label(&self) -> &'static str {
        match self {
            ClaudeRuntimeMode::ProfileOnly => "Profile 驱动",
            ClaudeRuntimeMode::ProfileWithAuth => "Profile + 官方订阅",
            ClaudeRuntimeMode::ProfilePendingAuth => "Profile 等待官方订阅",
            ClaudeRuntimeMode::RuntimeOnly => "仅官方订阅运行时",
            ClaudeRuntimeMode::Unresolved => "未解析",
        }
    }
}

/// Claude runtime summary used by CLI/TUI/Web to explain current control state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClaudeRuntimeSummary {
    pub mode: ClaudeRuntimeMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_profile_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_profile_provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_profile_auth_mode: Option<ClaudeProfileAuthMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_profile_auth_source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_login_name: Option<String>,
    pub official_login_state: ClaudeLoginState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_auth_name: Option<String>,
    pub login_state: ClaudeLoginState,
    #[serde(default)]
    pub auth_diagnosis: ClaudeAuthDiagnosis,
}

impl ClaudeRuntimeSummary {
    pub fn profile_label(&self) -> String {
        let Some(name) = &self.current_profile_name else {
            return "未绑定".to_string();
        };

        let mut parts = vec![name.clone()];

        if let Some(provider) = self
            .current_profile_provider
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            parts.push(provider.to_string());
        }

        if let Some(auth_mode) = self.current_profile_auth_mode {
            parts.push(auth_mode.as_str().to_string());
        }

        if matches!(
            self.current_profile_auth_mode,
            Some(ClaudeProfileAuthMode::ApiKey)
        ) && let Some(source) = self
            .current_profile_auth_source
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            parts.push(source.to_string());
        }

        parts.join(" · ")
    }

    pub fn official_login_label(&self) -> String {
        match &self.official_login_state {
            ClaudeLoginState::NotLoggedIn => "未登录".to_string(),
            ClaudeLoginState::LoggedInUnsaved => "已登录 (未保存)".to_string(),
            ClaudeLoginState::LoggedInSaved { account_name } => format!("已登录: {account_name}"),
            ClaudeLoginState::ApiKeyActive => "API Key 模式".to_string(),
        }
    }

    pub fn auth_label(&self) -> String {
        match self.mode {
            ClaudeRuntimeMode::ProfileOnly => self
                .current_profile_auth_source
                .as_deref()
                .map(|source| format!("Profile / {source}"))
                .unwrap_or_else(|| "Profile / API Key".to_string()),
            ClaudeRuntimeMode::ProfileWithAuth
            | ClaudeRuntimeMode::ProfilePendingAuth
            | ClaudeRuntimeMode::RuntimeOnly => self
                .current_auth_name
                .clone()
                .map(|name| format!("Official / {name}"))
                .unwrap_or_else(|| "Official / 未就绪".to_string()),
            ClaudeRuntimeMode::Unresolved => self
                .current_auth_name
                .clone()
                .map(|name| format!("Official / {name}"))
                .unwrap_or_else(|| "未解析".to_string()),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_auth_mode_as_str() {
        assert_eq!(ClaudeProfileAuthMode::Subscription.as_str(), "subscription");
        assert_eq!(ClaudeProfileAuthMode::ApiKey.as_str(), "api_key");
    }

    #[test]
    fn test_registry_default() {
        let registry = ClaudeAuthRegistry::default();
        assert_eq!(registry.version, "1.0");
        assert!(registry.current_auth.is_none());
        assert!(registry.accounts.is_empty());
    }

    #[test]
    fn test_login_state_serialization() {
        let state = ClaudeLoginState::LoggedInSaved {
            account_name: "work".to_string(),
        };
        let value = serde_json::to_value(&state).unwrap();
        assert_eq!(value["type"], "LoggedInSaved");
        assert_eq!(value["account_name"], "work");
    }

    #[test]
    fn test_runtime_summary_labels_distinguish_login_from_effective_auth() {
        let summary = ClaudeRuntimeSummary {
            mode: ClaudeRuntimeMode::ProfileOnly,
            current_profile_name: Some("main_pro".to_string()),
            current_profile_provider: Some("bah***@gmail.com".to_string()),
            current_profile_auth_mode: Some(ClaudeProfileAuthMode::ApiKey),
            current_profile_auth_source: Some("provider:pro".to_string()),
            current_login_name: Some("official-main".to_string()),
            official_login_state: ClaudeLoginState::LoggedInSaved {
                account_name: "official-main".to_string(),
            },
            current_auth_name: None,
            login_state: ClaudeLoginState::ApiKeyActive,
            auth_diagnosis: ClaudeAuthDiagnosis::default(),
        };

        assert_eq!(
            summary.profile_label(),
            "main_pro · bah***@gmail.com · api_key · provider:pro"
        );
        assert_eq!(summary.official_login_label(), "已登录: official-main");
        assert_eq!(summary.auth_label(), "Profile / provider:pro");
    }

    #[test]
    fn auth_diagnosis_serialization_contains_no_credential_values() {
        let diagnosis = ClaudeAuthDiagnosis {
            observations: vec![ClaudeAuthSourceObservation {
                kind: ClaudeAuthSourceKind::AnthropicApiKey,
                location: ClaudeAuthSourceLocation::SettingsEnv,
                confidence: ClaudeAuthConfidence::Potential,
                evidence: ClaudeAuthEvidence::OfficialContract,
                ownership: ClaudeAuthOwnership::UserOwned,
                suppresses_subscription: true,
            }],
            presumed_effective_source: None,
            custom_api_key_responses_present: true,
            unobservable: vec!["other_shell_environment".to_string()],
        };

        let json = serde_json::to_string(&diagnosis).unwrap();
        assert!(json.contains("anthropic_api_key"));
        assert!(json.contains("potential"));
        assert!(!json.contains("sk-secret"));
    }

    #[test]
    fn runtime_summary_defaults_missing_diagnosis_for_compatibility() {
        let summary: ClaudeRuntimeSummary = serde_json::from_value(serde_json::json!({
            "mode": "unresolved",
            "official_login_state": { "type": "NotLoggedIn" },
            "login_state": { "type": "NotLoggedIn" }
        }))
        .unwrap();

        assert_eq!(summary.auth_diagnosis, ClaudeAuthDiagnosis::default());
    }
}
