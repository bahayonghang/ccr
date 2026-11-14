use super::render::{GitignoreOutcome, RefDirOutcome};
use ccr_core::core::AtomicWriter;
use ccr_core::core::error::{CcrError, Result};
use ccr_core::core::logging::ColorOutput;
use std::fs;
use std::path::Path;

// 五项平台的本地目录，加上 Codex 共用的 `.agents/` 和本地 `ref/`。
// 不随本次勾选或所选引擎变化。`.trellis/` 要入库，不在此列。
pub(crate) const PROJECT_IGNORE_RULES: [&str; 7] = [
    ".agents/", ".claude/", ".codex/", ".cursor/", ".omp/", ".grok/", "ref/",
];

pub(crate) fn ensure_ref_dir(root: &Path) -> Result<RefDirOutcome> {
    let path = root.join("ref");
    if path.is_dir() {
        ColorOutput::info("  ref/ 目录已存在，保留现有内容");
        return Ok(RefDirOutcome::Existing);
    }
    if path.is_file() {
        return Err(CcrError::FileIoError(format!(
            "ref/ 阶段：{} 已是文件，无法创建目录",
            path.display()
        )));
    }
    if path.exists() {
        return Err(CcrError::FileIoError(format!(
            "ref/ 阶段：{} 已存在但不是目录",
            path.display()
        )));
    }

    fs::create_dir(&path).map_err(|error| {
        CcrError::FileIoError(format!("ref/ 阶段：创建 {} 失败: {error}", path.display()))
    })?;
    ColorOutput::success("已创建 ref/ 目录");
    Ok(RefDirOutcome::Created)
}

pub(crate) fn ensure_project_gitignore(root: &Path) -> Result<GitignoreOutcome> {
    let path = root.join(".gitignore");
    let existing = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => {
            return Err(CcrError::FileIoError(format!(
                ".gitignore 阶段：读取 {} 失败: {error}",
                path.display()
            )));
        }
    };

    let Some(plan) = plan_project_ignore_merge(&existing) else {
        ColorOutput::info("  .gitignore 已包含全部项目忽略规则，跳过写入");
        return Ok(GitignoreOutcome::Unchanged);
    };

    AtomicWriter::new(&path)
        .write_string(&plan.merged)
        .map_err(|error| {
            CcrError::FileIoError(format!(
                ".gitignore 阶段：原子写入 {} 失败: {error}",
                path.display()
            ))
        })?;

    ColorOutput::success(&gitignore_ready_message());
    Ok(GitignoreOutcome::Updated {
        added_rules: plan.added_rules,
    })
}

fn gitignore_ready_message() -> String {
    format!(
        ".gitignore 已包含 {}",
        join_chinese_list(&PROJECT_IGNORE_RULES)
    )
}

fn join_chinese_list(items: &[&str]) -> String {
    let Some((last, rest)) = items.split_last() else {
        return String::new();
    };
    if rest.is_empty() {
        return (*last).to_string();
    }
    format!("{} 和 {last}", rest.join("、"))
}

fn missing_project_ignore_rules(existing: &str) -> Vec<&'static str> {
    PROJECT_IGNORE_RULES
        .into_iter()
        .filter(|rule| !existing.lines().any(|line| line.trim() == *rule))
        .collect()
}

struct GitignoreMergePlan {
    merged: String,
    added_rules: Vec<&'static str>,
}

// 一次计算缺失规则并生成合并文本，保留原有内容与 LF/CRLF 换行风格。
fn plan_project_ignore_merge(existing: &str) -> Option<GitignoreMergePlan> {
    let added_rules = missing_project_ignore_rules(existing);
    if added_rules.is_empty() {
        return None;
    }

    let line_ending = if existing.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let mut merged = existing.to_owned();
    if !merged.is_empty() && !merged.ends_with('\n') {
        merged.push_str(line_ending);
    }
    for &rule in &added_rules {
        merged.push_str(rule);
        merged.push_str(line_ending);
    }

    Some(GitignoreMergePlan {
        merged,
        added_rules,
    })
}

#[cfg(test)]
fn merge_project_ignore_rules(existing: &str) -> Option<String> {
    plan_project_ignore_merge(existing).map(|plan| plan.merged)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn merge_creates_rules_for_empty_file() {
        assert_eq!(
            merge_project_ignore_rules(""),
            Some(".agents/\n.claude/\n.codex/\n.cursor/\n.omp/\n.grok/\nref/\n".into())
        );
    }

    #[test]
    fn merge_preserves_existing_content_and_adds_line_boundary() {
        assert_eq!(
            merge_project_ignore_rules("target/"),
            Some("target/\n.agents/\n.claude/\n.codex/\n.cursor/\n.omp/\n.grok/\nref/\n".into())
        );
    }

    #[test]
    fn merge_only_adds_missing_rules() {
        assert_eq!(
            merge_project_ignore_rules("target/\n.agents/\n.codex/\n"),
            Some("target/\n.agents/\n.codex/\n.claude/\n.cursor/\n.omp/\n.grok/\nref/\n".into())
        );
    }

    #[test]
    fn merge_appends_platform_dirs_and_ref_when_only_original_agent_rules_exist() {
        assert_eq!(
            merge_project_ignore_rules(".agents/\n.claude/\n.codex/\n"),
            Some(".agents/\n.claude/\n.codex/\n.cursor/\n.omp/\n.grok/\nref/\n".into())
        );
    }

    #[test]
    fn merge_appends_cursor_omp_and_grok_onto_legacy_four_rules() {
        assert_eq!(
            merge_project_ignore_rules(".agents/\n.claude/\n.codex/\nref/\n"),
            Some(".agents/\n.claude/\n.codex/\nref/\n.cursor/\n.omp/\n.grok/\n".into())
        );
    }

    #[test]
    fn merge_preserves_crlf_line_endings() {
        assert_eq!(
            merge_project_ignore_rules("target/\r\n.agents/\r\n"),
            Some(
                "target/\r\n.agents/\r\n.claude/\r\n.codex/\r\n.cursor/\r\n.omp/\r\n.grok/\r\nref/\r\n"
                    .into()
            )
        );
    }

    #[test]
    fn merge_is_unchanged_when_all_rules_exist_with_surrounding_whitespace() {
        let existing =
            "target/\n  .agents/  \n.claude/\n.codex/\n.cursor/\n.omp/\n.grok/\n  ref/  \n";
        assert_eq!(merge_project_ignore_rules(existing), None);
    }

    #[test]
    fn gitignore_ready_message_lists_every_rule() {
        assert_eq!(
            gitignore_ready_message(),
            ".gitignore 已包含 .agents/、.claude/、.codex/、.cursor/、.omp/、.grok/ 和 ref/"
        );
    }
}
