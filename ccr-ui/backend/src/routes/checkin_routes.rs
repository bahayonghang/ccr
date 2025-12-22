// 签到管理路由
use axum::{
    Router,
    routing::{delete, get, post, put},
};

use crate::api::handlers::checkin;

pub fn routes() -> Router {
    Router::new()
        // ═══════════════════════════════════════════════════════════
        // 提供商管理
        // ═══════════════════════════════════════════════════════════
        .route("/checkin/providers", get(checkin::list_providers))
        .route("/checkin/providers", post(checkin::create_provider))
        .route("/checkin/providers/{id}", get(checkin::get_provider))
        .route("/checkin/providers/{id}", put(checkin::update_provider))
        .route("/checkin/providers/{id}", delete(checkin::delete_provider))
        // ═══════════════════════════════════════════════════════════
        // 账号管理
        // ═══════════════════════════════════════════════════════════
        .route("/checkin/accounts", get(checkin::list_accounts))
        .route("/checkin/accounts", post(checkin::create_account))
        .route("/checkin/accounts/{id}", get(checkin::get_account))
        .route("/checkin/accounts/{id}", put(checkin::update_account))
        .route("/checkin/accounts/{id}", delete(checkin::delete_account))
        // ═══════════════════════════════════════════════════════════
        // 签到操作
        // ═══════════════════════════════════════════════════════════
        .route("/checkin/execute", post(checkin::execute_checkin))
        .route(
            "/checkin/accounts/{id}/checkin",
            post(checkin::checkin_account),
        )
        // ═══════════════════════════════════════════════════════════
        // 余额查询
        // ═══════════════════════════════════════════════════════════
        .route(
            "/checkin/accounts/{id}/balance",
            post(checkin::query_balance),
        )
        .route(
            "/checkin/accounts/{id}/balance/history",
            get(checkin::get_balance_history),
        )
        // ═══════════════════════════════════════════════════════════
        // 签到记录
        // ═══════════════════════════════════════════════════════════
        .route("/checkin/records", get(checkin::list_records))
        .route(
            "/checkin/accounts/{id}/records",
            get(checkin::get_account_records),
        )
        // ═══════════════════════════════════════════════════════════
        // 统计
        // ═══════════════════════════════════════════════════════════
        .route("/checkin/stats/today", get(checkin::get_today_stats))
        // ═══════════════════════════════════════════════════════════
        // 导入/导出
        // ═══════════════════════════════════════════════════════════
        .route("/checkin/export", post(checkin::export_config))
        .route("/checkin/import/preview", post(checkin::preview_import))
        .route("/checkin/import", post(checkin::execute_import))
        // ═══════════════════════════════════════════════════════════
        // 连接测试
        // ═══════════════════════════════════════════════════════════
        .route(
            "/checkin/accounts/{id}/test",
            post(checkin::test_connection),
        )
}
