// OpenCode 平台 API 处理器
//
// 提供 Provider、MCP、Plugin、Config 的 CRUD 接口

use axum::{Json, extract::Path, response::IntoResponse};
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;

use crate::api::handlers::response::{bad_request, internal_error, ok, ok_message};
use crate::managers::config::opencode_manager::{
    OpenCodeConfigManager, OpenCodeMcpServer, OpenCodeModel, OpenCodeModelLimit,
    OpenCodeProvider, OpenCodeProviderOptions,
};

const PLATFORM: &str = "OpenCode";

// ============ 请求/响应结构 ============

/// Provider 请求体
#[derive(Debug, Deserialize)]
pub struct OpenCodeProviderRequest {
    pub id: String,
    pub npm: String,
    pub name: Option<String>,
    pub options: Option<OpenCodeProviderOptionsRequest>,
    pub models: Option<HashMap<String, OpenCodeModelRequest>>,
}

#[derive(Debug, Deserialize)]
pub struct OpenCodeProviderOptionsRequest {
    #[serde(rename = "baseURL")]
    pub base_url: Option<String>,
    #[serde(rename = "apiKey")]
    pub api_key: Option<String>,
    pub headers: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
pub struct OpenCodeModelRequest {
    pub name: String,
    pub limit: Option<OpenCodeModelLimitRequest>,
}

#[derive(Debug, Deserialize)]
pub struct OpenCodeModelLimitRequest {
    pub context: Option<u64>,
    pub output: Option<u64>,
}

/// MCP 服务器请求体
#[derive(Debug, Deserialize)]
pub struct OpenCodeMcpServerRequest {
    pub id: String,
    #[serde(rename = "type")]
    pub server_type: String,
    pub command: Option<Vec<String>>,
    pub environment: Option<HashMap<String, String>>,
    pub url: Option<String>,
    pub headers: Option<HashMap<String, String>>,
}

/// Plugin 请求体
#[derive(Debug, Deserialize)]
pub struct OpenCodePluginRequest {
    pub npm: String,
}

// ============ Provider 接口 ============

/// GET /api/opencode/providers - 列出所有 Provider
pub async fn list_opencode_providers() -> impl IntoResponse {
    crate::with_manager!(
        OpenCodeConfigManager,
        PLATFORM,
        |manager: OpenCodeConfigManager| {
            match manager.list_providers() {
                Ok(providers) => {
                    let list: Vec<_> = providers
                        .into_iter()
                        .map(|(id, p)| {
                            json!({
                                "id": id,
                                "npm": p.npm,
                                "name": p.name,
                                "options": {
                                    "baseURL": p.options.base_url,
                                    "apiKey": p.options.api_key,
                                    "headers": p.options.headers,
                                },
                                "models": p.models.into_iter().map(|(mid, m)| {
                                    (mid, json!({
                                        "name": m.name,
                                        "limit": m.limit.map(|l| json!({
                                            "context": l.context,
                                            "output": l.output,
                                        })),
                                    }))
                                }).collect::<serde_json::Map<_, _>>(),
                            })
                        })
                        .collect();
                    ok(list).into_response()
                }
                Err(e) => internal_error(format!("读取 Provider 列表失败: {}", e)).into_response(),
            }
        }
    )
}

/// POST /api/opencode/providers - 添加 Provider
pub async fn add_opencode_provider(
    Json(request): Json<OpenCodeProviderRequest>,
) -> impl IntoResponse {
    crate::with_manager!(
        OpenCodeConfigManager,
        PLATFORM,
        |manager: OpenCodeConfigManager| {
            let provider = build_provider_from_request(&request);
            let id = request.id.clone();
            match manager.set_provider(id.clone(), provider) {
                Ok(()) => ok_message(format!("Provider '{}' 添加成功", id)).into_response(),
                Err(e) => bad_request(format!("添加 Provider 失败: {}", e)).into_response(),
            }
        }
    )
}

/// PUT /api/opencode/providers/:id - 更新 Provider
pub async fn update_opencode_provider(
    Path(id): Path<String>,
    Json(request): Json<OpenCodeProviderRequest>,
) -> impl IntoResponse {
    crate::with_manager!(
        OpenCodeConfigManager,
        PLATFORM,
        |manager: OpenCodeConfigManager| {
            let provider = build_provider_from_request(&request);
            match manager.set_provider(id.clone(), provider) {
                Ok(()) => ok_message(format!("Provider '{}' 更新成功", id)).into_response(),
                Err(e) => bad_request(format!("更新 Provider 失败: {}", e)).into_response(),
            }
        }
    )
}

/// DELETE /api/opencode/providers/:id - 删除 Provider
pub async fn delete_opencode_provider(Path(id): Path<String>) -> impl IntoResponse {
    crate::with_manager!(
        OpenCodeConfigManager,
        PLATFORM,
        |manager: OpenCodeConfigManager| {
            match manager.delete_provider(&id) {
                Ok(()) => ok_message(format!("Provider '{}' 删除成功", id)).into_response(),
                Err(e) => bad_request(format!("删除 Provider 失败: {}", e)).into_response(),
            }
        }
    )
}

// ============ MCP 服务器接口（原生格式）============

/// GET /api/opencode/mcp - 列出所有 MCP 服务器
pub async fn list_opencode_mcp_servers() -> impl IntoResponse {
    crate::with_manager!(
        OpenCodeConfigManager,
        PLATFORM,
        |manager: OpenCodeConfigManager| {
            match manager.list_mcp_servers() {
                Ok(servers) => {
                    let list: Vec<_> = servers
                        .into_iter()
                        .map(|(id, s)| {
                            json!({
                                "id": id,
                                "type": s.server_type,
                                "command": s.command,
                                "environment": s.environment,
                                "url": s.url,
                                "headers": s.headers,
                            })
                        })
                        .collect();
                    ok(list).into_response()
                }
                Err(e) => internal_error(format!("读取 MCP 服务器列表失败: {}", e)).into_response(),
            }
        }
    )
}

/// POST /api/opencode/mcp - 添加 MCP 服务器
pub async fn add_opencode_mcp_server(
    Json(request): Json<OpenCodeMcpServerRequest>,
) -> impl IntoResponse {
    crate::with_manager!(
        OpenCodeConfigManager,
        PLATFORM,
        |manager: OpenCodeConfigManager| {
            let server = OpenCodeMcpServer {
                server_type: request.server_type.clone(),
                command: request.command,
                environment: request.environment,
                url: request.url,
                headers: request.headers,
            };
            let id = request.id.clone();
            match manager.set_mcp_server(id.clone(), server) {
                Ok(()) => ok_message(format!("MCP 服务器 '{}' 添加成功", id)).into_response(),
                Err(e) => bad_request(format!("添加 MCP 服务器失败: {}", e)).into_response(),
            }
        }
    )
}

/// PUT /api/opencode/mcp/:id - 更新 MCP 服务器
pub async fn update_opencode_mcp_server(
    Path(id): Path<String>,
    Json(request): Json<OpenCodeMcpServerRequest>,
) -> impl IntoResponse {
    crate::with_manager!(
        OpenCodeConfigManager,
        PLATFORM,
        |manager: OpenCodeConfigManager| {
            let server = OpenCodeMcpServer {
                server_type: request.server_type.clone(),
                command: request.command,
                environment: request.environment,
                url: request.url,
                headers: request.headers,
            };
            match manager.set_mcp_server(id.clone(), server) {
                Ok(()) => ok_message(format!("MCP 服务器 '{}' 更新成功", id)).into_response(),
                Err(e) => bad_request(format!("更新 MCP 服务器失败: {}", e)).into_response(),
            }
        }
    )
}

/// DELETE /api/opencode/mcp/:id - 删除 MCP 服务器
pub async fn delete_opencode_mcp_server(Path(id): Path<String>) -> impl IntoResponse {
    crate::with_manager!(
        OpenCodeConfigManager,
        PLATFORM,
        |manager: OpenCodeConfigManager| {
            match manager.delete_mcp_server(&id) {
                Ok(()) => ok_message(format!("MCP 服务器 '{}' 删除成功", id)).into_response(),
                Err(e) => bad_request(format!("删除 MCP 服务器失败: {}", e)).into_response(),
            }
        }
    )
}

// ============ Plugin 接口 ============

/// GET /api/opencode/plugins - 列出所有 Plugin
pub async fn list_opencode_plugins() -> impl IntoResponse {
    crate::with_manager!(
        OpenCodeConfigManager,
        PLATFORM,
        |manager: OpenCodeConfigManager| {
            match manager.list_plugins() {
                Ok(plugins) => {
                    let list: Vec<_> = plugins
                        .into_iter()
                        .map(|npm| json!({ "npm": npm }))
                        .collect();
                    ok(list).into_response()
                }
                Err(e) => internal_error(format!("读取 Plugin 列表失败: {}", e)).into_response(),
            }
        }
    )
}

/// POST /api/opencode/plugins - 添加 Plugin
pub async fn add_opencode_plugin(Json(request): Json<OpenCodePluginRequest>) -> impl IntoResponse {
    crate::with_manager!(
        OpenCodeConfigManager,
        PLATFORM,
        |manager: OpenCodeConfigManager| {
            let npm = request.npm.clone();
            match manager.add_plugin(npm.clone()) {
                Ok(()) => ok_message(format!("Plugin '{}' 添加成功", npm)).into_response(),
                Err(e) => bad_request(format!("添加 Plugin 失败: {}", e)).into_response(),
            }
        }
    )
}

/// DELETE /api/opencode/plugins/:pkg - 删除 Plugin（URL 编码包名）
pub async fn delete_opencode_plugin(Path(pkg): Path<String>) -> impl IntoResponse {
    crate::with_manager!(
        OpenCodeConfigManager,
        PLATFORM,
        |manager: OpenCodeConfigManager| {
            match manager.remove_plugin(&pkg) {
                Ok(()) => ok_message(format!("Plugin '{}' 删除成功", pkg)).into_response(),
                Err(e) => bad_request(format!("删除 Plugin 失败: {}", e)).into_response(),
            }
        }
    )
}

// ============ 配置接口 ============

/// GET /api/opencode/config - 获取完整 opencode.json
pub async fn get_opencode_config() -> impl IntoResponse {
    crate::with_manager!(
        OpenCodeConfigManager,
        PLATFORM,
        |manager: OpenCodeConfigManager| {
            match manager.get_config() {
                Ok(config) => ok(config).into_response(),
                Err(e) => internal_error(format!("读取 OpenCode 配置失败: {}", e)).into_response(),
            }
        }
    )
}

// ============ 工具函数 ============

fn build_provider_from_request(request: &OpenCodeProviderRequest) -> OpenCodeProvider {
    let options = request
        .options
        .as_ref()
        .map(|o| OpenCodeProviderOptions {
            base_url: o.base_url.clone(),
            api_key: o.api_key.clone(),
            headers: o.headers.clone(),
            extra: Default::default(),
        })
        .unwrap_or_default();

    let models = request
        .models
        .as_ref()
        .map(|m| {
            m.iter()
                .map(|(k, v)| {
                    let model = OpenCodeModel {
                        name: v.name.clone(),
                        limit: v.limit.as_ref().map(|l| OpenCodeModelLimit {
                            context: l.context,
                            output: l.output,
                        }),
                        extra: Default::default(),
                    };
                    (k.clone(), model)
                })
                .collect()
        })
        .unwrap_or_default();

    OpenCodeProvider {
        npm: request.npm.clone(),
        name: request.name.clone(),
        options,
        models,
    }
}
