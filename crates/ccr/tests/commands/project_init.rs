#![allow(clippy::unwrap_used)]

use filetime::FileTime;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tempfile::TempDir;

const GIT_USER: &str = "fake-git-user";
const TRELLIS_ARGS: &str = "|init -u fake-git-user --claude --codex --cursor --omp --grok --yes";
const GITIGNORE_RULES: &str = ".agents/\n.claude/\n.codex/\nref/\n";
const SKIP_GITIGNORE_WRITE: &str = ".gitignore 已包含全部项目忽略规则，跳过写入";
const READY_SUMMARY: &str = "Git、Trellis、Agent 忽略规则和 ref/ 均已就绪";

struct ProjectInitFixture {
    _temp: TempDir,
    root: PathBuf,
    project: PathBuf,
    bin: PathBuf,
    log: PathBuf,
}

impl ProjectInitFixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().to_path_buf();
        let project = root.join("project");
        let bin = root.join("bin");
        let log = root.join("calls.log");
        fs::create_dir_all(&project).unwrap();
        fs::create_dir_all(&bin).unwrap();

        Self {
            _temp: temp,
            root,
            project,
            bin,
            log,
        }
    }

    fn in_parent_repository() -> Self {
        let fixture = Self::new();
        let nested = fixture.root.join("repository/nested/project");
        fs::create_dir_all(&nested).unwrap();
        Self {
            project: nested,
            ..fixture
        }
    }

    fn install_git(&self) {
        install_fake_executable(&self.bin, "git", fake_git_script());
    }

    fn install_trellis(&self) {
        install_fake_executable(&self.bin, "trellis", fake_trellis_script());
    }

    fn install_tools(&self) {
        self.install_git();
        self.install_trellis();
    }

    fn command(&self, git_mode: &str, trellis_mode: &str) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_ccr"));
        command
            .current_dir(&self.project)
            .env("CCR_ROOT", self.root.join("ccr"))
            .env("PATH", &self.bin)
            .env("FAKE_GIT_MODE", git_mode)
            .env("FAKE_TRELLIS_MODE", trellis_mode)
            .env("FAKE_GIT_ROOT", self.root.join("repository"))
            .env("CCR_PROJECT_INIT_LOG", &self.log)
            .env("NO_COLOR", "1")
            .env("CLICOLOR", "0")
            .env_remove("FAKE_GIT_USER");
        command
    }

    fn run(
        &self,
        git_mode: &str,
        trellis_mode: &str,
        git_user: Option<&str>,
        args: &[&str],
    ) -> Output {
        let mut command = self.command(git_mode, trellis_mode);
        if let Some(user) = git_user {
            command.env("FAKE_GIT_USER", user);
        }
        command.args(args).output().unwrap()
    }

    fn calls(&self) -> Vec<String> {
        fs::read_to_string(&self.log)
            .unwrap_or_default()
            .lines()
            .map(str::to_owned)
            .collect()
    }
}

#[test]
fn initializes_new_repository_then_trellis_and_gitignore() {
    let fixture = ProjectInitFixture::new();
    fixture.install_tools();

    let output = fixture.run(
        "none",
        "success",
        Some(GIT_USER),
        &["--yes", "project", "init"],
    );

    assert_success(&output);
    let calls = fixture.calls();
    assert_eq!(calls.len(), 4, "calls: {calls:?}");
    assert!(calls[0].ends_with("|rev-parse --show-toplevel"));
    assert!(calls[1].ends_with("|init"));
    assert!(calls[2].ends_with("|config user.name"));
    assert!(calls[3].starts_with("trellis|"));
    assert!(calls[3].ends_with(TRELLIS_ARGS));
    assert_calls_run_in(&calls, &fixture.project);
    assert_eq!(
        fs::read_to_string(fixture.project.join(".gitignore")).unwrap(),
        GITIGNORE_RULES
    );
    assert!(fixture.project.join("ref").is_dir());
    assert!(fixture.project.join(".git").is_dir());
    assert!(fixture.project.join(".trellis/workflow.md").is_file());
    assert!(fixture.project.join(".trellis/scripts/task.py").is_file());
    assert!(stdout(&output).contains(READY_SUMMARY));
}

