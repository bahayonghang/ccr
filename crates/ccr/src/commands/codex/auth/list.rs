//! 📋 codex auth list 命令实现
//!
//! 列出所有已保存的账号。

#![allow(clippy::unused_async)]

use crate::core::error::Result;
use crate::core::logging::ColorOutput;
use crate::models::AuthIntent;
use crate::services::CodexAuthService;
use chrono::Local;
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
    let auth_state = service.get_auth_state();

    // 检查登录状态
    if !service.is_logged_in() {
        ColorOutput::warning("未登录 Codex");
        ColorOutput::info(&format!(
            "认证状态: {} / {}",
            render_intent(&auth_state.intent),
            auth_state.store.as_str()
        ));
        ColorOutput::info(&format!("原因: {}", auth_state.reason));
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
    ColorOutput::info(&format!(
        "当前认证: {} / {}",
        render_intent(&auth_state.intent),
        auth_state.store.as_str()
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
            Cell::new("名称")
                .add_attribute(Attribute::Bold)
                .fg(TableColor::Cyan),
            Cell::new("邮箱")
                .add_attribute(Attribute::Bold)
                .fg(TableColor::Cyan),
            Cell::new("到期")
                .add_attribute(Attribute::Bold)
                .fg(TableColor::Cyan),
            Cell::new("添加日期")
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

        // 到期列
        let (expire_label, expire_color) = match account.expires_at {
            Some(ts) => {
                let expired = CodexAuthService::is_expired(account.expires_at);
                let local_ts = ts
                    .with_timezone(&Local)
                    .format("%Y-%m-%d %H:%M")
                    .to_string();
                if expired {
                    (format!("🔒 {}", local_ts), TableColor::Red)
                } else {
                    (local_ts, TableColor::Green)
                }
            }
            None => ("-".to_string(), TableColor::White),
        };
        let expire_cell = Cell::new(expire_label).fg(expire_color);

        // 添加日期列
        let saved_at = account
            .saved_at
            .map(|ts| ts.with_timezone(&Local).format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| "-".to_string());
        let saved_at_cell = Cell::new(saved_at).fg(TableColor::White);

        // 描述列
        let description = account.description.as_deref().unwrap_or("-");
        let desc_cell = Cell::new(description).fg(TableColor::Blue);

        table.add_row(vec![
            status,
            name_cell,
            email_cell,
            expire_cell,
            saved_at_cell,
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
    if let Some(column) = table.column_mut(4) {
        column.set_cell_alignment(CellAlignment::Center);
    }

    println!("{}", table);
    println!();

    // 统计过期账号
    let expired_count = accounts
        .iter()
        .filter(|a| CodexAuthService::is_expired(a.expires_at))
        .count();
    if expired_count > 0 {
        ColorOutput::warning(&format!(
            "有 {} 个账号已过期，切换将被阻止。",
            expired_count
        ));
    }

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

fn render_intent(intent: &AuthIntent) -> String {
    match intent {
        AuthIntent::OpenAiAuth { method } => match method {
            crate::models::OpenAiAuthMethod::Chatgpt => "OpenAI / ChatGPT".to_string(),
            crate::models::OpenAiAuthMethod::Api => "OpenAI / API Key".to_string(),
        },
        AuthIntent::ProviderEnvKey { env_key } => format!("Provider / {env_key}"),
        AuthIntent::NoAuth => "No Auth".to_string(),
    }
}
