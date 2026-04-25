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
    dry_run: bool,
    codex_home: Option<String>,
) -> Result<()> {
    let service = build_service(codex_home)?;
    let mut options = CodexHistorySyncOptions {
        provider,
        max_age_days,
        dry_run,
        ..Default::default()
    };
    if let Some(keep) = keep {
        options.keep_count = keep;
    }

    let result = service.sync(options)?;
    print_sync_result(&result);
    Ok(())
}

pub async fn status_command(codex_home: Option<String>) -> Result<()> {
    let service = build_service(codex_home)?;
    let status = service.status()?;
    print_status(&status);
    Ok(())
}

pub async fn restore_command(
    backup_dir: &str,
    codex_home: Option<String>,
    restore_state: bool,
) -> Result<()> {
    let service = build_service(codex_home)?;
    let result = if restore_state {
        service.restore_with_state(backup_dir)?
    } else {
        service.restore(backup_dir)?
    };

    ColorOutput::success("已恢复 sync-history 备份");
    ColorOutput::info(&format!("Codex home: {}", result.codex_home.display()));
    ColorOutput::info(&format!("Backup: {}", result.backup_dir.display()));
    ColorOutput::info(&format!(
        "Provider at backup time: {}",
        result.target_provider
    ));
    if result.restored_state {
        ColorOutput::warning(
            "Restored state_5.sqlite and global state; this can overwrite metadata created after the backup.",
        );
    } else {
        ColorOutput::warning(
            "Restored rollout provider metadata only. Use --restore-state only when you intentionally want the older SQLite/global-state snapshot.",
        );
    }
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
        "Codex runtime provider: {}{}",
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
    println!("Recent rollout files (last 7 days):");
    println!(
        "  sessions: {}",
        format_counts(&status.recent_rollout_counts.sessions)
    );
    println!(
        "  archived_sessions: {}",
        format_counts(&status.recent_rollout_counts.archived_sessions)
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
    println!();
    println!(
        "Rollout / SQLite difference: {}",
        format_bucket_diff(&status.rollout_counts, status.sqlite_counts.as_ref())
    );
}

fn print_sync_result(result: &CodexHistorySyncResult) {
    println!("Provider metadata target: {}", result.target_provider);
    println!("Mode: {}", if result.dry_run { "dry-run" } else { "write" });
    println!("Max age filter: last {} day(s)", result.max_age_days);
    println!("Codex home: {}", result.codex_home.display());
    println!(
        "Filtered rollout sessions: {}",
        format_counts(&result.rollout_counts.sessions)
    );
    println!(
        "Filtered rollout archived_sessions: {}",
        format_counts(&result.rollout_counts.archived_sessions)
    );
    if let Some(sqlite_counts) = &result.sqlite_counts {
        println!(
            "SQLite sessions before sync: {}",
            format_counts(&sqlite_counts.sessions)
        );
        println!(
            "SQLite archived_sessions before sync: {}",
            format_counts(&sqlite_counts.archived_sessions)
        );
    } else {
        println!("SQLite provider distribution: state_5.sqlite not found");
    }
    if let Some(backup_dir) = &result.backup_dir {
        println!("Backup: {}", backup_dir.display());
    } else {
        println!("Backup: not created");
    }
    let rollout_label = if result.dry_run {
        "Rollout files to update"
    } else {
        "Updated rollout files"
    };
    let sidebar_label = if result.dry_run {
        "Sidebar projects to add"
    } else {
        "Added sidebar projects"
    };
    println!("{}: {}", rollout_label, result.changed_rollout_files);
    println!("{}: {}", sidebar_label, result.added_sidebar_projects);
    if result.sqlite_present {
        let sqlite_label = if result.dry_run {
            "SQLite rows to update/insert"
        } else {
            "Updated SQLite rows"
        };
        println!("{}: {}", sqlite_label, result.sqlite_rows_updated);
    } else {
        println!(
            "SQLite rows to update/insert: {} (state_5.sqlite not found)",
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

fn format_bucket_diff(
    rollout_counts: &crate::services::CodexHistoryProviderBuckets,
    sqlite_counts: Option<&crate::services::CodexHistoryProviderBuckets>,
) -> String {
    let Some(sqlite_counts) = sqlite_counts else {
        return "state_5.sqlite not found".to_string();
    };

    let mut diffs = Vec::new();
    append_count_diffs(
        "sessions",
        &rollout_counts.sessions,
        &sqlite_counts.sessions,
        &mut diffs,
    );
    append_count_diffs(
        "archived_sessions",
        &rollout_counts.archived_sessions,
        &sqlite_counts.archived_sessions,
        &mut diffs,
    );

    if diffs.is_empty() {
        "none".to_string()
    } else {
        diffs.join("; ")
    }
}

fn append_count_diffs(
    scope: &str,
    rollout: &std::collections::BTreeMap<String, usize>,
    sqlite: &std::collections::BTreeMap<String, usize>,
    diffs: &mut Vec<String>,
) {
    let providers = rollout
        .keys()
        .chain(sqlite.keys())
        .cloned()
        .collect::<std::collections::BTreeSet<_>>();
    for provider in providers {
        let rollout_count = rollout.get(&provider).copied().unwrap_or_default();
        let sqlite_count = sqlite.get(&provider).copied().unwrap_or_default();
        if rollout_count != sqlite_count {
            diffs.push(format!(
                "{scope}/{provider}: rollout {rollout_count}, sqlite {sqlite_count}"
            ));
        }
    }
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
