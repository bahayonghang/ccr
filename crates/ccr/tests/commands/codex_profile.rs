#![allow(clippy::unwrap_used)]

use ccr::managers::{
    CcsConfig, ConfigManager, GlobalSettings, PlatformConfigEntry, PlatformConfigManager,
    UnifiedConfig,
};
use ccr_config::ProfileConfig;
use indexmap::IndexMap;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use tempfile::TempDir;

struct CodexProfileFixture {
    _temp_dir: TempDir,
    home: PathBuf,
    root: PathBuf,
    codex_dir: PathBuf,
}

impl CodexProfileFixture {
    fn new() -> Self {
        let temp_dir = tempfile::tempdir().unwrap();
        let home = temp_dir.path().join("home");
        let root = home.join(".ccr");
        let codex_dir = home.join(".codex");

        fs::create_dir_all(&home).unwrap();
        fs::create_dir_all(root.join("platforms").join("codex")).unwrap();
        fs::create_dir_all(&codex_dir).unwrap();

        Self {
            _temp_dir: temp_dir,
            home,
            root,
            codex_dir,
        }
    }

    fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_ccr"));
        command.env("CCR_ROOT", &self.root);
        command.env("CCR_CODEX_DIR", &self.codex_dir);
        command.env("CCR_BACKUP_DIR", self.home.join(".claude").join("backups"));
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

    fn write_unified_codex_profile(&self, current_profile: Option<&str>) {
        let manager = PlatformConfigManager::new(self.root.join("config.toml"));
        let mut unified = UnifiedConfig {
            default_platform: "codex".to_string(),
            current_platform: "codex".to_string(),
            platforms: IndexMap::new(),
        };
        unified.platforms.insert(
            "codex".to_string(),
            PlatformConfigEntry {
                enabled: true,
                current_profile: current_profile.map(str::to_string),
                description: None,
                last_used: None,
            },
        );
        manager.save(&unified).unwrap();
    }

    fn save_codex_profiles(&self, current_config: &str) {
        let manager = ConfigManager::new(
            self.root
                .join("platforms")
                .join("codex")
                .join("profiles.toml"),
        );

        let official = ProfileConfig {
            description: Some("Official".to_string()),
            provider: Some("openai".to_string()),
            provider_type: Some("official_relay".to_string()),
            ..Default::default()
        };

        let team = ProfileConfig {
            description: Some("Team".to_string()),
            provider: Some("openai".to_string()),
            provider_type: Some("official_relay".to_string()),
            auth_token: Some("sk-team-token".to_string()),
            model: Some("gpt-5".to_string()),
            ..Default::default()
        };

        let mut sections = IndexMap::new();
        sections.insert(
            "official".to_string(),
            ccr_config::profile_to_section(&official).unwrap(),
        );
        sections.insert(
            "team".to_string(),
            ccr_config::profile_to_section(&team).unwrap(),
        );

        manager
            .save(&CcsConfig {
                default_config: "official".to_string(),
                current_config: current_config.to_string(),
                settings: GlobalSettings::default(),
                sections,
            })
            .unwrap();
    }

    fn write_codex_runtime_official(&self) {
        fs::write(
            self.codex_dir.join("config.toml"),
            r#"
cli_auth_credentials_store = "file"
model_provider = "custom"

[model_providers.custom]
name = "openai"
base_url = "https://api.openai.com/v1"
wire_api = "responses"
requires_openai_auth = true
"#
            .trim_start(),
        )
        .unwrap();
    }

    fn write_auth_json_with_oauth(&self) -> String {
        let auth = r#"{
  "OPENAI_API_KEY": null,
  "tokens": {
    "id_token": "eyJhbGciOiJub25lIiwidHlwIjoiSldUIn0.eyJlbWFpbCI6InRlc3RAZXhhbXBsZS5jb20iLCJzdWIiOiIxMjMifQ.signature",
    "access_token": "at_test",
    "refresh_token": "rt_test",
    "account_id": "acct-123"
  },
  "last_refresh": "2026-01-08T03:09:53.894843900Z"
}"#;
        fs::write(self.codex_dir.join("auth.json"), auth).unwrap();
        auth.to_string()
    }
}

#[test]
fn codex_profile_list_json_reports_current_profile() {
    let fixture = CodexProfileFixture::new();
    fixture.write_unified_codex_profile(Some("team"));
    fixture.save_codex_profiles("team");
    fixture.write_codex_runtime_official();

    let (output, json) = fixture.run_json(&["codex", "profile", "list", "--json"]);

    assert!(output.status.success(), "{:?}", output.status);
    assert_eq!(json["current_profile"], "team");
    assert!(
        json["profiles"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["name"] == "team" && item["is_current"] == true)
    );
}

#[test]
fn codex_profile_current_json_returns_single_codex_card() {
    let fixture = CodexProfileFixture::new();
    fixture.write_unified_codex_profile(Some("team"));
    fixture.save_codex_profiles("team");
    fixture.write_codex_runtime_official();

    let (output, json) = fixture.run_json(&["codex", "profile", "current", "--json"]);

    assert!(output.status.success(), "{:?}", output.status);
    assert_eq!(json["platform"], "codex");
    assert_eq!(json["profile"], "team");
    assert!(json.get("schema_version").is_none());
    assert!(json.get("claude").is_none());
}

