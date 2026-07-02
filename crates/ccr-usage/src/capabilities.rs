use rusqlite::{Connection, OpenFlags};
use serde::Serialize;

use crate::{AppPaths, UsageError};

pub const MIN_SUPPORTED_SCHEMA_VERSION: i64 = 10;
pub const PROVIDER_BREAKDOWN_SCHEMA_VERSION: i64 = 14;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum FeatureKey {
    ProviderBreakdown,
}

impl FeatureKey {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ProviderBreakdown => "provider_breakdown",
        }
    }
}

pub(crate) fn ensure_feature(paths: &AppPaths, feature: FeatureKey) -> Result<(), UsageError> {
    if !paths.db_path.is_file() {
        return Err(UsageError::DbMissing(paths.db_path.clone()));
    }
    let conn = open_readonly_for_capabilities(paths).map_err(|error| {
        UsageError::DbUnreadable(format!("{}: {error}", paths.db_path.display()))
    })?;
    let schema_version = read_schema_version(&conn)?;
    let expected_schema_version = min_schema_version(feature);
    if schema_version.unwrap_or_default() < expected_schema_version {
        return Err(UsageError::SchemaUnsupported {
            expected: expected_schema_version,
            actual: schema_version,
        });
    }
    for (table, columns) in required_columns(feature) {
        if !table_exists(&conn, table)? {
            return Err(UsageError::FeatureUnavailable {
                feature: feature.as_str(),
                reason: format!("missing required table `{table}`"),
            });
        }
        for column in columns {
            if !column_exists(&conn, table, column)? {
                return Err(UsageError::FeatureUnavailable {
                    feature: feature.as_str(),
                    reason: format!("missing required column `{table}.{column}`"),
                });
            }
        }
    }
    Ok(())
}

fn min_schema_version(feature: FeatureKey) -> i64 {
    match feature {
        FeatureKey::ProviderBreakdown => PROVIDER_BREAKDOWN_SCHEMA_VERSION,
    }
}

pub(crate) fn required_columns(feature: FeatureKey) -> Vec<(&'static str, Vec<&'static str>)> {
    match feature {
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
    }
}

pub(crate) fn open_readonly_for_capabilities(paths: &AppPaths) -> rusqlite::Result<Connection> {
    let conn = Connection::open_with_flags(
        &paths.db_path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_URI,
    )?;
    conn.busy_timeout(std::time::Duration::from_secs(5))?;
    Ok(conn)
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
}
