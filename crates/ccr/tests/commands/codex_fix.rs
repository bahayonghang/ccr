#![allow(clippy::unwrap_used)]

use ccr_cli::managers::{
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
const INVALID_STORE_SENTINEL: &str = "invalid-store-secret-must-not-leak";
const DOCTOR_SECRET: &str = "doctor-secret-must-not-leak";

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

    fn write_deepseek_profile_state(&self) {
        let catalog = self.codex_dir.join("models.json");
        fs::write(&catalog, "[]").unwrap();
        let mut profile = ProfileConfig {
            description: Some("DeepSeek".to_string()),
            base_url: Some("https://api.deepseek.com/".to_string()),
            model: Some("deepseek-v4-flash".to_string()),
            provider: Some("deepseek".to_string()),
            provider_type: Some("third_party_model".to_string()),
            enabled: Some(true),
            ..Default::default()
        };
        profile
            .platform_data
            .insert("wire_api".to_string(), json!("responses"));
        profile
            .platform_data
            .insert("auth_mode".to_string(), json!("provider_bearer_token"));
        profile.platform_data.insert(
            "model_catalog_json".to_string(),
            json!(catalog.display().to_string()),
        );
        profile
            .platform_data
            .insert("model_reasoning_effort".to_string(), json!("high"));

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
                        "auth_mode": "provider_bearer_token",
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
            format!(
                r#"cli_auth_credentials_store = "file"
model = "deepseek-v4-flash"
model_provider = "custom"
model_catalog_json = "C:/missing/deepseek-models.json"
preferred_auth_method = "chatgpt"
forced_login_method = "chatgpt"

[model_providers.custom]
name = "DeepSeek"
base_url = "https://api.deepseek.com/"
wire_api = "responses"
requires_openai_auth = false
experimental_bearer_token = "{RUNTIME_SECRET}"
"#
            ),
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

    fn doctor_count_path(&self) -> PathBuf {
        self.home.join("doctor-count.txt")
    }

    fn doctor_spawn_count(&self) -> usize {
        fs::read_to_string(self.doctor_count_path())
            .map(|text| text.lines().filter(|line| !line.is_empty()).count())
            .unwrap_or(0)
    }

    fn install_fake_codex(&self, payload: &str) -> PathBuf {
        let bin_dir = self.home.join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let payload_path = self.home.join("doctor-payload.txt");
        fs::write(&payload_path, payload).unwrap();
        write_fake_codex_script(&bin_dir, &self.doctor_count_path(), &payload_path);
        bin_dir
    }

    fn install_fake_codex_json(&self) -> PathBuf {
        self.install_fake_codex(
            r#"{"schemaVersion":1,"overallStatus":"ok","codexVersion":"fixture-doctor","checks":{}}"#,
        )
    }

    fn install_fake_codex_json_with_secret(&self) -> PathBuf {
        self.install_fake_codex(&format!(
            r#"{{"schemaVersion":1,"overallStatus":"ok","codexVersion":"fixture-doctor","checks":{{"auth.credentials":{{"status":"ok","details":{{"OPENAI_API_KEY":"{DOCTOR_SECRET}"}}}}}}}}"#
        ))
    }

    fn install_fake_codex_text(&self) -> PathBuf {
        self.install_fake_codex("not-json-doctor-output\n")
    }

    fn install_fake_codex_warning(&self) -> PathBuf {
        self.install_fake_codex(
            r#"{"schemaVersion":1,"overallStatus":"warning","codexVersion":"fixture-doctor","checks":{"state.rollout_db_parity":{"status":"warning","details":{"search provider":"none","configured servers":[]}},"config.load":{"status":"ok","details":{"model provider":"custom"}}}}"#,
        )
    }

    fn make_runtime_match(&self) {
        fs::write(
            self.codex_dir.join("auth.json"),
            serde_json::to_vec_pretty(&json!({ "OPENAI_API_KEY": PROFILE_SECRET })).unwrap(),
        )
        .unwrap();
    }

    fn install_fake_codex_mutating_profiles(&self) -> PathBuf {
        let profiles = self.root.join("platforms/codex/profiles.toml");
        let mutated = self.home.join("mutated-profiles.toml");
        let original = fs::read_to_string(&profiles).unwrap();
        fs::write(
            &mutated,
            original.replace(
                "current_config = \"future\"",
                "current_config = \"mutated\"",
            ),
        )
        .unwrap();
        let bin_dir = self.home.join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let payload_path = self.home.join("doctor-payload.txt");
        fs::write(
            &payload_path,
            r#"{"schemaVersion":1,"overallStatus":"ok","codexVersion":"fixture-doctor","checks":{}}"#,
        )
        .unwrap();
        write_fake_codex_script_with_copy(
            &bin_dir,
            &self.doctor_count_path(),
            &payload_path,
            &mutated,
            &profiles,
        );
        bin_dir
    }
}

#[cfg(windows)]
fn write_fake_codex_script(bin_dir: &Path, count_path: &Path, payload_path: &Path) {
    let count = format!("\"{}\"", count_path.display());
    let payload = format!("\"{}\"", payload_path.display());
    let script = format!("@echo off\r\n>>{count} echo 1\r\ntype {payload}\r\n");
    fs::write(bin_dir.join("codex.cmd"), script).unwrap();
}

#[cfg(unix)]
fn write_fake_codex_script(bin_dir: &Path, count_path: &Path, payload_path: &Path) {
    use std::os::unix::fs::PermissionsExt;

    let count = format!(
        "'{}'",
        count_path.display().to_string().replace('\'', "'\\''")
    );
    let payload = format!(
        "'{}'",
        payload_path.display().to_string().replace('\'', "'\\''")
    );
    let script = format!("#!/bin/sh\nprintf '1\\n' >> {count}\n/bin/cat {payload}\n");
    let path = bin_dir.join("codex");
    fs::write(&path, script).unwrap();
    let mut permissions = fs::metadata(&path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).unwrap();
}

#[cfg(windows)]
fn write_fake_codex_script_with_copy(
    bin_dir: &Path,
    count_path: &Path,
    payload_path: &Path,
    source: &Path,
    dest: &Path,
) {
    let script = format!(
        "@echo off\r\n>>{count} echo 1\r\ncopy /Y {source} {dest} >NUL\r\ntype {payload}\r\n",
        count = format!("\"{}\"", count_path.display()),
        source = format!("\"{}\"", source.display()),
        dest = format!("\"{}\"", dest.display()),
        payload = format!("\"{}\"", payload_path.display()),
    );
    fs::write(bin_dir.join("codex.cmd"), script).unwrap();
}

#[cfg(unix)]
fn write_fake_codex_script_with_copy(
    bin_dir: &Path,
    count_path: &Path,
    payload_path: &Path,
    source: &Path,
    dest: &Path,
) {
    use std::os::unix::fs::PermissionsExt;

    let script = format!(
        "#!/bin/sh\nprintf '1\\n' >> {count}\n/bin/cp {source} {dest}\n/bin/cat {payload}\n",
        count = sh_single_quote(count_path),
        source = sh_single_quote(source),
        dest = sh_single_quote(dest),
        payload = sh_single_quote(payload_path),
    );
    let path = bin_dir.join("codex");
    fs::write(&path, script).unwrap();
    let mut permissions = fs::metadata(&path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).unwrap();
}

#[cfg(unix)]
fn sh_single_quote(path: &Path) -> String {
    format!("'{}'", path.display().to_string().replace('\'', "'\\''"))
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

    assert_eq!(output.status.code(), Some(3));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("process_state ="));
    assert!(stdout.contains("resolved profile = future"));
    assert!(stdout.contains("credential consistency = mismatch"));
    assert!(stdout.contains("provider_auth_validity = not_checked"));
    assert!(stdout.contains("--dry-run"));
    assert!(stdout.contains("doctor = skipped"));
    assert!(stdout.contains("ccr codex fix --doctor"));
    assert!(!stdout.contains(PROFILE_SECRET));
    assert!(!stdout.contains(RUNTIME_SECRET));

    let after = snapshot_files(&fixture.managed_paths());
    assert!(before == after);
}

