//! Home Insights State-free 服务层。
//!
//! 合并两条数据链路（边界说明见 `commands::usage` 模块文档）：
//! - usage：`ccr_usage::Dashboard::insights` 投影（请求数、token、时间分桶、榜单）；
//! - session：`ccr_db` 的 `usage_session_archive` 会话计数。
//!
//! 本模块不写 SQL，不读系统时钟：`as_of` 与 `generated_at` 由命令层传入。

use std::collections::BTreeMap;

use ccr_db::database::repositories::usage_repo::{self, SessionArchivePlatformSummary};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::llmusage_adapter::queries::{
    InsightsDay, InsightsPayload, InsightsSourceTally, InsightsTally, InsightsTrendSeries,
};
use crate::llmusage_adapter::{LlmusageRuntime, QueryFilter, SourceKind};

/// All-history totals for the Insights header row.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, TS)]
#[ts(export, export_to = "../../src/types/generated/usage/")]
pub struct InsightsTotals {
    #[ts(as = "f64")]
    pub requests: i64,
    #[ts(as = "f64")]
    pub tokens: i64,
    #[ts(as = "f64")]
    pub sessions: i64,
    /// Sources with any requests or tokens.
    #[ts(as = "f64")]
    pub agents: i64,
    /// Distinct non-empty project hashes.
    #[ts(as = "f64")]
    pub projects: i64,
    /// Local days with at least one request.
    #[ts(as = "f64")]
    pub active_days: i64,
}

/// Activity inside one 7-day local-date window.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, TS)]
#[ts(export, export_to = "../../src/types/generated/usage/")]
pub struct InsightsWindow {
    #[ts(as = "f64")]
    pub requests: i64,
    #[ts(as = "f64")]
    pub sessions: i64,
    #[ts(as = "f64")]
    pub active_days: i64,
}

/// Agent ranking row. `source` is the canonical llmusage source id, or the
/// raw stored source / session-archive platform name when `unmapped` is true.
/// Usage and session data for one key share one row.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, TS)]
#[ts(export, export_to = "../../src/types/generated/usage/")]
pub struct InsightsAgentTally {
    pub source: String,
    #[ts(as = "f64")]
    pub requests: i64,
    #[ts(as = "f64")]
    pub tokens: i64,
    #[ts(as = "f64")]
    pub sessions: i64,
    pub unmapped: bool,
}

/// Single-call snapshot for the home Insights section.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, TS)]
#[ts(export, export_to = "../../src/types/generated/usage/")]
pub struct HomeInsightsResponse {
    /// Snapshot local date, `YYYY-MM-DD`.
    pub as_of: String,
    /// Earliest local date with usage.
    pub first_date: Option<String>,
    pub totals: InsightsTotals,
    /// False when the session archive is empty; the UI shows `–`, not 0.
    pub sessions_indexed: bool,
    pub last_7_days: InsightsWindow,
    pub previous_7_days: InsightsWindow,
    /// Days with requests inside the 53-week window, ascending.
    pub daily: Vec<InsightsDay>,
    pub current_streak: u32,
    pub longest_streak: u32,
    pub busiest_day: Option<InsightsDay>,
    /// Requests per local hour (24 items).
    #[ts(as = "Vec<f64>")]
    pub hourly: Vec<i64>,
    /// Requests per weekday, Monday first (7 items).
    #[ts(as = "Vec<f64>")]
    pub weekday: Vec<i64>,
    /// Requests per calendar month, January first (12 items).
    #[ts(as = "Vec<f64>")]
    pub monthly: Vec<i64>,
    /// Monday of the first week in the 53-week trend window.
    pub trend_start: String,
    pub trend: Vec<InsightsTrendSeries>,
    pub agents: Vec<InsightsAgentTally>,
    pub projects: Vec<InsightsTally>,
    pub models: Vec<InsightsTally>,
    pub generated_at: String,
}

