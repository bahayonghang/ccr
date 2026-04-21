//! 🔄 claude auth switch 命令实现

#![allow(clippy::unused_async)]

use crate::models::ClaudeLoginState;
use crate::services::ClaudeAuthService;
use ccr_core::core::error::Result;
use ccr_core::core::logging::ColorOutput;
use colored::Colorize;

pub async fn switch_command(name: &str) -> Result<()> {
    let service = ClaudeAuthService::new()?;
    let had_api_key_override = service
        .get_runtime_summary()
        .ok()
        .is_some_and(|summary| matches!(summary.login_state, ClaudeLoginState::ApiKeyActive));

    match service.switch_account(name) {
        Ok(()) => {
            println!();
            ColorOutput::success(&format!(
                "已切换到 Claude 官方账号: {}",
                name.bright_green().bold()
            ));
            println!();
            ColorOutput::info("提示:");
            println!("  • 已更新 ~/.claude/.credentials.json");
            if had_api_key_override {
                println!("  • 已清理 settings.json 中当前 Profile 写入的 ANTHROPIC_* 覆盖");
            }
            println!("  • Claude Code 现在会优先回落到该官方账号");
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
