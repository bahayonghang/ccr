use super::error::LlmusageAdapterError;
use super::AppPaths;

pub use ccr_usage::{build_filter, DiagnosticsPayload, LogsPage, LogsQuery, QueryFilter};

/// Thin wrapper over `ccr_usage::Dashboard`: identical query surface, with
/// errors mapped to the adapter-owned `LlmusageAdapterError` at the boundary.
#[derive(Debug)]
pub struct Dashboard {
    inner: ccr_usage::Dashboard,
}

pub fn open_dashboard(paths: AppPaths) -> Result<Dashboard, LlmusageAdapterError> {
    Dashboard::open(paths)
}

impl Dashboard {
    pub fn open(paths: AppPaths) -> Result<Self, LlmusageAdapterError> {
        Ok(Self {
            inner: ccr_usage::Dashboard::open(paths).map_err(shared_usage_error)?,
        })
    }

    pub fn overview(
        &self,
        filter: &QueryFilter,
    ) -> Result<ccr_usage::OverviewPayload, LlmusageAdapterError> {
        self.inner.overview(filter).map_err(shared_usage_error)
    }

    pub fn trends_daily(
        &self,
        filter: &QueryFilter,
    ) -> Result<Vec<ccr_usage::DailyTrendDto>, LlmusageAdapterError> {
        self.inner.trends_daily(filter).map_err(shared_usage_error)
    }

    pub fn model_breakdown(
        &self,
        filter: &QueryFilter,
    ) -> Result<Vec<ccr_usage::ModelBreakdown>, LlmusageAdapterError> {
        self.inner
            .model_breakdown(filter)
            .map_err(shared_usage_error)
    }

    pub fn provider_breakdown(
        &self,
        filter: &QueryFilter,
    ) -> Result<Vec<ccr_usage::ProviderBreakdownDto>, LlmusageAdapterError> {
        self.inner
            .provider_breakdown(filter)
            .map_err(shared_usage_error)
    }

    pub fn project_breakdown(
        &self,
        filter: &QueryFilter,
    ) -> Result<Vec<ccr_usage::ProjectBreakdown>, LlmusageAdapterError> {
        self.inner
            .project_breakdown(filter)
            .map_err(shared_usage_error)
    }

    pub fn source_breakdown(
        &self,
        filter: &QueryFilter,
    ) -> Result<Vec<ccr_usage::SourceBreakdownDto>, LlmusageAdapterError> {
        self.inner
            .source_breakdown(filter)
            .map_err(shared_usage_error)
    }

    pub fn heatmap(
        &self,
        filter: &QueryFilter,
        days: u32,
    ) -> Result<Vec<ccr_usage::HeatmapPoint>, LlmusageAdapterError> {
        self.inner.heatmap(filter, days).map_err(shared_usage_error)
    }

    pub fn logs(&self, query: &LogsQuery) -> Result<LogsPage, LlmusageAdapterError> {
        self.inner.logs(query).map_err(shared_usage_error)
    }

    pub fn diagnostics(&self) -> Result<DiagnosticsPayload, LlmusageAdapterError> {
        self.inner.diagnostics().map_err(shared_usage_error)
    }

    pub fn home_overview(
        &self,
        filter: &QueryFilter,
    ) -> Result<ccr_usage::HomeOverviewPayload, LlmusageAdapterError> {
        self.inner.home_overview(filter).map_err(shared_usage_error)
    }
}

fn shared_usage_error(error: ccr_usage::UsageError) -> LlmusageAdapterError {
    match error {
        ccr_usage::UsageError::DbMissing(path) => LlmusageAdapterError::DbMissing(path),
        ccr_usage::UsageError::DbUnreadable(message) => LlmusageAdapterError::DbUnreadable(message),
        ccr_usage::UsageError::SchemaUnsupported { expected, actual } => {
            LlmusageAdapterError::SchemaUnsupported { expected, actual }
        }
        ccr_usage::UsageError::FeatureUnavailable { feature, reason } => {
            LlmusageAdapterError::FeatureUnavailable { feature, reason }
        }
        ccr_usage::UsageError::Query(message) => LlmusageAdapterError::Query(message),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_usage_error_maps_every_variant() {
        let path = std::path::PathBuf::from("/tmp/llmusage.db");
        assert!(matches!(
            shared_usage_error(ccr_usage::UsageError::DbMissing(path.clone())),
            LlmusageAdapterError::DbMissing(mapped) if mapped == path
        ));
        assert!(matches!(
            shared_usage_error(ccr_usage::UsageError::DbUnreadable("io".to_string())),
            LlmusageAdapterError::DbUnreadable(message) if message == "io"
        ));
        assert!(matches!(
            shared_usage_error(ccr_usage::UsageError::SchemaUnsupported {
                expected: 14,
                actual: Some(13),
            }),
            LlmusageAdapterError::SchemaUnsupported {
                expected: 14,
                actual: Some(13),
            }
        ));
        assert!(matches!(
            shared_usage_error(ccr_usage::UsageError::FeatureUnavailable {
                feature: "provider_breakdown",
                reason: "missing column".to_string(),
            }),
            LlmusageAdapterError::FeatureUnavailable {
                feature: "provider_breakdown",
                ..
            }
        ));
        assert!(matches!(
            shared_usage_error(ccr_usage::UsageError::Query("boom".to_string())),
            LlmusageAdapterError::Query(message) if message == "boom"
        ));
    }

    #[test]
    fn wrapper_propagates_schema_gate_as_adapter_error() {
        // 契约回归锚点：commands 层依赖 LlmusageAdapterError::SchemaUnsupported /
        // FeatureUnavailable 做 provider_stats 降级，wrapper 必须保住该错误面。
        let temp = tempfile::TempDir::new().expect("temp dir should be created");
        let db_path = temp.path().join("llmusage.db");
        let conn = rusqlite::Connection::open(&db_path).expect("test db should open");
        conn.execute_batch(
            r#"
            CREATE TABLE meta(key TEXT PRIMARY KEY, value TEXT NOT NULL);
            INSERT INTO meta(key, value) VALUES ('schema_version', '13');
            "#,
        )
        .expect("old schema should be created");
        drop(conn);

        let dashboard = Dashboard::open(AppPaths::from_root(temp.path()))
            .expect("dashboard should open schema 13 DB");
        let error = dashboard
            .provider_breakdown(&QueryFilter::default())
            .expect_err("provider breakdown should reject old schema");

        assert!(matches!(
            error,
            LlmusageAdapterError::SchemaUnsupported {
                expected: 14,
                actual: Some(13)
            }
        ));
    }
}
