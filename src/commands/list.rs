// 📜 list 命令实现 - 列出所有可用配置
// 📋 显示所有配置节，突出显示当前配置和默认配置

use crate::config::ConfigManager;
use crate::error::Result;
use crate::logging::ColorOutput;

/// 📜 列出所有可用配置
/// 
/// 显示内容:
/// - ⚙️ 配置文件路径
/// - 🎯 默认配置和当前配置
/// - 📋 所有配置节列表（带验证状态）
/// - ▶️ 当前配置的详细信息
pub fn list_command() -> Result<()> {
    ColorOutput::title("可用配置列表");

    let config_manager = ConfigManager::default()?;
    let config = config_manager.load()?;

    println!();
    ColorOutput::info(&format!("配置文件: {}", config_manager.config_path().display()));
    ColorOutput::info(&format!("默认配置: {}", config.default_config));
    ColorOutput::info(&format!("当前配置: {}", config.current_config));
    println!();

    ColorOutput::separator();

    // 列出所有配置节
    let sections = config.list_sections();
    if sections.is_empty() {
        ColorOutput::warning("未找到任何配置节");
        return Ok(());
    }

    let sections_count = sections.len();

    for section_name in &sections {
        let section = config.get_section(section_name)?;
        let is_current = section_name == &config.current_config;

        ColorOutput::config_status(
            &section_name,
            is_current,
            section.description.as_deref(),
        );

        if is_current {
            // 显示当前配置的详细信息
            if let Some(base_url) = &section.base_url {
                println!("    Base URL: {}", base_url);
            }
            if let Some(auth_token) = &section.auth_token {
                println!(
                    "    Token: {}",
                    ColorOutput::mask_sensitive(auth_token)
                );
            }
            if let Some(model) = &section.model {
                println!("    Model: {}", model);
            }
            if let Some(small_model) = &section.small_fast_model {
                println!("    Small Fast Model: {}", small_model);
            }

            // 显示验证状态
            match section.validate() {
                Ok(_) => println!("    状态: {}", "✓ 配置完整".green()),
                Err(e) => println!("    状态: {} - {}", "✗ 配置不完整".red(), e),
            }
        }
    }

    println!();
    ColorOutput::success(&format!("共找到 {} 个配置", sections_count));

    Ok(())
}

use colored::*;
