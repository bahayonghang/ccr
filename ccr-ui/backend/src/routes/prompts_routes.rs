// Prompts management routes
use axum::{
    Router,
    routing::{delete, get, post},
};

pub fn routes() -> Router {
    Router::new()
        .route("/prompts", get(crate::api::handlers::prompts::list_prompts))
        .route("/prompts", post(crate::api::handlers::prompts::add_prompt))
        .route(
            "/prompts/:name",
            get(crate::api::handlers::prompts::get_prompt),
        )
        .route(
            "/prompts/:name",
            delete(crate::api::handlers::prompts::delete_prompt),
        )
        .route(
            "/prompts/:name/apply",
            post(crate::api::handlers::prompts::apply_prompt),
        )
        .route(
            "/prompts/current/:target",
            get(crate::api::handlers::prompts::get_current_prompt),
        )
}
