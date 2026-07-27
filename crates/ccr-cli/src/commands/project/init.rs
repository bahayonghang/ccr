use ccr_core::core::error::{CcrError, Result};
use ccr_core::core::{AtomicWriter, ColorOutput};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const PROJECT_IGNORE_RULES: [&str; 3] = [".agents/", ".claude/", ".codex/"];

pub fn project_init_command(auto_yes: bool) -> Result<()> {
    let root = std::env::current_dir().map_err(|error| {
        CcrError::FileIoError(format!("项目初始化阶段：无法获取当前工作目录: {error}"))
    })?;

    ColorOutput::title("项目工作流初始化");
    ColorOutput::info(&format!("目标目录: {}", root.display()));
    println!();

    ensure_git_repository(&root)?;
    println!();
    run_trellis_init(&root, auto_yes)?;
    println!();
    ensure_project_gitignore(&root)?;

    println!();
    ColorOutput::separator();
    println!();
    ColorOutput::success("Git、Trellis 和 Agent 忽略规则均已就绪");

    Ok(())
}

fn ensure_git_repository(root: &Path) -> Result<()> {
    ColorOutput::step("检查 Git 仓库");

    let git = resolve_tool("git", "Git")?;

    let probe = Command::new(&git)
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(root)
        .output()
        .map_err(|error| {
            CcrError::ExternalCommandError(format!("Git 阶段：无法启动 git rev-parse: {error}"))
        })?;

    if probe.status.success() {
        let repository_root = PathBuf::from(String::from_utf8_lossy(&probe.stdout).trim());
        if repository_root.as_os_str().is_empty() {
            return Err(CcrError::ExternalCommandError(
                "Git 阶段：git rev-parse 成功退出但未返回仓库根目录".into(),
            ));
        }

        if paths_match(root, &repository_root) {
            ColorOutput::success("当前目录已经是 Git 仓库根，跳过 git init");
        } else {
            ColorOutput::warning(&format!(
                "当前目录位于 Git 仓库 {} 中，跳过嵌套 git init",
                repository_root.display()
            ));
        }
        return Ok(());
    }

    ColorOutput::info("当前目录不在 Git 工作树中，运行 git init");
    let status = Command::new(&git)
        .arg("init")
        .current_dir(root)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|error| {
            CcrError::ExternalCommandError(format!("Git 阶段：无法启动 git init: {error}"))
        })?;

    if !status.success() {
        return Err(CcrError::ExternalCommandError(format!(
            "Git 阶段：git init 返回非零状态 {status}"
        )));
    }

    ColorOutput::success("Git 仓库初始化完成");
    Ok(())
}

fn run_trellis_init(root: &Path, auto_yes: bool) -> Result<()> {
    ColorOutput::step("初始化 Trellis 工作流");

    let trellis = resolve_tool("trellis", "Trellis")?;
    let mut command = Command::new(trellis);
    command.arg("init").current_dir(root);
    if auto_yes {
        command.arg("--yes");
    }
    let status = command
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|error| {
            CcrError::ExternalCommandError(format!("Trellis 阶段：无法启动 trellis init: {error}"))
        })?;

    if !status.success() {
        return Err(CcrError::ExternalCommandError(format!(
            "Trellis 阶段：trellis init 返回非零状态 {status}"
        )));
    }

    validate_trellis_workflow(root)?;
    ColorOutput::success("Trellis 工作流已就绪");
    Ok(())
}

fn resolve_tool(name: &str, stage: &str) -> Result<PathBuf> {
    crate::services::install_detect::which_on_path(name).ok_or_else(|| {
        CcrError::ExternalCommandError(format!("{stage} 阶段：PATH 中找不到 {name} 可执行文件"))
    })
}

