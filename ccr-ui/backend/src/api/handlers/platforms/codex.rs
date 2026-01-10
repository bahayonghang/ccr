// Codex CLI 配置管理 API 处理器
//
// 使用统一的响应工具模块减少重复代码

use axum::{Json, extract::Path, response::IntoResponse};
use serde::Deserialize;
use serde_json::json;

use crate::api::handlers::response::{bad_request, internal_error, not_found, ok, ok_message};
use crate::managers::config::codex_manager::CodexConfigManager;
use crate::models::platforms::codex::*;

// CCR 平台 Profiles（~/.ccr/platforms/codex/profiles.toml）
use ccr::{LockManager, Platform, ProfileConfig, create_platform};

const PLATFORM: &str = "Codex";

// ============ 辅助函数 ============

fn mask_token(token: &str) -> String {
    if token.len() <= 10 {
        "*".repeat(token.len())
    } else {
        let prefix = &token[..4];
        let suffix = &token[token.len() - 4..];
        format!("{}...{}", prefix, suffix)
    }
}

/// 初始化 Codex Manager 并处理错误
macro_rules! with_codex_manager {
    ($body:expr) => {
        match CodexConfigManager::default() {
            Ok(manager) => $body(manager),
            Err(e) => internal_error(format!("初始化 {} 配置管理器失败: {}", PLATFORM, e))
                .into_response(),
        }
    };
}

/// 处理 spawn_blocking 的结果
#[allow(dead_code)]
fn handle_blocking_result<T, F>(
    result: Result<Result<T, String>, tokio::task::JoinError>,
    on_success: F,
) -> impl IntoResponse
where
    F: FnOnce(T) -> axum::response::Response,
{
    match result {
        Ok(Ok(data)) => on_success(data),
        Ok(Err(e)) => bad_request(e).into_response(),
        Err(e) => internal_error(format!("任务执行失败: {}", e)).into_response(),
    }
}

// ============ MCP 服务器管理 ============

/// GET /api/codex/mcp - 列出所有 Codex MCP 服务器
pub async fn list_codex_mcp_servers() -> impl IntoResponse {
    with_codex_manager!(|manager: CodexConfigManager| {
        match manager.list_mcp_servers() {
            Ok(servers) => ok(CodexMcpListResponse { servers }).into_response(),
            Err(e) => {
                internal_error(format!("读取 {} MCP 服务器失败: {}", PLATFORM, e)).into_response()
            }
        }
    })
}

/// POST /api/codex/mcp - 添加 Codex MCP 服务器
pub async fn add_codex_mcp_server(Json(req): Json<CodexMcpServerRequest>) -> impl IntoResponse {
    // 验证请求：至少需要 command（STDIO）或 url（HTTP）
    if req.command.is_none() && req.url.is_none() {
        return bad_request("必须提供 command（STDIO 服务器）或 url（HTTP 服务器）")
            .into_response();
    }

    with_codex_manager!(|manager: CodexConfigManager| {
        let name = req
            .name
            .clone()
            .or(req.command.clone())
            .or(req.url.clone())
            .unwrap_or_else(|| "unknown".to_string());

        let server: CodexMcpServer = req.into();

        match manager.add_mcp_server(name.clone(), server) {
            Ok(_) => {
                ok_message(format!("{} MCP 服务器 '{}' 已成功添加", PLATFORM, name)).into_response()
            }
            Err(e) => {
                internal_error(format!("添加 {} MCP 服务器失败: {}", PLATFORM, e)).into_response()
            }
        }
    })
}

/// PUT /api/codex/mcp/:name - 更新 Codex MCP 服务器
pub async fn update_codex_mcp_server(
    Path(name): Path<String>,
    Json(req): Json<CodexMcpServerRequest>,
) -> impl IntoResponse {
    with_codex_manager!(|manager: CodexConfigManager| {
        let server: CodexMcpServer = req.into();

        match manager.update_mcp_server(&name, server) {
            Ok(_) => {
                ok_message(format!("{} MCP 服务器 '{}' 已成功更新", PLATFORM, name)).into_response()
            }
            Err(e) => {
                internal_error(format!("更新 {} MCP 服务器失败: {}", PLATFORM, e)).into_response()
            }
        }
    })
}

