// Slash command management routes
use crate::state::AppState;
use axum::{
    Router,
    routing::{delete, get, post, put},
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/slash-commands",
            get(crate::api::handlers::slash_commands::list_slash_commands),
        )
        .route(
            "/slash-commands",
            post(crate::api::handlers::slash_commands::add_slash_command),
        )
        .route(
            "/slash-commands/{name}",
            put(crate::api::handlers::slash_commands::update_slash_command),
        )
        .route(
            "/slash-commands/{name}",
            delete(crate::api::handlers::slash_commands::delete_slash_command),
        )
        .route(
            "/slash-commands/{name}/toggle",
            put(crate::api::handlers::slash_commands::toggle_slash_command),
        )
}
