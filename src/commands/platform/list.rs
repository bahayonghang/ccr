//! 📜 platform list 命令实现
//!
//! 列出所有可用平台。

#![allow(clippy::unused_async)]

use super::types::{PlatformListItem, PlatformListOutput};
use crate::core::error::Result;
use crate::core::logging::ColorOutput;
use crate::managers::PlatformConfigManager;
use crate::platforms::PlatformRegistry;
use colored::Colorize;
use comfy_table::{
    Attribute, Cell, CellAlignment, Color as TableColor, ColumnConstraint, ContentArrangement,
    Table, presets::UTF8_FULL,
};

/// 📜 列出所有可用平台
///
/// 显示内容:
/// - 🎯 当前激活的平台
/// - 📋 所有注册的平台列表
/// - 🔌 平台启用状态
/// - ▶️ 当前 profile
/// - 📝 平台描述
///
/// # 参数
///
/// * `json` - 是否以 JSON 格式输出
///
/// # 返回
///
/// * `Ok(())` - 成功执行
/// * `Err(CcrError)` - 配置文件加载失败或其他错误
pub async fn platform_list_command(json: bool) -> Result<()> {
    let manager = PlatformConfigManager::with_default()?;
    let config = manager.load_or_create_default()?;

    // 获取所有支持的平台
    let registry = PlatformRegistry::new();
    let all_platforms = registry.list_platform_info();

    // 🔍 收集平台信息
    let mut platforms_data = Vec::new();

    for platform_info in &all_platforms {
        let platform_name = &platform_info.short_name;
        let registry_entry = config.platforms.get(platform_name);

        let is_current = platform_name == &config.current_platform;
        let is_default = platform_name == &config.default_platform;
        let enabled = registry_entry.map(|e| e.enabled).unwrap_or(false);
        let current_profile = registry_entry.and_then(|e| e.current_profile.clone());
        let description = registry_entry
            .and_then(|e| e.description.clone())
            .unwrap_or_else(|| platform_info.name.to_string());

        platforms_data.push(PlatformListItem {
            name: platform_name.clone(),
            is_current,
            is_default,
            enabled,
            current_profile,
            description,
        });
    }

    // 📤 输出格式选择
    if json {
        // JSON 输出
        let output = PlatformListOutput {
            config_file: manager.config_path().display().to_string(),
            default_platform: config.default_platform.clone(),
            current_platform: config.current_platform.clone(),
            platforms: platforms_data,
        };

        let json_str = serde_json::to_string_pretty(&output)?;
        println!("{}", json_str);

        return Ok(());
    }

    // 📊 表格输出
    ColorOutput::title("平台列表");

    println!();
    ColorOutput::info(&format!("配置文件: {}", manager.config_path().display()));
    ColorOutput::info(&format!(
        "默认平台: {}",
        config.default_platform.bright_yellow()
    ));
    ColorOutput::info(&format!(
        "当前平台: {}",
        config.current_platform.bright_green().bold()
    ));
    println!();

    // 创建表格
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::DynamicFullWidth)
        .set_header(vec![
            Cell::new("状态")
                .add_attribute(Attribute::Bold)
                .fg(TableColor::Cyan),
            Cell::new("平台名称")
                .add_attribute(Attribute::Bold)
                .fg(TableColor::Cyan),
            Cell::new("启用")
                .add_attribute(Attribute::Bold)
                .fg(TableColor::Cyan),
            Cell::new("当前 Profile")
                .add_attribute(Attribute::Bold)
                .fg(TableColor::Cyan),
            Cell::new("描述")
                .add_attribute(Attribute::Bold)
                .fg(TableColor::Cyan),
        ]);

    // 使用已收集的数据填充表格
    for platform in &platforms_data {
        // 状态列
        let status = if platform.is_current {
            Cell::new(">> 当前")
                .fg(TableColor::Green)
                .add_attribute(Attribute::Bold)
        } else if platform.is_default {
            Cell::new("* 默认").fg(TableColor::Yellow)
        } else {
            Cell::new("")
        };

        // 平台名称
        let name_cell = if platform.is_current {
            Cell::new(&platform.name)
                .fg(TableColor::Green)
                .add_attribute(Attribute::Bold)
        } else {
            Cell::new(&platform.name)
        };

        // 启用状态
        let enabled_cell = if platform.enabled {
            Cell::new("OK")
                .fg(TableColor::Green)
                .add_attribute(Attribute::Bold)
        } else {
            Cell::new("X").fg(TableColor::Red)
        };

        // 当前 profile
        let current_profile = platform.current_profile.as_deref().unwrap_or("-");

        table.add_row(vec![
            status,
            name_cell,
            enabled_cell,
            Cell::new(current_profile),
            Cell::new(&platform.description).fg(TableColor::Blue),
        ]);
    }

    // 为"启用"列设置固定宽度和居中对齐
    if let Some(column) = table.column_mut(2) {
        column.set_constraint(ColumnConstraint::ContentWidth);
        column.set_cell_alignment(CellAlignment::Center);
    }

    println!("{}", table);
    println!();

    ColorOutput::success(&format!("共找到 {} 个平台", platforms_data.len()));
    println!();
    ColorOutput::info("提示:");
    println!("  • 使用 'ccr platform switch <平台名>' 切换平台");
    println!("  • 使用 'ccr platform current' 查看当前平台详情");
    println!("  • 使用 'ccr platform info <平台名>' 查看平台信息");

    Ok(())
}