/// 会话归档计数（不上 wire）。
#[derive(Debug, Clone, Default)]
struct InsightsSessionCounts {
    indexed: bool,
    by_platform: Vec<SessionArchivePlatformSummary>,
    last_7_days: i64,
    previous_7_days: i64,
}

/// Home Insights 快照计算（State-free，同步；命令层负责 spawn_blocking、计时与缓存）。
pub fn compute_home_insights(
    llmusage: &LlmusageRuntime,
    pool: &ccr_db::database::DbPool,
    as_of: NaiveDate,
    generated_at: DateTime<Utc>,
) -> Result<HomeInsightsResponse, String> {
    let dashboard = llmusage
        .dashboard()
        .map_err(|e| format!("Dashboard open error: {e}"))?;
    let usage = dashboard
        .insights(&QueryFilter::default(), as_of)
        .map_err(|e| format!("Insights query error: {e}"))?;
    let sessions = load_session_counts(pool, &usage)?;
    Ok(merge_home_insights(usage, sessions, generated_at))
}

fn load_session_counts(
    pool: &ccr_db::database::DbPool,
    usage: &InsightsPayload,
) -> Result<InsightsSessionCounts, String> {
    let conn = pool.get().map_err(|e| format!("DB error: {e}"))?;
    let indexed = usage_repo::has_any_session_archive(&conn)
        .map_err(|e| format!("Session archive presence query error: {e}"))?;
    if !indexed {
        return Ok(InsightsSessionCounts::default());
    }
    let by_platform = usage_repo::count_session_archive_by_platform(&conn)
        .map_err(|e| format!("Session archive count query error: {e}"))?;
    // 7 天窗口的 UTC 半开边界来自投影，与请求数使用同一报告时区。
    let window_sessions = |start: DateTime<Utc>, end: DateTime<Utc>| {
        usage_repo::count_session_archive_by_platform_between(&conn, start, end)
            .map(|rows| rows.iter().map(|row| row.session_count.max(0)).sum::<i64>())
            .map_err(|e| format!("Session archive window query error: {e}"))
    };
    let last_7_days = window_sessions(usage.last_7_days.start_utc, usage.last_7_days.end_utc)?;
    let previous_7_days = window_sessions(
        usage.previous_7_days.start_utc,
        usage.previous_7_days.end_utc,
    )?;
    Ok(InsightsSessionCounts {
        indexed,
        by_platform,
        last_7_days,
        previous_7_days,
    })
}

fn merge_home_insights(
    usage: InsightsPayload,
    sessions: InsightsSessionCounts,
    generated_at: DateTime<Utc>,
) -> HomeInsightsResponse {
    let total_sessions = sessions
        .by_platform
        .iter()
        .map(|row| row.session_count.max(0))
        .sum();
    let agents = merge_agent_tallies(&usage.sources, &sessions.by_platform);
    HomeInsightsResponse {
        as_of: format_date(usage.as_of),
        first_date: usage.first_date.map(format_date),
        totals: InsightsTotals {
            requests: usage.requests,
            tokens: usage.tokens,
            sessions: total_sessions,
            agents: usage.agents,
            projects: usage.project_count,
            active_days: usage.active_days,
        },
        sessions_indexed: sessions.indexed,
        last_7_days: InsightsWindow {
            requests: usage.last_7_days.requests,
            sessions: sessions.last_7_days,
            active_days: usage.last_7_days.active_days,
        },
        previous_7_days: InsightsWindow {
            requests: usage.previous_7_days.requests,
            sessions: sessions.previous_7_days,
            active_days: usage.previous_7_days.active_days,
        },
        daily: usage.daily,
        current_streak: usage.current_streak,
        longest_streak: usage.longest_streak,
        busiest_day: usage.busiest_day,
        hourly: usage.hourly,
        weekday: usage.weekday,
        monthly: usage.monthly,
        trend_start: format_date(usage.trend_start),
        trend: usage.trend,
        agents,
        projects: usage.projects,
        models: usage.models,
        generated_at: generated_at.to_rfc3339(),
    }
}

