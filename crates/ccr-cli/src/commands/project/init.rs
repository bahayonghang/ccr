use crate::commands::common::prompt_required;
use ccr_core::core::error::{CcrError, Result};
use ccr_core::core::{AtomicWriter, ColorOutput};
use dialoguer::MultiSelect;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const PROJECT_IGNORE_RULES: [&str; 4] = [".agents/", ".claude/", ".codex/", "ref/"];
const PLATFORM_PROMPT: &str = "选择要初始化的平台";
const EMPTY_PLATFORM_WARNING: &str = "请至少选择一个平台";
const DEVELOPER_PROMPT: &str = "开发者用户名";
const DEVELOPER_HINT: &str = "用于 Trellis 工作区目录";
const MISSING_GIT_USER_MESSAGE: &str =
    "用户名阶段：需要设置 git config user.name，或去掉 -y 后手动输入开发者用户名";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TrellisPlatform {
    Claude,
    Codex,
    Cursor,
    Omp,
    Grok,
}

impl TrellisPlatform {
    const ALL: [Self; 5] = [
        Self::Claude,
        Self::Codex,
        Self::Cursor,
        Self::Omp,
        Self::Grok,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::Claude => "Claude Code",
            Self::Codex => "Codex",
            Self::Cursor => "Cursor",
            Self::Omp => "Oh My Pi",
            Self::Grok => "Grok",
        }
    }

    fn flag(self) -> &'static str {
        match self {
            Self::Claude => "--claude",
            Self::Codex => "--codex",
            Self::Cursor => "--cursor",
            Self::Omp => "--omp",
            Self::Grok => "--grok",
        }
    }
}

struct TrellisInitRequest {
    developer: String,
    platforms: Vec<TrellisPlatform>,
    auto_yes: bool,
}

pub fn project_init_command(auto_yes: bool) -> Result<()> {
    let root = std::env::current_dir().map_err(|error| {
        CcrError::FileIoError(format!("项目初始化阶段：无法获取当前工作目录: {error}"))
    })?;

    ColorOutput::title("项目工作流初始化");
    ColorOutput::key_value("目标目录", &root.display().to_string(), 2);
    println!();

    let git = ensure_git_repository(&root)?;
    println!();
    let request = resolve_trellis_request(auto_yes, &git, &root)?;
    run_trellis_init(&root, &request)?;
    println!();
    ensure_ref_dir(&root)?;
    ensure_project_gitignore(&root)?;

    println!();
    ColorOutput::separator();
    println!();
    ColorOutput::success("Git、Trellis、Agent 忽略规则和 ref/ 均已就绪");

    Ok(())
}

fn ensure_git_repository(root: &Path) -> Result<PathBuf> {
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
            ColorOutput::info("当前目录已经是 Git 仓库根，跳过 git init");
        } else {
            ColorOutput::warning(&format!(
                "当前目录位于 Git 仓库 {} 中，跳过嵌套 git init",
                repository_root.display()
            ));
        }
        return Ok(git);
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
    Ok(git)
}

fn resolve_trellis_request(auto_yes: bool, git: &Path, root: &Path) -> Result<TrellisInitRequest> {
    if auto_yes {
        return Ok(TrellisInitRequest {
            developer: read_git_user_name(git, root)?,
            platforms: TrellisPlatform::ALL.to_vec(),
            auto_yes: true,
        });
    }

    let platforms = prompt_platforms()?;
    let developer = prompt_required(DEVELOPER_PROMPT, DEVELOPER_HINT)?;
    Ok(TrellisInitRequest {
        developer,
        platforms,
        auto_yes: false,
    })
}

fn prompt_platforms() -> Result<Vec<TrellisPlatform>> {
    let labels = TrellisPlatform::ALL.map(TrellisPlatform::label);
    let defaults = [true; TrellisPlatform::ALL.len()];

    loop {
        // `defaults` applies to items already added, so `items` must come first.
        let selected = MultiSelect::new()
            .with_prompt(PLATFORM_PROMPT)
            .items(labels)
            .defaults(&defaults)
            .interact()
            .map_err(|error| CcrError::UiError(format!("平台选择阶段：交互失败: {error}")))?;

        if selected.is_empty() {
            ColorOutput::warning(EMPTY_PLATFORM_WARNING);
            continue;
        }

        return platforms_from_indices(&selected);
    }
}

fn platforms_from_indices(indices: &[usize]) -> Result<Vec<TrellisPlatform>> {
    let mut platforms = Vec::with_capacity(indices.len());
    for index in indices {
        let platform = TrellisPlatform::ALL.get(*index).copied().ok_or_else(|| {
            CcrError::ValidationError(format!("平台选择阶段：无效的平台下标 {index}"))
        })?;
        if !platforms.contains(&platform) {
            platforms.push(platform);
        }
    }
    Ok(platforms)
}

