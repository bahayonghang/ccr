//! 📋 claude auth list 命令实现

#![allow(clippy::unused_async)]

use crate::services::ClaudeAuthService;
use ccr_core::core::error::Result;
use ccr_core::core::logging::ColorOutput;
use chrono::Local;
use comfy_table::{
    Attribute, Cell, Color as TableColor, ContentArrangement, Table, presets::UTF8_FULL,
};

pub async fn list_command() -> Result<()> {
    let service = ClaudeAuthService::new()?;
    let snapshot = service.read_auth_snapshot()?;
    let accounts = service.build_account_items(&snapshot)?;

    println!();
    ColorOutput::title("Claude 官方账号列表");
    println!();

    if accounts.is_empty() {
        ColorOutput::info("尚未保存任何官方账号快照");
        if snapshot.current_info.is_some() {
            println!();
            ColorOutput::info("当前存在官方登录，可执行:");
            println!("  ccr claude auth save <名称>");
        } else {
            println!();
            ColorOutput::info("请先运行 `claude login`，然后再保存账号快照");
        }
        return Ok(());
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::DynamicFullWidth)
        .set_header(vec![
            Cell::new("状态")
                .add_attribute(Attribute::Bold)
                .fg(TableColor::Cyan),
            Cell::new("名称")
                .add_attribute(Attribute::Bold)
                .fg(TableColor::Cyan),
            Cell::new("邮箱")
                .add_attribute(Attribute::Bold)
                .fg(TableColor::Cyan),
            Cell::new("订阅")
                .add_attribute(Attribute::Bold)
                .fg(TableColor::Cyan),
            Cell::new("到期")
                .add_attribute(Attribute::Bold)
                .fg(TableColor::Cyan),
            Cell::new("描述")
                .add_attribute(Attribute::Bold)
                .fg(TableColor::Cyan),
        ]);

    for account in &accounts {
        let status = if account.is_current { ">> 当前" } else { "" };
        let expires_at = account
            .expires_at
            .map(|dt| {
                dt.with_timezone(&Local)
                    .format("%Y-%m-%d %H:%M")
                    .to_string()
            })
            .unwrap_or_else(|| "-".to_string());
        let expires_at = if ClaudeAuthService::is_expired(account.expires_at) {
            format!("🔒 {expires_at}")
        } else {
            expires_at
        };

        table.add_row(vec![
            Cell::new(status).fg(if account.is_current {
                TableColor::Green
            } else {
                TableColor::White
            }),
            Cell::new(&account.name).fg(if account.is_current {
                TableColor::Green
            } else {
                TableColor::White
            }),
            Cell::new(account.email.as_deref().unwrap_or("-")),
            Cell::new(account.subscription_type.as_deref().unwrap_or("-")),
            Cell::new(expires_at).fg(if ClaudeAuthService::is_expired(account.expires_at) {
                TableColor::Red
            } else {
                TableColor::White
            }),
            Cell::new(account.description.as_deref().unwrap_or("-")).fg(TableColor::Blue),
        ]);
    }

    println!("{}", table);
    println!();

    if snapshot.current_info.is_some() {
        ColorOutput::info("提示:");
        println!("  • 使用 'ccr claude auth current' 查看当前官方登录详情");
        println!("  • 使用 'ccr claude auth switch <名称>' 切换官方账号");
        println!("  • 使用 'ccr claude auth delete <名称>' 删除账号快照");
    } else {
        ColorOutput::warning(
            "当前未检测到可用的官方登录，切换后请确认 Claude Code 能正常刷新/续期",
        );
    }

    Ok(())
}
