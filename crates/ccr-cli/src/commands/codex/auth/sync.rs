//! 🔄 codex auth sync 命令实现
//!
//! 将当前 runtime OAuth tokens 与匹配的已保存账号快照按新鲜度对齐。

#![allow(clippy::unused_async)]

use crate::commands::common::print_next_steps;
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
            ColorOutput::key_value("账号", &name, 2);
        }
        RuntimeSyncOutcome::Unchanged(name) => {
            ColorOutput::info("OAuth tokens 已是最新");
            ColorOutput::key_value("账号", &name, 2);
        }
        RuntimeSyncOutcome::RuntimeUpdated(name) => {
            ColorOutput::success("已将较新的账号快照写回 ~/.codex/auth.json");
            ColorOutput::key_value("账号", &name, 2);
        }
        RuntimeSyncOutcome::SkippedStaleRuntime(name) => {
            ColorOutput::warning("账号快照比 runtime 更新，未覆盖快照");
            ColorOutput::key_value("账号", &name, 2);
        }
        RuntimeSyncOutcome::NoOp => {
            ColorOutput::warning("未找到可同步的 OAuth tokens");
            ColorOutput::info(
                "当前 ~/.codex/auth.json 需要包含 ChatGPT 登录态的 tokens，并匹配已保存账号",
            );
            print_next_steps(&[
                ("查看当前账号", "ccr codex auth current"),
                ("查看保存帮助", "ccr codex auth save --help"),
            ]);
        }
    }
    Ok(())
}
