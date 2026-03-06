// 🆕 平台管理处理器 (Unified Mode)
// 处理多平台配置管理请求

use crate::application::{SwitchPlatformRequest as AppSwitchPlatformRequest, switch_platform};
use crate::managers::{ConfigManager, PlatformConfigManager};
use crate::web::{
    error_utils::{spawn_blocking_string, *},
    models::*,
};
use axum::Json;
use axum::response::Response;

/// 获取平台信息
/// 返回当前模式（Legacy/Unified）和平台列表
pub async fn handle_get_platform_info() -> Response {
    // 🔍 检测配置模式
    let (is_unified, unified_config_path) = ConfigManager::detect_unified_mode();

    if !is_unified {
        // Legacy 模式
        let response = PlatformInfoResponse {
            mode: "legacy".to_string(),
            current_platform: None,
            available_platforms: None,
        };
        return success_response(response);
    }

    // Unified 模式
    let unified_path = match unified_config_path {
        Some(path) => path,
        None => return internal_server_error("无法获取统一配置路径"),
    };

    let platform_manager = PlatformConfigManager::new(unified_path);

    let result = spawn_blocking_string(move || {
        // 读取统一配置
        let unified_config = platform_manager.load()?;
        let current_platform = unified_config.current_platform.clone();

        // 构建平台列表
        let mut platforms: Vec<PlatformItem> = unified_config
            .platforms
            .into_iter()
            .map(|(name, entry)| PlatformItem {
                name: name.clone(),
                enabled: entry.enabled,
                current_profile: entry.current_profile,
                description: entry.description,
                last_used: entry.last_used,
                is_current: name == current_platform,
            })
            .collect();

        // 按名称排序
        platforms.sort_by(|a, b| a.name.cmp(&b.name));

        Ok::<(String, Vec<PlatformItem>), crate::core::error::CcrError>((
            current_platform,
            platforms,
        ))
    })
    .await;

    match result {
        Ok((current_platform, platforms)) => {
            let response = PlatformInfoResponse {
                mode: "unified".to_string(),
                current_platform: Some(current_platform),
                available_platforms: Some(platforms),
            };
            success_response(response)
        }
        Err(e) => internal_server_error(e),
    }
}

/// 处理切换平台请求
/// 仅在 Unified 模式下可用
pub async fn handle_switch_platform(Json(req): Json<SwitchPlatformRequest>) -> Response {
    // 🔍 检测配置模式
    let (is_unified, unified_config_path) = ConfigManager::detect_unified_mode();

    if !is_unified {
        return bad_request("平台切换仅在 Unified 模式下可用");
    }

    let _unified_path = match unified_config_path {
        Some(path) => path,
        None => return internal_server_error("无法获取统一配置路径"),
    };

    let platform_name = req.platform_name.clone();
    let result = spawn_blocking_string(move || {
        let result = switch_platform(AppSwitchPlatformRequest {
            platform_name: platform_name.clone(),
        })?;
        Ok::<String, crate::core::error::CcrError>(result.new_platform)
    })
    .await;

    match result {
        Ok(platform_name) => {
            let message = format!("已切换到平台: {}", platform_name);
            success_response(message)
        }
        Err(e) => bad_request(e),
    }
}
