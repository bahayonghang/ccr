// UI state routes
use crate::state::AppState;
use axum::{
    Router,
    routing::{delete, get, post},
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/ui-state/favorites",
            get(crate::api::handlers::ui_state::get_favorites),
        )
        .route(
            "/ui-state/favorites",
            post(crate::api::handlers::ui_state::add_favorite),
        )
        .route(
            "/ui-state/favorites/{id}",
            delete(crate::api::handlers::ui_state::remove_favorite),
        )
        .route(
            "/ui-state/history",
            get(crate::api::handlers::ui_state::get_history),
        )
        .route(
            "/ui-state/history",
            post(crate::api::handlers::ui_state::add_history),
        )
        .route(
            "/ui-state/history",
            delete(crate::api::handlers::ui_state::clear_history),
        )
}