#[test]
fn skips_git_init_at_existing_repository_root() {
    let fixture = ProjectInitFixture::new();
    fixture.install_tools();

    let output = fixture.run(
        "root",
        "success",
        Some(GIT_USER),
        &["--yes", "project", "init"],
    );

    assert_success(&output);
    assert!(!git_init_was_called(&fixture.calls()));
    assert!(stdout(&output).contains("已经是 Git 仓库根"));
    assert!(fixture.project.join(".gitignore").is_file());
    assert!(fixture.project.join("ref").is_dir());
}

#[test]
fn preserves_parent_repository_boundary_but_initializes_current_directory() {
    let fixture = ProjectInitFixture::in_parent_repository();
    fixture.install_tools();

    let output = fixture.run(
        "parent",
        "success",
        Some(GIT_USER),
        &["--yes", "project", "init"],
    );

    assert_success(&output);
    assert!(!git_init_was_called(&fixture.calls()));
    assert!(stdout(&output).contains("跳过嵌套 git init"));
    assert!(stdout(&output).contains(&fixture.root.join("repository").display().to_string()));
    assert!(!fixture.project.join(".git").exists());
    assert!(fixture.project.join(".trellis/workflow.md").is_file());
    assert!(fixture.project.join(".gitignore").is_file());
    assert!(fixture.project.join("ref").is_dir());
    assert_calls_run_in(&fixture.calls(), &fixture.project);
}

#[test]
fn auto_yes_uses_git_user_and_five_platforms() {
    let fixture = ProjectInitFixture::new();
    fixture.install_tools();

    let output = fixture.run(
        "root",
        "success",
        Some(GIT_USER),
        &["--yes", "project", "init"],
    );

    assert_success(&output);
    let calls = fixture.calls();
    assert_eq!(calls.len(), 3, "calls: {calls:?}");
    assert!(calls[0].ends_with("|rev-parse --show-toplevel"));
    assert!(calls[1].ends_with("|config user.name"));
    assert!(calls[2].ends_with(TRELLIS_ARGS));
    assert!(
        calls
            .iter()
            .filter(|call| call.starts_with("git|"))
            .all(|call| !call.contains("--yes"))
    );
    assert_eq!(
        fs::read_to_string(fixture.project.join(".gitignore")).unwrap(),
        GITIGNORE_RULES
    );
    assert!(fixture.project.join("ref").is_dir());
}

#[test]
fn missing_git_user_stops_before_trellis() {
    let fixture = ProjectInitFixture::new();
    fixture.install_tools();

    let output = fixture.run("none", "success", None, &["--yes", "project", "init"]);

    assert_eq!(output.status.code(), Some(90));
    let error = stderr(&output);
    assert!(error.contains("git config user.name"), "{error}");
    assert!(error.contains("-y"), "{error}");
    let calls = fixture.calls();
    assert!(calls.iter().any(|call| call.ends_with("|config user.name")));
    assert!(!calls.iter().any(|call| call.starts_with("trellis|")));
    assert!(!stdout(&output).contains("选择要初始化的平台"));
    assert!(!stdout(&output).contains("均已就绪"));
    assert!(!fixture.project.join("ref").exists());
    assert!(!fixture.project.join(".gitignore").exists());
}

#[test]
fn git_init_failure_stops_before_trellis_and_gitignore() {
    let fixture = ProjectInitFixture::new();
    fixture.install_tools();

    let output = fixture.run("fail", "success", Some(GIT_USER), &["project", "init"]);

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(75));
    assert!(stderr(&output).contains("Git 阶段"));
    assert!(!stdout(&output).contains("选择要初始化的平台"));
    let calls = fixture.calls();
    assert!(!calls.iter().any(|call| call.starts_with("trellis|")));
    assert!(!calls.iter().any(|call| call.contains("config user.name")));
    assert!(!fixture.project.join(".gitignore").exists());
    assert!(!fixture.project.join("ref").exists());
}

