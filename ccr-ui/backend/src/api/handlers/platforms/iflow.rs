// iFlow CLI API 处理器（Stub 实现）
//
// 使用统一的响应工具模块

use axum::{Json, extract::Path, http::StatusCode, response::IntoResponse};
use serde_json::json;

use crate::api::handlers::response::ok;

/// Stub 响应 - 功能未实现
fn not_implemented(feature: &str) -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({
            "success": false,
            "data": null,
            "message": format!("{} 功能待实现", feature)
        })),
    )
}

/// Stub 响应 - 返回空列表
fn empty_list(feature: &str) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({
            "success": true,
            "data": [],
            "message": format!("{} 功能待实现", feature)
        })),
    )
}

/// Stub 响应 - 返回空的 agents/commands 结构
fn empty_folder_list(feature: &str) -> impl IntoResponse {
    ok(json!({
        "agents": [],
        "folders": [],
        "message": format!("{} 功能待实现", feature)
    }))
}

// ============ MCP 服务器管理（Stub）============

#[allow(dead_code)]
pub async fn list_iflow_mcp_servers() -> impl IntoResponse {
    empty_list("iFlow MCP 服务器")
}

#[allow(dead_code)]
pub async fn add_iflow_mcp_server(Json(_request): Json<serde_json::Value>) -> impl IntoResponse {
    not_implemented("iFlow MCP 服务器添加")
}

#[allow(dead_code)]
pub async fn update_iflow_mcp_server(
    Path(_name): Path<String>,
    Json(_request): Json<serde_json::Value>,
) -> impl IntoResponse {
    not_implemented("iFlow MCP 服务器更新")
}

#[allow(dead_code)]
pub async fn delete_iflow_mcp_server(Path(_name): Path<String>) -> impl IntoResponse {
    not_implemented("iFlow MCP 服务器删除")
}

// ============ Agents 管理（Stub）============

#[allow(dead_code)]
pub async fn list_iflow_agents() -> impl IntoResponse {
    empty_folder_list("iFlow Agents")
}

#[allow(dead_code)]
pub async fn add_iflow_agent(Json(_request): Json<serde_json::Value>) -> impl IntoResponse {
    not_implemented("iFlow Agents 添加")
}

#[allow(dead_code)]
pub async fn update_iflow_agent(
    Path(_name): Path<String>,
    Json(_request): Json<serde_json::Value>,
) -> impl IntoResponse {
    not_implemented("iFlow Agents 更新")
}

#[allow(dead_code)]
pub async fn delete_iflow_agent(Path(_name): Path<String>) -> impl IntoResponse {
    not_implemented("iFlow Agents 删除")
}

#[allow(dead_code)]
pub async fn toggle_iflow_agent(Path(_name): Path<String>) -> impl IntoResponse {
    not_implemented("iFlow Agents 切换")
}

// ============ Slash Commands 管理（Stub）============

#[allow(dead_code)]
pub async fn list_iflow_slash_commands() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({
            "success": true,
            "data": {
                "commands": [],
                "folders": []
            },
            "message": "iFlow Slash Commands 功能待实现"
        })),
    )
}

#[allow(dead_code)]
pub async fn add_iflow_slash_command(Json(_request): Json<serde_json::Value>) -> impl IntoResponse {
    not_implemented("iFlow Slash Commands 添加")
}

#[allow(dead_code)]
pub async fn update_iflow_slash_command(
    Path(_name): Path<String>,
    Json(_request): Json<serde_json::Value>,
) -> impl IntoResponse {
    not_implemented("iFlow Slash Commands 更新")
}

#[allow(dead_code)]
pub async fn delete_iflow_slash_command(Path(_name): Path<String>) -> impl IntoResponse {
    not_implemented("iFlow Slash Commands 删除")
}

#[allow(dead_code)]
pub async fn toggle_iflow_slash_command(Path(_name): Path<String>) -> impl IntoResponse {
    not_implemented("iFlow Slash Commands 切换")
}

// ============ Plugins 管理（Stub）============

#[allow(dead_code)]
pub async fn list_iflow_plugins() -> impl IntoResponse {
    empty_list("iFlow Plugins")
}

#[allow(dead_code)]
pub async fn add_iflow_plugin(Json(_request): Json<serde_json::Value>) -> impl IntoResponse {
    not_implemented("iFlow Plugins 添加")
}

#[allow(dead_code)]
pub async fn update_iflow_plugin(
    Path(_id): Path<String>,
    Json(_request): Json<serde_json::Value>,
) -> impl IntoResponse {
    not_implemented("iFlow Plugins 更新")
}

#[allow(dead_code)]
pub async fn delete_iflow_plugin(Path(_id): Path<String>) -> impl IntoResponse {
    not_implemented("iFlow Plugins 删除")
}

#[allow(dead_code)]
pub async fn toggle_iflow_plugin(Path(_id): Path<String>) -> impl IntoResponse {
    not_implemented("iFlow Plugins 切换")
}
