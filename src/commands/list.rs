// 📜 list 命令实现 - 列出所有可用配置
// 📋 显示所有配置节,突出显示当前配置和默认配置

use crate::error::Result;
use crate::logging::ColorOutput;
use crate::services::ConfigService;
use crate::utils::Validatable;
use colored::*;

/// 📜 列出所有可用配置
///
/// 显示内容:
/// - ⚙️ 配置文件路径
/// - 🎯 默认配置和当前配置
/// - 📋 所有配置节列表(带验证状态)
/// - ▶️ 当前配置的详细信息
pub fn list_command() -> Result<()> {
    ColorOutput::title("可用配置列表");

    // 使用 ConfigService
    let service = ConfigService::default()?;
    let list = service.list_configs()?;

    println!();
    ColorOutput::info(&format!(
        "配置文件: {}",
        service.config_manager().config_path().display()
    ));
    ColorOutput::info(&format!("默认配置: {}", list.default_config));
    ColorOutput::info(&format!("当前配置: {}", list.current_config));
    println!();

    ColorOutput::separator();

    // 列出所有配置节
    if list.configs.is_empty() {
        ColorOutput::warning("未找到任何配置节");
        return Ok(());
    }

    let sections_count = list.configs.len();

    for config_info in &list.configs {
        ColorOutput::config_status(
            &config_info.name,
            config_info.is_current,
            Some(&config_info.description),
        );

        if config_info.is_current {
            // 显示当前配置的详细信息
            if let Some(base_url) = &config_info.base_url {
                println!("    Base URL: {}", base_url);
            }
            if let Some(auth_token) = &config_info.auth_token {
                println!("    Token: {}", ColorOutput::mask_sensitive(auth_token));
            }
            if let Some(model) = &config_info.model {
                println!("    Model: {}", model);
            }
            if let Some(small_model) = &config_info.small_fast_model {
                println!("    Small Fast Model: {}", small_model);
            }

            // === 🆕 显示分类信息 ===
            if let Some(provider_type) = &config_info.provider_type {
                println!("    类型: {}", provider_type.cyan());
            }
            if let Some(provider) = &config_info.provider {
                println!("    提供商: {}", provider.cyan());
            }
            if let Some(account) = &config_info.account {
                println!("    账号: {}", account.yellow());
            }
            if let Some(tags) = &config_info.tags {
                if !tags.is_empty() {
                    println!("    标签: {}", tags.join(", ").magenta());
                }
            }

            // 从原始配置获取 section 来验证
            let config = service.load_config()?;
            let section = config.get_section(&config_info.name)?;

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
