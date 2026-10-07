//! 🩹 codex auth repair 命令实现
//!
//! 从 ~/.codex/auth.json 与 ~/.codex/backups 扫描最新 OAuth tokens，
//! 并回写到 ~/.ccr/platforms/codex/auth/<name>.json。

#![allow(clippy::unused_async)]

use crate::commands::common::print_next_steps;
use crate::services::CodexOAuthTokenService;
use ccr_core::core::error::Result;
use ccr_core::core::logging::ColorOutput;

/// 🩹 修复指定账号的 OAuth tokens
pub async fn repair_command(name: &str) -> Result<()> {
    let service = CodexOAuthTokenService::new()?;
    let outcome = service.repair_saved_account(name)?;

    if outcome.updated {
        ColorOutput::success(&format!("已修复账号: {}", name));
        if let Some(source) = &outcome.source {
            ColorOutput::key_value("来源", &source.label(), 2);
        }
        Ok(())
    } else {
        ColorOutput::warning(&format!("无需修复或修复失败: {}", name));
        ColorOutput::info(&outcome.message);
        if outcome.message.contains("未在 runtime/backups") {
            ColorOutput::info("重新登录后保存会覆盖该账号已保存的凭据快照");
            print_next_steps(&[
                ("重新登录 Codex", "codex login"),
                ("查看覆盖保存帮助", "ccr codex auth save --help"),
            ]);
        }
        Ok(())
    }
}
