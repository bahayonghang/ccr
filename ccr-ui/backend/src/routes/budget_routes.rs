// Budget management routes
use crate::state::AppState;
use axum::{
    Router,
    routing::{get, post},
};

pub fn routes() -> Router<AppState> {
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
