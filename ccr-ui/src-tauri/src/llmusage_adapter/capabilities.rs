use std::collections::BTreeMap;

use serde::Serialize;
use ts_rs::TS;

use super::AppPaths;
use crate::process::std_command;

pub use ccr_usage::{DbCapabilities, FeatureCapability, FeatureKey, UnsupportedReason};

/// Tauri-facing capability payload: shared DB capabilities from `ccr_usage`
/// merged with desktop-only CLI detection.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/usage/")]
pub struct CapabilityReport {
    pub cli_available: bool,
    pub cli_version: Option<String>,
    pub root_dir: String,
    pub db_path: String,
    pub db_exists: bool,
    pub db_readable: bool,
    // i64/u64 走 serde_json number，ts(as = "f64") 避免 ts-rs 默认 bigint
    #[ts(as = "Option<f64>")]
    pub schema_version: Option<i64>,
    pub features: BTreeMap<String, FeatureCapability>,
}

impl CapabilityReport {
    pub fn detect(paths: &AppPaths) -> Self {
        let cli_version = detect_cli_version();
        let cli_available = cli_version.is_some();
        let db = DbCapabilities::detect(paths);
        let mut features = db.features;

        features.insert(
            FeatureKey::SyncJsonEvents.as_str().to_string(),
            if cli_available {
                FeatureCapability::supported()
            } else {
                FeatureCapability::unsupported(
                    UnsupportedReason::CliMissing,
                    "installed `llmusage` command was not found",
                )
            },
        );
        // The current CLI exposes streaming json-events but no external graceful cancel command.
        features.insert(
            FeatureKey::Cancel.as_str().to_string(),
            FeatureCapability::unsupported(
                UnsupportedReason::WaitingForLlmusage,
                "llmusage needs a documented graceful cancellation contract; ccr-ui can only mark the local job cancelled",
            ),
        );

        Self {
            cli_available,
            cli_version,
            root_dir: paths.root_dir.display().to_string(),
            db_path: paths.db_path.display().to_string(),
            db_exists: db.db_exists,
            db_readable: db.db_readable,
            schema_version: db.schema_version,
            features,
        }
    }
}

fn detect_cli_version() -> Option<String> {
    // 走 process::std_command：在 Windows release 下隐藏 conhost 窗口，避免应用启动时
    // 探测 `llmusage --version` 一闪而过的弹窗。
    let output = std_command("llmusage").arg("--version").output().ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if stdout.is_empty() {
        None
    } else {
        Some(stdout)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capability_report_merges_db_and_cli_features() {
        // 无 DB 的临时目录：9 个 DB-backed 特征应统一 DbMissing，
        // CLI 域两键由 adapter 拼装（sync_json_events 的 supported 取决于本机
        // 是否安装 llmusage，这里只断言键存在与 cancel 的固定语义）。
        let temp = tempfile::TempDir::new().expect("temp dir should be created");
        let report = CapabilityReport::detect(&AppPaths::from_root(temp.path()));

        assert!(!report.db_exists);
        assert!(!report.db_readable);
        for key in ccr_usage::DB_BACKED_FEATURES {
            let capability = report
                .features
                .get(key.as_str())
                .expect("db-backed feature present");
            assert!(!capability.supported);
            assert_eq!(
                capability.reason.as_ref(),
                Some(&UnsupportedReason::DbMissing)
            );
        }
        assert!(report.features.contains_key("sync_json_events"));
        let cancel = report.features.get("cancel").expect("cancel key present");
        assert!(!cancel.supported);
        assert_eq!(
            cancel.reason.as_ref(),
            Some(&UnsupportedReason::WaitingForLlmusage)
        );
    }
}
