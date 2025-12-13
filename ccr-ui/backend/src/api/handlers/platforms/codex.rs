// Codex CLI 配置管理 API 处理器

use axum::{Json, extract::Path, http::StatusCode, response::IntoResponse};
use serde_json::json;

use crate::managers::config::codex_manager::CodexConfigManager;
use crate::models::platforms::codex::*;

// CCR 平台 Profiles（~/.ccr/platforms/codex/profiles.toml）
use ccr::{LockManager, Platform, ProfileConfig, create_platform};
use serde::Deserialize;

fn mask_token(token: &str) -> String {
    if token.len() <= 10 {
        "*".repeat(token.len())
    } else {
        let prefix = &token[..4];
        let suffix = &token[token.len() - 4..];
        format!("{}...{}", prefix, suffix)
    }
}

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
        .name
        .clone()
        .or(req.command.clone())
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

#[derive(Debug, Deserialize)]
pub struct CodexProfileUpsertRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub base_url: String,
    pub auth_token: String,
    pub model: String,
    pub small_fast_model: Option<String>,
    pub provider: Option<String>,
    pub provider_type: Option<String>,
    pub account: Option<String>,
    pub tags: Option<Vec<String>>,
    pub enabled: Option<bool>,
    pub extra: Option<serde_json::Map<String, serde_json::Value>>,
}

fn merge_profile(existing: Option<ProfileConfig>, req: CodexProfileUpsertRequest) -> ProfileConfig {
    let mut profile = existing.unwrap_or_else(ProfileConfig::new);

    profile.description = req.description;
    profile.base_url = Some(req.base_url);
    profile.auth_token = Some(req.auth_token);
    profile.model = Some(req.model);
    profile.small_fast_model = req.small_fast_model;
    profile.provider = req.provider;
    profile.provider_type = req.provider_type;
    profile.account = req.account;
    profile.tags = req.tags;
    profile.enabled = req.enabled.or(profile.enabled);

    if let Some(extra) = req.extra {
        profile.platform_data = extra.into_iter().collect();
    }

    profile
}

/// GET /api/codex/profiles - 列出所有 Codex Profiles（CCR 平台 profiles.toml）
pub async fn list_codex_profiles() -> impl IntoResponse {
    let result = tokio::task::spawn_blocking(move || {
        let platform = create_platform(Platform::Codex)
            .map_err(|e| format!("创建 Codex 平台失败: {}", e))?;

        let profiles = platform
            .load_profiles()
            .map_err(|e| format!("读取 Codex profiles.toml 失败: {}", e))?;

        let current_profile = platform
            .get_current_profile()
            .map_err(|e| format!("读取当前 Codex profile 失败: {}", e))?;

        let profiles = profiles
            .into_iter()
            .map(|(name, profile)| {
                let auth_token = profile
                    .auth_token
                    .as_deref()
                    .map(mask_token)
                    .unwrap_or_default();

                json!({
                    "name": name,
                    "description": profile.description,
                    "base_url": profile.base_url.unwrap_or_default(),
                    "auth_token": auth_token,
                    "model": profile.model.unwrap_or_default(),
                    "small_fast_model": profile.small_fast_model,
                    "provider": profile.provider,
                    "provider_type": profile.provider_type,
                    "account": profile.account,
                    "tags": profile.tags,
                    "usage_count": profile.usage_count.unwrap_or(0),
                    "enabled": profile.enabled.unwrap_or(true),
                    "extra": profile.platform_data,
                })
            })
            .collect::<Vec<_>>();

        Ok::<_, String>((profiles, current_profile))
    })
    .await;

    match result {
        Ok(Ok((profiles, current_profile))) => (
            StatusCode::OK,
            Json(json!({
                "success": true,
                "data": { "profiles": profiles, "current_profile": current_profile },
                "message": null
            })),
        )
            .into_response(),
        Ok(Err(e)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("读取 Codex Profiles 失败: {}", e)
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("任务执行失败: {}", e)
            })),
        )
            .into_response(),
    }
}

/// GET /api/codex/profiles/:name - 获取单个 Codex Profile（完整 token）
pub async fn get_codex_profile(Path(name): Path<String>) -> impl IntoResponse {
    let result = tokio::task::spawn_blocking(move || {
        let platform = create_platform(Platform::Codex)
            .map_err(|e| format!("创建 Codex 平台失败: {}", e))?;

        let profiles = platform
            .load_profiles()
            .map_err(|e| format!("读取 Codex profiles.toml 失败: {}", e))?;

        let profile = profiles
            .get(&name)
            .cloned()
            .ok_or_else(|| format!("Profile '{}' 不存在", name))?;

        Ok::<_, String>(json!({
            "name": name,
            "description": profile.description,
            "base_url": profile.base_url.unwrap_or_default(),
            "auth_token": profile.auth_token.unwrap_or_default(),
            "model": profile.model.unwrap_or_default(),
            "small_fast_model": profile.small_fast_model,
            "provider": profile.provider,
            "provider_type": profile.provider_type,
            "account": profile.account,
            "tags": profile.tags,
            "usage_count": profile.usage_count.unwrap_or(0),
            "enabled": profile.enabled.unwrap_or(true),
            "extra": profile.platform_data,
        }))
    })
    .await;

    match result {
        Ok(Ok(profile)) => (
            StatusCode::OK,
            Json(json!({
                "success": true,
                "data": { "profile": profile },
                "message": null
            })),
        )
            .into_response(),
        Ok(Err(e)) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "success": false,
                "data": null,
                "message": e
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("任务执行失败: {}", e)
            })),
        )
            .into_response(),
    }
}

