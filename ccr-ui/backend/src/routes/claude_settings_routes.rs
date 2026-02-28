// Claude Code Settings routes
use crate::state::AppState;
use axum::{
    Router,
    routing::{get, put},
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/claude-settings",
            get(crate::api::handlers::claude_settings::get_settings),
        )
        .route(
            "/claude-settings",
            put(crate::api::handlers::claude_settings::update_settings),
        )
}
