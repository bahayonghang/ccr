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
        // Custom Models routes
        .route(
            "/droid/models",
            get(crate::api::handlers::platforms::droid::list_droid_custom_models),
        )
        .route(
            "/droid/models",
            post(crate::api::handlers::platforms::droid::add_droid_custom_model),
        )
        .route(
            "/droid/models/{model_id}",
            put(crate::api::handlers::platforms::droid::update_droid_custom_model),
        )
        .route(
            "/droid/models/{model_id}",
            delete(crate::api::handlers::platforms::droid::delete_droid_custom_model),
        )
        // Profiles routes
        .route(
            "/droid/profiles",
            get(crate::api::handlers::platforms::droid::list_droid_profiles),
        )
        .route(
            "/droid/profiles",
            post(crate::api::handlers::platforms::droid::add_droid_profile),
        )
        .route(
            "/droid/profiles/{name}",
            put(crate::api::handlers::platforms::droid::update_droid_profile),
        )
        .route(
            "/droid/profiles/{name}",
            delete(crate::api::handlers::platforms::droid::delete_droid_profile),
        )
        .route(
            "/droid/profiles/{name}/switch",
            post(crate::api::handlers::platforms::droid::switch_droid_profile),
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
        // Droids (Subagents) routes
        .route(
            "/droid/droids",
            get(crate::api::handlers::platforms::droid::list_droids),
        )
        .route(
            "/droid/droids",
            post(crate::api::handlers::platforms::droid::create_droid),
        )
        .route(
            "/droid/droids/{name}",
            get(crate::api::handlers::platforms::droid::get_droid),
        )
        .route(
            "/droid/droids/{name}",
            put(crate::api::handlers::platforms::droid::update_droid),
        )
        .route(
            "/droid/droids/{name}",
            delete(crate::api::handlers::platforms::droid::delete_droid),
        )
}
