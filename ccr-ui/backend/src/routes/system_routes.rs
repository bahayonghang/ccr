// System information routes
// 系统信息相关路由

use axum::{Router, routing::get};

/// System information routes
pub fn routes() -> Router {
    Router::new().route(
        "/system",
        get(crate::api::handlers::system::get_system_info),
    )
}
