//! Additional Check-in operations reuse the captured database and domain services.
use super::*;
#[path = "inspection.rs"]
mod inspection;
use ccr_checkin::managers::checkin::export_manager::{ImportReceipt, ImportSession};
use ccr_checkin::{BalanceManager, ExportManager, WafCookieManager};
use ccr_core::Secret;
pub use inspection::{InspectionCommand, InspectionReport};

pub enum ExtendedCommand {
    Inspect(InspectionCommand),
    Checkin(Vec<String>),
    Delete(Vec<String>),
    BalanceStats,
    WafList,
    WafSave {
        provider: String,
        cookies: Secret,
    },
    WafDelete(String),
    Export(PathBuf),
    ExportStats(PathBuf),
    Import {
        session: Arc<ImportSession>,
        overwrite: bool,
    },
    CdkCredentials {
        account: String,
        patch: Secret,
    },
    Recharge {
        account: String,
        code: Secret,
    },
}
#[derive(Clone)]
pub struct BatchItem {
    pub account: String,
    pub result: Result<CheckinStatus, OperationError>,
}
#[derive(Clone)]
pub struct WafStatus {
    pub provider: String,
    pub available: bool,
    pub required: Vec<String>,
}
#[derive(Clone)]
pub enum ExtendedReport {
    Inspection(Arc<InspectionReport>),
    Batch(Vec<BatchItem>),
    Deleted {
        completed: Vec<String>,
        failed: Vec<String>,
    },
    Waf(Vec<WafStatus>),
    WafSaved,
    BalanceStats {
        groups: Vec<ccr_checkin::managers::checkin::balance_manager::BalanceTotals>,
        unusable: usize,
    },
    Exported {
        stats: bool,
    },
    Imported(ImportReceipt),
    CdkSaved {
        account: String,
    },
    Recharged {
        account: String,
        success: bool,
    },
}

pub(super) async fn execute_extended(
    storage: &Storage,
    source: &ConfigSource,
    owner: &CheckinOperations,
    command: ExtendedCommand,
) -> Result<ExtendedReport, OperationError> {
    let accounts = storage.accounts();
    let providers = storage.providers();
    match command {
        ExtendedCommand::Inspect(command) => inspection::inspect(storage, source, owner, command)
            .await
            .map(|value| ExtendedReport::Inspection(Arc::new(value))),
        ExtendedCommand::Checkin(ids) => {
            if ids.is_empty() || ids.len() > 1000 {
                return Err(OperationError::Invalid);
            }
            let mut results = Vec::new();
            // Keep each existing admission/network restriction and domain check-in outcome.
            for id in ids {
                let result = Box::pin(super::execute(
                    storage,
                    source,
                    owner,
                    Operation::Checkin(id.clone()),
                ))
                .await
                .and_then(|value| match value {
                    Outcome::Checked(status) => Ok(status),
                    _ => Err(OperationError::Invalid),
                });
                results.push(BatchItem {
                    account: id,
                    result,
                });
            }
            Ok(ExtendedReport::Batch(results))
        }
        ExtendedCommand::Delete(ids) => {
            if ids.is_empty() || ids.len() > 1000 {
                return Err(OperationError::Invalid);
            }
            let mut completed = Vec::new();
            let mut failed = Vec::new();
            for id in ids {
                let label = accounts
                    .get(&id)
                    .ok()
                    .map(|row| format!("{} · {}", redact_log_text(&row.name), redact_log_text(&id)))
                    .unwrap_or_else(|| redact_log_text(&id));
                match accounts.delete(&id) {
                    Ok(()) => completed.push(label),
                    Err(_) => failed.push(label),
                }
            }
            Ok(ExtendedReport::Deleted { completed, failed })
        }
        ExtendedCommand::BalanceStats => {
            let (groups, unusable) = BalanceManager::with_db(storage.db.clone())
                .totals_by_currency()
                .map_err(|_| OperationError::Storage)?;
            Ok(ExtendedReport::BalanceStats { groups, unusable })
        }
        ExtendedCommand::WafList => {
            let waf = WafCookieManager::with_db(storage.db.clone());
            let mut rows = Vec::new();
            for provider in providers.load_all().map_err(|_| OperationError::Storage)? {
                let required = WafCookieManager::policy_for_provider(&provider)
                    .map(|p| p.required_cookie_names)
                    .unwrap_or_default();
                rows.push(WafStatus {
                    provider: provider.id.clone(),
                    available: waf
                        .get_valid(&provider.id)
                        .map_err(|_| OperationError::Storage)?
                        .is_some(),
                    required,
                });
            }
            Ok(ExtendedReport::Waf(rows))
        }
        ExtendedCommand::WafSave { provider, cookies } => {
            let entry = providers
                .get(&provider)
                .map_err(|_| OperationError::Invalid)?;
            let values: std::collections::HashMap<String, String> =
                serde_json::from_str(cookies.expose()).map_err(|_| OperationError::Invalid)?;
            if values.is_empty() {
                return Err(OperationError::Invalid);
            }
            let values = if let Some(policy) = WafCookieManager::policy_for_provider(&entry) {
                let selected = WafCookieManager::select_required_cookies(
                    &values,
                    &policy.required_cookie_names,
                );
                if !selected.is_complete() {
                    return Err(OperationError::Invalid);
                }
                selected.cookies
            } else {
                values
            };
            WafCookieManager::with_db(storage.db.clone())
                .save(&provider, values)
                .map_err(|_| OperationError::Storage)?;
            Ok(ExtendedReport::WafSaved)
        }
        ExtendedCommand::WafDelete(id) => {
            providers.get(&id).map_err(|_| OperationError::Invalid)?;
            WafCookieManager::with_db(storage.db.clone())
                .delete(&id)
                .map_err(|_| OperationError::Storage)?;
            Ok(ExtendedReport::WafSaved)
        }
        ExtendedCommand::Export(path) => {
            validate_transfer_path(source, &path)?;
            storage.scoped(|| {
                ExportManager::with_db(&storage.crypto, storage.db.clone())
                    .export_file(&path)
                    .map_err(import_error)
            })?;
            Ok(ExtendedReport::Exported { stats: false })
        }
        ExtendedCommand::ExportStats(path) => {
            validate_transfer_path(source, &path)?;
            ExportManager::with_db(&storage.crypto, storage.db.clone())
                .export_stats_file(&path)
                .map_err(import_error)?;
            Ok(ExtendedReport::Exported { stats: true })
        }
        ExtendedCommand::Import { session, overwrite } => {
            owner.commit_transfer(source, &session, overwrite)
        }
        ExtendedCommand::CdkCredentials { account, patch } => {
            storage.scoped(|| {
                ccr_checkin::services::cdk_recharge::update_credentials(
                    &storage.crypto,
                    storage.db.clone(),
                    &account,
                    &patch,
                )
                .map_err(|_| OperationError::Invalid)
            })?;
            Ok(ExtendedReport::CdkSaved { account })
        }
        ExtendedCommand::Recharge { account, code } => {
            // Explicit recharge can resolve external builtin endpoints. Fixtures never call it.
            if source.is_fixture() {
                return Err(OperationError::FixtureNetwork);
            }
            accounts
                .get(&account)
                .map_err(|_| OperationError::Invalid)?;
            storage.require_existing_crypto()?;
            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .map_err(|_| OperationError::Network)?;
            let result = ccr_checkin::services::cdk_recharge::execute(
                &storage.crypto,
                storage.db.clone(),
                client,
                &account,
                &code,
            )
            .await
            .map_err(|_| OperationError::Network)?;
            Ok(ExtendedReport::Recharged {
                account,
                success: result
                    .get("success")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false),
            })
        }
    }
}

