#![allow(clippy::unwrap_used)]

use ccr_cli::managers::{
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

    fn command_with_redirected_codex_home(&self, sandbox: &PathBuf) -> Command {
        let mut command = self.command();
        command.env_remove("CCR_CODEX_DIR");
        command.env("CODEX_HOME", sandbox);
        command
    }

    fn run_output(&self, args: &[&str]) -> Output {
        self.command().args(args).output().unwrap()
    }

    fn run_json(&self, args: &[&str]) -> (Output, Value) {
        let output = self.run_output(args);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let json = serde_json::from_str::<Value>(&stdout).unwrap();
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
            auth_token: Some(ccr_core::Secret::from("sk-team-token")),
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

    fn save_deepseek_profile(&self) {
        let catalog = self.codex_dir.join("models.json");
        fs::write(&catalog, "[]").unwrap();
        let mut deepseek = ProfileConfig {
            description: Some("DeepSeek".to_string()),
            base_url: Some("https://api.deepseek.com/".to_string()),
            auth_token: Some(ccr_core::Secret::from("deepseek-command-secret")),
            model: Some("deepseek-v4-flash".to_string()),
            provider: Some("deepseek".to_string()),
            provider_type: Some("third_party_model".to_string()),
            enabled: Some(true),
            ..Default::default()
        };
        deepseek
            .platform_data
            .insert("wire_api".into(), serde_json::json!("responses"));
        deepseek.platform_data.insert(
            "auth_mode".into(),
            serde_json::json!("provider_bearer_token"),
        );
        deepseek.platform_data.insert(
            "model_catalog_json".into(),
            serde_json::json!(catalog.display().to_string()),
        );
        deepseek
            .platform_data
            .insert("model_reasoning_effort".into(), serde_json::json!("high"));

        let mut sections = IndexMap::new();
        sections.insert(
            "deepseek".to_string(),
            ccr_config::profile_to_section(&deepseek).unwrap(),
        );
        ConfigManager::new(
            self.root
                .join("platforms")
                .join("codex")
                .join("profiles.toml"),
        )
        .save(&CcsConfig {
            default_config: "deepseek".to_string(),
            current_config: String::new(),
            settings: GlobalSettings::default(),
            sections,
        })
        .unwrap();
    }

    fn write_codex_runtime_official(&self) {
        self.write_codex_runtime_official_with_credential_store("file");
    }

    fn write_codex_runtime_official_with_credential_store(&self, credential_store: &str) {
        let config = format!(
            r#"
cli_auth_credentials_store = "{credential_store}"
model_provider = "custom"

[model_providers.custom]
name = "openai"
base_url = "https://api.openai.com/v1"
wire_api = "responses"
requires_openai_auth = true
"#
        );
        fs::write(self.codex_dir.join("config.toml"), config.trim_start()).unwrap();
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
fn codex_profile_init_is_inactive_idempotent_and_preserves_runtime() {
    let fixture = CodexProfileFixture::new();
    let runtime_path = fixture.codex_dir.join("config.toml");
    let runtime_before = b"model = \"gpt-existing\"\n";
    fs::write(&runtime_path, runtime_before).unwrap();
    let profiles_path = fixture
        .root
        .join("platforms")
        .join("codex")
        .join("profiles.toml");

    let first = fixture.run_output(&["codex", "profile", "init"]);
    assert!(first.status.success(), "{:?}", first.status);
    let profiles_before = fs::read(&profiles_path).unwrap();
    assert_eq!(
        profiles_before,
        include_bytes!("../../../../examples/codex/profiles.toml")
    );
    assert_eq!(fs::read(&runtime_path).unwrap(), runtime_before);

    let registry = PlatformConfigManager::new(fixture.root.join("config.toml"))
        .load()
        .unwrap();
    assert_eq!(
        registry.get_platform("codex").unwrap().current_profile,
        None
    );

    let second = fixture.run_output(&["codex", "profile", "init"]);
    assert!(second.status.success(), "{:?}", second.status);
    assert!(String::from_utf8_lossy(&second.stdout).contains("已存在"));
    assert_eq!(fs::read(&profiles_path).unwrap(), profiles_before);
    assert_eq!(fs::read(&runtime_path).unwrap(), runtime_before);
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
fn codex_profile_switch_and_off_clears_runtime_route_and_auth_json() {
    let fixture = CodexProfileFixture::new();
    fixture.write_unified_codex_profile(Some("official"));
    fixture.save_codex_profiles("official");
    fixture.write_codex_runtime_official_with_credential_store("keyring");
    fixture.write_auth_json_with_oauth();

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

    let runtime_config: toml::Value =
        toml::from_str(&fs::read_to_string(fixture.codex_dir.join("config.toml")).unwrap())
            .unwrap();
    assert!(
        runtime_config
            .as_table()
            .unwrap()
            .get("forced_login_method")
            .is_none(),
        "API-key profile switching must not implicitly restrict future Codex login methods: {runtime_config:?}"
    );
    assert_eq!(
        runtime_config
            .as_table()
            .unwrap()
            .get("cli_auth_credentials_store")
            .and_then(|value| value.as_str()),
        Some("file"),
        "API-key profile switching must use file credential storage"
    );

    let runtime_auth: Value =
        serde_json::from_str(&fs::read_to_string(fixture.codex_dir.join("auth.json")).unwrap())
            .unwrap();
    assert_eq!(runtime_auth["OPENAI_API_KEY"], "sk-team-token");
    assert!(
        runtime_auth.get("tokens").is_none(),
        "API-key profile switching must clear stale OAuth tokens"
    );

    let config_path = fixture.codex_dir.join("config.toml");
    let mut runtime_config: toml::Value =
        toml::from_str(&fs::read_to_string(&config_path).unwrap()).unwrap();
    runtime_config.as_table_mut().unwrap().insert(
        "model_reasoning_effort".into(),
        toml::Value::String("xhigh".into()),
    );
    fs::write(&config_path, toml::to_string(&runtime_config).unwrap()).unwrap();

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

    let cleared_config: toml::Value =
        toml::from_str(&fs::read_to_string(fixture.codex_dir.join("config.toml")).unwrap())
            .unwrap();
    let cleared_root = cleared_config.as_table().unwrap();
    assert!(cleared_root.get("model_provider").is_none());
    assert_eq!(
        cleared_root
            .get("model_reasoning_effort")
            .and_then(toml::Value::as_str),
        Some("xhigh")
    );
    assert!(
        cleared_root
            .get("model_providers")
            .and_then(toml::Value::as_table)
            .and_then(|providers| providers.get("custom"))
            .is_none()
    );
    assert!(!fixture.codex_dir.join("auth.json").exists());
}

#[test]
fn codex_profile_switches_deepseek_bearer_and_clears_runtime_on_off() {
    const BEARER: &str = "deepseek-command-secret";

    let fixture = CodexProfileFixture::new();
    fixture.write_unified_codex_profile(None);
    fixture.save_deepseek_profile();
    fixture.write_codex_runtime_official();
    fixture.write_auth_json_with_oauth();

    let switch = fixture.run_output(&["codex", "profile", "switch", "deepseek"]);
    let switch_stdout = String::from_utf8_lossy(&switch.stdout);
    assert!(
        switch.status.success(),
        "status={:?}\nstdout:\n{}\nstderr:\n{}",
        switch.status,
        switch_stdout,
        String::from_utf8_lossy(&switch.stderr)
    );
    assert!(!switch_stdout.contains(BEARER));

    let runtime: toml::Value =
        toml::from_str(&fs::read_to_string(fixture.codex_dir.join("config.toml")).unwrap())
            .unwrap();
    let root = runtime.as_table().unwrap();
    assert_eq!(root["model"].as_str(), Some("deepseek-v4-flash"));
    assert_eq!(root["preferred_auth_method"].as_str(), Some("apikey"));
    assert_eq!(root["forced_login_method"].as_str(), Some("api"));
    assert_eq!(root["model_provider"].as_str(), Some("custom"));
    assert_eq!(
        root["model_providers"]["custom"]["experimental_bearer_token"].as_str(),
        Some(BEARER)
    );

    let auth_path = fixture.codex_dir.join("auth.json");
    let auth: Value = if auth_path.exists() {
        serde_json::from_str(&fs::read_to_string(&auth_path).unwrap()).unwrap()
    } else {
        Value::Object(Default::default())
    };
    assert!(auth.get("OPENAI_API_KEY").is_none());
    assert!(auth.get("tokens").is_none());

    let (_, current) = fixture.run_json(&["codex", "profile", "current", "--json"]);
    assert_eq!(current["profile"], "deepseek");
    assert!(!current.to_string().contains(BEARER));

    let off = fixture.run_output(&["codex", "profile", "off"]);
    assert!(off.status.success(), "{:?}", off.status);
    let cleared: toml::Value =
        toml::from_str(&fs::read_to_string(fixture.codex_dir.join("config.toml")).unwrap())
            .unwrap();
    let root = cleared.as_table().unwrap();
    assert!(root.get("model_provider").is_none());
    assert!(root.get("model_catalog_json").is_none());
    assert!(root.get("preferred_auth_method").is_none());
    assert!(root.get("forced_login_method").is_none());
    assert!(
        root.get("model_providers")
            .and_then(toml::Value::as_table)
            .and_then(|providers| providers.get("custom"))
            .and_then(toml::Value::as_table)
            .and_then(|provider| provider.get("experimental_bearer_token"))
            .is_none()
    );
    assert!(!auth_path.exists());
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
fn codex_profile_off_clears_stale_profiles_file_pointer_and_auth_json() {
    let fixture = CodexProfileFixture::new();
    fixture.write_unified_codex_profile(None);
    fixture.save_codex_profiles("team");
    fixture.write_codex_runtime_official();
    fixture.write_auth_json_with_oauth();

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

    assert!(!fixture.codex_dir.join("auth.json").exists());
}

#[test]
fn codex_profile_off_scrubs_api_key_without_snapshot_when_pointer_exists() {
    let fixture = CodexProfileFixture::new();
    fixture.write_unified_codex_profile(None);
    fixture.save_codex_profiles("team");
    fixture.write_codex_runtime_official();
    fs::write(
        fixture.codex_dir.join("auth.json"),
        r#"{"OPENAI_API_KEY":"sk-stale-codex-key"}"#,
    )
    .unwrap();

    let output = fixture.run_output(&["codex", "profile", "off"]);
    assert!(output.status.success(), "{:?}", output.status);
    let visible = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!visible.contains("sk-stale-codex-key"));

    assert!(!fixture.codex_dir.join("auth.json").exists());
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
}

#[test]
fn codex_profile_off_clears_default_home_auth_when_codex_home_redirects() {
    let fixture = CodexProfileFixture::new();
    fixture.write_unified_codex_profile(None);
    fixture.save_codex_profiles("");

    let sandbox = fixture.home.join("sandbox-codex");
    fs::create_dir_all(&sandbox).unwrap();
    fs::write(
        fixture.codex_dir.join("config.toml"),
        r#"
model_provider = "custom"
model_reasoning_effort = "xhigh"

[model_providers.custom]
name = "relay-plus-team"
base_url = "https://o10.top"
wire_api = "responses"
requires_openai_auth = true
"#,
    )
    .unwrap();
    fs::write(
        fixture.codex_dir.join("auth.json"),
        r#"{"OPENAI_API_KEY":"sk-stale-default-home"}"#,
    )
    .unwrap();

    let off = fixture
        .command_with_redirected_codex_home(&sandbox)
        .args(["codex", "profile", "off", "--json"])
        .output()
        .unwrap();
    assert!(off.status.success(), "{:?}", off.status);
    let json: Value = serde_json::from_str(&String::from_utf8_lossy(&off.stdout)).unwrap();
    assert_eq!(json["changed"], true);
    assert!(
        json["removed_auth_json"]
            .as_array()
            .unwrap()
            .iter()
            .any(|value| {
                value
                    .as_str()
                    .is_some_and(|path| path.ends_with("auth.json"))
            })
    );
    assert!(!fixture.codex_dir.join("auth.json").exists());
}

#[test]
fn codex_profile_off_keeps_official_api_key_without_pointer() {
    let fixture = CodexProfileFixture::new();
    fixture.write_unified_codex_profile(None);
    fixture.save_codex_profiles("");
    fixture.write_codex_runtime_official();
    let auth_before = r#"{"OPENAI_API_KEY":"sk-official-keep"}"#;
    fs::write(fixture.codex_dir.join("auth.json"), auth_before).unwrap();

    let output = fixture.run_output(&["codex", "profile", "off"]);
    assert!(output.status.success(), "{:?}", output.status);
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

#[test]
fn codex_auth_off_deletes_auth_json_and_keeps_profile_pointer() {
    let fixture = CodexProfileFixture::new();
    fixture.write_unified_codex_profile(Some("team"));
    fs::write(
        fixture.codex_dir.join("config.toml"),
        "cli_auth_credentials_store = \"file\"\nmodel_provider = \"custom\"\n",
    )
    .unwrap();
    fs::write(
        fixture.codex_dir.join("auth.json"),
        r#"{"OPENAI_API_KEY":"sk-runtime-secret"}"#,
    )
    .unwrap();

    let (output, json) = fixture.run_json(&["codex", "auth", "off", "--json"]);
    assert!(output.status.success(), "{:?}", output.status);
    assert_eq!(json["ok"], true);
    assert_eq!(json["changed"], true);
    assert_eq!(json["path"], "file");
    assert_eq!(json["profile_pointer"], "team");
    assert!(!String::from_utf8_lossy(&output.stdout).contains("sk-runtime-secret"));
    assert!(!fixture.codex_dir.join("auth.json").exists());

    let unified = PlatformConfigManager::new(fixture.root.join("config.toml"))
        .load()
        .unwrap();
    assert_eq!(
        unified
            .get_platform("codex")
            .unwrap()
            .current_profile
            .as_deref(),
        Some("team")
    );
}
