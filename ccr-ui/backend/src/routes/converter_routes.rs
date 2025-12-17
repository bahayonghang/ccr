// Config converter routes
use axum::{Router, routing::post};

pub fn routes() -> Router {
    Router::new().route(
        "/converter/convert",
        post(crate::api::handlers::converter::convert_config),
    )
}
