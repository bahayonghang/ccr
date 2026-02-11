// Builtin prompts routes
use crate::state::AppState;
use axum::{Router, routing::get};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/prompts/builtin",
            get(crate::api::handlers::builtin_prompts::list_builtin_prompts),
        )
        .route(
            "/prompts/builtin/{id}",
            get(crate::api::handlers::builtin_prompts::get_builtin_prompt),
        )
        .route(
            "/prompts/builtin/category/{category}",
            get(crate::api::handlers::builtin_prompts::get_builtin_prompts_by_category),
        )
}
