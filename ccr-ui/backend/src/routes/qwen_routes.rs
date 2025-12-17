// Qwen platform routes
use axum::{
    Router,
    routing::{delete, get, post, put},
};

pub fn routes() -> Router {
    Router::new()
        // MCP routes
        .route(
            "/qwen/mcp",
            get(crate::api::handlers::platforms::qwen::list_qwen_mcp_servers),
        )
        .route(
            "/qwen/mcp",
            post(crate::api::handlers::platforms::qwen::add_qwen_mcp_server),
        )
        .route(
            "/qwen/mcp/:name",
            put(crate::api::handlers::platforms::qwen::update_qwen_mcp_server),
        )
        .route(
            "/qwen/mcp/:name",
            delete(crate::api::handlers::platforms::qwen::delete_qwen_mcp_server),
        )
        // Config routes
        .route(
            "/qwen/config",
            get(crate::api::handlers::platforms::qwen::get_qwen_config),
        )
        .route(
            "/qwen/config",
            put(crate::api::handlers::platforms::qwen::update_qwen_config),
        )
}