fn validate_trellis_workflow(root: &Path) -> Result<()> {
    let workflow = root.join(".trellis/workflow.md");
    let task_script = root.join(".trellis/scripts/task.py");
    let missing = [workflow, task_script]
        .into_iter()
        .filter(|path| !path.is_file())
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>();

    if missing.is_empty() {
        Ok(())
    } else {
        Err(CcrError::ValidationError(format!(
            "Trellis 阶段：trellis init 成功退出，但缺少最低工作流文件: {}",
            missing.join(", ")
        )))
    }
}

fn ensure_project_gitignore(root: &Path) -> Result<()> {
    ColorOutput::step("更新 .gitignore 中的 Agent 目录规则");
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

    let Some(merged) = merge_project_ignore_rules(&existing) else {
        ColorOutput::info(".gitignore 已包含全部 Agent 目录规则，跳过写入");
        return Ok(());
    };

    AtomicWriter::new(&path)
        .write_string(&merged)
        .map_err(|error| {
            CcrError::FileIoError(format!(
                ".gitignore 阶段：原子写入 {} 失败: {error}",
                path.display()
            ))
        })?;

    ColorOutput::success(".gitignore 已包含 .agents/、.claude/ 和 .codex/");
    Ok(())
}

fn merge_project_ignore_rules(existing: &str) -> Option<String> {
    let missing = PROJECT_IGNORE_RULES
        .into_iter()
        .filter(|rule| !existing.lines().any(|line| line.trim() == *rule))
        .collect::<Vec<_>>();

    if missing.is_empty() {
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
    for rule in missing {
        merged.push_str(rule);
        merged.push_str(line_ending);
    }

    Some(merged)
}

fn paths_match(left: &Path, right: &Path) -> bool {
    normalize_existing_path(left) == normalize_existing_path(right)
}

fn normalize_existing_path(path: &Path) -> PathBuf {
    fs::canonicalize(path)
        .or_else(|_| std::path::absolute(path))
        .unwrap_or_else(|_| path.to_path_buf())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn merge_creates_rules_for_empty_file() {
        assert_eq!(
            merge_project_ignore_rules(""),
            Some(".agents/\n.claude/\n.codex/\n".into())
        );
    }

    #[test]
    fn merge_preserves_existing_content_and_adds_line_boundary() {
        assert_eq!(
            merge_project_ignore_rules("target/"),
            Some("target/\n.agents/\n.claude/\n.codex/\n".into())
        );
    }

    #[test]
    fn merge_only_adds_missing_rules() {
        assert_eq!(
            merge_project_ignore_rules("target/\n.agents/\n.codex/\n"),
            Some("target/\n.agents/\n.codex/\n.claude/\n".into())
        );
    }

    #[test]
    fn merge_preserves_crlf_line_endings() {
        assert_eq!(
            merge_project_ignore_rules("target/\r\n.agents/\r\n"),
            Some("target/\r\n.agents/\r\n.claude/\r\n.codex/\r\n".into())
        );
    }

    #[test]
    fn merge_is_unchanged_when_all_rules_exist_with_surrounding_whitespace() {
        let existing = "target/\n  .agents/  \n.claude/\n.codex/\n";
        assert_eq!(merge_project_ignore_rules(existing), None);
    }

    #[test]
    fn trellis_validation_requires_both_minimum_files() {
        let temp = tempdir().unwrap();
        fs::create_dir_all(temp.path().join(".trellis/scripts")).unwrap();
        fs::write(temp.path().join(".trellis/workflow.md"), "workflow").unwrap();

        let error = validate_trellis_workflow(temp.path()).unwrap_err();
        assert!(error.to_string().contains("task.py"));

        fs::write(temp.path().join(".trellis/scripts/task.py"), "script").unwrap();
        validate_trellis_workflow(temp.path()).unwrap();
    }

    #[test]
    fn path_matching_distinguishes_root_from_child() {
        let temp = tempdir().unwrap();
        let child = temp.path().join("child");
        fs::create_dir(&child).unwrap();

        assert!(paths_match(temp.path(), temp.path()));
        assert!(!paths_match(temp.path(), &child));
    }
}
