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
    assert!(stdout.contains("查看平台与 Profile"));
    assert!(stdout.contains("ccr current"));
    assert!(stdout.contains("ccr platform list"));
    assert!(stdout.contains("ccr claude profile --help"));
    assert!(stdout.contains("ccr codex profile --help"));
    assert!(stdout.contains("ccr grok profile --help"));
    for retired in [
        "ccr platform switch",
        "ccr platform current",
        "ccr platform info",
        "ccr platform init",
        "ccr platform profile",
    ] {
        assert!(
            !stdout.contains(retired),
            "root help contains {retired}: {stdout}"
        );
    }
    assert!(stdout.contains("切换 Codex Auth"));
    assert!(stdout.contains("登出官方运行时登录"));
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
    assert!(stdout.contains("--doctor"));
    assert!(stdout.contains("显式重放当前 CCR profile"));
    assert!(stdout.contains("默认只做本地进程清理与 CCR profile/runtime 诊断，不调用上游"));
}

#[test]
fn help_subcommand_supports_platform_path() {
    let direct_help = run_help(&["platform", "--help"]);
    let help_command = run_help(&["help", "platform"]);

    assert_eq!(help_command, direct_help);
    assert!(help_command.contains("ccr platform list"));
    assert!(help_command.contains("ccr current"));
    assert!(help_command.contains("ccr claude profile --help"));
    assert!(help_command.contains("ccr codex profile --help"));
    assert!(help_command.contains("ccr grok profile --help"));

    for retired in ["switch", "current", "info", "init", "profile"] {
        assert!(
            !help_command.lines().any(|line| {
                line.trim_start()
                    .strip_prefix(retired)
                    .is_some_and(|rest| rest.starts_with(char::is_whitespace))
            }),
            "platform help exposes retired `{retired}` command: {help_command}"
        );
        assert!(
            !help_command.contains(&format!("ccr platform {retired}")),
            "platform help recommends retired `{retired}` command: {help_command}"
        );
    }
}

#[test]
fn legacy_platform_init_grok_remains_parseable_and_returns_migration_error() {
    let temp_dir = tempfile::tempdir().unwrap();
    let lock_dir = temp_dir.path().join("locks");
    std::fs::create_dir_all(&lock_dir).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_ccr"))
        .args(["platform", "init", "grok"])
        .env("HOME", temp_dir.path())
        .env("USERPROFILE", temp_dir.path())
        .env("CCR_ROOT", temp_dir.path())
        .env("CCR_LOCK_DIR", lock_dir)
        .env("NO_COLOR", "1")
        .env("CLICOLOR", "0")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("legacy command retired"), "{stderr}");
    assert!(stderr.contains("`ccr platform init`"), "{stderr}");
    assert!(stderr.contains("`ccr claude profile init`"), "{stderr}");
    assert!(stderr.contains("`ccr codex profile init`"), "{stderr}");
    assert!(stderr.contains("`ccr grok profile init`"), "{stderr}");
}

#[test]
fn platform_profile_help_lists_init_for_supported_platforms() {
    for platform in ["claude", "codex", "grok"] {
        let stdout = run_help(&[platform, "profile", "--help"]);
        assert!(
            stdout
                .lines()
                .any(|line| line.trim_start().starts_with("init")),
            "{platform} profile help does not list init: {stdout}"
        );

        let direct_help_action = run_help(&[platform, "profile", "help"]);
        assert_eq!(direct_help_action, stdout);

        let help_command = run_help(&["help", platform, "profile"]);
        assert_eq!(help_command, stdout);
    }
}

#[test]
fn initialized_ccr_init_output_does_not_recommend_retired_platform_init() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path().join(".ccr");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("config.toml"), "[claude]\nenabled = true\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_ccr"))
        .arg("init")
        .env("CCR_ROOT", &root)
        .env("HOME", temp_dir.path())
        .env("USERPROFILE", temp_dir.path())
        .env("NO_COLOR", "1")
        .env("CLICOLOR", "0")
        .output()
        .unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("ccr platform init"), "{stdout}");
    assert!(stdout.contains("profile init"), "{stdout}");
}

#[test]
fn platform_not_found_message_recommends_supported_profile_init_commands() {
    let message = ccr_core::CcrError::PlatformNotFound("unknown".to_string()).user_message();

    assert!(!message.contains("ccr platform init"), "{message}");
    assert!(message.contains("ccr claude profile init"), "{message}");
    assert!(message.contains("ccr codex profile init"), "{message}");
    assert!(message.contains("ccr grok profile init"), "{message}");
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
fn grok_auth_help_includes_logout_boundary() {
    let stdout = run_help(&["grok", "auth", "--help"]);

    assert!(stdout.contains("ccr grok auth off"));
    assert!(stdout.contains("auth.json"));
    assert!(
        !stdout.contains("mcp_credentials.json")
            || stdout.contains("不读取或写入 mcp_credentials.json")
    );
}

#[test]
fn version_subcommand_omits_opencode_entry() {
    let stdout = run_help(&["version"]);

    assert!(!stdout.contains("opencode"));
    assert!(stdout.contains("ccr grok auth"));
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
