#![allow(clippy::unwrap_used)]

use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use tempfile::TempDir;

struct PlatformProfileSurfaceFixture {
    _temp_dir: TempDir,
    home: PathBuf,
    root: PathBuf,
}

impl PlatformProfileSurfaceFixture {
    fn new() -> Self {
        let temp_dir = tempfile::tempdir().unwrap();
        let home = temp_dir.path().join("home");
        let root = home.join(".ccr");
        fs::create_dir_all(&home).unwrap();
        fs::create_dir_all(&root).unwrap();

        Self {
            _temp_dir: temp_dir,
            home,
            root,
        }
    }

    fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_ccr"));
        command.env("CCR_ROOT", &self.root);
        command.env("CCR_CODEX_DIR", self.home.join(".codex"));
        command.env("CLAUDE_CONFIG_DIR", self.home.join(".claude"));
        command.env("CLAUDE_JSON_PATH", self.home.join(".claude.json"));
        command.env(
            "CCR_SETTINGS_PATH",
            self.home.join(".claude").join("settings.json"),
        );
        command.env("CCR_BACKUP_DIR", self.home.join(".backups"));
        command.env("CCR_LOCK_DIR", self.home.join(".locks"));
        command.env("HOME", &self.home);
        command.env("USERPROFILE", &self.home);
        command.env("NO_COLOR", "1");
        command.env("CLICOLOR", "0");
        command.env("COLUMNS", "120");
        command.env("CCR_LOG_LEVEL", "off");
        command
    }

    fn run_output(&self, args: &[&str]) -> Output {
        self.command().args(args).output().unwrap()
    }

    fn run_json(&self, args: &[&str]) -> (Output, Value) {
        let output = self.run_output(args);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let json = serde_json::from_str::<Value>(&stdout)
            .unwrap_or_else(|error| panic!("failed to parse stdout as json: {error}\n{stdout}"));
        (output, json)
    }
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "status={:?}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn claude_and_codex_profile_crud_surfaces_accept_supported_platforms() {
    let fixture = PlatformProfileSurfaceFixture::new();

    let (claude_output, claude_json) = fixture.run_json(&[
        "claude",
        "profile",
        "create",
        "work",
        "--description",
        "Work profile",
        "--base-url",
        "https://anthropic-proxy.example.com",
        "--auth-token",
        "sk-work",
        "--model",
        "claude-sonnet-4-20250514",
        "--json",
    ]);
    assert_success(&claude_output);
    assert_eq!(claude_json["platform"], "claude");
    assert_eq!(claude_json["name"], "work");

    let (codex_output, codex_json) = fixture.run_json(&[
        "codex",
        "profile",
        "create",
        "team",
        "--provider",
        "openai",
        "--provider-type",
        "official_relay",
        "--model",
        "gpt-5",
        "--json",
    ]);
    assert_success(&codex_output);
    assert_eq!(codex_json["platform"], "codex");
    assert_eq!(codex_json["name"], "team");
}

#[test]
fn legacy_platform_profile_gemini_reports_migration_instead_of_mutating() {
    let fixture = PlatformProfileSurfaceFixture::new();

    let output = fixture.run_output(&["platform", "profile", "create", "gemini", "team"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success(), "{stderr}");
    assert!(stderr.contains("legacy"), "{stderr}");
    assert!(stderr.contains("current_platform"), "{stderr}");
    assert!(stderr.contains("ccr claude profile"), "{stderr}");
    assert!(stderr.contains("ccr codex profile"), "{stderr}");
}
