// 🔍 current 命令实现 - 显示当前配置状态
// 📊 显示当前激活的配置详情和 Claude Code 环境变量状态

use crate::error::Result;
use crate::logging::ColorOutput;
use crate::services::{ConfigService, SettingsService};
use crate::utils::Validatable;

/// 🔍 显示当前配置状态
///
/// 显示内容分为两部分:
/// 1. 📝 配置文件信息
///    - 当前配置名称
///    - 配置详情（描述、URL、Token、模型等）
///    - 配置验证状态
///
/// 2. 🌍 Claude Code 环境变量状态
///    - ANTHROPIC_* 环境变量当前值
///    - 设置验证状态
pub fn current_command() -> Result<()> {
    ColorOutput::title("当前配置状态");

    // 使用 ConfigService
    let config_service = ConfigService::default()?;
    let current_info = config_service.get_current()?;

    println!();
    ColorOutput::key_value(
        "配置文件",
        &config_service
            .config_manager()
            .config_path()
            .display()
            .to_string(),
        2,
    );
    ColorOutput::key_value("当前配置", &current_info.name, 2);
    ColorOutput::key_value("默认配置", &config_service.load_config()?.default_config, 2);
    println!();

    // 显示当前配置节的详细信息
    ColorOutput::step("配置详情:");
    ColorOutput::key_value("  描述", &current_info.description, 2);
    if let Some(base_url) = &current_info.base_url {
        ColorOutput::key_value("  Base URL", base_url, 2);
    }
    if let Some(auth_token) = &current_info.auth_token {
        ColorOutput::key_value_sensitive("  Auth Token", auth_token, 2);
    }
    if let Some(model) = &current_info.model {
        ColorOutput::key_value("  Model", model, 2);
    }
    if let Some(small_model) = &current_info.small_fast_model {
        ColorOutput::key_value("  Small Fast Model", small_model, 2);
    }

    println!();

    // 验证配置
    let config = config_service.load_config()?;
    let section = config.get_current_section()?;
    match section.validate() {
        Ok(_) => ColorOutput::success("配置验证通过"),
        Err(e) => ColorOutput::error(&format!("配置验证失败: {}", e)),
    }

    println!();
    ColorOutput::separator();
    println!();

    // 使用 SettingsService 显示 Claude Code 设置状态
    match SettingsService::default() {
        Ok(settings_service) => {
            ColorOutput::step("Claude Code 环境变量状态:");
            println!();

            match settings_service.get_current_settings() {
                Ok(settings) => {
                    let env_status = settings.anthropic_env_status();

                    for (var_name, value) in env_status {
                        let is_sensitive = var_name.contains("TOKEN") || var_name.contains("KEY");
                        ColorOutput::env_status(&var_name, value.as_deref(), is_sensitive);
                    }

                    println!();

                    // 验证设置
                    match settings.validate() {
                        Ok(_) => ColorOutput::success("Claude Code 设置验证通过"),
                        Err(e) => ColorOutput::warning(&format!("设置验证警告: {}", e)),
                    }
                }
                Err(e) => {
                    ColorOutput::warning(&format!("无法加载 Claude Code 设置: {}", e));
                    ColorOutput::info(
                        "提示: 可能是首次使用，运行 'ccr switch <config>' 来初始化设置",
                    );
                }
            }
        }
        Err(e) => {
            ColorOutput::warning(&format!("无法访问 Claude Code 设置: {}", e));
        }
    }

    Ok(())
}
