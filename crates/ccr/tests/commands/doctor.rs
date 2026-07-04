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

struct DoctorFixture {
    _temp_dir: TempDir,
    home: PathBuf,
    root: PathBuf,
    claude_settings_path: PathBuf,
    codex_dir: PathBuf,
}

impl DoctorFixture {
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
        command
    }

    fn run_json(&self, args: &[&str]) -> (Output, Value) {
        let output = self.command().args(args).output().unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        let json = serde_json::from_str::<Value>(&stdout).unwrap();
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

    fn write_claude_empty_settings(&self) {
        fs::write(
            &self.claude_settings_path,
            serde_json::to_string_pretty(&ClaudeSettings::default()).unwrap(),
        )
        .unwrap();
    }

    fn write_codex_config(&self, store: &str) {
        fs::write(
            self.codex_dir.join("config.toml"),
            format!("cli_auth_credentials_store = \"{store}\"\n"),
        )
        .unwrap();
    }
}

fn claude_api_key_section() -> ConfigSection {
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
    }
}

fn claude_glm_placeholder_section() -> ConfigSection {
    let mut section = ConfigSection {
        description: Some("GLM placeholder".to_string()),
        base_url: Some("https://api.z.ai/api/anthropic".to_string()),
        auth_token: Some(ccr_core::Secret::from("sk-xxx")),
        default_opus_model: Some("glm-5.2[1m]".to_string()),
        default_sonnet_model: Some("glm-5.2[1m]".to_string()),
        default_haiku_model: Some("glm-4.7".to_string()),
        default_fable_model: Some("glm-5.2[1m]".to_string()),
        provider: Some("glm".to_string()),
        provider_type: Some(ccr::managers::ProviderType::ThirdPartyModel),
        account: None,
        tags: None,
        usage_count: Some(0),
        enabled: Some(true),
        other: IndexMap::new(),
        ..Default::default()
    };
    section.other.insert(
        "auth_mode".to_string(),
        toml::Value::String("api_key".to_string()),
    );
    section
}

fn claude_subscription_section() -> ConfigSection {
    let mut other = IndexMap::new();
    other.insert(
        "auth_mode".to_string(),
        toml::Value::String("subscription".to_string()),
    );
    ConfigSection {
        description: Some("Claude subscription".to_string()),
        base_url: None,
        auth_token: None,
        model: Some("claude-sub".to_string()),
        small_fast_model: None,
        provider: None,
        provider_type: None,
        account: None,
        tags: None,
        usage_count: Some(0),
        enabled: Some(true),
        other,
        ..Default::default()
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
        ..Default::default()
    }
}

fn codex_no_auth_section() -> ConfigSection {
    ConfigSection {
        description: Some("Codex custom".to_string()),
        base_url: Some("https://api.example.com/v1".to_string()),
        auth_token: None,
        model: Some("gpt-4.1".to_string()),
        small_fast_model: None,
        provider: Some("custom".to_string()),
        provider_type: Some(ccr::managers::ProviderType::ThirdPartyModel),
        account: None,
        tags: None,
        usage_count: Some(0),
        enabled: Some(true),
        other: IndexMap::new(),
        ..Default::default()
    }
}

#[test]
fn doctor_reports_healthy_configured_claude_runtime() {
    let fixture = DoctorFixture::new();
    fixture.write_unified_config("claude", &[("claude", "main")]);
    fixture.write_profile("claude", "main", claude_api_key_section());
    fixture.write_claude_api_key_settings();

    let (output, json) = fixture.run_json(&["doctor", "--json"]);

    assert!(output.status.success(), "{:?}", output.status);
    assert_eq!(json["summary"]["failed"], 0);
    assert_eq!(
        json["scope"],
        "global + configured Claude/Codex runtimes (claude)"
    );
    let checks = json["checks"].as_array().unwrap();
    assert!(
        checks
            .iter()
            .any(|check| check["id"] == "global.claude.runtime" && check["status"] == "ok")
    );
    assert!(
        checks
            .iter()
            .any(|check| check["id"] == "global.codex.runtime" && check["status"] == "skip")
    );
    assert!(
        checks
            .iter()
            .all(|check| check["id"] != "global.current_platform")
    );
    let saved_registry = fs::read_to_string(fixture.root.join("config.toml")).unwrap();
    assert!(!saved_registry.contains("current_platform"));
    assert!(!saved_registry.contains("default_platform"));
}

#[test]
fn doctor_fails_when_claude_subscription_is_missing() {
    let fixture = DoctorFixture::new();
    fixture.write_unified_config("claude", &[("claude", "sub")]);
    fixture.write_profile("claude", "sub", claude_subscription_section());
    fixture.write_claude_empty_settings();

    let (output, json) = fixture.run_json(&["doctor", "--json"]);

    assert!(!output.status.success(), "{:?}", output.status);
    assert!(json["summary"]["failed"].as_u64().unwrap() >= 1);
    assert!(json["checks"].as_array().unwrap().iter().any(|check| {
        check["id"] == "platform.claude.runtime_auth" && check["status"] == "fail"
    }));
}

