// Sync management routes
use axum::{
    Router,
    routing::{delete, get, post, put},
};

pub fn routes() -> Router {
    Router::new()
        // Basic sync routes
        .route(
            "/sync/status",
            get(crate::api::handlers::sync::get_sync_status),
        )
        .route("/sync/push", post(crate::api::handlers::sync::push_config))
        .route("/sync/pull", post(crate::api::handlers::sync::pull_config))
        .route("/sync/info", get(crate::api::handlers::sync::get_sync_info))
        .route(
            "/sync/config",
            post(crate::api::handlers::sync::configure_sync),
        )
        // Folder sync routes
        .route(
            "/sync/folders",
            get(crate::api::handlers::sync::list_sync_folders),
        )
        .route(
            "/sync/folders",
            post(crate::api::handlers::sync::add_sync_folder),
        )
        .route(
            "/sync/folders/{name}",
            delete(crate::api::handlers::sync::remove_sync_folder),
        )
        .route(
            "/sync/folders/{name}",
            get(crate::api::handlers::sync::get_sync_folder_info),
        )
        .route(
            "/sync/folders/{name}/enable",
            put(crate::api::handlers::sync::enable_sync_folder),
        )
        .route(
            "/sync/folders/{name}/disable",
            put(crate::api::handlers::sync::disable_sync_folder),
        )
        .route(
            "/sync/folders/{name}/push",
            post(crate::api::handlers::sync::push_sync_folder),
        )
        .route(
            "/sync/folders/{name}/pull",
            post(crate::api::handlers::sync::pull_sync_folder),
        )
        .route(
            "/sync/folders/{name}/status",
            get(crate::api::handlers::sync::get_sync_folder_status),
        )
        // Batch sync operations
        .route(
            "/sync/all/push",
            post(crate::api::handlers::sync::push_all_folders),
        )
        .route(
            "/sync/all/pull",
            post(crate::api::handlers::sync::pull_all_folders),
        )
        .route(
            "/sync/all/status",
            get(crate::api::handlers::sync::get_all_folders_status),
        )
}
