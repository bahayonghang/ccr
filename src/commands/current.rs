// current 命令实现 - 显示当前配置状态

use crate::config::ConfigManager;
use crate::error::Result;
use crate::logging::ColorOutput;
use crate::settings::SettingsManager;

/// 显示当前配置状态
pub fn current_command() -> Result<()> {
    ColorOutput::title("当前配置状态");

    // 加载配置
    let config_manager = ConfigManager::default()?;
    let config = config_manager.load()?;

    println!();
    ColorOutput::key_value("配置文件", &config_manager.config_path().display().to_string(), 2);
    ColorOutput::key_value("当前配置", &config.current_config, 2);
    ColorOutput::key_value("默认配置", &config.default_config, 2);
    println!();

    // 显示当前配置节的详细信息
    if let Ok(section) = config.get_current_section() {
        ColorOutput::step("配置详情:");
        ColorOutput::key_value("  描述", &section.display_description(), 2);
        if let Some(base_url) = &section.base_url {
            ColorOutput::key_value("  Base URL", base_url, 2);
        }
        if let Some(auth_token) = &section.auth_token {
            ColorOutput::key_value_sensitive("  Auth Token", auth_token, 2);
        }
        if let Some(model) = &section.model {
            ColorOutput::key_value("  Model", model, 2);
        }
        if let Some(small_model) = &section.small_fast_model {
            ColorOutput::key_value("  Small Fast Model", small_model, 2);
        }

        println!();

        // 验证配置
        match section.validate() {
            Ok(_) => ColorOutput::success("配置验证通过"),
            Err(e) => ColorOutput::error(&format!("配置验证失败: {}", e)),
        }
    } else {
        ColorOutput::warning(&format!("当前配置 '{}' 不存在", config.current_config));
    }

    println!();
    ColorOutput::separator();
    println!();

    // 加载并显示 Claude Code 设置状态
    match SettingsManager::default() {
        Ok(settings_manager) => {
            ColorOutput::step("Claude Code 环境变量状态:");
            println!();

            match settings_manager.load() {
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
                    ColorOutput::info("提示: 可能是首次使用，运行 'ccr switch <config>' 来初始化设置");
                }
            }
        }
        Err(e) => {
            ColorOutput::warning(&format!("无法访问 Claude Code 设置: {}", e));
        }
    }

    Ok(())
}