#[test]
fn missing_git_stops_before_trellis() {
    let fixture = ProjectInitFixture::new();
    fixture.install_trellis();

    let output = fixture.run("root", "success", Some(GIT_USER), &["project", "init"]);

    assert_eq!(output.status.code(), Some(75));
    assert!(stderr(&output).contains("Git 阶段"));
    assert!(!stdout(&output).contains("选择要初始化的平台"));
    assert!(fixture.calls().is_empty());
    assert!(!fixture.project.join("ref").exists());
    assert!(!fixture.project.join(".gitignore").exists());
}

#[test]
fn trellis_failure_keeps_git_result_and_skips_gitignore() {
    let fixture = ProjectInitFixture::new();
    fixture.install_tools();

    let output = fixture.run(
        "none",
        "fail",
        Some(GIT_USER),
        &["--yes", "project", "init"],
    );

    assert_eq!(output.status.code(), Some(75));
    assert!(stderr(&output).contains("Trellis 阶段"));
    assert!(
        fixture
            .calls()
            .iter()
            .any(|call| call.starts_with("trellis|"))
    );
    assert!(fixture.project.join(".git").is_dir());
    assert!(!fixture.project.join(".gitignore").exists());
    assert!(!fixture.project.join("ref").exists());
    assert!(!stdout(&output).contains("均已就绪"));
}

#[test]
fn missing_trellis_is_reported_after_git_is_ready() {
    let fixture = ProjectInitFixture::new();
    fixture.install_git();

    let output = fixture.run(
        "root",
        "success",
        Some(GIT_USER),
        &["--yes", "project", "init"],
    );

    assert_eq!(output.status.code(), Some(75));
    assert!(stderr(&output).contains("Trellis 阶段"));
    assert!(!fixture.project.join(".gitignore").exists());
    assert!(!fixture.project.join("ref").exists());
}

#[test]
fn rejects_trellis_zero_exit_without_minimum_workflow() {
    let fixture = ProjectInitFixture::new();
    fixture.install_tools();

    let output = fixture.run(
        "root",
        "missing",
        Some(GIT_USER),
        &["--yes", "project", "init"],
    );

    assert_eq!(output.status.code(), Some(90));
    assert!(stderr(&output).contains("缺少最低工作流文件"));
    assert!(
        fixture
            .calls()
            .iter()
            .any(|call| call.starts_with("trellis|"))
    );
    assert!(!fixture.project.join(".gitignore").exists());
    assert!(!fixture.project.join("ref").exists());
    assert!(!stdout(&output).contains("均已就绪"));
}

#[test]
fn preserves_existing_gitignore_and_does_not_rewrite_when_complete() {
    let fixture = ProjectInitFixture::new();
    fixture.install_tools();
    let gitignore = fixture.project.join(".gitignore");
    fs::write(&gitignore, "target/\n.agents/\n").unwrap();

    let first = fixture.run(
        "root",
        "success",
        Some(GIT_USER),
        &["--yes", "project", "init"],
    );
    assert_success(&first);
    assert_eq!(
        fs::read_to_string(&gitignore).unwrap(),
        "target/\n.agents/\n.claude/\n.codex/\nref/\n"
    );
    assert!(fixture.project.join("ref").is_dir());

    let fixed_time = FileTime::from_unix_time(1_700_000_000, 0);
    filetime::set_file_mtime(&gitignore, fixed_time).unwrap();
    let second = fixture.run(
        "root",
        "success",
        Some(GIT_USER),
        &["--yes", "project", "init"],
    );

    assert_success(&second);
    assert_eq!(
        FileTime::from_last_modification_time(&fs::metadata(&gitignore).unwrap()),
        fixed_time
    );
    assert!(stdout(&second).contains(SKIP_GITIGNORE_WRITE));
}

