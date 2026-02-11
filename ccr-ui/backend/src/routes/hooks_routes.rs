// Hook management routes
use crate::state::AppState;
use axum::{
    Router,
    routing::{delete, get, patch, post},
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/hooks", get(crate::api::handlers::hooks::list_hooks))
        .route("/hooks", post(crate::api::handlers::hooks::add_hook))
        .route(
            "/hooks/{event}",
            patch(crate::api::handlers::hooks::update_hook),
        )
        .route(
            "/hooks/{event}",
            delete(crate::api::handlers::hooks::delete_hook),
        )
        .route(
            "/hooks/{event}/toggle",
            patch(crate::api::handlers::hooks::toggle_hook),
        )
}
