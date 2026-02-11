// Droid CLI API 处理器
//
// 使用统一的响应工具模块减少重复代码

use axum::{Json, extract::Path, response::IntoResponse};
use serde_json::json;
use std::collections::HashMap;

use crate::api::handlers::response::{bad_request, internal_error, ok, ok_message};
use crate::managers::config::droid_manager::DroidConfigManager;
use crate::models::platforms::droid::{
    DroidConfig, DroidCustomModel, DroidMcpServer, DroidMcpServerRequest,
};

// 导入核心 CLI 功能
use ccr::{Platform, ProfileConfig, create_platform};

const PLATFORM: &str = "Droid";

// ============ MCP 服务器管理 ============

/// GET /api/droid/mcp - 列出所有 MCP 服务器
pub async fn list_droid_mcp_servers() -> impl IntoResponse {
    crate::with_manager!(
        DroidConfigManager,
        PLATFORM,
        |manager: DroidConfigManager| {
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
        }
    )
}

/// POST /api/droid/mcp - 添加 MCP 服务器
pub async fn add_droid_mcp_server(Json(request): Json<DroidMcpServerRequest>) -> impl IntoResponse {
    crate::with_manager!(
        DroidConfigManager,
        PLATFORM,
        |manager: DroidConfigManager| {
            let server = DroidMcpServer {
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
                Ok(()) => {
                    ok_message(format!("MCP 服务器 '{}' 添加成功", request.name)).into_response()
                }
                Err(e) => bad_request(format!("添加 MCP 服务器失败: {}", e)).into_response(),
            }
        }
    )
}

