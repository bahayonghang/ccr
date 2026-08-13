#![allow(clippy::unwrap_used)]

use ccr_cli::managers::{
    CcsConfig, ConfigManager, GlobalSettings, PlatformConfigEntry, PlatformConfigManager,
    UnifiedConfig,
};
#[cfg(not(target_os = "macos"))]
use ccr_cli::models::ClaudeAuthRegistry;
use ccr_config::ProfileConfig;
use indexmap::IndexMap;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use tempfile::TempDir;

struct ClaudeProfileFixture {
    _temp_dir: TempDir,
    home: PathBuf,
    root: PathBuf,
    claude_dir: PathBuf,
}

impl ClaudeProfileFixture {
    fn new() -> Self {
        let temp_dir = tempfile::tempdir().unwrap();
        let home = temp_dir.path().join("home");
        let root = home.join(".ccr");
        let claude_dir = home.join(".claude");

        fs::create_dir_all(&home).unwrap();
        fs::create_dir_all(root.join("platforms").join("claude")).unwrap();
        fs::create_dir_all(&claude_dir).unwrap();

        Self {
            _temp_dir: temp_dir,
            home,
            root,
            claude_dir,
        }
    }

    fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_ccr"));
        command.env("CCR_ROOT", &self.root);
        command.env("CLAUDE_CONFIG_DIR", &self.claude_dir);
        command.env("CLAUDE_JSON_PATH", self.home.join(".claude.json"));
        command.env("CCR_SETTINGS_PATH", self.claude_dir.join("settings.json"));
        command.env("CCR_BACKUP_DIR", self.claude_dir.join("backups"));
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
        let json = serde_json::from_str::<Value>(&stdout).unwrap();
        (output, json)
    }

    fn write_unified_claude_profile(&self, current_profile: Option<&str>) {
        let manager = PlatformConfigManager::new(self.root.join("config.toml"));
        let mut unified = UnifiedConfig {
            default_platform: "claude".to_string(),
            current_platform: "claude".to_string(),
            platforms: IndexMap::new(),
        };
        unified.platforms.insert(
            "claude".to_string(),
            PlatformConfigEntry {
                enabled: true,
                current_profile: current_profile.map(str::to_string),
                description: None,
                last_used: None,
            },
        );
        manager.save(&unified).unwrap();
    }

    fn save_claude_profiles(&self, current_config: &str) {
        let manager = ConfigManager::new(
            self.root
                .join("platforms")
                .join("claude")
                .join("profiles.toml"),
        );

        let mut subscription = ProfileConfig {
            description: Some("Subscription".to_string()),
            ..Default::default()
        };
        subscription.platform_data.insert(
            "auth_mode".to_string(),
            Value::String("subscription".to_string()),
        );

        let mut proxy = ProfileConfig {
            description: Some("Proxy".to_string()),
            base_url: Some("https://anthropic-proxy.example.com".to_string()),
            auth_token: Some(ccr_core::Secret::from("sk-claude-proxy")),
            provider: Some("anyrouter".to_string()),
            model: Some("claude-sonnet-4-20250514".to_string()),
            ..Default::default()
        };
        proxy.platform_data.insert(
            "auth_mode".to_string(),
            Value::String("api_key".to_string()),
        );

        let mut sections = IndexMap::new();
        sections.insert(
            "subscription".to_string(),
            ccr_config::profile_to_section(&subscription).unwrap(),
        );
        sections.insert(
            "proxy".to_string(),
            ccr_config::profile_to_section(&proxy).unwrap(),
        );

        manager
            .save(&CcsConfig {
                default_config: "subscription".to_string(),
                current_config: current_config.to_string(),
                settings: GlobalSettings::default(),
                sections,
            })
            .unwrap();
    }

    #[cfg(not(target_os = "macos"))]
    fn write_official_runtime_login(&self) {
        fs::write(
            self.claude_dir.join(".credentials.json"),
            serde_json::to_string_pretty(&serde_json::json!({
                "claudeAiOauth": {
                    "accessToken": "access-token",
                    "refreshToken": "refresh-token",
                    "expiresAt": "2099-01-01T00:00:00Z",
                    "subscriptionType": "pro",
                    "rateLimitTier": "default_claude_ai",
                    "scopes": ["user:profile"]
                }
            }))
            .unwrap(),
        )
        .unwrap();

        fs::write(
            self.home.join(".claude.json"),
            serde_json::to_string_pretty(&serde_json::json!({
                "oauthAccount": {
                    "accountUuid": "account-123",
                    "emailAddress": "user@example.com",
                    "billingType": "apple_subscription"
                }
            }))
            .unwrap(),
        )
        .unwrap();
    }

    fn write_settings_overrides(&self) {
        fs::write(
            self.claude_dir.join("settings.json"),
            serde_json::to_string_pretty(&serde_json::json!({
                "env": {
                    "ANTHROPIC_BASE_URL": "https://anthropic-proxy.example.com",
                    "ANTHROPIC_AUTH_TOKEN": "sk-claude-proxy",
                    "ANTHROPIC_MODEL": "claude-sonnet-4-20250514"
                }
            }))
            .unwrap(),
        )
        .unwrap();
    }

    #[cfg(not(target_os = "macos"))]
    fn save_official_account_snapshot(&self, name: &str) {
        let output = self.run_output(&["claude", "auth", "save", name]);
        assert!(
            output.status.success(),
            "status={:?}\nstdout:\n{}\nstderr:\n{}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn claude_profile_init_is_inactive_idempotent_and_preserves_runtime() {
    let fixture = ClaudeProfileFixture::new();
    let runtime_path = fixture.claude_dir.join("settings.json");
    let runtime_before = br#"{"theme":"dark"}"#;
    fs::write(&runtime_path, runtime_before).unwrap();
    let profiles_path = fixture
        .root
        .join("platforms")
        .join("claude")
        .join("profiles.toml");

    let first = fixture.run_output(&["claude", "profile", "init"]);
    assert!(first.status.success(), "{:?}", first.status);
    let profiles_before = fs::read(&profiles_path).unwrap();
    assert_eq!(
        profiles_before,
        include_bytes!("../../../../examples/claude/profiles.example.toml")
    );
    assert_eq!(fs::read(&runtime_path).unwrap(), runtime_before);

    let registry = PlatformConfigManager::new(fixture.root.join("config.toml"))
        .load()
        .unwrap();
    assert_eq!(
        registry.get_platform("claude").unwrap().current_profile,
        None
    );

    let second = fixture.run_output(&["claude", "profile", "init"]);
    assert!(second.status.success(), "{:?}", second.status);
    assert!(String::from_utf8_lossy(&second.stdout).contains("已存在"));
    assert_eq!(fs::read(&profiles_path).unwrap(), profiles_before);
    assert_eq!(fs::read(&runtime_path).unwrap(), runtime_before);
}

#[test]
fn claude_profile_list_json_reports_current_profile() {
    let fixture = ClaudeProfileFixture::new();
    fixture.write_unified_claude_profile(Some("proxy"));
    fixture.save_claude_profiles("proxy");

    let (output, json) = fixture.run_json(&["claude", "profile", "list", "--json"]);

    assert!(output.status.success(), "{:?}", output.status);
    assert_eq!(json["current_profile"], "proxy");
    assert!(
        json["profiles"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["name"] == "proxy" && item["is_current"] == true)
    );
}

#[test]
fn claude_profile_current_json_returns_single_claude_card() {
    let fixture = ClaudeProfileFixture::new();
    fixture.write_unified_claude_profile(Some("proxy"));
    fixture.save_claude_profiles("proxy");

    let (output, json) = fixture.run_json(&["claude", "profile", "current", "--json"]);

    assert!(output.status.success(), "{:?}", output.status);
    assert_eq!(json["platform"], "claude");
    assert_eq!(json["profile"], "proxy");
    assert!(json.get("schema_version").is_none());
    assert!(json.get("codex").is_none());
}

#[cfg(not(target_os = "macos"))]
#[test]
fn claude_profile_switch_and_off_keep_official_auth_current() {
    let fixture = ClaudeProfileFixture::new();
    fixture.write_unified_claude_profile(Some("subscription"));
    fixture.save_claude_profiles("subscription");
    fixture.write_official_runtime_login();
    fixture.save_official_account_snapshot("work");

    let switch_output = fixture.run_output(&["claude", "profile", "switch", "proxy"]);
    assert!(
        switch_output.status.success(),
        "status={:?}\nstdout:\n{}\nstderr:\n{}",
        switch_output.status,
        String::from_utf8_lossy(&switch_output.stdout),
        String::from_utf8_lossy(&switch_output.stderr)
    );

    let settings_before: Value = serde_json::from_str(
        &fs::read_to_string(fixture.claude_dir.join("settings.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        settings_before["env"]["ANTHROPIC_AUTH_TOKEN"],
        Value::String("sk-claude-proxy".to_string())
    );

    let (_, current_json) = fixture.run_json(&["claude", "profile", "current", "--json"]);
    assert_eq!(current_json["profile"], "proxy");

    let off_output = fixture.run_output(&["claude", "profile", "off"]);
    assert!(off_output.status.success(), "{:?}", off_output.status);

    let registry = PlatformConfigManager::new(fixture.root.join("config.toml"))
        .load()
        .unwrap();
    assert_eq!(
        registry
            .get_platform("claude")
            .unwrap()
            .current_profile
            .as_deref(),
        None
    );

    let profiles = ConfigManager::new(
        fixture
            .root
            .join("platforms")
            .join("claude")
            .join("profiles.toml"),
    )
    .load()
    .unwrap();
    assert_eq!(profiles.current_config, "");

    let settings_after: Value = serde_json::from_str(
        &fs::read_to_string(fixture.claude_dir.join("settings.json")).unwrap(),
    )
    .unwrap();
    let env = settings_after["env"].as_object().unwrap();
    assert!(!env.contains_key("ANTHROPIC_BASE_URL"));
    assert!(!env.contains_key("ANTHROPIC_AUTH_TOKEN"));
    assert!(!env.contains_key("ANTHROPIC_MODEL"));

    let (_, auth_current_json) = fixture.run_json(&["claude", "auth", "current", "--json"]);
    assert_eq!(
        auth_current_json["runtime_summary"]["current_auth_name"],
        "work"
    );
    assert_eq!(
        auth_current_json["runtime_summary"]["mode"],
        Value::String("runtime_only".to_string())
    );

    let auth_registry: ClaudeAuthRegistry = toml::from_str(
        &fs::read_to_string(
            fixture
                .root
                .join("platforms")
                .join("claude")
                .join("auth_registry.toml"),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(auth_registry.current_auth.as_deref(), Some("work"));
    assert!(
        fixture
            .root
            .join("platforms")
            .join("claude")
            .join("auth")
            .join("work.json")
            .exists()
    );
}

#[cfg(not(target_os = "macos"))]
#[test]
fn claude_profile_off_is_idempotent_when_no_active_profile() {
    let fixture = ClaudeProfileFixture::new();
    fixture.write_unified_claude_profile(None);
    fixture.save_claude_profiles("");
    fixture.write_official_runtime_login();
    fixture.save_official_account_snapshot("work");

    let output = fixture.run_output(&["claude", "profile", "off"]);
    assert!(output.status.success(), "{:?}", output.status);

    let (_, auth_current_json) = fixture.run_json(&["claude", "auth", "current", "--json"]);
    assert_eq!(
        auth_current_json["runtime_summary"]["current_auth_name"],
        "work"
    );
    assert_eq!(
        auth_current_json["runtime_summary"]["mode"],
        Value::String("runtime_only".to_string())
    );
}

#[test]
fn claude_profile_off_clears_stale_profiles_file_pointer_and_settings_overrides() {
    let fixture = ClaudeProfileFixture::new();
    fixture.write_unified_claude_profile(None);
    fixture.save_claude_profiles("proxy");
    fixture.write_settings_overrides();

    let output = fixture.run_output(&["claude", "profile", "off"]);
    assert!(output.status.success(), "{:?}", output.status);

    let profiles = ConfigManager::new(
        fixture
            .root
            .join("platforms")
            .join("claude")
            .join("profiles.toml"),
    )
    .load()
    .unwrap();
    assert_eq!(profiles.current_config, "");

    let settings_after: Value = serde_json::from_str(
        &fs::read_to_string(fixture.claude_dir.join("settings.json")).unwrap(),
    )
    .unwrap();
    let env = settings_after["env"].as_object().unwrap();
    assert!(!env.contains_key("ANTHROPIC_BASE_URL"));
    assert!(!env.contains_key("ANTHROPIC_AUTH_TOKEN"));
    assert!(!env.contains_key("ANTHROPIC_MODEL"));
}

#[test]
fn claude_profile_crud_commands_support_vscode_surface() {
    let fixture = ClaudeProfileFixture::new();

    let (create_output, create_json) = fixture.run_json(&[
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
        "--provider",
        "proxy",
        "--tag",
        "team",
        "--json",
    ]);
    assert!(create_output.status.success(), "{:?}", create_output.status);
    assert_eq!(create_json["platform"], "claude");
    assert_eq!(create_json["name"], "work");
    assert_eq!(create_json["enabled"], true);

    let (set_output, set_json) = fixture.run_json(&[
        "claude",
        "profile",
        "set-field",
        "work",
        "model",
        "--value",
        "claude-opus-4-20250514",
        "--json",
    ]);
    assert!(set_output.status.success(), "{:?}", set_output.status);
    assert_eq!(set_json["name"], "work");

    let (disable_output, disable_json) =
        fixture.run_json(&["claude", "profile", "disable", "work", "--force", "--json"]);
    assert!(
        disable_output.status.success(),
        "{:?}",
        disable_output.status
    );
    assert_eq!(disable_json["enabled"], false);

    let (enable_output, enable_json) =
        fixture.run_json(&["claude", "profile", "enable", "work", "--json"]);
    assert!(enable_output.status.success(), "{:?}", enable_output.status);
    assert_eq!(enable_json["enabled"], true);

    let (delete_output, delete_json) =
        fixture.run_json(&["claude", "profile", "delete", "work", "--force", "--json"]);
    assert!(delete_output.status.success(), "{:?}", delete_output.status);
    assert_eq!(delete_json["name"], "work");
}

#[test]
fn claude_profile_off_clears_managed_env_without_pointer_and_keeps_user_key() {
    let fixture = ClaudeProfileFixture::new();
    fixture.write_unified_claude_profile(None);
    fixture.save_claude_profiles("");
    fs::write(
        fixture.claude_dir.join("settings.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "env": {
                "ANTHROPIC_BASE_URL": "https://anthropic-proxy.example.com",
                "ANTHROPIC_AUTH_TOKEN": "sk-claude-proxy",
                "ANTHROPIC_MODEL": "claude-sonnet-4-20250514",
                "ANTHROPIC_API_KEY": "user-owned-key"
            }
        }))
        .unwrap(),
    )
    .unwrap();

    let (output, json) = fixture.run_json(&["claude", "profile", "off", "--json"]);
    assert!(output.status.success(), "{:?}", output.status);
    assert_eq!(json["ok"], true);
    assert_eq!(json["changed"], true);
    assert_eq!(json["runtime_mode"], "official_auth");
    assert!(!output_contains_secret(&output, "sk-claude-proxy"));
    assert!(!output_contains_secret(&output, "user-owned-key"));

    let settings_after: Value = serde_json::from_str(
        &fs::read_to_string(fixture.claude_dir.join("settings.json")).unwrap(),
    )
    .unwrap();
    let env = settings_after["env"].as_object().unwrap();
    assert!(!env.contains_key("ANTHROPIC_BASE_URL"));
    assert!(!env.contains_key("ANTHROPIC_AUTH_TOKEN"));
    assert!(!env.contains_key("ANTHROPIC_MODEL"));
    assert_eq!(env["ANTHROPIC_API_KEY"], "user-owned-key");
}

fn output_contains_secret(output: &Output, secret: &str) -> bool {
    String::from_utf8_lossy(&output.stdout).contains(secret)
        || String::from_utf8_lossy(&output.stderr).contains(secret)
}
