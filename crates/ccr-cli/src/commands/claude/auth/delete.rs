//! 🗑️ claude auth delete 命令实现

#![allow(clippy::unused_async)]

use crate::services::ClaudeAuthService;
use ccr_core::core::error::{CcrError, Result};
use ccr_core::core::logging::ColorOutput;
use colored::Colorize;
use std::io::{self, Write};

pub async fn delete_command(name: &str, force: bool) -> Result<()> {
    let service = ClaudeAuthService::new()?;
    let accounts = service.list_accounts()?;
    let account = accounts.iter().find(|account| account.name == name);

    if account.is_none() {
        ColorOutput::error(&format!("账号 '{}' 不存在", name));
        println!();
        ColorOutput::info("使用以下命令查看可用账号:");
        println!("  ccr claude auth list");
        return Ok(());
    }

    let account = account.ok_or_else(|| CcrError::ConfigError("account should exist".into()))?;

    if !force {
        println!();
        ColorOutput::warning(&format!("即将删除账号: {}", name.bright_yellow().bold()));
        if let Some(email) = &account.email {
            ColorOutput::info(&format!("邮箱: {}", email));
        }
        if let Some(description) = &account.description {
            ColorOutput::info(&format!("描述: {}", description));
        }
        if account.is_current {
            println!();
            ColorOutput::warning("注意: 这是当前匹配到的运行时官方账号");
            ColorOutput::info(
                "删除后不会修改 ~/.claude/.credentials.json，但 CCR 将不再跟踪这个快照",
            );
        }

        println!();
        let confirmed = tokio::task::spawn_blocking(|| -> io::Result<bool> {
            print!("确认删除? (输入 'yes' 确认): ");
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            Ok(input.trim().eq_ignore_ascii_case("yes"))
        })
        .await
        .map_err(|e| CcrError::FileIoError(format!("读取确认输入失败: {}", e)))??;

        if !confirmed {
            ColorOutput::info("已取消删除");
            return Ok(());
        }
    }

    match service.delete_account(name) {
        Ok(()) => {
            println!();
            ColorOutput::success(&format!(
                "已删除 Claude 官方账号: {}",
                name.bright_red().bold()
            ));
        }
        Err(e) => ColorOutput::error(&format!("删除失败: {}", e)),
    }

    Ok(())
}