/// PUT /api/droid/mcp/:name - 更新 MCP 服务器
pub async fn update_droid_mcp_server(
    Path(name): Path<String>,
    Json(request): Json<DroidMcpServerRequest>,
) -> impl IntoResponse {
    crate::with_manager!(
        DroidConfigManager,
        PLATFORM,
        |manager: DroidConfigManager| {
            let server = DroidMcpServer {
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
        }
    )
}

/// DELETE /api/droid/mcp/:name - 删除 MCP 服务器
pub async fn delete_droid_mcp_server(Path(name): Path<String>) -> impl IntoResponse {
    crate::with_manager!(
        DroidConfigManager,
        PLATFORM,
        |manager: DroidConfigManager| {
            match manager.delete_mcp_server(&name) {
                Ok(()) => ok_message(format!("MCP 服务器 '{}' 删除成功", name)).into_response(),
                Err(e) => bad_request(format!("删除 MCP 服务器失败: {}", e)).into_response(),
            }
        }
    )
}

// ============ Custom Models 管理 ============

/// GET /api/droid/models - 列出所有自定义模型
pub async fn list_droid_custom_models() -> impl IntoResponse {
    crate::with_manager!(
        DroidConfigManager,
        PLATFORM,
        |manager: DroidConfigManager| {
            match manager.list_custom_models() {
                Ok(models) => ok(models).into_response(),
                Err(e) => internal_error(format!("读取自定义模型失败: {}", e)).into_response(),
            }
        }
    )
}

/// POST /api/droid/models - 添加自定义模型
pub async fn add_droid_custom_model(Json(model): Json<DroidCustomModel>) -> impl IntoResponse {
    crate::with_manager!(
        DroidConfigManager,
        PLATFORM,
        |manager: DroidConfigManager| {
            match manager.add_custom_model(model.clone()) {
                Ok(()) => {
                    ok_message(format!("自定义模型 '{}' 添加成功", model.model)).into_response()
                }
                Err(e) => bad_request(format!("添加自定义模型失败: {}", e)).into_response(),
            }
        }
    )
}

/// PUT /api/droid/models/:model_id - 更新自定义模型
pub async fn update_droid_custom_model(
    Path(model_id): Path<String>,
    Json(model): Json<DroidCustomModel>,
) -> impl IntoResponse {
    crate::with_manager!(
        DroidConfigManager,
        PLATFORM,
        |manager: DroidConfigManager| {
            match manager.update_custom_model(&model_id, model) {
                Ok(()) => ok_message(format!("自定义模型 '{}' 更新成功", model_id)).into_response(),
                Err(e) => bad_request(format!("更新自定义模型失败: {}", e)).into_response(),
            }
        }
    )
}

/// DELETE /api/droid/models/:model_id - 删除自定义模型
pub async fn delete_droid_custom_model(Path(model_id): Path<String>) -> impl IntoResponse {
    crate::with_manager!(
        DroidConfigManager,
        PLATFORM,
        |manager: DroidConfigManager| {
            match manager.delete_custom_model(&model_id) {
                Ok(()) => ok_message(format!("自定义模型 '{}' 删除成功", model_id)).into_response(),
                Err(e) => bad_request(format!("删除自定义模型失败: {}", e)).into_response(),
            }
        }
    )
}

// ============ Profiles 管理 ============

/// GET /api/droid/profiles - 列出所有 Droid Profiles
pub async fn list_droid_profiles() -> impl IntoResponse {
    let result = tokio::task::spawn_blocking(move || {
        let platform =
            create_platform(Platform::Droid).map_err(|e| format!("创建 Droid 平台失败: {}", e))?;

        let profiles = platform
            .load_profiles()
            .map_err(|e| format!("读取 Droid profiles.toml 失败: {}", e))?;

        let current_profile = platform
            .get_current_profile()
            .map_err(|e| format!("读取当前 Droid profile 失败: {}", e))?;

        let profiles: Vec<_> = profiles
            .into_iter()
            .map(|(name, profile)| {
                // 从 platform_data 中提取 Droid 特定字段
                let max_output_tokens = profile
                    .platform_data
                    .get("max_output_tokens")
                    .and_then(|v| v.as_u64())
                    .map(|v| v as u32);
                let display_name = profile
                    .platform_data
                    .get("display_name")
                    .and_then(|v| v.as_str())
                    .map(String::from);

                json!({
                    "name": name,
                    "description": profile.description,
                    "base_url": profile.base_url.unwrap_or_default(),
                    "api_key": profile.auth_token.as_deref().unwrap_or(""),
                    "model": profile.model.unwrap_or_default(),
                    "provider": profile.provider,
                    "provider_type": profile.provider_type,
                    "max_output_tokens": max_output_tokens,
                    "display_name": display_name,
                    "tags": profile.tags,
                    "usage_count": profile.usage_count.unwrap_or(0),
                    "enabled": profile.enabled.unwrap_or(true),
                })
            })
            .collect();

        Ok::<_, String>((profiles, current_profile))
    })
    .await;

    match result {
        Ok(Ok((profiles, current_profile))) => ok(json!({
            "profiles": profiles,
            "current_profile": current_profile,
        }))
        .into_response(),
        Ok(Err(e)) => bad_request(e).into_response(),
        Err(e) => internal_error(format!("任务执行失败: {}", e)).into_response(),
    }
}

/// POST /api/droid/profiles - 添加 Droid Profile
pub async fn add_droid_profile(Json(request): Json<serde_json::Value>) -> impl IntoResponse {
    let result = tokio::task::spawn_blocking(move || {
        let name = request
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "缺少 name 字段".to_string())?
            .to_string();

        // 构建 platform_data
        let mut platform_data = serde_json::Map::new();
        if let Some(max_tokens) = request.get("max_output_tokens").and_then(|v| v.as_u64()) {
            platform_data.insert(
                "max_output_tokens".to_string(),
                serde_json::json!(max_tokens),
            );
        }
        if let Some(display_name) = request.get("display_name").and_then(|v| v.as_str()) {
            platform_data.insert("display_name".to_string(), serde_json::json!(display_name));
        }

        // 转换为 IndexMap
        let platform_data: indexmap::IndexMap<String, serde_json::Value> =
            platform_data.into_iter().collect();

        let profile = ProfileConfig {
            description: request
                .get("description")
                .and_then(|v| v.as_str())
                .map(String::from),
            base_url: request
                .get("base_url")
                .and_then(|v| v.as_str())
                .map(String::from),
            auth_token: request
                .get("api_key")
                .and_then(|v| v.as_str())
                .map(String::from),
            model: request
                .get("model")
                .and_then(|v| v.as_str())
                .map(String::from),
            provider: request
                .get("provider")
                .and_then(|v| v.as_str())
                .map(String::from),
            provider_type: request
                .get("provider_type")
                .and_then(|v| v.as_str())
                .map(String::from),
            tags: request.get("tags").and_then(|v| v.as_array()).map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            }),
            enabled: request.get("enabled").and_then(|v| v.as_bool()),
            usage_count: Some(0),
            platform_data,
            ..Default::default()
        };

        let platform =
            create_platform(Platform::Droid).map_err(|e| format!("创建 Droid 平台失败: {}", e))?;

        platform
            .save_profile(&name, &profile)
            .map_err(|e| format!("保存 Droid profile 失败: {}", e))?;

        Ok::<_, String>(name)
    })
    .await;

    match result {
        Ok(Ok(name)) => ok_message(format!("Profile '{}' 添加成功", name)).into_response(),
        Ok(Err(e)) => bad_request(e).into_response(),
        Err(e) => internal_error(format!("任务执行失败: {}", e)).into_response(),
    }
}

