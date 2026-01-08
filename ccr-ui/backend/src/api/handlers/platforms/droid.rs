// Droid CLI API 处理器
//
// 使用统一的响应工具模块减少重复代码

use axum::{Json, extract::Path, response::IntoResponse};
use serde_json::json;
use std::collections::HashMap;

use crate::api::handlers::response::{bad_request, internal_error, ok, ok_message};
use crate::managers::config::droid_manager::DroidConfigManager;
use crate::models::platforms::droid::{DroidConfig, DroidCustomModel, DroidMcpServer, DroidMcpServerRequest};

// 导入核心 CLI 功能
use ccr::{create_platform, Platform, ProfileConfig};

const PLATFORM: &str = "Droid";

// ============ 辅助宏 ============

/// 初始化 Manager 并处理错误
macro_rules! with_droid_manager {
    ($body:expr) => {
        match DroidConfigManager::default() {
            Ok(manager) => $body(manager),
            Err(e) => internal_error(format!("初始化 {} 配置管理器失败: {}", PLATFORM, e))
                .into_response(),
        }
    };
}

// ============ MCP 服务器管理 ============

/// GET /api/droid/mcp - 列出所有 MCP 服务器
pub async fn list_droid_mcp_servers() -> impl IntoResponse {
    with_droid_manager!(|manager: DroidConfigManager| {
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

/// POST /api/droid/mcp - 添加 MCP 服务器
pub async fn add_droid_mcp_server(Json(request): Json<DroidMcpServerRequest>) -> impl IntoResponse {
    with_droid_manager!(|manager: DroidConfigManager| {
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
            Ok(()) => ok_message(format!("MCP 服务器 '{}' 添加成功", request.name)).into_response(),
            Err(e) => bad_request(format!("添加 MCP 服务器失败: {}", e)).into_response(),
        }
    })
}

/// PUT /api/droid/mcp/:name - 更新 MCP 服务器
pub async fn update_droid_mcp_server(
    Path(name): Path<String>,
    Json(request): Json<DroidMcpServerRequest>,
) -> impl IntoResponse {
    with_droid_manager!(|manager: DroidConfigManager| {
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
    })
}

/// DELETE /api/droid/mcp/:name - 删除 MCP 服务器
pub async fn delete_droid_mcp_server(Path(name): Path<String>) -> impl IntoResponse {
    with_droid_manager!(|manager: DroidConfigManager| {
        match manager.delete_mcp_server(&name) {
            Ok(()) => ok_message(format!("MCP 服务器 '{}' 删除成功", name)).into_response(),
            Err(e) => bad_request(format!("删除 MCP 服务器失败: {}", e)).into_response(),
        }
    })
}

// ============ Custom Models 管理 ============

/// GET /api/droid/models - 列出所有自定义模型
pub async fn list_droid_custom_models() -> impl IntoResponse {
    with_droid_manager!(|manager: DroidConfigManager| {
        match manager.list_custom_models() {
            Ok(models) => ok(models).into_response(),
            Err(e) => internal_error(format!("读取自定义模型失败: {}", e)).into_response(),
        }
    })
}

/// POST /api/droid/models - 添加自定义模型
pub async fn add_droid_custom_model(Json(model): Json<DroidCustomModel>) -> impl IntoResponse {
    with_droid_manager!(|manager: DroidConfigManager| {
        match manager.add_custom_model(model.clone()) {
            Ok(()) => ok_message(format!("自定义模型 '{}' 添加成功", model.model)).into_response(),
            Err(e) => bad_request(format!("添加自定义模型失败: {}", e)).into_response(),
        }
    })
}

/// PUT /api/droid/models/:model_id - 更新自定义模型
pub async fn update_droid_custom_model(
    Path(model_id): Path<String>,
    Json(model): Json<DroidCustomModel>,
) -> impl IntoResponse {
    with_droid_manager!(|manager: DroidConfigManager| {
        match manager.update_custom_model(&model_id, model) {
            Ok(()) => ok_message(format!("自定义模型 '{}' 更新成功", model_id)).into_response(),
            Err(e) => bad_request(format!("更新自定义模型失败: {}", e)).into_response(),
        }
    })
}

/// DELETE /api/droid/models/:model_id - 删除自定义模型
pub async fn delete_droid_custom_model(Path(model_id): Path<String>) -> impl IntoResponse {
    with_droid_manager!(|manager: DroidConfigManager| {
        match manager.delete_custom_model(&model_id) {
            Ok(()) => ok_message(format!("自定义模型 '{}' 删除成功", model_id)).into_response(),
            Err(e) => bad_request(format!("删除自定义模型失败: {}", e)).into_response(),
        }
    })
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
                let max_output_tokens = profile.platform_data.get("max_output_tokens")
                    .and_then(|v| v.as_u64())
                    .map(|v| v as u32);
                let display_name = profile.platform_data.get("display_name")
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
pub async fn add_droid_profile(
    Json(request): Json<serde_json::Value>,
) -> impl IntoResponse {
    let result = tokio::task::spawn_blocking(move || {
        let name = request
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "缺少 name 字段".to_string())?
            .to_string();

        // 构建 platform_data
        let mut platform_data = serde_json::Map::new();
        if let Some(max_tokens) = request.get("max_output_tokens").and_then(|v| v.as_u64()) {
            platform_data.insert("max_output_tokens".to_string(), serde_json::json!(max_tokens));
        }
        if let Some(display_name) = request.get("display_name").and_then(|v| v.as_str()) {
            platform_data.insert("display_name".to_string(), serde_json::json!(display_name));
        }

        // 转换为 IndexMap
        let platform_data: indexmap::IndexMap<String, serde_json::Value> =
            platform_data.into_iter().collect();

        let profile = ProfileConfig {
            description: request.get("description").and_then(|v| v.as_str()).map(String::from),
            base_url: request.get("base_url").and_then(|v| v.as_str()).map(String::from),
            auth_token: request.get("api_key").and_then(|v| v.as_str()).map(String::from),
            model: request.get("model").and_then(|v| v.as_str()).map(String::from),
            provider: request.get("provider").and_then(|v| v.as_str()).map(String::from),
            provider_type: request.get("provider_type").and_then(|v| v.as_str()).map(String::from),
            tags: request.get("tags").and_then(|v| v.as_array()).map(|arr| {
                arr.iter().filter_map(|v| v.as_str().map(String::from)).collect()
            }),
            enabled: request.get("enabled").and_then(|v| v.as_bool()),
            usage_count: Some(0),
            platform_data,
            ..Default::default()
        };

        let platform = create_platform(Platform::Droid)
            .map_err(|e| format!("创建 Droid 平台失败: {}", e))?;

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
            platform_data.insert("max_output_tokens".to_string(), serde_json::json!(max_tokens));
        }
        if let Some(display_name) = request.get("display_name").and_then(|v| v.as_str()) {
            platform_data.insert("display_name".to_string(), serde_json::json!(display_name));
        }

        // 转换为 IndexMap
        let platform_data: indexmap::IndexMap<String, serde_json::Value> =
            platform_data.into_iter().collect();

        let profile = ProfileConfig {
            description: request.get("description").and_then(|v| v.as_str()).map(String::from),
            base_url: request.get("base_url").and_then(|v| v.as_str()).map(String::from),
            auth_token: request.get("api_key").and_then(|v| v.as_str()).map(String::from),
            model: request.get("model").and_then(|v| v.as_str()).map(String::from),
            provider: request.get("provider").and_then(|v| v.as_str()).map(String::from),
            provider_type: request.get("provider_type").and_then(|v| v.as_str()).map(String::from),
            tags: request.get("tags").and_then(|v| v.as_array()).map(|arr| {
                arr.iter().filter_map(|v| v.as_str().map(String::from)).collect()
            }),
            enabled: request.get("enabled").and_then(|v| v.as_bool()),
            usage_count: request.get("usage_count").and_then(|v| v.as_u64()).map(|v| v as u32),
            platform_data,
            ..Default::default()
        };

        let platform = create_platform(Platform::Droid)
            .map_err(|e| format!("创建 Droid 平台失败: {}", e))?;

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
        let platform = create_platform(Platform::Droid)
            .map_err(|e| format!("创建 Droid 平台失败: {}", e))?;

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
        let platform = create_platform(Platform::Droid)
            .map_err(|e| format!("创建 Droid 平台失败: {}", e))?;

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
    with_droid_manager!(|manager: DroidConfigManager| {
        match manager.get_config() {
            Ok(config) => ok(config).into_response(),
            Err(e) => internal_error(format!("读取配置失败: {}", e)).into_response(),
        }
    })
}

/// PUT /api/droid/config - 更新完整配置
pub async fn update_droid_config(Json(config): Json<DroidConfig>) -> impl IntoResponse {
    with_droid_manager!(|manager: DroidConfigManager| {
        match manager.update_config(&config) {
            Ok(()) => ok_message("配置更新成功").into_response(),
            Err(e) => bad_request(format!("更新配置失败: {}", e)).into_response(),
        }
    })
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
