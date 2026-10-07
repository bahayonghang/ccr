//! 🔍 platform current 命令实现
//!
//! 显示当前平台信息。

#![allow(clippy::unused_async)]

use super::types::{PlatformInfoOutput, PlatformPathsOutput};
use crate::managers::PlatformConfigManager;
use crate::models::{Platform, PlatformPaths};
use crate::platforms::create_platform;
use ccr_core::core::error::Result;
use ccr_core::core::logging::ColorOutput;
use std::str::FromStr;

/// 🔍 显示当前平台信息
///
/// 显示当前激活平台的详细信息，包括平台状态、profile、路径等。
///
/// # 参数
///
/// * `json` - 是否以 JSON 格式输出
///
/// # 返回
///
/// * `Ok(())` - 成功显示信息
/// * `Err(CcrError::PlatformNotFound)` - 当前平台不存在（配置损坏）
pub async fn platform_current_command(json: bool) -> Result<()> {
    let manager = PlatformConfigManager::with_default()?;
    let config = manager.load_or_create_default()?;

    let current_platform = &config.current_platform;
    let registry = config.get_platform(current_platform)?;

    // 获取路径信息
    let paths = if let Ok(platform) = Platform::from_str(current_platform) {
        PlatformPaths::new(platform).ok()
    } else {
        None
    };

    // 获取 profile 列表
    let profiles = if let Ok(platform) = Platform::from_str(current_platform) {
        if let Ok(platform_impl) = create_platform(platform) {
            platform_impl.list_profile_names().unwrap_or_else(|e| {
                tracing::debug!("获取 profile 列表失败: {}", e);
                Vec::new()
            })
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    // 📤 JSON 输出
    if json {
        let paths_output = paths.as_ref().map(|p| PlatformPathsOutput {
            platform_dir: p.platform_dir.display().to_string(),
            profiles_file: p.profiles_file.display().to_string(),
            history_file: p.history_file.display().to_string(),
            backups_dir: p.backups_dir.display().to_string(),
        });

        let output = PlatformInfoOutput {
            name: current_platform.clone(),
            display_name: registry
                .description
                .clone()
                .unwrap_or_else(|| current_platform.clone()),
            is_current: true,
            enabled: registry.enabled,
            current_profile: registry.current_profile.clone(),
            description: registry.description.clone(),
            paths: paths_output.unwrap_or_else(|| PlatformPathsOutput {
                platform_dir: String::new(),
                profiles_file: String::new(),
                history_file: String::new(),
                backups_dir: String::new(),
            }),
            profiles,
        };

        let json_str = serde_json::to_string_pretty(&output)?;
        println!("{}", json_str);

        return Ok(());
    }

    // 📊 表格输出
    ColorOutput::title("当前平台信息");

    println!();
    ColorOutput::key_value("平台名称", &current_platform.to_string(), 2);

    if let Some(desc) = &registry.description {
        ColorOutput::key_value("描述", &desc.to_string(), 2);
    }

    ColorOutput::key_value(
        "启用状态",
        if registry.enabled {
            "已启用"
        } else {
            "已禁用"
        },
        2,
    );

    if let Some(profile) = &registry.current_profile {
        ColorOutput::key_value("当前 Profile", &profile.to_string(), 2);
    } else {
        ColorOutput::warning("当前 Profile: 未配置");
    }

    if let Some(last_used) = &registry.last_used {
        ColorOutput::key_value("最后使用", &last_used.to_string(), 2);
    }

    println!();

    // 显示路径信息
    if let Some(p) = paths {
        println!();
        ColorOutput::info("平台路径:");
        println!("  配置目录: {}", p.platform_dir.display());
        println!("  Profiles: {}", p.profiles_file.display());
        println!("  设置文件: {}", p.settings_file.display());
        println!("  历史记录: {}", p.history_file.display());
        println!("  备份目录: {}", p.backups_dir.display());
    }

    Ok(())
}