/// PUT /api/droid/profiles/:name - 更新 Droid Profile
pub async fn update_droid_profile(
    Path(name): Path<String>,
    Json(request): Json<serde_json::Value>,
) -> impl IntoResponse {
    let result = tokio::task::spawn_blocking(move || {
        let profile_name = name.clone();

        // 构建 platform_data
        let mut platform_data = serde_json::Map::new();
        if let Some(max_tokens) = request.get("max_output_tokens").and_then(|v| v.as_u64()) {
            platform_data.insert(
                "max_output_tokens".to_string(),
                serde_json::json!(max_tokens),
            );
        }
        if let Some(display_name) = request.get("display_name").and_then(|v| v.as_str()) {
            platform_data.insert("display_name".to_string(), serde_json::json!(display_name));
        }

        // 转换为 IndexMap
        let platform_data: indexmap::IndexMap<String, serde_json::Value> =
            platform_data.into_iter().collect();

        let profile = ProfileConfig {
            description: request
                .get("description")
                .and_then(|v| v.as_str())
                .map(String::from),
            base_url: request
                .get("base_url")
                .and_then(|v| v.as_str())
                .map(String::from),
            auth_token: request
                .get("api_key")
                .and_then(|v| v.as_str())
                .map(String::from),
            model: request
                .get("model")
                .and_then(|v| v.as_str())
                .map(String::from),
            provider: request
                .get("provider")
                .and_then(|v| v.as_str())
                .map(String::from),
            provider_type: request
                .get("provider_type")
                .and_then(|v| v.as_str())
                .map(String::from),
            tags: request.get("tags").and_then(|v| v.as_array()).map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            }),
            enabled: request.get("enabled").and_then(|v| v.as_bool()),
            usage_count: request
                .get("usage_count")
                .and_then(|v| v.as_u64())
                .map(|v| v as u32),
            platform_data,
            ..Default::default()
        };

        let platform =
            create_platform(Platform::Droid).map_err(|e| format!("创建 Droid 平台失败: {}", e))?;

        platform
            .save_profile(&profile_name, &profile)
            .map_err(|e| format!("更新 Droid profile 失败: {}", e))?;

        Ok::<_, String>(name)
    })
    .await;

    match result {
        Ok(Ok(name)) => ok_message(format!("Profile '{}' 更新成功", name)).into_response(),
        Ok(Err(e)) => bad_request(e).into_response(),
        Err(e) => internal_error(format!("任务执行失败: {}", e)).into_response(),
    }
}

