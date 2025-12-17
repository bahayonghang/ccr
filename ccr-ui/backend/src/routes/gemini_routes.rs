// Gemini platform routes
use axum::{
    Router,
    routing::{delete, get, post, put},
};

pub fn routes() -> Router {
    Router::new()
        // MCP routes
        .route(
            "/gemini/mcp",
            get(crate::api::handlers::platforms::gemini::list_gemini_mcp_servers),
        )
        .route(
            "/gemini/mcp",
            post(crate::api::handlers::platforms::gemini::add_gemini_mcp_server),
        )
        .route(
            "/gemini/mcp/:name",
            put(crate::api::handlers::platforms::gemini::update_gemini_mcp_server),
        )
        .route(
            "/gemini/mcp/:name",
            delete(crate::api::handlers::platforms::gemini::delete_gemini_mcp_server),
        )
        // Config routes
        .route(
            "/gemini/config",
            get(crate::api::handlers::platforms::gemini::get_gemini_config),
        )
        .route(
            "/gemini/config",
            put(crate::api::handlers::platforms::gemini::update_gemini_config),
        )
}
