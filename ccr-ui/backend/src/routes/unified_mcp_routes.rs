// 统一 MCP 管理路由
// /api/unified/mcp 系列端点，聚合所有平台的 MCP CRUD

use crate::state::AppState;
use axum::{
    Router,
    routing::{delete, get, post, put},
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/unified/mcp",
            get(crate::api::handlers::unified_mcp::list_unified_mcp),
        )
        .route(
            "/unified/mcp",
            post(crate::api::handlers::unified_mcp::add_unified_mcp),
        )
        .route(
            "/unified/mcp/{platform}/{name}",
            put(crate::api::handlers::unified_mcp::update_unified_mcp),
        )
        .route(
            "/unified/mcp/{platform}/{name}",
            delete(crate::api::handlers::unified_mcp::delete_unified_mcp),
        )
        .route(
            "/unified/mcp/{platform}/{name}/toggle",
            put(crate::api::handlers::unified_mcp::toggle_unified_mcp),
        )
}
