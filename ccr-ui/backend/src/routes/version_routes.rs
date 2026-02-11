// Version management routes
// 版本管理相关路由

use crate::state::AppState;
use axum::{
    Router,
    routing::{get, post},
};

/// Version management routes
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/version", get(crate::api::handlers::version::get_version))
        .route(
            "/version/check-update",
            get(crate::api::handlers::version::check_update),
        )
        .route(
            "/version/update",
            post(crate::api::handlers::version::update_ccr),
        )
}
