//! ✏️ codex auth rename 命令实现
//!
//! 重命名已保存的 Codex 账号，同步更新 auth 文件、registry 与 usage_ledger。

#![allow(clippy::unused_async)]

use crate::services::CodexAuthService;
use ccr_core::core::error::{CcrError, Result};
use ccr_core::core::logging::ColorOutput;
use colored::Colorize;
use serde::Serialize;

#[derive(Debug, Serialize)]
struct CodexAuthRenameOutput {
    ok: bool,
    old_name: String,
    new_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    message: String,
}

/// ✏️ 重命名已保存账号
///
/// # 参数
///
/// * `old_name` - 当前账号名称
/// * `new_name` - 目标新名称
/// * `force`    - 当新名称已存在时是否强制覆盖
/// * `json`     - 是否以 JSON 格式输出
pub async fn rename_command(old_name: &str, new_name: &str, force: bool, json: bool) -> Result<()> {
    let service = CodexAuthService::new()?;
    let updated = service
        .rename_account(old_name, new_name, force)
        .map_err(|e| {
            // 常见冲突错误补充 --force 提示
            let msg = e.to_string();
            if !force && msg.contains("已存在") {
                CcrError::ConfigError(format!("{}\n提示: 使用 --force 覆盖同名账号", msg))
            } else {
                e
            }
        })?;

    let output = CodexAuthRenameOutput {
        ok: true,
        old_name: old_name.to_string(),
        new_name: new_name.to_string(),
        description: updated.description.clone(),
        message: format!("已重命名 Codex Auth '{}' -> '{}'", old_name, new_name),
    };

    if json {
        println!(
            "{}",
            serde_json::to_string(&output).map_err(CcrError::JsonError)?
        );
        return Ok(());
    }

    ColorOutput::success(&format!(
        "已重命名: {} -> {}",
        old_name.bright_yellow(),
        new_name.bright_green().bold()
    ));

    println!();
    ColorOutput::info("提示:");
    println!("  • 使用 'ccr codex auth list' 查看账号");

    Ok(())
}