/// 用量来源与会话平台按同一键合并为一行：requests/tokens 取用量数据，sessions 取会话数据。
/// 归档平台名经 `SourceKind::parse_id` 映射（`omp`→`pi`、`kimi`→`kimi_code`）；
/// 映射失败的键（用量侧投影已保留原标签）保留原名，`unmapped = true`。
fn merge_agent_tallies(
    sources: &[InsightsSourceTally],
    sessions: &[SessionArchivePlatformSummary],
) -> Vec<InsightsAgentTally> {
    fn row_for(
        rows: &mut BTreeMap<String, InsightsAgentTally>,
        key: String,
    ) -> &mut InsightsAgentTally {
        rows.entry(key.clone())
            .or_insert_with(|| InsightsAgentTally {
                unmapped: SourceKind::parse_id(&key).is_none(),
                source: key,
                ..InsightsAgentTally::default()
            })
    }
    let mut rows = BTreeMap::<String, InsightsAgentTally>::new();
    for source in sources {
        let row = row_for(&mut rows, source.source.clone());
        row.requests += source.requests;
        row.tokens += source.tokens;
    }
    for platform in sessions {
        let key = SourceKind::parse_id(&platform.platform)
            .map(|kind| kind.as_str().to_string())
            .unwrap_or_else(|| platform.platform.clone());
        let row = row_for(&mut rows, key);
        row.sessions += platform.session_count.max(0);
    }
    let mut agents = rows.into_values().collect::<Vec<_>>();
    // 前端按当前度量重排；这里给出确定顺序：请求数、会话数降序，再按键升序。
    agents.sort_by(|left, right| {
        right
            .requests
            .cmp(&left.requests)
            .then_with(|| right.sessions.cmp(&left.sessions))
            .then_with(|| left.source.cmp(&right.source))
    });
    agents
}

