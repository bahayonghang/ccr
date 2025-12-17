// Budget management routes
use axum::{
    Router,
    routing::{get, post},
};

pub fn routes() -> Router {
    Router::new()
        .route(
            "/budget/status",
            get(crate::api::handlers::budget::get_budget_status),
        )
        .route(
            "/budget/set",
            post(crate::api::handlers::budget::set_budget),
        )
        .route(
            "/budget/reset",
            post(crate::api::handlers::budget::reset_budget),
        )
}
