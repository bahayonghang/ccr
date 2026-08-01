//! Usage V2 State-free 服务层。
//!
//! 从 `commands::usage` 平移的 wire DTO、纯投影函数与同步查询编排：命令层只保留
//! 计时 / State 提取 / spawn_blocking / 缓存，业务编排在此以具名 DTO 出入，
//! 便于无 Tauri app 的单元测试（usage/session 双链路边界见 `commands::usage` 模块文档）。

use std::collections::BTreeMap;
use std::time::Instant;

use ccr_config::Platform;
use ccr_store::sessions::parser::SessionParser;
use chrono::{DateTime, Duration, Local, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::llmusage_adapter::error::LlmusageAdapterError;
use crate::llmusage_adapter::queries::{
    DailyTrendDto, HeatmapResponseDto, ModelStatDto, PaginatedLogsDto, ProjectStatDto,
    ProviderBreakdownDto, SourceBreakdownDto, UsageSummaryDto,
};
use crate::llmusage_adapter::{
    JobEvent, LlmusageRuntime, LogsQuery as LlmusageLogsQuery, SourceKind, SourceSyncStats,
    build_filter, is_optional_source_absent, platform_scope_label, queries,
};
use crate::session_index_jobs::SessionIndexJobSnapshot;
use crate::usage_jobs::UsageImportJobSnapshot;

pub(crate) const HOME_USAGE_PLATFORMS: [&str; 7] = [
    "claude",
    "codex",
    "opencode",
    "antigravity",
    "kimi_code",
    "pi",
    "grok",
];
pub(crate) const USAGE_SNAPSHOT_CACHE_PREFIX: &str = "usage:snapshot:";
pub(crate) const USAGE_SNAPSHOT_CACHE_TTL_SECS: u64 = 30;
pub(crate) const USAGE_FRESHNESS_STALE_AFTER_SECS: i64 = 24 * 60 * 60;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export, export_to = "../../src/types/generated/usage/")]
pub enum UsageLogsMode {
    #[default]
    Cursor,
    Offset,
}

#[derive(Debug, Clone, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "../../src/types/generated/usage/")]
pub struct UsageLogsQuery {
    // 输入类型：缺键与 None 等价，optional 精确表达 wire 契约（生成 `field?: T`）。
    #[ts(optional)]
    pub platform: Option<String>,
    #[ts(optional)]
    pub model: Option<String>,
    #[serde(alias = "startDate")]
    #[ts(optional)]
    pub start_date: Option<String>,
    #[serde(alias = "endDate")]
    #[ts(optional)]
    pub end_date: Option<String>,
    // i64/u64 走 serde_json number，ts(as = "f64") 避免 ts-rs 默认 bigint
    #[allow(dead_code)]
    #[ts(as = "Option<f64>", optional)]
    pub page: Option<i64>,
    #[serde(alias = "pageSize")]
    #[ts(as = "Option<f64>", optional)]
    pub page_size: Option<i64>,
    #[ts(optional)]
    pub cursor: Option<String>,
    #[serde(alias = "includeTotal")]
    #[ts(optional)]
    pub include_total: Option<bool>,
    #[allow(dead_code)]
    #[ts(optional)]
    pub mode: Option<UsageLogsMode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/usage/")]
pub struct UsageImportResultV2 {
    pub platform: String,
    pub files_processed: usize,
    pub records_imported: usize,
    pub records_skipped: usize,
    #[ts(as = "f64")]
    pub duration_ms: u64,
    pub completed: bool,
    pub error: Option<String>,
    /// optional source（如 OpenCode）缺失安装时为 true。
    /// 前端依此判断"导入失败"是不是用户没装这个 optional source 的正常情况，
    /// 不要嗅探 `error` 字符串。
    #[serde(default)]
    pub is_optional_absent: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
#[ts(export, export_to = "../../src/types/generated/usage/")]
pub struct UsageImportSummary {
    pub success_count: usize,
    pub failure_count: usize,
    pub imported_records: usize,
    pub processed_files: usize,
    pub has_partial: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/usage/")]
pub struct ImportAllUsageResponse {
    pub results: Vec<UsageImportResultV2>,
    pub summary: UsageImportSummary,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "../../src/types/generated/usage/")]
pub enum UsageFreshnessState {
    Fresh,
    Stale,
    #[default]
    Missing,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, TS)]
#[ts(export, export_to = "../../src/types/generated/usage/")]
pub struct UsageFreshnessProjection {
    pub state: UsageFreshnessState,
    pub latest_completed_at: Option<String>,
    #[ts(as = "Option<f64>")]
    pub age_seconds: Option<i64>,
    #[ts(as = "f64")]
    pub stale_after_seconds: i64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "../../src/types/generated/usage/")]
pub enum UsageSourceHealthState {
    Live,
    Degraded,
    #[default]
    Missing,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, TS)]
#[ts(export, export_to = "../../src/types/generated/usage/")]
pub struct UsageSourceHealth {
    pub source: String,
    pub state: UsageSourceHealthState,
    #[ts(as = "f64")]
    pub live_sources: u64,
    #[ts(as = "f64")]
    pub missing_sources: u64,
    #[ts(as = "f64")]
    pub deleted_sources: u64,
    pub recent_completed_at: Option<String>,
    pub history_completed_at: Option<String>,
    pub freshness: UsageFreshnessProjection,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "../../src/types/generated/usage/")]
pub enum UsageReadinessState {
    Ready,
    Syncing,
    Stale,
    Degraded,
    NeedsImport,
    NeedsSessionIndex,
    #[default]
    Empty,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, TS)]
#[ts(export, export_to = "../../src/types/generated/usage/")]
pub struct UsageReadinessProjection {
    pub state: UsageReadinessState,
    pub next_action: Option<String>,
    pub detail: String,
    pub has_live_sources: bool,
    pub has_missing_sources: bool,
    pub has_deleted_sources: bool,
    pub active_usage_import: bool,
    pub active_session_index: bool,
    pub recent_completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, TS)]
#[ts(export, export_to = "../../src/types/generated/usage/")]
pub struct UsageDrilldownProjection {
    pub dimensions: Vec<String>,
    pub supports_logs: bool,
    pub supports_projects: bool,
    pub supports_sessions: bool,
}

impl Default for UsageDrilldownProjection {
    fn default() -> Self {
        Self {
            dimensions: vec![
                "source".to_string(),
                "project_path".to_string(),
                "model".to_string(),
                "session_id".to_string(),
                "cwd".to_string(),
                "branch".to_string(),
                "worktree".to_string(),
            ],
            supports_logs: true,
            supports_projects: true,
            supports_sessions: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, TS)]
#[ts(export, export_to = "../../src/types/generated/usage/")]
pub struct UsageSnapshotProjection {
    pub generated_at: String,
    pub platform_scope: String,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    #[ts(as = "f64")]
    pub cache_ttl_seconds: u64,
    pub freshness: UsageFreshnessProjection,
    pub readiness: UsageReadinessProjection,
    pub source_health: Vec<UsageSourceHealth>,
    pub drilldown: UsageDrilldownProjection,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/usage/")]
pub struct UsageArchiveDiagnostics {
    pub archive_root: String,
    #[ts(as = "f64")]
    pub live_sources: u64,
    #[ts(as = "f64")]
    pub missing_sources: u64,
    #[ts(as = "f64")]
    pub deleted_sources: u64,
    #[ts(as = "f64")]
    pub archived_sessions: u64,
    pub recent_completed_at: Option<String>,
    pub history_completed_at: Option<String>,
    #[serde(default)]
    pub source_health: Vec<UsageSourceHealth>,
    #[serde(default)]
    pub freshness: UsageFreshnessProjection,
    #[serde(default)]
    pub readiness: UsageReadinessProjection,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/usage/")]
pub struct StartUsageImportJobResponse {
    pub job_id: String,
    pub snapshot: UsageImportJobSnapshot,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/usage/")]
pub struct StartSessionIndexJobResponse {
    pub job_id: String,
    pub snapshot: SessionIndexJobSnapshot,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, TS)]
#[ts(export, export_to = "../../src/types/generated/usage/")]
pub struct HomeOverviewPlatformStats {
    #[ts(as = "f64")]
    pub sessions: u64,
    #[ts(as = "f64")]
    pub requests: u64,
    #[ts(as = "f64")]
    pub tokens: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, TS)]
#[ts(export, export_to = "../../src/types/generated/usage/")]
pub struct HomeOverviewSummary {
    #[ts(as = "f64")]
    pub total_sessions: u64,
    #[ts(as = "f64")]
    pub total_requests: u64,
    #[ts(as = "f64")]
    pub total_tokens: u64,
    #[ts(as = "f64")]
    pub active_days: u64,
    #[ts(as = "f64")]
    pub platforms: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, TS)]
#[ts(export, export_to = "../../src/types/generated/usage/")]
pub struct HomeOverviewBootstrap {
    pub usage_import_attempted: bool,
    pub usage_imported_records: usize,
    pub session_reindex_attempted: bool,
    #[ts(as = "f64")]
    pub indexed_sessions: u64,
    pub usage_job_id: Option<String>,
    pub session_job_id: Option<String>,
    pub needs_usage_import: bool,
    pub needs_session_index: bool,
    pub is_warm: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, TS)]
