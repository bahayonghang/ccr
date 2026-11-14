use super::WorkflowEngine;
use super::gitignore::{ensure_project_gitignore, ensure_ref_dir};
use super::render::{
    GitRepoOutcome, ProjectInitSummary, WizardCapabilities, format_config_confirmation,
    format_final_ready_line, format_header, format_stage_1_git, format_stage_2_config,
    format_stage_3_engine, format_stage_4_workspace_files, format_subprocess_end,
    format_subprocess_start, format_wizard_status, next_steps_for_engine, render_summary_table,
};
use super::wizard::resolve_workflow_request;
use super::workflow::{WorkflowInitRequest, build_workflow_init_args, validate_workflow_outputs};
use crate::commands::common::print_next_steps;
use ccr_core::core::error::{CcrError, Result};
use ccr_core::core::logging::OutputStatus;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub fn project_init_command(auto_yes: bool, workflow: Option<WorkflowEngine>) -> Result<()> {
    let root = std::env::current_dir().map_err(|error| {
        CcrError::FileIoError(format!("项目初始化阶段：无法获取当前工作目录: {error}"))
    })?;
    let caps = WizardCapabilities::for_stdout();

    println!("{}", format_header(&root, auto_yes, workflow, caps));
    println!();

    let (git, git_outcome) = ensure_git_repository(&root, caps)?;
    println!();

    println!("{}", format_stage_2_config(caps));
    let request = resolve_workflow_request(auto_yes, workflow, &git, &root, caps)?;
    println!("{}", format_config_confirmation(&request, caps));
    println!();

    run_workflow_init(&root, &request, caps)?;
    println!();

    println!("{}", format_stage_4_workspace_files(caps));
    let ref_outcome = ensure_ref_dir(&root, caps)?;
    let gitignore_outcome = ensure_project_gitignore(&root, caps)?;
    println!();

    let summary = ProjectInitSummary {
        git: &git_outcome,
        request: &request,
        ref_dir: ref_outcome,
        gitignore: &gitignore_outcome,
    };
    println!("{}", render_summary_table(&summary, caps));
    println!("{}", format_final_ready_line(request.workflow, caps));
    print_next_steps(next_steps_for_engine(request.workflow));

    Ok(())
}

fn ensure_git_repository(
    root: &Path,
    caps: WizardCapabilities,
) -> Result<(PathBuf, GitRepoOutcome)> {
    println!("{}", format_stage_1_git(caps));

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

        let outcome = if paths_match(root, &repository_root) {
            println!("  当前目录已经是 Git 仓库根，跳过 git init");
            GitRepoOutcome::ExistingRoot
        } else {
            println!(
                "{}",
                format_wizard_status(
                    OutputStatus::Warning,
                    &format!(
                        "当前目录位于 Git 仓库 {} 中，跳过嵌套 git init",
                        repository_root.display()
                    ),
                    caps,
                )
            );
            GitRepoOutcome::ParentWorktree { repository_root }
        };
        return Ok((git, outcome));
    }

    println!("  当前目录不在 Git 工作树中，运行 git init");
    let _ = io::stdout().flush();
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

    println!(
        "{}",
        format_wizard_status(OutputStatus::Success, "Git 仓库初始化完成", caps)
    );
    Ok((git, GitRepoOutcome::Initialized))
}

fn run_workflow_init(
    root: &Path,
    request: &WorkflowInitRequest,
    caps: WizardCapabilities,
) -> Result<()> {
    let engine = request.workflow;
    let stage = engine.stage_name();
    let binary_name = engine.binary_name();
    println!("{}", format_stage_3_engine(engine, caps));

    let executable = resolve_tool(binary_name, stage)?;
    let args = build_workflow_init_args(request)?;

    println!("{}", format_subprocess_start(engine, caps));
    let _ = io::stdout().flush();
    let status = Command::new(executable)
        .args(&args)
        .current_dir(root)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|error| {
            CcrError::ExternalCommandError(format!(
                "{stage} 阶段：无法启动 {binary_name} init: {error}"
            ))
        })?;
    println!("{}", format_subprocess_end(engine, caps));

    if !status.success() {
        return Err(CcrError::ExternalCommandError(format!(
            "{stage} 阶段：{binary_name} init 返回非零状态 {status}"
        )));
    }

    validate_workflow_outputs(root, engine)?;
    println!(
        "{}",
        format_wizard_status(
            OutputStatus::Success,
            &format!("{binary_name} 工作流产物校验通过"),
            caps,
        )
    );
    Ok(())
}

