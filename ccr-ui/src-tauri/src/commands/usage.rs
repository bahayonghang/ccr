//! Usage V2 命令模块，基于 SQLite 查询与导入用量数据。
//!
//! # 数据源边界（v6.1.0 deliberate 双链路）
//!
//! 本模块同时消费两条独立数据链路：
//!
//! - **usage events** 走 [`crate::llmusage_adapter`]：导入只调用已安装的
//!   `llmusage` CLI，渲染只读 `~/.llmusage/llmusage.db`，覆盖仪表盘、趋势、模型/项目分布、成本、heatmap、
//!   logs 分页、source 状态机等查询（仪表盘主链路）。ccr-ui 不再链接 llmusage crate，
//!   也不 bootstrap / migrate / repair 这份 DB。
//! - **session_archive**（会话归档摘要）暂存 `ccr_db::database::repositories::usage_repo`
//!   (`~/.ccr/analytics/usage.db`)，覆盖 `upsert_session_archive_entry` /
//!   `mark_session_archive_missing_by_platform` /
//!   `mark_session_archive_deleted_by_path` /
//!   `get_session_archive_platform_summaries` /
//!   `get_session_archive_daily_trends` / `has_any_session_archive` 等调用。
//!
//! ## 为什么是双链路
//!
//! 长期目标是把 session_archive 也迁到 llmusage 上游 schema（见
//! `docs/llmusage-integration-prd.md` §9c 与 follow-up issue
//! <https://github.com/bahayonghang/ccr/issues/35>）。在 llmusage 补齐
//! "session 摘要"表 + dashboard API 之前，本仓保留 ccr_db 旁路。
//!
//! ## 给 reviewer
//!
//! 凡是 `ccr_db::database::repositories::usage_repo::*session_archive*` 调用
//! 都属于这条旁路，不是漏迁。usage events 链路里出现 `usage_repo` 引用才是 bug。

use std::collections::BTreeMap;
use std::time::Instant;

use ccr_config::Platform;
use ccr_store::sessions::{SessionIndexer, parser::SessionParser};
use chrono::{Duration, Local, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager, State};
use uuid::Uuid;

