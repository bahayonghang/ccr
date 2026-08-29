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
//!
//! wire DTO 与 State-free 业务编排已下沉 [`crate::services::usage`]；本模块只保留
//! 计时/metrics、State 提取、spawn_blocking 调度、缓存与 job 事件桥接。

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use ccr_config::Platform;
use ccr_store::sessions::{SessionIndexer, parser::SessionParser};
use chrono::Utc;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};
use uuid::Uuid;

use crate::events::{self, UsageImportPayload};
use crate::llmusage_adapter::queries::{
    DailyTrendDto, HeatmapResponseDto, ModelStatDto, PaginatedLogsDto, ProjectStatDto,
    ProviderBreakdownDto, UsageSummaryDto,
};
use crate::llmusage_adapter::{
    CapabilityReport, JobEvent, LlmusageRuntime, SyncCommandOptions, SyncSummaryEvent,
    canonical_source_id, platform_scope_label,
};
use crate::monitoring::{emit_and_record_monitoring_event, should_persist, usage_import_entry};
use crate::services;
use crate::session_index_jobs::{SessionIndexJobSnapshot, SessionIndexJobStatus};
use crate::state::{AppState, CacheFillRegistration};
use crate::usage_jobs::{UsageImportJobSnapshot, UsageImportJobStage, UsageImportJobStatus};

// wire 类型 / 纯函数已随服务层下沉，此处 pub use 保住既有 `crate::commands::usage::*`
// 引用路径（usage_jobs.rs 等外部消费点文本不变）。部分类型在本模块内不再被点名，
// 仅作路径兼容 re-export，故显式 allow。
#[allow(unused_imports)]
pub use crate::services::usage::{
    HomeOverviewBootstrap, HomeOverviewPlatformStats, HomeOverviewSeriesItem, HomeOverviewSummary,
    HomeUsageOverviewResponse, ImportAllUsageResponse, StartSessionIndexJobResponse,
    StartUsageImportJobResponse, UsageArchiveDiagnostics, UsageDashboardResponse,
    UsageDrilldownProjection, UsageFreshnessProjection, UsageFreshnessState, UsageImportResultV2,
    UsageImportSummary, UsageLogsMode, UsageLogsQuery, UsageReadinessProjection,
    UsageReadinessState, UsageSnapshotProjection, UsageSourceHealth, UsageSourceHealthState,
    build_import_summary, collect_llmusage_sync_results, default_import_results, elapsed_ms,
    source_import_result,
};
use crate::services::usage::{USAGE_SNAPSHOT_CACHE_PREFIX, USAGE_SNAPSHOT_CACHE_TTL_SECS};

#[derive(Debug, Clone, Serialize)]
pub struct UsageSnapshotUpdatedPayload {
    pub reason: String,
    pub platform_scope: String,
    pub job_id: Option<String>,
    pub imported_records: usize,
    pub generated_at: String,
}

#[ccr_tauri_command_macros::command]
pub async fn get_usage_capabilities_v2(
    state: State<'_, AppState>,
) -> Result<CapabilityReport, String> {
    Ok(state.llmusage.capabilities().await)
}

fn record_command_duration(state: &AppState, command_started: Instant) {
    state.record_command_duration_ms(elapsed_ms(command_started));
}

fn record_db_duration(state: &AppState, db_ms: f64) {
    state.record_db_query_duration_ms(db_ms);
}

fn usage_dashboard_cache_key(
    platform: Option<&str>,
    provider: Option<&str>,
    start_date: Option<&str>,
    end_date: Option<&str>,
    heatmap_days: u32,
    include_heatmap: bool,
) -> String {
    format!(
        "{}dashboard:platform={}:provider={}:start={}:end={}:heatmap_days={}:include_heatmap={}",
        USAGE_SNAPSHOT_CACHE_PREFIX,
        platform.unwrap_or("all"),
        provider.unwrap_or("all"),
        start_date.unwrap_or(""),
        end_date.unwrap_or(""),
        heatmap_days,
        include_heatmap
    )
}

