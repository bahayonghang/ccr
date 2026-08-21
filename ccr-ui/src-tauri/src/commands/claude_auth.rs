use super::*;
use ccr_cli::models::{
    ClaudeAuthConfidence as ServiceAuthConfidence, ClaudeAuthDiagnosis as ServiceAuthDiagnosis,
    ClaudeAuthEvidence as ServiceAuthEvidence, ClaudeAuthOwnership as ServiceAuthOwnership,
    ClaudeAuthSourceKind as ServiceAuthSourceKind,
    ClaudeAuthSourceLocation as ServiceAuthSourceLocation,
    ClaudeAuthSourceObservation as ServiceAuthSourceObservation,
    ClaudeCurrentAuthInfo as ServiceCurrentAuthInfo, ClaudeLoginState as ServiceLoginState,
    ClaudeProfileAuthMode as ServiceProfileAuthMode, ClaudeRuntimeMode as ServiceRuntimeMode,
    ClaudeRuntimeSummary as ServiceRuntimeSummary,
};
use ccr_cli::application::{auth_off_for_platform, needs_auth_off};
use ccr_cli::models::Platform;
use ccr_cli::services::ClaudeAuthItem as ServiceAuthItem;
use serde_json::Value;
use ts_rs::TS;

use crate::commands::settings_raw::ensure_local_env;
use crate::state::AppState;
use tauri::State;

#[derive(Debug, Clone, Copy, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "../../src/types/generated/claude_auth/")]
pub enum ClaudeProfileAuthMode {
    Subscription,
    ApiKey,
}

