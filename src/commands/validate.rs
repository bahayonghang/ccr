// ✅ validate 命令实现 - 验证配置和设置
// 🔍 全面检查配置文件和 Claude Code 设置的完整性

use crate::config::ConfigManager;
use crate::error::Result;
use crate::logging::ColorOutput;
use crate::settings::SettingsManager;
use colored::*;

/// ✅ 验证配置和设置
/// 
/// 验证流程:
/// 1. 📝 验证配置文件 (~/.ccs_config.toml)
///    - 文件是否存在
///    - 格式是否正确
///    - 所有配置节是否有效
///    - 当前配置是否存在
/// 
/// 2. 🌍 验证 Claude Code 设置 (~/.claude/settings.json)
///    - 文件是否存在
///    - 必需环境变量是否设置
///    - 环境变量值是否有效
/// 
/// 3. 📊 生成验证报告
///    - 显示错误和警告
///    - 提供修复建议
pub fn validate_command() -> Result<()> {
    ColorOutput::title("配置验证报告");
    println!();

    let mut has_errors = false;
    let mut has_warnings = false;

    // 验证配置文件
    ColorOutput::step("验证配置文件 (~/.ccs_config.toml)");
    let config_manager = ConfigManager::default()?;

    match config_manager.load() {
        Ok(config) => {
            ColorOutput::success(&format!("配置文件存在: {}", config_manager.config_path().display()));

            // 验证所有配置节
            let validation_results = config.validate_all();
            let mut valid_count = 0;
            let mut invalid_count = 0;

            println!();
            for (name, result) in &validation_results {
                match result {
                    Ok(_) => {
                        println!("  {} {}", "✓".green(), name);
                        valid_count += 1;
                    }
                    Err(e) => {
                        println!("  {} {} - {}", "✗".red(), name, e);
                        invalid_count += 1;
                        has_errors = true;
                    }
                }
            }

            println!();
            if invalid_count > 0 {
                ColorOutput::warning(&format!(
                    "配置节验证: {} 个通过, {} 个失败",
                    valid_count, invalid_count
                ));
            } else {
                ColorOutput::success(&format!("所有 {} 个配置节验证通过", valid_count));
            }

            // 验证当前配置
            println!();
            ColorOutput::step("当前配置验证");
            if config.sections.contains_key(&config.current_config) {
                ColorOutput::success(&format!("当前配置 '{}' 存在", config.current_config));
            } else {
                ColorOutput::error(&format!("当前配置 '{}' 不存在", config.current_config));
                has_errors = true;
            }
        }
        Err(e) => {
            ColorOutput::error(&format!("配置文件加载失败: {}", e));
            has_errors = true;
        }
    }

    println!();
    ColorOutput::separator();
    println!();

    // 验证 Claude Code 设置
    ColorOutput::step("验证 Claude Code 设置 (~/.claude/settings.json)");
    let settings_manager = match SettingsManager::default() {
        Ok(m) => m,
        Err(e) => {
            ColorOutput::error(&format!("无法访问设置管理器: {}", e));
            has_errors = true;
            return generate_report(has_errors, has_warnings);
        }
    };

    match settings_manager.load() {
        Ok(settings) => {
            ColorOutput::success(&format!("设置文件存在: {}", settings_manager.settings_path().display()));

            // 验证环境变量
            println!();
            ColorOutput::step("环境变量验证");

            let env_status = settings.anthropic_env_status();
            for (var_name, value) in env_status {
                match value {
                    Some(v) if !v.is_empty() => {
                        println!("  {} {}: {}", "✓".green(), var_name, ColorOutput::mask_sensitive(&v));
                    }
                    Some(_) => {
                        println!("  {} {}: {}", "⚠".yellow(), var_name, "值为空");
                        has_warnings = true;
                    }
                    None => {
                        // ANTHROPIC_SMALL_FAST_MODEL 是可选的
                        if var_name.contains("SMALL_FAST_MODEL") {
                            println!("  {} {}: {}", "○".dimmed(), var_name, "未设置（可选）");
                        } else {
                            println!("  {} {}: {}", "✗".red(), var_name, "未设置");
                            has_errors = true;
                        }
                    }
                }
            }

            println!();
            match settings.validate() {
                Ok(_) => ColorOutput::success("设置验证通过"),
                Err(e) => {
                    ColorOutput::error(&format!("设置验证失败: {}", e));
                    has_errors = true;
                }
            }
        }
        Err(e) => {
            ColorOutput::warning(&format!("设置文件不存在或无法读取: {}", e));
            ColorOutput::info("提示: 运行 'ccr switch <config>' 来初始化设置");
            has_warnings = true;
        }
    }

    println!();
    ColorOutput::separator();

    generate_report(has_errors, has_warnings)
}

fn generate_report(has_errors: bool, has_warnings: bool) -> Result<()> {
    println!();
    ColorOutput::title("验证总结");
    println!();

    if !has_errors && !has_warnings {
        ColorOutput::success("✓ 所有验证通过，配置状态正常");
        println!();
        return Ok(());
    }

    if has_errors {
        ColorOutput::error("✗ 发现配置错误，请修复后重试");
        println!();
        ColorOutput::info("建议:");
        println!("  1. 检查配置文件格式是否正确");
        println!("  2. 确保所有必填字段都已填写");
        println!("  3. 运行 'ccr list' 查看可用配置");
        println!("  4. 运行 'ccr switch <config>' 切换到有效配置");
    }

    if has_warnings {
        ColorOutput::warning("⚠ 发现配置警告，建议检查");
    }

    println!();
    Ok(())
}