#[test]
fn keeps_existing_ref_directory_contents() {
    let fixture = ProjectInitFixture::new();
    fixture.install_tools();
    let keep = fixture.project.join("ref/keep.txt");
    fs::create_dir(fixture.project.join("ref")).unwrap();
    fs::write(&keep, "keep-me").unwrap();

    let output = fixture.run(
        "root",
        "success",
        Some(GIT_USER),
        &["--yes", "project", "init"],
    );

    assert_success(&output);
    assert_eq!(fs::read_to_string(&keep).unwrap(), "keep-me");
    assert!(stdout(&output).contains("ref/ 目录已存在，保留现有内容"));
    assert!(
        fs::read_to_string(fixture.project.join(".gitignore"))
            .unwrap()
            .contains("ref/\n")
    );
}

#[test]
fn ref_file_conflict_fails_after_trellis_without_gitignore_write() {
    let fixture = ProjectInitFixture::new();
    fixture.install_tools();
    let gitignore = fixture.project.join(".gitignore");
    fs::write(&gitignore, "keep\n").unwrap();
    fs::write(fixture.project.join("ref"), "not-a-dir").unwrap();
    let fixed_time = FileTime::from_unix_time(1_700_000_000, 0);
    filetime::set_file_mtime(&gitignore, fixed_time).unwrap();

    let output = fixture.run(
        "root",
        "success",
        Some(GIT_USER),
        &["--yes", "project", "init"],
    );

    assert_eq!(output.status.code(), Some(51));
    assert!(stderr(&output).contains("ref/ 阶段"));
    assert!(!stderr(&output).contains(".gitignore 阶段"));
    assert!(fixture.project.join(".trellis/workflow.md").is_file());
    assert!(
        fixture
            .calls()
            .iter()
            .any(|call| call.starts_with("trellis|"))
    );
    assert_eq!(fs::read_to_string(&gitignore).unwrap(), "keep\n");
    assert_eq!(
        FileTime::from_last_modification_time(&fs::metadata(&gitignore).unwrap()),
        fixed_time
    );
    assert_eq!(
        fs::read_to_string(fixture.project.join("ref")).unwrap(),
        "not-a-dir"
    );
    assert!(!stdout(&output).contains("均已就绪"));
}

#[test]
fn complete_gitignore_still_creates_missing_ref_directory() {
    let fixture = ProjectInitFixture::new();
    fixture.install_tools();
    let gitignore = fixture.project.join(".gitignore");
    let original = "notes\n.agents/\n.claude/\n.codex/\nref/\n";
    fs::write(&gitignore, original).unwrap();
    let fixed_time = FileTime::from_unix_time(1_700_000_000, 0);
    filetime::set_file_mtime(&gitignore, fixed_time).unwrap();
    assert!(!fixture.project.join("ref").exists());

    let output = fixture.run(
        "root",
        "success",
        Some(GIT_USER),
        &["--yes", "project", "init"],
    );

    assert_success(&output);
    assert!(fixture.project.join("ref").is_dir());
    assert_eq!(fs::read_to_string(&gitignore).unwrap(), original);
    assert_eq!(
        FileTime::from_last_modification_time(&fs::metadata(&gitignore).unwrap()),
        fixed_time
    );
    assert!(stdout(&output).contains(SKIP_GITIGNORE_WRITE));
}

#[test]
fn gitignore_read_failure_reports_file_stage_after_trellis() {
    let fixture = ProjectInitFixture::new();
    fixture.install_tools();
    fs::create_dir(fixture.project.join(".gitignore")).unwrap();

    let output = fixture.run(
        "root",
        "success",
        Some(GIT_USER),
        &["--yes", "project", "init"],
    );

    assert_eq!(output.status.code(), Some(51));
    assert!(stderr(&output).contains(".gitignore 阶段"));
    assert!(fixture.project.join(".trellis/workflow.md").is_file());
    assert!(fixture.project.join("ref").is_dir());
    assert!(!stdout(&output).contains("均已就绪"));
}

