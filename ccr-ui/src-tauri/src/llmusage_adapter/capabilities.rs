use std::collections::BTreeMap;

use serde::Serialize;
use ts_rs::TS;

use super::AppPaths;

pub use ccr_usage::FeatureCapability;

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
    pub async fn detect(paths: &AppPaths) -> Self {
        Self::detect_with_policy(paths, true).await
    }

    pub async fn detect_with_policy(paths: &AppPaths, probe_cli: bool) -> Self {
        let report = ccr_app::usage::capabilities::Report::detect(paths, probe_cli).await;
        Self {
            cli_available: report.cli_available,
            cli_version: report.cli_version,
            root_dir: report.root_dir,
            db_path: report.db_path,
            db_exists: report.db_exists,
            db_readable: report.db_readable,
            schema_version: report.schema_version,
            features: report.features,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ccr_usage::UnsupportedReason;

    #[tokio::test]
    async fn capability_report_merges_db_and_cli_features() {
        // 无 DB 的临时目录：全部 DB-backed 特征应统一 DbMissing，
        // CLI 域两键由 adapter 拼装（sync_json_events 的 supported 取决于本机
        // 是否安装 llmusage，这里只断言键存在与 cancel 的固定语义）。
        let temp = tempfile::TempDir::new().expect("temp dir should be created");
        let report = CapabilityReport::detect(&AppPaths::from_root(temp.path())).await;

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