fn provider_activation_map_path() -> Option<PathBuf> {
    let root = ccr_config::managers::provider_activation::default_ccr_root()?;
    let path = ccr_config::managers::provider_activation::activation_log_path(&root);
    // llmusage treats an explicit --provider-map as strict, so avoid turning a
    // not-yet-created activation log into a sync failure.
    path.is_file().then_some(path)
}

async fn invalidate_usage_snapshot_cache(app_handle: &AppHandle) {
    app_handle
        .state::<AppState>()
        .cache_remove_prefix(USAGE_SNAPSHOT_CACHE_PREFIX)
        .await;
}

async fn emit_usage_snapshot_updated(
    app_handle: &AppHandle,
    reason: impl Into<String>,
    platform: Option<&str>,
    job_id: Option<String>,
    imported_records: usize,
) {
    let payload = UsageSnapshotUpdatedPayload {
        reason: reason.into(),
        platform_scope: platform_scope_label(platform),
        job_id,
        imported_records,
        generated_at: Utc::now().to_rfc3339(),
    };

    if let Err(error) = app_handle.emit("usage:snapshot-updated", payload) {
        tracing::warn!(?error, "Failed to emit usage:snapshot-updated");
    }
}

fn is_active_usage_import_job(job: Option<&UsageImportJobSnapshot>) -> bool {
    job.is_some_and(|job| {
        !matches!(
            job.status,
            UsageImportJobStatus::Finished
                | UsageImportJobStatus::Failed
                | UsageImportJobStatus::Cancelled
        )
    })
}

