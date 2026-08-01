use std::collections::BTreeMap;

use rusqlite::{Connection, OpenFlags};
use serde::Serialize;

use crate::{AppPaths, UsageError};

#[cfg(test)]
std::thread_local! {
    static CAPABILITY_CONNECTION_OPENS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

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

/// Feature keys answered by reading the llmusage SQLite database. CLI-backed
/// keys (`SyncJsonEvents`, `Cancel`) are owned by the desktop adapter.
pub const DB_BACKED_FEATURES: [FeatureKey; 9] = [
    FeatureKey::Overview,
    FeatureKey::DailyTrends,
    FeatureKey::ModelBreakdown,
    FeatureKey::ProviderBreakdown,
    FeatureKey::ProjectBreakdown,
    FeatureKey::Heatmap,
    FeatureKey::Logs,
    FeatureKey::Diagnostics,
    FeatureKey::HomeOverview,
];

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "../../../ccr-ui/src/types/generated/usage/")
)]
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
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "../../../ccr-ui/src/types/generated/usage/")
)]
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

/// Read-only capability snapshot of the llmusage database. Presentation layers
/// (Tauri `CapabilityReport`, …) merge CLI-side keys on top of this.
#[derive(Debug, Clone, Serialize)]
pub struct DbCapabilities {
    pub db_exists: bool,
    pub db_readable: bool,
    pub schema_version: Option<i64>,
    pub features: BTreeMap<String, FeatureCapability>,
}

