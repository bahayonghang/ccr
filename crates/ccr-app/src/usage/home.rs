//! Shared Home Insights policy. Projection SQL remains in ccr-usage and archive SQL in ccr-db.
//! No process environment, default path discovery, clock reads or UI framework dependency.
use ccr_db::database::repositories::usage_repo::{self, SessionArchivePlatformSummary};
use ccr_usage::{
    InsightsDay, InsightsPayload, InsightsSourceTally, InsightsTally, InsightsTrendSeries,
    SourceKind,
};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// All-history totals for the Insights header row.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct InsightsTotals {
    pub requests: i64,
    pub tokens: i64,
    pub sessions: i64,
    /// Sources with any requests or tokens.
    pub agents: i64,
    /// Distinct non-empty project hashes.
    pub projects: i64,
    /// Local days with at least one request.
    pub active_days: i64,
}

/// Activity inside one 7-day local-date window.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct InsightsWindow {
    pub requests: i64,
    pub sessions: i64,
    pub active_days: i64,
}

/// Agent ranking row. `source` is the canonical llmusage source id, or the
/// raw stored source / session-archive platform name when `unmapped` is true.
/// Usage and session data for one key share one row.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct InsightsAgentTally {
    pub source: String,
    pub requests: i64,
    pub tokens: i64,
    pub sessions: i64,
    pub unmapped: bool,
}

/// Single-call snapshot for the home Insights section.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HomeInsightsSnapshot {
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
    pub hourly: Vec<i64>,
    /// Requests per weekday, Monday first (7 items).
    pub weekday: Vec<i64>,
    /// Requests per calendar month, January first (12 items).
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
pub struct InsightsSessionCounts {
    pub indexed: bool,
    pub by_platform: Vec<SessionArchivePlatformSummary>,
    pub last_7_days: i64,
    pub previous_7_days: i64,
}

/// Read the existing usage projection and join archive counts using its exact
/// reporting windows. Adapters supply paths, reporting date and capture time.
pub fn read_home_insights(
    paths: ccr_usage::AppPaths,
    filter: &ccr_usage::QueryFilter,
    pool: &ccr_db::database::DbPool,
    as_of: NaiveDate,
    generated_at: DateTime<Utc>,
) -> Result<HomeInsightsSnapshot, String> {
    let dashboard = ccr_usage::open_dashboard(paths)
        .map_err(|error| format!("Dashboard open error: {error}"))?;
    let usage = dashboard
        .insights(filter, as_of)
        .map_err(|error| format!("Insights query error: {error}"))?;
    assemble_home_insights(usage, pool, generated_at)
}

/// Assemble the existing usage projection with archive counts from the same report windows.
/// Callers own path/time resolution and blocking execution; this function adds no usage SQL.
pub fn assemble_home_insights(
    usage: InsightsPayload,
    pool: &ccr_db::database::DbPool,
    generated_at: DateTime<Utc>,
) -> Result<HomeInsightsSnapshot, String> {
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

pub fn merge_home_insights(
    usage: InsightsPayload,
    sessions: InsightsSessionCounts,
    generated_at: DateTime<Utc>,
) -> HomeInsightsSnapshot {
    let total_sessions = sessions
        .by_platform
        .iter()
        .map(|row| row.session_count.max(0))
        .sum();
    let agents = merge_agent_tallies(&usage.sources, &sessions.by_platform);
    HomeInsightsSnapshot {
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
#[path = "home_tests.rs"]
mod tests;
