// Qwen CLI API 处理器
//
// MCP/Config 使用 QwenConfigManager，Slash Commands 委托 toml_commands 共享模块

use axum::{Json, extract::Path, response::IntoResponse};
use serde_json::json;
use std::collections::HashMap;

use crate::api::handlers::response::{bad_request, internal_error, ok, ok_message};
use crate::managers::config::qwen_manager::QwenConfigManager;
use crate::models::platforms::qwen::{QwenConfig, QwenMcpServer, QwenMcpServerRequest};

use super::toml_commands;

const PLATFORM: &str = "Qwen";

// ============ MCP 服务器管理 ============

/// GET /api/qwen/mcp - 列出所有 MCP 服务器
pub async fn list_qwen_mcp_servers() -> impl IntoResponse {
    crate::with_manager!(QwenConfigManager, PLATFORM, |manager: QwenConfigManager| {
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
                            "url": server.url,
                            "httpUrl": server.http_url,
                            "headers": server.headers,
                            "timeout": server.timeout,
                            "transportType": server.transport_type(),
                        })
                    })
                    .collect();
                ok(servers_vec).into_response()
            }
            Err(e) => internal_error(format!("读取 MCP 服务器失败: {}", e)).into_response(),
        }
    })
}

/// POST /api/qwen/mcp - 添加 MCP 服务器
pub async fn add_qwen_mcp_server(Json(request): Json<QwenMcpServerRequest>) -> impl IntoResponse {
    crate::with_manager!(QwenConfigManager, PLATFORM, |manager: QwenConfigManager| {
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
            Ok(()) => ok_message(format!("MCP 服务器 '{}' 添加成功", request.name)).into_response(),
            Err(e) => bad_request(format!("添加 MCP 服务器失败: {}", e)).into_response(),
        }
    })
}

/// PUT /api/qwen/mcp/:name - 更新 MCP 服务器
pub async fn update_qwen_mcp_server(
    Path(name): Path<String>,
    Json(request): Json<QwenMcpServerRequest>,
) -> impl IntoResponse {
    crate::with_manager!(QwenConfigManager, PLATFORM, |manager: QwenConfigManager| {
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
            Ok(()) => ok_message(format!("MCP 服务器 '{}' 更新成功", name)).into_response(),
            Err(e) => bad_request(format!("更新 MCP 服务器失败: {}", e)).into_response(),
        }
    })
}

/// DELETE /api/qwen/mcp/:name - 删除 MCP 服务器
pub async fn delete_qwen_mcp_server(Path(name): Path<String>) -> impl IntoResponse {
    crate::with_manager!(QwenConfigManager, PLATFORM, |manager: QwenConfigManager| {
        match manager.delete_mcp_server(&name) {
            Ok(()) => ok_message(format!("MCP 服务器 '{}' 删除成功", name)).into_response(),
            Err(e) => bad_request(format!("删除 MCP 服务器失败: {}", e)).into_response(),
        }
    })
}

// ============ 基础配置管理 ============

/// GET /api/qwen/config - 获取完整配置
pub async fn get_qwen_config() -> impl IntoResponse {
    crate::with_manager!(QwenConfigManager, PLATFORM, |manager: QwenConfigManager| {
        match manager.get_config() {
            Ok(config) => ok(config).into_response(),
            Err(e) => internal_error(format!("读取配置失败: {}", e)).into_response(),
        }
    })
}

/// PUT /api/qwen/config - 更新完整配置
pub async fn update_qwen_config(Json(config): Json<QwenConfig>) -> impl IntoResponse {
    crate::with_manager!(QwenConfigManager, PLATFORM, |manager: QwenConfigManager| {
        match manager.update_config(&config) {
            Ok(()) => ok_message("配置更新成功").into_response(),
            Err(e) => bad_request(format!("更新配置失败: {}", e)).into_response(),
        }
    })
}

// ============ 斜杠命令管理（委托 toml_commands 共享模块）============

/// GET /api/qwen/slash-commands - 列出所有斜杠命令
pub async fn list_qwen_slash_commands() -> impl IntoResponse {
    toml_commands::list_slash_commands("qwen")
}

/// POST /api/qwen/slash-commands - 添加斜杠命令
pub async fn add_qwen_slash_command(
    Json(req): Json<toml_commands::UiSlashCommandRequest>,
) -> impl IntoResponse {
    toml_commands::add_slash_command("qwen", PLATFORM, req)
}

/// PUT /api/qwen/slash-commands/:name - 更新斜杠命令
pub async fn update_qwen_slash_command(
    Path(name): Path<String>,
    Json(req): Json<toml_commands::UiSlashCommandRequest>,
) -> impl IntoResponse {
    toml_commands::update_slash_command("qwen", PLATFORM, &name, req)
}

/// DELETE /api/qwen/slash-commands/:name - 删除斜杠命令
pub async fn delete_qwen_slash_command(Path(name): Path<String>) -> impl IntoResponse {
    toml_commands::delete_slash_command("qwen", PLATFORM, &name)
}

/// PATCH /api/qwen/slash-commands/:name/toggle - 切换启用/禁用（不支持）
pub async fn toggle_qwen_slash_command(Path(_name): Path<String>) -> impl IntoResponse {
    toml_commands::toggle_not_supported("Qwen Code")
}
