// Codex platform routes
use axum::{
    Router,
    routing::{delete, get, post, put},
};

pub fn routes() -> Router {
    Router::new()
        // MCP routes
        .route(
            "/codex/mcp",
            get(crate::api::handlers::platforms::codex::list_codex_mcp_servers),
        )
        .route(
            "/codex/mcp",
            post(crate::api::handlers::platforms::codex::add_codex_mcp_server),
        )
        .route(
            "/codex/mcp/:name",
            put(crate::api::handlers::platforms::codex::update_codex_mcp_server),
        )
        .route(
            "/codex/mcp/:name",
            delete(crate::api::handlers::platforms::codex::delete_codex_mcp_server),
        )
        // Profiles routes
        .route(
            "/codex/profiles",
            get(crate::api::handlers::platforms::codex::list_codex_profiles),
        )
        .route(
            "/codex/profiles",
            post(crate::api::handlers::platforms::codex::add_codex_profile),
        )
        .route(
            "/codex/profiles/:name",
            get(crate::api::handlers::platforms::codex::get_codex_profile),
        )
        .route(
            "/codex/profiles/:name",
            put(crate::api::handlers::platforms::codex::update_codex_profile),
        )
        .route(
            "/codex/profiles/:name",
            delete(crate::api::handlers::platforms::codex::delete_codex_profile),
        )
        .route(
            "/codex/profiles/:name/apply",
            post(crate::api::handlers::platforms::codex::apply_codex_profile),
        )
        // Config routes
        .route(
            "/codex/config",
            get(crate::api::handlers::platforms::codex::get_codex_config),
        )
        .route(
            "/codex/config",
            put(crate::api::handlers::platforms::codex::update_codex_base_config),
        )
}
