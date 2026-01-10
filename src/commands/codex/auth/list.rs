//! 📋 codex auth list 命令实现
//!
//! 列出所有已保存的账号。

#![allow(clippy::unused_async)]

use crate::core::error::Result;
use crate::core::logging::ColorOutput;
use crate::models::TokenFreshness;
use crate::services::CodexAuthService;
use comfy_table::{
    Attribute, Cell, CellAlignment, Color as TableColor, ContentArrangement, Table,
    presets::UTF8_FULL,
};

/// 📋 列出所有已保存的账号
///
/// 显示所有已保存的 Codex 账号，包括当前登录状态。
///
/// # 返回
///
/// * `Ok(())` - 成功执行
/// * `Err(CcrError)` - 执行失败
pub async fn list_command() -> Result<()> {
    let service = CodexAuthService::new()?;

    // 检查登录状态
    if !service.is_logged_in() {
        ColorOutput::warning("未登录 Codex");
        println!();
        ColorOutput::info("请先运行以下命令登录:");
        println!("  codex login");
        println!();
        ColorOutput::info("登录后可以使用以下命令保存账号:");
        println!("  ccr codex auth save <名称>");
        return Ok(());
    }

    // 获取账号列表
    let accounts = service.list_accounts()?;

    if accounts.is_empty() {
        ColorOutput::info("没有已保存的账号");
        println!();
        ColorOutput::info("使用以下命令保存当前登录:");
        println!("  ccr codex auth save <名称>");
        return Ok(());
    }

    // 显示标题
    println!();
    ColorOutput::title("Codex 账号列表");
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
            Cell::new("名称")
                .add_attribute(Attribute::Bold)
                .fg(TableColor::Cyan),
            Cell::new("邮箱")
                .add_attribute(Attribute::Bold)
                .fg(TableColor::Cyan),
            Cell::new("新鲜度")
                .add_attribute(Attribute::Bold)
                .fg(TableColor::Cyan),
            Cell::new("描述")
                .add_attribute(Attribute::Bold)
                .fg(TableColor::Cyan),
        ]);

    for account in &accounts {
        // 状态列
        let status = if account.is_current {
            Cell::new(">> 当前")
                .fg(TableColor::Green)
                .add_attribute(Attribute::Bold)
        } else {
            Cell::new("")
        };

        // 名称列
        let name_cell = if account.is_virtual {
            Cell::new(format!("{} *", account.name))
                .fg(TableColor::Yellow)
                .add_attribute(Attribute::Italic)
        } else if account.is_current {
            Cell::new(&account.name)
                .fg(TableColor::Green)
                .add_attribute(Attribute::Bold)
        } else {
            Cell::new(&account.name)
        };

        // 邮箱列
        let email = account.email.as_deref().unwrap_or("-");
        let email_cell = Cell::new(email);

        // 新鲜度列
        let freshness_cell = match account.freshness {
            TokenFreshness::Fresh => Cell::new("🟢 新鲜").fg(TableColor::Green),
            TokenFreshness::Stale => Cell::new("🟡 陈旧").fg(TableColor::Yellow),
            TokenFreshness::Old => Cell::new("🔴 过期").fg(TableColor::Red),
            TokenFreshness::Unknown => Cell::new("⚪ 未知").fg(TableColor::White),
        };

        // 描述列
        let description = account.description.as_deref().unwrap_or("-");
        let desc_cell = Cell::new(description).fg(TableColor::Blue);

        table.add_row(vec![
            status,
            name_cell,
            email_cell,
            freshness_cell,
            desc_cell,
        ]);
    }

    // 设置列对齐
    if let Some(column) = table.column_mut(0) {
        column.set_cell_alignment(CellAlignment::Left);
    }
    if let Some(column) = table.column_mut(3) {
        column.set_cell_alignment(CellAlignment::Center);
    }

    println!("{}", table);
    println!();

    // 统计信息
    let saved_count = accounts.iter().filter(|a| !a.is_virtual).count();
    let virtual_count = accounts.iter().filter(|a| a.is_virtual).count();

    if virtual_count > 0 {
        ColorOutput::info(&format!(
            "共 {} 个已保存账号，{} 个未保存的当前登录",
            saved_count, virtual_count
        ));
        println!();
        ColorOutput::warning("* 标记的账号为未保存的当前登录，使用以下命令保存:");
        println!("  ccr codex auth save <名称>");
    } else {
        ColorOutput::success(&format!("共 {} 个已保存账号", saved_count));
    }

    println!();
    ColorOutput::info("提示:");
    println!("  • 使用 'ccr codex auth switch <名称>' 切换账号");
    println!("  • 使用 'ccr codex auth current' 查看当前账号详情");
    println!("  • 使用 'ccr codex auth delete <名称>' 删除账号");

    Ok(())
}
