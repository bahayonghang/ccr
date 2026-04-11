//! Usage V2 命令模块，基于 SQLite 查询与导入用量数据。

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Instant;

use ccr_config::Platform;
use ccr_store::sessions::{SessionFilter, SessionIndexer, SessionSummary, parser::SessionParser};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager, State};
use uuid::Uuid;

use crate::events::{self, UsageImportPayload};
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
    pub page: Option<i64>,
    #[serde(alias = "pageSize")]
    pub page_size: Option<i64>,
    pub cursor: Option<String>,
    #[serde(alias = "includeTotal")]
    pub include_total: Option<bool>,
    pub mode: Option<UsageLogsMode>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PaginatedLogsV2 {
    pub records: Vec<ccr_db::database::repositories::usage_repo::UsageRecord>,
    pub total: Option<i64>,
    pub page: i64,
    pub page_size: i64,
    pub next_cursor: Option<String>,
    pub mode: UsageLogsMode,
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

#[derive(Debug, Clone)]
struct UsageImportJobFile {
    platform: String,
    path: PathBuf,
    modified_at: Option<std::time::SystemTime>,
}

const HOME_USAGE_PLATFORMS: [&str; 4] = ["claude", "codex", "gemini", "qwen"];

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
    pub qwen: HomeOverviewPlatformStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomeUsageOverviewResponse {
    pub summary: HomeOverviewSummary,
    pub by_platform: BTreeMap<String, HomeOverviewPlatformStats>,
    pub series: Vec<HomeOverviewSeriesItem>,
    pub bootstrap: HomeOverviewBootstrap,
    pub empty_reason: Option<String>,
    pub last_updated: String,
}

struct HomeUsageSnapshot {
    summary: ccr_db::database::repositories::usage_repo::UsageSummary,
    active_days: u64,
    by_platform: BTreeMap<String, HomeOverviewPlatformStats>,
    daily_by_platform: BTreeMap<String, BTreeMap<String, HomeOverviewPlatformStats>>,
}

fn list_home_sessions(
    indexer: &SessionIndexer,
    from_date: Option<chrono::DateTime<Utc>>,
    to_date: Option<chrono::DateTime<Utc>>,
    limit: Option<usize>,
) -> Vec<SessionSummary> {
    let filter = SessionFilter {
        platform: None,
        from_date,
        to_date,
        cwd_prefix: None,
        limit,
        offset: None,
        today_only: false,
    };

    indexer.list(filter).unwrap_or_default()
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

fn normalize_import_result(
    result: ccr_db::services::usage_import_service::ImportResult,
) -> UsageImportResultV2 {
    UsageImportResultV2 {
        platform: result.platform,
        files_processed: result.files_processed,
        records_imported: result.records_imported,
        records_skipped: result.records_skipped,
        duration_ms: result.duration_ms,
        completed: result.completed,
        error: None,
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

fn session_index_platforms() -> [Platform; 4] {
    [
        Platform::Claude,
        Platform::Codex,
        Platform::Gemini,
        Platform::Qwen,
    ]
}

fn session_index_platform_label(platform: Platform) -> &'static str {
    match platform {
        Platform::Claude => "claude",
        Platform::Codex => "codex",
        Platform::Gemini => "gemini",
        Platform::Qwen => "qwen",
        Platform::Droid => "droid",
    }
}

fn session_index_file_count(platform: Platform) -> u64 {
    let Some(session_dir) = SessionParser::get_platform_session_dir(&platform)
    else {
        return 0;
    };

    SessionParser::scan_directory(&session_dir, platform)
        .map(|files| files.len() as u64)
        .unwrap_or(0)
}

fn platform_scope_label(platform: Option<&str>) -> String {
    platform.unwrap_or("all").to_string()
}

fn create_platform_results(platforms: &[String]) -> BTreeMap<String, UsageImportResultV2> {
    platforms
        .iter()
        .map(|platform| {
            (
                platform.clone(),
                UsageImportResultV2 {
                    platform: platform.clone(),
                    files_processed: 0,
                    records_imported: 0,
                    records_skipped: 0,
                    duration_ms: 0,
                    completed: true,
                    error: None,
                },
            )
        })
        .collect()
}

fn sort_job_files(files: &mut [UsageImportJobFile]) {
    files.sort_by(|left, right| {
        right
            .modified_at
            .cmp(&left.modified_at)
            .then_with(|| left.path.cmp(&right.path))
    });
}

fn build_usage_import_plan(
    platforms: &[String],
    recent_window_days: usize,
) -> Result<(Vec<UsageImportJobFile>, Vec<UsageImportJobFile>), String> {
    let service = ccr_db::services::usage_import_service::UsageImportService::new(
        ccr_db::services::usage_import_service::ImportConfig::default(),
    );

    let cutoff = std::time::SystemTime::now()
        .checked_sub(std::time::Duration::from_secs(
            recent_window_days.max(1) as u64 * 24 * 60 * 60,
        ))
        .unwrap_or(std::time::SystemTime::UNIX_EPOCH);

    let mut recent = Vec::new();
    let mut history = Vec::new();

    for platform in platforms {
        for path in service.list_usage_files(platform)? {
            let modified_at = std::fs::metadata(&path)
                .and_then(|meta| meta.modified())
                .ok();
            let job_file = UsageImportJobFile {
                platform: platform.clone(),
                path,
                modified_at,
            };

            if modified_at.is_some_and(|modified| modified >= cutoff) {
                recent.push(job_file);
            } else {
                history.push(job_file);
            }
        }
    }

    sort_job_files(&mut recent);
    sort_job_files(&mut history);

    if recent.is_empty() && !history.is_empty() {
        let promote_count = history.len().min(12);
        recent.extend(history.drain(0..promote_count));
    }

    Ok((recent, history))
}

async fn process_usage_import_phase(
    app_handle: &AppHandle,
    job_id: &str,
    files_total: usize,
    stage: UsageImportJobStage,
    files: &[UsageImportJobFile],
    results_by_platform: &mut BTreeMap<String, UsageImportResultV2>,
) -> Result<(), String> {
    for file in files {
        let display_path = file.path.display().to_string();

        if let Some(snapshot) = app_handle
            .state::<AppState>()
            .update_usage_import_job(job_id, |job: &mut UsageImportJobSnapshot| {
                job.mark_running(stage, files_total, Some(display_path.clone()));
            })
            .await
        {
            emit_usage_import_job_snapshot(app_handle, "usage:job-progress", &snapshot).await;
        }

        let platform = file.platform.clone();
        let path = file.path.clone();
        let import_started = Instant::now();
        let import_result = tokio::task::spawn_blocking(move || {
            let service = ccr_db::services::usage_import_service::UsageImportService::new(
                ccr_db::services::usage_import_service::ImportConfig::default(),
            );
            service.import_file_path(&platform, &path)
        })
        .await
        .map_err(|error| format!("Import task join error: {error}"))?;

        match import_result {
            Ok((imported, skipped)) => {
                if let Some(platform_result) = results_by_platform.get_mut(&file.platform) {
                    platform_result.files_processed += 1;
                    platform_result.records_imported += imported;
                    platform_result.records_skipped += skipped;
                    platform_result.duration_ms += import_started.elapsed().as_millis() as u64;
                }

                if let Some(snapshot) = app_handle
                    .state::<AppState>()
                    .update_usage_import_job(job_id, |job: &mut UsageImportJobSnapshot| {
                        job.record_file_result(Some(display_path.clone()), imported, skipped);
                    })
                    .await
                {
                    emit_usage_import_job_snapshot(app_handle, "usage:job-progress", &snapshot)
                        .await;
                }
            }
            Err(error) => {
                if let Some(platform_result) = results_by_platform.get_mut(&file.platform) {
                    platform_result.files_processed += 1;
                    platform_result.duration_ms += import_started.elapsed().as_millis() as u64;
                    platform_result.completed = false;
                    platform_result.error = Some(error.clone());
                }

                if let Some(snapshot) = app_handle
                    .state::<AppState>()
                    .update_usage_import_job(job_id, |job: &mut UsageImportJobSnapshot| {
                        job.record_file_result(Some(display_path.clone()), 0, 0);
                        job.push_warning(format!("{}: {}", display_path, error));
                    })
                    .await
                {
                    emit_usage_import_job_snapshot(app_handle, "usage:job-progress", &snapshot)
                        .await;
                }
            }
        }
    }

    Ok(())
}

async fn run_usage_import_job(
    app_handle: AppHandle,
    job_id: String,
    platform: Option<String>,
    recent_window_days: usize,
    reset_sources: bool,
) {
    let platforms = match platform.as_deref() {
        Some(value) => vec![value.to_string()],
        None => HOME_USAGE_PLATFORMS
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
    };

    let execution = async {
        if reset_sources {
            for platform_name in &platforms {
                let platform_name = platform_name.clone();
                let source_target = platform_name.clone();
                let source_count = tokio::task::spawn_blocking(move || {
                    let service = ccr_db::services::usage_import_service::UsageImportService::new(
                        ccr_db::services::usage_import_service::ImportConfig::default(),
                    );
                    service
                        .list_usage_files(&source_target)
                        .map(|files| files.len())
                })
                .await
                .map_err(|error| format!("Repair preflight join error: {error}"))??;

                if source_count == 0 {
                    return Err(format!(
                        "No usage source files found for {}. Refusing to reset imported history.",
                        platform_name
                    ));
                }

                let reset_target = platform_name.clone();
                let reset_result = tokio::task::spawn_blocking(move || {
                    let service = ccr_db::services::usage_import_service::UsageImportService::new(
                        ccr_db::services::usage_import_service::ImportConfig::default(),
                    );
                    service.reset_platform_sources(&reset_target)
                })
                .await
                .map_err(|error| format!("Repair task join error: {error}"))??;

                if let Some(snapshot) = app_handle
                    .state::<AppState>()
                    .update_usage_import_job(&job_id, |job: &mut UsageImportJobSnapshot| {
                        job.push_warning(format!(
                            "Reset {} sources and {} records for {} before re-import",
                            reset_result.0, reset_result.1, platform_name
                        ));
                    })
                    .await
                {
                    emit_usage_import_job_snapshot(&app_handle, "usage:job-progress", &snapshot)
                        .await;
                }
            }
        }

        let (recent_files, history_files) =
            build_usage_import_plan(&platforms, recent_window_days)?;
        let files_total = recent_files.len() + history_files.len();
        let mut results_by_platform = create_platform_results(&platforms);

        if let Some(snapshot) = app_handle
            .state::<AppState>()
            .update_usage_import_job(&job_id, |job: &mut UsageImportJobSnapshot| {
                job.files_total = files_total;
                job.updated_at = Utc::now().to_rfc3339();
            })
            .await
        {
            emit_usage_import_job_snapshot(&app_handle, "usage:job-progress", &snapshot).await;
        }

        process_usage_import_phase(
            &app_handle,
            &job_id,
            files_total,
            UsageImportJobStage::ImportingRecent,
            &recent_files,
            &mut results_by_platform,
        )
        .await?;

        if let Some(snapshot) = app_handle
            .state::<AppState>()
            .update_usage_import_job(&job_id, |job: &mut UsageImportJobSnapshot| {
                job.mark_recent_ready(!history_files.is_empty());
            })
            .await
        {
            emit_usage_import_job_snapshot(&app_handle, "usage:job-recent-ready", &snapshot).await;
        }

        process_usage_import_phase(
            &app_handle,
            &job_id,
            files_total,
            UsageImportJobStage::ImportingHistory,
            &history_files,
            &mut results_by_platform,
        )
        .await?;

        let results = results_by_platform.into_values().collect::<Vec<_>>();
        let summary = build_import_summary(&results);

        let final_snapshot = app_handle
            .state::<AppState>()
            .update_usage_import_job(&job_id, |job: &mut UsageImportJobSnapshot| {
                job.mark_finished(results.clone(), summary.clone());
            })
            .await
            .ok_or_else(|| format!("Usage import job '{}' not found", job_id))?;

        let payload = UsageImportPayload {
            imported_count: summary.imported_records,
            platform: platform_scope_label(platform.as_deref()),
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

        emit_usage_import_job_snapshot(&app_handle, "usage:job-finished", &final_snapshot).await;
        Ok::<(), String>(())
    };

    if let Err(error) = execution.await {
        tracing::error!(job_id = %job_id, ?error, "Usage import job failed");
        if let Some(snapshot) = app_handle
            .state::<AppState>()
            .update_usage_import_job(&job_id, |job: &mut UsageImportJobSnapshot| {
                job.mark_failed(error.clone())
            })
            .await
        {
            emit_usage_import_job_snapshot(&app_handle, "usage:job-failed", &snapshot).await;
        }
    }
}

async fn run_session_index_job(app_handle: AppHandle, job_id: String) {
    let platforms = session_index_platforms();

    let execution = async {
        let files_total: u64 = platforms.iter().copied().map(session_index_file_count).sum();

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

        emit_session_index_job_snapshot(&app_handle, "usage:session-index-finished", &final_snapshot)
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

fn normalize_home_platform(raw: &str) -> Option<&'static str> {
    match raw.trim().to_lowercase().as_str() {
        "claude" | "claude-code" | "claude code" => Some("claude"),
        "codex" | "openai-codex" | "openai codex" => Some("codex"),
        "gemini" | "gemini-cli" | "gemini cli" | "google-gemini" | "google gemini" => {
            Some("gemini")
        }
        "qwen" | "qwen-cli" | "qwen cli" | "qwen-code" | "qwen code" | "alibaba-qwen"
        | "alibaba qwen" => Some("qwen"),
        _ => None,
    }
}

fn non_negative_i64(value: i64) -> u64 {
    value.max(0) as u64
}

fn build_home_date_range(days: usize) -> Vec<String> {
    let safe_days = days.max(1);
    let end = Utc::now().date_naive();
    let start = end - Duration::days((safe_days - 1) as i64);

    (0..safe_days)
        .map(|offset| {
            (start + Duration::days(offset as i64))
                .format("%Y-%m-%d")
                .to_string()
        })
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
    for platform in [
        Platform::Claude,
        Platform::Codex,
        Platform::Gemini,
        Platform::Qwen,
    ] {
        let Some(session_dir) =
            SessionParser::get_platform_session_dir(&platform)
        else {
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
    pool: &ccr_db::database::pool::DbPool,
    start_date: &str,
    end_date: &str,
) -> Result<HomeUsageSnapshot, String> {
    let conn = pool.get().map_err(|e| format!("DB error: {e}"))?;
    let start = Some(start_date.to_string());
    let end = Some(end_date.to_string());
    let summary =
        ccr_db::database::repositories::usage_repo::get_usage_summary(&conn, &None, &start, &end)
            .map_err(|e| format!("Summary query error: {e}"))?;

    let trends =
        ccr_db::database::repositories::usage_repo::get_daily_trends(&conn, &None, &start, &end)
            .map_err(|e| format!("Trend query error: {e}"))?;
    let platform_summaries =
        ccr_db::database::repositories::usage_repo::get_platform_summaries(&conn, &start, &end)
            .map_err(|e| format!("Platform summary query error: {e}"))?;
    let platform_trends =
        ccr_db::database::repositories::usage_repo::get_daily_platform_trends(&conn, &start, &end)
            .map_err(|e| format!("Platform trend query error: {e}"))?;

    let mut by_platform = empty_home_platform_map();
    let mut daily_by_platform = BTreeMap::new();

    for platform_summary in platform_summaries {
        if let Some(stats) = by_platform.get_mut(platform_summary.platform.as_str()) {
            stats.requests = non_negative_i64(platform_summary.request_count);
            stats.tokens = non_negative_i64(platform_summary.total_tokens);
        }
    }

    for trend in platform_trends {
        let day_entry = daily_by_platform
            .entry(trend.date.clone())
            .or_insert_with(empty_home_platform_map);
        if let Some(stats) = day_entry.get_mut(trend.platform.as_str()) {
            stats.requests = non_negative_i64(trend.request_count);
            stats.tokens = non_negative_i64(trend.input_tokens + trend.output_tokens);
        }
    }

    Ok(HomeUsageSnapshot {
        summary,
        active_days: trends
            .iter()
            .filter(|trend| trend.request_count > 0)
            .count() as u64,
        by_platform,
        daily_by_platform,
    })
}

fn load_home_usage_presence(pool: &ccr_db::database::pool::DbPool) -> Result<bool, String> {
    let conn = pool.get().map_err(|e| format!("DB error: {e}"))?;
    let summary =
        ccr_db::database::repositories::usage_repo::get_usage_summary(&conn, &None, &None, &None)
            .map_err(|e| format!("Presence query error: {e}"))?;

    Ok(summary.total_requests > 0)
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
    let pool = state.db_pool.clone();
    let result = tokio::task::spawn_blocking(move || {
        let db_started = Instant::now();
        let conn = pool.get().map_err(|e| format!("DB error: {e}"))?;
        let summary = ccr_db::database::repositories::usage_repo::get_usage_summary(
            &conn,
            &platform,
            &start_date,
            &end_date,
        )
        .map_err(|e| format!("Query error: {e}"))?;
        Ok::<_, String>((summary, elapsed_ms(db_started)))
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
    let pool = state.db_pool.clone();
    let result = tokio::task::spawn_blocking(move || {
        let db_started = Instant::now();
        let conn = pool.get().map_err(|e| format!("DB error: {e}"))?;
        let trends = ccr_db::database::repositories::usage_repo::get_daily_trends(
            &conn,
            &platform,
            &start_date,
            &end_date,
        )
        .map_err(|e| format!("Query error: {e}"))?;
        Ok::<_, String>((trends, elapsed_ms(db_started)))
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
    let pool = state.db_pool.clone();
    let result = tokio::task::spawn_blocking(move || {
        let db_started = Instant::now();
        let conn = pool.get().map_err(|e| format!("DB error: {e}"))?;
        let stats = ccr_db::database::repositories::usage_repo::get_model_stats(
            &conn,
            &platform,
            &start_date,
            &end_date,
        )
        .map_err(|e| format!("Query error: {e}"))?;
        Ok::<_, String>((stats, elapsed_ms(db_started)))
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
    let pool = state.db_pool.clone();
    let result = tokio::task::spawn_blocking(move || {
        let db_started = Instant::now();
        let conn = pool.get().map_err(|e| format!("DB error: {e}"))?;
        let stats = ccr_db::database::repositories::usage_repo::get_project_stats(
            &conn,
            &platform,
            &start_date,
            &end_date,
        )
        .map_err(|e| format!("Query error: {e}"))?;
        Ok::<_, String>((stats, elapsed_ms(db_started)))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?;

    record_command_duration(&state, command_started);
    let (stats, db_ms) = result?;
    record_db_duration(&state, db_ms);
    serde_json::to_value(stats).map_err(|e| format!("Serialize error: {e}"))
}

/// 获取热力图数据（V2，来自 SQLite usage_daily_agg）
#[tauri::command]
pub async fn get_usage_heatmap_v2(
    state: State<'_, AppState>,
    platform: Option<String>,
    days: Option<i64>,
) -> Result<Value, String> {
    let command_started = Instant::now();
    let pool = state.db_pool.clone();
    let days = days.unwrap_or(365).max(1);

    let result = tokio::task::spawn_blocking(move || {
        let db_started = Instant::now();
        let conn = pool.get().map_err(|e| format!("DB error: {e}"))?;
        let heatmap =
            ccr_db::database::repositories::usage_repo::get_heatmap_data(&conn, &platform, days)
                .map_err(|e| format!("Query error: {e}"))?;

        Ok::<_, String>((
            serde_json::json!({
                "data": heatmap,
            }),
            elapsed_ms(db_started),
        ))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?;

    record_command_duration(&state, command_started);
    let (heatmap, db_ms) = result?;
    record_db_duration(&state, db_ms);
    Ok(heatmap)
}

/// 获取用量日志列表，支持游标与分页两种模式
#[tauri::command]
pub async fn get_usage_logs_v2(
    state: State<'_, AppState>,
    query: UsageLogsQuery,
) -> Result<Value, String> {
    let command_started = Instant::now();

    let mode = query.mode.unwrap_or_default();
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(50).clamp(1, 500);
    let include_total = query
        .include_total
        .unwrap_or(matches!(mode, UsageLogsMode::Offset));

    let pool = state.db_pool.clone();
    let platform = query.platform.clone();
    let model = query.model.clone();
    let start_date = query.start_date.clone();
    let end_date = query.end_date.clone();
    let cursor = query.cursor.clone();

    let result = tokio::task::spawn_blocking(move || {
        let db_started = Instant::now();
        let conn = pool.get().map_err(|e| format!("DB error: {e}"))?;

        let logs = match mode {
            UsageLogsMode::Cursor => {
                ccr_db::database::repositories::usage_repo::get_logs_by_cursor(
                    &conn,
                    &platform,
                    &start_date,
                    &end_date,
                    page_size,
                    &model,
                    &cursor,
                    include_total,
                )
            }
            UsageLogsMode::Offset => {
                ccr_db::database::repositories::usage_repo::get_paginated_logs(
                    &conn,
                    &platform,
                    &start_date,
                    &end_date,
                    page,
                    page_size,
                    &model,
                    include_total,
                )
            }
        }
        .map_err(|e| format!("Query error: {e}"))?;

        Ok::<_, String>((logs, elapsed_ms(db_started)))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?;

    record_command_duration(&state, command_started);
    let (logs, db_ms) = result?;
    record_db_duration(&state, db_ms);

    let normalized = PaginatedLogsV2 {
        records: logs.records,
        total: logs.total,
        page: if matches!(mode, UsageLogsMode::Cursor) {
            page
        } else {
            logs.page
        },
        page_size: logs.page_size,
        next_cursor: logs.next_cursor,
        mode,
    };
    serde_json::to_value(normalized).map_err(|e| format!("Serialize error: {e}"))
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
    let pool = state.db_pool.clone();
    let heatmap_days = heatmap_days.unwrap_or(365).max(1);
    let include_heatmap = include_heatmap.unwrap_or(false);
    let result = tokio::task::spawn_blocking(move || {
        let db_started = Instant::now();
        let conn = pool.get().map_err(|e| format!("DB error: {e}"))?;

        let summary = ccr_db::database::repositories::usage_repo::get_usage_summary(
            &conn,
            &platform,
            &start_date,
            &end_date,
        )
        .map_err(|e| format!("Summary query error: {e}"))?;

        let trends = ccr_db::database::repositories::usage_repo::get_daily_trends(
            &conn,
            &platform,
            &start_date,
            &end_date,
        )
        .map_err(|e| format!("Trends query error: {e}"))?;

        let by_model = ccr_db::database::repositories::usage_repo::get_model_stats(
            &conn,
            &platform,
            &start_date,
            &end_date,
        )
        .map_err(|e| format!("Model stats query error: {e}"))?;

        let by_project = ccr_db::database::repositories::usage_repo::get_project_stats(
            &conn,
            &platform,
            &start_date,
            &end_date,
        )
        .map_err(|e| format!("Project stats query error: {e}"))?;

        let heatmap = if include_heatmap {
            Some(
                ccr_db::database::repositories::usage_repo::get_heatmap_data(
                    &conn,
                    &platform,
                    heatmap_days,
                )
                .map_err(|e| format!("Heatmap query error: {e}"))?,
            )
        } else {
            None
        };

        Ok::<Value, String>(serde_json::json!({
            "summary": summary,
            "trends": trends,
            "model_stats": by_model,
            "project_stats": by_project,
            "heatmap": heatmap.map(|data| serde_json::json!({ "data": data })),
            "generated_at": chrono::Utc::now().to_rfc3339(),
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

/// 获取首页工作区概览数据，统一 usage + session 统计链路。
#[tauri::command]
pub async fn get_home_usage_overview_v2(
    state: State<'_, AppState>,
    days: Option<usize>,
) -> Result<Value, String> {
    let command_started = Instant::now();
    let pool = state.db_pool.clone();
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
        let end_day = Utc::now().date_naive();
        let start_day = end_day - Duration::days((days - 1) as i64);
        let start_date = start_day.format("%Y-%m-%d").to_string();
        let end_date = end_day.format("%Y-%m-%d").to_string();
        let has_any_usage = load_home_usage_presence(&pool)?;
        let mut usage_snapshot = load_home_usage_snapshot(&pool, &start_date, &end_date)?;

        let session_start = start_day
            .and_hms_opt(0, 0, 0)
            .ok_or_else(|| "Invalid session start date".to_string())?
            .and_utc();
        let session_end = end_day
            .and_hms_opt(23, 59, 59)
            .ok_or_else(|| "Invalid session end date".to_string())?
            .and_utc();

        let mut sessions: Vec<SessionSummary> = Vec::new();
        let mut has_any_sessions = false;
        if let Ok(indexer) = SessionIndexer::new() {
            has_any_sessions = !list_home_sessions(&indexer, None, None, Some(1)).is_empty();
            if has_any_sessions {
                sessions =
                    list_home_sessions(&indexer, Some(session_start), Some(session_end), None);
            }
        }
        let needs_usage_import = !has_any_usage;
        let needs_session_index = !has_any_sessions && has_any_raw_sessions();

        for session in &sessions {
            let platform_name = session.platform.to_string();
            let Some(platform) = normalize_home_platform(&platform_name) else {
                continue;
            };
            let date = session.created_at.format("%Y-%m-%d").to_string();

            if let Some(stats) = usage_snapshot.by_platform.get_mut(platform) {
                stats.sessions += 1;
            }

            let day_entry = usage_snapshot
                .daily_by_platform
                .entry(date)
                .or_insert_with(empty_home_platform_map);
            if let Some(stats) = day_entry.get_mut(platform) {
                stats.sessions += 1;
            }
        }

        let date_range = build_home_date_range(days);
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
                    qwen: day_stats.remove("qwen").unwrap_or_default(),
                }
            })
            .collect::<Vec<_>>();

        let total_sessions = sessions.len() as u64;
        let total_requests = non_negative_i64(usage_snapshot.summary.total_requests);
        let total_tokens = non_negative_i64(
            usage_snapshot.summary.total_input_tokens + usage_snapshot.summary.total_output_tokens,
        );
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
    _state: State<'_, AppState>,
    platform: String,
) -> Result<Value, String> {
    let result = tokio::task::spawn_blocking(move || {
        let service = ccr_db::services::usage_import_service::UsageImportService::new(
            ccr_db::services::usage_import_service::ImportConfig::default(),
        );
        service
            .import_platform(&platform)
            .map_err(|e| format!("Import error: {e}"))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))??;

    let payload = UsageImportPayload {
        imported_count: result.records_imported,
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

    serde_json::to_value(normalize_import_result(result))
        .map_err(|e| format!("Serialize error: {e}"))
}
#[tauri::command]
pub async fn import_all_usage_v2(
    app_handle: tauri::AppHandle,
    _state: State<'_, AppState>,
) -> Result<Value, String> {
    use std::sync::Arc;
    use tokio::sync::Semaphore;

    let semaphore = Arc::new(Semaphore::new(2));
    let mut tasks = tokio::task::JoinSet::new();
    for platform in HOME_USAGE_PLATFORMS {
        let sem = Arc::clone(&semaphore);
        let platform_name = platform.to_string();
        tasks.spawn(async move {
            let _permit = sem
                .acquire_owned()
                .await
                .map_err(|e| (platform_name.clone(), format!("Semaphore error: {e}")))?;

            let import_platform = platform_name.clone();
            tokio::task::spawn_blocking(move || {
                let service = ccr_db::services::usage_import_service::UsageImportService::new(
                    ccr_db::services::usage_import_service::ImportConfig::default(),
                );
                service
                    .import_platform(&import_platform)
                    .map(normalize_import_result)
                    .map_err(|e| (import_platform, e))
            })
            .await
            .map_err(|e| (platform_name.clone(), format!("Task join error: {e}")))?
            .map_err(|(platform, error)| (platform, error.to_string()))
        });
    }

    let mut results: Vec<UsageImportResultV2> = Vec::new();
    while let Some(result) = tasks.join_next().await {
        match result {
            Ok(Ok(import_result)) => {
                let payload = UsageImportPayload {
                    imported_count: import_result.records_imported,
                    platform: import_result.platform.clone(),
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

                results.push(import_result);
            }
            Ok(Err((platform, error))) => {
                results.push(UsageImportResultV2 {
                    platform,
                    files_processed: 0,
                    records_imported: 0,
                    records_skipped: 0,
                    duration_ms: 0,
                    completed: false,
                    error: Some(error),
                });
            }
            Err(e) => {
                results.push(UsageImportResultV2 {
                    platform: "unknown".to_string(),
                    files_processed: 0,
                    records_imported: 0,
                    records_skipped: 0,
                    duration_ms: 0,
                    completed: false,
                    error: Some(format!("Join error: {e}")),
                });
            }
        }
    }

    results.sort_by(|left, right| left.platform.cmp(&right.platform));
    let response = ImportAllUsageResponse {
        summary: build_import_summary(&results),
        results,
    };

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
    let job_id = format!("usage-import-{}", Uuid::new_v4());
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
        recent_window_days,
        reset_sources.unwrap_or(false),
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
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_home_platform_supports_common_aliases() {
        assert_eq!(normalize_home_platform("Claude Code"), Some("claude"));
        assert_eq!(normalize_home_platform("openai-codex"), Some("codex"));
        assert_eq!(normalize_home_platform("gemini-cli"), Some("gemini"));
        assert_eq!(normalize_home_platform("qwen-cli"), Some("qwen"));
        assert_eq!(normalize_home_platform("unknown"), None);
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
            },
            UsageImportResultV2 {
                platform: "codex".into(),
                files_processed: 0,
                records_imported: 0,
                records_skipped: 0,
                duration_ms: 0,
                completed: false,
                error: Some("boom".into()),
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
            },
            UsageImportResultV2 {
                platform: "gemini".into(),
                files_processed: 0,
                records_imported: 0,
                records_skipped: 0,
                duration_ms: 4,
                completed: true,
                error: None,
            },
        ]);

        assert_eq!(summary.success_count, 2);
        assert_eq!(summary.failure_count, 0);
        assert_eq!(summary.imported_records, 0);
        assert_eq!(summary.processed_files, 0);
        assert!(!summary.has_partial);
    }
}
