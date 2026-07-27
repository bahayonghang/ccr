use super::*;

use ccr_codex::{CodexAccountQuota, CodexAuthItem, CodexQuota, CodexRuntimeMode, LoginState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexTrayAccountRow {
    pub name: String,
    pub description: Option<String>,
    pub email: Option<String>,
    pub is_current: bool,
    pub is_virtual: bool,
    pub saved_at: Option<String>,
    pub last_used: Option<String>,
    pub last_refresh: Option<String>,
    pub plan_type: Option<String>,
    pub can_switch: bool,
    pub quota: Option<CodexQuota>,
    pub quota_error: Option<String>,
    pub quota_fetched_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexTraySnapshot {
    pub fetched_at: String,
    pub runtime_mode: String,
    pub runtime_description: String,
    pub profile_label: String,
    pub auth_label: String,
    pub current_profile_name: Option<String>,
    pub current_profile_provider: Option<String>,
    pub current_profile_auth_mode: Option<String>,
    pub current_auth_name: Option<String>,
    pub login_state: LoginState,
    pub can_manage_accounts: bool,
    pub current_account: Option<CodexTrayAccountRow>,
    pub accounts: Vec<CodexTrayAccountRow>,
}

pub(crate) async fn compute_codex_tray_snapshot(force: bool) -> Result<CodexTraySnapshot, String> {
    let (runtime_summary, account_items) = tokio::task::spawn_blocking(|| -> Result<_, String> {
        let service =
            CodexAuthService::new().map_err(|e| format!("初始化 Codex Auth 服务失败: {e}"))?;
        let snapshot = service
            .read_auth_snapshot()
            .map_err(|e| format!("读取 Codex Auth 快照失败: {e}"))?;
        let accounts = service
            .build_account_items(&snapshot)
            .map_err(|e| format!("构建 Codex Auth 列表失败: {e}"))?;
        let runtime_summary = service
            .get_runtime_summary()
            .map_err(|e| format!("读取 Codex Runtime 摘要失败: {e}"))?;

        Ok((runtime_summary, accounts))
    })
    .await
    .map_err(|e| format!("读取 Codex Auth 信息任务失败: {e}"))??;

    let current_quota = if should_query_current_quota(runtime_summary.mode) {
        let quota_service = ccr_codex::CodexQuotaService::new()
            .map_err(|e| format!("初始化 Codex 配额服务失败: {e}"))?;
        Some(if force {
            quota_service.fetch_current_quota_force_refresh().await
        } else {
            quota_service.fetch_current_quota().await
        })
    } else {
        None
    };

    let can_manage_accounts = runtime_summary
        .current_profile_auth_mode
        .is_some_and(|mode| mode.uses_openai_auth());

    let (accounts, current_account) =
        build_tray_account_rows(account_items, current_quota.as_ref(), can_manage_accounts);

    Ok(CodexTraySnapshot {
        fetched_at: Utc::now().to_rfc3339(),
        runtime_mode: runtime_mode_to_string(runtime_summary.mode),
        runtime_description: runtime_summary.mode.label().to_string(),
        profile_label: runtime_summary.profile_label(),
        auth_label: runtime_summary.auth_label(),
        current_profile_name: runtime_summary.current_profile_name,
        current_profile_provider: runtime_summary.current_profile_provider,
        current_profile_auth_mode: runtime_summary
            .current_profile_auth_mode
            .map(|mode| mode.as_str().to_string()),
        current_auth_name: runtime_summary.current_auth_name,
        login_state: runtime_summary.login_state,
        can_manage_accounts,
        current_account,
        accounts,
    })
}

#[tauri::command]
pub async fn codex_get_tray_snapshot(force: Option<bool>) -> Result<OpenJsonValueDto, String> {
    let snapshot = compute_codex_tray_snapshot(force.unwrap_or(false)).await?;
    serde_json::to_value(snapshot)
        .map_err(|error| format!("序列化 Codex tray snapshot 失败: {error}"))?
        .try_into()
}

fn runtime_mode_to_string(mode: CodexRuntimeMode) -> String {
    match mode {
        CodexRuntimeMode::ProfileOnly => "profile_only".to_string(),
        CodexRuntimeMode::ProfileWithAuth => "profile_with_auth".to_string(),
        CodexRuntimeMode::ProfilePendingAuth => "profile_pending_auth".to_string(),
        CodexRuntimeMode::RuntimeOnly => "runtime_only".to_string(),
        CodexRuntimeMode::Unresolved => "unresolved".to_string(),
    }
}

fn should_query_current_quota(mode: CodexRuntimeMode) -> bool {
    !matches!(
        mode,
        CodexRuntimeMode::ProfileOnly | CodexRuntimeMode::Unresolved
    )
}

fn build_tray_account_rows(
    account_items: Vec<CodexAuthItem>,
    current_quota: Option<&CodexAccountQuota>,
    can_manage_accounts: bool,
) -> (Vec<CodexTrayAccountRow>, Option<CodexTrayAccountRow>) {
    let mut current_account = None;

    let accounts = account_items
        .into_iter()
        .map(|item| {
            let quota = item.is_current.then_some(current_quota).flatten();
            let row = CodexTrayAccountRow {
                name: item.name,
                description: item.description,
                email: item.email,
                is_current: item.is_current,
                is_virtual: item.is_virtual,
                saved_at: item.saved_at.map(|dt| dt.to_rfc3339()),
                last_used: item.last_used.map(|dt| dt.to_rfc3339()),
                last_refresh: item.last_refresh.map(|dt| dt.to_rfc3339()),
                plan_type: item.plan_type,
                can_switch: can_manage_accounts && !item.is_current,
                quota: quota.and_then(|entry| entry.quota.clone()),
                quota_error: quota.and_then(|entry| entry.error.clone()),
                quota_fetched_at: quota.map(|entry| entry.fetched_at.to_rfc3339()),
            };

            if row.is_current {
                current_account = Some(row.clone());
            }

            row
        })
        .collect();

    (accounts, current_account)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn sample_auth_item(name: &str, is_current: bool) -> CodexAuthItem {
        CodexAuthItem {
            name: name.to_string(),
            description: None,
            email: Some(format!("{name}@example.com")),
            is_current,
            is_virtual: false,
            plan_type: None,
            saved_at: Some(Utc::now()),
            last_used: None,
            last_refresh: None,
        }
    }

    #[test]
    fn tray_snapshot_only_attaches_quota_to_current_account() {
        let items = vec![
            sample_auth_item("current", true),
            sample_auth_item("other", false),
        ];
        let quota = CodexAccountQuota {
            account_name: "default".to_string(),
            email: Some("current@example.com".to_string()),
            quota: Some(CodexQuota {
                hourly_percentage: 87,
                hourly_reset_time: None,
                hourly_window_minutes: None,
                hourly_window_present: None,
                weekly_percentage: 46,
                weekly_reset_time: None,
                weekly_window_minutes: None,
                weekly_window_present: None,
                plan_type: None,
                raw_data: None,
            }),
            error: None,
            fetched_at: Utc::now(),
        };

        let (accounts, current_account) = build_tray_account_rows(items, Some(&quota), true);

        assert_eq!(accounts.len(), 2);
        assert!(accounts[0].quota.is_some());
        assert!(accounts[1].quota.is_none());
        assert_eq!(
            current_account
                .as_ref()
                .and_then(|account| account.quota.as_ref())
                .map(|quota| quota.hourly_percentage),
            Some(87)
        );
    }

    #[test]
    fn tray_snapshot_skips_quota_lookup_for_profile_only_mode() {
        assert!(!should_query_current_quota(CodexRuntimeMode::ProfileOnly));
        assert!(!should_query_current_quota(CodexRuntimeMode::Unresolved));
        assert!(should_query_current_quota(
            CodexRuntimeMode::ProfileWithAuth
        ));
        assert!(should_query_current_quota(CodexRuntimeMode::RuntimeOnly));
    }
}
