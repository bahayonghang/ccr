// Qwen CLI API 处理器

use axum::{Json, extract::Path, http::StatusCode, response::IntoResponse};
use serde_json::json;

use crate::managers::config::qwen_manager::QwenConfigManager;
use crate::models::platforms::qwen::{QwenConfig, QwenMcpServer, QwenMcpServerRequest};
use std::collections::HashMap;

// ============ MCP 服务器管理 ============

/// GET /api/qwen/mcp - 列出所有 MCP 服务器
pub async fn list_qwen_mcp_servers() -> impl IntoResponse {
    match QwenConfigManager::default() {
        Ok(manager) => match manager.list_mcp_servers() {
            Ok(servers) => {
                let servers_vec: Vec<_> = servers
                    .into_iter()
                    .map(|(name, server)| {
                        json!({
                            "name": name,
                            "command": server.command,
                            "args": server.args,
                            "env": server.env,
                            "url": server.url,
                            "httpUrl": server.http_url,
                            "headers": server.headers,
                            "timeout": server.timeout,
                            "transportType": server.transport_type(),
                        })
                    })
                    .collect();

                (
                    StatusCode::OK,
                    Json(json!({
                        "success": true,
                        "data": servers_vec,
                        "message": null
                    })),
                )
            }
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "data": null,
                    "message": format!("读取 MCP 服务器失败: {}", e)
                })),
            ),
        },
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("初始化配置管理器失败: {}", e)
            })),
        ),
    }
}

/// POST /api/qwen/mcp - 添加 MCP 服务器
pub async fn add_qwen_mcp_server(Json(request): Json<QwenMcpServerRequest>) -> impl IntoResponse {
    match QwenConfigManager::default() {
        Ok(manager) => {
            let server = QwenMcpServer {
                command: request.command,
                args: request.args,
                env: request.env,
                url: request.url,
                http_url: request.http_url,
                headers: request.headers,
                timeout: request.timeout,
                other: HashMap::new(),
            };

            match manager.add_mcp_server(request.name.clone(), server) {
                Ok(()) => (
                    StatusCode::OK,
                    Json(json!({
                        "success": true,
                        "data": null,
                        "message": format!("MCP 服务器 '{}' 添加成功", request.name)
                    })),
                ),
                Err(e) => (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "success": false,
                        "data": null,
                        "message": format!("添加 MCP 服务器失败: {}", e)
                    })),
                ),
            }
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("初始化配置管理器失败: {}", e)
            })),
        ),
    }
}

/// PUT /api/qwen/mcp/:name - 更新 MCP 服务器
pub async fn update_qwen_mcp_server(
    Path(name): Path<String>,
    Json(request): Json<QwenMcpServerRequest>,
) -> impl IntoResponse {
    match QwenConfigManager::default() {
        Ok(manager) => {
            let server = QwenMcpServer {
                command: request.command,
                args: request.args,
                env: request.env,
                url: request.url,
                http_url: request.http_url,
                headers: request.headers,
                timeout: request.timeout,
                other: HashMap::new(),
            };

            match manager.update_mcp_server(&name, server) {
                Ok(()) => (
                    StatusCode::OK,
                    Json(json!({
                        "success": true,
                        "data": null,
                        "message": format!("MCP 服务器 '{}' 更新成功", name)
                    })),
                ),
                Err(e) => (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "success": false,
                        "data": null,
                        "message": format!("更新 MCP 服务器失败: {}", e)
                    })),
                ),
            }
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("初始化配置管理器失败: {}", e)
            })),
        ),
    }
}

/// DELETE /api/qwen/mcp/:name - 删除 MCP 服务器
pub async fn delete_qwen_mcp_server(Path(name): Path<String>) -> impl IntoResponse {
    match QwenConfigManager::default() {
        Ok(manager) => match manager.delete_mcp_server(&name) {
            Ok(()) => (
                StatusCode::OK,
                Json(json!({
                    "success": true,
                    "data": null,
                    "message": format!("MCP 服务器 '{}' 删除成功", name)
                })),
            ),
            Err(e) => (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "data": null,
                    "message": format!("删除 MCP 服务器失败: {}", e)
                })),
            ),
        },
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("初始化配置管理器失败: {}", e)
            })),
        ),
    }
}

// ============ 基础配置管理 ============

/// GET /api/qwen/config - 获取完整配置
pub async fn get_qwen_config() -> impl IntoResponse {
    match QwenConfigManager::default() {
        Ok(manager) => match manager.get_config() {
            Ok(config) => (
                StatusCode::OK,
                Json(json!({
                    "success": true,
                    "data": config,
                    "message": null
                })),
            ),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "data": null,
                    "message": format!("读取配置失败: {}", e)
                })),
            ),
        },
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("初始化配置管理器失败: {}", e)
            })),
        ),
    }
}

/// PUT /api/qwen/config - 更新完整配置
pub async fn update_qwen_config(Json(config): Json<QwenConfig>) -> impl IntoResponse {
    match QwenConfigManager::default() {
        Ok(manager) => match manager.update_config(&config) {
            Ok(()) => (
                StatusCode::OK,
                Json(json!({
                    "success": true,
                    "data": null,
                    "message": "配置更新成功"
                })),
            ),
            Err(e) => (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "data": null,
                    "message": format!("更新配置失败: {}", e)
                })),
            ),
        },
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("初始化配置管理器失败: {}", e)
            })),
        ),
    }
}
