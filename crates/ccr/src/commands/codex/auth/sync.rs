//! 🔄 codex auth sync 命令实现
//!
//! 将当前 runtime OAuth tokens 回写到匹配的已保存账号快照中。

#![allow(clippy::unused_async)]

use crate::core::error::Result;
use crate::core::logging::ColorOutput;
use crate::services::CodexOAuthTokenService;

/// 🔄 同步当前 runtime OAuth tokens 到已保存账号
pub async fn sync_command() -> Result<()> {
    let service = CodexOAuthTokenService::new()?;
    match service.sync_runtime_tokens_to_saved_account()? {
        Some(name) => {
            ColorOutput::success("已同步 OAuth tokens");
            ColorOutput::info(&format!("账号: {}", name));
            Ok(())
        }
        None => {
            ColorOutput::warning("未找到可同步的 OAuth tokens");
            ColorOutput::info("提示:");
            println!("  • 确认当前 ~/.codex/auth.json 为 ChatGPT 登录态（包含 tokens）");
            println!("  • 确认该账号已通过 `ccr codex auth save <name>` 保存");
            Ok(())
        }
    }
}