fn resolve_tool(name: &str, stage: &str) -> Result<PathBuf> {
    crate::services::install_detect::which_on_path(name).ok_or_else(|| {
        CcrError::ExternalCommandError(format!("{stage} 阶段：PATH 中找不到 {name} 可执行文件"))
    })
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
    use crate::commands::project::fingerprint::WorkflowInitMode;
    use crate::commands::project::workflow::WorkflowPlatform;
    use tempfile::tempdir;

    fn request(
        developer: &str,
        platforms: &[WorkflowPlatform],
        auto_yes: bool,
    ) -> WorkflowInitRequest {
        WorkflowInitRequest {
            workflow: WorkflowEngine::Trellis,
            mode: WorkflowInitMode::Fresh,
            developer: developer.to_string(),
            platforms: platforms.to_vec(),
            auto_yes,
        }
    }

    fn assert_args(actual: &[String], expected: &[&str]) {
        let expected = expected
            .iter()
            .map(|item| (*item).to_string())
            .collect::<Vec<_>>();
        assert_eq!(actual, expected);
    }

    #[test]
    fn trellis_args_include_all_five_platforms_in_order() {
        let args = build_workflow_init_args(&request(
            "  Ada Lovelace  ",
            &WorkflowPlatform::TRELLIS_PLATFORMS,
            false,
        ))
        .unwrap();

        assert_args(
            &args,
            &[
                "init",
                "-u",
                "Ada Lovelace",
                "--claude",
                "--codex",
                "--cursor",
                "--omp",
                "--grok",
            ],
        );
        assert!(!args.iter().any(|arg| arg == "--yes" || arg == "--kimi"));
    }

    #[test]
    fn trellis_args_drop_middle_platform_and_keep_order() {
        let platforms = [
            WorkflowPlatform::Grok,
            WorkflowPlatform::Claude,
            WorkflowPlatform::Omp,
            WorkflowPlatform::Codex,
        ];
        let args = build_workflow_init_args(&request("Ada", &platforms, false)).unwrap();

        assert_args(
            &args,
            &[
                "init", "-u", "Ada", "--claude", "--codex", "--omp", "--grok",
            ],
        );
        assert!(
            !args
                .iter()
                .any(|arg| arg == "--cursor" || arg == "--kimi" || arg == "--yes")
        );
    }

    #[test]
    fn trellis_args_reject_empty_platforms_and_blank_name() {
        let empty_platforms = build_workflow_init_args(&request("Ada", &[], false)).unwrap_err();
        assert!(matches!(empty_platforms, CcrError::ValidationError(_)));

        let blank_name =
            build_workflow_init_args(&request("   ", &WorkflowPlatform::TRELLIS_PLATFORMS, true))
                .unwrap_err();
        assert!(matches!(blank_name, CcrError::ValidationError(_)));
    }

    #[test]
    fn trellis_args_put_yes_last() {
        let args =
            build_workflow_init_args(&request("Ada", &WorkflowPlatform::TRELLIS_PLATFORMS, true))
                .unwrap();

        assert_args(
            &args,
            &[
                "init", "-u", "Ada", "--claude", "--codex", "--cursor", "--omp", "--grok", "--yes",
            ],
        );
        assert_eq!(args.last().map(String::as_str), Some("--yes"));
    }

    #[test]
    fn trellis_validation_requires_both_minimum_files() {
        let temp = tempdir().unwrap();
        fs::create_dir_all(temp.path().join(".trellis/scripts")).unwrap();
        fs::write(temp.path().join(".trellis/workflow.md"), "workflow").unwrap();

        let error = validate_workflow_outputs(temp.path(), WorkflowEngine::Trellis).unwrap_err();
        assert!(error.to_string().contains("task.py"));

        fs::write(temp.path().join(".trellis/scripts/task.py"), "script").unwrap();
        validate_workflow_outputs(temp.path(), WorkflowEngine::Trellis).unwrap();
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
