//! Read-side account diagnostics and history reuse the existing domain queries.
use super::*;

#[derive(Clone)]
pub struct RecordQuery {
    pub account: Option<String>,
    pub provider: Option<String>,
    pub status: Option<String>,
    pub keyword: String,
    pub page: usize,
}
pub enum InspectionCommand {
    Connection(String),
    Waf(String),
    Records(RecordQuery),
    Dashboard {
        account: String,
        year: i32,
        month: u32,
        days: u32,
    },
}
#[derive(Clone)]
pub enum InspectionReport {
    Connection {
        account: String,
        success: bool,
    },
    Waf {
        account: String,
        success: bool,
        status: Option<u16>,
        challenge: String,
    },
    Records {
        query: RecordQuery,
        rows: Vec<RecordSummary>,
        total: usize,
    },
    Dashboard {
        year: i32,
        month: u32,
        data: Box<ccr_db::models::checkin::CheckinAccountDashboardResponse>,
    },
}
pub(super) async fn inspect(
    storage: &Storage,
    source: &ConfigSource,
    owner: &CheckinOperations,
    command: InspectionCommand,
) -> Result<InspectionReport, OperationError> {
    match command {
        InspectionCommand::Connection(account) => {
            let service = super::super::service_for_account(storage, source, owner, &account)?;
            let success = service
                .test_connection(&account)
                .await
                .map_err(service_error)?;
            Ok(InspectionReport::Connection { account, success })
        }
        InspectionCommand::Waf(account) => {
            let service = super::super::service_for_account(storage, source, owner, &account)?;
            let result = service
                .validate_cached_waf_access(&account)
                .await
                .map_err(service_error)?;
            Ok(InspectionReport::Waf {
                account,
                success: result.success,
                status: result.status_code,
                challenge: redact_log_text(&result.challenge),
            })
        }
        InspectionCommand::Records(mut query) => {
            query.page = query.page.max(1);
            if query.page > 100_000
                || query.keyword.len() > 512
                || query.status.as_deref().is_some_and(|value| {
                    !matches!(
                        value,
                        "success" | "already_checked_in" | "failed" | "skipped"
                    )
                })
            {
                return Err(OperationError::QueryInvalid);
            }
            let (rows, total) = RecordManager::with_db(storage.db.clone())
                .get_paginated_advanced(
                    query.status.as_deref(),
                    query.account.as_deref(),
                    query.provider.as_deref(),
                    Some(&query.keyword),
                    query.page,
                    20,
                )
                .map_err(|_| OperationError::Storage)?;
            let rows = rows
                .into_iter()
                .map(|row| RecordSummary {
                    account_id: row.account_id,
                    status: row.status,
                    at: row.checked_in_at.to_rfc3339(),
                })
                .collect();
            Ok(InspectionReport::Records { query, rows, total })
        }
        InspectionCommand::Dashboard {
            account,
            year,
            month,
            days,
        } => {
            if !(1..=12).contains(&month)
                || !(1..=366).contains(&days)
                || !(1..=9999).contains(&year)
            {
                return Err(OperationError::QueryInvalid);
            }
            storage
                .accounts()
                .get(&account)
                .map_err(|_| OperationError::Invalid)?;
            let service = CheckinService::with_db(storage.crypto.clone(), storage.db.clone());
            let data = service
                .get_account_dashboard(&account, year, month, days)
                .map_err(|_| OperationError::Storage)?;
            Ok(InspectionReport::Dashboard {
                year,
                month,
                data: Box::new(data),
            })
        }
    }
}