#[test]
fn codex_fix_repair_runtime_restores_deepseek_fields_without_secret_output() {
    let fixture = CodexFixFixture::new();
    fixture.write_deepseek_profile_state();

    let output = fixture
        .command()
        .env("PATH", "")
        .args(["codex", "fix", "--repair-runtime"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Codex runtime 本地漂移已修复"));
    assert!(stdout.contains("doctor = skipped"));
    assert!(!stdout.contains(PROFILE_SECRET));
    assert!(!stdout.contains(RUNTIME_SECRET));

    let config: toml::Value =
        toml::from_str(&fs::read_to_string(fixture.codex_dir.join("config.toml")).unwrap())
            .unwrap();
    let root = config.as_table().unwrap();
    assert_eq!(
        root["model_catalog_json"].as_str(),
        Some(
            fixture
                .codex_dir
                .join("models.json")
                .display()
                .to_string()
                .as_str()
        )
    );
    assert_eq!(root["preferred_auth_method"].as_str(), Some("apikey"));
    assert_eq!(root["forced_login_method"].as_str(), Some("api"));
    assert_eq!(
        root["model_providers"]["custom"]["experimental_bearer_token"].as_str(),
        Some(PROFILE_SECRET)
    );
    let auth_path = fixture.codex_dir.join("auth.json");
    let auth: serde_json::Value = if auth_path.exists() {
        serde_json::from_str(&fs::read_to_string(auth_path).unwrap()).unwrap()
    } else {
        json!({})
    };
    assert!(auth.get("OPENAI_API_KEY").is_none());
    assert!(auth.get("tokens").is_none());
}

#[test]
fn codex_fix_runs_doctor_when_runtime_inspection_is_unavailable() {
    let fixture = CodexFixFixture::new();
    let fake_bin = fixture.install_fake_codex_json();
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
        .args(["codex", "fix", "--dry-run", "--doctor"])
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

#[test]
fn codex_fix_default_path_does_not_invoke_codex_binary() {
    let fixture = CodexFixFixture::new();
    let fake_bin = fixture.install_fake_codex_json();

    let output = fixture
        .command()
        .env("PATH", fake_bin)
        .args(["codex", "fix", "--dry-run"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(3));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("doctor = skipped"));
    assert!(stdout.contains("ccr codex fix --doctor"));
    assert!(stdout.contains("进程清理"));
    assert!(stdout.contains("本地诊断"));
    assert!(stdout.contains("skipped"));
    assert!(!stdout.contains("codexVersion = fixture-doctor"));
    assert_eq!(fixture.doctor_spawn_count(), 0);
}

#[test]
fn codex_fix_doctor_missing_binary_exits_127() {
    let fixture = CodexFixFixture::new();
    let output = fixture
        .command()
        .env("PATH", "")
        .args(["codex", "fix", "--dry-run", "--doctor"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(127));
}

#[test]
fn codex_fix_doctor_json_renders_version_and_status() {
    let fixture = CodexFixFixture::new();
    let fake_bin = fixture.install_fake_codex_json();

    let output = fixture
        .command()
        .env("PATH", &fake_bin)
        .args(["codex", "fix", "--dry-run", "--doctor"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(3));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("codexVersion = fixture-doctor"));
    assert!(stdout.contains("overallStatus = ok"));
    assert!(!stdout.contains("完整报告已保存到"));
    assert_eq!(fixture.doctor_spawn_count(), 1);
}

#[test]
fn codex_fix_doctor_non_json_spawns_once_and_renders_text() {
    let fixture = CodexFixFixture::new();
    let fake_bin = fixture.install_fake_codex_text();

    let output = fixture
        .command()
        .env("PATH", fake_bin)
        .args(["codex", "fix", "--dry-run", "--doctor"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(3));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("not-json-doctor-output"));
    assert!(stdout.contains("未返回有效 JSON"));
    assert_eq!(fixture.doctor_spawn_count(), 1);
}

#[test]
fn codex_fix_doctor_persists_sanitized_report_when_not_dry_run() {
    let fixture = CodexFixFixture::new();
    let fake_bin = fixture.install_fake_codex_json_with_secret();

    let output = fixture
        .command()
        .env("PATH", fake_bin)
        .args(["codex", "fix", "--doctor"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(3));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains(DOCTOR_SECRET));
    assert!(!stdout.contains(PROFILE_SECRET));
    assert!(!stdout.contains(RUNTIME_SECRET));
    let report_path = persist_path_from_stdout(&stdout).expect("report path");
    let saved = fs::read_to_string(&report_path).unwrap();
    assert!(!saved.contains(DOCTOR_SECRET));
    assert!(saved.contains("<redacted>"));
    let _ = fs::remove_file(report_path);
}

#[test]
fn codex_fix_doctor_warning_prints_non_ok_check_id() {
    let fixture = CodexFixFixture::new();
    let fake_bin = fixture.install_fake_codex_warning();

    let output = fixture
        .command()
        .env("PATH", fake_bin)
        .args(["codex", "fix", "--dry-run", "--doctor"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("overallStatus = warning"));
    assert!(stdout.contains("state.rollout_db_parity = warning"));
    assert!(!stdout.contains("search provider"));
    assert!(!stdout.contains("configured servers"));
    assert!(!stdout.contains("model provider = custom"));
}

#[test]
fn codex_fix_doctor_snapshot_change_exits_3_and_does_not_keep_old_profile() {
    let fixture = CodexFixFixture::new();
    fixture.make_runtime_match();
    let fake_bin = fixture.install_fake_codex_mutating_profiles();

    let output = fixture
        .command()
        .env("PATH", fake_bin)
        .args(["codex", "fix", "--dry-run", "--doctor"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(3));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("<changed_during_doctor>"));
    assert!(stdout.contains("profile/runtime 状态发生变化"));
}

fn persist_path_from_stdout(stdout: &str) -> Option<PathBuf> {
    stdout.lines().find_map(|line| {
        line.split_once("完整报告已保存到：")
            .map(|(_, path)| PathBuf::from(path.trim()))
    })
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