fn format_date(date: NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ccr_db::database::repositories::usage_repo::{
        UsageSessionArchiveEntry, UsageSourceState, agent_session_archive_id,
        upsert_session_archive_entry,
    };
    use ccr_usage::fixtures::{SeedBucket, create_projection_db, seed_bucket};
    use chrono::{Duration, Local, NaiveTime};
    use rusqlite::Connection;
    use tempfile::TempDir;

    fn as_of() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 9, 24).expect("fixture date should be valid")
    }

    fn generated_at() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-09-24T12:00:00Z")
            .expect("fixture timestamp should parse")
            .with_timezone(&Utc)
    }

    /// 本机时区下某本地日某时刻对应的 UTC 时刻（Dashboard 同样用本机 IANA 时区）。
    fn local_instant(date: NaiveDate, hour: u32, minute: u32) -> DateTime<Utc> {
        date.and_time(
            NaiveTime::from_hms_opt(hour, minute, 0).expect("fixture time should be valid"),
        )
        .and_local_timezone(Local)
        .earliest()
        .expect("local fixture time should exist")
        .with_timezone(&Utc)
    }

    fn temp_usage_pool(temp: &TempDir) -> ccr_db::database::DbPool {
        let pool = ccr_db::database::create_pool(&temp.path().join("usage.db"), None)
            .expect("usage pool should be created");
        let conn = pool.get().expect("pool should hand out a connection");
        ccr_db::database::migrations::run_all_migrations(&conn, temp.path())
            .expect("migrations should run");
        drop(conn);
        pool
    }

    fn open_runtime(temp: &TempDir, seeds: &[SeedBucket]) -> LlmusageRuntime {
        let paths = create_projection_db(&temp.path().join("llmusage"));
        let conn = Connection::open(&paths.db_path).expect("fixture db should reopen");
        for seed in seeds {
            seed_bucket(&conn, seed);
        }
        drop(conn);
        LlmusageRuntime::from_paths(paths)
    }

    fn seed_session(
        pool: &ccr_db::database::DbPool,
        platform: &str,
        file: &str,
        created_at: DateTime<Utc>,
    ) {
        let entry = UsageSessionArchiveEntry {
            archive_id: agent_session_archive_id(platform, file, ""),
            session_id: file.to_string(),
            platform: platform.to_string(),
            title: None,
            cwd: String::new(),
            file_path: file.to_string(),
            file_hash: None,
            source_variant: "fixture".to_string(),
            source_kind: "file".to_string(),
            source_member_id: String::new(),
            source_size: None,
            source_mtime_ns: None,
            source_stat_hash: None,
            message_count: 1,
            user_message_count: 1,
            assistant_message_count: 0,
            tool_use_count: 0,
            source_fidelity: "full".to_string(),
            created_at,
            updated_at: created_at,
            source_state: UsageSourceState::Live,
            last_seen_at: Some(created_at),
            raw_deleted_at: None,
            archived_at: created_at,
        };
        let conn = pool.get().expect("pool should hand out a connection");
        upsert_session_archive_entry(&conn, &entry).expect("session entry should insert");
    }

    fn bucket(source: &str, date: NaiveDate, event_count: i64) -> SeedBucket {
        SeedBucket {
            source: source.to_string(),
            hour_start: local_instant(date, 12, 0).to_rfc3339(),
            event_count,
            ..SeedBucket::default()
        }
    }

    #[test]
    fn home_insights_maps_session_platforms_and_keeps_unknown_rows() {
        let temp = TempDir::new().expect("temp dir should be created");
        let runtime = open_runtime(
            &temp,
            &[bucket("pi", as_of(), 4), bucket("kimi_code", as_of(), 2)],
        );
        let pool = temp_usage_pool(&temp);
        let noon = local_instant(as_of(), 12, 0);
        seed_session(&pool, "omp", "/s/omp-1.jsonl", noon);
        seed_session(&pool, "omp", "/s/omp-2.jsonl", noon);
        seed_session(&pool, "kimi", "/s/kimi-1.jsonl", noon);
        seed_session(&pool, "mystery-cli", "/s/mystery-1.jsonl", noon);

        let response = compute_home_insights(&runtime, &pool, as_of(), generated_at())
            .expect("insights should compute");

        assert!(response.sessions_indexed);
        assert_eq!(response.totals.sessions, 4);
        assert_eq!(response.totals.requests, 6);
        assert_eq!(response.totals.agents, 2);
        let find = |source: &str| {
            response
                .agents
                .iter()
                .find(|row| row.source == source)
                .cloned()
                .expect("agent row should exist")
        };
        let pi = find("pi");
        assert_eq!((pi.requests, pi.sessions, pi.unmapped), (4, 2, false));
        let kimi = find("kimi_code");
        assert_eq!((kimi.requests, kimi.sessions, kimi.unmapped), (2, 1, false));
        let mystery = find("mystery-cli");
        assert_eq!(
            (mystery.requests, mystery.sessions, mystery.unmapped),
            (0, 1, true)
        );
        assert!(response.agents.iter().all(|row| row.source != "omp"));
        assert!(response.agents.iter().all(|row| row.source != "kimi"));
        assert_eq!(response.as_of, "2026-09-24");
        assert_eq!(response.generated_at, generated_at().to_rfc3339());
    }

    #[test]
    fn home_insights_merges_usage_and_session_rows_for_one_unmapped_key() {
        let temp = TempDir::new().expect("temp dir should be created");
        let runtime = open_runtime(
            &temp,
            &[
                bucket("mystery-cli", as_of(), 5),
                bucket("codex", as_of(), 1),
            ],
        );
        let pool = temp_usage_pool(&temp);
        let noon = local_instant(as_of(), 12, 0);
        seed_session(&pool, "mystery-cli", "/s/mystery-1.jsonl", noon);
        seed_session(&pool, "mystery-cli", "/s/mystery-2.jsonl", noon);
        seed_session(&pool, "mystery-cli", "/s/mystery-3.jsonl", noon);

        let response = compute_home_insights(&runtime, &pool, as_of(), generated_at())
            .expect("insights should compute");

        let rows = response
            .agents
            .iter()
            .filter(|row| row.source == "mystery-cli")
            .collect::<Vec<_>>();
        assert_eq!(rows.len(), 1);
        assert_eq!(
            (rows[0].requests, rows[0].sessions, rows[0].unmapped),
            (5, 3, true)
        );
        assert_eq!(response.agents.len(), 2);
        assert_eq!(response.totals.agents, 2);
        assert_eq!(response.totals.requests, 6);
        assert_eq!(response.totals.sessions, 3);
    }

    #[test]
    fn home_insights_last_7_days_sessions_include_as_of_day() {
        let temp = TempDir::new().expect("temp dir should be created");
        let runtime = open_runtime(&temp, &[bucket("codex", as_of(), 3)]);
        let pool = temp_usage_pool(&temp);
        // as_of 当天本地 23:30：西半球时区下 UTC 日期已是次日。
        seed_session(
            &pool,
            "codex",
            "/s/late.jsonl",
            local_instant(as_of(), 23, 30),
        );
        seed_session(
            &pool,
            "codex",
            "/s/first.jsonl",
            local_instant(as_of() - Duration::days(6), 0, 0),
        );
        seed_session(
            &pool,
            "claude",
            "/s/prev.jsonl",
            local_instant(as_of() - Duration::days(7), 12, 0),
        );
        seed_session(
            &pool,
            "claude",
            "/s/next.jsonl",
            local_instant(as_of() + Duration::days(1), 0, 0),
        );

        let response = compute_home_insights(&runtime, &pool, as_of(), generated_at())
            .expect("insights should compute");

        assert_eq!(response.last_7_days.sessions, 2);
        assert_eq!(response.previous_7_days.sessions, 1);
        assert_eq!(response.totals.sessions, 4);
        assert_eq!(response.last_7_days.requests, 3);
        assert_eq!(response.last_7_days.active_days, 1);
    }

    #[test]
    fn home_insights_reports_unindexed_sessions_honestly() {
        let temp = TempDir::new().expect("temp dir should be created");
        let runtime = open_runtime(&temp, &[bucket("codex", as_of(), 3)]);
        let pool = temp_usage_pool(&temp);

        let response = compute_home_insights(&runtime, &pool, as_of(), generated_at())
            .expect("insights should compute");

        assert!(!response.sessions_indexed);
        assert_eq!(response.totals.sessions, 0);
        assert_eq!(response.hourly.len(), 24);
        assert_eq!(response.weekday.len(), 7);
        assert_eq!(response.monthly.len(), 12);
        assert_eq!(
            response.trend[0].weekly.len(),
            crate::llmusage_adapter::queries::INSIGHTS_WEEKS
        );
        assert_eq!(response.trend_start, "2025-09-22");
    }

    #[test]
    fn home_insights_response_cache_value_round_trips() {
        let temp = TempDir::new().expect("temp dir should be created");
        let runtime = open_runtime(&temp, &[bucket("codex", as_of(), 3)]);
        let pool = temp_usage_pool(&temp);

        let response = compute_home_insights(&runtime, &pool, as_of(), generated_at())
            .expect("insights should compute");

        // 命令层 cache_set / from_value 路径：Value 往返后与原值相等。
        let cached = serde_json::to_value(&response).expect("response should serialize");
        let decoded: HomeInsightsResponse =
            serde_json::from_value(cached).expect("cached value should decode");
        assert_eq!(decoded, response);
    }

    #[test]
    fn home_insights_surfaces_missing_llmusage_db_as_error() {
        let temp = TempDir::new().expect("temp dir should be created");
        let runtime = LlmusageRuntime::from_paths(crate::llmusage_adapter::AppPaths::from_root(
            temp.path().join("absent"),
        ));
        let pool = temp_usage_pool(&temp);

        let error = compute_home_insights(&runtime, &pool, as_of(), generated_at())
            .expect_err("missing llmusage DB should be an error");

        assert!(error.contains("Dashboard open error"));
    }
}
