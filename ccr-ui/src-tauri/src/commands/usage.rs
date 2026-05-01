//! Usage V2 命令模块，基于 SQLite 查询与导入用量数据。

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Instant;

use ccr_config::Platform;
use ccr_store::{
    ModelPricing, PricingManager,
    sessions::{SessionIndexer, parser::SessionParser},
};
use ccr_types::{ModelRateCatalog, ModelRateOverride, official_model_rate_override_for};
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

#[derive(Debug, Clone)]
struct UsageImportJobFile {
    platform: String,
    path: PathBuf,
    file_size: i64,
    modified_at: Option<std::time::SystemTime>,
}

const HOME_USAGE_PLATFORMS: [&str; 3] = ["claude", "codex", "gemini"];

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
    summary: ccr_db::database::repositories::usage_repo::UsageSummary,
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

fn same_price(left: f64, right: f64) -> bool {
    (left - right).abs() < 0.000_001
}

fn same_optional_price(left: Option<f64>, right: Option<f64>) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => same_price(left, right),
        (None, None) => true,
        _ => false,
    }
}

fn same_model_rate_price(left: &ModelRateOverride, right: &ModelRateOverride) -> bool {
    same_price(left.input_price, right.input_price)
        && same_price(left.output_price, right.output_price)
        && same_optional_price(left.cache_read_price, right.cache_read_price)
        && same_optional_price(left.cache_write_price, right.cache_write_price)
}

fn custom_rate_override(model: String, pricing: ModelPricing) -> Option<ModelRateOverride> {
    let rate = ModelRateOverride {
        model,
        input_price: pricing.input_price,
        output_price: pricing.output_price,
        cache_read_price: pricing.cache_read_price,
        cache_write_price: pricing.cache_write_price,
    };

    official_model_rate_override_for(&rate.model)
        .is_none_or(|official| !same_model_rate_price(&rate, &official))
        .then_some(rate)
}

fn usage_pricing_catalog() -> ModelRateCatalog {
    let manager = match PricingManager::with_default() {
        Ok(manager) => manager,
        Err(error) => {
            tracing::warn!(?error, "Failed to load pricing overrides; using official catalog");
            return ModelRateCatalog::official();
        }
    };

    let overrides = manager
        .export_pricing()
        .into_iter()
        .filter_map(|(model, pricing)| custom_rate_override(model, pricing))
        .collect::<Vec<_>>();

    if overrides.is_empty() {
        ModelRateCatalog::official()
    } else {
        ModelRateCatalog::with_overrides(overrides)
    }
}

