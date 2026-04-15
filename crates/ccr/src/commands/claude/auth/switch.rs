//! 🔄 claude auth switch 命令实现

#![allow(clippy::unused_async)]

use crate::services::ClaudeAuthService;
use ccr_core::core::error::Result;
use ccr_core::core::logging::ColorOutput;
use colored::Colorize;

pub async fn switch_command(name: &str) -> Result<()> {
    let service = ClaudeAuthService::new()?;

    match service.switch_account(name) {
        Ok(()) => {
            println!();
            ColorOutput::success(&format!(
                "已切换到 Claude 官方账号: {}",
                name.bright_green().bold()
            ));
            println!();
            ColorOutput::info("提示:");
            println!("  • 已仅更新 ~/.claude/.credentials.json");
            println!("  • 若当前 Profile 为 subscription，Claude Code 将回落到该官方账号");
            println!("  • 若当前 Profile 为 api_key，仍以 ANTHROPIC_* 覆盖为准");
        }
        Err(e) => {
            ColorOutput::error(&format!("切换失败: {}", e));
            if e.to_string().contains("不存在") {
                println!();
                ColorOutput::info("使用以下命令查看可用账号:");
                println!("  ccr claude auth list");
            }
        }
    }

    Ok(())
}
