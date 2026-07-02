use chrono::NaiveDate;
use rusqlite::{Connection, OpenFlags, ToSql, params_from_iter};

use crate::{
    AppPaths, FeatureKey, UsageError,
    capabilities::{MIN_SUPPORTED_SCHEMA_VERSION, ensure_feature, read_schema_version},
    queries::ProviderBreakdownDto,
    source::SourceKind,
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ReportTimezone {
    #[default]
    Local,
    Utc,
}

#[derive(Debug, Clone, Default)]
pub struct QueryFilter {
    pub source: Option<SourceKind>,
    pub provider: Option<String>,
    pub since: Option<NaiveDate>,
    pub until: Option<NaiveDate>,
    pub timezone: ReportTimezone,
}

#[derive(Debug, Clone)]
struct SqlFilter {
    clauses: Vec<String>,
    params: Vec<String>,
}

impl SqlFilter {
    fn new() -> Self {
        Self {
            clauses: Vec::new(),
            params: Vec::new(),
        }
    }

    fn push(&mut self, clause: impl Into<String>, value: impl Into<String>) {
        self.clauses.push(clause.into());
        self.params.push(value.into());
    }

    fn where_sql(&self) -> String {
        if self.clauses.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", self.clauses.join(" AND "))
        }
    }

    fn param_refs(&self) -> Vec<&dyn ToSql> {
        self.params
            .iter()
            .map(|value| value as &dyn ToSql)
            .collect()
    }
}

impl QueryFilter {
    fn local_time_modifier(&self) -> &'static str {
        match self.timezone {
            ReportTimezone::Local => "localtime",
            ReportTimezone::Utc => "utc",
        }
    }

    fn bucket_filter(&self, alias: Option<&str>) -> SqlFilter {
        let prefix = alias.map(|value| format!("{value}.")).unwrap_or_default();
        let mut filter = SqlFilter::new();
        if let Some(source) = self.source {
            filter.push(format!("{prefix}source = ?"), source.as_str());
        }
        if let Some(provider) = self.provider.as_ref() {
            filter.push(format!("{prefix}provider_label = ?"), provider);
        }
        if let Some(since) = self.since {
            filter.push(
                format!(
                    "date({prefix}hour_start, '{}') >= ?",
                    self.local_time_modifier()
                ),
                since.to_string(),
            );
        }
        if let Some(until) = self.until {
            filter.push(
                format!(
                    "date({prefix}hour_start, '{}') <= ?",
                    self.local_time_modifier()
                ),
                until.to_string(),
            );
        }
        filter
    }
}

#[derive(Debug)]
pub struct Dashboard {
    paths: AppPaths,
    conn: Connection,
}

pub fn open_dashboard(paths: AppPaths) -> Result<Dashboard, UsageError> {
    Dashboard::open(paths)
}

impl Dashboard {
    pub fn open(paths: AppPaths) -> Result<Self, UsageError> {
        if !paths.db_path.is_file() {
            return Err(UsageError::DbMissing(paths.db_path.clone()));
        }
        let conn = Connection::open_with_flags(
            &paths.db_path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_URI,
        )
        .map_err(|error| UsageError::DbUnreadable(error.to_string()))?;
        conn.busy_timeout(std::time::Duration::from_secs(5))?;
        let schema_version = read_schema_version(&conn)?;
        if schema_version.unwrap_or_default() < MIN_SUPPORTED_SCHEMA_VERSION {
            return Err(UsageError::SchemaUnsupported {
                expected: MIN_SUPPORTED_SCHEMA_VERSION,
                actual: schema_version,
            });
        }
        Ok(Self { paths, conn })
    }

