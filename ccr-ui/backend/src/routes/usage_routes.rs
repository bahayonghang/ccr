// Usage analytics routes
use crate::state::AppState;
use axum::{
    Router,
    routing::{get, post},
};

use crate::api::handlers::usage;

pub fn routes() -> Router<AppState> {
    Router::new()
        // 原有端点：从文件读取 usage 记录 (带内存缓存)
        .route("/usage/records", get(usage::get_usage_records))
        // 新增端点：增量导入到 SQLite
        .route("/usage/import", post(usage::import_usage))
        // 新增端点：导入所有平台
        .route("/usage/import/all", post(usage::import_all_usage))
        // 新增端点：获取导入统计
        .route("/usage/stats", get(usage::get_usage_stats))
        // 新增端点：清理旧记录
        .route("/usage/cleanup", post(usage::cleanup_usage))
        // ═══ V2 聚合 API ═══
        .route("/v2/usage/dashboard", get(usage::get_usage_dashboard_v2))
        .route("/v2/usage/summary", get(usage::get_usage_summary_v2))
        .route("/v2/usage/trends", get(usage::get_usage_trends_v2))
        .route("/v2/usage/by-model", get(usage::get_usage_by_model_v2))
        .route("/v2/usage/by-project", get(usage::get_usage_by_project_v2))
        .route("/v2/usage/heatmap", get(usage::get_usage_heatmap_v2))
        .route("/v2/usage/logs", get(usage::get_usage_logs_v2))
        .route("/v2/usage/import", post(usage::import_usage_v2))
}
