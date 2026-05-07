#![allow(clippy::unwrap_used)]

use ccr::managers::{
    CcsConfig, ClaudeSettings, ConfigManager, ConfigSection, GlobalSettings, PlatformConfigEntry,
    PlatformConfigManager, UnifiedConfig,
};
use indexmap::IndexMap;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use tempfile::TempDir;

struct CurrentFixture {
    _temp_dir: TempDir,
    home: PathBuf,
    root: PathBuf,
    claude_settings_path: PathBuf,
    codex_dir: PathBuf,
}

impl CurrentFixture {
    fn new() -> Self {
        let temp_dir = tempfile::tempdir().unwrap();
        let home = temp_dir.path().join("home");
        let root = home.join(".ccr");
        let claude_settings_path = home.join(".claude").join("settings.json");
        let codex_dir = home.join(".codex");

        fs::create_dir_all(&home).unwrap();
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
        command.env("CCR_SETTINGS_PATH", &self.claude_settings_path);
        command.env("CCR_BACKUP_DIR", self.home.join(".claude").join("backups"));
        command.env("CCR_CODEX_DIR", &self.codex_dir);
        command.env("CLAUDE_CONFIG_DIR", self.home.join(".claude"));
        command.env("HOME", &self.home);
        command.env("USERPROFILE", &self.home);
        command.env("NO_COLOR", "1");
        command.env("CLICOLOR", "0");
        command.env("COLUMNS", "120");
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

    fn write_unified_config(&self, current_platform: &str, entries: &[(&str, &str)]) {
        let manager = PlatformConfigManager::new(self.root.join("config.toml"));
        let mut unified = UnifiedConfig {
            default_platform: current_platform.to_string(),
            current_platform: current_platform.to_string(),
            platforms: IndexMap::new(),
        };

        for (platform, profile) in entries {
            unified.platforms.insert(
                (*platform).to_string(),
                PlatformConfigEntry {
                    enabled: true,
                    current_profile: Some((*profile).to_string()),
                    description: None,
                    last_used: None,
                },
            );
        }

        manager.save(&unified).unwrap();
    }

    fn write_profile(&self, platform: &str, name: &str, section: ConfigSection) {
        let manager = ConfigManager::new(
            self.root
                .join("platforms")
                .join(platform)
                .join("profiles.toml"),
        );
        let mut config = CcsConfig {
            default_config: name.to_string(),
            current_config: name.to_string(),
            settings: GlobalSettings::default(),
            sections: IndexMap::new(),
        };
        config.sections.insert(name.to_string(), section);
        manager.save(&config).unwrap();
    }

    fn write_claude_api_key_settings(&self) {
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
                other: HashMap::new(),
            })
            .unwrap(),
        )
        .unwrap();
    }

    fn write_codex_official_runtime_config(&self, store: &str) {
        let content = format!(
            r#"
cli_auth_credentials_store = "{store}"
model_provider = "custom"

[model_providers.custom]
name = "openai"
base_url = "https://api.openai.com/v1"
wire_api = "responses"
requires_openai_auth = true
"#
        );
        fs::write(self.codex_dir.join("config.toml"), content.trim_start()).unwrap();
    }
}

fn claude_api_key_section() -> ConfigSection {
    ConfigSection {
        description: Some("Claude API key".to_string()),
        base_url: Some("https://api.example.com".to_string()),
        auth_token: Some("sk-claude-test".to_string()),
        model: Some("claude-test".to_string()),
        small_fast_model: None,
        provider: Some("example".to_string()),
        provider_type: None,
        account: None,
        tags: None,
        usage_count: Some(0),
        enabled: Some(true),
        other: IndexMap::new(),
    }
}

fn codex_openai_chatgpt_section() -> ConfigSection {
    let mut other = IndexMap::new();
    other.insert(
        "auth_mode".to_string(),
        toml::Value::String("openai_chatgpt".to_string()),
    );
    ConfigSection {
        description: Some("Codex official".to_string()),
        base_url: None,
        auth_token: None,
        model: Some("gpt-5".to_string()),
        small_fast_model: None,
        provider: Some("openai".to_string()),
        provider_type: None,
        account: None,
        tags: None,
        usage_count: Some(0),
        enabled: Some(true),
        other,
    }
}

#[test]
fn current_json_uses_runtime_overview_schema_without_legacy_current_platform() {
    let fixture = CurrentFixture::new();

    let (output, json) = fixture.run_json(&["current", "--json"]);

    assert!(output.status.success(), "{:?}", output.status);
    assert_eq!(json["schema_version"], 2);
    assert!(json["generated_at"].is_string());
    assert!(json.get("current_platform").is_none());
    assert_eq!(json["claude"]["platform"], "claude");
    assert_eq!(json["codex"]["platform"], "codex");
}

#[test]
fn current_default_output_centers_runtime_cards_not_current_platform_banner() {
    let fixture = CurrentFixture::new();

    let output = fixture.run_output(&["current"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success(), "{:?}", output.status);
    assert!(stdout.contains("当前运行状态"));
    assert!(stdout.contains("Claude Code"));
    assert!(stdout.contains("Codex"));
    assert!(!stdout.contains("当前平台:"));
}

#[test]
fn current_json_reports_mixed_runtime_states_with_legacy_registry_present() {
    let fixture = CurrentFixture::new();
    fixture.write_unified_config("codex", &[("claude", "main"), ("codex", "official")]);
    fixture.write_profile("claude", "main", claude_api_key_section());
    fixture.write_profile("codex", "official", codex_openai_chatgpt_section());
    fixture.write_claude_api_key_settings();
    fixture.write_codex_official_runtime_config("file");

    let (output, json) = fixture.run_json(&["current", "--json"]);

    assert!(output.status.success(), "{:?}", output.status);
    assert!(json.get("current_platform").is_none());
    assert_eq!(json["claude"]["profile"], "main");
    assert_eq!(json["claude"]["auth_kind"], "third_party_api");
    assert_eq!(json["claude"]["health"], "ready");
    assert_eq!(json["codex"]["profile"], "official");
    assert_eq!(json["codex"]["auth_kind"], "missing");
    assert_eq!(json["codex"]["health"], "needs_login");
}
