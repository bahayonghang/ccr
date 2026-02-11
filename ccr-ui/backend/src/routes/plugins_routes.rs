// Plugin management routes
use crate::state::AppState;
use axum::{
    Router,
    routing::{delete, get, post, put},
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/plugins", get(crate::api::handlers::plugins::list_plugins))
        .route("/plugins", post(crate::api::handlers::plugins::add_plugin))
        .route(
            "/plugins/{id}",
            put(crate::api::handlers::plugins::update_plugin),
        )
        .route(
            "/plugins/{id}",
            delete(crate::api::handlers::plugins::delete_plugin),
        )
        .route(
            "/plugins/{id}/toggle",
            put(crate::api::handlers::plugins::toggle_plugin),
        )
}
