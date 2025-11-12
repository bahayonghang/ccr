// ⚙️ 配置管理处理器
// 处理配置的 CRUD 操作、导入导出等请求

use crate::core::error::CcrError;
use crate::managers::config::ConfigSection;
use crate::services::ConfigService;
use crate::web::{
    error_utils::{spawn_blocking_string, *},
    handlers::AppState,
    models::*,
};
use axum::{
    Json,
    extract::{Path, State},
    response::Response,
};
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

/// 处理切换配置请求
pub async fn handle_switch_config(
    State(state): State<AppState>,
    Json(req): Json<SwitchRequest>,
) -> Response {
    let config_name = req.config_name.clone();

    let result = spawn_blocking_string(move || {
        // 注意：这里使用 commands 模块的函数，因为它会更新实际的环境变量
        crate::commands::switch_command(&config_name)
    })
    .await;

    match result {
        Ok(_) => {
            // 切换成功后，给文件系统一点时间确保历史记录写入完成
            // 这对于某些文件系统（特别是网络文件系统）可能是必要的
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

            // 验证历史记录已成功写入
            match state.history_service.get_recent(1) {
                Ok(_) => success_response("配置切换成功"),
                Err(e) => {
                    log::warn!("历史记录可能未正确保存: {}", e);
                    success_response("配置切换成功（历史记录可能延迟）")
                }
            }
        }
        Err(e) => internal_server_error(e),
    }
}

/// 处理添加配置请求
pub async fn handle_add_config(
    State(state): State<AppState>,
    Json(req): Json<UpdateConfigRequest>,
) -> Response {
    let section = req.to_config_section();
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
    let section = req.to_config_section();
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
    let profiles = platform_config.load_profiles()?;

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

// 🎯 为 UpdateConfigRequest 实现 to_config_section 方法
impl UpdateConfigRequest {
    fn to_config_section(&self) -> ConfigSection {
        ConfigSection {
            description: self.description.clone(),
            base_url: Some(self.base_url.clone()),
            auth_token: Some(self.auth_token.clone()),
            model: self.model.clone(),
            small_fast_model: self.small_fast_model.clone(),
            provider: self.provider.clone(),
            provider_type: self.provider_type.as_ref().and_then(|t| match t.as_str() {
                "official_relay" => Some(crate::managers::config::ProviderType::OfficialRelay),
                "third_party_model" => Some(crate::managers::config::ProviderType::ThirdPartyModel),
                _ => None,
            }),
            account: self.account.clone(),
            tags: self.tags.clone(),
        }
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
        }
    }
}
