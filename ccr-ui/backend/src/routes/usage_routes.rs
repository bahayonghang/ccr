// Usage analytics routes (unified)
use crate::state::AppState;
use axum::{
    Router,
    routing::{get, post},
};

use crate::api::handlers::usage;

pub fn routes() -> Router<AppState> {
    Router::new()
        // 聚合 Dashboard（一次返回 summary + trends + model + project + heatmap）
        .route("/usage/dashboard", get(usage::get_usage_dashboard))
        // 单独查询端点
        .route("/usage/summary", get(usage::get_usage_summary))
        .route("/usage/trends", get(usage::get_usage_trends))
        .route("/usage/by-model", get(usage::get_usage_by_model))
        .route("/usage/by-project", get(usage::get_usage_by_project))
        .route("/usage/heatmap", get(usage::get_usage_heatmap))
        .route("/usage/logs", get(usage::get_usage_logs))
        // 导入
        .route("/usage/import", post(usage::import_usage))
}
