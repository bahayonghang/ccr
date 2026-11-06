//! Safe display projection of the existing credential-refreshing quota owner.
use super::*;

#[derive(Clone)]
pub struct QuotaRow {
    pub account: String,
    pub hourly: Option<i32>,
    pub weekly: Option<i32>,
    pub hourly_reset: Option<i64>,
    pub weekly_reset: Option<i64>,
    pub fetched_at: String,
    pub failed: bool,
    pub cache_hit: Option<bool>,
    pub history_warning: bool,
}
impl From<ccr_codex::CodexAccountQuota> for QuotaRow {
    fn from(result: ccr_codex::CodexAccountQuota) -> Self {
        let failed = result.error.is_some();
        let hourly = result
            .quota
            .as_ref()
            .filter(|q| q.hourly_window_present == Some(true))
            .map(|q| q.hourly_percentage);
        let weekly = result
            .quota
            .as_ref()
            .filter(|q| q.weekly_window_present == Some(true))
            .map(|q| q.weekly_percentage);
        Self {
            account: safe_name(&result.account_name).unwrap_or_else(|| "[hidden]".into()),
            hourly,
            weekly,
            hourly_reset: result.quota.as_ref().and_then(|q| q.hourly_reset_time),
            weekly_reset: result.quota.as_ref().and_then(|q| q.weekly_reset_time),
            fetched_at: result.fetched_at.to_rfc3339(),
            failed,
            cache_hit: result.observation.as_ref().map(|o| o.cache_hit),
            history_warning: result
                .observation
                .as_ref()
                .is_some_and(|o| o.history_warning.is_some()),
        }
    }
}

pub fn fetch(
    session: &PlatformSession,
    account: Option<&str>,
) -> Result<ResultValue, PlatformError> {
    // Reject synthetic sources before constructing a network or credential owner.
    if session.roots.fixture
        || session.platform != Platform::Codex
        || session.surface != Surface::Auth
    {
        return Err(PlatformError::Unsupported);
    }
    if account.is_some_and(|name| name.contains(['/', '\\']) || safe_name(name).is_none()) {
        return Err(PlatformError::Invalid);
    }
    let handle = tokio::runtime::Handle::try_current().map_err(|_| PlatformError::Storage)?;
    let service = ccr_codex::services::CodexQuotaService::from_dirs(
        session.roots.codex_store.clone(),
        session.roots.codex_runtime.clone(),
    );
    let results = handle.block_on(async {
        match account {
            Some("default") => vec![service.fetch_current_quota_force_refresh().await],
            Some(account) => vec![service.fetch_account_quota_force_refresh(account).await],
            None => service.fetch_all_quotas_force_refresh().await,
        }
    });
    Ok(ResultValue::Quotas(
        results.into_iter().map(QuotaRow::from).collect(),
    ))
}