/// DELETE /api/droid/profiles/:name - 删除 Droid Profile
pub async fn delete_droid_profile(Path(name): Path<String>) -> impl IntoResponse {
    let result = tokio::task::spawn_blocking(move || {
        let profile_name = name.clone();
        let platform =
            create_platform(Platform::Droid).map_err(|e| format!("创建 Droid 平台失败: {}", e))?;

        platform
            .delete_profile(&profile_name)
            .map_err(|e| format!("删除 Droid profile 失败: {}", e))?;

        Ok::<_, String>(name)
    })
    .await;

    match result {
        Ok(Ok(name)) => ok_message(format!("Profile '{}' 删除成功", name)).into_response(),
        Ok(Err(e)) => bad_request(e).into_response(),
        Err(e) => internal_error(format!("任务执行失败: {}", e)).into_response(),
    }
}

/// POST /api/droid/profiles/:name/switch - 切换到指定 Droid Profile
pub async fn switch_droid_profile(Path(name): Path<String>) -> impl IntoResponse {
    let result = tokio::task::spawn_blocking(move || {
        let profile_name = name.clone();
        let platform =
            create_platform(Platform::Droid).map_err(|e| format!("创建 Droid 平台失败: {}", e))?;

        platform
            .apply_profile(&profile_name)
            .map_err(|e| format!("切换 Droid profile 失败: {}", e))?;

        Ok::<_, String>(name)
    })
    .await;

    match result {
        Ok(Ok(name)) => ok_message(format!("已切换到 Profile '{}'", name)).into_response(),
        Ok(Err(e)) => bad_request(e).into_response(),
        Err(e) => internal_error(format!("任务执行失败: {}", e)).into_response(),
    }
}

// ============ 基础配置管理 ============

/// GET /api/droid/config - 获取完整配置
pub async fn get_droid_config() -> impl IntoResponse {
    crate::with_manager!(
        DroidConfigManager,
        PLATFORM,
        |manager: DroidConfigManager| {
            match manager.get_config() {
                Ok(config) => ok(config).into_response(),
                Err(e) => internal_error(format!("读取配置失败: {}", e)).into_response(),
            }
        }
    )
}

/// PUT /api/droid/config - 更新完整配置
pub async fn update_droid_config(Json(config): Json<DroidConfig>) -> impl IntoResponse {
    crate::with_manager!(
        DroidConfigManager,
        PLATFORM,
        |manager: DroidConfigManager| {
            match manager.update_config(&config) {
                Ok(()) => ok_message("配置更新成功").into_response(),
                Err(e) => bad_request(format!("更新配置失败: {}", e)).into_response(),
            }
        }
    )
}

// ============ Droids (Subagents) 管理 ============

/// GET /api/droid/droids - 列出所有 Droids
pub async fn list_droids() -> impl IntoResponse {
    use crate::managers::droids_manager::DroidsManager;

    match DroidsManager::default() {
        Ok(manager) => match manager.list_droids() {
            Ok(droids) => ok(droids).into_response(),
            Err(e) => internal_error(format!("列出 Droids 失败: {}", e)).into_response(),
        },
        Err(e) => internal_error(format!("初始化 Droids 管理器失败: {}", e)).into_response(),
    }
}

/// GET /api/droid/droids/:name - 获取单个 Droid
pub async fn get_droid(Path(name): Path<String>) -> impl IntoResponse {
    use crate::managers::droids_manager::DroidsManager;

    match DroidsManager::default() {
        Ok(manager) => match manager.get_droid(&name) {
            Ok(droid) => ok(droid).into_response(),
            Err(e) => bad_request(format!("获取 Droid 失败: {}", e)).into_response(),
        },
        Err(e) => internal_error(format!("初始化 Droids 管理器失败: {}", e)).into_response(),
    }
}

