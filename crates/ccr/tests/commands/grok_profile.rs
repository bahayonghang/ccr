#![allow(clippy::unwrap_used)]

use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use tempfile::TempDir;

const INLINE_SECRET: &str = "INLINE_GROK_SECRET_SENTINEL";

struct GrokProfileFixture {
    _temp_dir: TempDir,
    home: PathBuf,
    root: PathBuf,
    grok_home: PathBuf,
}

impl GrokProfileFixture {
    fn new() -> Self {
        let temp_dir = tempfile::tempdir().unwrap();
        let home = temp_dir.path().join("home");
        let root = home.join(".ccr");
        let grok_home = home.join(".grok");
        fs::create_dir_all(&root).unwrap();
        fs::create_dir_all(&grok_home).unwrap();
        fs::write(
            grok_home.join("config.toml"),
            "[models]\ndefault = \"grok-native\"\n\n[ui]\ntheme = \"dark\"\n",
        )
        .unwrap();

        Self {
            _temp_dir: temp_dir,
            home,
            root,
            grok_home,
        }
    }

    fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_ccr"));
        command.env("CCR_ROOT", &self.root);
        command.env("CCR_LOCK_DIR", self.home.join(".locks"));
        command.env("GROK_HOME", &self.grok_home);
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
        let json = serde_json::from_str::<Value>(&stdout).unwrap_or_else(|error| {
            panic!(
                "invalid json: {error}\nstatus={:?}\nstdout={stdout}\nstderr={}",
                output.status,
                String::from_utf8_lossy(&output.stderr)
            )
        });
        (output, json)
    }

    fn runtime(&self) -> toml::Value {
        toml::from_str(&fs::read_to_string(self.grok_home.join("config.toml")).unwrap()).unwrap()
    }

    fn create_inline(&self, name: &str) -> (Output, Value) {
        self.run_json(&[
            "grok",
            "profile",
            "create",
            name,
            "--description",
            "Example relay",
            "--base-url",
            "https://user:password@api.example.com/v1?token=QUERY_SECRET",
            "--auth-token",
            INLINE_SECRET,
            "--model",
            "grok-example",
            "--api-backend",
            "responses",
            "--context-window",
            "1000000",
            "--supports-backend-search",
            "--json",
        ])
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
fn grok_profile_command_flow_masks_secrets_and_restores_entry_runtime() {
    let fixture = GrokProfileFixture::new();

    let (create_output, create_json) = fixture.create_inline("relay");
    assert_success(&create_output);
    assert_eq!(create_json["platform"], "grok");
    assert!(!String::from_utf8_lossy(&create_output.stdout).contains(INLINE_SECRET));

    assert_success(&fixture.run_output(&["grok", "profile", "switch", "relay"]));
    let runtime = fixture.runtime();
    assert_eq!(runtime["models"]["default"].as_str(), Some("custom"));
    assert_eq!(
        runtime["model"]["custom"]["api_key"].as_str(),
        Some(INLINE_SECRET)
    );
    assert_eq!(runtime["ui"]["theme"].as_str(), Some("dark"));

    let (current_output, current_json) =
        fixture.run_json(&["grok", "profile", "current", "--json"]);
    assert_success(&current_output);
    let current_stdout = String::from_utf8_lossy(&current_output.stdout);
    assert_eq!(current_json["platform"], "grok");
    assert_eq!(current_json["profile"], "relay");
    assert_eq!(current_json["details"]["auth_mode"], "inline_api_key");
    assert_eq!(
        current_json["details"]["base_url"],
        "https://api.example.com/v1"
    );
    for secret in [INLINE_SECRET, "password", "QUERY_SECRET"] {
        assert!(!current_stdout.contains(secret), "{current_stdout}");
    }

    let (list_output, list_json) = fixture.run_json(&["grok", "profile", "list", "--json"]);
    assert_success(&list_output);
    assert_eq!(list_json["current_profile"], "relay");
    assert!(!String::from_utf8_lossy(&list_output.stdout).contains(INLINE_SECRET));

    let (set_output, set_json) = fixture.run_json(&[
        "grok",
        "profile",
        "set-field",
        "relay",
        "context_window",
        "--value",
        "200000",
        "--json",
    ]);
    assert_success(&set_output);
    assert_eq!(set_json["name"], "relay");

    assert_success(&fixture.run_output(&["grok", "profile", "off"]));
    let restored = fixture.runtime();
    assert_eq!(restored["models"]["default"].as_str(), Some("grok-native"));
    assert!(restored.get("model").is_none());
    assert_eq!(restored["ui"]["theme"].as_str(), Some("dark"));

    let (delete_output, delete_json) =
        fixture.run_json(&["grok", "profile", "delete", "relay", "--json"]);
    assert_success(&delete_output);
    assert_eq!(delete_json["name"], "relay");
}

#[test]
fn grok_profile_force_delete_restores_runtime_before_removing_profile() {
    let fixture = GrokProfileFixture::new();
    assert_success(&fixture.create_inline("relay").0);
    assert_success(&fixture.run_output(&["grok", "profile", "switch", "relay"]));
    assert_success(&fixture.run_output(&[
        "grok",
        "profile",
        "set-field",
        "relay",
        "model",
        "--value",
        "grok-example-updated",
    ]));

    let rejected = fixture.run_output(&["grok", "profile", "delete", "relay"]);
    assert!(!rejected.status.success());
    let visible_error = format!(
        "{}{}",
        String::from_utf8_lossy(&rejected.stdout),
        String::from_utf8_lossy(&rejected.stderr)
    );
    assert!(visible_error.contains("off"), "{visible_error}");

    let (delete_output, delete_json) =
        fixture.run_json(&["grok", "profile", "delete", "relay", "--force", "--json"]);
    assert_success(&delete_output);
    assert_eq!(delete_json["name"], "relay");
    assert_eq!(
        fixture.runtime()["models"]["default"].as_str(),
        Some("grok-native")
    );
}

#[test]
fn grok_profile_rejects_array_env_key_and_legacy_platform_switch() {
    let fixture = GrokProfileFixture::new();
    let (create_output, _) = fixture.run_json(&[
        "grok",
        "profile",
        "create",
        "official",
        "--model",
        "grok-example",
        "--json",
    ]);
    assert_success(&create_output);

    let invalid = fixture.run_output(&[
        "grok",
        "profile",
        "set-field",
        "official",
        "env_key",
        "--value-json",
        "[\"KEY_A\",\"KEY_B\"]",
    ]);
    assert!(!invalid.status.success());
    assert!(String::from_utf8_lossy(&invalid.stderr).contains("仅支持单个环境变量名"));

    let legacy = fixture.run_output(&["platform", "switch", "grok"]);
    assert!(!legacy.status.success());
    let stderr = String::from_utf8_lossy(&legacy.stderr);
    assert!(stderr.contains("legacy"));
    assert!(stderr.contains("ccr grok profile"));
}
