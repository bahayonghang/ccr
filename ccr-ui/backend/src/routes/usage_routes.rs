// Usage analytics routes
use axum::{Router, routing::get};

pub fn routes() -> Router {
    Router::new().route(
        "/usage/records",
        get(crate::api::handlers::usage::get_usage_records),
    )
}
