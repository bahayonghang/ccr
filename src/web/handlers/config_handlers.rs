// ⚙️ 配置管理处理器
// 处理配置的 CRUD 操作、导入导出等请求

use crate::application::{SwitchProfileRequest, switch_profile};
use crate::core::error::CcrError;
use crate::managers::config::ConfigSection;
use crate::models::ProfileConfig;
use crate::platforms::base::profile_to_section;
use crate::services::ConfigService;
use crate::web::{
    error_utils::{spawn_blocking_string, *},
    handlers::AppState,
    models::*,
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use indexmap::IndexMap;
use serde_json::json;
use std::str::FromStr;
use std::sync::Arc;

/// 处理列出配置请求
pub async fn handle_list_configs(State(_state): State<AppState>) -> Response {
    let result = spawn_blocking_string(get_platform_configs).await;

    let (current_config_name, configs_list) = match result {
        Ok(data) => data,
        Err(e) => return internal_server_error(e),
    };

    let configs: Vec<ConfigItem> = configs_list.into_iter().map(ConfigItem::from).collect();

    success_response(ConfigListResponse {
        current_config: current_config_name,
        default_config: "-".to_string(),
        configs,
    })
}

/// 处理获取单个配置请求（返回完整信息，不掩码 token）
/// 用于编辑功能，需要获取完整的 auth_token
pub async fn handle_get_config(
    State(_state): State<AppState>,
    Path(name): Path<String>,
) -> Response {
    let result = spawn_blocking_string(move || get_single_config(&name)).await;

    match result {
        Ok(config_item) => success_response(config_item),
        Err(e) => not_found(e),
    }
}

/// 处理切换配置请求
pub async fn handle_switch_config(
    State(_state): State<AppState>,
    Json(req): Json<SwitchRequest>,
) -> Response {
    let request = SwitchProfileRequest {
        config_name: req.config_name.clone(),
        platform_name: None,
    };

    match switch_profile(request).await {
        Ok(_) => success_response("配置切换成功"),
        Err(e) => internal_server_error(e.to_string()),
    }
}

/// 处理添加配置请求
pub async fn handle_add_config(
    State(state): State<AppState>,
    Json(req): Json<UpdateConfigRequest>,
) -> Response {
    let section = match req.to_config_section() {
        Ok(section) => section,
        Err(e) => return bad_request(e.to_string()),
    };
    let config_service = Arc::clone(&state.config_service);
    let name = req.name.clone();

    let result = spawn_blocking_string(move || config_service.add_config(name, section)).await;

    match result {
        Ok(_) => empty_success_response(),
        Err(e) => internal_server_error(e),
    }
}

/// 处理更新配置请求
pub async fn handle_update_config(
    State(state): State<AppState>,
    Path(old_name): Path<String>,
    Json(req): Json<UpdateConfigRequest>,
) -> Response {
    let section = match req.to_config_section() {
        Ok(section) => section,
        Err(e) => return bad_request(e.to_string()),
    };
    let new_name = req.name.clone();
    let config_service = Arc::clone(&state.config_service);

    let result =
        spawn_blocking_string(move || config_service.update_config(&old_name, new_name, section))
            .await;

    match result {
        Ok(_) => empty_success_response(),
        Err(e) => internal_server_error(e),
    }
}

/// 处理删除配置请求
pub async fn handle_delete_config(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Response {
    let config_service = Arc::clone(&state.config_service);

    let result = spawn_blocking_string(move || config_service.delete_config(&name)).await;

    match result {
        Ok(_) => empty_success_response(),
        Err(e) => internal_server_error(e),
    }
}

/// 处理导出配置请求
pub async fn handle_export(
    State(state): State<AppState>,
    Json(req): Json<ExportRequest>,
) -> Response {
    let include_secrets = req.include_secrets;
    let config_service = Arc::clone(&state.config_service);

    let result = spawn_blocking_string(move || config_service.export_config(include_secrets)).await;

    match result {
        Ok(content) => {
            let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
            let filename = format!(".ccs_config_{}.toml", timestamp);

            success_response(ExportResponse { content, filename })
        }
        Err(e) => internal_server_error(e),
    }
}

/// 处理导入配置请求
pub async fn handle_import(
    State(state): State<AppState>,
    Json(req): Json<ImportRequest>,
) -> Response {
    let mode = if req.mode == "replace" {
        crate::services::config_service::ImportMode::Replace
    } else {
        crate::services::config_service::ImportMode::Merge
    };

    let content = req.content.clone();
    let backup = req.backup;
    let config_service = Arc::clone(&state.config_service);

    let result =
        spawn_blocking_string(move || config_service.import_config(&content, mode, backup)).await;

    match result {
        Ok(result) => {
            let response_data = ImportResponse {
                added: result.added,
                updated: result.updated,
                skipped: result.skipped,
            };
            success_response(response_data)
        }
        Err(e) => internal_server_error(e),
    }
}

// 🎯 内部辅助函数

/// 通用的配置获取逻辑（消除重复代码！）
fn get_platform_configs()
-> Result<(String, Vec<crate::services::config_service::ConfigInfo>), CcrError> {
    use crate::managers::ConfigManager;

    let (is_unified, unified_config_path) = ConfigManager::detect_unified_mode();

    if is_unified {
        get_unified_mode_configs(unified_config_path)
    } else {
        get_legacy_mode_configs()
    }
}

/// 🎯 Unified 模式配置获取
fn get_unified_mode_configs(
    unified_path: Option<std::path::PathBuf>,
) -> Result<(String, Vec<crate::services::config_service::ConfigInfo>), CcrError> {
    use crate::managers::PlatformConfigManager;

    let unified_path =
        unified_path.ok_or_else(|| CcrError::ConfigError("无法获取统一配置路径".to_string()))?;

    let platform_manager = PlatformConfigManager::new(unified_path.clone());
    let unified_config = platform_manager.load()?;
    let current_platform = unified_config.current_platform.clone();

    let platform = match crate::models::Platform::from_str(&current_platform) {
        Ok(p) => p,
        Err(_) => return Err(CcrError::ConfigError("无效的平台".to_string())),
    };

    let platform_config = crate::platforms::create_platform(platform)?;

    // 🎯 处理平台加载错误 - 返回空配置而不是错误
    let profiles = match platform_config.load_profiles() {
        Ok(p) => p,
        Err(CcrError::PlatformNotSupported(msg)) => {
            tracing::warn!("平台 {} 尚未实现: {}", current_platform, msg);
            // 返回空配置,避免前端 500 错误
            return Ok(("-".to_string(), Vec::new()));
        }
        Err(e) => {
            // 其他错误（如文件读取失败、格式错误等）也返回空配置
            tracing::warn!("加载平台 {} 配置失败: {}, 返回空配置", current_platform, e);
            return Ok(("-".to_string(), Vec::new()));
        }
    };

    let current_profile = unified_config
        .platforms
        .get(&current_platform)
        .and_then(|p| p.current_profile.clone())
        .unwrap_or_else(|| "-".to_string());

    let configs: Vec<crate::services::config_service::ConfigInfo> = profiles
        .into_iter()
        .map(
            |(name, profile)| crate::services::config_service::ConfigInfo {
                name: name.clone(),
                description: profile.description.unwrap_or_default(),
                base_url: profile.base_url.clone(),
                auth_token: profile.auth_token.clone(),
                model: profile.model.clone(),
                small_fast_model: profile.small_fast_model.clone(),
                is_current: name == current_profile,
                is_default: false,
                provider: profile.provider.clone(),
                provider_type: profile.provider_type.clone(),
                account: profile.account.clone(),
                tags: profile.tags.clone(),
                usage_count: profile.usage_count.unwrap_or(0),
                enabled: profile.enabled.unwrap_or(true),
            },
        )
        .collect();

    Ok((current_profile, configs))
}

/// 🎯 Legacy 模式配置获取
fn get_legacy_mode_configs()
-> Result<(String, Vec<crate::services::config_service::ConfigInfo>), CcrError> {
    let config_service = ConfigService::with_default()?;
    let list = config_service.list_configs()?;
    Ok((list.current_config, list.configs))
}

/// 🎯 获取单个配置（不掩码 token，供编辑使用）
fn get_single_config(name: &str) -> Result<ConfigItem, CcrError> {
    use crate::managers::ConfigManager;

    let (is_unified, unified_config_path) = ConfigManager::detect_unified_mode();

    if is_unified {
        get_single_config_unified(name, unified_config_path)
    } else {
        get_single_config_legacy(name)
    }
}

/// 🎯 Unified 模式获取单个配置
fn get_single_config_unified(
    name: &str,
    unified_path: Option<std::path::PathBuf>,
) -> Result<ConfigItem, CcrError> {
    use crate::managers::PlatformConfigManager;

    let unified_path =
        unified_path.ok_or_else(|| CcrError::ConfigError("无法获取统一配置路径".to_string()))?;

    let platform_manager = PlatformConfigManager::new(unified_path.clone());
    let unified_config = platform_manager.load()?;
    let current_platform = unified_config.current_platform.clone();

    let platform = crate::models::Platform::from_str(&current_platform)
        .map_err(|_| CcrError::ConfigError("无效的平台".to_string()))?;

    let platform_config = crate::platforms::create_platform(platform)?;
    let profiles = platform_config.load_profiles()?;

    let current_profile = unified_config
        .platforms
        .get(&current_platform)
        .and_then(|p| p.current_profile.clone())
        .unwrap_or_else(|| "-".to_string());

    let profile = profiles
        .get(name)
        .ok_or_else(|| CcrError::ConfigError(format!("配置 '{}' 不存在", name)))?;

    Ok(ConfigItem {
        name: name.to_string(),
        description: profile.description.clone().unwrap_or_default(),
        base_url: profile.base_url.clone().unwrap_or_default(),
        auth_token: profile.auth_token.clone().unwrap_or_default(), // 🔑 完整 token，不掩码
        model: profile.model.clone(),
        small_fast_model: profile.small_fast_model.clone(),
        is_current: name == current_profile,
        is_default: false,
        provider: profile.provider.clone(),
        provider_type: profile.provider_type.clone(),
        account: profile.account.clone(),
        tags: profile.tags.clone(),
        usage_count: profile.usage_count.unwrap_or(0),
        enabled: profile.enabled.unwrap_or(true),
    })
}

/// 🎯 Legacy 模式获取单个配置
fn get_single_config_legacy(name: &str) -> Result<ConfigItem, CcrError> {
    let config_service = ConfigService::with_default()?;
    let config = config_service.get_config(name)?;

    Ok(ConfigItem {
        name: name.to_string(),
        description: config.description,               // String 类型
        base_url: config.base_url.unwrap_or_default(), // Option -> String
        auth_token: config.auth_token.unwrap_or_default(), // 🔑 完整 token，不掩码
        model: config.model,
        small_fast_model: config.small_fast_model,
        is_current: config.is_current,
        is_default: config.is_default,
        provider: config.provider,
        provider_type: config.provider_type,
        account: config.account,
        tags: config.tags,
        usage_count: config.usage_count, // u32 类型
        enabled: config.enabled,         // bool 类型
    })
}

// 🎯 为 UpdateConfigRequest 实现 to_config_section 方法
impl UpdateConfigRequest {
    fn to_profile_config(&self) -> ProfileConfig {
        ProfileConfig {
            description: self.description.clone(),
            base_url: Some(self.base_url.clone()),
            auth_token: Some(self.auth_token.clone()),
            model: self.model.clone(),
            small_fast_model: self.small_fast_model.clone(),
            provider: self.provider.clone(),
            provider_type: self.provider_type.clone(),
            account: self.account.clone(),
            tags: self.tags.clone(),
            usage_count: Some(0),
            enabled: Some(true),
            platform_data: IndexMap::new(),
        }
    }

    fn to_config_section(&self) -> Result<ConfigSection, CcrError> {
        profile_to_section(&self.to_profile_config())
    }
}

// 🎯 为 ConfigInfo 实现 From trait 用于转换为 ConfigItem
impl From<crate::services::config_service::ConfigInfo> for ConfigItem {
    fn from(info: crate::services::config_service::ConfigInfo) -> Self {
        ConfigItem {
            name: info.name,
            description: info.description,
            base_url: info.base_url.unwrap_or_default(),
            auth_token: crate::core::logging::ColorOutput::mask_sensitive(
                &info.auth_token.unwrap_or_default(),
            ),
            model: info.model,
            small_fast_model: info.small_fast_model,
            is_current: info.is_current,
            is_default: info.is_default,
            provider: info.provider,
            provider_type: info.provider_type,
            account: info.account,
            tags: info.tags,
            usage_count: info.usage_count,
            enabled: info.enabled,
        }
    }
}

/// ✅ 启用配置
pub async fn enable_config(Path(name): Path<String>) -> impl IntoResponse {
    match ConfigService::with_default() {
        Ok(service) => match service.enable_config(&name) {
            Ok(_) => (
                StatusCode::OK,
                Json(json!({ "message": format!("配置 '{}' 已启用", name) })),
            ),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            ),
        },
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

/// ❌ 禁用配置
pub async fn disable_config(Path(name): Path<String>) -> impl IntoResponse {
    match ConfigService::with_default() {
        Ok(service) => match service.disable_config(&name) {
            Ok(_) => (
                StatusCode::OK,
                Json(json!({ "message": format!("配置 '{}' 已禁用", name) })),
            ),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            ),
        },
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}
