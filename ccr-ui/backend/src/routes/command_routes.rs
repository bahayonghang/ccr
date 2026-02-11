// Command execution routes
// 命令执行相关路由

use crate::state::AppState;
use axum::{
    Router,
    routing::{get, post},
};

/// Command execution routes
pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/command/execute",
            post(crate::api::handlers::command::execute_command),
        )
        .route(
            "/command/execute/stream",
            post(crate::api::handlers::command::execute_command_stream),
        )
        .route(
            "/command/list",
            get(crate::api::handlers::command::list_commands),
        )
        .route(
            "/command/help/{command}",
            get(crate::api::handlers::command::get_command_help),
        )
}