/// POST /api/droid/droids - 创建新 Droid
pub async fn create_droid(Json(request): Json<serde_json::Value>) -> impl IntoResponse {
    use crate::managers::droids_manager::DroidsManager;
    use crate::models::platforms::droid::DroidRequest;

    // 解析请求
    let droid_request: DroidRequest = match serde_json::from_value(request) {
        Ok(req) => req,
        Err(e) => return bad_request(format!("无效的请求格式: {}", e)).into_response(),
    };

    match DroidsManager::default() {
        Ok(manager) => match manager.create_droid(droid_request) {
            Ok(()) => ok_message("Droid 创建成功").into_response(),
            Err(e) => bad_request(format!("创建 Droid 失败: {}", e)).into_response(),
        },
        Err(e) => internal_error(format!("初始化 Droids 管理器失败: {}", e)).into_response(),
    }
}

/// PUT /api/droid/droids/:name - 更新 Droid
pub async fn update_droid(
    Path(name): Path<String>,
    Json(request): Json<serde_json::Value>,
) -> impl IntoResponse {
    use crate::managers::droids_manager::DroidsManager;
    use crate::models::platforms::droid::DroidRequest;

    // 解析请求
    let droid_request: DroidRequest = match serde_json::from_value(request) {
        Ok(req) => req,
        Err(e) => return bad_request(format!("无效的请求格式: {}", e)).into_response(),
    };

    match DroidsManager::default() {
        Ok(manager) => match manager.update_droid(&name, droid_request) {
            Ok(()) => ok_message(format!("Droid '{}' 更新成功", name)).into_response(),
            Err(e) => bad_request(format!("更新 Droid 失败: {}", e)).into_response(),
        },
        Err(e) => internal_error(format!("初始化 Droids 管理器失败: {}", e)).into_response(),
    }
}

/// DELETE /api/droid/droids/:name - 删除 Droid
pub async fn delete_droid(Path(name): Path<String>) -> impl IntoResponse {
    use crate::managers::droids_manager::DroidsManager;

    match DroidsManager::default() {
        Ok(manager) => match manager.delete_droid(&name) {
            Ok(()) => ok_message(format!("Droid '{}' 删除成功", name)).into_response(),
            Err(e) => bad_request(format!("删除 Droid 失败: {}", e)).into_response(),
        },
        Err(e) => internal_error(format!("初始化 Droids 管理器失败: {}", e)).into_response(),
    }
}

// ============ Agents 管理 ============

use crate::managers::droid_markdown_manager::{
    DroidAgentFrontmatter, DroidCommandFrontmatter, DroidMarkdownFile, DroidMarkdownManager,
};
use crate::models::api::{Agent, AgentRequest, SlashCommand, SlashCommandRequest};

/// GET /api/droid/agents - 列出所有 Droid Agents
pub async fn list_droid_agents() -> impl IntoResponse {
    match DroidMarkdownManager::from_factory_subdir("agents") {
        Ok(manager) => match manager.list_files_with_folders() {
            Ok(files) => {
                let mut agents = Vec::new();

                for (file_name, folder_path) in files {
                    let full_name = if folder_path.is_empty() {
                        file_name.clone()
                    } else {
                        format!("{}/{}", folder_path, file_name)
                    };

                    match manager.read_file::<DroidAgentFrontmatter>(&full_name) {
                        Ok(file) => {
                            let tools = file
                                .frontmatter
                                .tools
                                .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
                                .unwrap_or_default();

                            agents.push(Agent {
                                name: file_name.clone(),
                                model: file.frontmatter.model.unwrap_or_default(),
                                tools,
                                system_prompt: Some(file.content),
                                disabled: false,
                                folder: folder_path,
                            });
                        }
                        Err(e) => {
                            tracing::warn!("Failed to read Droid agent {}: {}", full_name, e);
                        }
                    }
                }

                // Collect unique folders
                let folders: Vec<String> = agents
                    .iter()
                    .filter_map(|a| {
                        if !a.folder.is_empty() {
                            Some(a.folder.clone())
                        } else {
                            None
                        }
                    })
                    .collect::<std::collections::HashSet<_>>()
                    .into_iter()
                    .collect();

                ok(json!({
                    "agents": agents,
                    "folders": folders
                }))
                .into_response()
            }
            Err(e) => internal_error(format!("列出 Droid Agents 失败: {}", e)).into_response(),
        },
        Err(e) => internal_error(format!("初始化 Droid Agents 管理器失败: {}", e)).into_response(),
    }
}

