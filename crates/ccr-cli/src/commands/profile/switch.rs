// 🔄 switch 命令实现 - 切换配置
// 💎 这是 CCR 最核心的命令,负责完整的配置切换流程
//
// 执行流程(3 个步骤):
// 1. 📖 读取并验证目标配置 (从平台配置加载)
// 2. ✏️ 应用配置 (更新设置文件 + 更新配置标记)
// 3. 📚 记录操作历史(带环境变量变化)

#![allow(clippy::unused_async)]

use crate::application::profile_switch::switch_profile_for_platform as run_switch_profile_for_platform;
use crate::managers::settings::SettingsManager;
use crate::models::Platform;
use crate::platforms::create_platform;
use ccr_core::Validatable;
use ccr_core::core::error::{CcrError, Result};
use ccr_core::core::logging::ColorOutput;
use colored::Colorize;
use comfy_table::{
    Attribute, Cell, Color as TableColor, ContentArrangement, Table, presets::UTF8_FULL,
};

/// 🔄 切换到指定配置
///
/// 这是一个原子性操作,确保配置切换的完整性和可追溯性
pub async fn switch_command(config_name: &str) -> Result<()> {
    // 🔍 加载平台配置
    Err(crate::commands::migration::legacy_switch_error(config_name))
}

/// 🔄 在指定平台内切换到配置
///
/// 用于需要固定平台上下文的调用方（如 Web API）。
pub async fn switch_command_for_platform(config_name: &str, platform_name: &str) -> Result<()> {
    ColorOutput::title(&format!("切换配置: {}", config_name));
    println!();

    // 📖 步骤 1: 读取并校验目标配置（由统一用例执行）
    ColorOutput::step("步骤 1/3: 读取配置文件");
    ColorOutput::info(&format!("使用平台: {}", platform_name.bright_yellow()));
    let result = run_switch_profile_for_platform(config_name, platform_name).await?;
    let platform = result.platform;
    let target_section = result.target_section;
    let old_env = result.old_env;
    let new_env_display = result.new_env;
    let old_current = result.previous_profile.unwrap_or_default();

    // ✏️ 步骤 2: 应用配置（已在用例中执行）
    ColorOutput::step("步骤 2/3: 应用配置");
    ColorOutput::success(&format!(
        "✅ 平台 {} 的当前配置已设置为: {}",
        result.platform_name, result.current_profile
    ));

    println!();

    // 📚 步骤 3: 记录历史（已在用例中执行）
    ColorOutput::step("步骤 3/3: 记录操作历史");
    ColorOutput::success("✅ 操作历史已记录");
    println!();

    let platform_config = create_platform(platform)
        .map_err(|e| CcrError::ConfigError(format!("创建平台 {} 失败: {}", platform_name, e)))?;

    // 📋 输出新配置细节与校验结果
    ColorOutput::separator();
    println!();
    ColorOutput::title("🎉 配置切换成功");
    println!();

    // === 配置详情表格 ===
    let mut config_table = Table::new();
    config_table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::DynamicFullWidth)
        .set_header(vec![
            Cell::new("属性")
                .add_attribute(Attribute::Bold)
                .fg(TableColor::Cyan),
            Cell::new("新配置")
                .add_attribute(Attribute::Bold)
                .fg(TableColor::Cyan),
        ]);

    // 配置名称
    config_table.add_row(vec![
        Cell::new("配置名称").fg(TableColor::Yellow),
        Cell::new(config_name)
            .fg(TableColor::Green)
            .add_attribute(Attribute::Bold),
    ]);

    // 描述
    config_table.add_row(vec![
        Cell::new("描述"),
        Cell::new(target_section.display_description()),
    ]);

    // 提供商类型（如果有）
    if let Some(provider_type) = target_section.provider_type.as_ref() {
        let type_display = match provider_type.to_string_value() {
            "official_relay" => "🔄 官方中转",
            "third_party_model" => "🤖 第三方模型",
            _ => provider_type.to_string_value(),
        };
        config_table.add_row(vec![
            Cell::new("提供商类型"),
            Cell::new(type_display).fg(TableColor::Cyan),
        ]);
    }

    // 提供商（如果有）
    if let Some(provider) = &target_section.provider {
        config_table.add_row(vec![
            Cell::new("提供商"),
            Cell::new(provider).fg(TableColor::Cyan),
        ]);
    }

    if platform == Platform::Claude {
        let auth_mode = target_section
            .other
            .get("auth_mode")
            .and_then(|value| value.as_str())
            .unwrap_or_else(|| {
                if target_section.base_url.is_some() || target_section.auth_token.is_some() {
                    "api_key"
                } else {
                    "subscription"
                }
            });
        let auth_source = if auth_mode == "subscription" {
            "subscription".to_string()
        } else {
            target_section
                .provider
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(|provider| format!("provider:{provider}"))
                .unwrap_or_else(|| "settings:anthropic_env".to_string())
        };

        config_table.add_row(vec![
            Cell::new("认证模式"),
            Cell::new(auth_mode).fg(TableColor::Cyan),
        ]);
        config_table.add_row(vec![
            Cell::new("认证来源"),
            Cell::new(auth_source).fg(TableColor::Cyan),
        ]);
    }

    // Base URL
    if let Some(base_url) = &target_section.base_url {
        config_table.add_row(vec![
            Cell::new("Base URL")
                .fg(TableColor::Yellow)
                .add_attribute(Attribute::Bold),
            Cell::new(base_url).fg(TableColor::Blue),
        ]);
    }

    // Auth Token (脱敏)
    if let Some(auth_token) = &target_section.auth_token {
        config_table.add_row(vec![
            Cell::new("Auth Token")
                .fg(TableColor::Yellow)
                .add_attribute(Attribute::Bold),
            Cell::new(ColorOutput::mask_sensitive(auth_token)).fg(TableColor::DarkGrey),
        ]);
    }

    // Model
    if let Some(model) = &target_section.model {
        config_table.add_row(vec![
            Cell::new("主模型"),
            Cell::new(model).fg(TableColor::Magenta),
        ]);
    }

    // Small Fast Model
    if let Some(small_model) = &target_section.small_fast_model {
        config_table.add_row(vec![
            Cell::new("快速小模型"),
            Cell::new(small_model).fg(TableColor::Magenta),
        ]);
    }

    // 账号（如果有）
    if let Some(account) = &target_section.account {
        config_table.add_row(vec![
            Cell::new("账号标识"),
            Cell::new(format!("👤 {}", account)).fg(TableColor::Yellow),
        ]);
    }

    // 标签（如果有）
    if let Some(tags) = &target_section.tags
        && !tags.is_empty()
    {
        config_table.add_row(vec![
            Cell::new("标签"),
            Cell::new(format!("🏷️  {}", tags.join(", "))).fg(TableColor::Magenta),
        ]);
    }

    println!("{}", config_table);
    println!();

    // === 环境变量变化对比表格 ===
    // 显示环境变量变化（动态获取平台环境变量）
    let env_vars = platform_config.get_env_var_names();

    if !env_vars.is_empty() {
        ColorOutput::step("🔄 环境变量变化");
        println!();

        let mut env_changes_table = Table::new();
        env_changes_table
            .load_preset(UTF8_FULL)
            .set_content_arrangement(ContentArrangement::DynamicFullWidth)
            .set_header(vec![
                Cell::new("环境变量")
                    .add_attribute(Attribute::Bold)
                    .fg(TableColor::Cyan),
                Cell::new("变化")
                    .add_attribute(Attribute::Bold)
                    .fg(TableColor::Cyan),
            ]);

        for var_name in &env_vars {
            let old_val = old_env.get(var_name.as_str()).and_then(|v| v.as_ref());
            let new_val = new_env_display
                .get(var_name.as_str())
                .and_then(|v| v.as_ref());

            let is_sensitive = var_name.contains("TOKEN") || var_name.contains("KEY");

            let change_display = match (old_val, new_val) {
                (None, None) => "-".to_string(),
                (None, Some(new)) => {
                    let new_display = if is_sensitive {
                        ColorOutput::mask_sensitive(new)
                    } else if new.len() > 35 {
                        format!("{}...", &new[..32])
                    } else {
                        new.to_string()
                    };
                    format!("➕ 新增: {}", new_display)
                }
                (Some(old), None) => {
                    let old_display = if is_sensitive {
                        ColorOutput::mask_sensitive(old)
                    } else if old.len() > 35 {
                        format!("{}...", &old[..32])
                    } else {
                        old.to_string()
                    };
                    format!("➖ 删除: {}", old_display)
                }
                (Some(old), Some(new)) => {
                    if old == new {
                        "○ 未变化".to_string()
                    } else {
                        let old_display = if is_sensitive {
                            ColorOutput::mask_sensitive(old)
                        } else if old.len() > 20 {
                            format!("{}...", &old[..17])
                        } else {
                            old.to_string()
                        };
                        let new_display = if is_sensitive {
                            ColorOutput::mask_sensitive(new)
                        } else if new.len() > 20 {
                            format!("{}...", &new[..17])
                        } else {
                            new.to_string()
                        };
                        format!("🔄 {} → {}", old_display, new_display)
                    }
                }
            };

            let change_cell = if change_display.starts_with("➕") {
                Cell::new(change_display).fg(TableColor::Green)
            } else if change_display.starts_with("➖") {
                Cell::new(change_display).fg(TableColor::Red)
            } else if change_display.starts_with("🔄") {
                Cell::new(change_display).fg(TableColor::Yellow)
            } else {
                Cell::new(change_display).fg(TableColor::DarkGrey)
            };

            env_changes_table.add_row(vec![Cell::new(var_name.as_str()), change_cell]);
        }

        println!("{}", env_changes_table);
        println!();
    }

    // 最终验证（仅 Claude 平台）
    if platform == Platform::Claude {
        let settings_manager = SettingsManager::with_default()?;
        if let Ok(settings) = settings_manager.load() {
            match settings.validate() {
                Ok(_) => {
                    ColorOutput::success("✓ 配置已生效,Claude Code 可以使用新的 API 配置");
                }
                Err(e) => {
                    ColorOutput::warning(&format!("⚠ 配置可能不完整: {}", e));
                }
            }
        }
    } else {
        ColorOutput::success(&format!("✓ 平台 {} 配置已生效", platform_name));
    }

    println!();
    ColorOutput::separator();
    println!();
    ColorOutput::info(&format!(
        "💡 提示: 从 {} {} 切换到 {} {}",
        old_current.dimmed(),
        "→".dimmed(),
        config_name.bright_green().bold(),
        "✓".bright_green()
    ));

    let restart_hint = match platform {
        Platform::Claude => "建议重启 Claude Code 以确保配置完全生效",
        Platform::Codex => "建议重启 Codex CLI 以确保配置完全生效",
        Platform::Gemini => "建议重启 Gemini CLI 以确保配置完全生效",
        _ => "建议重启对应 CLI 以确保配置完全生效",
    };
    ColorOutput::info(&format!("🔄 {}", restart_hint));

    Ok(())
}