#[test]
fn doctor_warns_for_glm_placeholder_and_missing_runtime_envs() {
    let fixture = DoctorFixture::new();
    fixture.write_unified_config("claude", &[("claude", "glm")]);
    fixture.write_profile("claude", "glm", claude_glm_placeholder_section());
    fs::write(
        &fixture.claude_settings_path,
        serde_json::to_string_pretty(&ClaudeSettings {
            env: HashMap::from([
                (
                    "ANTHROPIC_BASE_URL".to_string(),
                    "https://api.z.ai/api/anthropic".to_string(),
                ),
                ("ANTHROPIC_AUTH_TOKEN".to_string(), "sk-xxx".to_string()),
                (
                    "ANTHROPIC_DEFAULT_OPUS_MODEL".to_string(),
                    "glm-5.2[1m]".to_string(),
                ),
                (
                    "ANTHROPIC_DEFAULT_SONNET_MODEL".to_string(),
                    "glm-5.2[1m]".to_string(),
                ),
                (
                    "ANTHROPIC_DEFAULT_HAIKU_MODEL".to_string(),
                    "glm-4.7".to_string(),
                ),
                (
                    "ANTHROPIC_DEFAULT_FABLE_MODEL".to_string(),
                    "glm-5.2[1m]".to_string(),
                ),
            ]),
            other: HashMap::new(),
        })
        .unwrap(),
    )
    .unwrap();

    let (output, json) = fixture.run_json(&["doctor", "--json"]);

    assert!(output.status.success(), "{:?}", output.status);
    assert!(json["summary"]["warnings"].as_u64().unwrap() >= 1);
    let settings_check = json["checks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|check| check["id"] == "platform.claude.settings_file")
        .expect("settings_file check should exist");
    assert_eq!(settings_check["status"], "warn");
    let detail = settings_check["detail"].as_str().unwrap();
    assert!(detail.contains("placeholder"), "{detail}");
    assert!(
        detail.contains("CLAUDE_CODE_AUTO_COMPACT_WINDOW"),
        "{detail}"
    );
}

#[test]
fn doctor_fails_when_codex_credential_store_is_not_file() {
    let fixture = DoctorFixture::new();
    fixture.write_unified_config("codex", &[("codex", "official")]);
    fixture.write_profile("codex", "official", codex_openai_chatgpt_section());
    fixture.write_codex_config("keychain");

    let (output, json) = fixture.run_json(&["doctor", "--platform", "codex", "--json"]);

    assert!(!output.status.success(), "{:?}", output.status);
    assert!(json["checks"].as_array().unwrap().iter().any(|check| {
        check["id"] == "platform.codex.runtime_auth" && check["status"] == "fail"
    }));
}

#[test]
fn doctor_all_platforms_includes_each_configured_platform() {
    let fixture = DoctorFixture::new();
    fixture.write_unified_config("claude", &[("claude", "main"), ("codex", "edge")]);
    fixture.write_profile("claude", "main", claude_api_key_section());
    fixture.write_profile("codex", "edge", codex_no_auth_section());
    fixture.write_claude_api_key_settings();
    fixture.write_codex_config("file");

    let (output, json) = fixture.run_json(&["doctor", "--all-platforms", "--json"]);

    assert!(output.status.success(), "{:?}", output.status);
    let checks = json["checks"].as_array().unwrap();
    assert!(
        checks
            .iter()
            .any(|check| check["id"] == "platform.claude.current_profile")
    );
    assert!(
        checks
            .iter()
            .any(|check| check["id"] == "platform.codex.current_profile")
    );
}

#[test]
fn doctor_rejects_conflicting_scope_flags() {
    let fixture = DoctorFixture::new();
    let output = fixture
        .command()
        .args(["doctor", "--platform", "claude", "--all-platforms"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("cannot be used with"));
}

#[test]
fn doctor_default_scope_does_not_follow_legacy_current_platform() {
    let fixture = DoctorFixture::new();
    fs::write(
        fixture.root.join("config.toml"),
        r#"
default_platform = "claude"
current_platform = "codex"

[claude]
enabled = true
current_profile = "main"
"#,
    )
    .unwrap();
    fixture.write_profile("claude", "main", claude_api_key_section());
    fixture.write_claude_api_key_settings();

    let (output, json) = fixture.run_json(&["doctor", "--json"]);

    assert!(output.status.success(), "{:?}", output.status);
    assert_eq!(
        json["scope"],
        "global + configured Claude/Codex runtimes (claude)"
    );
    assert!(json["checks"].as_array().unwrap().iter().all(|check| {
        check["id"] != "global.current_platform" && !check["id"].as_str().unwrap().contains("codex")
            || check["id"] == "global.codex.runtime"
    }));
}