fn is_active_session_index_job(job: Option<&SessionIndexJobSnapshot>) -> bool {
    job.is_some_and(|job| {
        !matches!(
            job.status,
            SessionIndexJobStatus::Finished | SessionIndexJobStatus::Failed
        )
    })
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

/// Tauri 事件 `claude_observer:updated` 的 payload。
/// 字段命名与 `claude_observer::scanner::IngestReport` 对齐，仅用 bool 暴露 errors 状态，
/// 避免把内部错误字符串数组泄漏到前端。
#[derive(Debug, Clone, Serialize)]
struct ClaudeObserverUpdatedPayload {
    files_scanned: usize,
    files_changed: usize,
    calls_inserted: usize,
    has_errors: bool,
}

/// 在 usage 导入收尾后，异步触发一次 `claude_observer` 增量解析。
///
/// 设计要点：
/// 1) 与用户感知到的 usage 导入响应解耦 —— 用 `tokio::spawn` + `spawn_blocking`
///    托管，调用方立即返回，不阻塞 UI。
/// 2) 解析完成后无论是否有错误，都发 `claude_observer:updated` 事件，让前端
///    `claudeObserver` store 刷新；errors 仅以 `has_errors` 暴露。
/// 3) 失败信息只走 tracing，避免把内部 IO 错误带到前端。
///
/// `scope_label` 用于日志识别触发来源（如 `"import_all"`、`"job_finished"`）。
fn spawn_claude_observer_ingest_tail(
    app_handle: AppHandle,
    state: &AppState,
    scope_label: &'static str,
) {
    /* ====================================================================
     * 步骤1：克隆 db_pool 句柄，把所有权移交后台任务
     * ====================================================================
     */
    let db_pool = state.db_pool.clone();
    tracing::info!(
        scope = scope_label,
        "[claude_observer] 触发增量解析 tail-hook"
    );

    tokio::spawn(async move {
        /* ====================================================================
         * 步骤2：spawn_blocking 中执行 ingest_incremental
         * ====================================================================
         * - 取 db_pool 连接（短期持有，跑完即归还）
         * - 解析 ~/.claude/projects 下的 jsonl 增量行
         */
        let join = tokio::task::spawn_blocking(move || {
            let conn = match db_pool.get() {
                Ok(conn) => conn,
                Err(error) => {
                    return Err(format!("db pool error: {error}"));
                }
            };
            // 把 PooledConnection 解引用为 &mut Connection 给 scanner 使用
            let mut conn = conn;
            let report = crate::claude_observer::scanner::ingest_incremental(&mut conn);
            Ok(report)
        })
        .await;

        let report = match join {
            Ok(Ok(report)) => report,
            Ok(Err(error)) => {
                tracing::warn!(
                    scope = scope_label,
                    error = %error,
                    "[claude_observer] ingest 跳过（DB 不可用）"
                );
                return;
            }
            Err(error) => {
                tracing::warn!(
                    scope = scope_label,
                    ?error,
                    "[claude_observer] ingest 后台任务异常退出"
                );
                return;
            }
        };

        /* ====================================================================
         * 步骤3：info 级日志 + 发 Tauri 事件 `claude_observer:updated`
         * ====================================================================
         */
        let has_errors = !report.errors.is_empty();
        tracing::info!(
            scope = scope_label,
            "claude_observer ingest: files_scanned={} files_changed={} calls_inserted={} errors={}",
            report.files_scanned,
            report.files_changed,
            report.calls_inserted,
            report.errors.len()
        );

        let payload = ClaudeObserverUpdatedPayload {
            files_scanned: report.files_scanned,
            files_changed: report.files_changed,
            calls_inserted: report.calls_inserted,
            has_errors,
        };
        if let Err(error) = app_handle.emit("claude_observer:updated", payload) {
            tracing::warn!(
                scope = scope_label,
                ?error,
                "Failed to emit claude_observer:updated"
            );
        }
    });
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
    let file_path = session.file_path.display().to_string();
    let source_variant = match platform_label {
        "claude" => "claude-jsonl",
        "codex"
            if session
                .file_path
                .components()
                .any(|part| part.as_os_str() == "archived_sessions") =>
        {
            "codex-archived"
        }
        "codex" => "codex-live",
        "gemini" => "gemini-jsonl",
        _ => "legacy-jsonl",
    };
    ccr_db::database::repositories::usage_repo::UsageSessionArchiveEntry {
        archive_id: ccr_db::database::repositories::usage_repo::agent_session_archive_id(
            platform_label,
            &file_path,
            "",
        ),
        session_id: session.id.clone(),
        platform: platform_label.to_string(),
        title: session.title.clone(),
        cwd: session.cwd.display().to_string(),
        file_path,
        file_hash: Some(session.file_hash.clone()),
        source_variant: source_variant.to_string(),
        source_kind: "file".to_string(),
        source_member_id: String::new(),
        source_size: session
            .file_path
            .metadata()
            .ok()
            .and_then(|metadata| i64::try_from(metadata.len()).ok()),
        source_mtime_ns: session
            .file_path
            .metadata()
            .ok()
            .and_then(|metadata| metadata.modified().ok())
            .and_then(|value| value.duration_since(std::time::UNIX_EPOCH).ok())
            .and_then(|value| i64::try_from(value.as_nanos()).ok()),
        source_stat_hash: Some(session.file_hash.clone()),
        message_count: i64::from(session.message_count),
        user_message_count: i64::from(session.user_message_count),
        assistant_message_count: i64::from(session.assistant_message_count),
        tool_use_count: i64::from(session.tool_use_count),
        source_fidelity: "full".to_string(),
        created_at: session.created_at,
        updated_at: session.updated_at,
        source_state: ccr_db::database::repositories::usage_repo::UsageSourceState::Live,
        last_seen_at: Some(Utc::now()),
        raw_deleted_at: None,
        archived_at: Utc::now(),
    }
}

fn sync_platform_session_archive(
    usage_db_pool: &ccr_db::database::DbPool,
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
        | JobEvent::PricingUpgradeStarted { .. }
        | JobEvent::PricingUpgradeProgress { .. }
        | JobEvent::PricingBucketReconcileStarted { .. }
        | JobEvent::PricingUpgradeFinished { .. }
        | JobEvent::LockWaiting { .. }
        | JobEvent::LockAcquired { .. }
        | JobEvent::TokenAccountingRepairStarted { .. }
        | JobEvent::TokenAccountingRepairFinished { .. } => {
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
                invalidate_usage_snapshot_cache(app_handle).await;
                emit_usage_snapshot_updated(
                    app_handle,
                    "job-recent-ready",
                    platform,
                    Some(job_id.to_string()),
                    snapshot.records_imported,
                )
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

    invalidate_usage_snapshot_cache(app_handle).await;
    emit_usage_snapshot_updated(
        app_handle,
        "job-finished",
        platform,
        Some(job_id.to_string()),
        summary.imported_records,
    )
    .await;
    emit_usage_import_job_snapshot(app_handle, "usage:job-finished", &final_snapshot).await;

    // 步骤N: 流式 usage 导入任务收尾时，触发 claude_observer 增量解析。
    // 与 `import_all_usage_v2` 共用同一份 tail-hook 工具方法；events 链路也共用
    // `claude_observer:updated`，前端只需订阅这一个事件。
    let state = app_handle.state::<AppState>();
    spawn_claude_observer_ingest_tail(app_handle.clone(), &state, "usage_job_finished");

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
            provider_map: provider_activation_map_path(),
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
            provider_map: provider_activation_map_path(),
        },
    )
    .await
    .map_err(|error| error.to_string())?;
    let mut results = collect_llmusage_sync_results(events, requested_source.as_deref())?;
    Ok(results
        .pop()
        .unwrap_or_else(|| default_import_results(requested_source.as_deref()).remove(0)))
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
        invalidate_usage_snapshot_cache(&app_handle).await;
        emit_usage_snapshot_updated(
            &app_handle,
            "session-index-finished",
            None,
            Some(job_id.clone()),
            0,
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

/// 获取用量汇总数据
#[ccr_tauri_command_macros::command]
pub async fn get_usage_summary_v2(
    state: State<'_, AppState>,
    platform: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<UsageSummaryDto, String> {
    let command_started = Instant::now();
    let llmusage = state.llmusage.clone();
    let result = tokio::task::spawn_blocking(move || {
        let db_started = Instant::now();
        services::usage::usage_summary(&llmusage, platform, start_date, end_date)
            .map(|summary| (summary, elapsed_ms(db_started)))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?;

    record_command_duration(&state, command_started);
    let (summary, db_ms) = result?;
    record_db_duration(&state, db_ms);
    Ok(summary)
}

/// 获取用量趋势数据
#[ccr_tauri_command_macros::command]
pub async fn get_usage_trends_v2(
    state: State<'_, AppState>,
    platform: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<Vec<DailyTrendDto>, String> {
    let command_started = Instant::now();
    let llmusage = state.llmusage.clone();
    let result = tokio::task::spawn_blocking(move || {
        let db_started = Instant::now();
        services::usage::usage_trends(&llmusage, platform, start_date, end_date)
            .map(|trends| (trends, elapsed_ms(db_started)))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?;

    record_command_duration(&state, command_started);
    let (trends, db_ms) = result?;
    record_db_duration(&state, db_ms);
    Ok(trends)
}

/// 获取按模型聚合的用量统计
#[ccr_tauri_command_macros::command]
pub async fn get_usage_by_model_v2(
    state: State<'_, AppState>,
    platform: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<Vec<ModelStatDto>, String> {
    let command_started = Instant::now();
    let llmusage = state.llmusage.clone();
    let result = tokio::task::spawn_blocking(move || {
        let db_started = Instant::now();
        services::usage::usage_by_model(&llmusage, platform, start_date, end_date)
            .map(|stats| (stats, elapsed_ms(db_started)))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?;

    record_command_duration(&state, command_started);
    let (stats, db_ms) = result?;
    record_db_duration(&state, db_ms);
    Ok(stats)
}

/// 获取按 provider 聚合的用量统计
#[ccr_tauri_command_macros::command]
pub async fn get_usage_by_provider_v2(
    state: State<'_, AppState>,
    platform: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<Vec<ProviderBreakdownDto>, String> {
    let command_started = Instant::now();
    let llmusage = state.llmusage.clone();
    let result = tokio::task::spawn_blocking(move || {
        let db_started = Instant::now();
        services::usage::usage_by_provider(&llmusage, platform, start_date, end_date)
            .map(|stats| (stats, elapsed_ms(db_started)))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?;

    record_command_duration(&state, command_started);
    let (stats, db_ms) = result?;
    record_db_duration(&state, db_ms);
    Ok(stats)
}

/// 获取按项目聚合的用量统计
#[ccr_tauri_command_macros::command]
pub async fn get_usage_by_project_v2(
    state: State<'_, AppState>,
    platform: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<Vec<ProjectStatDto>, String> {
    let command_started = Instant::now();
    let llmusage = state.llmusage.clone();
    let result = tokio::task::spawn_blocking(move || {
        let db_started = Instant::now();
        services::usage::usage_by_project(&llmusage, platform, start_date, end_date)
            .map(|stats| (stats, elapsed_ms(db_started)))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?;

    record_command_duration(&state, command_started);
    let (stats, db_ms) = result?;
    record_db_duration(&state, db_ms);
    Ok(stats)
}

/// 获取热力图数据（V2，来自 llmusage usage_bucket_30m）
#[ccr_tauri_command_macros::command]
pub async fn get_usage_heatmap_v2(
    state: State<'_, AppState>,
    platform: Option<String>,
    days: Option<i64>,
) -> Result<HeatmapResponseDto, String> {
    let command_started = Instant::now();
    let llmusage = state.llmusage.clone();
    let days = days.unwrap_or(365).clamp(1, 366) as u32;

    let result = tokio::task::spawn_blocking(move || {
        let db_started = Instant::now();
        services::usage::usage_heatmap(&llmusage, platform, days)
            .map(|heatmap| (heatmap, elapsed_ms(db_started)))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?;

    record_command_duration(&state, command_started);
    let (heatmap, db_ms) = result?;
    record_db_duration(&state, db_ms);
    Ok(heatmap)
}

/// 获取用量日志列表，保持前端 cursor 分页契约
#[ccr_tauri_command_macros::command]
pub async fn get_usage_logs_v2(
    state: State<'_, AppState>,
    query: UsageLogsQuery,
) -> Result<PaginatedLogsDto, String> {
    let command_started = Instant::now();
    let llmusage = state.llmusage.clone();

    let result = tokio::task::spawn_blocking(move || {
        let db_started = Instant::now();
        services::usage::usage_logs(&llmusage, &query).map(|logs| (logs, elapsed_ms(db_started)))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?;

    record_command_duration(&state, command_started);
    let (logs, db_ms) = result?;
    record_db_duration(&state, db_ms);
    Ok(logs)
}

/// spawn_blocking 包装：调用 service 计算 dashboard 聚合并附带 DB 计时。
#[allow(clippy::too_many_arguments)]
async fn compute_usage_dashboard_payload(
    llmusage: Arc<LlmusageRuntime>,
    usage_db_pool: ccr_db::database::DbPool,
    platform: Option<String>,
    provider: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
    heatmap_days: u32,
    include_heatmap: bool,
    active_usage_import: bool,
    active_session_index: bool,
) -> Result<(UsageDashboardResponse, f64), String> {
    tokio::task::spawn_blocking(move || {
        let db_started = Instant::now();
        // 同步 FS 扫描：保持在 spawn_blocking 内计算后传入（语义与原地调用一致）。
        let raw_sessions_present = services::usage::has_any_raw_sessions();
        services::usage::compute_dashboard(
            &llmusage,
            &usage_db_pool,
            platform,
            provider,
            start_date,
            end_date,
            heatmap_days,
            include_heatmap,
            active_usage_import,
            active_session_index,
            raw_sessions_present,
        )
        .map(|dashboard| (dashboard, elapsed_ms(db_started)))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

/// 获取用量仪表盘数据，聚合汇总、趋势、模型、项目统计与 usage snapshot 投影。
#[ccr_tauri_command_macros::command]
pub async fn get_usage_dashboard_v2(
    state: State<'_, AppState>,
    platform: Option<String>,
    provider: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
    heatmap_days: Option<i64>,
    include_heatmap: Option<bool>,
) -> Result<UsageDashboardResponse, String> {
    let command_started = Instant::now();
    let heatmap_days = heatmap_days.unwrap_or(365).clamp(1, 366) as u32;
    let include_heatmap = include_heatmap.unwrap_or(false);
    let active_usage_job = state.get_active_usage_import_job().await;
    let active_session_job = state.get_active_session_index_job().await;
    let active_usage_import = is_active_usage_import_job(active_usage_job.as_ref());
    let active_session_index = is_active_session_index_job(active_session_job.as_ref());
    let cacheable = !active_usage_import && !active_session_index;
    let cache_key = usage_dashboard_cache_key(
        platform.as_deref(),
        provider.as_deref(),
        start_date.as_deref(),
        end_date.as_deref(),
        heatmap_days,
        include_heatmap,
    );

    if cacheable {
        if let Some(cached) = state.cache_get(&cache_key).await {
            record_command_duration(&state, command_started);
            return serde_json::from_value(cached).map_err(|e| format!("Cache decode error: {e}"));
        }

        match state.begin_cache_fill(&cache_key).await {
            CacheFillRegistration::Wait(notify) => {
                notify.notified().await;
                if let Some(cached) = state.cache_get(&cache_key).await {
                    record_command_duration(&state, command_started);
                    return serde_json::from_value(cached)
                        .map_err(|e| format!("Cache decode error: {e}"));
                }
            }
            CacheFillRegistration::Leader => {
                let result = compute_usage_dashboard_payload(
                    state.llmusage.clone(),
                    state.usage_db_pool.clone(),
                    platform.clone(),
                    provider.clone(),
                    start_date.clone(),
                    end_date.clone(),
                    heatmap_days,
                    include_heatmap,
                    active_usage_import,
                    active_session_index,
                )
                .await;
                record_command_duration(&state, command_started);

                match result {
                    Ok((dashboard, db_ms)) => {
                        // 序列化失败也必须 finish_cache_fill，否则 Wait 侧会悬挂。
                        match serde_json::to_value(&dashboard) {
                            Ok(cache_value) => {
                                state
                                    .cache_set(
                                        cache_key.clone(),
                                        cache_value,
                                        USAGE_SNAPSHOT_CACHE_TTL_SECS,
                                    )
                                    .await;
                                state.finish_cache_fill(&cache_key).await;
                                record_db_duration(&state, db_ms);
                                return Ok(dashboard);
                            }
                            Err(error) => {
                                state.finish_cache_fill(&cache_key).await;
                                return Err(format!("Serialize error: {error}"));
                            }
                        }
                    }
                    Err(error) => {
                        state.finish_cache_fill(&cache_key).await;
                        return Err(error);
                    }
                }
            }
        }
    }

    let result = compute_usage_dashboard_payload(
        state.llmusage.clone(),
        state.usage_db_pool.clone(),
        platform,
        provider,
        start_date,
        end_date,
        heatmap_days,
        include_heatmap,
        active_usage_import,
        active_session_index,
    )
    .await;
    record_command_duration(&state, command_started);
    let (dashboard, db_ms) = result?;
    record_db_duration(&state, db_ms);
    Ok(dashboard)
}

/// 获取首页工作区概览数据，统一 llmusage usage + ccr session 统计链路。
#[ccr_tauri_command_macros::command]
pub async fn get_home_usage_overview_v2(
    state: State<'_, AppState>,
    days: Option<usize>,
) -> Result<HomeUsageOverviewResponse, String> {
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
    let active_usage_import = is_active_usage_import_job(active_usage_job.as_ref());
    let active_session_index = is_active_session_index_job(active_session_job.as_ref());

    let result = tokio::task::spawn_blocking(move || {
        let db_started = Instant::now();
        // ctx 在 spawn_blocking 内组装：has_any_raw_sessions 是同步 FS 扫描，
        // 不能占用 async 线程（语义与原先 service 体内原地调用一致）。
        let ctx = services::usage::HomeJobContext {
            active_usage_job_id,
            active_usage_imported_records,
            active_session_job_id,
            active_session_indexed,
            active_usage_import,
            active_session_index,
            raw_sessions_present: services::usage::has_any_raw_sessions(),
        };
        services::usage::compute_home_overview(&llmusage, &pool, days, ctx)
            .map(|payload| (payload, elapsed_ms(db_started)))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?;

    record_command_duration(&state, command_started);
    let (payload, db_ms) = result?;
    record_db_duration(&state, db_ms);
    Ok(payload)
}

#[ccr_tauri_command_macros::command]
pub async fn ensure_session_index_v2(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<StartSessionIndexJobResponse, String> {
    if let Some(snapshot) = state.get_active_session_index_job().await {
        return Ok(StartSessionIndexJobResponse {
            job_id: snapshot.job_id.clone(),
            snapshot,
        });
    }

    let job_id = format!("session-index-{}", Uuid::new_v4());
    let snapshot = SessionIndexJobSnapshot::new(job_id.clone(), session_index_platforms().len());
    state.insert_session_index_job(snapshot.clone()).await;

    tauri::async_runtime::spawn(run_session_index_job(app_handle, job_id.clone()));

    Ok(StartSessionIndexJobResponse { job_id, snapshot })
}

#[ccr_tauri_command_macros::command]
pub async fn get_session_index_job_status_v2(
    state: State<'_, AppState>,
    job_id: String,
) -> Result<SessionIndexJobSnapshot, String> {
    state
        .get_session_index_job(&job_id)
        .await
        .ok_or_else(|| format!("Session index job '{}' not found", job_id))
}

/// 从 JSONL 文件导入单个平台的用量数据
#[ccr_tauri_command_macros::command]
pub async fn import_usage_v2(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    platform: String,
) -> Result<UsageImportResultV2, String> {
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
    invalidate_usage_snapshot_cache(&app_handle).await;
    emit_usage_snapshot_updated(
        &app_handle,
        "import-usage",
        Some(result.platform.as_str()),
        None,
        summary.imported_records,
    )
    .await;

    // 步骤N: 单平台导入完成后，仅当目标平台是 claude 时跑一次 claude_observer 增量解析。
    // 其他平台的 JSONL 不在
    // `~/.claude/projects/` 树下，scanner 跑出来也是 0 文件，没必要发事件。
    if source.as_str() == "claude" {
        spawn_claude_observer_ingest_tail(app_handle.clone(), &state, "import_usage_v2");
    }

    Ok(result)
}

#[ccr_tauri_command_macros::command]
pub async fn import_all_usage_v2(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<ImportAllUsageResponse, String> {
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
    invalidate_usage_snapshot_cache(&app_handle).await;
    emit_usage_snapshot_updated(
        &app_handle,
        "import-all-usage",
        None,
        None,
        summary.imported_records,
    )
    .await;

    // 步骤N: usage 导入完成后顺带跑一次 claude_observer 增量解析。
    // 该任务异步进行，不会阻塞当前 IPC 响应；完成后发 `claude_observer:updated`
    // 让前端 Pinia store 刷新工具调用维度数据。
    spawn_claude_observer_ingest_tail(app_handle.clone(), &state, "import_all_usage_v2");

    Ok(ImportAllUsageResponse { summary, results })
}

#[ccr_tauri_command_macros::command]
pub async fn start_usage_import_job_v2(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    platform: Option<String>,
    recent_days: Option<usize>,
    reset_sources: Option<bool>,
) -> Result<StartUsageImportJobResponse, String> {
    if let Some(snapshot) = state.get_active_usage_import_job().await {
        return Ok(StartUsageImportJobResponse {
            job_id: snapshot.job_id.clone(),
            snapshot,
        });
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
        provider_map: provider_activation_map_path(),
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

    Ok(StartUsageImportJobResponse { job_id, snapshot })
}

#[ccr_tauri_command_macros::command]
pub async fn get_usage_import_job_status_v2(
    state: State<'_, AppState>,
    job_id: String,
) -> Result<UsageImportJobSnapshot, String> {
    state
        .get_usage_import_job(&job_id)
        .await
        .ok_or_else(|| format!("Usage import job '{}' not found", job_id))
}

#[ccr_tauri_command_macros::command]
pub async fn cancel_usage_import_job_v2(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    job_id: String,
) -> Result<UsageImportJobSnapshot, String> {
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
    Ok(snapshot)
}
