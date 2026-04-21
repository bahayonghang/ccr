//! ✏️ codex auth update 命令实现

#![allow(clippy::unused_async)]

use crate::services::CodexAuthService;
use ccr_core::core::error::{CcrError, Result};
use ccr_core::core::logging::ColorOutput;
use serde::Serialize;

#[derive(Debug, Serialize)]
struct CodexAuthUpdateOutput {
    ok: bool,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    message: String,
}

pub async fn update_command(
    name: &str,
    description: Option<String>,
    clear_description: bool,
    json: bool,
) -> Result<()> {
    if description.is_some() == clear_description {
        return Err(CcrError::ValidationError(
            "请提供 --description 或 --clear-description，且二者只能选一个".into(),
        ));
    }

    let service = CodexAuthService::new()?;
    let updated = service.update_account_description(
        name,
        if clear_description {
            None
        } else {
            description.filter(|value| !value.trim().is_empty())
        },
    )?;

    let output = CodexAuthUpdateOutput {
        ok: true,
        name: name.to_string(),
        description: updated.description,
        message: format!("已更新 Codex auth '{}' 的描述", name),
    };

    if json {
        println!(
            "{}",
            serde_json::to_string(&output).map_err(CcrError::JsonError)?
        );
        return Ok(());
    }

    ColorOutput::success(&output.message);
    Ok(())
}
