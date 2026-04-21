//! 📥 opencode auth import-codex 命令实现
//!
//! 从已保存的 Codex Auth 账号导入可兼容的 OpenCode 账号快照。

#![allow(clippy::unused_async)]

use crate::models::CodexToOpenCodeMigrationReport;
use crate::services::OpenCodeAuthService;
use ccr_core::core::error::Result;

pub async fn import_codex_command(dry_run: bool, json: bool) -> Result<()> {
    let service = OpenCodeAuthService::new()?;
    let report = service.import_saved_codex_accounts(dry_run)?;

    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
        return Ok(());
    }

    println!("{}", render_import_codex_report(&report));
    Ok(())
}

fn render_import_codex_report(report: &CodexToOpenCodeMigrationReport) -> String {
    let mut lines = Vec::new();

    if report.dry_run {
        lines.push("OpenCode 导入预览完成".to_string());
    } else {
        lines.push("OpenCode 导入完成".to_string());
    }
    lines.push(String::new());

    if report.total() == 0 {
        lines.push("未发现已保存的 Codex 账号。".to_string());
        return lines.join("\n");
    }

    lines.push(format!(
        "{}: {}",
        if report.dry_run {
            "可导入"
        } else {
            "已导入"
        },
        report.imported
    ));
    lines.push(format!("同名跳过: {}", report.skipped_existing_name));
    lines.push(format!(
        "同 account_id 跳过: {}",
        report.skipped_existing_account_id
    ));
    lines.push(format!("认证不兼容: {}", report.skipped_incompatible_auth));
    lines.push(format!("缺少快照: {}", report.skipped_missing_snapshot));
    lines.push(format!("快照无效: {}", report.skipped_invalid_snapshot));

    if report.has_importable_accounts() {
        lines.push(String::new());
        lines.push(
            "说明: 不会覆盖现有 OpenCode 账号，也不会修改当前 OpenCode 运行时登录。".to_string(),
        );
    }

    if !report.outcomes.is_empty() {
        lines.push(String::new());
        lines.push("账号结果:".to_string());
        for outcome in &report.outcomes {
            lines.push(format!(
                "  • {} [{}] {}",
                outcome.name,
                outcome_status_label(outcome),
                outcome.message
            ));
        }
    }

    lines.join("\n")
}

fn outcome_status_label(item: &crate::models::CodexToOpenCodeMigrationItem) -> &'static str {
    use crate::models::CodexToOpenCodeMigrationStatus;

    match item.status {
        CodexToOpenCodeMigrationStatus::Imported => "imported",
        CodexToOpenCodeMigrationStatus::SkippedExistingName => "existing-name",
        CodexToOpenCodeMigrationStatus::SkippedExistingAccountId => "existing-account-id",
        CodexToOpenCodeMigrationStatus::SkippedIncompatibleAuth => "incompatible-auth",
        CodexToOpenCodeMigrationStatus::SkippedMissingSnapshot => "missing-snapshot",
        CodexToOpenCodeMigrationStatus::SkippedInvalidSnapshot => "invalid-snapshot",
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::models::{CodexToOpenCodeMigrationItem, CodexToOpenCodeMigrationStatus};

    #[test]
    fn render_import_codex_report_includes_counts_and_outcomes() {
        let report = CodexToOpenCodeMigrationReport {
            dry_run: true,
            imported: 1,
            skipped_existing_name: 1,
            skipped_existing_account_id: 0,
            skipped_incompatible_auth: 1,
            skipped_missing_snapshot: 0,
            skipped_invalid_snapshot: 0,
            outcomes: vec![
                CodexToOpenCodeMigrationItem {
                    name: "work".to_string(),
                    status: CodexToOpenCodeMigrationStatus::Imported,
                    account_id: Some("acc-1".to_string()),
                    message: "可导入到 OpenCode".to_string(),
                },
                CodexToOpenCodeMigrationItem {
                    name: "personal".to_string(),
                    status: CodexToOpenCodeMigrationStatus::SkippedExistingName,
                    account_id: Some("acc-2".to_string()),
                    message: "OpenCode 已存在同名账号 'personal'".to_string(),
                },
            ],
        };

        let rendered = render_import_codex_report(&report);
        assert!(rendered.contains("OpenCode 导入预览完成"));
        assert!(rendered.contains("可导入: 1"));
        assert!(rendered.contains("同名跳过: 1"));
        assert!(rendered.contains("认证不兼容: 1"));
        assert!(rendered.contains("work [imported]"));
        assert!(rendered.contains("personal [existing-name]"));
    }

    #[test]
    fn json_report_serializes() {
        let report = CodexToOpenCodeMigrationReport {
            dry_run: false,
            imported: 1,
            skipped_existing_name: 0,
            skipped_existing_account_id: 0,
            skipped_incompatible_auth: 0,
            skipped_missing_snapshot: 0,
            skipped_invalid_snapshot: 0,
            outcomes: vec![],
        };

        let json = serde_json::to_string_pretty(&report).unwrap();
        assert!(json.contains("\"imported\": 1"));
        assert!(json.contains("\"dry_run\": false"));
    }
}