    pub fn provider_breakdown(
        &self,
        filter: &QueryFilter,
    ) -> Result<Vec<ProviderBreakdownDto>, UsageError> {
        ensure_feature(&self.paths, FeatureKey::ProviderBreakdown)?;
        let sql_filter = filter.bucket_filter(None);
        let sql = format!(
            r#"
            SELECT
                provider_label,
                COALESCE(SUM(event_count), 0),
                COALESCE(SUM(input_tokens), 0),
                COALESCE(SUM(cache_read_tokens), 0),
                COALESCE(SUM(cache_creation_tokens), 0),
                COALESCE(SUM(output_tokens), 0),
                COALESCE(SUM(reasoning_output_tokens), 0),
                COALESCE(SUM(total_tokens), 0),
                COALESCE(SUM(cost_with_cache_usd), 0.0),
                COALESCE(SUM(cost_without_cache_usd), 0.0)
            FROM usage_bucket_30m
            {}
            GROUP BY provider_label
            ORDER BY SUM(total_tokens) DESC, provider_label ASC
        "#,
            sql_filter.where_sql()
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params_from_iter(sql_filter.param_refs()), |row| {
            Ok(ProviderBreakdownDto {
                provider: non_empty(row.get(0)?),
                request_count: row.get(1)?,
                input_tokens: row.get(2)?,
                cache_read_tokens: row.get(3)?,
                cache_creation_tokens: row.get(4)?,
                output_tokens: row.get(5)?,
                reasoning_output_tokens: row.get(6)?,
                total_tokens: row.get(7)?,
                cost_with_cache_usd: row.get(8)?,
                cost_without_cache_usd: row.get(9)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }
}

fn non_empty(value: String) -> Option<String> {
    let value = value.trim().to_string();
    (!value.is_empty()).then_some(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn create_provider_fixture(root: &std::path::Path) -> Dashboard {
        let db_path = root.join("llmusage.db");
        let conn = Connection::open(&db_path).expect("test db should open");
        conn.execute_batch(
            r#"
            CREATE TABLE meta(key TEXT PRIMARY KEY, value TEXT NOT NULL);
            INSERT INTO meta(key, value) VALUES ('schema_version', '14');
            CREATE TABLE usage_event(provider_label TEXT NOT NULL DEFAULT '');
            CREATE TABLE usage_bucket_30m(
                source TEXT NOT NULL,
                provider_label TEXT NOT NULL DEFAULT '',
                model TEXT NOT NULL,
                hour_start TEXT NOT NULL,
                project_hash TEXT NOT NULL DEFAULT '',
                input_tokens INTEGER NOT NULL,
                cache_read_tokens INTEGER NOT NULL,
                cache_creation_tokens INTEGER NOT NULL,
                output_tokens INTEGER NOT NULL,
                reasoning_output_tokens INTEGER NOT NULL,
                total_tokens INTEGER NOT NULL,
                event_count INTEGER NOT NULL,
                cost_with_cache_usd REAL NOT NULL,
                cost_without_cache_usd REAL NOT NULL
            );
            INSERT INTO usage_bucket_30m VALUES
              ('codex', 'openai', 'gpt-5', '2026-07-01T00:00:00Z', 'p1', 40, 10, 5, 30, 15, 100, 2, 0.10, 0.15),
              ('codex', 'openai', 'gpt-4', '2026-07-01T00:30:00Z', 'p1', 10, 0, 0, 5, 5, 20, 1, 0.02, 0.03),
              ('claude', 'anthropic', 'claude-sonnet', '2026-07-01T01:00:00Z', 'p2', 20, 0, 0, 25, 5, 50, 1, 0.20, 0.25),
              ('codex', '', 'gpt-5', '2026-07-01T01:30:00Z', 'p3', 10, 0, 0, 10, 10, 30, 1, 0.03, 0.04);
            "#,
        )
        .expect("provider fixture should be created");
        drop(conn);

        Dashboard::open(AppPaths::from_root(root)).expect("dashboard should open test db")
    }

    #[test]
    fn provider_breakdown_sums_rows_and_marks_unattributed() {
        let temp = tempfile::TempDir::new().expect("temp dir should be created");
        let dashboard = create_provider_fixture(temp.path());

        let provider_stats = dashboard
            .provider_breakdown(&QueryFilter::default())
            .expect("provider breakdown should query");

        assert_eq!(
            provider_stats
                .iter()
                .map(|row| row.total_tokens)
                .sum::<i64>(),
            200
        );
        let openai = provider_stats
            .iter()
            .find(|row| row.provider.as_deref() == Some("openai"))
            .expect("openai row should exist");
        assert_eq!(openai.total_tokens, 120);
        assert_eq!(openai.request_count, 3);
        let unattributed = provider_stats
            .iter()
            .find(|row| row.provider.is_none())
            .expect("empty provider label should map to unattributed");
        assert_eq!(unattributed.total_tokens, 30);
    }

    #[test]
    fn provider_breakdown_respects_source_filter() {
        let temp = tempfile::TempDir::new().expect("temp dir should be created");
        let dashboard = create_provider_fixture(temp.path());

        let stats = dashboard
            .provider_breakdown(&QueryFilter {
                source: Some(SourceKind::Claude),
                ..QueryFilter::default()
            })
            .expect("provider breakdown should query");

        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].provider.as_deref(), Some("anthropic"));
        assert_eq!(stats[0].total_tokens, 50);
    }

    #[test]
    fn provider_filter_can_match_unattributed_rows() {
        let temp = tempfile::TempDir::new().expect("temp dir should be created");
        let dashboard = create_provider_fixture(temp.path());

        let stats = dashboard
            .provider_breakdown(&QueryFilter {
                provider: Some(String::new()),
                ..QueryFilter::default()
            })
            .expect("empty provider filter should query unattributed rows");

        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].provider, None);
        assert_eq!(stats[0].total_tokens, 30);
    }

    #[test]
    fn provider_breakdown_rejects_schema_below_14() {
        let temp = tempfile::TempDir::new().expect("temp dir should be created");
        let db_path = temp.path().join("llmusage.db");
        let conn = Connection::open(&db_path).expect("test db should open");
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
            UsageError::SchemaUnsupported {
                expected: 14,
                actual: Some(13)
            }
        ));
    }
}
