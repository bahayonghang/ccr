// Codex CLI 配置管理 API 处理器

use axum::{Json, extract::Path, http::StatusCode, response::IntoResponse};
use serde_json::json;

use crate::codex_config_manager::CodexConfigManager;
use crate::codex_models::*;

// ============ MCP 服务器管理 ============

/// GET /api/codex/mcp - 列出所有 Codex MCP 服务器
pub async fn list_codex_mcp_servers() -> impl IntoResponse {
    let manager = match CodexConfigManager::default() {
        Ok(m) => m,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "data": null,
                    "message": format!("初始化 Codex 配置管理器失败: {}", e)
                })),
            )
                .into_response();
        }
    };

    match manager.list_mcp_servers() {
        Ok(servers) => (
            StatusCode::OK,
            Json(json!({
                "success": true,
                "data": CodexMcpListResponse { servers },
                "message": null
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("读取 Codex MCP 服务器失败: {}", e)
            })),
        )
            .into_response(),
    }
}

/// POST /api/codex/mcp - 添加 Codex MCP 服务器
pub async fn add_codex_mcp_server(Json(req): Json<CodexMcpServerRequest>) -> impl IntoResponse {
    // 验证请求：至少需要 command（STDIO）或 url（HTTP）
    if req.command.is_none() && req.url.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "data": null,
                "message": "必须提供 command（STDIO 服务器）或 url（HTTP 服务器）"
            })),
        )
            .into_response();
    }

    let manager = match CodexConfigManager::default() {
        Ok(m) => m,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "data": null,
                    "message": format!("初始化 Codex 配置管理器失败: {}", e)
                })),
            )
                .into_response();
        }
    };

    // 从 JSON 中提取服务器名称
    let name = req
        .command
        .clone()
        .or(req.url.clone())
        .unwrap_or_else(|| "unknown".to_string());

    let server: CodexMcpServer = req.into();

    match manager.add_mcp_server(name.clone(), server) {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({
                "success": true,
                "data": format!("Codex MCP 服务器 '{}' 已成功添加", name),
                "message": null
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("添加 Codex MCP 服务器失败: {}", e)
            })),
        )
            .into_response(),
    }
}

/// PUT /api/codex/mcp/:name - 更新 Codex MCP 服务器
pub async fn update_codex_mcp_server(
    Path(name): Path<String>,
    Json(req): Json<CodexMcpServerRequest>,
) -> impl IntoResponse {
    let manager = match CodexConfigManager::default() {
        Ok(m) => m,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "data": null,
                    "message": format!("初始化 Codex 配置管理器失败: {}", e)
                })),
            )
                .into_response();
        }
    };

    let server: CodexMcpServer = req.into();

    match manager.update_mcp_server(&name, server) {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({
                "success": true,
                "data": format!("Codex MCP 服务器 '{}' 已成功更新", name),
                "message": null
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("更新 Codex MCP 服务器失败: {}", e)
            })),
        )
            .into_response(),
    }
}

/// DELETE /api/codex/mcp/:name - 删除 Codex MCP 服务器
pub async fn delete_codex_mcp_server(Path(name): Path<String>) -> impl IntoResponse {
    let manager = match CodexConfigManager::default() {
        Ok(m) => m,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "data": null,
                    "message": format!("初始化 Codex 配置管理器失败: {}", e)
                })),
            )
                .into_response();
        }
    };

    match manager.delete_mcp_server(&name) {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({
                "success": true,
                "data": format!("Codex MCP 服务器 '{}' 已成功删除", name),
                "message": null
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("删除 Codex MCP 服务器失败: {}", e)
            })),
        )
            .into_response(),
    }
}

// ============ Profile 管理 ============

