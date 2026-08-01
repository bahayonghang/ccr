//! Test-only llmusage read-only projection fixtures.
//!
//! This module builds a throwaway `llmusage.db` whose schema covers every
//! table required by the projection queries in this crate: `usage_bucket_30m`,
//! `usage_event`, `usage_event_raw`, `run_log`, `source_file`,
//! `source_sync_status` and `meta`. Column sets follow the real SQL in
//! `src/db.rs` and the capability gates in `src/capabilities.rs` (schema
//! version 19, including the current upstream range indexes).
//!
//! Consumers: this crate's own tests and `ccr-ui/src-tauri` service tests via
//! the `test-fixtures` feature. Helpers panic on failure by design (test-only
//! ergonomics).

use std::path::Path;

use rusqlite::{Connection, params};

use crate::AppPaths;

/// Creates `llmusage.db` under `root` with the full projection schema and
/// returns the matching [`AppPaths`]. The database starts empty; use the
/// `seed_*` helpers to insert rows.
pub fn create_projection_db(root: &Path) -> AppPaths {
    std::fs::create_dir_all(root).expect("fixture root dir should be creatable");
    let paths = AppPaths::from_root(root);
    let conn = Connection::open(&paths.db_path).expect("fixture db should open");
    conn.execute_batch(
        r#"
        CREATE TABLE meta(key TEXT PRIMARY KEY, value TEXT NOT NULL);
        INSERT INTO meta(key, value) VALUES ('schema_version', '19');
        CREATE TABLE usage_bucket_30m(
            source TEXT NOT NULL,
            provider_label TEXT NOT NULL DEFAULT '',
            model TEXT NOT NULL,
            hour_start TEXT NOT NULL,
            project_hash TEXT NOT NULL DEFAULT '',
            project_label TEXT NOT NULL DEFAULT '',
            project_ref TEXT,
            input_tokens INTEGER NOT NULL,
            cache_read_tokens INTEGER NOT NULL,
            cache_creation_tokens INTEGER NOT NULL,
            output_tokens INTEGER NOT NULL,
            reasoning_output_tokens INTEGER NOT NULL,
            total_tokens INTEGER NOT NULL,
            event_count INTEGER NOT NULL,
            cost_with_cache_usd REAL NOT NULL,
            cost_without_cache_usd REAL NOT NULL,
            pricing_status TEXT NOT NULL,
            pricing_source TEXT,
            pricing_rate TEXT
        );
        CREATE INDEX idx_usage_bucket_30m_hour_start ON usage_bucket_30m(hour_start);
        CREATE TABLE usage_event(
            event_key TEXT PRIMARY KEY,
            source TEXT NOT NULL,
            provider_label TEXT NOT NULL DEFAULT '',
            model TEXT NOT NULL,
            event_at TEXT NOT NULL,
            input_tokens INTEGER NOT NULL,
            cache_read_tokens INTEGER NOT NULL,
            cache_creation_tokens INTEGER NOT NULL,
            output_tokens INTEGER NOT NULL,
            reasoning_output_tokens INTEGER NOT NULL,
            total_tokens INTEGER NOT NULL,
            project_hash TEXT NOT NULL DEFAULT '',
            project_label TEXT NOT NULL DEFAULT '',
            project_ref TEXT,
            project_path TEXT,
            cost_with_cache_usd REAL NOT NULL,
            cost_without_cache_usd REAL NOT NULL,
            pricing_status TEXT NOT NULL,
            pricing_source TEXT
        );
        CREATE INDEX idx_usage_event_event_at ON usage_event(event_at);
        CREATE INDEX idx_usage_event_activity_cost
            ON usage_event(event_at, source, model, cost_with_cache_usd);
        CREATE TABLE usage_event_raw(
            event_key TEXT PRIMARY KEY,
            raw_json TEXT NOT NULL
        );
        CREATE TABLE run_log(
            command TEXT NOT NULL,
            status TEXT NOT NULL,
            finished_at TEXT NOT NULL
        );
        CREATE TABLE source_file(source TEXT NOT NULL, state TEXT NOT NULL);
        CREATE TABLE source_sync_status(
            source TEXT PRIMARY KEY,
            recent_completed_at TEXT,
            history_completed_at TEXT
        );
        "#,
    )
    .expect("fixture schema should be created");
    drop(conn);
    paths
}

