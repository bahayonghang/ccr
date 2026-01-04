// Statusline Configuration Routes

use axum::{
    Router,
    routing::{get, put},
};

use crate::api::handlers::statusline;

pub fn routes() -> Router {
    Router::new()
        .route("/statusline", get(statusline::get_statusline))
        .route("/statusline", put(statusline::update_statusline))
}
