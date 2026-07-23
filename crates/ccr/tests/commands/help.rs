#![allow(clippy::unwrap_used)]

use std::process::Command;

fn run_help(args: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_ccr"))
        .args(args)
        .env("NO_COLOR", "1")
        .env("CLICOLOR", "0")
        .env("COLUMNS", "120")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "command failed: {:?}\nstdout:\n{}\nstderr:\n{}",
        args,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn root_help_includes_guided_tasks() {
    let stdout = run_help(&["--help"]);

    assert!(stdout.contains("常用任务:"));
    assert!(stdout.contains("切换平台"));
    assert!(stdout.contains("切换 Codex Auth"));
    assert!(stdout.contains("把 Codex 订阅导入 OpenCode"));
    assert!(stdout.contains("初始化当前项目"));
    assert!(stdout.contains("project"));
}

#[test]
fn help_subcommand_matches_root_help() {
    let direct_help = run_help(&["--help"]);
    let help_command = run_help(&["help"]);

    assert_eq!(help_command, direct_help);
}

#[test]
fn help_subcommand_supports_nested_codex_auth_path() {
    let direct_help = run_help(&["codex", "auth", "--help"]);
    let help_command = run_help(&["help", "codex", "auth"]);

    assert_eq!(help_command, direct_help);
    assert!(help_command.contains("ccr.exe codex auth") || help_command.contains("ccr codex auth"));
    assert!(help_command.contains("cli_auth_credentials_store = file"));
}

#[test]
fn codex_fix_help_exposes_explicit_runtime_repair() {
    let stdout = run_help(&["codex", "fix", "--help"]);

    assert!(stdout.contains("--repair-runtime"));
    assert!(stdout.contains("--dry-run"));
    assert!(stdout.contains("显式重放当前 CCR profile"));
}

#[test]
fn help_subcommand_supports_platform_path() {
    let direct_help = run_help(&["platform", "--help"]);
    let help_command = run_help(&["help", "platform"]);

    assert_eq!(help_command, direct_help);
    assert!(help_command.contains("ccr platform list"));
    assert!(help_command.contains("ccr platform switch codex"));
    assert!(help_command.contains("ccr platform current"));
}

#[test]
fn help_subcommand_supports_clean_path() {
    let direct_help = run_help(&["clean", "--help"]);
    let help_command = run_help(&["help", "clean"]);

    assert_eq!(help_command, direct_help);
    assert!(help_command.contains("打开菜单: ccr clean"));
    assert!(help_command.contains("ccr clean planfiles --dry-run"));
    assert!(help_command.contains("ccr clean planfiles --all --dry-run"));
    assert!(help_command.contains("ccr clean --all"));
    assert!(help_command.contains("ccr clean backups --dry-run"));
}

#[test]
fn help_subcommand_supports_project_init_path() {
    let project_help = run_help(&["project", "--help"]);
    assert!(project_help.contains("Git"));
    assert!(project_help.contains("Trellis"));
    assert!(project_help.contains(".gitignore"));

    let direct_help = run_help(&["project", "init", "--help"]);
    let help_command = run_help(&["help", "project", "init"]);
    assert_eq!(help_command, direct_help);
    assert!(help_command.contains("git init"));
    assert!(help_command.contains("trellis init"));
    assert!(help_command.contains(".agents/"));
    assert!(help_command.contains("--yes"));
}

#[test]
fn legacy_init_help_remains_user_configuration_command() {
    let stdout = run_help(&["init", "--help"]);

    assert!(stdout.contains("初始化配置文件"));
    assert!(stdout.contains("--force"));
    assert!(!stdout.contains("trellis init"));
}

#[test]
fn bare_project_requires_an_explicit_action() {
    let output = Command::new(env!("CARGO_BIN_EXE_ccr"))
        .arg("project")
        .env("NO_COLOR", "1")
        .env("CLICOLOR", "0")
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Usage"));
}

#[test]
fn opencode_auth_help_includes_preview_and_boundary() {
    let stdout = run_help(&["opencode", "auth", "--help"]);

    assert!(stdout.contains("import-codex --dry-run"));
    assert!(stdout.contains("只迁移 ChatGPT OAuth 账号"));
    assert!(stdout.contains("API key / provider 账号会跳过"));
}

#[test]
fn version_flag_returns_short_version_output() {
    let stdout = run_help(&["--version"]);

    assert!(stdout.contains(env!("CARGO_PKG_VERSION")));
    assert!(!stdout.contains("常用入口"));
}

#[test]
fn version_subcommand_reports_short_flag_usage_hint() {
    let stdout = run_help(&["version", "--help"]);

    assert!(stdout.contains("ccr --version"));
    assert!(stdout.contains("ccr -V"));
}
