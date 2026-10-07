//! 📋 claude auth list 命令实现

#![allow(clippy::unused_async)]

use crate::commands::common::new_utf8_table;
use crate::commands::common::print_next_steps;
use crate::services::ClaudeAuthService;
use ccr_core::core::error::Result;
use ccr_core::core::logging::ColorOutput;
use chrono::Local;
use comfy_table::{Attribute, Cell, Color as TableColor, ContentArrangement};

pub async fn list_command() -> Result<()> {
    let service = ClaudeAuthService::new()?;
    let snapshot = service.read_auth_snapshot()?;
    let runtime_summary = service.get_runtime_summary().ok();
    let accounts = service.build_account_items(&snapshot, runtime_summary.as_ref())?;

    ColorOutput::title("Claude 官方账号列表");
    println!();

    if accounts.is_empty() {
        ColorOutput::info("尚未保存任何官方账号快照");
        if snapshot.current_info.is_some() {
            print_next_steps(&[("查看保存帮助", "ccr claude auth save --help")]);
        } else {
            print_next_steps(&[("登录 Claude", "claude login")]);
        }
        return Ok(());
    }

    let mut table = new_utf8_table();
    table
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
        let status = if account.is_current {
            ">> 生效"
        } else if account.is_logged_in {
            "↪ 已登录"
        } else {
            ""
        };
        let expires_at = account
            .expires_at
            .map(|dt| {
                dt.with_timezone(&Local)
                    .format("%Y-%m-%d %H:%M")
                    .to_string()
            })
            .unwrap_or_else(|| "-".to_string());

        table.add_row(vec![
            Cell::new(status).fg(if account.is_current {
                TableColor::Green
            } else if account.is_logged_in {
                TableColor::Cyan
            } else {
                TableColor::White
            }),
            Cell::new(&account.name).fg(if account.is_current {
                TableColor::Green
            } else if account.is_logged_in {
                TableColor::Cyan
            } else {
                TableColor::White
            }),
            Cell::new(account.email.as_deref().unwrap_or("-")),
            Cell::new(account.subscription_type.as_deref().unwrap_or("-")),
            Cell::new(expires_at),
            Cell::new(account.description.as_deref().unwrap_or("-")).fg(TableColor::Blue),
        ]);
    }

    println!("{}", table);
    println!();

    if snapshot.current_info.is_some() {
        if let Some(summary) = runtime_summary {
            ColorOutput::key_value("当前 Profile", &summary.profile_label(), 2);
            ColorOutput::key_value("当前官方登录", &summary.official_login_label(), 2);
            ColorOutput::key_value("当前生效认证", &summary.auth_label(), 2);
        }
        print_next_steps(&[
            ("查看当前认证", "ccr claude auth current"),
            ("查看切换帮助", "ccr claude auth switch --help"),
        ]);
    } else {
        ColorOutput::warning(
            "当前未检测到可用的官方登录，切换后请确认 Claude Code 能正常刷新/续期",
        );
    }

    Ok(())
}