use crate::events::{self, UsageImportPayload};
use crate::llmusage_adapter::{
    JobEvent, LogsQuery as LlmusageLogsQuery, SourceKind, SourceSyncStats, SyncCommandOptions,
    SyncSummaryEvent, build_filter, canonical_source_id, is_optional_source_absent,
    platform_scope_label, queries,
};
use crate::monitoring::{emit_and_record_monitoring_event, should_persist, usage_import_entry};
use crate::session_index_jobs::SessionIndexJobSnapshot;
use crate::state::AppState;
use crate::usage_jobs::{UsageImportJobSnapshot, UsageImportJobStage};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum UsageLogsMode {
    #[default]
    Cursor,
    Offset,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct UsageLogsQuery {
    pub platform: Option<String>,
    pub model: Option<String>,
    #[serde(alias = "startDate")]
    pub start_date: Option<String>,
    #[serde(alias = "endDate")]
    pub end_date: Option<String>,
    #[allow(dead_code)]
    pub page: Option<i64>,
    #[serde(alias = "pageSize")]
    pub page_size: Option<i64>,
    pub cursor: Option<String>,
    #[serde(alias = "includeTotal")]
    pub include_total: Option<bool>,
    #[allow(dead_code)]
    pub mode: Option<UsageLogsMode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageImportResultV2 {
    pub platform: String,
    pub files_processed: usize,
    pub records_imported: usize,
    pub records_skipped: usize,
    pub duration_ms: u64,
    pub completed: bool,
    pub error: Option<String>,
    /// optional source（如 OpenCode）缺失安装时为 true。
    /// 前端依此判断"导入失败"是不是用户没装这个 optional source 的正常情况，
    /// 不要嗅探 `error` 字符串。
    #[serde(default)]
    pub is_optional_absent: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UsageImportSummary {
    pub success_count: usize,
    pub failure_count: usize,
    pub imported_records: usize,
    pub processed_files: usize,
    pub has_partial: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportAllUsageResponse {
    pub results: Vec<UsageImportResultV2>,
    pub summary: UsageImportSummary,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UsageArchiveDiagnostics {
    pub archive_root: String,
    pub live_sources: u64,
    pub missing_sources: u64,
    pub deleted_sources: u64,
    pub archived_sessions: u64,
    pub recent_completed_at: Option<String>,
    pub history_completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StartUsageImportJobResponse {
    pub job_id: String,
    pub snapshot: UsageImportJobSnapshot,
}

#[derive(Debug, Clone, Serialize)]
pub struct StartSessionIndexJobResponse {
    pub job_id: String,
    pub snapshot: SessionIndexJobSnapshot,
}

#[tauri::command]
pub async fn get_usage_capabilities_v2(state: State<'_, AppState>) -> Result<Value, String> {
    serde_json::to_value(state.llmusage.capabilities()).map_err(|e| format!("Serialize error: {e}"))
}

const HOME_USAGE_PLATFORMS: [&str; 4] = ["claude", "codex", "gemini", "opencode"];

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct HomeOverviewPlatformStats {
    pub sessions: u64,
    pub requests: u64,
    pub tokens: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct HomeOverviewSummary {
    pub total_sessions: u64,
    pub total_requests: u64,
    pub total_tokens: u64,
    pub active_days: u64,
    pub platforms: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct HomeOverviewBootstrap {
    pub usage_import_attempted: bool,
    pub usage_imported_records: usize,
    pub session_reindex_attempted: bool,
    pub indexed_sessions: u64,
    pub usage_job_id: Option<String>,
    pub session_job_id: Option<String>,
    pub needs_usage_import: bool,
    pub needs_session_index: bool,
    pub is_warm: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HomeOverviewSeriesItem {
    pub date: String,
    pub claude: HomeOverviewPlatformStats,
    pub codex: HomeOverviewPlatformStats,
    pub gemini: HomeOverviewPlatformStats,
    pub opencode: HomeOverviewPlatformStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomeUsageOverviewResponse {
    pub summary: HomeOverviewSummary,
    pub by_platform: BTreeMap<String, HomeOverviewPlatformStats>,
    pub series: Vec<HomeOverviewSeriesItem>,
    pub bootstrap: HomeOverviewBootstrap,
    pub archive: UsageArchiveDiagnostics,
    pub empty_reason: Option<String>,
    pub last_updated: String,
}

struct HomeUsageSnapshot {
    total_requests: u64,
    total_tokens: u64,
    active_days: u64,
    by_platform: BTreeMap<String, HomeOverviewPlatformStats>,
    daily_by_platform: BTreeMap<String, BTreeMap<String, HomeOverviewPlatformStats>>,
}

struct HomeSessionSnapshot {
    total_sessions: u64,
    has_any_sessions: bool,
    by_platform: BTreeMap<String, HomeOverviewPlatformStats>,
    daily_by_platform: BTreeMap<String, BTreeMap<String, HomeOverviewPlatformStats>>,
}

fn elapsed_ms(started: Instant) -> f64 {
    started.elapsed().as_secs_f64() * 1000.0
}

fn record_command_duration(state: &AppState, command_started: Instant) {
    state.record_command_duration_ms(elapsed_ms(command_started));
}

fn record_db_duration(state: &AppState, db_ms: f64) {
    state.record_db_query_duration_ms(db_ms);
}

fn source_import_result(source: SourceKind, stats: &SourceSyncStats) -> UsageImportResultV2 {
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

fn build_import_summary(results: &[UsageImportResultV2]) -> UsageImportSummary {
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

fn default_import_results(platform: Option<&str>) -> Vec<UsageImportResultV2> {
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

async fn emit_usage_import_job_snapshot(
    app_handle: &AppHandle,
    event: &str,
    snapshot: &UsageImportJobSnapshot,
) {
    if let Err(error) = app_handle.emit(event, snapshot.clone()) {
        tracing::warn!(event, ?error, job_id = %snapshot.job_id, "Failed to emit usage import job event");
    }
}

async fn emit_session_index_job_snapshot(
    app_handle: &AppHandle,
    event: &str,
    snapshot: &SessionIndexJobSnapshot,
) {
    if let Err(error) = app_handle.emit(event, snapshot.clone()) {
        tracing::warn!(event, ?error, job_id = %snapshot.job_id, "Failed to emit session index job event");
    }
}

fn session_index_platforms() -> [Platform; 3] {
    [Platform::Claude, Platform::Codex, Platform::Gemini]
}

fn session_index_platform_label(platform: Platform) -> &'static str {
    match platform {
        Platform::Claude => "claude",
        Platform::Codex => "codex",
        Platform::Gemini => "gemini",
        _ => "legacy",
    }
}

fn session_index_file_count(platform: Platform) -> u64 {
    let Some(session_dir) = SessionParser::get_platform_session_dir(&platform) else {
        return 0;
    };

    SessionParser::scan_directory(&session_dir, platform)
        .map(|files| files.len() as u64)
        .unwrap_or(0)
}

fn archive_entry_from_session(
    platform_label: &str,
    session: &ccr_store::sessions::Session,
) -> ccr_db::database::repositories::usage_repo::UsageSessionArchiveEntry {
    ccr_db::database::repositories::usage_repo::UsageSessionArchiveEntry {
        archive_id: format!(
            "{}:{}:{}",
            platform_label,
            session.id,
            session.file_path.display()
        ),
        session_id: session.id.clone(),
        platform: platform_label.to_string(),
        title: session.title.clone(),
        cwd: session.cwd.display().to_string(),
        file_path: session.file_path.display().to_string(),
        file_hash: Some(session.file_hash.clone()),
        message_count: i64::from(session.message_count),
        created_at: session.created_at,
        updated_at: session.updated_at,
        source_state: ccr_db::database::repositories::usage_repo::UsageSourceState::Live,
        last_seen_at: Some(Utc::now()),
        raw_deleted_at: None,
        archived_at: Utc::now(),
    }
}

fn sync_platform_session_archive(
    usage_db_pool: &ccr_db::database::pool::DbPool,
    platform: Platform,
) -> Result<(), String> {
    let platform_label = session_index_platform_label(platform).to_string();
    let Some(session_dir) = SessionParser::get_platform_session_dir(&platform) else {
        let conn = usage_db_pool.get().map_err(|e| format!("DB error: {e}"))?;
        ccr_db::database::repositories::usage_repo::mark_session_archive_missing_by_platform(
            &conn,
            &platform_label,
            &[],
        )
        .map_err(|e| format!("Session archive reconcile error: {e}"))?;
        return Ok(());
    };

    let files = SessionParser::scan_directory(&session_dir, platform)
        .map_err(|error| format!("Session archive scan failed: {error}"))?;
    let seen_paths = files
        .iter()
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>();
    let (sessions, _stats) = SessionParser::parse_files(&files, platform);
    let conn = usage_db_pool.get().map_err(|e| format!("DB error: {e}"))?;

    /*
     * ========================================================================
     * 步骤1：刷新当前平台的最小 session 摘要归档
     * ========================================================================
     * 目标：
     * 1) 将 parser 产出的最小 session 摘要持久化到 usage_session_archive
     * 2) 保留 durable archive，不依赖原始 session 文件持续存在
     */
    tracing::info!("[usage-session-archive] start sync platform={platform_label}");
    for session in sessions {
        ccr_db::database::repositories::usage_repo::upsert_session_archive_entry(
            &conn,
            &archive_entry_from_session(&platform_label, &session),
        )
        .map_err(|e| format!("Session archive upsert error: {e}"))?;
    }
    tracing::info!("[usage-session-archive] finish sync platform={platform_label}");

    ccr_db::database::repositories::usage_repo::mark_session_archive_missing_by_platform(
        &conn,
        &platform_label,
        &seen_paths,
    )
    .map_err(|e| format!("Session archive reconcile error: {e}"))?;
    Ok(())
}

async fn run_usage_import_job(
    app_handle: AppHandle,
    job_id: String,
    platform: Option<String>,
    options: SyncCommandOptions,
) {
    let state = app_handle.state::<AppState>();
    let cli = state.llmusage.cli().clone();
    let cancel_token = tokio_util::sync::CancellationToken::new();
    state
        .register_usage_import_cancel_token(&job_id, cancel_token.clone())
        .await;
    let results_by_platform = std::sync::Arc::new(tokio::sync::Mutex::new(BTreeMap::<
        String,
        UsageImportResultV2,
    >::new()));
    let platform_for_bridge = platform.clone();
    let results_for_bridge = results_by_platform.clone();
    let job_id_for_bridge = job_id.clone();
    let app_for_bridge = app_handle.clone();

    let execution = crate::llmusage_adapter::cli::run_sync_stream(
        &cli,
        options,
        cancel_token.clone(),
        move |event| {
            let app_handle = app_for_bridge.clone();
            let job_id = job_id_for_bridge.clone();
            let platform = platform_for_bridge.clone();
            let results_by_platform = results_for_bridge.clone();
            async move {
                bridge_llmusage_import_event(
                    &app_handle,
                    &job_id,
                    platform.as_deref(),
                    event,
                    &results_by_platform,
                )
                .await
            }
        },
    )
    .await;

    // 子进程已退出 / 取消 / 报错都到这里，统一清理 cancel token 防泄漏。
    let _ = app_handle
        .state::<AppState>()
        .take_usage_import_cancel_token(&job_id)
        .await;

    if let Err(error) = execution {
        tracing::error!(job_id = %job_id, ?error, "Usage import job failed");
        if let Some(snapshot) =
            sync_llmusage_failure_snapshot(&app_handle, &job_id, error.to_string()).await
        {
            emit_usage_import_job_snapshot(&app_handle, "usage:job-failed", &snapshot).await;
        }
    }
}

async fn bridge_llmusage_import_event(
    app_handle: &AppHandle,
    job_id: &str,
    platform: Option<&str>,
    event: JobEvent,
    results_by_platform: &tokio::sync::Mutex<BTreeMap<String, UsageImportResultV2>>,
) -> Result<(), String> {
    if state_usage_import_job_is_terminal(app_handle, job_id).await {
        return Ok(());
    }

    match event {
        JobEvent::Started { files_total, .. } => {
            update_usage_job_progress(app_handle, job_id, |job| {
                job.mark_running(
                    UsageImportJobStage::ImportingRecent,
                    files_total as usize,
                    None,
                );
            })
            .await;
        }
        JobEvent::BootstrapStarted
        | JobEvent::MigrationStarted { .. }
        | JobEvent::MigrationFinished { .. }
        | JobEvent::LockWaiting { .. }
        | JobEvent::LockAcquired { .. } => {
            update_usage_job_progress(app_handle, job_id, |job| {
                job.mark_running(UsageImportJobStage::ImportingRecent, job.files_total, None);
            })
            .await;
        }
        JobEvent::SourceStarted {
            source,
            files_total,
        } => {
            update_usage_job_progress(app_handle, job_id, |job| {
                job.mark_running(
                    UsageImportJobStage::ImportingRecent,
                    files_total as usize,
                    Some(source.as_str().to_string()),
                );
            })
            .await;
        }
        JobEvent::Progress {
            source,
            files_scanned,
            records_imported,
            current_file,
        } => {
            update_usage_job_progress(app_handle, job_id, |job| {
                job.status = crate::usage_jobs::UsageImportJobStatus::Running;
                job.stage = UsageImportJobStage::ImportingRecent;
                job.files_scanned = files_scanned as usize;
                job.records_imported = records_imported as usize;
                job.current_file = current_file.or_else(|| Some(source.as_str().to_string()));
                job.updated_at = Utc::now().to_rfc3339();
            })
            .await;
        }
        JobEvent::RecentReady { .. } => {
            if let Some(snapshot) = app_handle
                .state::<AppState>()
                .update_usage_import_job(job_id, |job: &mut UsageImportJobSnapshot| {
                    job.mark_recent_ready(true);
                })
                .await
            {
                emit_usage_import_job_snapshot(app_handle, "usage:job-recent-ready", &snapshot)
                    .await;
            }
        }
        JobEvent::SourceFinished { source, stats } => {
            let result = source_import_result(source, &stats);
            let result_error = result.error.clone();
            let mut guard = results_by_platform.lock().await;
            guard.insert(source.as_str().to_string(), result);
            let files_processed = guard.values().map(|result| result.files_processed).sum();
            let records_imported = guard.values().map(|result| result.records_imported).sum();
            let records_skipped = guard.values().map(|result| result.records_skipped).sum();
            let files_imported = guard
                .values()
                .filter(|result| result.records_imported > 0)
                .map(|result| result.files_processed)
                .sum();
            drop(guard);

            update_usage_job_progress(app_handle, job_id, |job| {
                job.status = crate::usage_jobs::UsageImportJobStatus::Running;
                job.stage = UsageImportJobStage::ImportingHistory;
                job.files_total = job.files_total.max(files_processed);
                job.files_scanned = files_processed;
                job.files_imported = files_imported;
                job.records_imported = records_imported;
                job.records_skipped = records_skipped;
                job.current_file = None;
                if let Some(error) = result_error {
                    job.push_warning(format!("{}: {}", source.as_str(), error));
                }
                job.updated_at = Utc::now().to_rfc3339();
            })
            .await;
        }
        JobEvent::Finished { summary } => {
            let guard = results_by_platform.lock().await;
            finish_llmusage_import_job(app_handle, job_id, platform, guard.clone(), summary)
                .await?;
        }
        JobEvent::Failed { error } => return Err(error),
        JobEvent::Cancelled => {
            if let Some(snapshot) = app_handle
                .state::<AppState>()
                .update_usage_import_job(job_id, |job: &mut UsageImportJobSnapshot| {
                    job.mark_cancelled();
                })
                .await
            {
                emit_usage_import_job_snapshot(app_handle, "usage:job-failed", &snapshot).await;
            }
        }
    }

    Ok(())
}

async fn update_usage_job_progress<F>(app_handle: &AppHandle, job_id: &str, updater: F)
where
    F: FnOnce(&mut UsageImportJobSnapshot),
{
    if let Some(snapshot) = app_handle
        .state::<AppState>()
        .update_usage_import_job(job_id, updater)
        .await
    {
        emit_usage_import_job_snapshot(app_handle, "usage:job-progress", &snapshot).await;
    }
}

async fn state_usage_import_job_is_terminal(app_handle: &AppHandle, job_id: &str) -> bool {
    app_handle
        .state::<AppState>()
        .get_usage_import_job(job_id)
        .await
        .is_some_and(|snapshot| {
            matches!(
                snapshot.status,
                crate::usage_jobs::UsageImportJobStatus::Finished
                    | crate::usage_jobs::UsageImportJobStatus::Failed
                    | crate::usage_jobs::UsageImportJobStatus::Cancelled
            )
        })
}

async fn finish_llmusage_import_job(
    app_handle: &AppHandle,
    job_id: &str,
    platform: Option<&str>,
    mut results_by_platform: BTreeMap<String, UsageImportResultV2>,
    summary_event: SyncSummaryEvent,
) -> Result<(), String> {
    let mut results = if results_by_platform.is_empty() {
        default_import_results(canonical_source_id(platform).as_deref())
    } else {
        results_by_platform
            .values_mut()
            .for_each(|result| result.completed = result.error.is_none());
        results_by_platform.into_values().collect::<Vec<_>>()
    };
    results.sort_by(|left, right| left.platform.cmp(&right.platform));

    let mut summary = build_import_summary(&results);
    summary.imported_records = summary.imported_records.max(summary_event.total_inserted);
    let final_snapshot = app_handle
        .state::<AppState>()
        .update_usage_import_job(job_id, |job: &mut UsageImportJobSnapshot| {
            job.files_total = job.files_total.max(summary_event.sources);
            job.records_imported = job.records_imported.max(summary_event.total_inserted);
            job.mark_finished(results.clone(), summary.clone());
        })
        .await
        .ok_or_else(|| format!("Usage import job '{}' not found", job_id))?;

    let payload = UsageImportPayload {
        imported_count: summary.imported_records,
        platform: platform_scope_label(platform),
    };
    let entry = usage_import_entry(&payload);
    let persist = should_persist(entry.level, &entry.event_type);
    emit_and_record_monitoring_event(
        app_handle,
        events::channels::USAGE_IMPORT,
        &payload,
        entry,
        persist,
    )
    .await;

    emit_usage_import_job_snapshot(app_handle, "usage:job-finished", &final_snapshot).await;
    Ok(())
}

async fn sync_llmusage_failure_snapshot(
    app_handle: &AppHandle,
    job_id: &str,
    fallback_error: String,
) -> Option<UsageImportJobSnapshot> {
    app_handle
        .state::<AppState>()
        .update_usage_import_job(job_id, |job: &mut UsageImportJobSnapshot| {
            job.mark_failed(fallback_error.clone());
        })
        .await
}

async fn run_llmusage_sync_all(
    state: &AppState,
    recent_days: Option<u32>,
    rebuild: bool,
) -> Result<Vec<UsageImportResultV2>, String> {
    let events = crate::llmusage_adapter::run_sync_collect(
        state.llmusage.cli(),
        SyncCommandOptions {
            rebuild,
            recent_days,
            source: None,
        },
    )
    .await
    .map_err(|error| error.to_string())?;
    collect_llmusage_sync_results(events, None)
}

async fn run_llmusage_sync_once(
    state: &AppState,
    source: Option<String>,
    recent_days: Option<u32>,
    rebuild: bool,
) -> Result<UsageImportResultV2, String> {
    let requested_source = source.clone();
    let parsed_source = source
        .as_deref()
        .and_then(crate::llmusage_adapter::parse_source_filter);
    let events = crate::llmusage_adapter::run_sync_collect(
        state.llmusage.cli(),
        SyncCommandOptions {
            rebuild,
            recent_days,
            source: parsed_source,
        },
    )
    .await
    .map_err(|error| error.to_string())?;
    let mut results = collect_llmusage_sync_results(events, requested_source.as_deref())?;
    Ok(results
        .pop()
        .unwrap_or_else(|| default_import_results(requested_source.as_deref()).remove(0)))
}

fn collect_llmusage_sync_results(
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

async fn run_session_index_job(app_handle: AppHandle, job_id: String) {
    let platforms = session_index_platforms();
    let usage_db_pool = app_handle.state::<AppState>().usage_db_pool.clone();

    let execution = async {
        let files_total: u64 = platforms
            .iter()
            .copied()
            .map(session_index_file_count)
            .sum();

        if let Some(snapshot) = app_handle
            .state::<AppState>()
            .update_session_index_job(&job_id, |job: &mut SessionIndexJobSnapshot| {
                job.files_total = files_total;
            })
            .await
        {
            emit_session_index_job_snapshot(&app_handle, "usage:session-index-progress", &snapshot)
                .await;
        }

        let indexer = SessionIndexer::new()
            .map_err(|error| format!("Failed to create session indexer: {error}"))?;
        let mut completed_files = 0u64;

        for platform in platforms {
            let platform_label = session_index_platform_label(platform).to_string();
            if let Some(snapshot) = app_handle
                .state::<AppState>()
                .update_session_index_job(&job_id, |job: &mut SessionIndexJobSnapshot| {
                    job.mark_running(Some(platform_label.clone()), files_total, completed_files);
                })
                .await
            {
                emit_session_index_job_snapshot(
                    &app_handle,
                    "usage:session-index-progress",
                    &snapshot,
                )
                .await;
            }

            match indexer.index_platform(platform) {
                Ok(stats) => {
                    // sync_platform_session_archive 内部做 SQLite get + upsert 是同步阻塞 IO，
                    // 必须放到 spawn_blocking 里跑，避免占用 tokio worker 线程。
                    let archive_pool = usage_db_pool.clone();
                    tokio::task::spawn_blocking(move || {
                        sync_platform_session_archive(&archive_pool, platform)
                    })
                    .await
                    .map_err(|join_err| {
                        format!("Session archive sync task panicked: {join_err}")
                    })??;
                    completed_files += stats.files_scanned;
                    if let Some(snapshot) = app_handle
                        .state::<AppState>()
                        .update_session_index_job(&job_id, |job: &mut SessionIndexJobSnapshot| {
                            job.record_platform_result(
                                stats.files_scanned,
                                stats.sessions_added,
                                stats.sessions_updated,
                                stats.errors,
                            );
                        })
                        .await
                    {
                        emit_session_index_job_snapshot(
                            &app_handle,
                            "usage:session-index-progress",
                            &snapshot,
                        )
                        .await;
                    }
                }
                Err(error) => {
                    if let Some(snapshot) = app_handle
                        .state::<AppState>()
                        .update_session_index_job(&job_id, |job: &mut SessionIndexJobSnapshot| {
                            job.push_warning(format!("{}: {}", platform_label, error));
                            job.record_platform_result(0, 0, 0, 1);
                        })
                        .await
                    {
                        emit_session_index_job_snapshot(
                            &app_handle,
                            "usage:session-index-progress",
                            &snapshot,
                        )
                        .await;
                    }
                }
            }
        }

        let final_snapshot = app_handle
            .state::<AppState>()
            .update_session_index_job(&job_id, |job: &mut SessionIndexJobSnapshot| {
                job.mark_finished();
            })
            .await
            .ok_or_else(|| format!("Session index job '{}' not found", job_id))?;

        emit_session_index_job_snapshot(
            &app_handle,
            "usage:session-index-finished",
            &final_snapshot,
        )
        .await;
        Ok::<(), String>(())
    };

    if let Err(error) = execution.await {
        tracing::error!(job_id = %job_id, ?error, "Session index job failed");
        if let Some(snapshot) = app_handle
            .state::<AppState>()
            .update_session_index_job(&job_id, |job: &mut SessionIndexJobSnapshot| {
                job.mark_failed(error.clone())
            })
            .await
        {
            emit_session_index_job_snapshot(&app_handle, "usage:session-index-failed", &snapshot)
                .await;
        }
    }
}

fn empty_home_platform_map() -> BTreeMap<String, HomeOverviewPlatformStats> {
    let mut map = BTreeMap::new();
    for platform in HOME_USAGE_PLATFORMS {
        map.insert(platform.to_string(), HomeOverviewPlatformStats::default());
    }
    map
}

#[cfg(test)]
fn normalize_home_platform(raw: &str) -> Option<&'static str> {
    match raw.trim().to_lowercase().as_str() {
        "claude" | "claude-code" | "claude code" => Some("claude"),
        "codex" | "openai-codex" | "openai codex" => Some("codex"),
        "gemini" | "gemini-cli" | "gemini cli" | "google-gemini" | "google gemini" => {
            Some("gemini")
        }
        "opencode" | "open-code" | "open code" => Some("opencode"),
        _ => None,
    }
}

fn non_negative_i64(value: i64) -> u64 {
    value.max(0) as u64
}

fn local_usage_date_window(days: usize) -> (chrono::NaiveDate, chrono::NaiveDate) {
    let safe_days = days.max(1);
    let end = Local::now().date_naive();
    let start = end - Duration::days((safe_days - 1) as i64);
    (start, end)
}

fn format_local_usage_date(date: chrono::NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}

fn build_home_date_range_from(start: chrono::NaiveDate, days: usize) -> Vec<String> {
    let safe_days = days.max(1);
    (0..safe_days)
        .map(|offset| format_local_usage_date(start + Duration::days(offset as i64)))
        .collect()
}

fn detect_home_empty_reason(
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

fn has_any_raw_sessions() -> bool {
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

fn load_home_usage_snapshot(
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
        if let Some(stats) = day_stats.get_mut("gemini") {
            stats.requests = non_negative_i64(item.gemini.requests);
            stats.tokens = non_negative_i64(item.gemini.tokens);
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

fn load_home_usage_presence(
    dashboard: &crate::llmusage_adapter::Dashboard,
) -> Result<bool, String> {
    let summary = dashboard
        .overview(&crate::llmusage_adapter::QueryFilter::default())
        .map_err(|e| format!("Usage presence query error: {e}"))?;

    Ok(summary.total_events > 0)
}

fn load_llmusage_archive_diagnostics(
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

    for source in diagnostics.by_source {
        live_sources = live_sources.saturating_add(source.live_files);
        missing_sources = missing_sources.saturating_add(source.missing_files);
        deleted_sources = deleted_sources.saturating_add(source.deleted_files);
        recent_completed_at = queries::max_rfc3339(recent_completed_at, source.recent_completed_at);
        history_completed_at =
            queries::max_rfc3339(history_completed_at, source.history_completed_at);
    }

    Ok(UsageArchiveDiagnostics {
        archive_root: diagnostics.archive_root,
        live_sources,
        missing_sources,
        deleted_sources,
        archived_sessions,
        recent_completed_at,
        history_completed_at,
    })
}

fn count_archived_sessions(pool: &ccr_db::database::pool::DbPool) -> Result<u64, String> {
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

fn load_home_session_snapshot(
    pool: &ccr_db::database::pool::DbPool,
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

/// 获取用量汇总数据
#[tauri::command]
pub async fn get_usage_summary_v2(
    state: State<'_, AppState>,
    platform: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<Value, String> {
    let command_started = Instant::now();
    let llmusage = state.llmusage.clone();
    let result = tokio::task::spawn_blocking(move || {
        let db_started = Instant::now();
        let filter = build_filter(platform, None, start_date, end_date)?;
        let dashboard = llmusage
            .dashboard()
            .map_err(|e| format!("Dashboard open error: {e}"))?;
        let summary = dashboard
            .overview(&filter)
            .map_err(|e| format!("Summary query error: {e}"))?;
        Ok::<_, String>((queries::to_usage_summary(summary), elapsed_ms(db_started)))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?;

    record_command_duration(&state, command_started);
    let (summary, db_ms) = result?;
    record_db_duration(&state, db_ms);
    serde_json::to_value(summary).map_err(|e| format!("Serialize error: {e}"))
}

/// 获取用量趋势数据
#[tauri::command]
pub async fn get_usage_trends_v2(
    state: State<'_, AppState>,
    platform: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<Value, String> {
    let command_started = Instant::now();
    let llmusage = state.llmusage.clone();
    let result = tokio::task::spawn_blocking(move || {
        let db_started = Instant::now();
        let filter = build_filter(platform, None, start_date, end_date)?;
        let dashboard = llmusage
            .dashboard()
            .map_err(|e| format!("Dashboard open error: {e}"))?;
        let trends = dashboard
            .trends_daily(&filter)
            .map_err(|e| format!("Trends query error: {e}"))?;
        Ok::<_, String>((queries::to_daily_trends(trends), elapsed_ms(db_started)))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?;

    record_command_duration(&state, command_started);
    let (trends, db_ms) = result?;
    record_db_duration(&state, db_ms);
    serde_json::to_value(trends).map_err(|e| format!("Serialize error: {e}"))
}

/// 获取按模型聚合的用量统计
#[tauri::command]
pub async fn get_usage_by_model_v2(
    state: State<'_, AppState>,
    platform: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<Value, String> {
    let command_started = Instant::now();
    let llmusage = state.llmusage.clone();
    let result = tokio::task::spawn_blocking(move || {
        let db_started = Instant::now();
        let filter = build_filter(platform, None, start_date, end_date)?;
        let dashboard = llmusage
            .dashboard()
            .map_err(|e| format!("Dashboard open error: {e}"))?;
        let stats = dashboard
            .model_breakdown(&filter)
            .map_err(|e| format!("Model stats query error: {e}"))?;
        Ok::<_, String>((queries::to_model_stats(stats), elapsed_ms(db_started)))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?;

    record_command_duration(&state, command_started);
    let (stats, db_ms) = result?;
    record_db_duration(&state, db_ms);
    serde_json::to_value(stats).map_err(|e| format!("Serialize error: {e}"))
}

/// 获取按项目聚合的用量统计
#[tauri::command]
pub async fn get_usage_by_project_v2(
    state: State<'_, AppState>,
    platform: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<Value, String> {
    let command_started = Instant::now();
    let llmusage = state.llmusage.clone();
    let result = tokio::task::spawn_blocking(move || {
        let db_started = Instant::now();
        let filter = build_filter(platform, None, start_date, end_date)?;
        let dashboard = llmusage
            .dashboard()
            .map_err(|e| format!("Dashboard open error: {e}"))?;
        let stats = dashboard
            .project_breakdown(&filter)
            .map_err(|e| format!("Project stats query error: {e}"))?;
        Ok::<_, String>((queries::to_project_stats(stats), elapsed_ms(db_started)))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?;

    record_command_duration(&state, command_started);
    let (stats, db_ms) = result?;
    record_db_duration(&state, db_ms);
    serde_json::to_value(stats).map_err(|e| format!("Serialize error: {e}"))
}

/// 获取热力图数据（V2，来自 llmusage usage_bucket_30m）
#[tauri::command]
pub async fn get_usage_heatmap_v2(
    state: State<'_, AppState>,
    platform: Option<String>,
    days: Option<i64>,
) -> Result<Value, String> {
    let command_started = Instant::now();
    let llmusage = state.llmusage.clone();
    let days = days.unwrap_or(365).clamp(1, 366) as u32;

    let result = tokio::task::spawn_blocking(move || {
        let db_started = Instant::now();
        let filter = build_filter(platform, None, None, None)?;
        let dashboard = llmusage
            .dashboard()
            .map_err(|e| format!("Dashboard open error: {e}"))?;
        let heatmap = dashboard
            .heatmap(&filter, days)
            .map_err(|e| format!("Heatmap query error: {e}"))?;
        Ok::<_, String>((
            queries::to_heatmap_response(heatmap),
            elapsed_ms(db_started),
        ))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?;

    record_command_duration(&state, command_started);
    let (heatmap, db_ms) = result?;
    record_db_duration(&state, db_ms);
    serde_json::to_value(heatmap).map_err(|e| format!("Serialize error: {e}"))
}

/// 获取用量日志列表，保持前端 cursor 分页契约
#[tauri::command]
pub async fn get_usage_logs_v2(
    state: State<'_, AppState>,
    query: UsageLogsQuery,
) -> Result<Value, String> {
    let command_started = Instant::now();

    let page_size = query.page_size.unwrap_or(50).clamp(1, 500);
    let include_total = query.include_total.unwrap_or(false);
    // 注：query.page / query.mode 历史上为 offset 模式预留，但 cursor 模式已够用，
    //     llmusage 上游 logs() 也只接受 cursor。直到 offset 模式真正落地前，这两个
    //     字段在前端契约里保留可空、在后端这里完全忽略。
    let llmusage = state.llmusage.clone();
    let platform = query.platform.clone();
    let model = query.model.clone();
    let start_date = query.start_date.clone();
    let end_date = query.end_date.clone();
    let cursor = query.cursor.clone();

    let result = tokio::task::spawn_blocking(move || {
        let db_started = Instant::now();
        let filter = build_filter(platform, model, start_date, end_date)?;
        let dashboard = llmusage
            .dashboard()
            .map_err(|e| format!("Dashboard open error: {e}"))?;
        let logs = dashboard
            .logs(&LlmusageLogsQuery {
                filter,
                page_size: page_size as u32,
                cursor,
                include_total,
                include_raw_json: true,
            })
            .map_err(|e| format!("Logs query error: {e}"))?;

        Ok::<_, String>((
            queries::to_paginated_logs(logs, page_size),
            elapsed_ms(db_started),
        ))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?;

    record_command_duration(&state, command_started);
    let (logs, db_ms) = result?;
    record_db_duration(&state, db_ms);
    serde_json::to_value(logs).map_err(|e| format!("Serialize error: {e}"))
}

/// 获取用量仪表盘数据，聚合汇总、趋势、模型与项目统计
#[tauri::command]
pub async fn get_usage_dashboard_v2(
    state: State<'_, AppState>,
    platform: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
    heatmap_days: Option<i64>,
    include_heatmap: Option<bool>,
) -> Result<Value, String> {
    let command_started = Instant::now();
    let llmusage = state.llmusage.clone();
    let usage_db_pool = state.usage_db_pool.clone();
    let heatmap_days = heatmap_days.unwrap_or(365).clamp(1, 366) as u32;
    let include_heatmap = include_heatmap.unwrap_or(false);
    let result = tokio::task::spawn_blocking(move || {
        let db_started = Instant::now();
        let filter = build_filter(platform, None, start_date, end_date)?;
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
        let archive = load_llmusage_archive_diagnostics(
            &dashboard,
            count_archived_sessions(&usage_db_pool)?,
        )?;
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

        Ok::<Value, String>(serde_json::json!({
            "summary": summary,
            "trends": trends,
            "model_stats": model_stats,
            "project_stats": project_stats,
            "archive": archive,
            "heatmap": heatmap,
            "generated_at": queries::generated_at(),
        }))
        .map(|payload| (payload, elapsed_ms(db_started)))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?;

    record_command_duration(&state, command_started);
    let (dashboard, db_ms) = result?;
    record_db_duration(&state, db_ms);
    Ok(dashboard)
}

/// 获取首页工作区概览数据，统一 llmusage usage + ccr session 统计链路。
#[tauri::command]
pub async fn get_home_usage_overview_v2(
    state: State<'_, AppState>,
    days: Option<usize>,
) -> Result<Value, String> {
    let command_started = Instant::now();
    let pool = state.usage_db_pool.clone();
    let llmusage = state.llmusage.clone();
    let days = days.unwrap_or(30).max(1);
    let active_usage_job = state.get_active_usage_import_job().await;
    let active_usage_job_id = active_usage_job.as_ref().map(|job| job.job_id.clone());
    let active_usage_imported_records = active_usage_job
        .as_ref()
        .map(|job| job.records_imported)
        .unwrap_or(0);
    let active_session_job = state.get_active_session_index_job().await;
    let active_session_job_id = active_session_job.as_ref().map(|job| job.job_id.clone());
    let active_session_indexed = active_session_job
        .as_ref()
        .map(|job| job.sessions_added + job.sessions_updated)
        .unwrap_or(0);

    let result = tokio::task::spawn_blocking(move || {
        let db_started = Instant::now();
        let (start_day, end_day) = local_usage_date_window(days);
        let start_date = format_local_usage_date(start_day);
        let end_date = format_local_usage_date(end_day);
        let filter = build_filter(None, None, Some(start_date.clone()), Some(end_date.clone()))?;
        let dashboard = llmusage
            .dashboard()
            .map_err(|e| format!("Dashboard open error: {e}"))?;
        let has_any_usage = load_home_usage_presence(&dashboard)?;
        let mut usage_snapshot = load_home_usage_snapshot(&dashboard, &filter)?;
        let session_snapshot = load_home_session_snapshot(&pool, &start_date, &end_date)?;
        let archive =
            load_llmusage_archive_diagnostics(&dashboard, count_archived_sessions(&pool)?)?;
        let has_any_sessions = session_snapshot.has_any_sessions;
        let needs_usage_import = !has_any_usage;
        let needs_session_index = !has_any_sessions && has_any_raw_sessions();

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
                    gemini: day_stats.remove("gemini").unwrap_or_default(),
                    opencode: day_stats.remove("opencode").unwrap_or_default(),
                }
            })
            .collect::<Vec<_>>();

        let total_sessions = session_snapshot.total_sessions;
        let total_requests = usage_snapshot.total_requests;
        let total_tokens = usage_snapshot.total_tokens;
        let platforms = usage_snapshot
            .by_platform
            .values()
            .filter(|stats| stats.sessions > 0 || stats.requests > 0 || stats.tokens > 0)
            .count() as u64;
        let bootstrap = HomeOverviewBootstrap {
            usage_import_attempted: active_usage_job_id.is_some(),
            usage_imported_records: active_usage_imported_records,
            session_reindex_attempted: active_session_job_id.is_some(),
            indexed_sessions: active_session_indexed,
            usage_job_id: active_usage_job_id.clone(),
            session_job_id: active_session_job_id.clone(),
            needs_usage_import,
            needs_session_index,
            is_warm: !needs_usage_import && !needs_session_index,
        };

        let payload = HomeUsageOverviewResponse {
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
            empty_reason: detect_home_empty_reason(
                total_requests,
                total_sessions,
                has_any_usage,
                has_any_sessions,
            ),
            last_updated: Utc::now().to_rfc3339(),
        };

        Ok::<_, String>((payload, elapsed_ms(db_started)))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?;

    record_command_duration(&state, command_started);
    let (payload, db_ms) = result?;
    record_db_duration(&state, db_ms);
    serde_json::to_value(payload).map_err(|e| format!("Serialize error: {e}"))
}

#[tauri::command]
pub async fn ensure_session_index_v2(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<Value, String> {
    if let Some(snapshot) = state.get_active_session_index_job().await {
        return serde_json::to_value(StartSessionIndexJobResponse {
            job_id: snapshot.job_id.clone(),
            snapshot,
        })
        .map_err(|e| format!("Serialization error: {e}"));
    }

    let job_id = format!("session-index-{}", Uuid::new_v4());
    let snapshot = SessionIndexJobSnapshot::new(job_id.clone(), session_index_platforms().len());
    state.insert_session_index_job(snapshot.clone()).await;

    tauri::async_runtime::spawn(run_session_index_job(app_handle, job_id.clone()));

    serde_json::to_value(StartSessionIndexJobResponse { job_id, snapshot })
        .map_err(|e| format!("Serialization error: {e}"))
}

#[tauri::command]
pub async fn get_session_index_job_status_v2(
    state: State<'_, AppState>,
    job_id: String,
) -> Result<Value, String> {
    let snapshot = state
        .get_session_index_job(&job_id)
        .await
        .ok_or_else(|| format!("Session index job '{}' not found", job_id))?;

    serde_json::to_value(snapshot).map_err(|e| format!("Serialization error: {e}"))
}

/// 从 JSONL 文件导入单个平台的用量数据
#[tauri::command]
pub async fn import_usage_v2(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    platform: String,
) -> Result<Value, String> {
    let source = canonical_source_id(Some(&platform))
        .ok_or_else(|| format!("Unsupported usage platform '{platform}'"))?;
    let result = run_llmusage_sync_once(&state, Some(source.clone()), None, false).await?;

    let summary = build_import_summary(std::slice::from_ref(&result));
    let payload = UsageImportPayload {
        imported_count: summary.imported_records,
        platform: result.platform.clone(),
    };
    let entry = usage_import_entry(&payload);
    let persist = should_persist(entry.level, &entry.event_type);
    emit_and_record_monitoring_event(
        &app_handle,
        events::channels::USAGE_IMPORT,
        &payload,
        entry,
        persist,
    )
    .await;

    serde_json::to_value(result).map_err(|e| format!("Serialize error: {e}"))
}

#[tauri::command]
pub async fn import_all_usage_v2(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<Value, String> {
    let mut results = run_llmusage_sync_all(&state, None, false).await?;

    results.sort_by(|left, right| left.platform.cmp(&right.platform));
    let summary = build_import_summary(&results);
    let payload = UsageImportPayload {
        imported_count: summary.imported_records,
        platform: "all".to_string(),
    };
    let entry = usage_import_entry(&payload);
    let persist = should_persist(entry.level, &entry.event_type);
    emit_and_record_monitoring_event(
        &app_handle,
        events::channels::USAGE_IMPORT,
        &payload,
        entry,
        persist,
    )
    .await;

    let response = ImportAllUsageResponse { summary, results };

    serde_json::to_value(response).map_err(|e| format!("Serialize error: {e}"))
}

#[tauri::command]
pub async fn start_usage_import_job_v2(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    platform: Option<String>,
    recent_days: Option<usize>,
    reset_sources: Option<bool>,
) -> Result<Value, String> {
    if let Some(snapshot) = state.get_active_usage_import_job().await {
        return serde_json::to_value(StartUsageImportJobResponse {
            job_id: snapshot.job_id.clone(),
            snapshot,
        })
        .map_err(|e| format!("Serialization error: {e}"));
    }

    let recent_window_days = recent_days.unwrap_or(30).max(1);
    let source = platform
        .as_deref()
        .and_then(crate::llmusage_adapter::parse_source_filter);
    let job_id = format!("llmusage-cli-{}", Uuid::new_v4());
    let options = SyncCommandOptions {
        rebuild: reset_sources.unwrap_or(false),
        recent_days: Some(recent_window_days as u32),
        source,
    };
    let snapshot = UsageImportJobSnapshot::new(
        job_id.clone(),
        platform_scope_label(platform.as_deref()),
        recent_window_days,
    );
    state.insert_usage_import_job(snapshot.clone()).await;

    tauri::async_runtime::spawn(run_usage_import_job(
        app_handle,
        job_id.clone(),
        platform,
        options,
    ));

    serde_json::to_value(StartUsageImportJobResponse { job_id, snapshot })
        .map_err(|e| format!("Serialization error: {e}"))
}

#[tauri::command]
pub async fn get_usage_import_job_status_v2(
    state: State<'_, AppState>,
    job_id: String,
) -> Result<Value, String> {
    let snapshot = state
        .get_usage_import_job(&job_id)
        .await
        .ok_or_else(|| format!("Usage import job '{}' not found", job_id))?;

    serde_json::to_value(snapshot).map_err(|e| format!("Serialization error: {e}"))
}

#[tauri::command]
pub async fn cancel_usage_import_job_v2(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    job_id: String,
) -> Result<Value, String> {
    // 先取出 token 触发子进程退出；llmusage 0.5.3 没有 graceful contract，
    // 这里走 kill 让本机资源立即释放，配合 run_sync_stream 内的 cancel 分支返回 cancelled。
    if let Some(token) = state.take_usage_import_cancel_token(&job_id).await {
        token.cancel();
        tracing::info!(
            ccr_job_id = %job_id,
            "llmusage sync subprocess cancel signal sent (kill)"
        );
    } else {
        tracing::warn!(
            ccr_job_id = %job_id,
            "no cancel token registered for usage import job; only marking local snapshot cancelled"
        );
    }

    let snapshot = state
        .update_usage_import_job(&job_id, |job: &mut UsageImportJobSnapshot| {
            job.mark_cancelled();
        })
        .await
        .ok_or_else(|| format!("Usage import job '{}' not found", job_id))?;

    emit_usage_import_job_snapshot(&app_handle, "usage:job-failed", &snapshot).await;
    serde_json::to_value(snapshot).map_err(|e| format!("Serialization error: {e}"))
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_home_platform_supports_common_aliases() {
        assert_eq!(normalize_home_platform("Claude Code"), Some("claude"));
        assert_eq!(normalize_home_platform("openai-codex"), Some("codex"));
        assert_eq!(normalize_home_platform("gemini-cli"), Some("gemini"));
        assert_eq!(normalize_home_platform("Open Code"), Some("opencode"));
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
                platform: "gemini".into(),
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
}