#[ts(export, export_to = "../../src/types/generated/usage/")]
pub struct HomeOverviewSeriesItem {
    pub date: String,
    pub claude: HomeOverviewPlatformStats,
    pub codex: HomeOverviewPlatformStats,
    pub antigravity: HomeOverviewPlatformStats,
    pub opencode: HomeOverviewPlatformStats,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/usage/")]
pub struct HomeUsageOverviewResponse {
    pub summary: HomeOverviewSummary,
    pub by_platform: BTreeMap<String, HomeOverviewPlatformStats>,
    pub series: Vec<HomeOverviewSeriesItem>,
    pub bootstrap: HomeOverviewBootstrap,
    pub archive: UsageArchiveDiagnostics,
    pub snapshot: UsageSnapshotProjection,
    pub empty_reason: Option<String>,
    pub last_updated: String,
}

/// dashboard V2 聚合响应：字段顺序与既有 `json!()` 拼装 wire 契约一致。
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/usage/")]
pub struct UsageDashboardResponse {
    pub summary: UsageSummaryDto,
    pub trends: Vec<DailyTrendDto>,
    pub model_stats: Vec<ModelStatDto>,
    pub project_stats: Vec<ProjectStatDto>,
    pub source_stats: Vec<SourceBreakdownDto>,
    pub provider_stats: Vec<ProviderBreakdownDto>,
    pub archive: UsageArchiveDiagnostics,
    pub snapshot: UsageSnapshotProjection,
    pub heatmap: Option<HeatmapResponseDto>,
    pub generated_at: String,
}

pub(crate) struct HomeUsageSnapshot {
    total_requests: u64,
    total_tokens: u64,
    active_days: u64,
    by_platform: BTreeMap<String, HomeOverviewPlatformStats>,
    daily_by_platform: BTreeMap<String, BTreeMap<String, HomeOverviewPlatformStats>>,
}

pub(crate) struct HomeSessionSnapshot {
    total_sessions: u64,
    has_any_sessions: bool,
    by_platform: BTreeMap<String, HomeOverviewPlatformStats>,
    daily_by_platform: BTreeMap<String, BTreeMap<String, HomeOverviewPlatformStats>>,
}

/// home overview 命令层收集的活跃 job 上下文（不上 wire，不导出 TS）。
pub struct HomeJobContext {
    pub active_usage_job_id: Option<String>,
    pub active_usage_imported_records: usize,
    pub active_session_job_id: Option<String>,
    pub active_session_indexed: u64,
    pub active_usage_import: bool,
    pub active_session_index: bool,
    /// 原始 session 目录是否存在文件（命令层在 spawn_blocking 内经
    /// `has_any_raw_sessions()` 计算后传入，service 保持测试封闭）。
    pub raw_sessions_present: bool,
}

pub fn elapsed_ms(started: Instant) -> f64 {
    started.elapsed().as_secs_f64() * 1000.0
}

pub fn usage_freshness_from_completed_at(
    latest_completed_at: Option<String>,
    now: DateTime<Utc>,
) -> UsageFreshnessProjection {
    let age_seconds = latest_completed_at
        .as_deref()
        .and_then(|value| DateTime::parse_from_rfc3339(value).ok())
        .map(|completed_at| {
            now.signed_duration_since(completed_at.with_timezone(&Utc))
                .num_seconds()
                .max(0)
        });
    let state = match age_seconds {
        Some(age) if age <= USAGE_FRESHNESS_STALE_AFTER_SECS => UsageFreshnessState::Fresh,
        Some(_) => UsageFreshnessState::Stale,
        None => UsageFreshnessState::Missing,
    };

    UsageFreshnessProjection {
        state,
        latest_completed_at,
        age_seconds,
        stale_after_seconds: USAGE_FRESHNESS_STALE_AFTER_SECS,
    }
}

pub fn usage_source_health_state(
    live_sources: u64,
    missing_sources: u64,
    deleted_sources: u64,
    freshness: UsageFreshnessState,
) -> UsageSourceHealthState {
    if live_sources == 0 && missing_sources == 0 && deleted_sources == 0 {
        return UsageSourceHealthState::Missing;
    }

    if missing_sources > 0 || deleted_sources > 0 || freshness != UsageFreshnessState::Fresh {
        UsageSourceHealthState::Degraded
    } else {
        UsageSourceHealthState::Live
    }
}

pub fn build_usage_readiness(
    archive: &UsageArchiveDiagnostics,
    total_requests: u64,
    total_sessions: u64,
    needs_session_index: bool,
    active_usage_import: bool,
    active_session_index: bool,
) -> UsageReadinessProjection {
    let has_live_sources = archive.live_sources > 0 || total_requests > 0 || total_sessions > 0;
    let has_missing_sources = archive.missing_sources > 0;
    let has_deleted_sources = archive.deleted_sources > 0;
    let (state, next_action, detail) = if active_usage_import || active_session_index {
        (
            UsageReadinessState::Syncing,
            None,
            "Usage import or session indexing is running.".to_string(),
        )
    } else if total_requests == 0 && archive.live_sources == 0 {
        (
            UsageReadinessState::NeedsImport,
            Some("import_usage".to_string()),
            "No imported usage source is available yet.".to_string(),
        )
    } else if needs_session_index {
        (
            UsageReadinessState::NeedsSessionIndex,
            Some("index_sessions".to_string()),
            "Raw sessions exist but the session archive is not indexed.".to_string(),
        )
    } else if archive.freshness.state == UsageFreshnessState::Stale {
        (
            UsageReadinessState::Stale,
            Some("refresh_usage".to_string()),
            "Imported usage data is older than the freshness window.".to_string(),
        )
    } else if has_missing_sources || has_deleted_sources {
        (
            UsageReadinessState::Degraded,
            Some("inspect_sources".to_string()),
            "Some usage sources are missing or deleted.".to_string(),
        )
    } else if total_requests == 0 && total_sessions == 0 {
        (
            UsageReadinessState::Empty,
            Some("import_usage".to_string()),
            "Usage sources are reachable but this window has no records.".to_string(),
        )
    } else {
        (
            UsageReadinessState::Ready,
            None,
            "Usage read model is ready.".to_string(),
        )
    };

    UsageReadinessProjection {
        state,
        next_action,
        detail,
        has_live_sources,
        has_missing_sources,
        has_deleted_sources,
        active_usage_import,
        active_session_index,
        recent_completed_at: archive.recent_completed_at.clone(),
    }
}

pub fn build_usage_snapshot_projection(
    platform_scope: String,
    start_date: Option<String>,
    end_date: Option<String>,
    archive: &UsageArchiveDiagnostics,
) -> UsageSnapshotProjection {
    UsageSnapshotProjection {
        generated_at: Utc::now().to_rfc3339(),
        platform_scope,
        start_date,
        end_date,
        cache_ttl_seconds: USAGE_SNAPSHOT_CACHE_TTL_SECS,
        freshness: archive.freshness.clone(),
        readiness: archive.readiness.clone(),
        source_health: archive.source_health.clone(),
        drilldown: UsageDrilldownProjection::default(),
    }
}

pub fn source_import_result(source: SourceKind, stats: &SourceSyncStats) -> UsageImportResultV2 {
    let ignored_absent_source = is_optional_source_absent(stats);

    UsageImportResultV2 {
        platform: source.as_str().to_string(),
        files_processed: stats.files_processed,
        records_imported: stats.events_inserted,
        records_skipped: stats.events_seen.saturating_sub(stats.events_inserted),
        duration_ms: stats
            .parse_ms
            .saturating_add(stats.write_ms)
            .saturating_add(stats.lock_wait_ms),
        completed: ignored_absent_source || stats.last_error.is_none(),
        error: if ignored_absent_source {
            None
        } else {
            stats.last_error.clone()
        },
        is_optional_absent: ignored_absent_source,
    }
}

pub fn build_import_summary(results: &[UsageImportResultV2]) -> UsageImportSummary {
    let success_count = results
        .iter()
        .filter(|result| result.error.is_none())
        .count();
    let failure_count = results.len().saturating_sub(success_count);
    let imported_records = results.iter().map(|result| result.records_imported).sum();
    let processed_files = results.iter().map(|result| result.files_processed).sum();
    let has_partial = results.iter().any(|result| {
        result.error.is_some()
            || !result.completed
            || (result.files_processed > 0
                && result.records_imported == 0
                && result.records_skipped > 0)
    });

    UsageImportSummary {
        success_count,
        failure_count,
        imported_records,
        processed_files,
        has_partial,
    }
}

pub fn default_import_results(platform: Option<&str>) -> Vec<UsageImportResultV2> {
    platform
        .map(|platform| vec![platform.to_string()])
        .unwrap_or_else(|| {
            HOME_USAGE_PLATFORMS
                .iter()
                .map(|platform| (*platform).to_string())
                .collect()
        })
        .into_iter()
        .map(|platform| UsageImportResultV2 {
            platform,
            files_processed: 0,
            records_imported: 0,
            records_skipped: 0,
            duration_ms: 0,
            completed: true,
            error: None,
            is_optional_absent: false,
        })
        .collect()
}

pub fn collect_llmusage_sync_results(
    events: Vec<JobEvent>,
    fallback_source: Option<&str>,
) -> Result<Vec<UsageImportResultV2>, String> {
    let mut results_by_platform = BTreeMap::<String, UsageImportResultV2>::new();

    for event in events {
        match event {
            JobEvent::SourceFinished { source, stats } => {
                results_by_platform.insert(
                    source.as_str().to_string(),
                    source_import_result(source, &stats),
                );
            }
            JobEvent::Finished { summary } => {
                let mut results = if results_by_platform.is_empty() {
                    default_import_results(fallback_source)
                } else {
                    results_by_platform.into_values().collect()
                };
                let imported_records = results
                    .iter()
                    .map(|result| result.records_imported)
                    .sum::<usize>();
                if summary.total_inserted > imported_records
                    && let Some(first) = results.first_mut()
                {
                    first.records_imported = first.records_imported.max(summary.total_inserted);
                    first.records_skipped = first
                        .records_skipped
                        .max(summary.total_seen.saturating_sub(summary.total_inserted));
                }
                results.sort_by(|left, right| left.platform.cmp(&right.platform));
                return Ok(results);
            }
            JobEvent::Failed { error } => return Err(error),
            JobEvent::Cancelled => return Err("Usage import job was cancelled".to_string()),
            _ => {}
        }
    }

    Err("Usage import job ended before a terminal event was emitted".to_string())
}

pub fn empty_home_platform_map() -> BTreeMap<String, HomeOverviewPlatformStats> {
    let mut map = BTreeMap::new();
    for platform in HOME_USAGE_PLATFORMS {
        map.insert(platform.to_string(), HomeOverviewPlatformStats::default());
    }
    map
}

#[cfg(test)]
pub fn normalize_home_platform(raw: &str) -> Option<&'static str> {
    match raw.trim().to_lowercase().as_str() {
        "claude" | "claude-code" | "claude code" => Some("claude"),
        "codex" | "openai-codex" | "openai codex" => Some("codex"),
        "antigravity"
        | "gemini"
        | "gemini-cli"
        | "gemini cli"
        | "google-gemini"
        | "google gemini" => {
            Some("antigravity")
        }
        "opencode" | "open-code" | "open code" => Some("opencode"),
        "kimi_code" | "kimi-code" | "kimi code" => Some("kimi_code"),
        "pi" | "oh-my-pi" | "oh my pi" | "omp" => Some("pi"),
        "grok" | "grok-build" | "grok build" => Some("grok"),
        _ => None,
    }
}

