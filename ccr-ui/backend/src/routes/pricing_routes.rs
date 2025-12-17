// Pricing management routes
use axum::{
    Router,
    routing::{delete, get, post},
};

pub fn routes() -> Router {
    Router::new()
        .route(
            "/pricing/list",
            get(crate::api::handlers::pricing::get_pricing_list),
        )
        .route(
            "/pricing/set",
            post(crate::api::handlers::pricing::set_pricing),
        )
        .route(
            "/pricing/remove/:model",
            delete(crate::api::handlers::pricing::remove_pricing),
        )
        .route(
            "/pricing/reset",
            post(crate::api::handlers::pricing::reset_pricing),
        )
}
