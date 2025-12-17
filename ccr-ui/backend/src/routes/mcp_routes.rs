// MCP server management routes
use axum::{
    Router,
    routing::{delete, get, post, put},
};

pub fn routes() -> Router {
    Router::new()
        .route("/mcp", get(crate::api::handlers::mcp::list_mcp_servers))
        .route("/mcp", post(crate::api::handlers::mcp::add_mcp_server))
        .route(
            "/mcp/:name",
            put(crate::api::handlers::mcp::update_mcp_server),
        )
        .route(
            "/mcp/:name",
            delete(crate::api::handlers::mcp::delete_mcp_server),
        )
        .route(
            "/mcp/:name/toggle",
            put(crate::api::handlers::mcp::toggle_mcp_server),
        )
}

pub fn presets_routes() -> Router {
    Router::new()
        .route(
            "/mcp/presets",
            get(crate::api::handlers::mcp_presets::list_presets),
        )
        .route(
            "/mcp/presets/:id",
            get(crate::api::handlers::mcp_presets::get_preset),
        )
        .route(
            "/mcp/presets/:id/install",
            post(crate::api::handlers::mcp_presets::install_preset),
        )
        .route(
            "/mcp/presets/install",
            post(crate::api::handlers::mcp_presets::install_preset_single),
        )
}

pub fn sync_routes() -> Router {
    Router::new()
        .route(
            "/mcp/sync/source",
            get(crate::api::handlers::mcp_presets::list_source_mcp_servers),
        )
        .route(
            "/mcp/sync/all",
            post(crate::api::handlers::mcp_presets::sync_all_mcp_servers),
        )
        .route(
            "/mcp/sync/:name",
            post(crate::api::handlers::mcp_presets::sync_mcp_server),
        )
}
