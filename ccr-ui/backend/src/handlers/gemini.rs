// Gemini CLI API 处理器

use axum::{Json, extract::Path, http::StatusCode, response::IntoResponse};
use serde_json::json;

use crate::gemini_config_manager::GeminiConfigManager;
use crate::gemini_models::{GeminiConfig, GeminiMcpServer, GeminiMcpServerRequest};
use std::collections::HashMap;

// ============ MCP 服务器管理 ============

/// GET /api/gemini/mcp - 列出所有 MCP 服务器
pub async fn list_gemini_mcp_servers() -> impl IntoResponse {
    match GeminiConfigManager::default() {
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
                            "cwd": server.cwd,
                            "timeout": server.timeout,
                            "trust": server.trust,
                            "includeTools": server.include_tools,
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

/// POST /api/gemini/mcp - 添加 MCP 服务器
pub async fn add_gemini_mcp_server(
    Json(request): Json<GeminiMcpServerRequest>,
) -> impl IntoResponse {
    match GeminiConfigManager::default() {
        Ok(manager) => {
            let server = GeminiMcpServer {
                command: request.command,
                args: request.args,
                env: request.env,
                cwd: request.cwd,
                timeout: request.timeout,
                trust: request.trust,
                include_tools: request.include_tools,
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

/// PUT /api/gemini/mcp/:name - 更新 MCP 服务器
pub async fn update_gemini_mcp_server(
    Path(name): Path<String>,
    Json(request): Json<GeminiMcpServerRequest>,
) -> impl IntoResponse {
    match GeminiConfigManager::default() {
        Ok(manager) => {
            let server = GeminiMcpServer {
                command: request.command,
                args: request.args,
                env: request.env,
                cwd: request.cwd,
                timeout: request.timeout,
                trust: request.trust,
                include_tools: request.include_tools,
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

/// DELETE /api/gemini/mcp/:name - 删除 MCP 服务器
pub async fn delete_gemini_mcp_server(Path(name): Path<String>) -> impl IntoResponse {
    match GeminiConfigManager::default() {
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

/// GET /api/gemini/config - 获取完整配置
pub async fn get_gemini_config() -> impl IntoResponse {
    match GeminiConfigManager::default() {
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

/// PUT /api/gemini/config - 更新完整配置
pub async fn update_gemini_config(Json(config): Json<GeminiConfig>) -> impl IntoResponse {
    match GeminiConfigManager::default() {
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