fn read_git_user_name(git: &Path, root: &Path) -> Result<String> {
    let output = Command::new(git)
        .args(["config", "user.name"])
        .current_dir(root)
        .output()
        .map_err(|error| {
            CcrError::ExternalCommandError(format!(
                "用户名阶段：无法启动 git config user.name: {error}"
            ))
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    developer_name_from_git_config(output.status.success(), &stdout)
}

fn developer_name_from_git_config(command_ok: bool, stdout: &str) -> Result<String> {
    let name = stdout.trim();
    if !command_ok || name.is_empty() {
        return Err(CcrError::ValidationError(MISSING_GIT_USER_MESSAGE.into()));
    }
    Ok(name.to_string())
}

fn run_trellis_init(root: &Path, request: &TrellisInitRequest) -> Result<()> {
    ColorOutput::step("初始化 Trellis 工作流");

    let trellis = resolve_tool("trellis", "Trellis")?;
    let args = trellis_init_args(request)?;
    let status = Command::new(trellis)
        .args(&args)
        .current_dir(root)
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

fn trellis_init_args(request: &TrellisInitRequest) -> Result<Vec<String>> {
    let developer = request.developer.trim();
    if developer.is_empty() {
        return Err(CcrError::ValidationError(
            "Trellis 阶段：开发者用户名不能为空".into(),
        ));
    }
    if request.platforms.is_empty() {
        return Err(CcrError::ValidationError(
            "Trellis 阶段：至少选择一个平台".into(),
        ));
    }

    let mut args = vec!["init".to_string(), "-u".to_string(), developer.to_string()];
    for platform in TrellisPlatform::ALL {
        if request.platforms.contains(&platform) {
            args.push(platform.flag().to_string());
        }
    }
    if request.auto_yes {
        args.push("--yes".to_string());
    }
    Ok(args)
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

fn ensure_ref_dir(root: &Path) -> Result<()> {
    ColorOutput::step("准备 ref/ 目录");
    let path = root.join("ref");
    if path.is_dir() {
        ColorOutput::info("ref/ 目录已存在，保留现有内容");
        return Ok(());
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
    Ok(())
}

fn ensure_project_gitignore(root: &Path) -> Result<()> {
    ColorOutput::step("更新 .gitignore 中的项目忽略规则");
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
        ColorOutput::info(".gitignore 已包含全部项目忽略规则，跳过写入");
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

    ColorOutput::success(".gitignore 已包含 .agents/、.claude/、.codex/ 和 ref/");
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

    fn request(
        developer: &str,
        platforms: &[TrellisPlatform],
        auto_yes: bool,
    ) -> TrellisInitRequest {
        TrellisInitRequest {
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
        let args =
            trellis_init_args(&request("  Ada Lovelace  ", &TrellisPlatform::ALL, false)).unwrap();

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
            TrellisPlatform::Grok,
            TrellisPlatform::Claude,
            TrellisPlatform::Omp,
            TrellisPlatform::Codex,
        ];
        let args = trellis_init_args(&request("Ada", &platforms, false)).unwrap();

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
        let empty_platforms = trellis_init_args(&request("Ada", &[], false)).unwrap_err();
        assert!(matches!(empty_platforms, CcrError::ValidationError(_)));

        let blank_name =
            trellis_init_args(&request("   ", &TrellisPlatform::ALL, true)).unwrap_err();
        assert!(matches!(blank_name, CcrError::ValidationError(_)));
    }

    #[test]
    fn trellis_args_put_yes_last() {
        let args = trellis_init_args(&request("Ada", &TrellisPlatform::ALL, true)).unwrap();

        assert_args(
            &args,
            &[
                "init", "-u", "Ada", "--claude", "--codex", "--cursor", "--omp", "--grok", "--yes",
            ],
        );
        assert_eq!(args.last().map(String::as_str), Some("--yes"));
    }

    #[test]
    fn git_user_name_trims_surrounding_whitespace() {
        assert_eq!(
            developer_name_from_git_config(true, "  Ada Lovelace \r\n").unwrap(),
            "Ada Lovelace"
        );
    }

    #[test]
    fn git_user_name_rejects_failed_or_blank_output() {
        for (command_ok, stdout) in [(false, "Ada\n"), (true, " \n"), (true, "")] {
            let error = developer_name_from_git_config(command_ok, stdout).unwrap_err();
            let message = error.to_string();
            assert!(matches!(error, CcrError::ValidationError(_)));
            assert_eq!(error.exit_code(), 90);
            assert!(message.contains("git config user.name"), "{message}");
            assert!(message.contains("-y"), "{message}");
        }
    }

    #[test]
    fn merge_creates_rules_for_empty_file() {
        assert_eq!(
            merge_project_ignore_rules(""),
            Some(".agents/\n.claude/\n.codex/\nref/\n".into())
        );
    }

    #[test]
    fn merge_preserves_existing_content_and_adds_line_boundary() {
        assert_eq!(
            merge_project_ignore_rules("target/"),
            Some("target/\n.agents/\n.claude/\n.codex/\nref/\n".into())
        );
    }

    #[test]
    fn merge_only_adds_missing_rules() {
        assert_eq!(
            merge_project_ignore_rules("target/\n.agents/\n.codex/\n"),
            Some("target/\n.agents/\n.codex/\n.claude/\nref/\n".into())
        );
    }

    #[test]
    fn merge_appends_ref_when_agent_rules_already_exist() {
        assert_eq!(
            merge_project_ignore_rules(".agents/\n.claude/\n.codex/\n"),
            Some(".agents/\n.claude/\n.codex/\nref/\n".into())
        );
    }

    #[test]
    fn merge_preserves_crlf_line_endings() {
        assert_eq!(
            merge_project_ignore_rules("target/\r\n.agents/\r\n"),
            Some("target/\r\n.agents/\r\n.claude/\r\n.codex/\r\nref/\r\n".into())
        );
    }

    #[test]
    fn merge_is_unchanged_when_all_rules_exist_with_surrounding_whitespace() {
        let existing = "target/\n  .agents/  \n.claude/\n.codex/\n  ref/  \n";
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
