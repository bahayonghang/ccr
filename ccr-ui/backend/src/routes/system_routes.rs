// System information routes
// 系统信息相关路由

use crate::state::AppState;
use axum::{Router, routing::get};

/// System information routes
pub fn routes() -> Router<AppState> {
    Router::new().route(
        "/system",
        get(crate::api::handlers::system::get_system_info),
    )
}