/// One `usage_bucket_30m` row. `Default` yields a plausible non-zero codex
/// bucket (token components sum to `total_tokens`).
#[derive(Debug, Clone)]
pub struct SeedBucket {
    pub source: String,
    pub provider_label: String,
    pub model: String,
    pub hour_start: String,
    pub project_hash: String,
    pub project_label: String,
    pub project_ref: Option<String>,
    pub input_tokens: i64,
    pub cache_read_tokens: i64,
    pub cache_creation_tokens: i64,
    pub output_tokens: i64,
    pub reasoning_output_tokens: i64,
    pub total_tokens: i64,
    pub event_count: i64,
    pub cost_with_cache_usd: f64,
    pub cost_without_cache_usd: f64,
    pub pricing_status: String,
    pub pricing_source: Option<String>,
    pub pricing_rate: Option<String>,
}

impl Default for SeedBucket {
    fn default() -> Self {
        Self {
            source: "codex".to_string(),
            provider_label: "openai".to_string(),
            model: "gpt-5".to_string(),
            // 正午 UTC：date(hour_start, 'localtime') 在 ±11h 时区内保持同一天，
            // 避免测试机时区导致本地日期漂移。
            hour_start: "2026-07-01T12:00:00Z".to_string(),
            project_hash: "p1".to_string(),
            project_label: "Project 1".to_string(),
            project_ref: Some("/repo/p1".to_string()),
            input_tokens: 40,
            cache_read_tokens: 10,
            cache_creation_tokens: 5,
            output_tokens: 30,
            reasoning_output_tokens: 15,
            total_tokens: 100,
            event_count: 2,
            cost_with_cache_usd: 0.10,
            cost_without_cache_usd: 0.15,
            pricing_status: "priced".to_string(),
            pricing_source: Some("catalog".to_string()),
            pricing_rate: Some("rate-a".to_string()),
        }
    }
}

/// Inserts one `usage_bucket_30m` row.
pub fn seed_bucket(conn: &Connection, seed: &SeedBucket) {
    conn.execute(
        r#"
        INSERT INTO usage_bucket_30m(
            source, provider_label, model, hour_start,
            project_hash, project_label, project_ref,
            input_tokens, cache_read_tokens, cache_creation_tokens,
            output_tokens, reasoning_output_tokens, total_tokens, event_count,
            cost_with_cache_usd, cost_without_cache_usd,
            pricing_status, pricing_source, pricing_rate
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)
        "#,
        params![
            seed.source,
            seed.provider_label,
            seed.model,
            seed.hour_start,
            seed.project_hash,
            seed.project_label,
            seed.project_ref,
            seed.input_tokens,
            seed.cache_read_tokens,
            seed.cache_creation_tokens,
            seed.output_tokens,
            seed.reasoning_output_tokens,
            seed.total_tokens,
            seed.event_count,
            seed.cost_with_cache_usd,
            seed.cost_without_cache_usd,
            seed.pricing_status,
            seed.pricing_source,
            seed.pricing_rate,
        ],
    )
    .expect("bucket row should insert");
}

/// One `usage_event` row plus its paired `usage_event_raw` row (the logs
/// query left-joins raw JSON by `event_key`). `Default` yields a plausible
/// non-zero codex event.
#[derive(Debug, Clone)]
pub struct SeedEvent {
    pub event_key: String,
    pub source: String,
    pub provider_label: String,
    pub model: String,
    pub event_at: String,
    pub input_tokens: i64,
    pub cache_read_tokens: i64,
    pub cache_creation_tokens: i64,
    pub output_tokens: i64,
    pub reasoning_output_tokens: i64,
    pub total_tokens: i64,
    pub project_hash: String,
    pub project_label: String,
    pub project_ref: Option<String>,
    pub project_path: Option<String>,
    pub cost_with_cache_usd: f64,
    pub cost_without_cache_usd: f64,
    pub pricing_status: String,
    pub pricing_source: Option<String>,
    pub raw_json: String,
}

