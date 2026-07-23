#![allow(clippy::unwrap_used)]

use ccr::managers::{
    CcsConfig, ConfigManager, GlobalSettings, PlatformConfigEntry, PlatformConfigManager,
    UnifiedConfig,
};
use ccr_config::ProfileConfig;
use indexmap::IndexMap;
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

const PROFILE_SECRET: &str = "fix-profile-secret-must-not-leak";
const RUNTIME_SECRET: &str = "fix-runtime-secret-must-not-leak";
#[cfg(unix)]
const INVALID_STORE_SENTINEL: &str = "invalid-store-secret-must-not-leak";

struct CodexFixFixture {
    _temp_dir: TempDir,
    home: PathBuf,
    root: PathBuf,
    codex_dir: PathBuf,
}

impl CodexFixFixture {
    fn new() -> Self {
        let temp_dir = tempfile::tempdir().unwrap();
        let home = temp_dir.path().join("home");
        let root = home.join(".ccr");
        let codex_dir = home.join(".codex");
        fs::create_dir_all(root.join("platforms/codex")).unwrap();
        fs::create_dir_all(&codex_dir).unwrap();

        let fixture = Self {
            _temp_dir: temp_dir,
            home,
            root,
            codex_dir,
        };
        fixture.write_profile_state();
        fixture
    }

    fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_ccr"));
        command
            .env("CCR_ROOT", &self.root)
            .env("CCR_CODEX_DIR", &self.codex_dir)
            .env("CCR_LOCK_DIR", self.home.join(".locks"))
            .env("HOME", &self.home)
            .env("USERPROFILE", &self.home)
            .env("NO_COLOR", "1")
            .env("CLICOLOR", "0")
            .env("CCR_LOG_LEVEL", "off")
            .env_remove("CODEX_HOME")
            .env_remove("CODEX_API_KEY")
            .env_remove("OPENAI_API_KEY")
            .env_remove("OPENAI_BASE_URL");
        command
    }

    fn write_profile_state(&self) {
        let registry_manager = PlatformConfigManager::new(self.root.join("config.toml"));
        let mut platforms = IndexMap::new();
        platforms.insert(
            "codex".to_string(),
            PlatformConfigEntry {
                enabled: true,
                current_profile: Some("future".to_string()),
                description: None,
                last_used: None,
            },
        );
        registry_manager
            .save(&UnifiedConfig {
                default_platform: "codex".to_string(),
                current_platform: "codex".to_string(),
                platforms,
            })
            .unwrap();

        let mut profile = ProfileConfig {
            description: Some("Future Provider".to_string()),
            base_url: Some("https://www.futureapi.cc/v1".to_string()),
            model: Some("gpt-5.6-sol".to_string()),
            provider_type: Some("third_party_model".to_string()),
            enabled: Some(true),
            ..Default::default()
        };
        profile
            .platform_data
            .insert("wire_api".to_string(), json!("responses"));
        profile
            .platform_data
            .insert("auth_mode".to_string(), json!("openai_api_key"));
        let mut sections = IndexMap::new();
        sections.insert(
            "future".to_string(),
            ccr_config::profile_to_section(&profile).unwrap(),
        );
        ConfigManager::new(self.root.join("platforms/codex/profiles.toml"))
            .save(&CcsConfig {
                default_config: "future".to_string(),
                current_config: "future".to_string(),
                settings: GlobalSettings::default(),
                sections,
            })
            .unwrap();

        fs::write(
            self.root.join("platforms/codex/profile_secrets.json"),
            serde_json::to_vec_pretty(&json!({
                "version": "1.0",
                "profiles": {
                    "future": {
                        "auth_mode": "open_ai_api_key",
                        "secret": PROFILE_SECRET,
                        "updated_at": "2026-07-22T00:00:00Z"
                    }
                }
            }))
            .unwrap(),
        )
        .unwrap();

        fs::write(
            self.codex_dir.join("config.toml"),
            r#"cli_auth_credentials_store = "file"
model = "gpt-5.6-sol"
model_provider = "custom"

[model_providers.custom]
name = "Future Provider"
base_url = "https://www.futureapi.cc/v1"
wire_api = "responses"
requires_openai_auth = true
"#,
        )
        .unwrap();
        fs::write(
            self.codex_dir.join("auth.json"),
            serde_json::to_vec_pretty(&json!({ "OPENAI_API_KEY": RUNTIME_SECRET })).unwrap(),
        )
        .unwrap();
    }

    fn managed_paths(&self) -> Vec<PathBuf> {
        vec![
            self.root.join("config.toml"),
            self.root.join("platforms/codex/profiles.toml"),
            self.root.join("platforms/codex/profile_secrets.json"),
            self.codex_dir.join("config.toml"),
            self.codex_dir.join("auth.json"),
        ]
    }

    #[cfg(unix)]
    fn install_fake_codex_doctor(&self) -> PathBuf {
        use std::os::unix::fs::PermissionsExt;

        let bin_dir = self.home.join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let codex = bin_dir.join("codex");
        fs::write(
            &codex,
            "#!/bin/sh\nprintf '%s\\n' '{\"schemaVersion\":1,\"overallStatus\":\"ok\",\"codexVersion\":\"fixture-doctor\",\"checks\":{}}'\n",
        )
        .unwrap();
        let mut permissions = fs::metadata(&codex).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&codex, permissions).unwrap();
        bin_dir
    }
}

#[test]
fn codex_fix_dry_run_repair_reports_drift_without_writing_runtime() {
    let fixture = CodexFixFixture::new();
    let before = snapshot_files(&fixture.managed_paths());
    let output = fixture
        .command()
        .env("PATH", "")
        .args(["codex", "fix", "--dry-run", "--repair-runtime"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(127));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("process_state ="));
    assert!(stdout.contains("resolved profile = future"));
    assert!(stdout.contains("credential consistency = mismatch"));
    assert!(stdout.contains("provider_auth_validity = not_checked"));
    assert!(stdout.contains("--dry-run"));
    assert!(!stdout.contains(PROFILE_SECRET));
    assert!(!stdout.contains(RUNTIME_SECRET));

    let after = snapshot_files(&fixture.managed_paths());
    assert!(before == after);
}

#[cfg(unix)]
#[test]
fn codex_fix_runs_doctor_when_runtime_inspection_is_unavailable() {
    let fixture = CodexFixFixture::new();
    let fake_bin = fixture.install_fake_codex_doctor();
    let secret_store = fixture.root.join("platforms/codex/profile_secrets.json");
    fs::write(
        &secret_store,
        format!("{{ invalid json: {INVALID_STORE_SENTINEL}"),
    )
    .unwrap();
    let before = snapshot_files(&fixture.managed_paths());

    let output = fixture
        .command()
        .env("PATH", fake_bin)
        .args(["codex", "fix", "--dry-run"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("process_state ="));
    assert!(stdout.contains("runtime_consistency = unavailable"));
    assert!(stdout.contains("codexVersion = fixture-doctor"));
    assert!(!stdout.contains(INVALID_STORE_SENTINEL));
    assert!(!stdout.contains(PROFILE_SECRET));
    assert!(!stdout.contains(RUNTIME_SECRET));

    let after = snapshot_files(&fixture.managed_paths());
    assert_eq!(before, after);
}

fn snapshot_files(paths: &[PathBuf]) -> Vec<(PathBuf, Vec<u8>)> {
    paths
        .iter()
        .map(|path| (path.clone(), read(path)))
        .collect()
}

fn read(path: &Path) -> Vec<u8> {
    fs::read(path).unwrap()
}
