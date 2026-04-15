// ✅ validate 命令实现 - 验证配置和设置
// 🔍 全面检查配置文件和 Claude Code 设置的完整性

#![allow(clippy::unused_async)]

use crate::managers::PlatformConfigManager;
use crate::models::{ClaudeProfileAuthMode, PlatformConfig};
use crate::platforms::ClaudePlatform;
use crate::services::{ClaudeAuthService, ConfigService, SettingsService};
use ccr_core::core::error::Result;
use ccr_core::core::logging::ColorOutput;
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
pub async fn validate_command() -> Result<()> {
    ColorOutput::title("配置验证报告");
    println!();

    let mut has_errors = false;
    let mut has_warnings = false;

    // 使用 ConfigService 验证配置文件
    ColorOutput::step("验证配置文件 (~/.ccs_config.toml)");
    let config_service = ConfigService::with_default()?;

    match config_service.validate_all() {
        Ok(report) => {
            ColorOutput::success(&format!(
                "配置文件存在: {}",
                config_service.config_manager().config_path().display()
            ));

            // 显示验证结果
            println!();
            for (name, is_valid, error_msg) in &report.results {
                if *is_valid {
                    println!("  {} {}", "✓".green(), name);
                } else {
                    if let Some(msg) = error_msg {
                        println!("  {} {} - {}", "✗".red(), name, msg);
                    } else {
                        println!("  {} {}", "✗".red(), name);
                    }
                    has_errors = true;
                }
            }

            println!();
            if report.invalid_count > 0 {
                ColorOutput::warning(&format!(
                    "配置节验证: {} 个通过, {} 个失败",
                    report.valid_count, report.invalid_count
                ));
            } else {
                ColorOutput::success(&format!("所有 {} 个配置节验证通过", report.valid_count));
            }

            // 验证当前配置
            println!();
            ColorOutput::step("当前配置验证");
            let config = config_service.load_config()?;
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

    let current_platform = PlatformConfigManager::with_default()
        .and_then(|manager| manager.load())
        .map(|config| config.current_platform)
        .unwrap_or_else(|_| "claude".to_string());

    // 使用 SettingsService 验证 Claude Code 设置
    ColorOutput::step("验证 Claude Code 设置 (~/.claude/settings.json)");
    let settings_service = match SettingsService::with_default() {
        Ok(s) => s,
        Err(e) => {
            ColorOutput::error(&format!("无法访问设置服务: {}", e));
            has_errors = true;
            return generate_report(has_errors, has_warnings);
        }
    };

    match settings_service.get_current_settings_async().await {
        Ok(settings) => {
            ColorOutput::success(&format!(
                "设置文件存在: {}",
                settings_service
                    .settings_manager()
                    .settings_path()
                    .display()
            ));

            let claude_profile_auth_mode = if current_platform == "claude" {
                ClaudePlatform::new()
                    .and_then(|platform| {
                        let current_profile = platform.get_current_profile()?;
                        let profiles = platform.load_profiles()?;
                        Ok(current_profile.and_then(|name| {
                            profiles.get(&name).map(ClaudePlatform::profile_auth_mode)
                        }))
                    })
                    .ok()
                    .flatten()
            } else {
                None
            };

            println!();
            ColorOutput::step("环境变量验证");

            let env_status = settings.anthropic_env_status();
            for (var_name, value) in env_status {
                match value {
                    Some(v) if !v.is_empty() => {
                        println!(
                            "  {} {}: {}",
                            "✓".green(),
                            var_name,
                            ColorOutput::mask_sensitive(&v)
                        );
                    }
                    Some(_) => {
                        println!("  {} {}: 值为空", "⚠".yellow(), var_name);
                        has_warnings = true;
                    }
                    None => {
                        let is_optional = var_name.contains("SMALL_FAST_MODEL");
                        let is_subscription_mode =
                            claude_profile_auth_mode == Some(ClaudeProfileAuthMode::Subscription);

                        if is_optional || is_subscription_mode {
                            println!("  {} {}: 未设置", "○".dimmed(), var_name);
                        } else {
                            println!("  {} {}: 未设置", "✗".red(), var_name);
                            has_errors = true;
                        }
                    }
                }
            }

            println!();
            match claude_profile_auth_mode {
                Some(ClaudeProfileAuthMode::Subscription) => {
                    ColorOutput::info("当前 Claude Profile 使用 subscription 模式");
                    let auth_service = ClaudeAuthService::new()?;
                    let snapshot = auth_service.read_auth_snapshot()?;
                    if let Some(info) = snapshot.current_info {
                        if let Some(email) = info.email {
                            ColorOutput::info(&format!(
                                "检测到官方登录邮箱: {}",
                                auth_service.mask_email(&email)
                            ));
                        }
                        if let Some(subscription_type) = info.subscription_type {
                            ColorOutput::info(&format!("订阅类型: {}", subscription_type));
                        }
                        if let Some(rate_limit_tier) = info.rate_limit_tier {
                            ColorOutput::info(&format!("速率档位: {}", rate_limit_tier));
                        }
                    } else {
                        ColorOutput::error("未检测到 Claude 官方订阅凭据");
                        has_errors = true;
                    }

                    if snapshot.runtime_usable {
                        ColorOutput::success("subscription 凭据验证通过");
                    } else {
                        ColorOutput::error("subscription 凭据不存在、无法解析或已过期");
                        has_errors = true;
                    }

                    if settings.has_anthropic_overrides() {
                        ColorOutput::warning(
                            "subscription 模式下检测到 ANTHROPIC_* 覆盖；实际运行时可能仍受 API key 覆盖影响",
                        );
                        has_warnings = true;
                    }
                }
                _ => match settings.validate_api_key_mode() {
                    Ok(_) => ColorOutput::success("设置验证通过"),
                    Err(e) => {
                        ColorOutput::error(&format!("设置验证失败: {}", e));
                        has_errors = true;
                    }
                },
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
        ColorOutput::success("✓ 所有验证通过,配置状态正常");
        println!();
        return Ok(());
    }

    if has_errors {
        ColorOutput::error("✗ 发现配置错误,请修复后重试");
        println!();
        ColorOutput::info("建议:");
        println!("  1. 检查配置文件格式是否正确");
        println!("  2. 确保所有必填字段都已填写");
        println!("  3. 运行 'ccr list' 查看可用配置");
        println!("  4. 运行 'ccr switch <config>' 切换到有效配置");
    }

    if has_warnings {
        ColorOutput::warning("⚠ 发现配置警告,建议检查");
    }

    println!();
    Ok(())
}