impl Default for SeedEvent {
    fn default() -> Self {
        Self {
            event_key: "ev-1".to_string(),
            source: "codex".to_string(),
            provider_label: "openai".to_string(),
            model: "gpt-5".to_string(),
            event_at: "2026-07-01T12:00:00Z".to_string(),
            input_tokens: 10,
            cache_read_tokens: 2,
            cache_creation_tokens: 1,
            output_tokens: 5,
            reasoning_output_tokens: 1,
            total_tokens: 19,
            project_hash: "p1".to_string(),
            project_label: "Project 1".to_string(),
            project_ref: None,
            project_path: Some("/repo/p1".to_string()),
            cost_with_cache_usd: 0.03,
            cost_without_cache_usd: 0.04,
            pricing_status: "priced".to_string(),
            pricing_source: Some("catalog".to_string()),
            raw_json: r#"{"kind":"fixture"}"#.to_string(),
        }
    }
}

/// Inserts one `usage_event` row and its `usage_event_raw` counterpart.
pub fn seed_event(conn: &Connection, seed: &SeedEvent) {
    conn.execute(
        r#"
        INSERT INTO usage_event(
            event_key, source, provider_label, model, event_at,
            input_tokens, cache_read_tokens, cache_creation_tokens,
            output_tokens, reasoning_output_tokens, total_tokens,
            project_hash, project_label, project_ref, project_path,
            cost_with_cache_usd, cost_without_cache_usd,
            pricing_status, pricing_source
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)
        "#,
        params![
            seed.event_key,
            seed.source,
            seed.provider_label,
            seed.model,
            seed.event_at,
            seed.input_tokens,
            seed.cache_read_tokens,
            seed.cache_creation_tokens,
            seed.output_tokens,
            seed.reasoning_output_tokens,
            seed.total_tokens,
            seed.project_hash,
            seed.project_label,
            seed.project_ref,
            seed.project_path,
            seed.cost_with_cache_usd,
            seed.cost_without_cache_usd,
            seed.pricing_status,
            seed.pricing_source,
        ],
    )
    .expect("event row should insert");
    conn.execute(
        "INSERT INTO usage_event_raw(event_key, raw_json) VALUES (?1, ?2)",
        params![seed.event_key, seed.raw_json],
    )
    .expect("raw event row should insert");
}

/// Records one completed sync run: appends a successful `run_log` entry
/// (overview freshness fields) and upserts `source_sync_status` for `source`.
/// `kind` selects the column, matching the diagnostics SQL: `"recent"` sets
/// `recent_completed_at`, `"history"` sets `history_completed_at`.
pub fn seed_run_log(conn: &Connection, source: &str, kind: &str, completed_at: &str) {
    let column = match kind {
        "recent" => "recent_completed_at",
        "history" => "history_completed_at",
        other => panic!("unsupported sync kind '{other}', expected 'recent' or 'history'"),
    };
    conn.execute(
        "INSERT INTO run_log(command, status, finished_at) VALUES ('sync', 'success', ?1)",
        params![completed_at],
    )
    .expect("run_log row should insert");
    conn.execute(
        "INSERT OR IGNORE INTO source_sync_status(source) VALUES (?1)",
        params![source],
    )
    .expect("source_sync_status row should upsert");
    // column 来自上方白名单 match，无注入风险。
    conn.execute(
        &format!("UPDATE source_sync_status SET {column} = ?2 WHERE source = ?1"),
        params![source, completed_at],
    )
    .expect("source_sync_status column should update");
}

