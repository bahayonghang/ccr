//! Explicit provider diagnostics using the existing domain services.
use super::*;

pub enum InspectionCommand {
    Connection(String),
    Waf(String),
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
    }
}