pub fn non_negative_i64(value: i64) -> u64 {
    value.max(0) as u64
}

pub fn local_usage_date_window(days: usize) -> (chrono::NaiveDate, chrono::NaiveDate) {
    let safe_days = days.max(1);
    let end = Local::now().date_naive();
    let start = end - Duration::days((safe_days - 1) as i64);
    (start, end)
}

pub fn format_local_usage_date(date: chrono::NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}

pub fn build_home_date_range_from(start: chrono::NaiveDate, days: usize) -> Vec<String> {
    let safe_days = days.max(1);
    (0..safe_days)
        .map(|offset| format_local_usage_date(start + Duration::days(offset as i64)))
        .collect()
}

pub fn detect_home_empty_reason(
    total_requests: u64,
    total_sessions: u64,
    has_any_usage: bool,
    has_any_sessions: bool,
) -> Option<String> {
    if total_requests == 0 && !has_any_usage && total_sessions == 0 && !has_any_sessions {
        return Some("no_usage_and_sessions".to_string());
    }
    if total_requests == 0 && !has_any_usage {
        return Some("no_usage_logs".to_string());
    }
    if total_sessions == 0 && !has_any_sessions {
        return Some("no_session_index".to_string());
    }

    None
}

/// 探测 ~/.claude 等真实目录是否存在原始 session 文件（同步 FS 扫描）。
/// 仅供命令层在 spawn_blocking 内调用；service 计算函数只接收布尔结果，
/// 保持无真实用户目录依赖、可单测。
pub(crate) fn has_any_raw_sessions() -> bool {
    for platform in [Platform::Claude, Platform::Codex, Platform::Gemini] {
        let Some(session_dir) = SessionParser::get_platform_session_dir(&platform) else {
            continue;
        };

        match SessionParser::scan_directory(&session_dir, platform) {
            Ok(files) if !files.is_empty() => return true,
            Ok(_) => continue,
            Err(error) => {
                tracing::debug!(?platform, ?error, "Failed to inspect raw session files");
            }
        }
    }

    false
}

pub fn load_home_usage_snapshot(
    dashboard: &crate::llmusage_adapter::Dashboard,
    filter: &crate::llmusage_adapter::QueryFilter,
) -> Result<HomeUsageSnapshot, String> {
    let payload = dashboard
        .home_overview(filter)
        .map_err(|e| format!("Home usage overview query error: {e}"))?;
    let mut by_platform = empty_home_platform_map();
    for (platform, stats) in payload.by_platform {
        if let Some(target) = by_platform.get_mut(platform.as_str()) {
            target.requests = non_negative_i64(stats.requests);
            target.tokens = non_negative_i64(stats.tokens);
        }
    }

    let mut daily_by_platform = BTreeMap::new();
    for item in payload.series {
        let mut day_stats = empty_home_platform_map();
        if let Some(stats) = day_stats.get_mut("claude") {
            stats.requests = non_negative_i64(item.claude.requests);
            stats.tokens = non_negative_i64(item.claude.tokens);
        }
        if let Some(stats) = day_stats.get_mut("codex") {
            stats.requests = non_negative_i64(item.codex.requests);
            stats.tokens = non_negative_i64(item.codex.tokens);
        }
        if let Some(stats) = day_stats.get_mut("antigravity") {
            stats.requests = non_negative_i64(item.antigravity.requests);
            stats.tokens = non_negative_i64(item.antigravity.tokens);
        }
        if let Some(stats) = day_stats.get_mut("opencode") {
            stats.requests = non_negative_i64(item.opencode.requests);
            stats.tokens = non_negative_i64(item.opencode.tokens);
        }
        daily_by_platform.insert(item.date, day_stats);
    }

    Ok(HomeUsageSnapshot {
        total_requests: non_negative_i64(payload.summary.total_requests),
        total_tokens: non_negative_i64(payload.summary.total_tokens),
        active_days: non_negative_i64(payload.summary.active_days),
        by_platform,
        daily_by_platform,
    })
}

pub fn load_home_usage_presence(
    dashboard: &crate::llmusage_adapter::Dashboard,
) -> Result<bool, String> {
    let summary = dashboard
        .overview(&crate::llmusage_adapter::QueryFilter::default())
        .map_err(|e| format!("Usage presence query error: {e}"))?;

    Ok(summary.total_events > 0)
}

pub fn load_llmusage_archive_diagnostics(
    dashboard: &crate::llmusage_adapter::Dashboard,
    archived_sessions: u64,
) -> Result<UsageArchiveDiagnostics, String> {
    let diagnostics = dashboard
        .diagnostics()
        .map_err(|e| format!("Archive diagnostics query error: {e}"))?;
    let mut recent_completed_at: Option<String> = None;
    let mut history_completed_at: Option<String> = None;
    let mut live_sources = 0u64;
    let mut missing_sources = 0u64;
    let mut deleted_sources = 0u64;
    let mut source_health = Vec::new();
    let now = Utc::now();

    for source in diagnostics.by_source {
        let source_recent_completed_at = source.recent_completed_at.clone();
        let source_history_completed_at = source.history_completed_at.clone();
        live_sources = live_sources.saturating_add(source.live_files);
        missing_sources = missing_sources.saturating_add(source.missing_files);
        deleted_sources = deleted_sources.saturating_add(source.deleted_files);
        recent_completed_at =
            queries::max_rfc3339(recent_completed_at, source_recent_completed_at.clone());
        history_completed_at =
            queries::max_rfc3339(history_completed_at, source_history_completed_at.clone());

        let latest_completed_at = queries::max_rfc3339(
            source_recent_completed_at.clone(),
            source_history_completed_at.clone(),
        );
        let freshness = usage_freshness_from_completed_at(latest_completed_at, now);
        source_health.push(UsageSourceHealth {
            source: source.source,
            state: usage_source_health_state(
                source.live_files,
                source.missing_files,
                source.deleted_files,
                freshness.state,
            ),
            live_sources: source.live_files,
            missing_sources: source.missing_files,
            deleted_sources: source.deleted_files,
            recent_completed_at: source_recent_completed_at,
            history_completed_at: source_history_completed_at,
            freshness,
        });
    }

    let freshness = usage_freshness_from_completed_at(
        queries::max_rfc3339(recent_completed_at.clone(), history_completed_at.clone()),
        now,
    );
    let mut archive = UsageArchiveDiagnostics {
        archive_root: diagnostics.archive_root,
        live_sources,
        missing_sources,
        deleted_sources,
        archived_sessions,
        recent_completed_at,
        history_completed_at,
        source_health,
        freshness,
        readiness: UsageReadinessProjection::default(),
    };
    archive.readiness = build_usage_readiness(&archive, 0, archived_sessions, false, false, false);
    Ok(archive)
}

pub fn count_archived_sessions(pool: &ccr_db::database::DbPool) -> Result<u64, String> {
    let conn = pool.get().map_err(|e| format!("DB error: {e}"))?;
    ccr_db::database::repositories::usage_repo::get_session_archive_platform_summaries(
        &conn, &None, &None,
    )
    .map_err(|e| format!("Archived session summary query error: {e}"))
    .map(|items| {
        items
            .into_iter()
            .map(|item| non_negative_i64(item.session_count))
            .sum()
    })
}

pub fn load_home_session_snapshot(
    pool: &ccr_db::database::DbPool,
    start_date: &str,
    end_date: &str,
) -> Result<HomeSessionSnapshot, String> {
    let conn = pool.get().map_err(|e| format!("DB error: {e}"))?;
    let start = Some(start_date.to_string());
    let end = Some(end_date.to_string());

    let summaries =
        ccr_db::database::repositories::usage_repo::get_session_archive_platform_summaries(
            &conn, &start, &end,
        )
        .map_err(|e| format!("Session summary query error: {e}"))?;
    let trends = ccr_db::database::repositories::usage_repo::get_session_archive_daily_trends(
        &conn, &start, &end,
    )
    .map_err(|e| format!("Session trend query error: {e}"))?;
    let has_any_sessions =
        ccr_db::database::repositories::usage_repo::has_any_session_archive(&conn)
            .map_err(|e| format!("Session archive presence query error: {e}"))?;

    let mut by_platform = empty_home_platform_map();
    let mut daily_by_platform = BTreeMap::new();
    let mut total_sessions = 0u64;

    for summary in summaries {
        total_sessions += non_negative_i64(summary.session_count);
        if let Some(stats) = by_platform.get_mut(summary.platform.as_str()) {
            stats.sessions = non_negative_i64(summary.session_count);
        }
    }

    for trend in trends {
        let day_entry = daily_by_platform
            .entry(trend.date.clone())
            .or_insert_with(empty_home_platform_map);
        if let Some(stats) = day_entry.get_mut(trend.platform.as_str()) {
            stats.sessions = non_negative_i64(trend.session_count);
        }
    }

    Ok(HomeSessionSnapshot {
        total_sessions,
        has_any_sessions,
        by_platform,
        daily_by_platform,
    })
}