pub(super) fn import_error(
    error: ccr_checkin::managers::checkin::export_manager::ExportError,
) -> OperationError {
    use ccr_checkin::managers::checkin::export_manager::ExportError;
    match error {
        ExportError::DestinationExists => OperationError::ExportDestinationExists,
        ExportError::ExistingKeyUnavailable => OperationError::CryptoKeyUnavailable,
        ExportError::CryptoError(_) => OperationError::ImportCredentials,
        ExportError::Conflict => OperationError::ImportConflict,
        ExportError::VersionIncompatible(_) | ExportError::ImportFailed(_) => {
            OperationError::ImportInvalid
        }
        _ => OperationError::Storage,
    }
}

pub(super) fn validate_transfer_path(
    source: &ConfigSource,
    path: &Path,
) -> Result<(), OperationError> {
    if let Some(layout) = source.benchmark() {
        return layout
            .check_path(path)
            .map_err(|_| OperationError::FixturePath);
    }
    if let ConfigSource::Fixture { registry, .. } = source {
        if !path.is_absolute() {
            return Err(OperationError::FixturePath);
        }
        let root = registry
            .parent()
            .ok_or(OperationError::FixturePath)?
            .canonicalize()
            .map_err(|_| OperationError::Storage)?;
        // Compare filesystem paths, not lock identities: on Windows the latter
        // strip verbatim prefixes and fold case. New fixture files require an
        // existing parent; canonicalization also catches linked parents escaping
        // the fixture. The domain transfer owner still rejects linked paths.
        let target = if path.exists() {
            path
        } else {
            path.parent().ok_or(OperationError::FixturePath)?
        }
        .canonicalize()
        .map_err(|_| OperationError::FixturePath)?;
        if !target.starts_with(&root) {
            return Err(OperationError::FixturePath);
        }
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn transfer_fixture_paths_accept_native_aliases_and_refuse_escapes() {
        let container = tempfile::tempdir().unwrap();
        let root = container.path().join("MixedCaseFixture");
        let sibling = container.path().join("MixedCaseFixture-other");
        std::fs::create_dir(&root).unwrap();
        std::fs::create_dir(&sibling).unwrap();
        let source = ConfigSource::Fixture {
            profiles: root.join("profiles.toml"),
            registry: root.join("registry.toml"),
            delay: std::time::Duration::ZERO,
        };
        for parent in [&root, &root.canonicalize().unwrap()] {
            assert_eq!(
                validate_transfer_path(&source, &parent.join("today.json")),
                Ok(()),
                "ordinary and canonical paths identify the same fixture"
            );
        }
        for target in [
            sibling.join("today.json"),
            root.join("..").join("outside.json"),
            root.join("missing-parent").join("today.json"),
            PathBuf::from("relative.json"),
        ] {
            assert_eq!(
                validate_transfer_path(&source, &target),
                Err(OperationError::FixturePath)
            );
            assert!(!target.exists());
        }
    }
}
