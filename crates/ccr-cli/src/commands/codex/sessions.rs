//! Codex session trash/restore commands.

use crate::services::CodexSessionTrashService;
use ccr_core::core::error::Result;
use ccr_core::core::logging::ColorOutput;
use std::path::PathBuf;

pub async fn trash_command(session_ids: Vec<String>, codex_home: Option<String>) -> Result<()> {
    let service = build_service(codex_home)?;
    let result = service.trash_sessions(session_ids)?;

    ColorOutput::success(&format!(
        "已移动 {} / {} 条 Codex 会话到垃圾箱",
        result.trashed_session_count, result.requested_session_count
    ));
    ColorOutput::info(&format!("Trash root: {}", result.trash_root.display()));
    for session in result.trashed_sessions {
        println!(
            "{}\t{}\t{}",
            session.session_id, session.title, session.original_relative_path
        );
    }
    Ok(())
}

pub async fn list_command(codex_home: Option<String>) -> Result<()> {
    let service = build_service(codex_home)?;
    let sessions = service.list_trashed_sessions()?;
    if sessions.is_empty() {
        ColorOutput::info("没有可恢复的 Codex 会话");
        return Ok(());
    }

    for session in sessions {
        println!(
            "{}\t{}\t{}\t{}",
            session.session_id, session.title, session.deleted_at, session.original_relative_path
        );
    }
    Ok(())
}

pub async fn restore_command(session_ids: Vec<String>, codex_home: Option<String>) -> Result<()> {
    let service = build_service(codex_home)?;
    let result = service.restore_sessions(session_ids)?;

    ColorOutput::success(&format!(
        "已恢复 {} / {} 条 Codex 会话",
        result.restored_session_count, result.requested_session_count
    ));
    for session in result.restored_sessions {
        println!(
            "{}\t{}\t{}",
            session.session_id, session.title, session.original_relative_path
        );
    }
    Ok(())
}

fn build_service(codex_home: Option<String>) -> Result<CodexSessionTrashService> {
    match codex_home {
        Some(path) => Ok(CodexSessionTrashService::with_codex_home(PathBuf::from(
            path,
        ))),
        None => CodexSessionTrashService::new(),
    }
}
