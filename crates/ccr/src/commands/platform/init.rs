//! 🆕 platform init 命令实现
//!
//! 初始化平台配置。

#![allow(clippy::unused_async)]

use crate::managers::PlatformConfigManager;
use crate::managers::config::{CcsConfig, GlobalSettings};
use crate::models::{Platform, PlatformPaths};
use crate::platforms::create_platform;
use ccr_core::core::error::{CcrError, Result};
use ccr_core::core::logging::ColorOutput;
use indexmap::IndexMap;
use std::fs;
use std::str::FromStr;

/// 🆕 初始化平台配置
///
/// 初始化指定平台的目录结构和配置文件，为平台使用做准备。
///
/// # 参数
///
/// * `platform_name` - 要初始化的平台名称 (`"claude"`, `"codex"`, `"gemini"` 等)
///
/// # 返回
///
/// * `Ok(())` - 成功初始化平台
/// * `Err(CcrError::PlatformNotFound)` - 指定的平台不存在或未实现
/// * `Err(CcrError::IoError)` - 创建目录或文件失败
///
/// # 初始化操作
///
/// 1. **验证平台**: 检查平台名称是否在支持列表中
/// 2. **创建目录**: 创建平台专用目录结构
///    - `~/.ccr/{platform}/` - 平台根目录
///    - `~/.ccr/{platform}/backups/` - 备份目录
/// 3. **注册平台**: 在 `~/.ccr/config.toml` 中注册平台
/// 4. **设置默认**: 如果是首个平台，设为默认和当前平台
pub async fn platform_init_command(platform_name: &str) -> Result<()> {
    ColorOutput::title(&format!("初始化平台: {}", platform_name));

    // 验证平台是否存在
    let platform = Platform::from_str(platform_name)
        .map_err(|_| CcrError::PlatformNotFound(platform_name.to_string()))?;

    let platform_impl = create_platform(platform)?;
    let paths = PlatformPaths::new(platform)?;

    println!();
    ColorOutput::info("正在创建平台目录结构...");

    // 使用 PlatformPaths 的统一方法创建所有必需的目录
    paths.ensure_directories()?;

    ColorOutput::success(&format!("✓ 根目录: {}", paths.root.display()));
    ColorOutput::success(&format!("✓ 平台目录: {}", paths.platform_dir.display()));
    ColorOutput::success(&format!(
        "✓ 历史目录: {}",
        paths
            .history_file
            .parent()
            .ok_or_else(|| CcrError::ConfigError("历史文件路径没有父目录".into()))?
            .display()
    ));
    ColorOutput::success(&format!("✓ 备份目录: {}", paths.backups_dir.display()));

    // 创建默认 profiles.toml 文件
    if !paths.profiles_file.exists() {
        ColorOutput::info(&format!("正在创建默认 {} profiles.toml...", platform_name));

        let default_ccs = CcsConfig {
            default_config: "default".to_string(),
            current_config: "default".to_string(),
            settings: GlobalSettings::default(),
            sections: IndexMap::new(),
        };

        let content = toml::to_string_pretty(&default_ccs)
            .map_err(|e| CcrError::ConfigError(format!("序列化默认配置失败: {}", e)))?;
        fs::write(&paths.profiles_file, content)
            .map_err(|e| CcrError::ConfigError(format!("写入默认 profiles.toml 失败: {}", e)))?;

        ColorOutput::success(&format!(
            "✓ Profiles 文件: {}",
            paths.profiles_file.display()
        ));
    } else {
        ColorOutput::info(&format!(
            "Profiles 文件已存在: {}",
            paths.profiles_file.display()
        ));
    }

    // 注册平台到统一配置
    let manager = PlatformConfigManager::with_default()?;
    let mut config = manager.load_or_create_default()?;

    if !config.platforms.contains_key(platform_name) {
        let registry = crate::managers::PlatformConfigEntry {
            description: Some(platform_impl.platform_name().to_string()),
            ..Default::default()
        };
        config.register_platform(platform_name.to_string(), registry)?;
        manager.save(&config)?;

        println!();
        ColorOutput::success(&format!("✓ 平台 '{}' 已注册到配置文件", platform_name));
    } else {
        println!();
        ColorOutput::info(&format!("平台 '{}' 已经注册", platform_name));
    }

    println!();
    ColorOutput::success("平台初始化完成！");
    println!();
    ColorOutput::info("下一步:");
    println!("  1. 使用相应平台的命令配置 profile");
    println!(
        "  2. 使用 'ccr platform switch {}' 切换到该平台",
        platform_name
    );

    Ok(())
}
