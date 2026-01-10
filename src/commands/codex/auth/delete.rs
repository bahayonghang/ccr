//! 🗑️ codex auth delete 命令实现
//!
//! 删除指定账号。

#![allow(clippy::unused_async)]

use crate::core::error::Result;
use crate::core::logging::ColorOutput;
use crate::services::CodexAuthService;
use colored::Colorize;
use std::io::{self, Write};

/// 🗑️ 删除指定账号
///
/// 删除已保存的账号（不会影响当前登录状态）。
///
/// # 参数
///
/// * `name` - 要删除的账号名称
/// * `force` - 是否跳过确认提示
///
/// # 返回
///
/// * `Ok(())` - 删除成功
/// * `Err(CcrError)` - 删除失败
pub async fn delete_command(name: &str, force: bool) -> Result<()> {
    let service = CodexAuthService::new()?;

    // 检查账号是否存在
    let accounts = service.list_accounts()?;
    let account = accounts.iter().find(|a| a.name == name && !a.is_virtual);

    if account.is_none() {
        ColorOutput::error(&format!("账号 '{}' 不存在", name));
        println!();
        ColorOutput::info("使用以下命令查看可用账号:");
        println!("  ccr codex auth list");
        return Ok(());
    }

    let account = account.expect("account should exist");

    // 确认删除
    if !force {
        println!();
        ColorOutput::warning(&format!("即将删除账号: {}", name.bright_yellow().bold()));

        if let Some(email) = &account.email {
            ColorOutput::info(&format!("邮箱: {}", email));
        }
        if let Some(desc) = &account.description {
            ColorOutput::info(&format!("描述: {}", desc));
        }

        // 检查是否是当前账号
        if account.is_current {
            println!();
            ColorOutput::warning("注意: 这是当前正在使用的账号！");
            ColorOutput::info("删除后当前登录状态不会受影响，但无法再切换回此账号");
        }

        println!();
        print!("确认删除? (输入 'yes' 确认): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if input.trim().to_lowercase() != "yes" {
            ColorOutput::info("已取消删除");
            return Ok(());
        }
    }

    // 执行删除
    match service.delete_account(name) {
        Ok(()) => {
            println!();
            ColorOutput::success(&format!("已删除账号: {}", name.bright_red().bold()));
            println!();
            ColorOutput::info("提示:");
            println!("  • 使用 'ccr codex auth list' 查看剩余账号");
        }
        Err(e) => {
            ColorOutput::error(&format!("删除失败: {}", e));
        }
    }

    Ok(())
}
