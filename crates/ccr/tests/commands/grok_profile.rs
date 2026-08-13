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
            "[models]\ndefault = \"grok-native\"\ndefault_reasoning_effort = \"low\"\n\n[ui]\ntheme = \"dark\"\n",
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
            "--api-key",
            INLINE_SECRET,
            "--model",
            "grok-example",
            "--api-backend",
            "responses",
            "--context-window",
            "1000000",
            "--supports-backend-search",
            "--reasoning-effort",
            "HIGH",
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

fn count_files(path: &std::path::Path) -> usize {
    if !path.exists() {
        return 0;
    }
    fs::read_dir(path)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .map(|path| if path.is_dir() { count_files(&path) } else { 1 })
        .sum()
}

#[test]
fn grok_profile_init_creates_inactive_template_without_touching_runtime() {
    let fixture = GrokProfileFixture::new();
    let runtime_path = fixture.grok_home.join("config.toml");
    let runtime_before = fs::read(&runtime_path).unwrap();
    let profiles_path = fixture
        .root
        .join("platforms")
        .join("grok")
        .join("profiles.toml");

    let first = fixture.run_output(&["grok", "profile", "init"]);
    assert_success(&first);
    assert_eq!(
        fs::read(&profiles_path).unwrap(),
        include_bytes!("../../../../examples/grok/profiles.toml")
    );
    assert_eq!(fs::read(&runtime_path).unwrap(), runtime_before);

    let (list_output, list_json) = fixture.run_json(&["grok", "profile", "list", "--json"]);
    assert_success(&list_output);
    assert!(list_json["current_profile"].is_null());
    let names = list_json["profiles"]
        .as_array()
        .unwrap()
        .iter()
        .map(|profile| profile["name"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(names, vec!["official", "relay"]);

    let current = fixture.run_output(&["grok", "profile", "current"]);
    assert_success(&current);
    assert!(String::from_utf8_lossy(&current.stdout).contains("不在 Grok profile mode"));

    let registry = ccr_config::PlatformConfigManager::new(fixture.root.join("config.toml"))
        .load()
        .unwrap();
    assert_eq!(registry.get_platform("grok").unwrap().current_profile, None);

    let profiles_before = fs::read(&profiles_path).unwrap();
    let backups_before = count_files(&fixture.root.join("backups"));
    let second = fixture.run_output(&["grok", "profile", "init"]);
    assert_success(&second);
    assert!(String::from_utf8_lossy(&second.stdout).contains("已存在"));
    assert_eq!(fs::read(&profiles_path).unwrap(), profiles_before);
    assert_eq!(count_files(&fixture.root.join("backups")), backups_before);
    assert_eq!(fs::read(&runtime_path).unwrap(), runtime_before);
}

#[test]
fn grok_profile_init_preserves_existing_unparseable_profiles_file() {
    let fixture = GrokProfileFixture::new();
    let profiles_path = fixture
        .root
        .join("platforms")
        .join("grok")
        .join("profiles.toml");
    fs::create_dir_all(profiles_path.parent().unwrap()).unwrap();
    let existing = b"auth_token = [\"do-not-read-or-replace\"\n";
    fs::write(&profiles_path, existing).unwrap();

    let (output, json) = fixture.run_json(&["grok", "profile", "init", "--json"]);
    assert_success(&output);
    assert_eq!(json["created"], false);
    assert_eq!(fs::read(&profiles_path).unwrap(), existing);
}

#[test]
fn grok_profile_init_json_reports_first_and_idempotent_runs() {
    let fixture = GrokProfileFixture::new();

    let (first_output, first) = fixture.run_json(&["grok", "profile", "init", "--json"]);
    assert_success(&first_output);
    assert_eq!(first["ok"], true);
    assert_eq!(first["platform"], "grok");
    assert_eq!(first["created"], true);
    assert_eq!(first["registered"], true);
    assert!(
        first["profiles_file"]
            .as_str()
            .unwrap()
            .ends_with("profiles.toml")
    );

    let (second_output, second) = fixture.run_json(&["grok", "profile", "init", "--json"]);
    assert_success(&second_output);
    assert_eq!(second["created"], false);
    assert_eq!(second["registered"], false);
}

#[test]
fn concurrent_grok_profile_init_is_lossless_and_idempotent() {
    let fixture = GrokProfileFixture::new();
    let mut first = fixture.command();
    first.args(["grok", "profile", "init"]);
    let mut second = fixture.command();
    second.args(["grok", "profile", "init"]);

    let first = first.spawn().unwrap();
    let second = second.spawn().unwrap();
    assert_success(&first.wait_with_output().unwrap());
    assert_success(&second.wait_with_output().unwrap());

    assert_eq!(
        fs::read(
            fixture
                .root
                .join("platforms")
                .join("grok")
                .join("profiles.toml")
        )
        .unwrap(),
        include_bytes!("../../../../examples/grok/profiles.toml")
    );
}

#[cfg(unix)]
#[test]
fn grok_profile_init_creates_owner_only_profiles_file() {
    use std::os::unix::fs::PermissionsExt;

    let fixture = GrokProfileFixture::new();
    assert_success(&fixture.run_output(&["grok", "profile", "init"]));
    let mode = fs::metadata(
        fixture
            .root
            .join("platforms")
            .join("grok")
            .join("profiles.toml"),
    )
    .unwrap()
    .permissions()
    .mode();
    assert_eq!(mode & 0o777, 0o600);
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
        runtime["models"]["default_reasoning_effort"].as_str(),
        Some("high")
    );
    assert_eq!(
        runtime["model"]["custom"]["api_key"].as_str(),
        Some(INLINE_SECRET)
    );
    assert_eq!(
        runtime["model"]["custom"]["supports_reasoning_effort"].as_bool(),
        Some(true)
    );
    assert_eq!(
        runtime["model"]["custom"]["reasoning_effort"].as_str(),
        Some("high")
    );
    assert_eq!(runtime["ui"]["theme"].as_str(), Some("dark"));

    let (current_output, current_json) =
        fixture.run_json(&["grok", "profile", "current", "--json"]);
    assert_success(&current_output);
    let current_stdout = String::from_utf8_lossy(&current_output.stdout);
    assert_eq!(current_json["platform"], "grok");
    assert_eq!(current_json["profile"], "relay");
    assert_eq!(current_json["details"]["auth_mode"], "inline_api_key");
    assert_eq!(current_json["details"]["reasoning_effort"], "high");
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
    assert_eq!(list_json["profiles"][0]["reasoning_effort"], "high");
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

    let invalid_effort = fixture.run_output(&[
        "grok",
        "profile",
        "set-field",
        "relay",
        "reasoning_effort",
        "--value",
        "model-option",
    ]);
    assert!(!invalid_effort.status.success());
    assert!(String::from_utf8_lossy(&invalid_effort.stderr).contains("允许值为"));

    assert_success(&fixture.run_output(&[
        "grok",
        "profile",
        "set-field",
        "relay",
        "reasoning_effort",
        "--value",
        "XHIGH",
    ]));
    assert_success(&fixture.run_output(&["grok", "profile", "switch", "relay"]));
    let runtime = fixture.runtime();
    assert_eq!(
        runtime["models"]["default_reasoning_effort"].as_str(),
        Some("xhigh")
    );
    assert_eq!(
        runtime["model"]["custom"]["reasoning_effort"].as_str(),
        Some("xhigh")
    );

    assert_success(&fixture.run_output(&[
        "grok",
        "profile",
        "set-field",
        "relay",
        "reasoning_effort",
        "--clear",
    ]));
    assert_success(&fixture.run_output(&["grok", "profile", "switch", "relay"]));
    let runtime = fixture.runtime();
    assert_eq!(
        runtime["models"]["default_reasoning_effort"].as_str(),
        Some("low")
    );
    assert!(
        runtime["model"]["custom"]
            .get("supports_reasoning_effort")
            .is_none()
    );
    assert!(runtime["model"]["custom"].get("reasoning_effort").is_none());

    assert_success(&fixture.run_output(&["grok", "profile", "off"]));
    let restored = fixture.runtime();
    assert_eq!(restored["models"]["default"].as_str(), Some("grok-native"));
    assert_eq!(
        restored["models"]["default_reasoning_effort"].as_str(),
        Some("low")
    );
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

#[test]
fn grok_profile_off_uses_shared_core_and_is_idempotent() {
    let fixture = GrokProfileFixture::new();
    assert_success(&fixture.create_inline("relay").0);
    assert_success(&fixture.run_output(&["grok", "profile", "switch", "relay"]));

    let (off, json) = fixture.run_json(&["grok", "profile", "off", "--json"]);
    assert_success(&off);
    assert_eq!(json["ok"], true);
    assert_eq!(json["changed"], true);
    assert_eq!(json["previous_profile"], "relay");
    assert_eq!(json["runtime_mode"], "grok_native");
    let visible = format!(
        "{}{}",
        String::from_utf8_lossy(&off.stdout),
        String::from_utf8_lossy(&off.stderr)
    );
    assert!(!visible.contains(INLINE_SECRET));
    assert_eq!(
        fixture.runtime()["models"]["default"].as_str(),
        Some("grok-native")
    );

    let (again, again_json) = fixture.run_json(&["grok", "profile", "off", "--json"]);
    assert_success(&again);
    assert_eq!(again_json["changed"], false);
    assert!(again_json["previous_profile"].is_null());
}

#[test]
fn grok_profile_off_fails_closed_when_entry_state_is_missing() {
    let fixture = GrokProfileFixture::new();
    assert_success(&fixture.create_inline("relay").0);
    assert_success(&fixture.run_output(&["grok", "profile", "switch", "relay"]));

    let entry_state = fixture
        .root
        .join("platforms")
        .join("grok")
        .join("profile_entry_config_state.json");
    fs::remove_file(&entry_state).unwrap();
    let before = fs::read(fixture.grok_home.join("config.toml")).unwrap();

    let output = fixture.run_output(&["grok", "profile", "off"]);
    assert!(!output.status.success(), "{:?}", output.status);
    let visible = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(visible.contains("入口配置状态缺失"));
    assert!(!visible.contains(INLINE_SECRET));
    assert_eq!(
        fs::read(fixture.grok_home.join("config.toml")).unwrap(),
        before
    );
}