/// GET /api/droid/agents/:name - 获取单个 Droid Agent
pub async fn get_droid_agent(Path(name): Path<String>) -> impl IntoResponse {
    match DroidMarkdownManager::from_factory_subdir("agents") {
        Ok(manager) => match manager.read_file::<DroidAgentFrontmatter>(&name) {
            Ok(file) => {
                let tools = file
                    .frontmatter
                    .tools
                    .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
                    .unwrap_or_default();

                let agent = Agent {
                    name: file.frontmatter.name,
                    model: file.frontmatter.model.unwrap_or_default(),
                    tools,
                    system_prompt: Some(file.content),
                    disabled: false,
                    folder: String::new(),
                };

                ok(agent).into_response()
            }
            Err(e) => bad_request(format!("获取 Droid Agent 失败: {}", e)).into_response(),
        },
        Err(e) => internal_error(format!("初始化 Droid Agents 管理器失败: {}", e)).into_response(),
    }
}

/// POST /api/droid/agents - 创建 Droid Agent
pub async fn add_droid_agent(Json(req): Json<AgentRequest>) -> impl IntoResponse {
    match DroidMarkdownManager::from_factory_subdir("agents") {
        Ok(manager) => {
            if manager.file_exists(&req.name) {
                return bad_request(format!("Agent '{}' 已存在", req.name)).into_response();
            }

            let file = DroidMarkdownFile {
                frontmatter: DroidAgentFrontmatter {
                    name: req.name.clone(),
                    description: None,
                    tools: req.tools.map(|t| t.join(", ")),
                    model: Some(req.model),
                },
                content: req.system_prompt.unwrap_or_default(),
            };

            match manager.write_file(&req.name, &file) {
                Ok(()) => ok_message(format!("Agent '{}' 创建成功", req.name)).into_response(),
                Err(e) => bad_request(format!("创建 Agent 失败: {}", e)).into_response(),
            }
        }
        Err(e) => internal_error(format!("初始化 Droid Agents 管理器失败: {}", e)).into_response(),
    }
}

/// PUT /api/droid/agents/:name - 更新 Droid Agent
pub async fn update_droid_agent(
    Path(name): Path<String>,
    Json(req): Json<AgentRequest>,
) -> impl IntoResponse {
    match DroidMarkdownManager::from_factory_subdir("agents") {
        Ok(manager) => {
            if !manager.file_exists(&name) {
                return bad_request(format!("Agent '{}' 不存在", name)).into_response();
            }

            let file = DroidMarkdownFile {
                frontmatter: DroidAgentFrontmatter {
                    name: req.name.clone(),
                    description: None,
                    tools: req.tools.map(|t| t.join(", ")),
                    model: Some(req.model),
                },
                content: req.system_prompt.unwrap_or_default(),
            };

            // If name changed, delete old file first
            if name != req.name {
                let _ = manager.delete_file(&name);
            }

            match manager.write_file(&req.name, &file) {
                Ok(()) => ok_message(format!("Agent '{}' 更新成功", req.name)).into_response(),
                Err(e) => bad_request(format!("更新 Agent 失败: {}", e)).into_response(),
            }
        }
        Err(e) => internal_error(format!("初始化 Droid Agents 管理器失败: {}", e)).into_response(),
    }
}

/// DELETE /api/droid/agents/:name - 删除 Droid Agent
pub async fn delete_droid_agent(Path(name): Path<String>) -> impl IntoResponse {
    match DroidMarkdownManager::from_factory_subdir("agents") {
        Ok(manager) => match manager.delete_file(&name) {
            Ok(()) => ok_message(format!("Agent '{}' 删除成功", name)).into_response(),
            Err(e) => bad_request(format!("删除 Agent 失败: {}", e)).into_response(),
        },
        Err(e) => internal_error(format!("初始化 Droid Agents 管理器失败: {}", e)).into_response(),
    }
}

// ============ Slash Commands 管理 ============

