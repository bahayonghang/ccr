//! Read-only usage capabilities and the closed installed-CLI version probe.
use crate::jobs::process::{ProcessDescriptor, ProcessGateway};
use ccr_usage::{AppPaths, DbCapabilities, FeatureCapability, FeatureKey, UnsupportedReason};
use std::{collections::BTreeMap, ffi::OsString};

#[derive(Clone, Debug)]
pub struct Report {
    pub cli_available: bool,
    pub cli_version: Option<String>,
    pub root_dir: String,
    pub db_path: String,
    pub db_exists: bool,
    pub db_readable: bool,
    pub schema_version: Option<i64>,
    pub features: BTreeMap<String, FeatureCapability>,
}

impl Report {
    /// `probe_cli=false` never resolves or executes an ambient binary. It is a
    /// capability restriction, not evidence that the host CLI is absent.
    pub async fn detect(paths: &AppPaths, probe_cli: bool) -> Self {
        let cli_version = if probe_cli {
            detect_cli_version().await
        } else {
            None
        };
        let db = DbCapabilities::detect(paths);
        Self::merge(paths, db, cli_version, probe_cli)
    }

    pub fn merge(
        paths: &AppPaths,
        db: DbCapabilities,
        cli_version: Option<String>,
        probe_cli: bool,
    ) -> Self {
        let cli_available = probe_cli && cli_version.is_some();
        let mut features = db.features;
        features.insert(
            FeatureKey::SyncJsonEvents.as_str().into(),
            if cli_available {
                FeatureCapability::supported()
            } else {
                FeatureCapability::unsupported(
                    if probe_cli {
                        UnsupportedReason::CliMissing
                    } else {
                        UnsupportedReason::WaitingForLlmusage
                    },
                    if probe_cli {
                        "installed `llmusage` command was not found"
                    } else {
                        "CLI execution is disabled for this fixture"
                    },
                )
            },
        );
        features.insert(FeatureKey::Cancel.as_str().into(), FeatureCapability::unsupported(
            UnsupportedReason::WaitingForLlmusage,
            "llmusage needs a documented graceful cancellation contract; ccr-ui can only mark the local job cancelled",
        ));
        Self {
            cli_available,
            cli_version: probe_cli.then_some(cli_version).flatten(),
            root_dir: paths.root_dir.display().to_string(),
            db_path: paths.db_path.display().to_string(),
            db_exists: db.db_exists,
            db_readable: db.db_readable,
            schema_version: db.schema_version,
            features,
        }
    }
}

async fn detect_cli_version() -> Option<String> {
    let descriptor = ProcessDescriptor::llmusage_version("llmusage").ok()?;
    let output = ProcessGateway::execute(&descriptor, &[OsString::from("--version")])
        .await
        .ok()?;
    if !output.status.success()
        || output.timed_out
        || output.stdout_truncated
        || output.stderr_truncated
    {
        return None;
    }
    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (!version.is_empty()).then_some(version)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn fixture_capabilities_do_not_create_storage_or_probe_host_cli() {
        let root = tempfile::tempdir().unwrap();
        let paths = AppPaths::from_root(root.path().join("absent"));
        let report = Report::detect(&paths, false).await;
        assert!(!paths.root_dir.exists());
        assert!(!report.db_exists);
        assert!(!report.cli_available);
        assert!(report.cli_version.is_none());
        for feature in ccr_usage::DB_BACKED_FEATURES {
            assert_eq!(
                report.features[feature.as_str()].reason,
                Some(UnsupportedReason::DbMissing)
            );
        }
        assert!(
            report.features[FeatureKey::SyncJsonEvents.as_str()]
                .detail
                .as_deref()
                .unwrap()
                .contains("disabled for this fixture")
        );
        assert!(!report.features[FeatureKey::Cancel.as_str()].supported);
    }

    #[test]
    fn merged_cli_capability_does_not_claim_graceful_upstream_cancel_or_fix_missing_db() {
        let root = tempfile::tempdir().unwrap();
        let paths = AppPaths::from_root(root.path());
        let db = DbCapabilities::detect(&paths);
        let report = Report::merge(&paths, db, Some("synthetic-version".into()), true);
        assert!(report.cli_available);
        assert!(report.features[FeatureKey::SyncJsonEvents.as_str()].supported);
        assert!(!report.features[FeatureKey::Overview.as_str()].supported);
        assert!(!report.features[FeatureKey::Cancel.as_str()].supported);
    }
}