fn build_usage_import_service(
    usage_db_pool: ccr_db::database::pool::DbPool,
) -> ccr_db::services::usage_import_service::UsageImportService {
    ccr_db::services::usage_import_service::UsageImportService::with_pool_and_catalog(
        ccr_db::services::usage_import_service::ImportConfig::default(),
        usage_db_pool,
        usage_pricing_catalog(),
    )
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

fn system_time_to_utc(value: std::time::SystemTime) -> chrono::DateTime<Utc> {
    chrono::DateTime::<Utc>::from(value)
}

fn source_requires_import(
    source: &ccr_db::database::repositories::usage_repo::UsageSource,
    file_size: i64,
    modified_at: Option<std::time::SystemTime>,
) -> bool {
    if source.source_state != ccr_db::database::repositories::usage_repo::UsageSourceState::Live {
        return true;
    }

    if source.file_size != Some(file_size) {
        return true;
    }

    let modified_at = modified_at.map(system_time_to_utc);
    source.modified_at != modified_at
}

fn build_usage_import_plan(
    usage_db_pool: ccr_db::database::pool::DbPool,
    platforms: &[String],
    recent_window_days: usize,
) -> Result<(Vec<UsageImportJobFile>, Vec<UsageImportJobFile>), String> {
    let service = build_usage_import_service(usage_db_pool.clone());

    let cutoff = std::time::SystemTime::now()
        .checked_sub(std::time::Duration::from_secs(
            recent_window_days.max(1) as u64 * 24 * 60 * 60,
        ))
        .unwrap_or(std::time::SystemTime::UNIX_EPOCH);

    let mut recent = Vec::new();
    let mut history = Vec::new();

    for platform in platforms {
        let existing_sources = {
            let conn = usage_db_pool.get().map_err(|e| format!("DB error: {e}"))?;
            ccr_db::database::repositories::usage_repo::get_sources_by_platform(&conn, platform)
                .map_err(|e| format!("Source lookup error: {e}"))?
                .into_iter()
                .map(|source| (source.file_path.clone(), source))
                .collect::<std::collections::HashMap<_, _>>()
        };
        let mut seen_paths = Vec::new();

        for path in service.list_usage_files(platform)? {
            let metadata = std::fs::metadata(&path).ok();
            let file_size = metadata.as_ref().map(|meta| meta.len() as i64).unwrap_or(0);
            let modified_at = metadata.as_ref().and_then(|meta| meta.modified().ok());
            let file_path = path.display().to_string();
            seen_paths.push(file_path.clone());

            if let Some(source) = existing_sources.get(&file_path)
                && !source_requires_import(source, file_size, modified_at)
            {
                continue;
            }

            let job_file = UsageImportJobFile {
                platform: platform.clone(),
                path,
                file_size,
                modified_at,
            };

            if modified_at.is_some_and(|modified| modified >= cutoff) {
                recent.push(job_file);
            } else {
                history.push(job_file);
            }
        }

        let conn = usage_db_pool.get().map_err(|e| format!("DB error: {e}"))?;
        ccr_db::database::repositories::usage_repo::mark_sources_missing_by_platform(
            &conn,
            platform,
            &seen_paths,
        )
        .map_err(|e| format!("Source reconcile error: {e}"))?;
    }

    sort_job_files(&mut recent);
    sort_job_files(&mut history);

    if recent.is_empty() && !history.is_empty() {
        let promote_count = history.len().min(12);
        recent.extend(history.drain(0..promote_count));
    }

    Ok((recent, history))
}

fn upsert_usage_history_cursor(
    usage_db_pool: &ccr_db::database::pool::DbPool,
    platform: &str,
    recent_window_days: usize,
    file: Option<&UsageImportJobFile>,
    recent_completed_at: Option<chrono::DateTime<Utc>>,
    history_completed_at: Option<chrono::DateTime<Utc>>,
) -> Result<(), String> {
    let conn = usage_db_pool.get().map_err(|e| format!("DB error: {e}"))?;
    let existing = ccr_db::database::repositories::usage_repo::get_history_cursor(&conn, platform)
        .map_err(|e| format!("History cursor lookup error: {e}"))?;
    let cursor = ccr_db::database::repositories::usage_repo::UsageHistoryCursor {
        platform: platform.to_string(),
        recent_window_days: recent_window_days as i64,
        last_history_file_path: file
            .map(|entry| entry.path.display().to_string())
            .or_else(|| {
                existing
                    .as_ref()
                    .and_then(|cursor| cursor.last_history_file_path.clone())
            }),
        last_history_file_modified_at: file
            .and_then(|entry| entry.modified_at.map(system_time_to_utc))
            .or_else(|| {
                existing
                    .as_ref()
                    .and_then(|cursor| cursor.last_history_file_modified_at)
            }),
        last_history_offset: file.map(|entry| entry.file_size).unwrap_or_else(|| {
            existing
                .as_ref()
                .map(|cursor| cursor.last_history_offset)
                .unwrap_or(0)
        }),
        recent_completed_at: recent_completed_at.or_else(|| {
            existing
                .as_ref()
                .and_then(|cursor| cursor.recent_completed_at)
        }),
        history_completed_at: history_completed_at.or_else(|| {
            existing
                .as_ref()
                .and_then(|cursor| cursor.history_completed_at)
        }),
        updated_at: Utc::now(),
    };

    ccr_db::database::repositories::usage_repo::upsert_history_cursor(&conn, &cursor)
        .map_err(|e| format!("History cursor upsert error: {e}"))
}

async fn process_usage_import_phase(
    app_handle: &AppHandle,
    usage_db_pool: ccr_db::database::pool::DbPool,
    job_id: &str,
    recent_window_days: usize,
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
        let db_pool = usage_db_pool.clone();
        let import_started = Instant::now();
        let import_result = tokio::task::spawn_blocking(move || {
            let service = build_usage_import_service(db_pool);
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

                if matches!(stage, UsageImportJobStage::ImportingHistory) {
                    upsert_usage_history_cursor(
                        &usage_db_pool,
                        &file.platform,
                        recent_window_days,
                        Some(file),
                        None,
                        None,
                    )?;
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

                if matches!(stage, UsageImportJobStage::ImportingHistory) {
                    upsert_usage_history_cursor(
                        &usage_db_pool,
                        &file.platform,
                        recent_window_days,
                        Some(file),
                        None,
                        None,
                    )?;
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
    let usage_db_pool = app_handle.state::<AppState>().usage_db_pool.clone();
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
                let db_pool = usage_db_pool.clone();
                let source_count = tokio::task::spawn_blocking(move || {
                    let service = build_usage_import_service(db_pool);
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
                let db_pool = usage_db_pool.clone();
                let reset_result = tokio::task::spawn_blocking(move || {
                    let service = build_usage_import_service(db_pool);
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
            build_usage_import_plan(usage_db_pool.clone(), &platforms, recent_window_days)?;
        let archive_diagnostics = load_archive_diagnostics(&usage_db_pool, platform.as_deref())?;
        let history_cursor_hit = load_history_cursor_hit(&usage_db_pool, &platforms)?;
        let files_total = recent_files.len() + history_files.len();
        let mut results_by_platform = create_platform_results(&platforms);

        if let Some(snapshot) = app_handle
            .state::<AppState>()
            .update_usage_import_job(&job_id, |job: &mut UsageImportJobSnapshot| {
                job.files_total = files_total;
                job.history_cursor_hit = history_cursor_hit;
                job.live_sources = archive_diagnostics.live_sources as usize;
                job.missing_sources = archive_diagnostics.missing_sources as usize;
                job.deleted_sources = archive_diagnostics.deleted_sources as usize;
                job.updated_at = Utc::now().to_rfc3339();
            })
            .await
        {
            emit_usage_import_job_snapshot(&app_handle, "usage:job-progress", &snapshot).await;
        }

        process_usage_import_phase(
            &app_handle,
            usage_db_pool.clone(),
            &job_id,
            recent_window_days,
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

        for platform_name in &platforms {
            upsert_usage_history_cursor(
                &usage_db_pool,
                platform_name,
                recent_window_days,
                None,
                Some(Utc::now()),
                if history_files.is_empty() {
                    Some(Utc::now())
                } else {
                    None
                },
            )?;
        }

        process_usage_import_phase(
            &app_handle,
            usage_db_pool.clone(),
            &job_id,
            recent_window_days,
            files_total,
            UsageImportJobStage::ImportingHistory,
            &history_files,
            &mut results_by_platform,
        )
        .await?;

        if !history_files.is_empty() {
            for platform_name in &platforms {
                upsert_usage_history_cursor(
                    &usage_db_pool,
                    platform_name,
                    recent_window_days,
                    None,
                    None,
                    Some(Utc::now()),
                )?;
            }
        }

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
                    sync_platform_session_archive(&usage_db_pool, platform)?;
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
    ] {
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

fn load_archive_diagnostics(
    pool: &ccr_db::database::pool::DbPool,
    platform: Option<&str>,
) -> Result<UsageArchiveDiagnostics, String> {
    let conn = pool.get().map_err(|e| format!("DB error: {e}"))?;
    let counts =
        ccr_db::database::repositories::usage_repo::get_source_state_counts(&conn, platform)
            .map_err(|e| format!("Source state query error: {e}"))?;
    let archived_sessions =
        ccr_db::database::repositories::usage_repo::get_session_archive_platform_summaries(
            &conn, &None, &None,
        )
        .map_err(|e| format!("Archived session summary query error: {e}"))?
        .into_iter()
        .map(|item| non_negative_i64(item.session_count))
        .sum();

    let mut recent_completed_at: Option<String> = None;
    let mut history_completed_at: Option<String> = None;
    let cursor_platforms = platform
        .map(|value| vec![value.to_string()])
        .unwrap_or_else(|| {
            HOME_USAGE_PLATFORMS
                .iter()
                .map(|value| value.to_string())
                .collect()
        });

    for cursor_platform in cursor_platforms {
        if let Some(cursor) =
            ccr_db::database::repositories::usage_repo::get_history_cursor(&conn, &cursor_platform)
                .map_err(|e| format!("History cursor query error: {e}"))?
        {
            if let Some(recent) = cursor.recent_completed_at {
                let raw = recent.to_rfc3339();
                if recent_completed_at
                    .as_ref()
                    .is_none_or(|current| raw > *current)
                {
                    recent_completed_at = Some(raw);
                }
            }
            if let Some(history) = cursor.history_completed_at {
                let raw = history.to_rfc3339();
                if history_completed_at
                    .as_ref()
                    .is_none_or(|current| raw > *current)
                {
                    history_completed_at = Some(raw);
                }
            }
        }
    }

    Ok(UsageArchiveDiagnostics {
        archive_root: ccr_db::database::get_usage_archive_db_path()
            .map_err(|e| format!("Archive root error: {e}"))?
            .display()
            .to_string(),
        live_sources: counts.live.max(0) as u64,
        missing_sources: counts.missing.max(0) as u64,
        deleted_sources: counts.deleted_by_user.max(0) as u64,
        archived_sessions,
        recent_completed_at,
        history_completed_at,
    })
}

fn load_history_cursor_hit(
    pool: &ccr_db::database::pool::DbPool,
    platforms: &[String],
) -> Result<bool, String> {
    let conn = pool.get().map_err(|e| format!("DB error: {e}"))?;
    for platform in platforms {
        if ccr_db::database::repositories::usage_repo::get_history_cursor(&conn, platform)
            .map_err(|e| format!("History cursor query error: {e}"))?
            .is_some()
        {
            return Ok(true);
        }
    }

    Ok(false)
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
    let pool = state.usage_db_pool.clone();
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
    let pool = state.usage_db_pool.clone();
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
    let pool = state.usage_db_pool.clone();
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
    let pool = state.usage_db_pool.clone();
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
    let pool = state.usage_db_pool.clone();
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

    let pool = state.usage_db_pool.clone();
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
    let pool = state.usage_db_pool.clone();
    let heatmap_days = heatmap_days.unwrap_or(365).max(1);
    let include_heatmap = include_heatmap.unwrap_or(false);
    let result = tokio::task::spawn_blocking(move || {
        let db_started = Instant::now();
        let conn = pool.get().map_err(|e| format!("DB error: {e}"))?;
        let archive = load_archive_diagnostics(&pool, platform.as_deref())?;

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
            "archive": archive,
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
    let pool = state.usage_db_pool.clone();
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
        let session_snapshot = load_home_session_snapshot(&pool, &start_date, &end_date)?;
        let archive = load_archive_diagnostics(&pool, None)?;
        let has_any_sessions = session_snapshot.has_any_sessions;
        let needs_usage_import = !has_any_usage;
        let needs_session_index = !has_any_sessions && has_any_raw_sessions();

        for (platform_name, session_stats) in &session_snapshot.by_platform {
            if let Some(stats) = usage_snapshot.by_platform.get_mut(platform_name.as_str()) {
                stats.sessions = session_stats.sessions;
            }
        }

        for (date, session_day_stats) in session_snapshot.daily_by_platform {
            let day_entry = usage_snapshot
                .daily_by_platform
                .entry(date)
                .or_insert_with(empty_home_platform_map);
            for (platform_name, session_stats) in session_day_stats {
                if let Some(stats) = day_entry.get_mut(platform_name.as_str()) {
                    stats.sessions = session_stats.sessions;
                }
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
                }
            })
            .collect::<Vec<_>>();

        let total_sessions = session_snapshot.total_sessions;
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
    let usage_db_pool = state.usage_db_pool.clone();
    let result = tokio::task::spawn_blocking(move || {
        let service = build_usage_import_service(usage_db_pool);
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
    state: State<'_, AppState>,
) -> Result<Value, String> {
    use std::sync::Arc;
    use tokio::sync::Semaphore;

    let semaphore = Arc::new(Semaphore::new(2));
    let usage_db_pool = state.usage_db_pool.clone();
    let mut tasks = tokio::task::JoinSet::new();
    for platform in HOME_USAGE_PLATFORMS {
        let sem = Arc::clone(&semaphore);
        let db_pool = usage_db_pool.clone();
        let platform_name = platform.to_string();
        tasks.spawn(async move {
            let _permit = sem
                .acquire_owned()
                .await
                .map_err(|e| (platform_name.clone(), format!("Semaphore error: {e}")))?;

            let import_platform = platform_name.clone();
            tokio::task::spawn_blocking(move || {
                let service = build_usage_import_service(db_pool);
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
        assert_eq!(normalize_home_platform("legacy-cli"), None);
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

    #[test]
    fn pricing_defaults_are_not_treated_as_overrides() {
        let pricing = ModelPricing {
            model: "claude-opus-4.6".to_string(),
            input_price: 5.0,
            output_price: 25.0,
            cache_read_price: Some(0.5),
            cache_write_price: Some(6.25),
        };

        assert!(custom_rate_override("anthropic/claude-opus-4.6".to_string(), pricing).is_none());
    }

    #[test]
    fn custom_pricing_is_kept_as_override() {
        let pricing = ModelPricing {
            model: "gpt-5.4".to_string(),
            input_price: 9.0,
            output_price: 18.0,
            cache_read_price: Some(0.9),
            cache_write_price: Some(9.0),
        };

        let override_rate = custom_rate_override("gpt-5.4".to_string(), pricing).unwrap();

        assert_eq!(override_rate.model, "gpt-5.4");
        assert_eq!(override_rate.input_price, 9.0);
        assert_eq!(override_rate.output_price, 18.0);
    }
}
