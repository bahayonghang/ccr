// MCP 预设 API 端点
// 提供 MCP 服务器预设模板的管理功能

use axum::{Json, extract::Path, response::IntoResponse};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

use ccr::managers::mcp_preset_manager::{McpPresetManager, McpSyncManager, get_builtin_presets};
use ccr::models::Platform;

use crate::models::api::ApiResponse;

/// 预设请求/响应模型
#[derive(Debug, Serialize, Deserialize)]
pub struct McpPresetResponse {
    pub id: String,
    pub name: String,
    pub description: String,
    pub command: Option<String>,
    pub args: Vec<String>,
    pub tags: Vec<String>,
    pub homepage: Option<String>,
    pub docs: Option<String>,
    pub requires_api_key: bool,
    pub api_key_env: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstallPresetRequest {
    pub preset_id: String,
    pub platforms: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstallResult {
    pub platform: String,
    pub success: bool,
    pub message: Option<String>,
}

/// GET /api/mcp/presets - 获取所有内置 MCP 预设
pub async fn list_presets() -> impl IntoResponse {
    let presets = get_builtin_presets();

    let preset_responses: Vec<McpPresetResponse> = presets
        .into_iter()
        .map(|p| McpPresetResponse {
            id: p.id,
            name: p.name,
            description: p.description,
            command: p.server.command,
            args: p.server.args,
            tags: p.tags,
            homepage: p.homepage,
            docs: p.docs,
            requires_api_key: p.requires_api_key,
            api_key_env: p.api_key_env,
        })
        .collect();

    ApiResponse::success(preset_responses)
}

/// GET /api/mcp/presets/:id - 获取单个预设详情
pub async fn get_preset(Path(id): Path<String>) -> impl IntoResponse {
    let presets = get_builtin_presets();

    match presets.into_iter().find(|p| p.id == id) {
        Some(p) => {
            let response = McpPresetResponse {
                id: p.id,
                name: p.name,
                description: p.description,
                command: p.server.command,
                args: p.server.args,
                tags: p.tags,
                homepage: p.homepage,
                docs: p.docs,
                requires_api_key: p.requires_api_key,
                api_key_env: p.api_key_env,
            };
            ApiResponse::success(response)
        }
        None => ApiResponse::error(format!("Preset '{}' not found", id)),
    }
}

/// POST /api/mcp/presets/:id/install - 安装预设到指定平台
pub async fn install_preset(
    Path(id): Path<String>,
    Json(req): Json<InstallPresetRequest>,
) -> impl IntoResponse {
    let sync_manager = McpSyncManager::new();

    // 解析目标平台
    let mut target_platforms: Vec<Platform> = Vec::new();
    for platform_str in &req.platforms {
        match platform_str.parse::<Platform>() {
            Ok(p) => target_platforms.push(p),
            Err(_) => {
                return Json(ApiResponse::error(format!(
                    "Invalid platform: {}",
                    platform_str
                )));
            }
        }
    }

    // 如果没有指定平台，默认安装到 Claude
    if target_platforms.is_empty() {
        target_platforms.push(Platform::Claude);
    }

    // 准备环境变量
    let custom_env = if req.env.is_empty() {
        None
    } else {
        Some(req.env.clone())
    };

    // 执行同步
    match sync_manager.sync_preset_to_all(&id, custom_env, &target_platforms) {
        Ok(results) => {
            let install_results: Vec<InstallResult> = results
                .into_iter()
                .map(|(platform, result)| InstallResult {
                    platform: platform.short_name().to_string(),
                    success: result.is_ok(),
                    message: result.err().map(|e| e.to_string()),
                })
                .collect();

            let all_success = install_results.iter().all(|r| r.success);
            if all_success {
                Json(ApiResponse::success(json!({
                    "message": format!("Preset '{}' installed successfully", id),
                    "results": install_results
                })))
            } else {
                Json(ApiResponse::success(json!({
                    "message": format!("Preset '{}' installed with some errors", id),
                    "results": install_results
                })))
            }
        }
        Err(e) => Json(ApiResponse::error(format!(
            "Failed to install preset: {}",
            e
        ))),
    }
}

/// POST /api/mcp/presets/install-single - 安装预设到单个平台
pub async fn install_preset_single(Json(req): Json<InstallPresetRequest>) -> impl IntoResponse {
    // 使用第一个平台
    let platform_str = req
        .platforms
        .first()
        .map(|s| s.as_str())
        .unwrap_or("claude");

    let platform = match platform_str.parse::<Platform>() {
        Ok(p) => p,
        Err(_) => {
            return Json(ApiResponse::error(format!(
                "Invalid platform: {}",
                platform_str
            )));
        }
    };

    let manager = match McpPresetManager::new(platform) {
        Ok(m) => m,
        Err(e) => {
            return Json(ApiResponse::error(format!(
                "Failed to create preset manager: {}",
                e
            )));
        }
    };

    let custom_env = if req.env.is_empty() {
        None
    } else {
        Some(req.env.clone())
    };

    match manager.install_preset(&req.preset_id, custom_env) {
        Ok(_) => Json(ApiResponse::success(json!({
            "message": format!("Preset '{}' installed to {} successfully", req.preset_id, platform_str),
            "platform": platform_str
        }))),
        Err(e) => Json(ApiResponse::error(format!(
            "Failed to install preset: {}",
            e
        ))),
    }
}

// ===================================
// MCP Sync API Endpoints
// ===================================

#[derive(Debug, Serialize, Deserialize)]
pub struct McpServerInfo {
    pub name: String,
    pub command: Option<String>,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncRequest {
    pub platforms: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncResult {
    pub platform: String,
    pub success: bool,
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncResponse {
    pub message: String,
    pub results: Vec<SyncResult>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncAllResponse {
    pub message: String,
    pub servers: HashMap<String, Vec<SyncResult>>,
}

/// GET /api/mcp/sync/source - 获取源平台（Claude）的 MCP 服务器列表
pub async fn list_source_mcp_servers() -> impl IntoResponse {
    let sync_manager = McpSyncManager::new();

    match sync_manager.list_source_mcp_servers() {
        Ok(servers) => {
            let server_list: Vec<McpServerInfo> = servers
                .into_iter()
                .map(|(name, spec)| McpServerInfo {
                    name,
                    command: spec.command,
                    args: spec.args,
                    env: spec.env,
                })
                .collect();

            ApiResponse::success(server_list)
        }
        Err(e) => ApiResponse::error(format!("Failed to list source MCP servers: {}", e)),
    }
}

/// POST /api/mcp/sync/:name - 同步指定 MCP 服务器到目标平台
pub async fn sync_mcp_server(
    Path(name): Path<String>,
    Json(req): Json<SyncRequest>,
) -> impl IntoResponse {
    let sync_manager = McpSyncManager::new();

    // 解析目标平台
    let mut target_platforms: Vec<Platform> = Vec::new();
    for platform_str in &req.platforms {
        match platform_str.parse::<Platform>() {
            Ok(p) => target_platforms.push(p),
            Err(_) => {
                return Json(ApiResponse::error(format!(
                    "Invalid platform: {}",
                    platform_str
                )));
            }
        }
    }

    // 执行同步
    match sync_manager.sync_mcp_server(&name, &target_platforms) {
        Ok(results) => {
            let sync_results: Vec<SyncResult> = results
                .into_iter()
                .map(|(platform, result)| SyncResult {
                    platform: platform.short_name().to_string(),
                    success: result.is_ok(),
                    message: result.err().map(|e| e.to_string()),
                })
                .collect();

            let all_success = sync_results.iter().all(|r| r.success);
            Json(ApiResponse::success(SyncResponse {
                message: if all_success {
                    format!("MCP server '{}' synced successfully", name)
                } else {
                    format!("MCP server '{}' synced with some errors", name)
                },
                results: sync_results,
            }))
        }
        Err(e) => Json(ApiResponse::error(format!(
            "Failed to sync MCP server: {}",
            e
        ))),
    }
}

/// POST /api/mcp/sync/all - 同步所有 MCP 服务器到目标平台
pub async fn sync_all_mcp_servers(Json(req): Json<SyncRequest>) -> impl IntoResponse {
    let sync_manager = McpSyncManager::new();

    // 解析目标平台
    let mut target_platforms: Vec<Platform> = Vec::new();
    for platform_str in &req.platforms {
        match platform_str.parse::<Platform>() {
            Ok(p) => target_platforms.push(p),
            Err(_) => {
                return Json(ApiResponse::error(format!(
                    "Invalid platform: {}",
                    platform_str
                )));
            }
        }
    }

    // 执行同步
    match sync_manager.sync_all_mcp_servers(&target_platforms) {
        Ok(results) => {
            let mut servers_results: HashMap<String, Vec<SyncResult>> = HashMap::new();

            for (server_name, platform_results) in results {
                let sync_results: Vec<SyncResult> = platform_results
                    .into_iter()
                    .map(|(platform, result)| SyncResult {
                        platform: platform.short_name().to_string(),
                        success: result.is_ok(),
                        message: result.err().map(|e| e.to_string()),
                    })
                    .collect();
                servers_results.insert(server_name, sync_results);
            }

            Json(ApiResponse::success(SyncAllResponse {
                message: format!("Synced {} MCP servers", servers_results.len()),
                servers: servers_results,
            }))
        }
        Err(e) => Json(ApiResponse::error(format!(
            "Failed to sync all MCP servers: {}",
            e
        ))),
    }
}
