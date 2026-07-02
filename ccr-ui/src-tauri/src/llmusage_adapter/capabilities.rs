use std::collections::BTreeMap;

use rusqlite::{Connection, OpenFlags};
use serde::Serialize;

use super::{AppPaths, error::LlmusageAdapterError};
use crate::process::std_command;

pub const MIN_SUPPORTED_SCHEMA_VERSION: i64 = 10;
pub const PROVIDER_BREAKDOWN_SCHEMA_VERSION: i64 = 14;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum FeatureKey {
    Overview,
    DailyTrends,
    ModelBreakdown,
    ProviderBreakdown,
    ProjectBreakdown,
    Heatmap,
    Logs,
    Diagnostics,
    HomeOverview,
    SyncJsonEvents,
    Cancel,
}

impl FeatureKey {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Overview => "overview",
            Self::DailyTrends => "daily_trends",
            Self::ModelBreakdown => "model_breakdown",
            Self::ProviderBreakdown => "provider_breakdown",
            Self::ProjectBreakdown => "project_breakdown",
            Self::Heatmap => "heatmap",
            Self::Logs => "logs",
            Self::Diagnostics => "diagnostics",
            Self::HomeOverview => "home_overview",
            Self::SyncJsonEvents => "sync_json_events",
            Self::Cancel => "cancel",
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UnsupportedReason {
    CliMissing,
    DbMissing,
    DbUnreadable,
    SchemaUnsupported,
    MissingTable,
    MissingColumn,
    WaitingForLlmusage,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FeatureCapability {
    pub supported: bool,
    pub reason: Option<UnsupportedReason>,
    pub detail: Option<String>,
}

impl FeatureCapability {
    pub fn supported() -> Self {
        Self {
            supported: true,
            reason: None,
            detail: None,
        }
    }

    pub fn unsupported(reason: UnsupportedReason, detail: impl Into<String>) -> Self {
        Self {
            supported: false,
            reason: Some(reason),
            detail: Some(detail.into()),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CapabilityReport {
    pub cli_available: bool,
    pub cli_version: Option<String>,
    pub root_dir: String,
    pub db_path: String,
    pub db_exists: bool,
    pub db_readable: bool,
    pub schema_version: Option<i64>,
    pub features: BTreeMap<String, FeatureCapability>,
}

impl CapabilityReport {
    pub fn detect(paths: &AppPaths) -> Self {
        let cli_version = detect_cli_version();
        let cli_available = cli_version.is_some();
        let db_exists = paths.db_path.is_file();
        let mut db_readable = false;
        let mut schema_version = None;
        let mut features = BTreeMap::new();

        if db_exists && let Ok(conn) = open_readonly_for_capabilities(paths) {
            db_readable = true;
            schema_version = read_schema_version(&conn).ok().flatten();
            populate_db_features(&conn, schema_version, &mut features);
        }

        if !db_exists || !db_readable {
            let reason = if db_exists {
                UnsupportedReason::DbUnreadable
            } else {
                UnsupportedReason::DbMissing
            };
            let detail = if db_exists {
                format!("llmusage DB is not readable at {}", paths.db_path.display())
            } else {
                format!("llmusage DB does not exist at {}", paths.db_path.display())
            };
            for key in [
                FeatureKey::Overview,
                FeatureKey::DailyTrends,
                FeatureKey::ModelBreakdown,
                FeatureKey::ProviderBreakdown,
                FeatureKey::ProjectBreakdown,
                FeatureKey::Heatmap,
                FeatureKey::Logs,
                FeatureKey::Diagnostics,
                FeatureKey::HomeOverview,
            ] {
                features.insert(
                    key.as_str().to_string(),
                    FeatureCapability::unsupported(reason.clone(), detail.clone()),
                );
            }
        }

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
            db_exists,
            db_readable,
            schema_version,
            features,
        }
    }
}

pub fn ensure_feature(paths: &AppPaths, feature: FeatureKey) -> Result<(), LlmusageAdapterError> {
    if !paths.db_path.is_file() {
        return Err(LlmusageAdapterError::DbMissing(paths.db_path.clone()));
    }
    let conn = open_readonly_for_capabilities(paths).map_err(|error| {
        LlmusageAdapterError::DbUnreadable(format!("{}: {error}", paths.db_path.display()))
    })?;
    let schema_version = read_schema_version(&conn)?;
    let expected_schema_version = min_schema_version(feature);
    if schema_version.unwrap_or_default() < expected_schema_version {
        return Err(LlmusageAdapterError::SchemaUnsupported {
            expected: expected_schema_version,
            actual: schema_version,
        });
    }
    let requirements = required_columns(feature);
    for (table, columns) in requirements {
        if !table_exists(&conn, table)? {
            return Err(LlmusageAdapterError::FeatureUnavailable {
                feature: feature.as_str(),
                reason: format!("missing required table `{table}`"),
            });
        }
        for column in columns {
            if !column_exists(&conn, table, column)? {
                return Err(LlmusageAdapterError::FeatureUnavailable {
                    feature: feature.as_str(),
                    reason: format!("missing required column `{table}.{column}`"),
                });
            }
        }
    }
    Ok(())
}

fn populate_db_features(
    conn: &Connection,
    schema_version: Option<i64>,
    features: &mut BTreeMap<String, FeatureCapability>,
) {
    for key in [
        FeatureKey::Overview,
        FeatureKey::DailyTrends,
        FeatureKey::ModelBreakdown,
        FeatureKey::ProviderBreakdown,
        FeatureKey::ProjectBreakdown,
        FeatureKey::Heatmap,
        FeatureKey::Logs,
        FeatureKey::Diagnostics,
        FeatureKey::HomeOverview,
    ] {
        let expected_schema_version = min_schema_version(key);
        let schema_ok = schema_version.unwrap_or_default() >= expected_schema_version;
        let cap = if !schema_ok {
            FeatureCapability::unsupported(
                UnsupportedReason::SchemaUnsupported,
                format!("expected schema >= {expected_schema_version}, got {schema_version:?}"),
            )
        } else {
            match missing_requirement(conn, key) {
                Ok(None) => FeatureCapability::supported(),
                Ok(Some((reason, detail))) => FeatureCapability::unsupported(reason, detail),
                Err(error) => FeatureCapability::unsupported(
                    UnsupportedReason::DbUnreadable,
                    error.to_string(),
                ),
            }
        };
        features.insert(key.as_str().to_string(), cap);
    }
}

fn missing_requirement(
    conn: &Connection,
    feature: FeatureKey,
) -> rusqlite::Result<Option<(UnsupportedReason, String)>> {
    for (table, columns) in required_columns(feature) {
        if !table_exists(conn, table)? {
            return Ok(Some((
                UnsupportedReason::MissingTable,
                format!("missing table `{table}`"),
            )));
        }
        for column in columns {
            if !column_exists(conn, table, column)? {
                return Ok(Some((
                    UnsupportedReason::MissingColumn,
                    format!("missing column `{table}.{column}`"),
                )));
            }
        }
    }
    Ok(None)
}

fn min_schema_version(feature: FeatureKey) -> i64 {
    match feature {
        FeatureKey::ProviderBreakdown => PROVIDER_BREAKDOWN_SCHEMA_VERSION,
        _ => MIN_SUPPORTED_SCHEMA_VERSION,
    }
}

pub fn required_columns(feature: FeatureKey) -> Vec<(&'static str, Vec<&'static str>)> {
    match feature {
        FeatureKey::Overview => vec![(
            "usage_bucket_30m",
            vec![
                "source",
                "hour_start",
                "input_tokens",
                "cache_read_tokens",
                "output_tokens",
                "reasoning_output_tokens",
                "total_tokens",
                "event_count",
                "cost_with_cache_usd",
            ],
        )],
        FeatureKey::DailyTrends | FeatureKey::Heatmap | FeatureKey::HomeOverview => vec![(
            "usage_bucket_30m",
            vec![
                "source",
                "hour_start",
                "input_tokens",
                "cache_read_tokens",
                "cache_creation_tokens",
                "output_tokens",
                "reasoning_output_tokens",
                "total_tokens",
                "event_count",
                "cost_with_cache_usd",
            ],
        )],
        FeatureKey::ModelBreakdown => vec![(
            "usage_bucket_30m",
            vec![
                "model",
                "input_tokens",
                "cache_read_tokens",
                "cache_creation_tokens",
                "output_tokens",
                "reasoning_output_tokens",
                "total_tokens",
                "event_count",
                "cost_with_cache_usd",
                "cost_without_cache_usd",
                "pricing_status",
                "pricing_source",
                "pricing_rate",
            ],
        )],
        FeatureKey::ProviderBreakdown => vec![
            (
                "usage_bucket_30m",
                vec![
                    "provider_label",
                    "input_tokens",
                    "cache_read_tokens",
                    "cache_creation_tokens",
                    "output_tokens",
                    "reasoning_output_tokens",
                    "total_tokens",
                    "event_count",
                    "cost_with_cache_usd",
                    "cost_without_cache_usd",
                ],
            ),
            ("usage_event", vec!["provider_label"]),
        ],
        FeatureKey::ProjectBreakdown => vec![(
            "usage_bucket_30m",
            vec![
                "project_hash",
                "project_label",
                "project_ref",
                "total_tokens",
                "event_count",
                "cost_with_cache_usd",
            ],
        )],
        FeatureKey::Logs => vec![(
            "usage_event",
            vec![
                "event_key",
                "source",
                "model",
                "event_at",
                "input_tokens",
                "cache_read_tokens",
                "cache_creation_tokens",
                "output_tokens",
                "reasoning_output_tokens",
                "total_tokens",
                "project_hash",
                "project_label",
                "project_ref",
                "project_path",
                "cost_with_cache_usd",
                "cost_without_cache_usd",
                "pricing_status",
                "pricing_source",
            ],
        )],
        FeatureKey::Diagnostics => vec![
            ("source_file", vec!["source", "state"]),
            (
                "source_sync_status",
                vec!["source", "recent_completed_at", "history_completed_at"],
            ),
        ],
        FeatureKey::SyncJsonEvents | FeatureKey::Cancel => vec![],
    }
}

pub fn open_readonly_for_capabilities(paths: &AppPaths) -> rusqlite::Result<Connection> {
    let conn = Connection::open_with_flags(
        &paths.db_path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_URI,
    )?;
    conn.busy_timeout(std::time::Duration::from_secs(5))?;
    Ok(conn)
}

pub fn read_schema_version(conn: &Connection) -> rusqlite::Result<Option<i64>> {
    if !table_exists(conn, "meta")?
        || !column_exists(conn, "meta", "key")?
        || !column_exists(conn, "meta", "value")?
    {
        return Ok(None);
    }
    match conn.query_row(
        "SELECT value FROM meta WHERE key = 'schema_version'",
        [],
        |row| row.get::<_, String>(0),
    ) {
        Ok(value) => Ok(value.parse::<i64>().ok()),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(error) => Err(error),
    }
}

pub fn table_exists(conn: &Connection, table: &str) -> rusqlite::Result<bool> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type IN ('table', 'view') AND name = ?1",
        [table],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

pub fn column_exists(conn: &Connection, table: &str, column: &str) -> rusqlite::Result<bool> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({})", quote_ident(table)))?;
    let rows = stmt.query_map([], |row| row.get::<_, String>(1))?;
    for row in rows {
        if row? == column {
            return Ok(true);
        }
    }
    Ok(false)
}

fn quote_ident(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
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
    use rusqlite::Connection;

    #[test]
    fn schema_gate_rejects_old_schema() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE meta(key TEXT PRIMARY KEY, value TEXT NOT NULL); INSERT INTO meta(key, value) VALUES ('schema_version', '9');").unwrap();
        assert_eq!(read_schema_version(&conn).unwrap(), Some(9));
    }

    #[test]
    fn required_logs_columns_include_raw_join_keys() {
        let cols = required_columns(FeatureKey::Logs);
        assert!(
            cols.iter()
                .any(|(table, cols)| *table == "usage_event" && cols.contains(&"event_key"))
        );
    }

    #[test]
    fn provider_breakdown_requires_schema_14() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE meta(key TEXT PRIMARY KEY, value TEXT NOT NULL);
             INSERT INTO meta(key, value) VALUES ('schema_version', '13');",
        )
        .unwrap();
        let mut features = BTreeMap::new();

        populate_db_features(&conn, Some(13), &mut features);

        let capability = features.get("provider_breakdown").unwrap();
        assert!(!capability.supported);
        assert_eq!(
            capability.reason.as_ref(),
            Some(&UnsupportedReason::SchemaUnsupported)
        );
        assert!(
            capability
                .detail
                .as_deref()
                .unwrap_or_default()
                .contains("expected schema >= 14")
        );
    }