impl DbCapabilities {
    pub fn detect(paths: &AppPaths) -> Self {
        let db_exists = paths.db_path.is_file();
        let mut db_readable = false;
        let mut schema_version = None;
        let mut features = BTreeMap::new();
        let mut read_error = None;

        if db_exists {
            match open_readonly_for_capabilities(paths) {
                Ok(conn) => match DbCapabilitySnapshot::from_connection(&conn) {
                    Ok(snapshot) => {
                        db_readable = true;
                        schema_version = snapshot.schema_version;
                        features = snapshot.features;
                    }
                    Err(error) => read_error = Some(error.to_string()),
                },
                Err(error) => read_error = Some(error.to_string()),
            }
        }

        if !db_exists || !db_readable {
            let reason = if db_exists {
                UnsupportedReason::DbUnreadable
            } else {
                UnsupportedReason::DbMissing
            };
            let detail = if db_exists {
                match read_error {
                    Some(error) => format!(
                        "llmusage DB is not readable at {}: {error}",
                        paths.db_path.display()
                    ),
                    None => format!("llmusage DB is not readable at {}", paths.db_path.display()),
                }
            } else {
                format!("llmusage DB does not exist at {}", paths.db_path.display())
            };
            for key in DB_BACKED_FEATURES {
                features.insert(
                    key.as_str().to_string(),
                    FeatureCapability::unsupported(reason.clone(), detail.clone()),
                );
            }
        }

        Self {
            db_exists,
            db_readable,
            schema_version,
            features,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct DbCapabilitySnapshot {
    pub(crate) schema_version: Option<i64>,
    features: BTreeMap<String, FeatureCapability>,
}

impl DbCapabilitySnapshot {
    pub(crate) fn from_connection(conn: &Connection) -> Result<Self, UsageError> {
        let schema_version = read_schema_version(conn)?;
        let mut features = BTreeMap::new();
        populate_db_features(conn, schema_version, &mut features);
        Ok(Self {
            schema_version,
            features,
        })
    }

    pub(crate) fn ensure(&self, feature: FeatureKey) -> Result<(), UsageError> {
        let expected = min_schema_version(feature);
        if self.schema_version.unwrap_or_default() < expected {
            return Err(UsageError::SchemaUnsupported {
                expected,
                actual: self.schema_version,
            });
        }
        let Some(capability) = self.features.get(feature.as_str()) else {
            return Err(UsageError::FeatureUnavailable {
                feature: feature.as_str(),
                reason: "capability snapshot is missing the feature".to_string(),
            });
        };
        if capability.supported {
            return Ok(());
        }
        Err(UsageError::FeatureUnavailable {
            feature: feature.as_str(),
            reason: capability
                .detail
                .clone()
                .unwrap_or_else(|| "required database capability is unavailable".to_string()),
        })
    }
}

pub(crate) fn populate_db_features(
    conn: &Connection,
    schema_version: Option<i64>,
    features: &mut BTreeMap<String, FeatureCapability>,
) {
    for key in DB_BACKED_FEATURES {
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

pub(crate) fn open_readonly_for_capabilities(paths: &AppPaths) -> rusqlite::Result<Connection> {
    #[cfg(test)]
    CAPABILITY_CONNECTION_OPENS.with(|opens| opens.set(opens.get().saturating_add(1)));
    let conn = Connection::open_with_flags(
        &paths.db_path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_URI,
    )?;
    conn.busy_timeout(std::time::Duration::from_secs(5))?;
    Ok(conn)
}

#[cfg(test)]
pub(crate) fn reset_capability_connection_open_count() {
    CAPABILITY_CONNECTION_OPENS.with(|opens| opens.set(0));
}

#[cfg(test)]
pub(crate) fn capability_connection_open_count() -> usize {
    CAPABILITY_CONNECTION_OPENS.with(std::cell::Cell::get)
}

pub(crate) fn read_schema_version(conn: &Connection) -> rusqlite::Result<Option<i64>> {
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

pub(crate) fn table_exists(conn: &Connection, table: &str) -> rusqlite::Result<bool> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type IN ('table', 'view') AND name = ?1",
        [table],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

pub(crate) fn column_exists(
    conn: &Connection,
    table: &str,
    column: &str,
) -> rusqlite::Result<bool> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_breakdown_requires_schema_14() {
        assert_eq!(
            min_schema_version(FeatureKey::ProviderBreakdown),
            PROVIDER_BREAKDOWN_SCHEMA_VERSION
        );
    }

    #[test]
    fn non_provider_features_keep_min_supported_schema() {
        assert_eq!(
            min_schema_version(FeatureKey::Overview),
            MIN_SUPPORTED_SCHEMA_VERSION
        );
        assert_eq!(
            min_schema_version(FeatureKey::Logs),
            MIN_SUPPORTED_SCHEMA_VERSION
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
    fn required_logs_columns_include_raw_join_keys() {
        let cols = required_columns(FeatureKey::Logs);
        assert!(
            cols.iter()
                .any(|(table, cols)| *table == "usage_event" && cols.contains(&"event_key"))
        );
    }

    #[test]
    fn schema_gate_rejects_old_schema() {
        let conn = rusqlite::Connection::open_in_memory().expect("in-memory db should open");
        conn.execute_batch("CREATE TABLE meta(key TEXT PRIMARY KEY, value TEXT NOT NULL); INSERT INTO meta(key, value) VALUES ('schema_version', '9');").expect("meta fixture should be created");
        assert_eq!(
            read_schema_version(&conn).expect("schema version should read"),
            Some(9)
        );
    }

    #[test]
    fn provider_breakdown_capability_requires_schema_14() {
        let conn = rusqlite::Connection::open_in_memory().expect("in-memory db should open");
        conn.execute_batch(
            "CREATE TABLE meta(key TEXT PRIMARY KEY, value TEXT NOT NULL);
             INSERT INTO meta(key, value) VALUES ('schema_version', '13');",
        )
        .expect("meta fixture should be created");
        let mut features = std::collections::BTreeMap::new();

        populate_db_features(&conn, Some(13), &mut features);

        let capability = features
            .get("provider_breakdown")
            .expect("provider_breakdown key should exist");
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
    fn provider_breakdown_reports_missing_provider_label_column() {
        let conn = rusqlite::Connection::open_in_memory().expect("in-memory db should open");
        conn.execute_batch(
            r#"
            CREATE TABLE meta(key TEXT PRIMARY KEY, value TEXT NOT NULL);
            INSERT INTO meta(key, value) VALUES ('schema_version', '14');
            CREATE TABLE usage_bucket_30m(total_tokens INTEGER NOT NULL);
            CREATE TABLE usage_event(event_key TEXT NOT NULL);
            "#,
        )
        .expect("schema 14 fixture should be created");
        let mut features = std::collections::BTreeMap::new();

        populate_db_features(&conn, Some(14), &mut features);

        let capability = features
            .get("provider_breakdown")
            .expect("provider_breakdown key should exist");
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

    #[test]
    fn db_capabilities_mark_missing_db_for_all_db_backed_features() {
        let temp = tempfile::TempDir::new().expect("temp dir should be created");
        let caps = DbCapabilities::detect(&AppPaths::from_root(temp.path()));

        assert!(!caps.db_exists);
        assert!(!caps.db_readable);
        assert_eq!(caps.schema_version, None);
        assert_eq!(caps.features.len(), DB_BACKED_FEATURES.len());
        for key in DB_BACKED_FEATURES {
            let capability = caps.features.get(key.as_str()).expect("feature present");
            assert!(!capability.supported);
            assert_eq!(
                capability.reason.as_ref(),
                Some(&UnsupportedReason::DbMissing)
            );
        }
    }

    #[test]
    fn db_capabilities_mark_schema_read_failure_as_unreadable() {
        let temp = tempfile::TempDir::new().expect("temp dir should be created");
        let paths = AppPaths::from_root(temp.path());
        let conn = Connection::open(&paths.db_path).expect("fixture db should open");
        conn.execute_batch(
            "CREATE TABLE meta(key TEXT PRIMARY KEY, value TEXT NOT NULL);
             INSERT INTO meta(key, value) VALUES ('schema_version', X'80');",
        )
        .expect("malformed schema value should be created");
        drop(conn);

        let caps = DbCapabilities::detect(&paths);

        assert!(caps.db_exists);
        assert!(!caps.db_readable);
        assert_eq!(caps.schema_version, None);
        assert_eq!(caps.features.len(), DB_BACKED_FEATURES.len());
        for key in DB_BACKED_FEATURES {
            let capability = caps.features.get(key.as_str()).expect("feature present");
            assert!(!capability.supported);
            assert_eq!(
                capability.reason.as_ref(),
                Some(&UnsupportedReason::DbUnreadable)
            );
        }
    }
}