/// 用量汇总查询（State-free，同步；由命令层放入 spawn_blocking）。
pub fn usage_summary(
    llmusage: &LlmusageRuntime,
    platform: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<UsageSummaryDto, String> {
    let filter = build_filter(platform, None, start_date, end_date)?;
    let dashboard = llmusage
        .dashboard()
        .map_err(|e| format!("Dashboard open error: {e}"))?;
    let summary = dashboard
        .overview(&filter)
        .map_err(|e| format!("Summary query error: {e}"))?;
    Ok(queries::to_usage_summary(summary))
}

/// 用量趋势查询。
pub fn usage_trends(
    llmusage: &LlmusageRuntime,
    platform: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<Vec<DailyTrendDto>, String> {
    let filter = build_filter(platform, None, start_date, end_date)?;
    let dashboard = llmusage
        .dashboard()
        .map_err(|e| format!("Dashboard open error: {e}"))?;
    let trends = dashboard
        .trends_daily(&filter)
        .map_err(|e| format!("Trends query error: {e}"))?;
    Ok(queries::to_daily_trends(trends))
}

/// 按模型聚合的用量统计。
pub fn usage_by_model(
    llmusage: &LlmusageRuntime,
    platform: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<Vec<ModelStatDto>, String> {
    let filter = build_filter(platform, None, start_date, end_date)?;
    let dashboard = llmusage
        .dashboard()
        .map_err(|e| format!("Dashboard open error: {e}"))?;
    let stats = dashboard
        .model_breakdown(&filter)
        .map_err(|e| format!("Model stats query error: {e}"))?;
    Ok(queries::to_model_stats(stats))
}

/// 按 provider 聚合的用量统计。
pub fn usage_by_provider(
    llmusage: &LlmusageRuntime,
    platform: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<Vec<ProviderBreakdownDto>, String> {
    let filter = build_filter(platform, None, start_date, end_date)?;
    let dashboard = llmusage
        .dashboard()
        .map_err(|e| format!("Dashboard open error: {e}"))?;
    let stats = dashboard
        .provider_breakdown(&filter)
        .map_err(|e| format!("Provider stats query error: {e}"))?;
    Ok(stats)
}

/// 按项目聚合的用量统计。
pub fn usage_by_project(
    llmusage: &LlmusageRuntime,
    platform: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<Vec<ProjectStatDto>, String> {
    let filter = build_filter(platform, None, start_date, end_date)?;
    let dashboard = llmusage
        .dashboard()
        .map_err(|e| format!("Dashboard open error: {e}"))?;
    let stats = dashboard
        .project_breakdown(&filter)
        .map_err(|e| format!("Project stats query error: {e}"))?;
    Ok(queries::to_project_stats(stats))
}

/// 热力图查询（days 已在命令层 clamp 到 1..=366）。
pub fn usage_heatmap(
    llmusage: &LlmusageRuntime,
    platform: Option<String>,
    days: u32,
) -> Result<HeatmapResponseDto, String> {
    let filter = build_filter(platform, None, None, None)?;
    let dashboard = llmusage
        .dashboard()
        .map_err(|e| format!("Dashboard open error: {e}"))?;
    let heatmap = dashboard
        .heatmap(&filter, days)
        .map_err(|e| format!("Heatmap query error: {e}"))?;
    Ok(queries::to_heatmap_response(heatmap))
}

/// 用量日志分页查询，保持前端 cursor 分页契约。
pub fn usage_logs(
    llmusage: &LlmusageRuntime,
    query: &UsageLogsQuery,
) -> Result<PaginatedLogsDto, String> {
    let page_size = query.page_size.unwrap_or(50).clamp(1, 500);
    let include_total = query.include_total.unwrap_or(false);
    // 注：query.page / query.mode 历史上为 offset 模式预留，但 cursor 模式已够用，
    //     llmusage 上游 logs() 也只接受 cursor。直到 offset 模式真正落地前，这两个
    //     字段在前端契约里保留可空、在后端这里完全忽略。
    let filter = build_filter(
        query.platform.clone(),
        query.model.clone(),
        query.start_date.clone(),
        query.end_date.clone(),
    )?;
    let dashboard = llmusage
        .dashboard()
        .map_err(|e| format!("Dashboard open error: {e}"))?;
    let logs = dashboard
        .logs(&LlmusageLogsQuery {
            filter,
            page_size: page_size as u32,
            cursor: query.cursor.clone(),
            include_total,
            include_raw_json: true,
        })
        .map_err(|e| format!("Logs query error: {e}"))?;

    Ok(queries::to_paginated_logs(logs, page_size))
}

/// dashboard V2 聚合计算（State-free，同步；命令层负责 spawn_blocking、计时与缓存）。
/// `raw_sessions_present` 由命令层经 `has_any_raw_sessions()` 原地计算传入，语义零变化。
#[allow(clippy::too_many_arguments)]
pub fn compute_dashboard(
    llmusage: &LlmusageRuntime,
    usage_db_pool: &ccr_db::database::DbPool,
    platform: Option<String>,
    provider: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
    heatmap_days: u32,
    include_heatmap: bool,
    active_usage_import: bool,
    active_session_index: bool,
    raw_sessions_present: bool,
) -> Result<UsageDashboardResponse, String> {
    let platform_scope = platform_scope_label(platform.as_deref());
    let snapshot_start_date = start_date.clone();
    let snapshot_end_date = end_date.clone();
    let filter = build_filter(platform.clone(), None, start_date.clone(), end_date.clone())?
        .with_provider(provider.clone());
    let dashboard = llmusage
        .dashboard()
        .map_err(|e| format!("Dashboard open error: {e}"))?;

    let summary = dashboard
        .overview(&filter)
        .map(queries::to_usage_summary)
        .map_err(|e| format!("Summary query error: {e}"))?;
    let trends = dashboard
        .trends_daily(&filter)
        .map(queries::to_daily_trends)
        .map_err(|e| format!("Trends query error: {e}"))?;
    let model_stats = dashboard
        .model_breakdown(&filter)
        .map(queries::to_model_stats)
        .map_err(|e| format!("Model stats query error: {e}"))?;
    let project_stats = dashboard
        .project_breakdown(&filter)
        .map(queries::to_project_stats)
        .map_err(|e| format!("Project stats query error: {e}"))?;
    let source_stats = dashboard
        .source_breakdown(&filter)
        .map_err(|e| format!("Source stats query error: {e}"))?;
    let provider_stats = match dashboard.provider_breakdown(&filter) {
        Ok(stats) => stats,
        Err(
            LlmusageAdapterError::SchemaUnsupported { .. }
            | LlmusageAdapterError::FeatureUnavailable { .. },
        ) if filter.provider.is_none() => Vec::new(),
        Err(error) => return Err(format!("Provider stats query error: {error}")),
    };
    let mut archive =
        load_llmusage_archive_diagnostics(&dashboard, count_archived_sessions(usage_db_pool)?)?;
    let needs_session_index = archive.archived_sessions == 0 && raw_sessions_present;
    archive.readiness = build_usage_readiness(
        &archive,
        non_negative_i64(summary.total_requests),
        archive.archived_sessions,
        needs_session_index,
        active_usage_import,
        active_session_index,
    );
    let snapshot = build_usage_snapshot_projection(
        platform_scope,
        snapshot_start_date,
        snapshot_end_date,
        &archive,
    );
    let generated_at = snapshot.generated_at.clone();
    let heatmap = if include_heatmap {
        Some(
            dashboard
                .heatmap(&filter, heatmap_days)
                .map(queries::to_heatmap_response)
                .map_err(|e| format!("Heatmap query error: {e}"))?,
        )
    } else {
        None
    };

    Ok(UsageDashboardResponse {
        summary,
        trends,
        model_stats,
        project_stats,
        source_stats,
        provider_stats,
        archive,
        snapshot,
        heatmap,
        generated_at,
    })
}

/// 首页工作区概览计算（State-free，同步）：统一 llmusage usage + ccr session 统计链路。
pub fn compute_home_overview(
    llmusage: &LlmusageRuntime,
    pool: &ccr_db::database::DbPool,
    days: usize,
    ctx: HomeJobContext,
) -> Result<HomeUsageOverviewResponse, String> {
    let (start_day, end_day) = local_usage_date_window(days);
    let start_date = format_local_usage_date(start_day);
    let end_date = format_local_usage_date(end_day);
    let filter = build_filter(None, None, Some(start_date.clone()), Some(end_date.clone()))?;
    let dashboard = llmusage
        .dashboard()
        .map_err(|e| format!("Dashboard open error: {e}"))?;
    let has_any_usage = load_home_usage_presence(&dashboard)?;
    let mut usage_snapshot = load_home_usage_snapshot(&dashboard, &filter)?;
    let session_snapshot = load_home_session_snapshot(pool, &start_date, &end_date)?;
    let mut archive =
        load_llmusage_archive_diagnostics(&dashboard, count_archived_sessions(pool)?)?;
    let has_any_sessions = session_snapshot.has_any_sessions;
    let needs_usage_import = !has_any_usage;
    let needs_session_index = !has_any_sessions && ctx.raw_sessions_present;

    for (platform_name, session_stats) in &session_snapshot.by_platform {
        if let Some(stats) = usage_snapshot.by_platform.get_mut(platform_name.as_str()) {
            stats.sessions = session_stats.sessions;
        }
    }

    for (date, session_day_stats) in &session_snapshot.daily_by_platform {
        let day_entry = usage_snapshot
            .daily_by_platform
            .entry(date.clone())
            .or_insert_with(empty_home_platform_map);
        for (platform_name, session_stats) in session_day_stats {
            if let Some(stats) = day_entry.get_mut(platform_name.as_str()) {
                stats.sessions = session_stats.sessions;
            }
        }
    }

    let date_range = build_home_date_range_from(start_day, days);
    let series = date_range
        .into_iter()
        .map(|date| {
            let mut day_stats = usage_snapshot
                .daily_by_platform
                .remove(&date)
                .unwrap_or_else(empty_home_platform_map);

            HomeOverviewSeriesItem {
                date,
                claude: day_stats.remove("claude").unwrap_or_default(),
                codex: day_stats.remove("codex").unwrap_or_default(),
                antigravity: day_stats.remove("antigravity").unwrap_or_default(),
                opencode: day_stats.remove("opencode").unwrap_or_default(),
            }
        })
        .collect::<Vec<_>>();

    let total_sessions = session_snapshot.total_sessions;
    let total_requests = usage_snapshot.total_requests;
    let total_tokens = usage_snapshot.total_tokens;
    archive.readiness = build_usage_readiness(
        &archive,
        total_requests,
        total_sessions,
        needs_session_index,
        ctx.active_usage_import,
        ctx.active_session_index,
    );
    let snapshot = build_usage_snapshot_projection(
        "all".to_string(),
        Some(start_date.clone()),
        Some(end_date.clone()),
        &archive,
    );
    let platforms = usage_snapshot
        .by_platform
        .values()
        .filter(|stats| stats.sessions > 0 || stats.requests > 0 || stats.tokens > 0)
        .count() as u64;
    let bootstrap = HomeOverviewBootstrap {
        usage_import_attempted: ctx.active_usage_job_id.is_some(),
        usage_imported_records: ctx.active_usage_imported_records,
        session_reindex_attempted: ctx.active_session_job_id.is_some(),
        indexed_sessions: ctx.active_session_indexed,
        usage_job_id: ctx.active_usage_job_id.clone(),
        session_job_id: ctx.active_session_job_id.clone(),
        needs_usage_import,
        needs_session_index,
        is_warm: !needs_usage_import && !needs_session_index,
    };

    Ok(HomeUsageOverviewResponse {
        summary: HomeOverviewSummary {
            total_sessions,
            total_requests,
            total_tokens,
            active_days: usage_snapshot.active_days,
            platforms,
        },
        by_platform: usage_snapshot.by_platform,
        series,
        bootstrap,
        archive,
        snapshot,
        empty_reason: detect_home_empty_reason(
            total_requests,
            total_sessions,
            has_any_usage,
            has_any_sessions,
        ),
        last_updated: Utc::now().to_rfc3339(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_home_platform_supports_common_aliases() {
        assert_eq!(normalize_home_platform("Claude Code"), Some("claude"));
        assert_eq!(normalize_home_platform("openai-codex"), Some("codex"));
        assert_eq!(
            normalize_home_platform("gemini-cli"),
            Some("antigravity")
        );
        assert_eq!(normalize_home_platform("Open Code"), Some("opencode"));
        assert_eq!(normalize_home_platform("Kimi Code"), Some("kimi_code"));
        assert_eq!(normalize_home_platform("oh-my-pi"), Some("pi"));
        assert_eq!(normalize_home_platform("Grok Build"), Some("grok"));
        assert_eq!(normalize_home_platform("legacy-cli"), None);
        assert_eq!(normalize_home_platform("unknown"), None);
    }

    #[test]
    fn home_date_range_is_inclusive_for_requested_days() {
        let start = chrono::NaiveDate::from_ymd_opt(2026, 5, 4).expect("valid date");
        assert_eq!(
            build_home_date_range_from(start, 7),
            vec![
                "2026-05-04".to_string(),
                "2026-05-05".to_string(),
                "2026-05-06".to_string(),
                "2026-05-07".to_string(),
                "2026-05-08".to_string(),
                "2026-05-09".to_string(),
                "2026-05-10".to_string(),
            ]
        );
    }

    #[test]
    fn detect_home_empty_reason_distinguishes_usage_and_sessions() {
        assert_eq!(
            detect_home_empty_reason(0, 0, false, false),
            Some("no_usage_and_sessions".to_string())
        );
        assert_eq!(
            detect_home_empty_reason(0, 3, false, true),
            Some("no_usage_logs".to_string())
        );
        assert_eq!(
            detect_home_empty_reason(5, 0, true, false),
            Some("no_session_index".to_string())
        );
        assert_eq!(detect_home_empty_reason(5, 3, true, true), None);
        assert_eq!(detect_home_empty_reason(0, 0, true, true), None);
    }

    #[test]
    fn usage_logs_query_supports_camel_case_aliases() {
        let value = serde_json::json!({
            "platform": "codex",
            "model": "gpt-5",
            "startDate": "2026-03-01",
            "endDate": "2026-03-05",
            "pageSize": 20,
            "includeTotal": true,
            "mode": "offset"
        });

        let query: UsageLogsQuery =
            serde_json::from_value(value).expect("query should deserialize");
        assert_eq!(query.start_date.as_deref(), Some("2026-03-01"));
        assert_eq!(query.end_date.as_deref(), Some("2026-03-05"));
        assert_eq!(query.page_size, Some(20));
        assert_eq!(query.include_total, Some(true));
        assert_eq!(query.mode, Some(UsageLogsMode::Offset));
    }

    #[test]
    fn usage_logs_mode_defaults_to_cursor() {
        let query = UsageLogsQuery {
            platform: None,
            model: None,
            start_date: None,
            end_date: None,
            page: None,
            page_size: None,
            cursor: None,
            include_total: None,
            mode: None,
        };

        assert_eq!(query.mode.unwrap_or_default(), UsageLogsMode::Cursor);
    }

    #[test]
    fn import_summary_marks_partial_and_failures() {
        let summary = build_import_summary(&[
            UsageImportResultV2 {
                platform: "claude".into(),
                files_processed: 3,
                records_imported: 12,
                records_skipped: 0,
                duration_ms: 10,
                completed: true,
                error: None,
                is_optional_absent: false,
            },
            UsageImportResultV2 {
                platform: "codex".into(),
                files_processed: 0,
                records_imported: 0,
                records_skipped: 0,
                duration_ms: 0,
                completed: false,
                error: Some("boom".into()),
                is_optional_absent: false,
            },
        ]);

        assert_eq!(summary.success_count, 1);
        assert_eq!(summary.failure_count, 1);
        assert_eq!(summary.imported_records, 12);
        assert_eq!(summary.processed_files, 3);
        assert!(summary.has_partial);
    }

    #[test]
    fn import_summary_detects_no_importable_logs() {
        let summary = build_import_summary(&[
            UsageImportResultV2 {
                platform: "claude".into(),
                files_processed: 0,
                records_imported: 0,
                records_skipped: 0,
                duration_ms: 3,
                completed: true,
                error: None,
                is_optional_absent: false,
            },
            UsageImportResultV2 {
                platform: "antigravity".into(),
                files_processed: 0,
                records_imported: 0,
                records_skipped: 0,
                duration_ms: 4,
                completed: true,
                error: None,
                is_optional_absent: false,
            },
        ]);

        assert_eq!(summary.success_count, 2);
        assert_eq!(summary.failure_count, 0);
        assert_eq!(summary.imported_records, 0);
        assert_eq!(summary.processed_files, 0);
        assert!(!summary.has_partial);
    }

    #[test]
    fn source_import_result_maps_llmusage_stats_to_ccr_contract() {
        let result = source_import_result(
            SourceKind::Codex,
            &SourceSyncStats {
                source: SourceKind::Codex,
                files_processed: 3,
                changed_files: 2,
                events_seen: 12,
                events_inserted: 9,
                parse_ms: 4,
                write_ms: 5,
                lock_wait_ms: 1,
                ..SourceSyncStats::default()
            },
        );

        assert_eq!(result.platform, "codex");
        assert_eq!(result.files_processed, 3);
        assert_eq!(result.records_imported, 9);
        assert_eq!(result.records_skipped, 3);
        assert_eq!(result.duration_ms, 10);
        assert!(result.completed);
        assert_eq!(result.error, None);
    }

    #[test]
    fn source_import_result_treats_missing_opencode_db_as_optional_absent_source() {
        let result = source_import_result(
            SourceKind::Opencode,
            &SourceSyncStats {
                source: SourceKind::Opencode,
                // llmusage 0.5.3+ 在 absent 路径同时填 typed flag + 旧 last_error 文案；
                // 下游只看 typed 字段，文案保留作 user-facing message。
                absent: true,
                last_error: Some("OpenCode SQLite DB 缺失".to_string()),
                ..SourceSyncStats::default()
            },
        );

        assert_eq!(result.platform, "opencode");
        assert!(result.completed);
        assert_eq!(result.error, None);
        assert!(result.is_optional_absent);

        let summary = build_import_summary(&[result]);
        assert_eq!(summary.success_count, 1);
        assert_eq!(summary.failure_count, 0);
        assert!(!summary.has_partial);
    }

    #[test]
    fn source_import_result_keeps_real_opencode_errors_actionable() {
        let result = source_import_result(
            SourceKind::Opencode,
            &SourceSyncStats {
                source: SourceKind::Opencode,
                last_error: Some("no such table: message".to_string()),
                ..SourceSyncStats::default()
            },
        );

        assert!(!result.completed);
        assert_eq!(result.error.as_deref(), Some("no such table: message"));
    }

    #[test]
    fn usage_freshness_classifies_missing_fresh_and_stale_sources() {
        let now = DateTime::parse_from_rfc3339("2026-05-25T12:00:00Z")
            .expect("valid test date")
            .with_timezone(&Utc);

        let missing = usage_freshness_from_completed_at(None, now);
        assert_eq!(missing.state, UsageFreshnessState::Missing);
        assert_eq!(missing.age_seconds, None);

        let fresh =
            usage_freshness_from_completed_at(Some("2026-05-25T11:55:00Z".to_string()), now);
        assert_eq!(fresh.state, UsageFreshnessState::Fresh);
        assert_eq!(fresh.age_seconds, Some(300));

        let stale =
            usage_freshness_from_completed_at(Some("2026-05-23T11:55:00Z".to_string()), now);
        assert_eq!(stale.state, UsageFreshnessState::Stale);
    }

    #[test]
    fn usage_source_health_promotes_missing_and_degraded_state() {
        assert_eq!(
            usage_source_health_state(0, 0, 0, UsageFreshnessState::Missing),
            UsageSourceHealthState::Missing
        );
        assert_eq!(
            usage_source_health_state(4, 0, 0, UsageFreshnessState::Fresh),
            UsageSourceHealthState::Live
        );
        assert_eq!(
            usage_source_health_state(4, 1, 0, UsageFreshnessState::Fresh),
            UsageSourceHealthState::Degraded
        );
        assert_eq!(
            usage_source_health_state(4, 0, 0, UsageFreshnessState::Stale),
            UsageSourceHealthState::Degraded
        );
    }

    #[test]
    fn usage_readiness_prioritizes_syncing_then_actions() {
        let mut archive = UsageArchiveDiagnostics {
            live_sources: 2,
            archived_sessions: 3,
            recent_completed_at: Some("2026-05-25T11:55:00Z".to_string()),
            freshness: UsageFreshnessProjection {
                state: UsageFreshnessState::Fresh,
                latest_completed_at: Some("2026-05-25T11:55:00Z".to_string()),
                age_seconds: Some(300),
                stale_after_seconds: USAGE_FRESHNESS_STALE_AFTER_SECS,
            },
            ..UsageArchiveDiagnostics::default()
        };

        let syncing = build_usage_readiness(&archive, 10, 3, false, true, false);
        assert_eq!(syncing.state, UsageReadinessState::Syncing);
        assert_eq!(syncing.next_action, None);

        archive.freshness.state = UsageFreshnessState::Stale;
        let stale = build_usage_readiness(&archive, 10, 3, false, false, false);
        assert_eq!(stale.state, UsageReadinessState::Stale);
        assert_eq!(stale.next_action.as_deref(), Some("refresh_usage"));

        archive.freshness.state = UsageFreshnessState::Fresh;
        archive.missing_sources = 1;
        let degraded = build_usage_readiness(&archive, 10, 3, false, false, false);
        assert_eq!(degraded.state, UsageReadinessState::Degraded);
        assert_eq!(degraded.next_action.as_deref(), Some("inspect_sources"));

        archive.missing_sources = 0;
        let ready = build_usage_readiness(&archive, 10, 3, false, false, false);
        assert_eq!(ready.state, UsageReadinessState::Ready);
        assert_eq!(ready.next_action, None);
    }

    #[test]
    fn usage_snapshot_projection_serializes_stable_drilldown_contract() {
        let archive = UsageArchiveDiagnostics {
            live_sources: 1,
            freshness: UsageFreshnessProjection {
                state: UsageFreshnessState::Fresh,
                latest_completed_at: Some("2026-05-25T11:55:00Z".to_string()),
                age_seconds: Some(300),
                stale_after_seconds: USAGE_FRESHNESS_STALE_AFTER_SECS,
            },
            readiness: UsageReadinessProjection {
                state: UsageReadinessState::Ready,
                detail: "ready".to_string(),
                has_live_sources: true,
                ..UsageReadinessProjection::default()
            },
            source_health: vec![UsageSourceHealth {
                source: "codex".to_string(),
                state: UsageSourceHealthState::Live,
                live_sources: 1,
                freshness: UsageFreshnessProjection {
                    state: UsageFreshnessState::Fresh,
                    latest_completed_at: Some("2026-05-25T11:55:00Z".to_string()),
                    age_seconds: Some(300),
                    stale_after_seconds: USAGE_FRESHNESS_STALE_AFTER_SECS,
                },
                ..UsageSourceHealth::default()
            }],
            ..UsageArchiveDiagnostics::default()
        };

        let snapshot = build_usage_snapshot_projection(
            "codex".to_string(),
            Some("2026-05-01".to_string()),
            Some("2026-05-25".to_string()),
            &archive,
        );
        let value = serde_json::to_value(snapshot).expect("snapshot should serialize");

        assert_eq!(value["platform_scope"], "codex");
        assert_eq!(value["readiness"]["state"], "ready");
        assert_eq!(value["source_health"][0]["source"], "codex");
        assert!(
            value["drilldown"]["dimensions"]
                .as_array()
                .expect("dimensions")
                .iter()
                .any(|dimension| dimension == "branch")
        );
    }
}

/// 查询编排 service 的封闭单测：llmusage 侧走 `ccr_usage::fixtures` 投影库，
/// session 侧走 temp ccr-db pool，全程不读写真实用户目录。
#[cfg(test)]
mod service_tests {
    use super::*;
    use ccr_usage::fixtures::{
        SeedBucket, SeedEvent, create_projection_db, seed_bucket, seed_event, seed_run_log,
        seed_source_file,
    };
    use rusqlite::Connection;
    use tempfile::TempDir;

    /// 打开 fixture 投影库，返回指向它的 runtime 与可继续 seed 的连接。
    fn open_fixture(temp: &TempDir) -> (LlmusageRuntime, Connection) {
        let paths = create_projection_db(temp.path());
        let conn = Connection::open(&paths.db_path).expect("fixture db should reopen");
        (LlmusageRuntime::from_paths(paths), conn)
    }

    /// 临时 ccr-db usage pool（home_dir 传 temp，防止迁移逻辑碰真实家目录）。
    fn temp_usage_pool(temp: &TempDir) -> ccr_db::database::DbPool {
        let pool = ccr_db::database::create_pool(&temp.path().join("usage.db"), None)
            .expect("usage pool should be created");
        let conn = pool.get().expect("pool should hand out a connection");
        ccr_db::database::migrations::run_all_migrations(&conn, temp.path())
            .expect("migrations should run");
        drop(conn);
        pool
    }

    /// 给定本地日期的正午 UTC 时刻：date(hour_start,'localtime') 在 ±11h 时区内
    /// 稳定落在该日期，规避测试机时区导致的日期漂移。
    fn local_noon_utc(date: chrono::NaiveDate) -> String {
        date.and_hms_opt(12, 0, 0)
            .expect("noon should be a valid time")
            .and_local_timezone(Local)
            .earliest()
            .expect("local noon should exist")
            .with_timezone(&Utc)
            .to_rfc3339()
    }

    /// 参照种子：codex/openai/gpt-5(p1, 100 tokens, 2 req) 与 codex 未归因
    /// gpt-4(p1, 20 tokens, 1 req) 同日，claude/anthropic(p2, 50 tokens, 1 req) 次日。
    fn seed_reference_buckets(conn: &Connection) {
        seed_bucket(conn, &SeedBucket::default());
        seed_bucket(
            conn,
            &SeedBucket {
                provider_label: String::new(),
                model: "gpt-4".to_string(),
                hour_start: "2026-07-01T12:30:00Z".to_string(),
                input_tokens: 10,
                cache_read_tokens: 0,
                cache_creation_tokens: 0,
                output_tokens: 5,
                reasoning_output_tokens: 5,
                total_tokens: 20,
                event_count: 1,
                cost_with_cache_usd: 0.02,
                cost_without_cache_usd: 0.03,
                pricing_rate: Some("rate-b".to_string()),
                ..SeedBucket::default()
            },
        );
        seed_bucket(
            conn,
            &SeedBucket {
                source: "claude".to_string(),
                provider_label: "anthropic".to_string(),
                model: "claude-sonnet".to_string(),
                hour_start: "2026-07-02T12:00:00Z".to_string(),
                project_hash: "p2".to_string(),
                project_label: "Project 2".to_string(),
                project_ref: None,
                input_tokens: 20,
                cache_read_tokens: 0,
                cache_creation_tokens: 0,
                output_tokens: 25,
                reasoning_output_tokens: 5,
                total_tokens: 50,
                event_count: 1,
                cost_with_cache_usd: 0.20,
                cost_without_cache_usd: 0.25,
                pricing_rate: Some("rate-c".to_string()),
                ..SeedBucket::default()
            },
        );
    }

    fn empty_home_ctx() -> HomeJobContext {
        HomeJobContext {
            active_usage_job_id: None,
            active_usage_imported_records: 0,
            active_session_job_id: None,
            active_session_indexed: 0,
            active_usage_import: false,
            active_session_index: false,
            raw_sessions_present: false,
        }
    }

    fn logs_query(
        page_size: Option<i64>,
        cursor: Option<String>,
        include_total: Option<bool>,
    ) -> UsageLogsQuery {
        UsageLogsQuery {
            platform: None,
            model: None,
            start_date: None,
            end_date: None,
            page: None,
            page_size,
            cursor,
            include_total,
            mode: None,
        }
    }

    fn assert_close(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() < 1e-9,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn usage_summary_aggregates_seeds_and_applies_platform_filter() {
        let temp = TempDir::new().expect("temp dir should be created");
        let (runtime, conn) = open_fixture(&temp);
        seed_reference_buckets(&conn);
        drop(conn);

        let all = usage_summary(&runtime, None, None, None).expect("summary should query");
        // 由种子推导：requests 2+1+1、tokens 100+20+50、input 40+10+20、
        // output(含 reasoning) 45+10+30、cache_read 10+0+0、cost 0.10+0.02+0.20。
        assert_eq!(all.total_requests, 4);
        assert_eq!(all.total_tokens, 170);
        assert_eq!(all.total_input_tokens, 70);
        assert_eq!(all.total_output_tokens, 85);
        assert_eq!(all.total_cache_read_tokens, 10);
        assert_close(all.total_cost_usd, 0.32);
        assert_close(all.cache_efficiency, 10.0 / 80.0);

        let claude = usage_summary(&runtime, Some("claude".to_string()), None, None)
            .expect("filtered summary should query");
        assert_eq!(claude.total_requests, 1);
        assert_eq!(claude.total_tokens, 50);
    }

    #[test]
    fn usage_trends_groups_by_day_and_respects_date_range() {
        let temp = TempDir::new().expect("temp dir should be created");
        let (runtime, conn) = open_fixture(&temp);
        seed_reference_buckets(&conn);
        drop(conn);

        let trends = usage_trends(&runtime, None, None, None).expect("trends should query");
        assert_eq!(trends.len(), 2);
        assert_eq!(trends[0].date, "2026-07-01");
        assert_eq!(trends[0].request_count, 3);
        assert_eq!(trends[0].input_tokens, 50);
        // 兼容口径：output 列 = output(35) + reasoning(20)
        assert_eq!(trends[0].output_tokens, 55);
        assert_eq!(trends[0].reasoning_output_tokens, 20);
        assert_eq!(trends[0].total_tokens, 120);
        assert_close(trends[0].cost_usd, 0.12);
        assert_eq!(trends[1].date, "2026-07-02");
        assert_eq!(trends[1].total_tokens, 50);

        let ranged = usage_trends(
            &runtime,
            None,
            Some("2026-07-02".to_string()),
            Some("2026-07-02".to_string()),
        )
        .expect("ranged trends should query");
        assert_eq!(ranged.len(), 1);
        assert_eq!(ranged[0].date, "2026-07-02");
    }

    #[test]
    fn usage_by_model_maps_breakdown_ordering_and_costs() {
        let temp = TempDir::new().expect("temp dir should be created");
        let (runtime, conn) = open_fixture(&temp);
        seed_reference_buckets(&conn);
        drop(conn);

        let stats = usage_by_model(&runtime, None, None, None).expect("model stats should query");
        assert_eq!(
            stats.iter().map(|s| s.model.as_str()).collect::<Vec<_>>(),
            vec!["gpt-5", "claude-sonnet", "gpt-4"]
        );
        let gpt5 = &stats[0];
        assert_eq!(gpt5.request_count, 2);
        assert_eq!(gpt5.total_tokens, 100);
        assert_eq!(gpt5.input_tokens, 40);
        // ModelStatDto 映射口径：output = output(30) + reasoning(15)
        assert_eq!(gpt5.output_tokens, 45);
        assert_eq!(gpt5.cache_read_tokens, 10);
        assert_eq!(gpt5.cache_creation_tokens, 5);
        assert_close(gpt5.total_cost, 0.10);
        assert_close(gpt5.cost_without_cache, 0.15);
        assert_close(gpt5.cache_savings, 0.05);
        assert_eq!(gpt5.pricing_status, "priced");
        assert_eq!(gpt5.pricing_source.as_deref(), Some("catalog"));
        assert_eq!(gpt5.pricing_rate.as_deref(), Some("rate-a"));
    }

    #[test]
    fn usage_by_model_preserves_llmusage_static_v1_fable_mythos_rows() {
        let temp = TempDir::new().expect("temp dir should be created");
        let (runtime, conn) = open_fixture(&temp);
        for model in ["claude-fable-5", "claude-mythos-5"] {
            seed_bucket(
                &conn,
                &SeedBucket {
                    source: "claude".to_string(),
                    provider_label: "anthropic".to_string(),
                    model: model.to_string(),
                    input_tokens: 1_000_000,
                    cache_read_tokens: 200_000,
                    cache_creation_tokens: 300_000,
                    output_tokens: 400_000,
                    reasoning_output_tokens: 0,
                    total_tokens: 1_900_000,
                    event_count: 1,
                    cost_with_cache_usd: 33.95,
                    cost_without_cache_usd: 35.0,
                    pricing_status: "static".to_string(),
                    pricing_source: Some("static-v1".to_string()),
                    pricing_rate: Some("10/1/50".to_string()),
                    ..SeedBucket::default()
                },
            );
        }
        drop(conn);

        let stats = usage_by_model(&runtime, Some("claude".to_string()), None, None)
            .expect("model stats should query");
        assert_eq!(
            stats
                .iter()
                .map(|row| row.model.as_str())
                .collect::<Vec<_>>(),
            vec!["claude-fable-5", "claude-mythos-5"]
        );

        for row in stats {
            assert_eq!(row.input_tokens, 1_000_000);
            assert_eq!(row.cache_read_tokens, 200_000);
            assert_eq!(row.cache_creation_tokens, 300_000);
            assert_eq!(row.output_tokens, 400_000);
            assert_close(row.total_cost, 33.95);
            assert_close(row.cost_with_cache, 33.95);
            assert_close(row.cost_without_cache, 35.0);
            assert_close(row.cache_savings, 1.05);
            assert_eq!(row.pricing_status, "static");
            assert_eq!(row.pricing_source.as_deref(), Some("static-v1"));
            assert_eq!(row.pricing_rate.as_deref(), Some("10/1/50"));
        }
    }

    #[test]
    fn usage_by_provider_aggregates_and_maps_unattributed_to_none() {
        let temp = TempDir::new().expect("temp dir should be created");
        let (runtime, conn) = open_fixture(&temp);
        seed_reference_buckets(&conn);
        drop(conn);

        let stats =
            usage_by_provider(&runtime, None, None, None).expect("provider stats should query");
        assert_eq!(stats.len(), 3);
        assert_eq!(stats[0].provider.as_deref(), Some("openai"));
        assert_eq!(stats[0].total_tokens, 100);
        assert_eq!(stats[0].request_count, 2);
        assert_eq!(stats[1].provider.as_deref(), Some("anthropic"));
        assert_eq!(stats[1].total_tokens, 50);
        // 空 provider_label → 未归因 None
        assert_eq!(stats[2].provider, None);
        assert_eq!(stats[2].total_tokens, 20);

        let claude_only = usage_by_provider(&runtime, Some("claude".to_string()), None, None)
            .expect("filtered provider stats should query");
        assert_eq!(claude_only.len(), 1);
        assert_eq!(claude_only[0].provider.as_deref(), Some("anthropic"));
    }

    #[test]
    fn usage_by_project_prefers_ref_and_excludes_empty_hash() {
        let temp = TempDir::new().expect("temp dir should be created");
        let (runtime, conn) = open_fixture(&temp);
        seed_reference_buckets(&conn);
        // 空 project_hash 的 bucket 不计入项目统计（WHERE project_hash <> ''）
        seed_bucket(
            &conn,
            &SeedBucket {
                project_hash: String::new(),
                total_tokens: 999,
                ..SeedBucket::default()
            },
        );
        drop(conn);

        let stats =
            usage_by_project(&runtime, None, None, None).expect("project stats should query");
        assert_eq!(stats.len(), 2);
        assert_eq!(stats[0].project_path, "/repo/p1");
        assert_eq!(stats[0].total_tokens, 120);
        assert_eq!(stats[0].request_count, 3);
        // project_ref 缺失时回退到 project_label。
        assert_eq!(stats[1].project_path, "Project 2");
        assert_eq!(stats[1].total_tokens, 50);
        assert_eq!(stats.iter().map(|s| s.total_tokens).sum::<i64>(), 170);
    }

    #[test]
    fn usage_heatmap_zero_fills_requested_window_and_maps_dates() {
        let temp = TempDir::new().expect("temp dir should be created");
        let (runtime, conn) = open_fixture(&temp);
        let today = Local::now().date_naive();
        let outside = today - Duration::days(5);
        seed_bucket(
            &conn,
            &SeedBucket {
                hour_start: local_noon_utc(today),
                event_count: 4,
                ..SeedBucket::default()
            },
        );
        seed_bucket(
            &conn,
            &SeedBucket {
                hour_start: local_noon_utc(outside),
                event_count: 7,
                ..SeedBucket::default()
            },
        );
        drop(conn);

        let heatmap = usage_heatmap(&runtime, None, 3).expect("heatmap should query");
        // days=3 → 窗口逐日补零，共 3 个键；窗口外的种子日期不出现
        assert_eq!(heatmap.data.len(), 3);
        assert_eq!(heatmap.data.get(&format_local_usage_date(today)), Some(&4));
        assert_eq!(
            heatmap
                .data
                .get(&format_local_usage_date(today - Duration::days(1))),
            Some(&0)
        );
        assert!(!heatmap.data.contains_key(&format_local_usage_date(outside)));
    }

    #[test]
    fn usage_logs_honors_cursor_pagination_total_and_page_size_clamp() {
        let temp = TempDir::new().expect("temp dir should be created");
        let (runtime, conn) = open_fixture(&temp);
        for (key, at) in [
            ("ev-1", "2026-07-01T12:00:00Z"),
            ("ev-2", "2026-07-01T12:30:00Z"),
            ("ev-3", "2026-07-01T13:00:00Z"),
        ] {
            seed_event(
                &conn,
                &SeedEvent {
                    event_key: key.to_string(),
                    event_at: at.to_string(),
                    raw_json: format!(r#"{{"key":"{key}"}}"#),
                    ..SeedEvent::default()
                },
            );
        }
        drop(conn);

        let first = usage_logs(&runtime, &logs_query(Some(2), None, Some(true)))
            .expect("first page should query");
        assert_eq!(first.records.len(), 2);
        // event_at 倒序：最新在前
        assert_eq!(first.records[0].id, "ev-3");
        assert_eq!(first.records[1].id, "ev-2");
        assert_eq!(first.total, Some(3));
        assert_eq!(first.page_size, 2);
        assert_eq!(first.mode, "cursor");
        // usage_event_raw join 生效：record_json 即种子 raw_json
        assert_eq!(first.records[0].record_json, r#"{"key":"ev-3"}"#);
        let cursor = first
            .next_cursor
            .clone()
            .expect("first page must have cursor");

        let second = usage_logs(&runtime, &logs_query(Some(2), Some(cursor), None))
            .expect("second page should query");
        assert_eq!(second.records.len(), 1);
        assert_eq!(second.records[0].id, "ev-1");
        assert_eq!(second.next_cursor, None);
        // include_total 缺省 false
        assert_eq!(second.total, None);

        // page_size clamp：0 → 1，9999 → 500
        let clamped_low = usage_logs(&runtime, &logs_query(Some(0), None, None))
            .expect("clamped-low page should query");
        assert_eq!(clamped_low.page_size, 1);
        assert_eq!(clamped_low.records.len(), 1);
        let clamped_high = usage_logs(&runtime, &logs_query(Some(9999), None, None))
            .expect("clamped-high page should query");
        assert_eq!(clamped_high.page_size, 500);
        assert_eq!(clamped_high.records.len(), 3);
        assert_eq!(clamped_high.next_cursor, None);
    }

    #[test]
    fn compute_dashboard_assembles_seeded_stats_without_heatmap() {
        let temp = TempDir::new().expect("temp dir should be created");
        let (runtime, conn) = open_fixture(&temp);
        seed_reference_buckets(&conn);
        seed_source_file(&conn, "codex", "live");
        seed_run_log(&conn, "codex", "recent", &Utc::now().to_rfc3339());
        drop(conn);
        let pool = temp_usage_pool(&temp);

        let response = compute_dashboard(
            &runtime, &pool, None, None, None, None, 30, false, false, false, false,
        )
        .expect("dashboard should compute");

        assert_eq!(response.summary.total_requests, 4);
        assert_eq!(response.summary.total_tokens, 170);
        assert_eq!(response.trends.len(), 2);
        assert_eq!(response.model_stats.len(), 3);
        assert_eq!(response.model_stats[0].model, "gpt-5");
        assert_eq!(response.project_stats.len(), 2);
        assert_eq!(response.project_stats[0].project_path, "/repo/p1");
        assert_eq!(response.source_stats.len(), 2);
        assert_eq!(response.source_stats[0].source, "codex");
        assert_eq!(response.source_stats[0].total_tokens, 120);
        assert_close(response.source_stats[0].share_tokens, 120.0 / 170.0);
        assert_eq!(response.provider_stats.len(), 3);
        assert_eq!(
            response.provider_stats[0].provider.as_deref(),
            Some("openai")
        );
        assert!(response.heatmap.is_none());
        assert_eq!(response.generated_at, response.snapshot.generated_at);
        assert_eq!(response.snapshot.platform_scope, "all");
        assert_eq!(response.archive.live_sources, 1);
        assert_eq!(response.archive.archived_sessions, 0);
        // raw_sessions_present=false：空归档不触发 NeedsSessionIndex，链路健康 → Ready
        assert_eq!(
            response.snapshot.readiness.state,
            UsageReadinessState::Ready
        );
        assert_eq!(
            response.snapshot.freshness.state,
            UsageFreshnessState::Fresh
        );

        // raw_sessions_present=true 且归档为空 → NeedsSessionIndex（新入参生效）
        let needs_index = compute_dashboard(
            &runtime, &pool, None, None, None, None, 30, false, false, false, true,
        )
        .expect("dashboard should compute with raw sessions present");
        assert_eq!(
            needs_index.snapshot.readiness.state,
            UsageReadinessState::NeedsSessionIndex
        );
    }

    #[test]
    fn compute_dashboard_includes_heatmap_with_seeded_dates_when_requested() {
        let temp = TempDir::new().expect("temp dir should be created");
        let (runtime, conn) = open_fixture(&temp);
        let today = Local::now().date_naive();
        seed_bucket(
            &conn,
            &SeedBucket {
                hour_start: local_noon_utc(today),
                ..SeedBucket::default()
            },
        );
        drop(conn);
        let pool = temp_usage_pool(&temp);

        let response = compute_dashboard(
            &runtime, &pool, None, None, None, None, 7, true, false, false, false,
        )
        .expect("dashboard should compute");

        let heatmap = response.heatmap.expect("heatmap should be included");
        assert_eq!(heatmap.data.len(), 7);
        // 默认种子 event_count=2 落在今日
        assert_eq!(heatmap.data.get(&format_local_usage_date(today)), Some(&2));
    }

    #[test]
    fn compute_home_overview_on_empty_stores_reports_cold_bootstrap() {
        let temp = TempDir::new().expect("temp dir should be created");
        let (runtime, conn) = open_fixture(&temp);
        drop(conn);
        let pool = temp_usage_pool(&temp);

        let response = compute_home_overview(&runtime, &pool, 7, empty_home_ctx())
            .expect("home overview should compute");

        assert_eq!(response.series.len(), 7);
        assert_eq!(
            response
                .by_platform
                .keys()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            vec![
                "antigravity",
                "claude",
                "codex",
                "grok",
                "kimi_code",
                "opencode",
                "pi",
            ]
        );
        assert_eq!(response.summary.total_sessions, 0);
        assert_eq!(response.summary.total_requests, 0);
        assert_eq!(response.summary.total_tokens, 0);
        assert_eq!(response.summary.platforms, 0);
        // 冷启动：usage 缺 → needs_usage_import；无原始 session → 不需要索引；is_warm 自洽
        assert!(response.bootstrap.needs_usage_import);
        assert!(!response.bootstrap.needs_session_index);
        assert_eq!(
            response.bootstrap.is_warm,
            !response.bootstrap.needs_usage_import && !response.bootstrap.needs_session_index
        );
        assert!(!response.bootstrap.usage_import_attempted);
        assert_eq!(response.bootstrap.usage_job_id, None);
        assert_eq!(
            response.empty_reason.as_deref(),
            Some("no_usage_and_sessions")
        );
        assert_eq!(
            response.snapshot.readiness.state,
            UsageReadinessState::NeedsImport
        );
        assert_eq!(response.snapshot.platform_scope, "all");
    }

    #[test]
    fn compute_home_overview_joins_usage_series_and_marks_warm_bootstrap() {
        let temp = TempDir::new().expect("temp dir should be created");
        let (runtime, conn) = open_fixture(&temp);
        let today = Local::now().date_naive();
        seed_bucket(
            &conn,
            &SeedBucket {
                hour_start: local_noon_utc(today),
                event_count: 3,
                total_tokens: 120,
                ..SeedBucket::default()
            },
        );
        seed_source_file(&conn, "codex", "live");
        seed_run_log(&conn, "codex", "recent", &Utc::now().to_rfc3339());
        drop(conn);
        let pool = temp_usage_pool(&temp);

        let response = compute_home_overview(&runtime, &pool, 7, empty_home_ctx())
            .expect("home overview should compute");

        assert_eq!(response.series.len(), 7);
        let today_key = format_local_usage_date(today);
        let today_item = response
            .series
            .iter()
            .find(|item| item.date == today_key)
            .expect("today should be in series");
        assert_eq!(today_item.codex.requests, 3);
        assert_eq!(today_item.codex.tokens, 120);
        assert_eq!(today_item.codex.sessions, 0);
        assert_eq!(response.by_platform["codex"].requests, 3);
        assert_eq!(response.summary.total_requests, 3);
        assert_eq!(response.summary.total_tokens, 120);
        assert_eq!(response.summary.active_days, 1);
        assert_eq!(response.summary.platforms, 1);
        assert_eq!(response.summary.total_sessions, 0);
        // usage 已有、无原始 session 文件：needs_* 全 false → warm
        assert!(!response.bootstrap.needs_usage_import);
        assert!(!response.bootstrap.needs_session_index);
        assert!(response.bootstrap.is_warm);
        assert_eq!(response.empty_reason.as_deref(), Some("no_session_index"));
        assert_eq!(
            response.snapshot.readiness.state,
            UsageReadinessState::Ready
        );
    }

    #[test]
    fn usage_dashboard_response_cache_value_round_trips() {
        let temp = TempDir::new().expect("temp dir should be created");
        let (runtime, conn) = open_fixture(&temp);
        seed_reference_buckets(&conn);
        seed_source_file(&conn, "codex", "live");
        seed_run_log(&conn, "codex", "recent", &Utc::now().to_rfc3339());
        drop(conn);
        let pool = temp_usage_pool(&temp);

        let response = compute_dashboard(
            &runtime, &pool, None, None, None, None, 7, true, false, false, false,
        )
        .expect("dashboard should compute");

        // 缓存往返（命令层 cache_set/from_value 路径）：Value 层相等，绕开 f64 PartialEq
        let cached = serde_json::to_value(&response).expect("response should serialize");
        let decoded: UsageDashboardResponse =
            serde_json::from_value(cached.clone()).expect("cached value should decode");
        let reencoded = serde_json::to_value(&decoded).expect("decoded response should serialize");
        assert_eq!(cached, reencoded);
    }
}