    #[test]
    fn provider_breakdown_requires_provider_label_columns() {
        let cols = required_columns(FeatureKey::ProviderBreakdown);
        assert!(cols.iter().any(|(table, cols)| {
            *table == "usage_bucket_30m" && cols.contains(&"provider_label")
        }));
        assert!(
            cols.iter()
                .any(|(table, cols)| *table == "usage_event" && cols.contains(&"provider_label"))
        );
    }

    #[test]
    fn provider_breakdown_reports_missing_provider_label_column() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            r#"
            CREATE TABLE meta(key TEXT PRIMARY KEY, value TEXT NOT NULL);
            INSERT INTO meta(key, value) VALUES ('schema_version', '14');
            CREATE TABLE usage_bucket_30m(total_tokens INTEGER NOT NULL);
            CREATE TABLE usage_event(event_key TEXT NOT NULL);
            "#,
        )
        .unwrap();
        let mut features = BTreeMap::new();

        populate_db_features(&conn, Some(14), &mut features);

        let capability = features.get("provider_breakdown").unwrap();
        assert!(!capability.supported);
        assert_eq!(
            capability.reason.as_ref(),
            Some(&UnsupportedReason::MissingColumn)
        );
        assert!(
            capability
                .detail
                .as_deref()
                .unwrap_or_default()
                .contains("usage_bucket_30m.provider_label")
        );
    }
}
