//! 🔄 codex auth sync 命令实现
//!
//! 将当前 runtime OAuth tokens 与匹配的已保存账号快照按新鲜度对齐。

#![allow(clippy::unused_async)]

use crate::services::CodexAuthService;
use ccr_codex::services::RuntimeSyncOutcome;
use ccr_core::core::error::Result;
use ccr_core::core::logging::ColorOutput;

/// 🔄 同步当前 runtime OAuth tokens 到已保存账号
pub async fn sync_command() -> Result<()> {
    let service = CodexAuthService::new()?;
    match service.sync_runtime_with_saved_account()? {
        RuntimeSyncOutcome::SnapshotUpdated(name) => {
            ColorOutput::success("已同步 OAuth tokens");
            ColorOutput::info(&format!("账号: {}", name));
        }
        RuntimeSyncOutcome::Unchanged(name) => {
            ColorOutput::success("OAuth tokens 已是最新");
            ColorOutput::info(&format!("账号: {}", name));
        }
        RuntimeSyncOutcome::RuntimeUpdated(name) => {
            ColorOutput::success("已将较新的账号快照写回 ~/.codex/auth.json");
            ColorOutput::info(&format!("账号: {}", name));
        }
        RuntimeSyncOutcome::SkippedStaleRuntime(name) => {
            ColorOutput::warning("账号快照比 runtime 更新，未覆盖快照");
            ColorOutput::info(&format!("账号: {}", name));
        }
        RuntimeSyncOutcome::NoOp => {
            ColorOutput::warning("未找到可同步的 OAuth tokens");
            ColorOutput::info("提示:");
            println!("  • 确认当前 ~/.codex/auth.json 为 ChatGPT 登录态（包含 tokens）");
            println!("  • 确认该账号已通过 `ccr codex auth save <name>` 保存");
        }
    }
    Ok(())
}