impl From<ServiceProfileAuthMode> for ClaudeProfileAuthMode {
    fn from(value: ServiceProfileAuthMode) -> Self {
        match value {
            ServiceProfileAuthMode::Subscription => Self::Subscription,
            ServiceProfileAuthMode::ApiKey => Self::ApiKey,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "../../src/types/generated/claude_auth/")]
pub enum ClaudeRuntimeMode {
    ProfileOnly,
    ProfileWithAuth,
    ProfilePendingAuth,
    RuntimeOnly,
    Unresolved,
}

impl From<ServiceRuntimeMode> for ClaudeRuntimeMode {
    fn from(value: ServiceRuntimeMode) -> Self {
        match value {
            ServiceRuntimeMode::ProfileOnly => Self::ProfileOnly,
            ServiceRuntimeMode::ProfileWithAuth => Self::ProfileWithAuth,
            ServiceRuntimeMode::ProfilePendingAuth => Self::ProfilePendingAuth,
            ServiceRuntimeMode::RuntimeOnly => Self::RuntimeOnly,
            ServiceRuntimeMode::Unresolved => Self::Unresolved,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "../../src/types/generated/claude_auth/")]
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

impl From<ServiceAuthSourceKind> for ClaudeAuthSourceKind {
    fn from(value: ServiceAuthSourceKind) -> Self {
        match value {
            ServiceAuthSourceKind::Bedrock => Self::Bedrock,
            ServiceAuthSourceKind::Vertex => Self::Vertex,
            ServiceAuthSourceKind::Foundry => Self::Foundry,
            ServiceAuthSourceKind::AnthropicAuthToken => Self::AnthropicAuthToken,
            ServiceAuthSourceKind::AnthropicApiKey => Self::AnthropicApiKey,
            ServiceAuthSourceKind::ApiKeyHelper => Self::ApiKeyHelper,
            ServiceAuthSourceKind::ClaudeCodeOauthToken => Self::ClaudeCodeOauthToken,
            ServiceAuthSourceKind::SubscriptionOauth => Self::SubscriptionOauth,
            ServiceAuthSourceKind::PrimaryApiKey => Self::PrimaryApiKey,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "../../src/types/generated/claude_auth/")]
pub enum ClaudeAuthSourceLocation {
    ProcessEnv,
    SettingsEnv,
    SettingsRoot,
    StateFile,
    CredentialsFile,
}

impl From<ServiceAuthSourceLocation> for ClaudeAuthSourceLocation {
    fn from(value: ServiceAuthSourceLocation) -> Self {
        match value {
            ServiceAuthSourceLocation::ProcessEnv => Self::ProcessEnv,
            ServiceAuthSourceLocation::SettingsEnv => Self::SettingsEnv,
            ServiceAuthSourceLocation::SettingsRoot => Self::SettingsRoot,
            ServiceAuthSourceLocation::StateFile => Self::StateFile,
            ServiceAuthSourceLocation::CredentialsFile => Self::CredentialsFile,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "../../src/types/generated/claude_auth/")]
pub enum ClaudeAuthConfidence {
    Confirmed,
    Potential,
    Unobservable,
}

impl From<ServiceAuthConfidence> for ClaudeAuthConfidence {
    fn from(value: ServiceAuthConfidence) -> Self {
        match value {
            ServiceAuthConfidence::Confirmed => Self::Confirmed,
            ServiceAuthConfidence::Potential => Self::Potential,
            ServiceAuthConfidence::Unobservable => Self::Unobservable,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "../../src/types/generated/claude_auth/")]
pub enum ClaudeAuthEvidence {
    OfficialContract,
    IssueReport,
}

impl From<ServiceAuthEvidence> for ClaudeAuthEvidence {
    fn from(value: ServiceAuthEvidence) -> Self {
        match value {
            ServiceAuthEvidence::OfficialContract => Self::OfficialContract,
            ServiceAuthEvidence::IssueReport => Self::IssueReport,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "../../src/types/generated/claude_auth/")]
pub enum ClaudeAuthOwnership {
    CcrManaged,
    UserOwned,
    ExternalRuntime,
}

impl From<ServiceAuthOwnership> for ClaudeAuthOwnership {
    fn from(value: ServiceAuthOwnership) -> Self {
        match value {
            ServiceAuthOwnership::CcrManaged => Self::CcrManaged,
            ServiceAuthOwnership::UserOwned => Self::UserOwned,
            ServiceAuthOwnership::ExternalRuntime => Self::ExternalRuntime,
        }
    }
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/claude_auth/")]
pub struct ClaudeAuthSourceObservation {
    pub kind: ClaudeAuthSourceKind,
    pub location: ClaudeAuthSourceLocation,
    pub confidence: ClaudeAuthConfidence,
    pub evidence: ClaudeAuthEvidence,
    pub ownership: ClaudeAuthOwnership,
    pub suppresses_subscription: bool,
}

impl From<ServiceAuthSourceObservation> for ClaudeAuthSourceObservation {
    fn from(value: ServiceAuthSourceObservation) -> Self {
        Self {
            kind: value.kind.into(),
            location: value.location.into(),
            confidence: value.confidence.into(),
            evidence: value.evidence.into(),
            ownership: value.ownership.into(),
            suppresses_subscription: value.suppresses_subscription,
        }
    }
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/claude_auth/")]
pub struct ClaudeAuthDiagnosis {
    pub observations: Vec<ClaudeAuthSourceObservation>,
    pub presumed_effective_source: Option<ClaudeAuthSourceObservation>,
    pub custom_api_key_responses_present: bool,
    pub unobservable: Vec<String>,
}

impl From<ServiceAuthDiagnosis> for ClaudeAuthDiagnosis {
    fn from(value: ServiceAuthDiagnosis) -> Self {
        Self {
            observations: value.observations.into_iter().map(Into::into).collect(),
            presumed_effective_source: value.presumed_effective_source.map(Into::into),
            custom_api_key_responses_present: value.custom_api_key_responses_present,
            unobservable: value.unobservable,
        }
    }
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(tag = "type")]
#[ts(export, export_to = "../../src/types/generated/claude_auth/")]
pub enum ClaudeLoginState {
    NotLoggedIn,
    LoggedInUnsaved,
    LoggedInSaved { account_name: String },
    ApiKeyActive,
}

impl From<ServiceLoginState> for ClaudeLoginState {
    fn from(value: ServiceLoginState) -> Self {
        match value {
            ServiceLoginState::NotLoggedIn => Self::NotLoggedIn,
            ServiceLoginState::LoggedInUnsaved => Self::LoggedInUnsaved,
            ServiceLoginState::LoggedInSaved { account_name } => {
                Self::LoggedInSaved { account_name }
            }
            ServiceLoginState::ApiKeyActive => Self::ApiKeyActive,
        }
    }
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/claude_auth/")]
pub struct ClaudeRuntimeSummary {
    pub mode: ClaudeRuntimeMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub current_profile_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub current_profile_provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub current_profile_auth_mode: Option<ClaudeProfileAuthMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub current_profile_auth_source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub current_login_name: Option<String>,
    pub official_login_state: ClaudeLoginState,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub current_auth_name: Option<String>,
    pub login_state: ClaudeLoginState,
    pub auth_diagnosis: ClaudeAuthDiagnosis,
}

impl From<ServiceRuntimeSummary> for ClaudeRuntimeSummary {
    fn from(value: ServiceRuntimeSummary) -> Self {
        Self {
            mode: value.mode.into(),
            current_profile_name: value.current_profile_name,
            current_profile_provider: value.current_profile_provider,
            current_profile_auth_mode: value.current_profile_auth_mode.map(Into::into),
            current_profile_auth_source: value.current_profile_auth_source,
            current_login_name: value.current_login_name,
            official_login_state: value.official_login_state.into(),
            current_auth_name: value.current_auth_name,
            login_state: value.login_state.into(),
            auth_diagnosis: value.auth_diagnosis.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/claude_auth/")]
pub struct ClaudeAuthAccountItem {
    pub name: String,
    pub description: Option<String>,
    pub email: Option<String>,
    pub billing_type: Option<String>,
    pub subscription_type: Option<String>,
    pub rate_limit_tier: Option<String>,
    pub is_current: bool,
    pub is_logged_in: bool,
    pub saved_at: String,
    pub last_used: Option<String>,
    pub expires_at: Option<String>,
}

impl From<ServiceAuthItem> for ClaudeAuthAccountItem {
    fn from(value: ServiceAuthItem) -> Self {
        Self {
            name: value.name,
            description: value.description,
            email: value.email,
            billing_type: value.billing_type,
            subscription_type: value.subscription_type,
            rate_limit_tier: value.rate_limit_tier,
            is_current: value.is_current,
            is_logged_in: value.is_logged_in,
            saved_at: value.saved_at.to_rfc3339(),
            last_used: value.last_used.map(|date| date.to_rfc3339()),
            expires_at: value.expires_at.map(|date| date.to_rfc3339()),
        }
    }
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/claude_auth/")]
pub struct ClaudeAuthCurrentInfo {
    pub account_uuid: Option<String>,
    pub email: Option<String>,
    pub billing_type: Option<String>,
    pub subscription_type: Option<String>,
    pub rate_limit_tier: Option<String>,
    pub expires_at: Option<String>,
}

impl From<ServiceCurrentAuthInfo> for ClaudeAuthCurrentInfo {
    fn from(value: ServiceCurrentAuthInfo) -> Self {
        Self {
            account_uuid: value.account_uuid,
            email: value.email,
            billing_type: value.billing_type,
            subscription_type: value.subscription_type,
            rate_limit_tier: value.rate_limit_tier,
            expires_at: value.expires_at.map(|date| date.to_rfc3339()),
        }
    }
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/claude_auth/")]
pub struct ClaudeAuthListResponse {
    pub accounts: Vec<ClaudeAuthAccountItem>,
    pub login_state: ClaudeLoginState,
    pub runtime_summary: ClaudeRuntimeSummary,
    pub current_profile_auth_mode: Option<ClaudeProfileAuthMode>,
    pub can_auth_off: bool,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/claude_auth/")]
pub struct ClaudeAuthCurrentResponse {
    pub logged_in: bool,
    pub info: Option<ClaudeAuthCurrentInfo>,
    pub runtime_summary: ClaudeRuntimeSummary,
    pub login_state: ClaudeLoginState,
    pub can_auth_off: bool,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/claude_auth/")]
pub struct ClaudeAuthActionResponse {
    pub success: bool,
    pub message: String,
    pub cleared_managed_sources: Vec<String>,
    pub remaining_suppressors: Vec<ClaudeAuthSourceObservation>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/claude_auth/")]
pub struct ClaudeAuthOffResponse {
    pub ok: bool,
    pub changed: bool,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub profile_pointer: Option<String>,
    pub warnings: Vec<String>,
}

#[ccr_tauri_command_macros::command]
pub async fn claude_list_auth_accounts() -> Result<ClaudeAuthListResponse, String> {
    tokio::task::spawn_blocking(|| {
        let service =
            ClaudeAuthService::new().map_err(|e| format!("初始化 Claude Auth 服务失败: {e}"))?;
        let snapshot = service
            .read_auth_snapshot()
            .map_err(|e| format!("读取认证快照失败: {e}"))?;
        let runtime_summary = service
            .get_runtime_summary()
            .map_err(|e| format!("读取运行时摘要失败: {e}"))?;
        let accounts = service
            .build_account_items(&snapshot, Some(&runtime_summary))
            .map_err(|e| format!("列出账号失败: {e}"))?;
        let current_profile_auth_mode = runtime_summary.current_profile_auth_mode.map(Into::into);

        Ok(ClaudeAuthListResponse {
            accounts: accounts.into_iter().map(Into::into).collect(),
            login_state: snapshot.login_state.into(),
            runtime_summary: runtime_summary.into(),
            current_profile_auth_mode,
            can_auth_off: needs_auth_off(Platform::Claude).unwrap_or(false),
        })
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

#[ccr_tauri_command_macros::command]
pub async fn claude_get_auth_current() -> Result<ClaudeAuthCurrentResponse, String> {
    tokio::task::spawn_blocking(|| {
        let service =
            ClaudeAuthService::new().map_err(|e| format!("初始化 Claude Auth 服务失败: {e}"))?;
        let runtime_summary = service
            .get_runtime_summary()
            .map_err(|e| format!("读取运行时摘要失败: {e}"))?;
        let current_info = service.get_current_auth_info().ok();
        let logged_in = current_info.is_some();
        let login_state = runtime_summary.login_state.clone().into();

        Ok(ClaudeAuthCurrentResponse {
            logged_in,
            info: current_info.map(Into::into),
            runtime_summary: runtime_summary.into(),
            login_state,
            can_auth_off: needs_auth_off(Platform::Claude).unwrap_or(false),
        })
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

#[ccr_tauri_command_macros::command]
pub async fn claude_auth_off(state: State<'_, AppState>) -> Result<ClaudeAuthOffResponse, String> {
    if let Some(response) = ensure_local_env(state.inner()).await {
        return Err(unsupported_env_error(&response));
    }
    tokio::task::spawn_blocking(|| {
        let result = auth_off_for_platform(Platform::Claude)
            .map_err(|error| format!("登出 Claude 官方会话失败: {error}"))?;
        Ok(ClaudeAuthOffResponse {
            ok: true,
            changed: result.changed,
            path: result.path.as_str().to_string(),
            profile_pointer: result.profile_pointer,
            warnings: result.warnings,
        })
    })
    .await
    .map_err(|error| format!("登出 Claude 官方会话后台任务失败: {error}"))?
}

fn unsupported_env_error(response: &Value) -> String {
    let env_type = response
        .get("envType")
        .or_else(|| response.get("env_type"))
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    format!("unsupported_environment:{env_type}")
}

#[ccr_tauri_command_macros::command]
pub async fn claude_save_auth(
    name: String,
    description: Option<String>,
    force: Option<bool>,
) -> Result<ClaudeAuthActionResponse, String> {
    tokio::task::spawn_blocking(move || {
        let service =
            ClaudeAuthService::new().map_err(|e| format!("初始化 Claude Auth 服务失败: {e}"))?;
        service
            .save_current(&name, description, force.unwrap_or(false))
            .map_err(|e| format!("{e}"))?;
        Ok(ClaudeAuthActionResponse {
            success: true,
            message: format!("Claude 官方账号 '{name}' 已成功保存"),
            cleared_managed_sources: Vec::new(),
            remaining_suppressors: Vec::new(),
            warnings: Vec::new(),
        })
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

#[ccr_tauri_command_macros::command]
pub async fn claude_switch_auth(name: String) -> Result<ClaudeAuthActionResponse, String> {
    let name_resp = name.clone();
    let outcome = tokio::task::spawn_blocking(move || {
        let service =
            ClaudeAuthService::new().map_err(|e| format!("初始化 Claude Auth 服务失败: {e}"))?;
        service.switch_account(&name).map_err(|e| format!("{e}"))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??;

    Ok(ClaudeAuthActionResponse {
        success: true,
        message: format!("已切换到 Claude 官方账号 '{name_resp}'"),
        cleared_managed_sources: outcome.cleared_managed_sources,
        remaining_suppressors: outcome
            .remaining_suppressors
            .into_iter()
            .map(Into::into)
            .collect(),
        warnings: outcome.warnings,
    })
}

#[ccr_tauri_command_macros::command]
pub async fn claude_delete_auth(name: String) -> Result<ClaudeAuthActionResponse, String> {
    let name_resp = name.clone();
    tokio::task::spawn_blocking(move || {
        let service =
            ClaudeAuthService::new().map_err(|e| format!("初始化 Claude Auth 服务失败: {e}"))?;
        service.delete_account(&name).map_err(|e| format!("{e}"))?;
        Ok::<_, String>(())
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??;

    Ok(ClaudeAuthActionResponse {
        success: true,
        message: format!("Claude 官方账号 '{name_resp}' 已成功删除"),
        cleared_managed_sources: Vec::new(),
        remaining_suppressors: Vec::new(),
        warnings: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    #[test]
    fn auth_account_item_preserves_rfc3339_wire_dates() {
        let saved_at = Utc.with_ymd_and_hms(2026, 7, 26, 12, 34, 56).unwrap();
        let item = ClaudeAuthAccountItem::from(ServiceAuthItem {
            name: "work".to_string(),
            description: Some("primary".to_string()),
            email: Some("masked@example.com".to_string()),
            billing_type: None,
            subscription_type: Some("pro".to_string()),
            rate_limit_tier: None,
            is_current: true,
            is_logged_in: true,
            saved_at,
            last_used: Some(saved_at),
            expires_at: None,
        });

        let value = serde_json::to_value(item).unwrap();
        assert_eq!(value["saved_at"], "2026-07-26T12:34:56+00:00");
        assert_eq!(value["last_used"], "2026-07-26T12:34:56+00:00");
        assert!(value["expires_at"].is_null());
    }

    #[test]
    fn runtime_summary_omits_absent_optional_fields() {
        let summary = ClaudeRuntimeSummary::from(ServiceRuntimeSummary {
            mode: ServiceRuntimeMode::Unresolved,
            current_profile_name: None,
            current_profile_provider: None,
            current_profile_auth_mode: None,
            current_profile_auth_source: None,
            current_login_name: None,
            official_login_state: ServiceLoginState::NotLoggedIn,
            current_auth_name: None,
            login_state: ServiceLoginState::NotLoggedIn,
            auth_diagnosis: ServiceAuthDiagnosis::default(),
        });

        let value = serde_json::to_value(summary).unwrap();
        assert_eq!(value["mode"], "unresolved");
        assert_eq!(value["login_state"]["type"], "NotLoggedIn");
        assert!(
            !value
                .as_object()
                .unwrap()
                .contains_key("current_profile_name")
        );
    }

    #[test]
    fn runtime_summary_preserves_secret_free_auth_diagnosis_metadata() {
        let source = ServiceAuthSourceObservation {
            kind: ServiceAuthSourceKind::PrimaryApiKey,
            location: ServiceAuthSourceLocation::StateFile,
            confidence: ServiceAuthConfidence::Potential,
            evidence: ServiceAuthEvidence::IssueReport,
            ownership: ServiceAuthOwnership::UserOwned,
            suppresses_subscription: true,
        };
        let summary = ClaudeRuntimeSummary::from(ServiceRuntimeSummary {
            mode: ServiceRuntimeMode::RuntimeOnly,
            current_profile_name: None,
            current_profile_provider: None,
            current_profile_auth_mode: None,
            current_profile_auth_source: None,
            current_login_name: None,
            official_login_state: ServiceLoginState::LoggedInUnsaved,
            current_auth_name: None,
            login_state: ServiceLoginState::LoggedInUnsaved,
            auth_diagnosis: ServiceAuthDiagnosis {
                observations: vec![source.clone()],
                presumed_effective_source: Some(source),
                custom_api_key_responses_present: true,
                unobservable: vec!["other_shell_environment".to_string()],
            },
        });

        let value = serde_json::to_value(summary).unwrap();
        assert_eq!(
            value["auth_diagnosis"]["observations"][0]["kind"],
            "primary_api_key"
        );
        assert_eq!(
            value["auth_diagnosis"]["observations"][0]["confidence"],
            "potential"
        );
        assert_eq!(
            value["auth_diagnosis"]["observations"][0]["evidence"],
            "issue_report"
        );
        assert!(!value.to_string().contains("diagnostic-secret"));
    }
}