fn git_init_was_called(calls: &[String]) -> bool {
    calls
        .iter()
        .any(|call| call.starts_with("git|") && call.ends_with("|init"))
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        stdout(output),
        stderr(output)
    );
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn assert_calls_run_in(calls: &[String], expected: &Path) {
    let expected = fs::canonicalize(expected).unwrap();
    for call in calls {
        let cwd = call.split('|').nth(1).expect("fake command cwd field");
        assert_eq!(fs::canonicalize(cwd).unwrap(), expected, "call: {call}");
    }
}

#[cfg(windows)]
fn install_fake_executable(bin: &Path, name: &str, script: &str) {
    fs::write(bin.join(format!("{name}.cmd")), script).unwrap();
}

#[cfg(unix)]
fn install_fake_executable(bin: &Path, name: &str, script: &str) {
    use std::os::unix::fs::PermissionsExt;

    let path = bin.join(name);
    fs::write(&path, script).unwrap();
    let mut permissions = fs::metadata(&path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).unwrap();
}

#[cfg(windows)]
fn fake_git_script() -> &'static str {
    r#"@echo off
>>"%CCR_PROJECT_INIT_LOG%" echo git^|%CD%^|%*
if /I "%1"=="rev-parse" goto revparse
if /I "%1"=="init" goto init
if /I "%1"=="config" goto config
exit /b 2

:revparse
if /I "%FAKE_GIT_MODE%"=="root" (
  echo %CD%
  exit /b 0
)
if /I "%FAKE_GIT_MODE%"=="parent" (
  echo %FAKE_GIT_ROOT%
  exit /b 0
)
exit /b 128

:init
if /I "%FAKE_GIT_MODE%"=="fail" exit /b 7
mkdir ".git" >nul 2>nul
exit /b 0

:config
if /I "%2"=="user.name" goto username
exit /b 2

:username
if not defined FAKE_GIT_USER exit /b 1
if "%FAKE_GIT_USER%"=="" exit /b 1
echo %FAKE_GIT_USER%
exit /b 0
"#
}

#[cfg(unix)]
fn fake_git_script() -> &'static str {
    r#"#!/bin/sh
printf 'git|%s|%s\n' "$PWD" "$*" >> "$CCR_PROJECT_INIT_LOG"
if [ "$1" = "rev-parse" ]; then
  case "$FAKE_GIT_MODE" in
    root) printf '%s\n' "$PWD"; exit 0 ;;
    parent) printf '%s\n' "$FAKE_GIT_ROOT"; exit 0 ;;
    *) exit 128 ;;
  esac
fi
if [ "$1" = "init" ]; then
  [ "$FAKE_GIT_MODE" = "fail" ] && exit 7
  /bin/mkdir -p .git
  exit 0
fi
if [ "$1" = "config" ]; then
  if [ "$2" != "user.name" ]; then
    exit 2
  fi
  if [ -n "$FAKE_GIT_USER" ]; then
    printf '%s\n' "$FAKE_GIT_USER"
    exit 0
  fi
  exit 1
fi
exit 2
"#
}

#[cfg(windows)]
fn fake_trellis_script() -> &'static str {
    r#"@echo off
>>"%CCR_PROJECT_INIT_LOG%" echo trellis^|%CD%^|%*
if /I "%FAKE_TRELLIS_MODE%"=="fail" exit /b 8
if /I "%FAKE_TRELLIS_MODE%"=="missing" exit /b 0
mkdir ".trellis\scripts" >nul 2>nul
> ".trellis\workflow.md" echo workflow
> ".trellis\scripts\task.py" echo script
exit /b 0
"#
}

#[cfg(unix)]
fn fake_trellis_script() -> &'static str {
    r#"#!/bin/sh
printf 'trellis|%s|%s\n' "$PWD" "$*" >> "$CCR_PROJECT_INIT_LOG"
[ "$FAKE_TRELLIS_MODE" = "fail" ] && exit 8
[ "$FAKE_TRELLIS_MODE" = "missing" ] && exit 0
/bin/mkdir -p .trellis/scripts
printf 'workflow\n' > .trellis/workflow.md
printf 'script\n' > .trellis/scripts/task.py
exit 0
"#
}