#[test]
fn codex_profile_switch_and_off_are_consistent_and_off_keeps_auth_json() {
    let fixture = CodexProfileFixture::new();
    fixture.write_unified_codex_profile(Some("official"));
    fixture.save_codex_profiles("official");
    fixture.write_codex_runtime_official();
    let auth_before = fixture.write_auth_json_with_oauth();

    let switch_output = fixture.run_output(&["codex", "profile", "switch", "team"]);
    assert!(
        switch_output.status.success(),
        "status={:?}\nstdout:\n{}\nstderr:\n{}",
        switch_output.status,
        String::from_utf8_lossy(&switch_output.stdout),
        String::from_utf8_lossy(&switch_output.stderr)
    );

    let (_, current_json) = fixture.run_json(&["codex", "profile", "current", "--json"]);
    assert_eq!(current_json["profile"], "team");

    let off_output = fixture.run_output(&["codex", "profile", "off"]);
    assert!(off_output.status.success(), "{:?}", off_output.status);

    let registry = PlatformConfigManager::new(fixture.root.join("config.toml"))
        .load()
        .unwrap();
    assert_eq!(
        registry
            .get_platform("codex")
            .unwrap()
            .current_profile
            .as_deref(),
        None
    );

    let profiles = ConfigManager::new(
        fixture
            .root
            .join("platforms")
            .join("codex")
            .join("profiles.toml"),
    )
    .load()
    .unwrap();
    assert_eq!(profiles.current_config, "");

    let auth_after = fs::read_to_string(fixture.codex_dir.join("auth.json")).unwrap();
    assert_eq!(auth_after, auth_before);
}

#[test]
fn codex_profile_off_is_idempotent_when_no_active_profile() {
    let fixture = CodexProfileFixture::new();
    fixture.write_unified_codex_profile(None);
    fixture.save_codex_profiles("");
    fixture.write_codex_runtime_official();
    let auth_before = fixture.write_auth_json_with_oauth();

    let output = fixture.run_output(&["codex", "profile", "off"]);
    assert!(output.status.success(), "{:?}", output.status);

    let auth_after = fs::read_to_string(fixture.codex_dir.join("auth.json")).unwrap();
    assert_eq!(auth_after, auth_before);
}

#[test]
fn codex_profile_off_clears_stale_profiles_file_pointer_without_touching_auth_json() {
    let fixture = CodexProfileFixture::new();
    fixture.write_unified_codex_profile(None);
    fixture.save_codex_profiles("team");
    fixture.write_codex_runtime_official();
    let auth_before = fixture.write_auth_json_with_oauth();

    let output = fixture.run_output(&["codex", "profile", "off"]);
    assert!(output.status.success(), "{:?}", output.status);

    let registry = PlatformConfigManager::new(fixture.root.join("config.toml"))
        .load()
        .unwrap();
    assert_eq!(
        registry
            .get_platform("codex")
            .unwrap()
            .current_profile
            .as_deref(),
        None
    );

    let profiles = ConfigManager::new(
        fixture
            .root
            .join("platforms")
            .join("codex")
            .join("profiles.toml"),
    )
    .load()
    .unwrap();
    assert_eq!(profiles.current_config, "");

    let auth_after = fs::read_to_string(fixture.codex_dir.join("auth.json")).unwrap();
    assert_eq!(auth_after, auth_before);
}

#[test]
fn codex_profile_crud_commands_support_vscode_surface() {
    let fixture = CodexProfileFixture::new();

    let (create_output, create_json) = fixture.run_json(&[
        "codex",
        "profile",
        "create",
        "team",
        "--description",
        "Team profile",
        "--model",
        "gpt-5",
        "--provider",
        "openai",
        "--provider-type",
        "official_relay",
        "--account",
        "team-account",
        "--tag",
        "team",
        "--json",
    ]);
    assert!(create_output.status.success(), "{:?}", create_output.status);
    assert_eq!(create_json["platform"], "codex");
    assert_eq!(create_json["name"], "team");
    assert_eq!(create_json["enabled"], true);

    let (set_output, set_json) = fixture.run_json(&[
        "codex",
        "profile",
        "set-field",
        "team",
        "tags",
        "--value-json",
        r#"["prod","shared"]"#,
        "--json",
    ]);
    assert!(set_output.status.success(), "{:?}", set_output.status);
    assert_eq!(set_json["name"], "team");

    let (disable_output, disable_json) =
        fixture.run_json(&["codex", "profile", "disable", "team", "--force", "--json"]);
    assert!(
        disable_output.status.success(),
        "{:?}",
        disable_output.status
    );
    assert_eq!(disable_json["enabled"], false);

    let (enable_output, enable_json) =
        fixture.run_json(&["codex", "profile", "enable", "team", "--json"]);
    assert!(enable_output.status.success(), "{:?}", enable_output.status);
    assert_eq!(enable_json["enabled"], true);

    let (delete_output, delete_json) =
        fixture.run_json(&["codex", "profile", "delete", "team", "--force", "--json"]);
    assert!(delete_output.status.success(), "{:?}", delete_output.status);
    assert_eq!(delete_json["name"], "team");
}