/// Inserts one `source_file` row (diagnostics aggregates per-source file
/// states: `live` / `missing` / `deleted_by_user`).
pub fn seed_source_file(conn: &Connection, source: &str, state: &str) {
    conn.execute(
        "INSERT INTO source_file(source, state) VALUES (?1, ?2)",
        params![source, state],
    )
    .expect("source_file row should insert");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        DB_BACKED_FEATURES, Dashboard, DbCapabilities, LogsQuery, QueryFilter, SourceKind,
    };

    #[test]
    fn fixture_schema_supports_every_db_backed_feature_and_overview_query() {
        let temp = tempfile::TempDir::new().expect("temp dir should be created");
        let paths = create_projection_db(temp.path());
        let conn = Connection::open(&paths.db_path).expect("fixture db should reopen");
        seed_bucket(&conn, &SeedBucket::default());
        seed_run_log(&conn, "codex", "recent", "2026-07-01T10:00:00Z");
        seed_run_log(&conn, "codex", "history", "2026-06-30T00:00:00Z");
        drop(conn);

        // 能力探测覆盖全部 DB-backed feature 的建表/列门槛（含 provider schema 14）。
        let caps = DbCapabilities::detect(&paths);
        assert_eq!(caps.schema_version, Some(19));
        for key in DB_BACKED_FEATURES {
            let capability = caps.features.get(key.as_str()).expect("feature present");
            assert!(
                capability.supported,
                "feature `{}` should be supported by the fixture schema: {:?}",
                key.as_str(),
                capability
            );
        }

        let dashboard = Dashboard::open(paths).expect("dashboard should open fixture db");
        let overview = dashboard
            .overview(&QueryFilter::default())
            .expect("overview should query");
        assert_eq!(overview.total.total_tokens, 100);
        assert_eq!(overview.total_events, 2);
        // run_log 表列与 overview 真实 SQL 对齐：MAX(finished_at) 命中 recent 那条。
        assert_eq!(
            overview.last_sync_at.as_deref(),
            Some("2026-07-01T10:00:00Z")
        );
    }

    #[test]
    fn schema_18_19_and_future_20_keep_compatible_projection_support() {
        for schema_version in [18_i64, 19, 20] {
            let temp = tempfile::TempDir::new().expect("temp dir should be created");
            let paths = create_projection_db(temp.path());
            let conn = Connection::open(&paths.db_path).expect("fixture db should reopen");
            conn.execute(
                "UPDATE meta SET value = ?1 WHERE key = 'schema_version'",
                [schema_version.to_string()],
            )
            .expect("schema version should update");
            seed_bucket(&conn, &SeedBucket::default());
            drop(conn);

            let caps = DbCapabilities::detect(&paths);
            assert_eq!(caps.schema_version, Some(schema_version));
            assert!(
                caps.features
                    .get("overview")
                    .expect("overview capability present")
                    .supported,
                "schema {schema_version} should retain compatible overview support"
            );

            let overview = Dashboard::open(paths)
                .expect("compatible schema should open")
                .overview(&QueryFilter::default())
                .expect("overview should query");
            assert_eq!(overview.total.total_tokens, 100, "schema {schema_version}");
        }
    }

    #[test]
    fn fixture_supports_logs_join_and_diagnostics_queries() {
        let temp = tempfile::TempDir::new().expect("temp dir should be created");
        let paths = create_projection_db(temp.path());
        let conn = Connection::open(&paths.db_path).expect("fixture db should reopen");
        let event = SeedEvent::default();
        seed_event(&conn, &event);
        seed_source_file(&conn, "codex", "live");
        seed_source_file(&conn, "codex", "missing");
        seed_run_log(&conn, "codex", "recent", "2026-07-01T10:00:00Z");
        drop(conn);

        let dashboard = Dashboard::open(paths).expect("dashboard should open fixture db");
        let page = dashboard
            .logs(&LogsQuery {
                filter: QueryFilter {
                    source: Some(SourceKind::Codex),
                    ..QueryFilter::default()
                },
                page_size: 10,
                cursor: None,
                include_total: true,
                include_raw_json: true,
            })
            .expect("logs should query");
        assert_eq!(page.records.len(), 1);
        assert_eq!(page.total, Some(1));
        assert_eq!(page.records[0].id, event.event_key);
        // usage_event_raw join 生效：record_json 即种子 raw_json。
        assert_eq!(page.records[0].record_json, event.raw_json);

        let diagnostics = dashboard.diagnostics().expect("diagnostics should query");
        assert_eq!(diagnostics.by_source.len(), 1);
        let codex = &diagnostics.by_source[0];
        assert_eq!(codex.source, "codex");
        assert_eq!(codex.live_files, 1);
        assert_eq!(codex.missing_files, 1);
        assert_eq!(
            codex.recent_completed_at.as_deref(),
            Some("2026-07-01T10:00:00Z")
        );
        assert_eq!(codex.history_completed_at, None);
    }
}
