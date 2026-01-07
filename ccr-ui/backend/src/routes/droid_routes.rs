// Droid platform routes
use axum::{
    Router,
    routing::{delete, get, post, put},
};

pub fn routes() -> Router {
    Router::new()
        // MCP routes
        .route(
            "/droid/mcp",
            get(crate::api::handlers::platforms::droid::list_droid_mcp_servers),
        )
        .route(
            "/droid/mcp",
            post(crate::api::handlers::platforms::droid::add_droid_mcp_server),
        )
        .route(
            "/droid/mcp/{name}",
            put(crate::api::handlers::platforms::droid::update_droid_mcp_server),
        )
        .route(
            "/droid/mcp/{name}",
            delete(crate::api::handlers::platforms::droid::delete_droid_mcp_server),
        )
        // Config routes
        .route(
            "/droid/config",
            get(crate::api::handlers::platforms::droid::get_droid_config),
        )
        .route(
            "/droid/config",
            put(crate::api::handlers::platforms::droid::update_droid_config),
        )
}
