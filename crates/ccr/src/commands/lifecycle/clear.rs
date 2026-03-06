// 🧹 clear 命令实现 - 清理 ccr 写入的配置
// 💎 用于清空 settings.json 中的 ANTHROPIC_* 环境变量，使其恢复默认状态
//
// 执行流程:
// 1. 加载当前 settings.json
// 2. 清空所有 ANTHROPIC_* 前缀的环境变量
// 3. 备份并保存更新后的设置

#![allow(clippy::unused_async)]

use crate::core::error::{CcrError, Result};
use crate::core::logging::ColorOutput;
use crate::managers::SettingsManager;
use crate::services::ConfigService;
use comfy_table::{
    Attribute, Cell, Color as TableColor, ContentArrangement, Table, presets::UTF8_FULL,
};

/// 🧹 清理 ccr 写入的配置
///
/// 执行流程:
/// 1. 📖 加载当前 settings.json
/// 2. 📊 显示将被清除的环境变量
/// 3. ❓ 确认执行（除非 --force 或 YOLO 模式）
/// 4. 💾 备份当前设置
/// 5. 🧹 清空所有 ANTHROPIC_* 环境变量
/// 6. 💾 保存更新后的设置
///
/// 参数:
/// - force: 跳过确认提示（危险操作）
pub async fn clear_command(force: bool) -> Result<()> {
    ColorOutput::title("清理 CCR 配置");
    println!();

    // ⚡ 检查自动确认模式：--force 参数 OR 配置文件中的 skip_confirmation
    let config_service = ConfigService::with_default()?;
    let config = config_service.load_config()?;
    let skip_confirmation = force || config.settings.skip_confirmation;

    if config.settings.skip_confirmation && !force {
        ColorOutput::info("⚡ 自动确认模式已启用，将跳过确认");
    }

    // 📖 加载设置文件
    let settings_manager = SettingsManager::with_default()?;
    let current_settings = settings_manager.load_async().await?;

    // 📊 收集将被清除的环境变量
    let anthropic_vars: Vec<(String, String)> = current_settings
        .env
        .iter()
        .filter(|(k, _)| k.starts_with("ANTHROPIC_"))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    if anthropic_vars.is_empty() {
        ColorOutput::success("✅ settings.json 中没有 ANTHROPIC_* 环境变量，无需清理");
        return Ok(());
    }

    // 📊 显示将被清除的变量
    ColorOutput::info(&format!(
        "📋 将清除 {} 个 ANTHROPIC_* 环境变量:",
        anthropic_vars.len()
    ));
    println!();

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::DynamicFullWidth)
        .set_header(vec![
            Cell::new("环境变量")
                .add_attribute(Attribute::Bold)
                .fg(TableColor::Cyan),
            Cell::new("当前值")
                .add_attribute(Attribute::Bold)
                .fg(TableColor::Cyan),
        ]);

    for (key, value) in &anthropic_vars {
        let masked_value = if key.contains("TOKEN") || key.contains("KEY") {
            crate::utils::mask_sensitive(value)
        } else {
            value.clone()
        };

        table.add_row(vec![
            Cell::new(key).fg(TableColor::Yellow),
            Cell::new(masked_value).fg(TableColor::DarkGrey),
        ]);
    }
    println!("{}", table);

    // 🚨 确认执行（除非 YOLO 模式）
    if !skip_confirmation {
        println!();
        ColorOutput::warning("⚠️  警告: 此操作将清空 settings.json 中的所有 ANTHROPIC_* 配置！");
        ColorOutput::info(
            "💡 提示: Claude Code 将无法正常工作，直到您重新执行 ccr switch 切换配置",
        );
        println!();

        let confirmed = tokio::task::spawn_blocking(|| -> std::io::Result<bool> {
            use std::io::{self, Write};
            print!("确认执行清理操作? (y/N): ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            Ok(input.trim().eq_ignore_ascii_case("y"))
        })
        .await
        .map_err(|e| CcrError::FileIoError(format!("读取确认输入失败: {}", e)))??;

        if !confirmed {
            ColorOutput::info("已取消清理操作");
            return Ok(());
        }
    }

    println!();
    ColorOutput::separator();
    println!();

    // 💾 备份当前设置
    ColorOutput::step("备份当前设置...");
    let backup_path = settings_manager.backup_async(Some("pre_clear")).await?;
    ColorOutput::success(&format!("✅ 已备份到: {}", backup_path.display()));

    // 🧹 清空 ANTHROPIC_* 环境变量
    ColorOutput::step("清空 ANTHROPIC_* 环境变量...");
    let mut updated_settings = current_settings;
    updated_settings.clear_anthropic_vars();

    // 💾 保存更新后的设置
    ColorOutput::step("保存更新后的设置...");
    settings_manager
        .save_atomic_async(&updated_settings)
        .await?;

    println!();
    ColorOutput::separator();
    println!();

    // 📊 显示结果
    ColorOutput::title("清理完成");
    println!();
    ColorOutput::success(&format!("✅ 已清除 {} 个环境变量", anthropic_vars.len()));
    ColorOutput::info(&format!(
        "📁 settings.json: {}",
        settings_manager.settings_path().display()
    ));

    println!();
    ColorOutput::info("💡 提示:");
    ColorOutput::info("   • 使用 'ccr switch <配置名>' 重新应用配置");
    ColorOutput::info("   • 使用 'ccr list' 查看可用配置");
    ColorOutput::info(&format!(
        "   • 如需恢复，可使用备份文件: {}",
        backup_path.display()
    ));

    Ok(())
}