/// POST /api/codex/profiles - 添加 Codex Profile（写入 CCR 平台 profiles.toml）
pub async fn add_codex_profile(Json(req): Json<CodexProfileUpsertRequest>) -> impl IntoResponse {
    let result = tokio::task::spawn_blocking(move || {
        let name = req
            .name
            .clone()
            .ok_or_else(|| "缺少 profile name".to_string())?;

        let lock_manager = LockManager::with_default_path()
            .map_err(|e| format!("初始化锁管理器失败: {}", e))?;
        let _lock = lock_manager
            .lock_resource("ccr_codex_profiles", std::time::Duration::from_secs(10))
            .map_err(|e| format!("获取锁失败: {}", e))?;

        let platform = create_platform(Platform::Codex)
            .map_err(|e| format!("创建 Codex 平台失败: {}", e))?;

        let profile = merge_profile(None, req);

        platform
            .save_profile(&name, &profile)
            .map_err(|e| format!("保存 profile 失败: {}", e))?;

        Ok::<_, String>(name)
    })
    .await;

    match result {
        Ok(Ok(name)) => (
            StatusCode::OK,
            Json(json!({
                "success": true,
                "data": format!("Codex Profile '{}' 已成功添加", name),
                "message": null
            })),
        )
            .into_response(),
        Ok(Err(e)) => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "data": null,
                "message": e
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("任务执行失败: {}", e)
            })),
        )
            .into_response(),
    }
}

/// PUT /api/codex/profiles/:name - 更新 Codex Profile（写入 CCR 平台 profiles.toml）
pub async fn update_codex_profile(
    Path(name): Path<String>,
    Json(req): Json<CodexProfileUpsertRequest>,
) -> impl IntoResponse {
    let name_for_response = name.clone();
    let result = tokio::task::spawn_blocking(move || {
        let lock_manager = LockManager::with_default_path()
            .map_err(|e| format!("初始化锁管理器失败: {}", e))?;
        let _lock = lock_manager
            .lock_resource("ccr_codex_profiles", std::time::Duration::from_secs(10))
            .map_err(|e| format!("获取锁失败: {}", e))?;

        let platform = create_platform(Platform::Codex)
            .map_err(|e| format!("创建 Codex 平台失败: {}", e))?;

        let profiles = platform
            .load_profiles()
            .map_err(|e| format!("读取 profiles 失败: {}", e))?;
        let existing = profiles
            .get(&name)
            .cloned()
            .ok_or_else(|| format!("Profile '{}' 不存在", name))?;

        let mut profile = merge_profile(Some(existing.clone()), req);
        // 更新时保留使用次数（避免 UI 覆盖统计）
        profile.usage_count = existing.usage_count;

        platform
            .save_profile(&name, &profile)
            .map_err(|e| format!("保存 profile 失败: {}", e))?;

        Ok::<_, String>(())
    })
    .await;

    match result {
        Ok(Ok(())) => (
            StatusCode::OK,
            Json(json!({
                "success": true,
                "data": format!("Codex Profile '{}' 已成功更新", name_for_response),
                "message": null
            })),
        )
            .into_response(),
        Ok(Err(e)) => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("更新 Codex Profile 失败: {}", e)
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("任务执行失败: {}", e)
            })),
        )
            .into_response(),
    }
}

/// DELETE /api/codex/profiles/:name - 删除 Codex Profile（CCR 平台 profiles.toml）
pub async fn delete_codex_profile(Path(name): Path<String>) -> impl IntoResponse {
    let name_for_response = name.clone();
    let result = tokio::task::spawn_blocking(move || {
        let lock_manager = LockManager::with_default_path()
            .map_err(|e| format!("初始化锁管理器失败: {}", e))?;
        let _lock = lock_manager
            .lock_resource("ccr_codex_profiles", std::time::Duration::from_secs(10))
            .map_err(|e| format!("获取锁失败: {}", e))?;

        let platform = create_platform(Platform::Codex)
            .map_err(|e| format!("创建 Codex 平台失败: {}", e))?;

        platform
            .delete_profile(&name)
            .map_err(|e| format!("删除 profile 失败: {}", e))?;

        Ok::<_, String>(())
    })
    .await;

    match result {
        Ok(Ok(())) => (
            StatusCode::OK,
            Json(json!({
                "success": true,
                "data": format!("Codex Profile '{}' 已成功删除", name_for_response),
                "message": null
            })),
        )
            .into_response(),
        Ok(Err(e)) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("删除 Codex Profile 失败: {}", e)
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("任务执行失败: {}", e)
            })),
        )
            .into_response(),
    }
}

/// POST /api/codex/profiles/:name/apply - 应用 Codex Profile（写入 Codex settings/config，并更新注册表）
pub async fn apply_codex_profile(Path(name): Path<String>) -> impl IntoResponse {
    let name_for_response = name.clone();
    let result = tokio::task::spawn_blocking(move || {
        let lock_manager = LockManager::with_default_path()
            .map_err(|e| format!("初始化锁管理器失败: {}", e))?;
        let _lock = lock_manager
            .lock_resource("ccr_codex_apply", std::time::Duration::from_secs(20))
            .map_err(|e| format!("获取锁失败: {}", e))?;

        let platform = create_platform(Platform::Codex)
            .map_err(|e| format!("创建 Codex 平台失败: {}", e))?;

        platform
            .apply_profile(&name)
            .map_err(|e| format!("应用 profile 失败: {}", e))?;

        Ok::<_, String>(())
    })
    .await;

    match result {
        Ok(Ok(())) => (
            StatusCode::OK,
            Json(json!({
                "success": true,
                "data": format!("Codex Profile '{}' 已成功应用", name_for_response),
                "message": null
            })),
        )
            .into_response(),
        Ok(Err(e)) => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "data": null,
                "message": e
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("任务执行失败: {}", e)
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
