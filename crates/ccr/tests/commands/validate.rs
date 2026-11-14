#![allow(clippy::unwrap_used)]

use ccr_cli::managers::{CcsConfig, ClaudeSettings, ConfigManager, ConfigSection, GlobalSettings};
use indexmap::IndexMap;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use tempfile::TempDir;

struct ValidateFixture {
    _temp_dir: TempDir,
    home: PathBuf,
    root: PathBuf,
    claude_settings_path: PathBuf,
    codex_dir: PathBuf,
}

impl ValidateFixture {
    fn new() -> Self {
        let temp_dir = tempfile::tempdir().unwrap();
        let home = temp_dir.path().join("home");
        let root = home.join(".ccr");
        let claude_settings_path = home.join(".claude").join("settings.json");
        let codex_dir = home.join(".codex");

        fs::create_dir_all(&root).unwrap();
        fs::create_dir_all(claude_settings_path.parent().unwrap()).unwrap();
        fs::create_dir_all(&codex_dir).unwrap();

        Self {
            _temp_dir: temp_dir,
            home,
            root,
            claude_settings_path,
            codex_dir,
        }
    }

    fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_ccr"));
        command.env("CCR_ROOT", &self.root);
        command.env("CCR_DATA_DIR", &self.root);
        command.env("CCR_LOCK_DIR", self.home.join(".locks"));
        command.env("CCR_SETTINGS_PATH", &self.claude_settings_path);
        command.env("CCR_BACKUP_DIR", self.home.join(".claude").join("backups"));
        command.env("CCR_CODEX_DIR", &self.codex_dir);
        command.env("CODEX_HOME", &self.codex_dir);
        command.env("GROK_HOME", self.home.join(".grok"));
        command.env("CLAUDE_CONFIG_DIR", self.home.join(".claude"));
        command.env(
            "CLAUDE_JSON_PATH",
            self.home.join(".claude").join("state.json"),
        );
        command.env("HOME", &self.home);
        command.env("USERPROFILE", &self.home);
        command.env("NO_COLOR", "1");
        command.env("CCR_LOG_LEVEL", "off");
        command
    }

    fn run(&self, args: &[&str]) -> Output {
        self.command().args(args).output().unwrap()
    }

    fn write_registry_without_legacy_routing_fields(&self) {
        fs::write(
            self.root.join("config.toml"),
            r#"
[claude]
enabled = true
current_profile = "main"
"#,
        )
        .unwrap();
    }

    fn write_claude_profile(&self) {
        // ConfigManager::save resolves CCR_LOCK_DIR even with an explicit target path.
        let _env = crate::setup_ccr_test_env();
        let manager = ConfigManager::new(
            self.root
                .join("platforms")
                .join("claude")
                .join("profiles.toml"),
        );
        let mut config = CcsConfig {
            default_config: "main".to_string(),
            current_config: "main".to_string(),
            settings: GlobalSettings::default(),
            sections: IndexMap::new(),
        };
        config.sections.insert(
            "main".to_string(),
            ConfigSection {
                description: Some("Claude API key".to_string()),
                base_url: Some("https://api.example.com".to_string()),
                auth_token: Some(ccr_core::Secret::from("sk-claude-test")),
                model: Some("claude-test".to_string()),
                small_fast_model: None,
                provider: Some("example".to_string()),
                provider_type: None,
                account: None,
                tags: None,
                usage_count: Some(0),
                enabled: Some(true),
                other: IndexMap::new(),
                ..Default::default()
            },
        );
        manager.save(&config).unwrap();
    }

    fn write_claude_settings(&self) {
        fs::write(
            &self.claude_settings_path,
            serde_json::to_string_pretty(&ClaudeSettings {
                env: HashMap::from([
                    (
                        "ANTHROPIC_BASE_URL".to_string(),
                        "https://api.example.com".to_string(),
                    ),
                    (
                        "ANTHROPIC_AUTH_TOKEN".to_string(),
                        "sk-claude-test".to_string(),
                    ),
                    ("ANTHROPIC_MODEL".to_string(), "claude-test".to_string()),
                ]),
                ..ClaudeSettings::default()
            })
            .unwrap(),
        )
        .unwrap();
    }
}

