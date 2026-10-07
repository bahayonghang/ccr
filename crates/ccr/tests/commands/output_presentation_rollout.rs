#![allow(clippy::unwrap_used)]

use ccr_cli::platforms::PlatformRegistry;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::SystemTime;
use tempfile::TempDir;

struct Fixture(TempDir);

impl Fixture {
    fn new() -> Self {
        let fixture = Self(tempfile::tempdir().unwrap());
        for path in ["ccr/platforms/claude", "claude", "codex", "grok", "work"] {
            fs::create_dir_all(fixture.0.path().join(path)).unwrap();
        }
        fixture.write("ccr/config.toml", "[claude]\nenabled = true\n");
        fixture
    }

    fn write(&self, path: &str, content: &str) {
        fs::write(self.0.path().join(path), content).unwrap();
    }

    fn command(&self, args: &[&str]) -> Command {
        let home = self.0.path();
        let mut command = Command::new(env!("CARGO_BIN_EXE_ccr"));
        for key in [
            "CCR_CONFIG_PATH",
            "CLICOLOR_FORCE",
            "FORCE_COLOR",
            "OPENAI_API_KEY",
            "ANTHROPIC_API_KEY",
            "ANTHROPIC_AUTH_TOKEN",
            "CLAUDE_CODE_OAUTH_TOKEN",
            "CLAUDE_CODE_USE_BEDROCK",
            "CLAUDE_CODE_USE_VERTEX",
            "CLAUDE_CODE_USE_FOUNDRY",
            "XAI_API_KEY",
        ] {
            command.env_remove(key);
        }
        command
            .args(args)
            .current_dir(home.join("work"))
            .env("HOME", home)
            .env("USERPROFILE", home)
            .env("CCR_ROOT", home.join("ccr"))
            .env("CCR_DATA_DIR", home.join("ccr"))
            .env("CCR_LOCK_DIR", home.join("locks"))
            .env("CCR_CODEX_DIR", home.join("codex"))
            .env("CODEX_HOME", home.join("codex"))
            .env("CLAUDE_CONFIG_DIR", home.join("claude"))
            .env("CLAUDE_JSON_PATH", home.join("claude-state.json"))
            .env("CCR_SETTINGS_PATH", home.join("claude/settings.json"))
            .env("CCR_BACKUP_DIR", home.join("backups"))
            .env("GROK_HOME", home.join("grok"))
            .env("NO_COLOR", "1")
            .env("CLICOLOR", "0")
            .env("CCR_LOG_LEVEL", "off");
        command
    }

    fn run(&self, args: &[&str]) -> Output {
        let output = self.command(args).output().unwrap();
        for bytes in [&output.stdout, &output.stderr] {
            let output = text(bytes);
            assert!(
                !output.contains('\u{1b}'),
                "ANSI in captured output: {output}"
            );
            for label in [
                "[OK]", "[INFO]", "[WARN]", "[ERR]", "[STEP]", "[FAIL]", "[SKIP]",
            ] {
                assert!(!output.contains(label), "legacy label {label}: {output}");
            }
        }
        output
    }
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec()).unwrap()
}

fn inventory(root: &Path) -> BTreeMap<PathBuf, (Option<Vec<u8>>, SystemTime)> {
    let mut entries = BTreeMap::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(directory).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let metadata = fs::metadata(&path).unwrap();
            let bytes = if metadata.is_dir() {
                pending.push(path.clone());
                None
            } else {
                Some(fs::read(&path).unwrap())
            };
            entries.insert(
                path.strip_prefix(root).unwrap().to_path_buf(),
                (bytes, metadata.modified().unwrap()),
            );
        }
    }
    entries
}

#[test]
fn doctor_maps_every_report_status_to_stdout_without_writes() {
    let f = Fixture::new();
    f.write("claude/settings.json", "{}");
    f.write(
        "ccr/platforms/claude/profiles.toml",
        "default_config = ''\ncurrent_config = ''\n",
    );
    for (runtime, expected_exit) in [("{}", 0), ("invalid synthetic JSON", 1)] {
        f.write("claude/settings.json", runtime);
        let before = inventory(f.0.path());
        let machine = f.run(&["doctor", "--platform", "claude", "--json"]);
        let human = f.run(&["doctor", "--platform", "claude", "--verbose"]);
        assert_eq!(
            machine.status.code(),
            Some(expected_exit),
            "{}",
            text(&machine.stdout)
        );
        assert_eq!(human.status.code(), Some(expected_exit));
        assert!(machine.stderr.is_empty());
        assert!(human.stderr.is_empty());
        let report: Value = serde_json::from_slice(&machine.stdout).unwrap();
        assert_eq!(
            report
                .as_object()
                .unwrap()
                .keys()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            ["checks", "online", "scope", "summary"]
        );
        assert_eq!(report["online"], false);
        assert!(report["summary"]["warnings"].as_u64().unwrap() > 0);
        assert!(report["summary"]["skipped"].as_u64().unwrap() > 0);
        let stdout = text(&human.stdout);
        for check in report["checks"].as_array().unwrap() {
            let prefix = match check["status"].as_str().unwrap() {
                "ok" => "成功:",
                "warn" => "警告:",
                "fail" => "错误:",
                "skip" => "跳过:",
                unknown => panic!("unexpected Doctor status: {unknown}"),
            };
            assert!(
                stdout
                    .lines()
                    .any(|line| line == format!("{prefix} {}", check["summary"].as_str().unwrap()))
            );
        }
        assert_eq!(inventory(f.0.path()), before);
    }
}

