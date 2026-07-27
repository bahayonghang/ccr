use super::*;
use ccr_cli::models::{
    ClaudeCurrentAuthInfo as ServiceCurrentAuthInfo, ClaudeLoginState as ServiceLoginState,
    ClaudeProfileAuthMode as ServiceProfileAuthMode, ClaudeRuntimeMode as ServiceRuntimeMode,
    ClaudeRuntimeSummary as ServiceRuntimeSummary,
};
use ccr_cli::services::ClaudeAuthItem as ServiceAuthItem;
use ts_rs::TS;

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
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/claude_auth/")]
pub struct ClaudeAuthCurrentResponse {
    pub logged_in: bool,
    pub info: Option<ClaudeAuthCurrentInfo>,
    pub runtime_summary: ClaudeRuntimeSummary,
    pub login_state: ClaudeLoginState,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/claude_auth/")]
pub struct ClaudeAuthActionResponse {
    pub success: bool,
    pub message: String,
}

#[tauri::command]
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
        })
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

#[tauri::command]
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
        })
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

#[tauri::command]
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
        })
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

#[tauri::command]
pub async fn claude_switch_auth(name: String) -> Result<ClaudeAuthActionResponse, String> {
    let name_resp = name.clone();
    tokio::task::spawn_blocking(move || {
        let service =
            ClaudeAuthService::new().map_err(|e| format!("初始化 Claude Auth 服务失败: {e}"))?;
        service.switch_account(&name).map_err(|e| format!("{e}"))?;
        Ok::<_, String>(())
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??;

    Ok(ClaudeAuthActionResponse {
        success: true,
        message: format!("已切换到 Claude 官方账号 '{name_resp}'"),
    })
}

#[tauri::command]
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
}
