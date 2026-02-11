// Config converter routes
use crate::state::AppState;
use axum::{Router, routing::post};

pub fn routes() -> Router<AppState> {
    Router::new().route(
        "/converter/convert",
        post(crate::api::handlers::converter::convert_config),
    )
}
