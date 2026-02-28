//! 🔄 platform switch 命令实现
//!
//! 切换当前激活的平台。

#![allow(clippy::unused_async)]

use crate::application::{SwitchPlatformRequest, switch_platform};
use crate::core::error::Result;
use crate::core::logging::ColorOutput;
use colored::Colorize;

/// 🔄 切换当前平台
///
/// 将当前激活的平台切换到指定的平台。如果目标平台未注册，会自动注册该平台。
///
/// # 参数
///
/// * `platform_name` - 要切换到的平台名称 (`"claude"`, `"codex"`, `"gemini"` 等)
///
/// # 返回
///
/// * `Ok(())` - 成功切换平台
/// * `Err(CcrError::PlatformNotFound)` - 指定的平台不存在或未实现
/// * `Err(CcrError::ConfigError)` - 平台已禁用或配置错误
///
/// # 行为说明
///
/// 1. **平台验证**: 检查平台名称是否有效（是否在支持列表中）
/// 2. **自动注册**: 如果平台未在配置中注册，自动注册并启用
/// 3. **状态更新**: 更新 `current_platform` 字段
/// 4. **时间戳记录**: 记录旧平台的最后使用时间
/// 5. **配置保存**: 将更改持久化到 `~/.ccr/config.toml`
/// 6. **提示信息**: 显示切换结果和当前 profile（如果有）
pub async fn platform_switch_command(platform_name: &str) -> Result<()> {
    ColorOutput::title(&format!("切换到平台: {}", platform_name));

    let result = switch_platform(SwitchPlatformRequest {
        platform_name: platform_name.to_string(),
    })?;
    let old_platform = result.old_platform;

    println!();
    ColorOutput::success(&format!(
        "已从平台 '{}' 切换到 '{}'",
        old_platform.bright_yellow(),
        platform_name.bright_green().bold()
    ));

    // 显示当前 profile
    if let Some(profile) = result.current_profile.as_ref() {
        println!();
        ColorOutput::info(&format!("当前 profile: {}", profile.bright_cyan()));
    } else {
        println!();
        ColorOutput::warning("该平台尚未配置 profile");
        println!("  提示: 使用相应平台的命令配置 profile");
    }

    Ok(())
}