/// Extract description from markdown content
fn extract_description_from_content(content: &str, name: &str) -> String {
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if !line.is_empty() {
            return line.to_string();
        }
    }
    format!("Slash command: {}", name)
}

/// GET /api/droid/slash-commands - 列出所有 Droid Slash Commands
pub async fn list_droid_slash_commands() -> impl IntoResponse {
    match DroidMarkdownManager::from_factory_subdir("commands") {
        Ok(manager) => match manager.list_files_with_folders() {
            Ok(files) => {
                let mut commands = Vec::new();

                for (file_name, folder_path) in files {
                    let full_name = if folder_path.is_empty() {
                        file_name.clone()
                    } else {
                        format!("{}/{}", folder_path, file_name)
                    };

                    let description = match manager.read_file::<DroidCommandFrontmatter>(&full_name)
                    {
                        Ok(file) => file.frontmatter.description.clone().unwrap_or_else(|| {
                            extract_description_from_content(&file.content, &file_name)
                        }),
                        Err(_) => {
                            // Try to read as plain text
                            format!("Slash command: {}", file_name)
                        }
                    };

                    commands.push(SlashCommand {
                        name: file_name.clone(),
                        description,
                        command: String::new(),
                        args: None,
                        disabled: false,
                        folder: folder_path,
                    });
                }

                // Collect unique folders
                let folders: Vec<String> = commands
                    .iter()
                    .filter_map(|c| {
                        if !c.folder.is_empty() {
                            Some(c.folder.clone())
                        } else {
                            None
                        }
                    })
                    .collect::<std::collections::HashSet<_>>()
                    .into_iter()
                    .collect();

                ok(json!({
                    "commands": commands,
                    "folders": folders
                }))
                .into_response()
            }
            Err(e) => {
                internal_error(format!("列出 Droid Slash Commands 失败: {}", e)).into_response()
            }
        },
        Err(e) => {
            internal_error(format!("初始化 Droid Commands 管理器失败: {}", e)).into_response()
        }
    }
}

/// POST /api/droid/slash-commands - 创建 Droid Slash Command
pub async fn add_droid_slash_command(Json(req): Json<SlashCommandRequest>) -> impl IntoResponse {
    match DroidMarkdownManager::from_factory_subdir("commands") {
        Ok(manager) => {
            if manager.file_exists(&req.name) {
                return bad_request(format!("Slash Command '{}' 已存在", req.name)).into_response();
            }

            let file = DroidMarkdownFile {
                frontmatter: DroidCommandFrontmatter {
                    name: Some(req.name.clone()),
                    description: Some(req.description.clone()),
                    argument_hint: None,
                },
                content: req.command.clone(),
            };

            match manager.write_file(&req.name, &file) {
                Ok(()) => {
                    ok_message(format!("Slash Command '{}' 创建成功", req.name)).into_response()
                }
                Err(e) => bad_request(format!("创建 Slash Command 失败: {}", e)).into_response(),
            }
        }
        Err(e) => {
            internal_error(format!("初始化 Droid Commands 管理器失败: {}", e)).into_response()
        }
    }
}

/// PUT /api/droid/slash-commands/:name - 更新 Droid Slash Command
pub async fn update_droid_slash_command(
    Path(name): Path<String>,
    Json(req): Json<SlashCommandRequest>,
) -> impl IntoResponse {
    match DroidMarkdownManager::from_factory_subdir("commands") {
        Ok(manager) => {
            if !manager.file_exists(&name) {
                return bad_request(format!("Slash Command '{}' 不存在", name)).into_response();
            }

            let file = DroidMarkdownFile {
                frontmatter: DroidCommandFrontmatter {
                    name: Some(req.name.clone()),
                    description: Some(req.description.clone()),
                    argument_hint: None,
                },
                content: req.command.clone(),
            };

            // If name changed, delete old file first
            if name != req.name {
                let _ = manager.delete_file(&name);
            }

            match manager.write_file(&req.name, &file) {
                Ok(()) => {
                    ok_message(format!("Slash Command '{}' 更新成功", req.name)).into_response()
                }
                Err(e) => bad_request(format!("更新 Slash Command 失败: {}", e)).into_response(),
            }
        }
        Err(e) => {
            internal_error(format!("初始化 Droid Commands 管理器失败: {}", e)).into_response()
        }
    }
}