#[test]
fn platform_registry_counts_are_neutral_and_json_fields_are_unchanged() {
    let f = Fixture::new();
    let machine = f.run(&["platform", "list", "--json"]);
    let human = f.run(&["platform", "list"]);
    assert!(machine.status.success());
    assert!(human.status.success());
    assert!(machine.stderr.is_empty());
    assert!(human.stderr.is_empty());
    let report: Value = serde_json::from_slice(&machine.stdout).unwrap();
    assert_eq!(
        report
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["config_file", "platforms"]
    );
    assert_eq!(
        report["config_file"],
        f.0.path()
            .join("ccr")
            .join("config.toml")
            .display()
            .to_string()
    );
    let platforms = report["platforms"].as_array().unwrap();
    assert_eq!(
        platforms.len(),
        PlatformRegistry::new().list_platform_info().len()
    );
    assert_eq!(
        platforms
            .iter()
            .filter(|platform| platform["enabled"] == true)
            .count(),
        1
    );
    for platform in platforms {
        assert_eq!(
            platform
                .as_object()
                .unwrap()
                .keys()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            ["current_profile", "description", "enabled", "name"]
        );
        assert!(platform["current_profile"].is_null());
    }
    let stdout = text(&human.stdout);
    assert!(stdout.contains(&format!("Found {} platforms", platforms.len())));
    assert!(!stdout.contains("成功:"));
    assert!(stdout.contains("\n下一步\n  查看运行时状态\n    ccr current\n"));
}

#[test]
fn empty_profile_list_keeps_neutral_output_and_json_null_current() {
    let f = Fixture::new();
    f.write(
        "ccr/platforms/claude/profiles.toml",
        "default_config = ''\ncurrent_config = ''\n",
    );
    let machine = f.run(&["claude", "profile", "list", "--json"]);
    let human = f.run(&["claude", "profile", "list"]);
    assert!(machine.status.success(), "{}", text(&machine.stderr));
    assert!(human.status.success());
    assert_eq!(
        serde_json::from_slice::<Value>(&machine.stdout).unwrap(),
        json!({"current_profile": null, "profiles": []})
    );
    assert!(human.stderr.is_empty());
    let stdout = text(&human.stdout);
    assert!(stdout.contains("未找到 Claude profiles"));
    assert!(!stdout.contains("成功:"));
    assert!(!stdout.contains("下一步"));
}

#[test]
fn clean_preview_preserves_long_paths_and_files_at_small_widths() {
    let f = Fixture::new();
    let long_dir = "long-synthetic-planning-directory-with-all-required-characters";
    fs::create_dir_all(f.0.path().join("work").join(long_dir)).unwrap();
    f.write(
        &format!("work/{long_dir}/task_plan.md"),
        "synthetic planning content",
    );
    for width in ["40", "80", "120"] {
        let output = f
            .command(&["clean", "planfiles", "--all", "--dry-run"])
            .env("COLUMNS", width)
            .output()
            .unwrap();
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        let stdout = text(&output.stdout);
        assert!(stdout.contains(long_dir), "truncated path: {stdout}");
        assert!(stdout.contains("命中数量: 1 个"));
        assert!(stdout.contains("\n下一步\n  执行实际清理\n    ccr clean --all\n"));
        assert!(!stdout.contains("成功:"));
        assert!(!stdout.contains('\u{1b}'));
    }
    assert_eq!(
        fs::read_to_string(f.0.path().join("work").join(long_dir).join("task_plan.md")).unwrap(),
        "synthetic planning content"
    );
}

#[test]
fn clean_cancel_retains_destruction_warning_and_neutral_result() {
    let f = Fixture::new();
    f.write("work/task_plan.md", "synthetic planning content");
    let mut child = f
        .command(&["clean", "planfiles"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(b"n\n").unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = text(&output.stdout);
    assert!(stdout.contains("警告: 即将删除当前目录根层的规划文件！"));
    assert!(stdout.contains("已取消清理操作"));
    assert!(!stdout.contains("成功:"));
    assert!(!stdout.contains("下一步"));
    assert_eq!(
        fs::read_to_string(f.0.path().join("work/task_plan.md")).unwrap(),
        "synthetic planning content"
    );
}