/// GET /api/codex/profiles - 列出所有 Codex Profiles
pub async fn list_codex_profiles() -> impl IntoResponse {
    let manager = match CodexConfigManager::default() {
        Ok(m) => m,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "data": null,
                    "message": format!("初始化 Codex 配置管理器失败: {}", e)
                })),
            )
                .into_response();
        }
    };

    match manager.list_profiles() {
        Ok(profiles) => (
            StatusCode::OK,
            Json(json!({
                "success": true,
                "data": CodexProfileListResponse { profiles },
                "message": null
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("读取 Codex Profiles 失败: {}", e)
            })),
        )
            .into_response(),
    }
}

/// POST /api/codex/profiles - 添加 Codex Profile
pub async fn add_codex_profile(Json(req): Json<CodexProfileRequest>) -> impl IntoResponse {
    let manager = match CodexConfigManager::default() {
        Ok(m) => m,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "data": null,
                    "message": format!("初始化 Codex 配置管理器失败: {}", e)
                })),
            )
                .into_response();
        }
    };

    // 从请求中提取 profile 名称（假设在 model 字段中）
    let name = req.model.clone().unwrap_or_else(|| "default".to_string());

    let profile: CodexProfile = req.into();

    match manager.add_profile(name.clone(), profile) {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({
                "success": true,
                "data": format!("Codex Profile '{}' 已成功添加", name),
                "message": null
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("添加 Codex Profile 失败: {}", e)
            })),
        )
            .into_response(),
    }
}

/// PUT /api/codex/profiles/:name - 更新 Codex Profile
pub async fn update_codex_profile(
    Path(name): Path<String>,
    Json(req): Json<CodexProfileRequest>,
) -> impl IntoResponse {
    let manager = match CodexConfigManager::default() {
        Ok(m) => m,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "data": null,
                    "message": format!("初始化 Codex 配置管理器失败: {}", e)
                })),
            )
                .into_response();
        }
    };

    let profile: CodexProfile = req.into();

    match manager.update_profile(&name, profile) {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({
                "success": true,
                "data": format!("Codex Profile '{}' 已成功更新", name),
                "message": null
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("更新 Codex Profile 失败: {}", e)
            })),
        )
            .into_response(),
    }
}

/// DELETE /api/codex/profiles/:name - 删除 Codex Profile
pub async fn delete_codex_profile(Path(name): Path<String>) -> impl IntoResponse {
    let manager = match CodexConfigManager::default() {
        Ok(m) => m,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "data": null,
                    "message": format!("初始化 Codex 配置管理器失败: {}", e)
                })),
            )
                .into_response();
        }
    };

    match manager.delete_profile(&name) {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({
                "success": true,
                "data": format!("Codex Profile '{}' 已成功删除", name),
                "message": null
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("删除 Codex Profile 失败: {}", e)
            })),
        )
            .into_response(),
    }
}

// ============ 基础配置管理 ============

/// GET /api/codex/config - 获取完整的 Codex 配置
pub async fn get_codex_config() -> impl IntoResponse {
    let manager = match CodexConfigManager::default() {
        Ok(m) => m,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "data": null,
                    "message": format!("初始化 Codex 配置管理器失败: {}", e)
                })),
            )
                .into_response();
        }
    };

    match manager.read_config() {
        Ok(config) => (
            StatusCode::OK,
            Json(json!({
                "success": true,
                "data": CodexConfigResponse { config },
                "message": null
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("读取 Codex 配置失败: {}", e)
            })),
        )
            .into_response(),
    }
}

/// PUT /api/codex/config - 更新 Codex 基础配置
pub async fn update_codex_base_config(Json(config): Json<CodexConfig>) -> impl IntoResponse {
    let manager = match CodexConfigManager::default() {
        Ok(m) => m,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "data": null,
                    "message": format!("初始化 Codex 配置管理器失败: {}", e)
                })),
            )
                .into_response();
        }
    };

    match manager.update_base_config(
        config.model,
        config.model_provider,
        config.approval_policy,
        config.sandbox_mode,
        config.model_reasoning_effort,
    ) {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({
                "success": true,
                "data": "Codex 基础配置已成功更新",
                "message": null
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("更新 Codex 基础配置失败: {}", e)
            })),
        )
            .into_response(),
    }
}
