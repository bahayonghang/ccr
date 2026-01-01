//! ℹ️ platform info 命令实现
//!
//! 显示指定平台的详细信息。

#![allow(clippy::unused_async)]

use super::types::{PlatformInfoOutput, PlatformPathsOutput};
use crate::core::error::{CcrError, Result};
use crate::core::logging::ColorOutput;
use crate::managers::PlatformConfigManager;
use crate::models::{Platform, PlatformPaths};
use crate::platforms::create_platform;
use colored::Colorize;
use std::str::FromStr;

/// ℹ️ 显示指定平台的详细信息
///
/// 显示特定平台的完整信息，包括状态、profiles、路径配置等。
/// 比 `platform_current_command` 更灵活，可以查看任意平台的信息。
///
/// # 参数
///
/// * `platform_name` - 要查看的平台名称 (`"claude"`, `"codex"`, `"gemini"` 等)
/// * `json` - 是否以 JSON 格式输出
///
/// # 返回
///
/// * `Ok(())` - 成功显示信息
/// * `Err(CcrError::PlatformNotFound)` - 指定的平台不存在或未实现
pub async fn platform_info_command(platform_name: &str, json: bool) -> Result<()> {
    // 验证平台是否存在
    let platform = Platform::from_str(platform_name)
        .map_err(|_| CcrError::PlatformNotFound(platform_name.to_string()))?;

    let platform_impl = create_platform(platform)?;
    let paths = PlatformPaths::new(platform)?;

    // 检查是否为当前平台
    let manager = PlatformConfigManager::with_default()?;
    let config = manager.load_or_create_default()?;
    let is_current = platform_name == config.current_platform;

    // 获取注册信息
    let registry = config.platforms.get(platform_name);
    let enabled = registry.map(|r| r.enabled).unwrap_or(false);
    let current_profile_name = registry.and_then(|r| r.current_profile.clone());
    let description = registry.and_then(|r| r.description.clone());

    // 获取 profiles 列表
    let profiles = platform_impl.list_profile_names().unwrap_or_else(|e| {
        tracing::debug!("获取 {} 平台 profile 列表失败: {}", platform_name, e);
        Vec::new()
    });

    // 📤 JSON 输出
    if json {
        let output = PlatformInfoOutput {
            name: platform_name.to_string(),
            display_name: platform_impl.platform_name().to_string(),
            is_current,
            enabled,
            current_profile: current_profile_name,
            description,
            paths: PlatformPathsOutput {
                platform_dir: paths.platform_dir.display().to_string(),
                profiles_file: paths.profiles_file.display().to_string(),
                history_file: paths.history_file.display().to_string(),
                backups_dir: paths.backups_dir.display().to_string(),
            },
            profiles,
        };

        let json_str = serde_json::to_string_pretty(&output)?;
        println!("{}", json_str);

        return Ok(());
    }

    // 📊 表格输出
    ColorOutput::title(&format!("平台信息: {}", platform_name));

    println!();
    ColorOutput::info(&format!(
        "平台类型: {}",
        format!("{:?}", platform_impl.platform_type()).bright_cyan()
    ));
    ColorOutput::info(&format!("显示名称: {}", platform_impl.platform_name()));

    // 获取路径信息
    println!();
    ColorOutput::info("路径配置:");
    println!("  根目录: {}", paths.root.display());
    println!("  注册表: {}", paths.registry_file.display());
    println!("  平台目录: {}", paths.platform_dir.display());
    println!("  Profiles 文件: {}", paths.profiles_file.display());
    println!("  设置文件: {}", paths.settings_file.display());
    println!("  历史文件: {}", paths.history_file.display());
    println!("  备份目录: {}", paths.backups_dir.display());

    // 检测平台状态
    println!();
    ColorOutput::info("平台状态:");
    println!(
        "  Profiles 文件: {}",
        if paths.profiles_file.exists() {
            "存在 ✓".green()
        } else {
            "不存在 ✗".red()
        }
    );
    println!(
        "  设置文件: {}",
        if paths.settings_file.exists() {
            "存在 ✓".green()
        } else {
            "不存在 ✗".red()
        }
    );
    println!(
        "  平台目录: {}",
        if paths.platform_dir.exists() {
            "存在 ✓".green()
        } else {
            "不存在 ✗".red()
        }
    );

    // 加载并显示 profiles
    println!();
    ColorOutput::info("已配置 Profiles:");

    match platform_impl.load_profiles() {
        Ok(profiles_map) => {
            if profiles_map.is_empty() {
                println!("  (无)");
            } else {
                // 获取当前 profile
                let current_profile = platform_impl.get_current_profile().ok().flatten();

                for (name, profile) in profiles_map {
                    let marker = if Some(&name) == current_profile.as_ref() {
                        "▶ ".green()
                    } else {
                        "  ".normal()
                    };

                    if let Some(desc) = &profile.description {
                        println!("{}{} - {}", marker, name.bright_cyan(), desc);
                    } else {
                        println!("{}{}", marker, name.bright_cyan());
                    }
                }
            }
        }
        Err(e) => {
            ColorOutput::warning(&format!("无法加载 profiles: {}", e));
        }
    }

    Ok(())
}
