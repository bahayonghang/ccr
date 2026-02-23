// OpenCode 平台路由定义

use crate::state::AppState;
use axum::{
    Router,
    routing::{delete, get, post, put},
};

pub fn routes() -> Router<AppState> {
    Router::new()
        // Provider 路由
        .route(
            "/opencode/providers",
            get(crate::api::handlers::platforms::opencode::list_opencode_providers),
        )
        .route(
            "/opencode/providers",
            post(crate::api::handlers::platforms::opencode::add_opencode_provider),
        )
        .route(
            "/opencode/providers/{id}",
            put(crate::api::handlers::platforms::opencode::update_opencode_provider),
        )
        .route(
            "/opencode/providers/{id}",
            delete(crate::api::handlers::platforms::opencode::delete_opencode_provider),
        )
        // MCP 服务器路由（原生格式）
        .route(
            "/opencode/mcp",
            get(crate::api::handlers::platforms::opencode::list_opencode_mcp_servers),
        )
        .route(
            "/opencode/mcp",
            post(crate::api::handlers::platforms::opencode::add_opencode_mcp_server),
        )
        .route(
            "/opencode/mcp/{id}",
            put(crate::api::handlers::platforms::opencode::update_opencode_mcp_server),
        )
        .route(
            "/opencode/mcp/{id}",
            delete(crate::api::handlers::platforms::opencode::delete_opencode_mcp_server),
        )
        // Plugin 路由
        .route(
            "/opencode/plugins",
            get(crate::api::handlers::platforms::opencode::list_opencode_plugins),
        )
        .route(
            "/opencode/plugins",
            post(crate::api::handlers::platforms::opencode::add_opencode_plugin),
        )
        .route(
            "/opencode/plugins/{pkg}",
            delete(crate::api::handlers::platforms::opencode::delete_opencode_plugin),
        )
        // 完整配置路由
        .route(
            "/opencode/config",
            get(crate::api::handlers::platforms::opencode::get_opencode_config),
        )
}
