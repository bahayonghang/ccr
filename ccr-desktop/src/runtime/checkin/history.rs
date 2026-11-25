//! Local history queries never acquire a writer, run migrations or load a key.
use super::*;
mod days;
use ccr_db::models::checkin::{CheckinRecord, CheckinStatus};
pub use days::*;

pub const RECORDS_PER_PAGE: usize = 20;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RecordQuery {
    pub account: Option<String>,
    pub provider: Option<String>,
    pub status: Option<String>,
    pub keyword: String,
    /// One-based; zero requests the first page.
    pub page: usize,
}

impl RecordQuery {
    pub fn filtered(&self) -> bool {
        self.account.is_some()
            || self.provider.is_some()
            || self.status.is_some()
            || !self.keyword.trim().is_empty()
    }
}

#[derive(Clone, Debug)]
pub struct HistoryRecord {
    pub id: String,
    pub account_id: String,
    pub account: Option<String>,
    pub provider: Option<String>,
    pub status: CheckinStatus,
    pub at: String,
    pub reason: Option<String>,
    pub error_code: Option<String>,
    pub reward: Option<String>,
    pub before: Option<String>,
    pub after: Option<String>,
}

pub fn credential_error_code(code: Option<&str>) -> bool {
    matches!(code, Some("cookie_expired" | "crypto_error"))
}

impl HistoryRecord {
    pub fn needs_credentials(&self) -> bool {
        self.status == CheckinStatus::Failed && credential_error_code(self.error_code.as_deref())
    }

    fn from_record(row: CheckinRecord, account: Option<String>, provider: Option<String>) -> Self {
        Self {
            id: row.id,
            account_id: row.account_id,
            account: account.as_deref().map(redact_log_text),
            provider: provider.as_deref().map(redact_log_text),
            status: row.status,
            at: row.checked_in_at.to_rfc3339(),
            reason: row.message.as_deref().map(redact_log_text),
            error_code: row.error_code.as_deref().map(redact_log_text),
            reward: row.reward.as_deref().map(redact_log_text),
            before: row.balance_before.and_then(finite_number),
            after: row.balance_after.and_then(finite_number),
        }
    }
}

#[derive(Clone, Debug)]
pub struct RecordPage {
    pub rows: Vec<HistoryRecord>,
    pub total: usize,
    pub has_records: bool,
    pub page: usize,
    pub enabled: usize,
    pub checked: usize,
    pub failed: usize,
}

impl Services {
    fn read_checkin_history<T>(
        &self,
        read: impl FnOnce(&rusqlite::Connection) -> rusqlite::Result<T>,
    ) -> Result<Option<T>, CheckinError> {
        let Some(path) = database_path(&self.source)? else {
            return Ok(None);
        };
        let mut conn = open_reader(&path)?;
        let tx = conn.transaction().map_err(|_| CheckinError::Storage)?;
        let result = read(&tx).map_err(|_| CheckinError::Storage)?;
        tx.commit().map_err(|_| CheckinError::Storage)?;
        Ok(Some(result))
    }

    pub fn checkin_records(&self, query: &RecordQuery) -> Result<Option<RecordPage>, CheckinError> {
        use rusqlite::OptionalExtension;
        self.read_checkin_history(|conn| {
            let read_page = |page| {
                checkin_repo::get_records_paginated_advanced(
                    conn,
                    query.status.as_deref(),
                    query.account.as_deref(),
                    query.provider.as_deref(),
                    Some(&query.keyword),
                    page,
                    RECORDS_PER_PAGE,
                )
            };
            let requested = query.page.clamp(1, i64::MAX as usize / RECORDS_PER_PAGE);
            let (mut rows, total) = read_page(requested)?;
            let page = requested.min(total.div_ceil(RECORDS_PER_PAGE).max(1));
            if page != requested {
                rows = read_page(page)?.0;
            }
            let mut names = conn.prepare(
                "SELECT a.name, p.name FROM checkin_accounts a
                LEFT JOIN checkin_providers p ON p.id = a.provider_id WHERE a.id = ?1",
            )?;
            let rows = rows
                .into_iter()
                .map(|row| {
                    let pair = names
                        .query_row([&row.account_id], |r| Ok((r.get(0)?, r.get(1)?)))
                        .optional()?
                        .unwrap_or_default();
                    Ok(HistoryRecord::from_record(row, pair.0, pair.1))
                })
                .collect::<rusqlite::Result<_>>()?;
            let enabled: Vec<String> = conn
                .prepare("SELECT id FROM checkin_accounts WHERE enabled = 1")?
                .query_map([], |row| row.get(0))?
                .collect::<rusqlite::Result<_>>()?;
            let (checked, failed) = checkin_repo::get_today_status_counts(conn, &enabled)?;
            let has_records = total > 0
                || conn.query_row("SELECT EXISTS(SELECT 1 FROM checkin_records)", [], |r| {
                    r.get(0)
                })?;
            Ok(RecordPage {
                rows,
                total,
                has_records,
                page,
                enabled: enabled.len(),
                checked,
                failed,
            })
        })
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum HistoryQuery {
    Records(RecordQuery),
    Account {
        account: String,
        month: chrono::NaiveDate,
        trend_days: u32,
    },
}
#[derive(Clone)]
pub enum HistoryResult {
    Records(Option<RecordPage>),
    Account(Option<AccountHistory>),
}
impl Services {
    pub fn checkin_history_request(
        &self,
        key: RequestKey,
        query: HistoryQuery,
    ) -> Result<TaskSubscription<HistoryResult, CheckinError>, AdmissionError> {
        let local = EnvironmentPolicy::Independent
            .admit(&key.environment)
            .is_ok();
        let services = self.clone();
        self.executor.read(key, move |_, _| {
            if !local {
                return Err(CheckinError::Unsupported);
            }
            match query {
                HistoryQuery::Records(q) => {
                    services.checkin_records(&q).map(HistoryResult::Records)
                }
                HistoryQuery::Account {
                    account,
                    month,
                    trend_days,
                } => services
                    .checkin_account_history(&account, month, trend_days)
                    .map(HistoryResult::Account),
            }
        })
    }
}
