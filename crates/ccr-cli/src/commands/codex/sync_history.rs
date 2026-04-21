//! 🔄 codex sync-history 命令实现
//!
//! 同步 Codex 历史会话的 provider 元数据，并提供状态/恢复/备份清理能力。

use crate::services::{
    CodexHistoryBackupPruneResult, CodexHistorySyncOptions, CodexHistorySyncResult,
    CodexHistorySyncService, CodexHistorySyncStatus,
};
use ccr_core::core::error::Result;
use ccr_core::core::logging::ColorOutput;
use std::path::PathBuf;

pub async fn sync_command(
    provider: Option<String>,
    keep: Option<usize>,
    max_age_days: u64,
    codex_home: Option<String>,
) -> Result<()> {
    let service = build_service(codex_home)?;
    let mut options = CodexHistorySyncOptions {
        provider,
        max_age_days,
        ..Default::default()
    };
    if let Some(keep) = keep {
        options.keep_count = keep;
    }

    let result = service.sync(options)?;
    print_sync_result("Synchronized", &result);
    Ok(())
}

pub async fn status_command(codex_home: Option<String>) -> Result<()> {
    let service = build_service(codex_home)?;
    let status = service.status()?;
    print_status(&status);
    Ok(())
}

pub async fn restore_command(backup_dir: &str, codex_home: Option<String>) -> Result<()> {
    let service = build_service(codex_home)?;
    let result = service.restore(backup_dir)?;

    ColorOutput::success("已恢复 sync-history 备份");
    ColorOutput::info(&format!("Codex home: {}", result.codex_home.display()));
    ColorOutput::info(&format!("Backup: {}", result.backup_dir.display()));
    ColorOutput::info(&format!(
        "Provider at backup time: {}",
        result.target_provider
    ));
    Ok(())
}

pub async fn prune_backups_command(keep: usize, codex_home: Option<String>) -> Result<()> {
    let service = build_service(codex_home)?;
    let result = service.prune_backups(keep)?;
    print_prune_result(&result);
    Ok(())
}

fn build_service(codex_home: Option<String>) -> Result<CodexHistorySyncService> {
    match codex_home {
        Some(path) => CodexHistorySyncService::with_codex_home(Some(PathBuf::from(path))),
        None => CodexHistorySyncService::new(),
    }
}

fn print_status(status: &CodexHistorySyncStatus) {
    println!("Codex home: {}", status.codex_home.display());
    println!(
        "Current provider: {}{}",
        status.current_provider,
        if status.current_provider_implicit {
            " (implicit default)"
        } else {
            ""
        }
    );
    println!(
        "Backups: {} ({})",
        status.backup_summary.count,
        format_bytes(status.backup_summary.total_bytes)
    );
    println!("Backup root: {}", status.backup_root.display());
    println!();
    println!("Rollout files:");
    println!(
        "  sessions: {}",
        format_counts(&status.rollout_counts.sessions)
    );
    println!(
        "  archived_sessions: {}",
        format_counts(&status.rollout_counts.archived_sessions)
    );
    println!();
    println!("SQLite state:");
    if let Some(sqlite_counts) = &status.sqlite_counts {
        println!("  sessions: {}", format_counts(&sqlite_counts.sessions));
        println!(
            "  archived_sessions: {}",
            format_counts(&sqlite_counts.archived_sessions)
        );
    } else {
        println!("  state_5.sqlite not found");
    }
}

fn print_sync_result(label: &str, result: &CodexHistorySyncResult) {
    println!("{label} provider: {}", result.target_provider);
    println!("Codex home: {}", result.codex_home.display());
    println!("Backup: {}", result.backup_dir.display());
    println!("Updated rollout files: {}", result.changed_rollout_files);
    println!("Added sidebar projects: {}", result.added_sidebar_projects);
    if result.sqlite_present {
        println!("Updated SQLite rows: {}", result.sqlite_rows_updated);
    } else {
        println!(
            "Updated SQLite rows: {} (state_5.sqlite not found)",
            result.sqlite_rows_updated
        );
    }

    if !result.skipped_locked_rollout_files.is_empty() {
        println!(
            "Skipped locked rollout files: {}",
            result.skipped_locked_rollout_files.len()
        );
        println!(
            "Locked file(s): {}",
            result
                .skipped_locked_rollout_files
                .iter()
                .map(|path| path.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    if let Some(backup_cleanup) = &result.backup_cleanup {
        println!(
            "Backup cleanup: deleted {}, remaining {}, freed {}",
            backup_cleanup.deleted_count,
            backup_cleanup.remaining_count,
            format_bytes(backup_cleanup.freed_bytes)
        );
    }
}

fn print_prune_result(result: &CodexHistoryBackupPruneResult) {
    println!("Backup root: {}", result.backup_root.display());
    println!("Deleted backups: {}", result.deleted_count);
    println!("Remaining backups: {}", result.remaining_count);
    println!("Freed space: {}", format_bytes(result.freed_bytes));
}

fn format_counts(counts: &std::collections::BTreeMap<String, usize>) -> String {
    if counts.is_empty() {
        return "(none)".to_string();
    }

    counts
        .iter()
        .map(|(provider, count)| format!("{provider}: {count}"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];

    let mut value = bytes as f64;
    let mut unit_index = 0usize;
    while value >= 1024.0 && unit_index < UNITS.len() - 1 {
        value /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{bytes} B")
    } else if value >= 10.0 {
        format!("{value:.1} {}", UNITS[unit_index]).replace(".0 ", " ")
    } else {
        format!("{value:.2} {}", UNITS[unit_index]).replace(".00 ", " ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_counts_returns_none_for_empty_bucket() {
        let counts = std::collections::BTreeMap::new();
        assert_eq!(format_counts(&counts), "(none)");
    }
}
