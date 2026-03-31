//! 🩹 codex auth repair 命令实现
//!
//! 从 ~/.codex/auth.json 与 ~/.codex/backups 扫描最新 OAuth tokens，
//! 并回写到 ~/.ccr/platforms/codex/auth/<name>.json。

#![allow(clippy::unused_async)]

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
            ColorOutput::info(&format!("来源: {}", source.label()));
        }
        Ok(())
    } else {
        ColorOutput::warning(&format!("无需修复或修复失败: {}", name));
        ColorOutput::info(&outcome.message);
        if outcome.message.contains("未在 runtime/backups") {
            ColorOutput::info("建议:");
            println!("  codex login");
            println!("  ccr codex auth save {} --force", name);
        }
        Ok(())
    }
}