/// DELETE /api/droid/slash-commands/:name - 删除 Droid Slash Command
pub async fn delete_droid_slash_command(Path(name): Path<String>) -> impl IntoResponse {
    match DroidMarkdownManager::from_factory_subdir("commands") {
        Ok(manager) => match manager.delete_file(&name) {
            Ok(()) => ok_message(format!("Slash Command '{}' 删除成功", name)).into_response(),
            Err(e) => bad_request(format!("删除 Slash Command 失败: {}", e)).into_response(),
        },
        Err(e) => {
            internal_error(format!("初始化 Droid Commands 管理器失败: {}", e)).into_response()
        }
    }
}

// ============ Plugins 管理 ============

use crate::managers::droid_plugins_manager::DroidPluginsManager;

/// GET /api/droid/plugins - 列出所有 Droid Plugins
pub async fn list_droid_plugins() -> impl IntoResponse {
    match DroidPluginsManager::default() {
        Ok(manager) => match manager.get_plugins() {
            Ok(plugins) => {
                let plugins_vec: Vec<_> = plugins
                    .into_iter()
                    .map(|(id, data)| {
                        json!({
                            "id": id,
                            "data": data
                        })
                    })
                    .collect();
                ok(plugins_vec).into_response()
            }
            Err(e) => internal_error(format!("列出 Droid Plugins 失败: {}", e)).into_response(),
        },
        Err(e) => internal_error(format!("初始化 Droid Plugins 管理器失败: {}", e)).into_response(),
    }
}

/// POST /api/droid/plugins - 添加 Droid Plugin
pub async fn add_droid_plugin(Json(request): Json<serde_json::Value>) -> impl IntoResponse {
    let id = match request.get("id").and_then(|v| v.as_str()) {
        Some(id) => id.to_string(),
        None => return bad_request("缺少 id 字段").into_response(),
    };

    let data = request.get("data").cloned().unwrap_or(json!({}));

    match DroidPluginsManager::default() {
        Ok(manager) => match manager.add_plugin(id.clone(), data) {
            Ok(()) => ok_message(format!("Plugin '{}' 添加成功", id)).into_response(),
            Err(e) => bad_request(format!("添加 Plugin 失败: {}", e)).into_response(),
        },
        Err(e) => internal_error(format!("初始化 Droid Plugins 管理器失败: {}", e)).into_response(),
    }
}

/// PUT /api/droid/plugins/:id - 更新 Droid Plugin
pub async fn update_droid_plugin(
    Path(id): Path<String>,
    Json(request): Json<serde_json::Value>,
) -> impl IntoResponse {
    let data = request.get("data").cloned().unwrap_or(json!({}));

    match DroidPluginsManager::default() {
        Ok(manager) => match manager.update_plugin(&id, data) {
            Ok(()) => ok_message(format!("Plugin '{}' 更新成功", id)).into_response(),
            Err(e) => bad_request(format!("更新 Plugin 失败: {}", e)).into_response(),
        },
        Err(e) => internal_error(format!("初始化 Droid Plugins 管理器失败: {}", e)).into_response(),
    }
}

/// DELETE /api/droid/plugins/:id - 删除 Droid Plugin
pub async fn delete_droid_plugin(Path(id): Path<String>) -> impl IntoResponse {
    match DroidPluginsManager::default() {
        Ok(manager) => match manager.delete_plugin(&id) {
            Ok(()) => ok_message(format!("Plugin '{}' 删除成功", id)).into_response(),
            Err(e) => bad_request(format!("删除 Plugin 失败: {}", e)).into_response(),
        },
        Err(e) => internal_error(format!("初始化 Droid Plugins 管理器失败: {}", e)).into_response(),
    }
}