/// DELETE /api/codex/mcp/:name - 删除 Codex MCP 服务器
pub async fn delete_codex_mcp_server(Path(name): Path<String>) -> impl IntoResponse {
    with_codex_manager!(|manager: CodexConfigManager| {
        match manager.delete_mcp_server(&name) {
            Ok(_) => {
                ok_message(format!("{} MCP 服务器 '{}' 已成功删除", PLATFORM, name)).into_response()
            }
            Err(e) => not_found(format!("删除 {} MCP 服务器失败: {}", PLATFORM, e)).into_response(),
        }
    })
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
    let mut profile = existing.unwrap_or_default();

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

/// GET /api/codex/profiles - 列出所有 Codex Profiles
pub async fn list_codex_profiles() -> impl IntoResponse {
    let result = tokio::task::spawn_blocking(move || {
        let platform =
            create_platform(Platform::Codex).map_err(|e| format!("创建 Codex 平台失败: {}", e))?;

        let profiles = platform
            .load_profiles()
            .map_err(|e| format!("读取 Codex profiles.toml 失败: {}", e))?;

        let current_profile = platform
            .get_current_profile()
            .map_err(|e| format!("读取当前 Codex profile 失败: {}", e))?;

        let profiles: Vec<_> = profiles
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
            .collect();

        Ok::<_, String>((profiles, current_profile))
    })
    .await;

    match result {
        Ok(Ok((profiles, current_profile))) => {
            ok(json!({ "profiles": profiles, "current_profile": current_profile })).into_response()
        }
        Ok(Err(e)) => internal_error(format!("读取 Codex Profiles 失败: {}", e)).into_response(),
        Err(e) => internal_error(format!("任务执行失败: {}", e)).into_response(),
    }
}

/// GET /api/codex/profiles/:name - 获取单个 Codex Profile（完整 token）
pub async fn get_codex_profile(Path(name): Path<String>) -> impl IntoResponse {
    let result = tokio::task::spawn_blocking(move || {
        let platform =
            create_platform(Platform::Codex).map_err(|e| format!("创建 Codex 平台失败: {}", e))?;

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
        Ok(Ok(profile)) => ok(json!({ "profile": profile })).into_response(),
        Ok(Err(e)) => not_found(e).into_response(),
        Err(e) => internal_error(format!("任务执行失败: {}", e)).into_response(),
    }
}

/// POST /api/codex/profiles - 添加 Codex Profile
pub async fn add_codex_profile(Json(req): Json<CodexProfileUpsertRequest>) -> impl IntoResponse {
    let result = tokio::task::spawn_blocking(move || {
        let name = req
            .name
            .clone()
            .ok_or_else(|| "缺少 profile name".to_string())?;

        let lock_manager =
            LockManager::with_default_path().map_err(|e| format!("初始化锁管理器失败: {}", e))?;
        let _lock = lock_manager
            .lock_resource("ccr_codex_profiles", std::time::Duration::from_secs(10))
            .map_err(|e| format!("获取锁失败: {}", e))?;

        let platform =
            create_platform(Platform::Codex).map_err(|e| format!("创建 Codex 平台失败: {}", e))?;

        let profile = merge_profile(None, req);

        platform
            .save_profile(&name, &profile)
            .map_err(|e| format!("保存 profile 失败: {}", e))?;

        Ok::<_, String>(name)
    })
    .await;

    match result {
        Ok(Ok(name)) => ok_message(format!("Codex Profile '{}' 已成功添加", name)).into_response(),
        Ok(Err(e)) => bad_request(e).into_response(),
        Err(e) => internal_error(format!("任务执行失败: {}", e)).into_response(),
    }
}

/// PUT /api/codex/profiles/:name - 更新 Codex Profile
pub async fn update_codex_profile(
    Path(name): Path<String>,
    Json(req): Json<CodexProfileUpsertRequest>,
) -> impl IntoResponse {
    let name_for_response = name.clone();
    let result = tokio::task::spawn_blocking(move || {
        let lock_manager =
            LockManager::with_default_path().map_err(|e| format!("初始化锁管理器失败: {}", e))?;
        let _lock = lock_manager
            .lock_resource("ccr_codex_profiles", std::time::Duration::from_secs(10))
            .map_err(|e| format!("获取锁失败: {}", e))?;

        let platform =
            create_platform(Platform::Codex).map_err(|e| format!("创建 Codex 平台失败: {}", e))?;

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
        Ok(Ok(())) => {
            ok_message(format!("Codex Profile '{}' 已成功更新", name_for_response)).into_response()
        }
        Ok(Err(e)) => bad_request(format!("更新 Codex Profile 失败: {}", e)).into_response(),
        Err(e) => internal_error(format!("任务执行失败: {}", e)).into_response(),
    }
}

/// DELETE /api/codex/profiles/:name - 删除 Codex Profile
pub async fn delete_codex_profile(Path(name): Path<String>) -> impl IntoResponse {
    let name_for_response = name.clone();
    let result = tokio::task::spawn_blocking(move || {
        let lock_manager =
            LockManager::with_default_path().map_err(|e| format!("初始化锁管理器失败: {}", e))?;
        let _lock = lock_manager
            .lock_resource("ccr_codex_profiles", std::time::Duration::from_secs(10))
            .map_err(|e| format!("获取锁失败: {}", e))?;

        let platform =
            create_platform(Platform::Codex).map_err(|e| format!("创建 Codex 平台失败: {}", e))?;

        platform
            .delete_profile(&name)
            .map_err(|e| format!("删除 profile 失败: {}", e))?;

        Ok::<_, String>(())
    })
    .await;

    match result {
        Ok(Ok(())) => {
            ok_message(format!("Codex Profile '{}' 已成功删除", name_for_response)).into_response()
        }
        Ok(Err(e)) => not_found(format!("删除 Codex Profile 失败: {}", e)).into_response(),
        Err(e) => internal_error(format!("任务执行失败: {}", e)).into_response(),
    }
}

/// POST /api/codex/profiles/:name/apply - 应用 Codex Profile
pub async fn apply_codex_profile(Path(name): Path<String>) -> impl IntoResponse {
    let name_for_response = name.clone();
    let result = tokio::task::spawn_blocking(move || {
        let lock_manager =
            LockManager::with_default_path().map_err(|e| format!("初始化锁管理器失败: {}", e))?;
        let _lock = lock_manager
            .lock_resource("ccr_codex_apply", std::time::Duration::from_secs(20))
            .map_err(|e| format!("获取锁失败: {}", e))?;

        let platform =
            create_platform(Platform::Codex).map_err(|e| format!("创建 Codex 平台失败: {}", e))?;

        platform
            .apply_profile(&name)
            .map_err(|e| format!("应用 profile 失败: {}", e))?;

        Ok::<_, String>(())
    })
    .await;

    match result {
        Ok(Ok(())) => {
            ok_message(format!("Codex Profile '{}' 已成功应用", name_for_response)).into_response()
        }
        Ok(Err(e)) => bad_request(e).into_response(),
        Err(e) => internal_error(format!("任务执行失败: {}", e)).into_response(),
    }
}

// ============ 基础配置管理 ============

/// GET /api/codex/config - 获取完整的 Codex 配置
pub async fn get_codex_config() -> impl IntoResponse {
    with_codex_manager!(|manager: CodexConfigManager| {
        match manager.read_config() {
            Ok(config) => ok(CodexConfigResponse { config }).into_response(),
            Err(e) => internal_error(format!("读取 {} 配置失败: {}", PLATFORM, e)).into_response(),
        }
    })
}

/// PUT /api/codex/config - 更新 Codex 基础配置
pub async fn update_codex_base_config(Json(config): Json<CodexConfig>) -> impl IntoResponse {
    with_codex_manager!(|manager: CodexConfigManager| {
        match manager.update_base_config(
            config.model,
            config.model_provider,
            config.approval_policy,
            config.sandbox_mode,
            config.model_reasoning_effort,
        ) {
            Ok(_) => ok_message(format!("{} 基础配置已成功更新", PLATFORM)).into_response(),
            Err(e) => {
                internal_error(format!("更新 {} 基础配置失败: {}", PLATFORM, e)).into_response()
            }
        }
    })
}

// ============ Auth 账号管理 ============

use ccr::models::{LoginState as CcrLoginState, TokenFreshness as CcrTokenFreshness};
use ccr::services::CodexAuthService;

/// 将 ccr 的 TokenFreshness 转换为 API 模型
fn convert_freshness(freshness: CcrTokenFreshness) -> TokenFreshness {
    match freshness {
        CcrTokenFreshness::Fresh => TokenFreshness::Fresh,
        CcrTokenFreshness::Stale => TokenFreshness::Stale,
        CcrTokenFreshness::Old => TokenFreshness::Old,
        CcrTokenFreshness::Unknown => TokenFreshness::Unknown,
    }
}

/// 将 ccr 的 LoginState 转换为 API 模型
fn convert_login_state(state: CcrLoginState) -> LoginState {
    match state {
        CcrLoginState::NotLoggedIn => LoginState::NotLoggedIn,
        CcrLoginState::LoggedInUnsaved => LoginState::LoggedInUnsaved,
        CcrLoginState::LoggedInSaved(name) => LoginState::LoggedInSaved(name),
    }
}

/// GET /api/codex/auth/accounts - 列出所有 Codex Auth 账号
pub async fn list_codex_auth_accounts() -> impl IntoResponse {
    let result = tokio::task::spawn_blocking(move || {
        let service =
            CodexAuthService::new().map_err(|e| format!("初始化 Codex Auth 服务失败: {}", e))?;

        let accounts = service
            .list_accounts()
            .map_err(|e| format!("列出账号失败: {}", e))?;

        let login_state = service
            .get_login_state()
            .map_err(|e| format!("获取登录状态失败: {}", e))?;

        let accounts: Vec<CodexAuthAccountItem> = accounts
            .into_iter()
            .map(|item| {
                let freshness = convert_freshness(item.freshness);
                CodexAuthAccountItem {
                    name: item.name,
                    description: item.description,
                    email: item.email,
                    is_current: item.is_current,
                    is_virtual: item.is_virtual,
                    last_used: item.last_used.map(|dt| dt.to_rfc3339()),
                    last_refresh: item.last_refresh.map(|dt| dt.to_rfc3339()),
                    freshness,
                    freshness_icon: freshness.icon().to_string(),
                    freshness_description: freshness.description().to_string(),
                }
            })
            .collect();

        Ok::<_, String>((accounts, convert_login_state(login_state)))
    })
    .await;

    match result {
        Ok(Ok((accounts, login_state))) => ok(CodexAuthListResponse {
            accounts,
            login_state,
        })
        .into_response(),
        Ok(Err(e)) => internal_error(format!("列出 Codex Auth 账号失败: {}", e)).into_response(),
        Err(e) => internal_error(format!("任务执行失败: {}", e)).into_response(),
    }
}

/// GET /api/codex/auth/current - 获取当前 Codex Auth 信息
pub async fn get_codex_auth_current() -> impl IntoResponse {
    let result = tokio::task::spawn_blocking(move || {
        let service =
            CodexAuthService::new().map_err(|e| format!("初始化 Codex Auth 服务失败: {}", e))?;

        let login_state = service
            .get_login_state()
            .map_err(|e| format!("获取登录状态失败: {}", e))?;

        let info = match service.get_current_auth_info() {
            Ok(current) => {
                let freshness = convert_freshness(current.freshness);
                Some(CodexAuthCurrentInfo {
                    account_id: current.account_id,
                    email: current.email,
                    last_refresh: current.last_refresh.map(|dt| dt.to_rfc3339()),
                    freshness,
                    freshness_icon: freshness.icon().to_string(),
                    freshness_description: freshness.description().to_string(),
                })
            }
            Err(_) => None,
        };

        let logged_in = info.is_some();

        Ok::<_, String>((logged_in, info, convert_login_state(login_state)))
    })
    .await;

    match result {
        Ok(Ok((logged_in, info, login_state))) => ok(CodexAuthCurrentResponse {
            logged_in,
            info,
            login_state,
        })
        .into_response(),
        Ok(Err(e)) => {
            internal_error(format!("获取当前 Codex Auth 信息失败: {}", e)).into_response()
        }
        Err(e) => internal_error(format!("任务执行失败: {}", e)).into_response(),
    }
}

/// POST /api/codex/auth/save - 保存当前登录到命名账号
pub async fn save_codex_auth(Json(req): Json<CodexAuthSaveRequest>) -> impl IntoResponse {
    let result = tokio::task::spawn_blocking(move || {
        let service =
            CodexAuthService::new().map_err(|e| format!("初始化 Codex Auth 服务失败: {}", e))?;

        service
            .save_current(&req.name, req.description, req.force)
            .map_err(|e| format!("{}", e))?;

        Ok::<_, String>(req.name)
    })
    .await;

    match result {
        Ok(Ok(name)) => {
            ok_message(format!("Codex Auth 账号 '{}' 已成功保存", name)).into_response()
        }
        Ok(Err(e)) => bad_request(format!("保存 Codex Auth 账号失败: {}", e)).into_response(),
        Err(e) => internal_error(format!("任务执行失败: {}", e)).into_response(),
    }
}

/// POST /api/codex/auth/switch/{name} - 切换到指定账号
pub async fn switch_codex_auth(Path(name): Path<String>) -> impl IntoResponse {
    let name_for_response = name.clone();
    let result = tokio::task::spawn_blocking(move || {
        let service =
            CodexAuthService::new().map_err(|e| format!("初始化 Codex Auth 服务失败: {}", e))?;

        service
            .switch_account(&name)
            .map_err(|e| format!("{}", e))?;

        Ok::<_, String>(())
    })
    .await;

    match result {
        Ok(Ok(())) => {
            ok_message(format!("已切换到 Codex Auth 账号 '{}'", name_for_response)).into_response()
        }
        Ok(Err(e)) => bad_request(format!("切换 Codex Auth 账号失败: {}", e)).into_response(),
        Err(e) => internal_error(format!("任务执行失败: {}", e)).into_response(),
    }
}

/// DELETE /api/codex/auth/{name} - 删除指定账号
pub async fn delete_codex_auth(Path(name): Path<String>) -> impl IntoResponse {
    let name_for_response = name.clone();
    let result = tokio::task::spawn_blocking(move || {
        let service =
            CodexAuthService::new().map_err(|e| format!("初始化 Codex Auth 服务失败: {}", e))?;

        service
            .delete_account(&name)
            .map_err(|e| format!("{}", e))?;

        Ok::<_, String>(())
    })
    .await;

    match result {
        Ok(Ok(())) => ok_message(format!(
            "Codex Auth 账号 '{}' 已成功删除",
            name_for_response
        ))
        .into_response(),
        Ok(Err(e)) => not_found(format!("删除 Codex Auth 账号失败: {}", e)).into_response(),
        Err(e) => internal_error(format!("任务执行失败: {}", e)).into_response(),
    }
}

/// GET /api/codex/auth/process - 检测运行中的 Codex 进程
pub async fn detect_codex_process() -> impl IntoResponse {
    let result = tokio::task::spawn_blocking(move || {
        let service =
            CodexAuthService::new().map_err(|e| format!("初始化 Codex Auth 服务失败: {}", e))?;

        let pids = service.detect_codex_process();
        let has_running_process = !pids.is_empty();

        let warning = if has_running_process {
            Some(format!(
                "检测到 {} 个运行中的 Codex 进程 (PID: {})，切换账号前请先关闭这些进程",
                pids.len(),
                pids.iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
        } else {
            None
        };

        Ok::<_, String>(CodexAuthProcessResponse {
            has_running_process,
            pids,
            warning,
        })
    })
    .await;

    match result {
        Ok(Ok(response)) => ok(response).into_response(),
        Ok(Err(e)) => internal_error(format!("检测 Codex 进程失败: {}", e)).into_response(),
        Err(e) => internal_error(format!("任务执行失败: {}", e)).into_response(),
    }
}
