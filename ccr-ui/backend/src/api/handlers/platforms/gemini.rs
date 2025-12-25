// Gemini CLI API 处理器
//
// 使用统一的响应工具模块减少重复代码

use axum::{Json, extract::Path, response::IntoResponse};
use serde_json::json;
use std::collections::HashMap;

use crate::api::handlers::response::{bad_request, internal_error, ok, ok_message};
use crate::managers::config::gemini_manager::GeminiConfigManager;
use crate::models::platforms::gemini::{GeminiConfig, GeminiMcpServer, GeminiMcpServerRequest};

const PLATFORM: &str = "Gemini";

// ============ 辅助宏 ============

/// 初始化 Manager 并处理错误
macro_rules! with_gemini_manager {
    ($body:expr) => {
        match GeminiConfigManager::default() {
            Ok(manager) => $body(manager),
            Err(e) => internal_error(format!("初始化 {} 配置管理器失败: {}", PLATFORM, e))
                .into_response(),
        }
    };
}

// ============ MCP 服务器管理 ============

/// GET /api/gemini/mcp - 列出所有 MCP 服务器
pub async fn list_gemini_mcp_servers() -> impl IntoResponse {
    with_gemini_manager!(|manager: GeminiConfigManager| {
        match manager.list_mcp_servers() {
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
                ok(servers_vec).into_response()
            }
            Err(e) => internal_error(format!("读取 MCP 服务器失败: {}", e)).into_response(),
        }
    })
}

/// POST /api/gemini/mcp - 添加 MCP 服务器
pub async fn add_gemini_mcp_server(
    Json(request): Json<GeminiMcpServerRequest>,
) -> impl IntoResponse {
    with_gemini_manager!(|manager: GeminiConfigManager| {
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
            Ok(()) => ok_message(format!("MCP 服务器 '{}' 添加成功", request.name)).into_response(),
            Err(e) => bad_request(format!("添加 MCP 服务器失败: {}", e)).into_response(),
        }
    })
}

/// PUT /api/gemini/mcp/:name - 更新 MCP 服务器
pub async fn update_gemini_mcp_server(
    Path(name): Path<String>,
    Json(request): Json<GeminiMcpServerRequest>,
) -> impl IntoResponse {
    with_gemini_manager!(|manager: GeminiConfigManager| {
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
            Ok(()) => ok_message(format!("MCP 服务器 '{}' 更新成功", name)).into_response(),
            Err(e) => bad_request(format!("更新 MCP 服务器失败: {}", e)).into_response(),
        }
    })
}

/// DELETE /api/gemini/mcp/:name - 删除 MCP 服务器
pub async fn delete_gemini_mcp_server(Path(name): Path<String>) -> impl IntoResponse {
    with_gemini_manager!(|manager: GeminiConfigManager| {
        match manager.delete_mcp_server(&name) {
            Ok(()) => ok_message(format!("MCP 服务器 '{}' 删除成功", name)).into_response(),
            Err(e) => bad_request(format!("删除 MCP 服务器失败: {}", e)).into_response(),
        }
    })
}

// ============ 基础配置管理 ============

/// GET /api/gemini/config - 获取完整配置
pub async fn get_gemini_config() -> impl IntoResponse {
    with_gemini_manager!(|manager: GeminiConfigManager| {
        match manager.get_config() {
            Ok(config) => ok(config).into_response(),
            Err(e) => internal_error(format!("读取配置失败: {}", e)).into_response(),
        }
    })
}

/// PUT /api/gemini/config - 更新完整配置
pub async fn update_gemini_config(Json(config): Json<GeminiConfig>) -> impl IntoResponse {
    with_gemini_manager!(|manager: GeminiConfigManager| {
        match manager.update_config(&config) {
            Ok(()) => ok_message("配置更新成功").into_response(),
            Err(e) => bad_request(format!("更新配置失败: {}", e)).into_response(),
        }
    })
}
