// 🔍 current 命令实现 - 显示当前配置状态
// 📊 显示当前激活的配置详情和 Claude Code 环境变量状态

use crate::core::error::Result;
use crate::core::logging::ColorOutput;
use crate::services::{ConfigService, SettingsService};
use crate::utils::Validatable;
use colored::Colorize;
use comfy_table::{
    Attribute, Cell, Color as TableColor, ContentArrangement, Table, presets::UTF8_FULL,
};

/// 🔍 显示当前配置状态
///
/// 显示内容分为两部分:
/// 1. 📝 配置文件信息
///    - 当前配置名称
///    - 配置详情(描述、URL、Token、模型等)
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
    let config = config_service.load_config()?;

    println!();
    ColorOutput::info(&format!(
        "配置文件: {}",
        config_service.config_manager().config_path().display()
    ));
    ColorOutput::info(&format!(
        "默认配置: {}",
        config.default_config.bright_yellow()
    ));
    println!();

    // === 第一部分：配置详情表格 ===
    ColorOutput::step("📋 配置详情");
    println!();

    let mut config_table = Table::new();
    config_table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("属性")
                .add_attribute(Attribute::Bold)
                .fg(TableColor::Cyan),
            Cell::new("值")
                .add_attribute(Attribute::Bold)
                .fg(TableColor::Cyan),
        ]);

    // 配置名称
    config_table.add_row(vec![
        Cell::new("配置名称").fg(TableColor::Yellow),
        Cell::new(&current_info.name)
            .fg(TableColor::Green)
            .add_attribute(Attribute::Bold),
    ]);

    // 描述
    config_table.add_row(vec![
        Cell::new("描述"),
        Cell::new(&current_info.description),
    ]);

    // 提供商类型
    if let Some(provider_type) = &current_info.provider_type {
        let type_display = match provider_type.as_str() {
            "official_relay" => "🔄 官方中转",
            "third_party_model" => "🤖 第三方模型",
            _ => provider_type.as_str(),
        };
        config_table.add_row(vec![
            Cell::new("提供商类型").fg(TableColor::Yellow),
            Cell::new(type_display).fg(TableColor::Cyan),
        ]);
    }

    // 提供商
    if let Some(provider) = &current_info.provider {
        config_table.add_row(vec![
            Cell::new("提供商").fg(TableColor::Yellow),
            Cell::new(provider).fg(TableColor::Cyan),
        ]);
    }

    // Base URL
    if let Some(base_url) = &current_info.base_url {
        config_table.add_row(vec![
            Cell::new("Base URL")
                .fg(TableColor::Yellow)
                .add_attribute(Attribute::Bold),
            Cell::new(base_url).fg(TableColor::Blue),
        ]);
    }

    // Auth Token (脱敏)
    if let Some(auth_token) = &current_info.auth_token {
        config_table.add_row(vec![
            Cell::new("Auth Token")
                .fg(TableColor::Yellow)
                .add_attribute(Attribute::Bold),
            Cell::new(ColorOutput::mask_sensitive(auth_token)).fg(TableColor::DarkGrey),
        ]);
    }

    // Model
    if let Some(model) = &current_info.model {
        config_table.add_row(vec![
            Cell::new("主模型"),
            Cell::new(model).fg(TableColor::Magenta),
        ]);
    }

    // Small Fast Model
    if let Some(small_model) = &current_info.small_fast_model {
        config_table.add_row(vec![
            Cell::new("快速小模型"),
            Cell::new(small_model).fg(TableColor::Magenta),
        ]);
    }

    // 账号
    if let Some(account) = &current_info.account {
        config_table.add_row(vec![
            Cell::new("账号标识"),
            Cell::new(format!("👤 {}", account)).fg(TableColor::Yellow),
        ]);
    }

    // 标签
    if let Some(tags) = &current_info.tags {
        if !tags.is_empty() {
            config_table.add_row(vec![
                Cell::new("标签"),
                Cell::new(format!("🏷️  {}", tags.join(", "))).fg(TableColor::Magenta),
            ]);
        }
    }

    // 验证状态
    let section = config.get_current_section()?;
    let validation_status = match section.validate() {
        Ok(_) => Cell::new("✓ 配置完整")
            .fg(TableColor::Green)
            .add_attribute(Attribute::Bold),
        Err(e) => Cell::new(format!("✗ 配置不完整: {}", e))
            .fg(TableColor::Red)
            .add_attribute(Attribute::Bold),
    };
    config_table.add_row(vec![
        Cell::new("验证状态").fg(TableColor::Yellow),
        validation_status,
    ]);

    println!("{}", config_table);
    println!();

    // === 第二部分：Claude Code 环境变量表格 ===
    ColorOutput::separator();
    println!();
    ColorOutput::step("🌍 Claude Code 环境变量状态");
    println!();

    match SettingsService::default() {
        Ok(settings_service) => {
            match settings_service.get_current_settings() {
                Ok(settings) => {
                    let mut env_table = Table::new();
                    env_table
                        .load_preset(UTF8_FULL)
                        .set_content_arrangement(ContentArrangement::Dynamic)
                        .set_header(vec![
                            Cell::new("环境变量")
                                .add_attribute(Attribute::Bold)
                                .fg(TableColor::Cyan),
                            Cell::new("当前值")
                                .add_attribute(Attribute::Bold)
                                .fg(TableColor::Cyan),
                            Cell::new("状态")
                                .add_attribute(Attribute::Bold)
                                .fg(TableColor::Cyan),
                        ]);

                    let env_status = settings.anthropic_env_status();
                    let env_vars = [
                        ("ANTHROPIC_BASE_URL", true),
                        ("ANTHROPIC_AUTH_TOKEN", true),
                        ("ANTHROPIC_MODEL", false),
                        ("ANTHROPIC_SMALL_FAST_MODEL", false),
                    ];

                    for (var_name, is_required) in env_vars {
                        let value = env_status.get(var_name).and_then(|v| v.as_ref());

                        let var_cell = if is_required {
                            Cell::new(format!("{} *", var_name)).fg(TableColor::Yellow)
                        } else {
                            Cell::new(var_name)
                        };

                        let (value_cell, status_cell) = match value {
                            Some(v) => {
                                let is_sensitive =
                                    var_name.contains("TOKEN") || var_name.contains("KEY");
                                let display_value = if is_sensitive {
                                    ColorOutput::mask_sensitive(v)
                                } else {
                                    if v.len() > 40 {
                                        format!("{}...", &v[..37])
                                    } else {
                                        v.to_string()
                                    }
                                };
                                (
                                    Cell::new(display_value).fg(TableColor::Blue),
                                    Cell::new("✓")
                                        .fg(TableColor::Green)
                                        .add_attribute(Attribute::Bold),
                                )
                            }
                            None => {
                                if is_required {
                                    (
                                        Cell::new("(未设置)").fg(TableColor::Red),
                                        Cell::new("✗")
                                            .fg(TableColor::Red)
                                            .add_attribute(Attribute::Bold),
                                    )
                                } else {
                                    (
                                        Cell::new("(未设置)").fg(TableColor::DarkGrey),
                                        Cell::new("○").fg(TableColor::DarkGrey),
                                    )
                                }
                            }
                        };

                        env_table.add_row(vec![var_cell, value_cell, status_cell]);
                    }

                    println!("{}", env_table);
                    println!();

                    // 验证设置
                    match settings.validate() {
                        Ok(_) => ColorOutput::success("✓ Claude Code 设置验证通过"),
                        Err(e) => ColorOutput::warning(&format!("⚠ 设置验证警告: {}", e)),
                    }

                    println!();
                    ColorOutput::info("提示: * 标记的为必需环境变量");
                }
                Err(e) => {
                    ColorOutput::warning(&format!("无法加载 Claude Code 设置: {}", e));
                    ColorOutput::info(
                        "提示: 可能是首次使用,运行 'ccr switch <config>' 来初始化设置",
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
