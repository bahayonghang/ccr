// Usage analytics routes
use axum::{
    Router,
    routing::{get, post},
};

use crate::api::handlers::usage;

pub fn routes() -> Router {
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
}
