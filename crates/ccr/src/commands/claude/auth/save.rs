//! 💾 claude auth save 命令实现

#![allow(clippy::unused_async)]

use crate::services::ClaudeAuthService;
use ccr_core::core::error::Result;
use ccr_core::core::logging::ColorOutput;
use colored::Colorize;

pub async fn save_command(name: &str, description: Option<String>, force: bool) -> Result<()> {
    let service = ClaudeAuthService::new()?;

    match service.save_current(name, description.clone(), force) {
        Ok(account) => {
            println!();
            ColorOutput::success(&format!(
                "已保存 Claude 官方账号: {}",
                name.bright_green().bold()
            ));
            if let Some(email) = &account.email {
                ColorOutput::info(&format!("邮箱: {}", email));
            }
            if let Some(subscription_type) = &account.subscription_type {
                ColorOutput::info(&format!("订阅类型: {}", subscription_type));
            }
            if let Some(expires_at) = account.expires_at {
                ColorOutput::info(&format!("Access Token 到期: {}", expires_at.to_rfc3339()));
            }
            println!();
            ColorOutput::info("提示:");
            println!("  • 使用 'ccr claude auth list' 查看所有已保存账号");
            println!("  • 使用 'ccr claude auth switch <名称>' 切换账号");
        }
        Err(e) => {
            ColorOutput::error(&format!("保存失败: {}", e));
            if e.to_string().contains("已存在") {
                println!();
                ColorOutput::info("提示: 使用 --force 覆盖已存在的账号快照");
            }
        }
    }

    Ok(())
}