#[test]
fn validate_accepts_registry_without_legacy_current_platform() {
    let fixture = ValidateFixture::new();
    fixture.write_registry_without_legacy_routing_fields();
    fixture.write_claude_profile();
    fixture.write_claude_settings();

    let output = fixture.run(&["validate"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "stdout={stdout}\nstderr={stderr}");
    assert!(stdout.contains("Codex"));
    assert!(!stdout.contains("ccr switch <config>"));
}

#[test]
fn validate_skips_missing_claude_and_codex_profiles_without_legacy_platform_gate() {
    let fixture = ValidateFixture::new();
    fs::write(fixture.root.join("config.toml"), "").unwrap();
    fs::write(
        &fixture.claude_settings_path,
        serde_json::to_string_pretty(&ClaudeSettings::default()).unwrap(),
    )
    .unwrap();

    let output = fixture.run(&["validate"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "stdout={stdout}\nstderr={stderr}");
    assert!(stdout.contains("Claude"));
    assert!(stdout.contains("Codex"));
    assert!(!stdout.contains("ccr switch <config>"));
}

#[test]
fn validate_json_emits_pure_report_without_creating_logs_and_preserves_error_exit_code() {
    let fixture = ValidateFixture::new();
    fixture.write_registry_without_legacy_routing_fields();
    fixture.write_claude_profile();
    fixture.write_claude_settings();

    let mut valid_cmd = fixture.command();
    valid_cmd.env_remove("NO_COLOR");
    valid_cmd.env("CLICOLOR", "1");
    valid_cmd.env("CLICOLOR_FORCE", "1");
    valid_cmd.env("CCR_DATA_DIR", &fixture.root);
    let valid_out = valid_cmd.args(["validate", "--json"]).output().unwrap();
    assert_eq!(
        valid_out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&valid_out.stderr)
    );
    assert!(valid_out.stderr.is_empty());
    let valid_stdout = String::from_utf8_lossy(&valid_out.stdout);
    assert!(!valid_stdout.contains('\u{1b}'), "{valid_stdout}");
    let valid_json: serde_json::Value = serde_json::from_str(&valid_stdout).unwrap();
    assert!(valid_json["checks"].is_array());
    assert!(!fixture.root.join("logs").exists());

    // Now test failing validation exit code + zero secret leakage with --json.
    let bad_fixture = ValidateFixture::new();
    fs::write(bad_fixture.root.join("config.toml"), "").unwrap();
    let sentinel = "VALIDATE_JSON_SECRET_SENTINEL_4482";
    let broken_toml = bad_fixture
        .root
        .join("platforms")
        .join("claude")
        .join("profiles.toml");
    fs::create_dir_all(broken_toml.parent().unwrap()).unwrap();
    fs::write(
        &broken_toml,
        format!("[broken]\nbase_url = \"not-a-valid-url\"\nauth_token = \"{sentinel}\"\n"),
    )
    .unwrap();

    let mut fail_cmd = bad_fixture.command();
    fail_cmd.env_remove("NO_COLOR");
    fail_cmd.env("CLICOLOR", "1");
    fail_cmd.env("CLICOLOR_FORCE", "1");
    fail_cmd.env("CCR_DATA_DIR", &bad_fixture.root);
    let fail_out = fail_cmd.args(["validate", "--json"]).output().unwrap();
    assert_eq!(fail_out.status.code(), Some(90));
    let fail_stdout = String::from_utf8_lossy(&fail_out.stdout);
    let fail_stderr = String::from_utf8_lossy(&fail_out.stderr);
    assert!(!fail_stdout.contains('\u{1b}'), "{fail_stdout}");
    assert!(!fail_stdout.contains(sentinel));
    assert!(!fail_stderr.contains(sentinel));
    let fail_json: serde_json::Value = serde_json::from_str(&fail_stdout).unwrap();
    assert!(
        fail_json["checks"]
            .as_array()
            .unwrap()
            .iter()
            .any(|check| check["severity"] == "error"),
        "expected error check in report: {fail_json}"
    );
    assert!(!bad_fixture.root.join("logs").exists());
}
