// Configuration management routes
// CCR 配置相关路由

use axum::{
    Router,
    routing::{delete, get, patch, post, put},
};

/// Configuration management routes
pub fn routes() -> Router {
    Router::new()
        // List configurations
        .route("/configs", get(crate::api::handlers::config::list_configs))
        // Switch configuration
        .route("/switch", post(crate::api::handlers::config::switch_config))
        // Validate configurations
        .route(
            "/validate",
            get(crate::api::handlers::config::validate_configs),
        )
        // Clean backups
        .route("/clean", post(crate::api::handlers::config::clean_backups))
        // Export configuration
        .route("/export", post(crate::api::handlers::config::export_config))
        // Import configuration
        .route("/import", post(crate::api::handlers::config::import_config))
        // Get history
        .route("/history", get(crate::api::handlers::config::get_history))
        // CRUD operations for configurations
        .route("/configs", post(crate::api::handlers::config::add_config))
        .route(
            "/configs/{name}",
            get(crate::api::handlers::config::get_config),
        )
        .route(
            "/configs/{name}",
            put(crate::api::handlers::config::update_config),
        )
        .route(
            "/configs/{name}",
            delete(crate::api::handlers::config::delete_config),
        )
        // Enable/disable configurations
        .route(
            "/configs/{name}/enable",
            patch(crate::api::handlers::config::enable_config),
        )
        .route(
            "/configs/{name}/disable",
            patch(crate::api::handlers::config::disable_config),
        )
}
