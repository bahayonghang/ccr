// 📜 list 命令实现 - 列出所有可用配置
// 📋 显示所有配置节,突出显示当前配置和默认配置
// 🔄 支持平台感知: 检测并显示当前平台信息(unified 模式)

use crate::core::error::Result;
use crate::core::logging::ColorOutput;
use crate::managers::PlatformConfigManager;
use crate::services::ConfigService;
use crate::utils::Validatable;
use colored::Colorize;
use comfy_table::{
    Attribute, Cell, Color as TableColor, ContentArrangement, Table, presets::UTF8_FULL,
};

/// 📜 列出所有可用配置
///
/// 显示内容:
/// - 🔄 配置模式 (Legacy / Unified)
/// - 🎯 当前平台 (unified 模式下)
/// - ⚙️ 配置文件路径
/// - 🎯 默认配置和当前配置
/// - 📋 所有配置节列表(带验证状态)
/// - ▶️ 使用表格形式突出显示关键信息
pub fn list_command() -> Result<()> {
    ColorOutput::title("可用配置列表");

    // 🔍 检测配置模式
    let unified_config = PlatformConfigManager::with_default()
        .ok()
        .and_then(|mgr| mgr.load().ok());
    let is_unified_mode = unified_config.is_some();

    println!();

    // 显示配置模式和平台信息
    if is_unified_mode {
        if let Some(ref uc) = unified_config {
            ColorOutput::info(&format!(
                "配置模式: {} (多平台支持)",
                "Unified".bright_cyan().bold()
            ));
            ColorOutput::info(&format!(
                "当前平台: {}",
                uc.current_platform.bright_yellow().bold()
            ));
        }
    } else {
        ColorOutput::info(&format!("配置模式: {} (传统模式)", "Legacy".bright_white()));
    }

    // 使用 ConfigService
    let service = ConfigService::with_default()?;
    let list = service.list_configs()?;
    let config = service.load_config()?;

    println!();
    ColorOutput::info(&format!(
        "配置文件: {}",
        service.config_manager().config_path().display()
    ));
    ColorOutput::info(&format!(
        "默认配置: {}",
        list.default_config.bright_yellow()
    ));
    ColorOutput::info(&format!(
        "当前配置: {}",
        list.current_config.bright_green().bold()
    ));
    println!();

    // 列出所有配置节
    if list.configs.is_empty() {
        ColorOutput::warning("未找到任何配置节");
        return Ok(());
    }

    // 创建表格
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("状态")
                .add_attribute(Attribute::Bold)
                .fg(TableColor::Cyan),
            Cell::new("配置名称")
                .add_attribute(Attribute::Bold)
                .fg(TableColor::Cyan),
            Cell::new("提供商")
                .add_attribute(Attribute::Bold)
                .fg(TableColor::Cyan),
            Cell::new("Base URL")
                .add_attribute(Attribute::Bold)
                .fg(TableColor::Cyan),
            Cell::new("模型")
                .add_attribute(Attribute::Bold)
                .fg(TableColor::Cyan),
            Cell::new("账号/标签")
                .add_attribute(Attribute::Bold)
                .fg(TableColor::Cyan),
            Cell::new("验证")
                .add_attribute(Attribute::Bold)
                .fg(TableColor::Cyan),
        ]);

    for config_info in &list.configs {
        // 状态列
        let status = if config_info.is_current {
            Cell::new("▶ 当前")
                .fg(TableColor::Green)
                .add_attribute(Attribute::Bold)
        } else if config_info.is_default {
            Cell::new("⭐ 默认").fg(TableColor::Yellow)
        } else {
            Cell::new("")
        };

        // 配置名称
        let name_cell = if config_info.is_current {
            Cell::new(&config_info.name)
                .fg(TableColor::Green)
                .add_attribute(Attribute::Bold)
        } else {
            Cell::new(&config_info.name)
        };

        // 提供商信息
        let provider_info = if let Some(provider) = &config_info.provider {
            let type_icon = match config_info.provider_type.as_deref() {
                Some("official_relay") => "🔄",
                Some("third_party_model") => "🤖",
                _ => "❓",
            };
            format!("{} {}", type_icon, provider)
        } else {
            "未分类".to_string()
        };
        let provider_cell = Cell::new(provider_info).fg(TableColor::Cyan);

        // Base URL (缩短显示)
        let base_url = config_info.base_url.as_deref().unwrap_or("N/A");
        let base_url_display = if base_url.len() > 35 {
            format!("{}...", &base_url[..32])
        } else {
            base_url.to_string()
        };
        let base_url_cell = Cell::new(base_url_display).fg(TableColor::Blue);

        // 模型信息
        let model_info = if let Some(model) = &config_info.model {
            let model_short = if model.len() > 25 {
                format!("{}...", &model[..22])
            } else {
                model.clone()
            };
            if let Some(small) = &config_info.small_fast_model {
                format!("{}\n(small: {})", model_short, small)
            } else {
                model_short
            }
        } else {
            "未设置".to_string()
        };

        // 账号/标签
        let mut extra_info = Vec::new();
        if let Some(account) = &config_info.account {
            extra_info.push(format!("👤 {}", account));
        }
        if let Some(tags) = &config_info.tags
            && !tags.is_empty()
        {
            extra_info.push(format!("🏷️  {}", tags.join(", ")));
        }
        let extra_info_str = if extra_info.is_empty() {
            "-".to_string()
        } else {
            extra_info.join("\n")
        };

        // 验证状态
        let section = config.get_section(&config_info.name)?;
        let validation_cell = match section.validate() {
            Ok(_) => Cell::new("✓")
                .fg(TableColor::Green)
                .add_attribute(Attribute::Bold),
            Err(_) => Cell::new("✗")
                .fg(TableColor::Red)
                .add_attribute(Attribute::Bold),
        };

        table.add_row(vec![
            status,
            name_cell,
            provider_cell,
            base_url_cell,
            Cell::new(model_info),
            Cell::new(extra_info_str).fg(TableColor::Yellow),
            validation_cell,
        ]);
    }

    println!("{}", table);
    println!();

    ColorOutput::success(&format!("共找到 {} 个配置", list.configs.len()));
    println!();

    // 根据模式显示不同的提示信息
    if is_unified_mode {
        ColorOutput::info("提示 (Unified 模式):");
        println!("  • 使用 'ccr platform switch <平台>' 切换平台");
        println!("  • 使用 'ccr platform current' 查看当前平台详情");
        println!("  • 使用 'ccr switch <名称>' 切换配置");
        println!("  • 🔄 = 官方中转  🤖 = 第三方模型");
    } else {
        ColorOutput::info("提示 (Legacy 模式):");
        println!("  • 使用 'ccr switch <名称>' 切换配置");
        println!("  • 使用 'ccr current' 查看当前配置详情");
        println!("  • 使用 'ccr migrate' 迁移到多平台模式");
        println!("  • 🔄 = 官方中转  🤖 = 第三方模型");
    }

    Ok(())
}
