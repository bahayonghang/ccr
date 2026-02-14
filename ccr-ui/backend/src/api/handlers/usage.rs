// Usage Analytics API Handler
// SQLite-backed usage analytics with caching

use axum::extract::Query;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{debug, error, info};

use crate::api::handlers::response::ApiSuccess;
use crate::core::error::{ApiError, ApiResult};
use crate::database::{self, repositories::usage_repo};
use crate::services::usage_import_service::{ImportConfig, ImportResult, UsageImportService};

// Dashboard 缓存
#[derive(Clone)]
struct CachedDashboardData {
    response: UsageDashboardResponse,
    timestamp: Instant,
}

lazy_static::lazy_static! {
    static ref USAGE_DASHBOARD_CACHE: Arc<Mutex<HashMap<String, CachedDashboardData>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

const DASHBOARD_CACHE_TTL: Duration = Duration::from_secs(120);

fn env_flag(name: &str, default: bool) -> bool {
    std::env::var(name)
        .ok()
        .map(|v| {
            let normalized = v.trim().to_ascii_lowercase();
            matches!(normalized.as_str(), "1" | "true" | "yes" | "on")
        })
        .unwrap_or(default)
}

fn usage_dashboard_aggregated_api_enabled() -> bool {
    env_flag("USAGE_DASHBOARD_AGGREGATED_API", true)
}

fn usage_logs_cursor_paging_enabled() -> bool {
    env_flag("USAGE_LOGS_CURSOR_PAGING", true)
}

// ═══════════════════════════════════════════════════════════
// 查询参数
// ═══════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
pub struct UsageQueryParams {
    pub platform: Option<String>,
    pub start: Option<String>,
    pub end: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct HeatmapQueryParams {
    pub platform: Option<String>,
    #[serde(default = "default_heatmap_days")]
    pub days: i64,
}

fn default_heatmap_days() -> i64 {
    365
}

#[derive(Debug, Deserialize)]
pub struct LogsQueryParams {
    pub platform: Option<String>,
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_page_size")]
    pub page_size: i64,
    pub model: Option<String>,
    pub cursor: Option<String>,
    #[serde(default)]
    pub include_total: bool,
}

fn default_page() -> i64 {
    1
}
fn default_page_size() -> i64 {
    50
}

#[derive(Debug, Deserialize)]
pub struct DashboardQueryParams {
    pub platform: Option<String>,
    pub start: Option<String>,
    pub end: Option<String>,
    #[serde(default = "default_heatmap_days")]
    pub days: i64,
    #[serde(default = "default_include_heatmap")]
    pub include_heatmap: bool,
}

fn default_include_heatmap() -> bool {
    true
}

#[derive(Debug, Deserialize)]
pub struct ImportQuery {
    #[serde(default = "default_platform")]
    pub platform: String,
}

fn default_platform() -> String {
    "claude".to_string()
}

// ═══════════════════════════════════════════════════════════
// 响应类型
// ═══════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize)]
pub struct HeatmapResponse {
    pub data: HashMap<String, i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UsageDashboardResponse {
    pub summary: usage_repo::UsageSummary,
    pub trends: Vec<usage_repo::DailyTrend>,
    pub model_stats: Vec<usage_repo::ModelStat>,
    pub project_stats: Vec<usage_repo::ProjectStat>,
    pub heatmap: HeatmapResponse,
    pub generated_at: String,
}

#[derive(Debug, Serialize)]
pub struct ImportResponse {
    pub result: ImportResult,
    pub message: String,
}

// ═══════════════════════════════════════════════════════════
// Handler 实现
// ═══════════════════════════════════════════════════════════

fn build_dashboard_cache_key(params: &DashboardQueryParams) -> String {
    format!(
        "platform={}|start={}|end={}|days={}|include_heatmap={}",
        params.platform.clone().unwrap_or_default(),
        params.start.clone().unwrap_or_default(),
        params.end.clone().unwrap_or_default(),
        params.days,
        params.include_heatmap
    )
}

/// GET /api/usage/dashboard
pub async fn get_usage_dashboard(
    Query(params): Query<DashboardQueryParams>,
) -> ApiResult<ApiSuccess<UsageDashboardResponse>> {
    let start = Instant::now();
    let use_cache = usage_dashboard_aggregated_api_enabled();
    let cache_key = build_dashboard_cache_key(&params);

    if use_cache {
        let cache = USAGE_DASHBOARD_CACHE
            .lock()
            .expect("无法获取 dashboard 缓存锁");
        if let Some(cached) = cache.get(&cache_key)
            && cached.timestamp.elapsed() < DASHBOARD_CACHE_TTL
        {
            debug!("Using cached usage dashboard: {}", cache_key);
            return Ok(ApiSuccess(cached.response.clone()));
        }
    }

    let response = database::with_connection(|conn| {
        let summary =
            usage_repo::get_usage_summary(conn, &params.platform, &params.start, &params.end)?;
        let trends =
            usage_repo::get_daily_trends(conn, &params.platform, &params.start, &params.end)?;
        let model_stats =
            usage_repo::get_model_stats(conn, &params.platform, &params.start, &params.end)?;
        let project_stats =
            usage_repo::get_project_stats(conn, &params.platform, &params.start, &params.end)?;
        let heatmap = if params.include_heatmap {
            let data = usage_repo::get_heatmap_data(conn, &params.platform, params.days)?;
            HeatmapResponse { data }
        } else {
            HeatmapResponse {
                data: HashMap::new(),
            }
        };

        Ok::<UsageDashboardResponse, rusqlite::Error>(UsageDashboardResponse {
            summary,
            trends,
            model_stats,
            project_stats,
            heatmap,
            generated_at: Utc::now().to_rfc3339(),
        })
    })?;

    if use_cache {
        let mut cache = USAGE_DASHBOARD_CACHE
            .lock()
            .expect("无法获取 dashboard 缓存锁");
        cache.insert(
            cache_key,
            CachedDashboardData {
                response: response.clone(),
                timestamp: Instant::now(),
            },
        );
    }

    info!(
        "usage dashboard generated in {} ms (platform={:?}, include_heatmap={})",
        start.elapsed().as_millis(),
        params.platform,
        params.include_heatmap
    );

    Ok(ApiSuccess(response))
}

/// GET /api/usage/summary
pub async fn get_usage_summary(
    Query(params): Query<UsageQueryParams>,
) -> ApiResult<ApiSuccess<usage_repo::UsageSummary>> {
    let start = Instant::now();
    let summary = database::with_connection(|conn| {
        usage_repo::get_usage_summary(conn, &params.platform, &params.start, &params.end)
    })?;
    info!(
        "usage summary query done in {} ms (platform={:?})",
        start.elapsed().as_millis(),
        params.platform
    );
    Ok(ApiSuccess(summary))
}

/// GET /api/usage/trends
pub async fn get_usage_trends(
    Query(params): Query<UsageQueryParams>,
) -> ApiResult<ApiSuccess<Vec<usage_repo::DailyTrend>>> {
    let start = Instant::now();
    let trends = database::with_connection(|conn| {
        usage_repo::get_daily_trends(conn, &params.platform, &params.start, &params.end)
    })?;
    info!(
        "usage trends query done in {} ms (platform={:?})",
        start.elapsed().as_millis(),
        params.platform
    );
    Ok(ApiSuccess(trends))
}

/// GET /api/usage/by-model
pub async fn get_usage_by_model(
    Query(params): Query<UsageQueryParams>,
) -> ApiResult<ApiSuccess<Vec<usage_repo::ModelStat>>> {
    let start = Instant::now();
    let stats = database::with_connection(|conn| {
        usage_repo::get_model_stats(conn, &params.platform, &params.start, &params.end)
    })?;
    info!(
        "usage by-model query done in {} ms (platform={:?})",
        start.elapsed().as_millis(),
        params.platform
    );
    Ok(ApiSuccess(stats))
}

/// GET /api/usage/by-project
pub async fn get_usage_by_project(
    Query(params): Query<UsageQueryParams>,
) -> ApiResult<ApiSuccess<Vec<usage_repo::ProjectStat>>> {
    let start = Instant::now();
    let stats = database::with_connection(|conn| {
        usage_repo::get_project_stats(conn, &params.platform, &params.start, &params.end)
    })?;
    info!(
        "usage by-project query done in {} ms (platform={:?})",
        start.elapsed().as_millis(),
        params.platform
    );
    Ok(ApiSuccess(stats))
}

/// GET /api/usage/heatmap
pub async fn get_usage_heatmap(
    Query(params): Query<HeatmapQueryParams>,
) -> ApiResult<ApiSuccess<HeatmapResponse>> {
    let start = Instant::now();
    let data = database::with_connection(|conn| {
        usage_repo::get_heatmap_data(conn, &params.platform, params.days)
    })?;
    info!(
        "usage heatmap query done in {} ms (platform={:?}, days={})",
        start.elapsed().as_millis(),
        params.platform,
        params.days
    );
    Ok(ApiSuccess(HeatmapResponse { data }))
}

/// GET /api/usage/logs
pub async fn get_usage_logs(
    Query(params): Query<LogsQueryParams>,
) -> ApiResult<ApiSuccess<usage_repo::PaginatedLogs>> {
    let start = Instant::now();
    let page = params.page.max(1);
    let page_size = params.page_size.clamp(1, 500);

    let logs = database::with_connection(|conn| {
        if usage_logs_cursor_paging_enabled() && params.cursor.is_some() {
            usage_repo::get_logs_by_cursor(
                conn,
                &params.platform,
                page_size,
                &params.model,
                &params.cursor,
                params.include_total,
            )
        } else {
            usage_repo::get_paginated_logs(
                conn,
                &params.platform,
                page,
                page_size,
                &params.model,
                params.include_total,
            )
        }
    })?;

    info!(
        "usage logs query done in {} ms (page={}, page_size={}, cursor={}, include_total={})",
        start.elapsed().as_millis(),
        page,
        page_size,
        params.cursor.is_some(),
        params.include_total
    );

    Ok(ApiSuccess(logs))
}

/// POST /api/usage/import
pub async fn import_usage(
    Query(params): Query<ImportQuery>,
) -> ApiResult<ApiSuccess<ImportResponse>> {
    info!("Import for platform: {}", params.platform);

    let valid_platforms = ["claude", "codex", "gemini"];
    if !valid_platforms.contains(&params.platform.as_str()) {
        return Err(ApiError::bad_request(format!(
            "Platform '{}' not supported. Supported: claude, codex, gemini",
            params.platform
        )));
    }

    let service = UsageImportService::new(ImportConfig::default());
    let result = service.import_platform(&params.platform).map_err(|e| {
        error!("Import failed for {}: {}", params.platform, e);
        ApiError::internal(format!("Import failed: {}", e))
    })?;

    let message = format!(
        "Imported {} records from {} files for {}",
        result.records_imported, result.files_processed, result.platform
    );
    Ok(ApiSuccess(ImportResponse { result, message }))
}
