#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

const SECRET: &str = "DIAGNOSTIC_SECRET_SENTINEL_8371";

struct Fixture {
    temp: TempDir,
}

impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir_all(temp.path().join(".ccr")).unwrap();
        fs::write(temp.path().join(".ccr/config.toml"), "").unwrap();
        Self { temp }
    }

    fn path(&self, platform: &str) -> PathBuf {
        self.temp
            .path()
            .join(format!(".ccr/platforms/{platform}/profiles.toml"))
    }

    fn write(&self, platform: &str, content: &str) {
        let path = self.path(platform);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    fn command(&self) -> Command {
        let home = self.temp.path();
        let mut command = Command::new(env!("CARGO_BIN_EXE_ccr"));
        for (key, value) in [
            ("HOME", home.to_path_buf()),
            ("USERPROFILE", home.to_path_buf()),
            ("CCR_ROOT", home.join(".ccr")),
            ("CCR_DATA_DIR", home.join(".ccr")),
            ("CCR_LOCK_DIR", home.join(".locks")),
            ("CLAUDE_CONFIG_DIR", home.join(".claude")),
            ("CLAUDE_JSON_PATH", home.join(".claude/state.json")),
            ("CCR_SETTINGS_PATH", home.join(".claude/settings.json")),
            ("CCR_BACKUP_DIR", home.join(".claude/backups")),
            ("CCR_CODEX_DIR", home.join(".codex")),
            ("CODEX_HOME", home.join(".codex")),
            ("GROK_HOME", home.join(".grok")),
        ] {
            command.env(key, value);
        }
        for key in [
            "CCR_CONFIG_PATH",
            "ANTHROPIC_AUTH_TOKEN",
            "ANTHROPIC_API_KEY",
            "CLAUDE_CODE_USE_BEDROCK",
            "CLAUDE_CODE_USE_VERTEX",
            "CLAUDE_CODE_USE_FOUNDRY",
            "CLAUDE_CODE_OAUTH_TOKEN",
            "OPENAI_API_KEY",
            "XAI_API_KEY",
        ] {
            command.env_remove(key);
        }
        command.env("CCR_LOG_LEVEL", "off").env("NO_COLOR", "1");
        command
    }

    fn validate(&self, exit: i32, category: &str) {
        let before = inventory(self.temp.path());
        let output = self.command().arg("validate").output().unwrap();
        assert_eq!(
            output.status.code(),
            Some(exit),
            "stdout={} stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let text = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(text.contains(category), "missing {category}: {text}");
        assert!(!text.contains(SECRET));
        assert_eq!(
            before,
            inventory(self.temp.path()),
            "diagnostics mutated the fixture"
        );
    }
}

// Include paths, bytes and mtimes, including lock/backup directories.
fn inventory(root: &Path) -> BTreeMap<PathBuf, (Vec<u8>, std::time::SystemTime)> {
    fn walk(
        root: &Path,
        path: &Path,
        entries: &mut BTreeMap<PathBuf, (Vec<u8>, std::time::SystemTime)>,
    ) {
        for entry in fs::read_dir(path).unwrap() {
            let path = entry.unwrap().path();
            let metadata = fs::metadata(&path).unwrap();
            entries.insert(
                path.strip_prefix(root).unwrap().to_path_buf(),
                (
                    if metadata.is_file() {
                        fs::read(&path).unwrap()
                    } else {
                        Vec::new()
                    },
                    metadata.modified().unwrap(),
                ),
            );
            if metadata.is_dir() {
                walk(root, &path, entries);
            }
        }
    }
    let mut entries = BTreeMap::new();
    walk(root, root, &mut entries);
    entries
}

fn api_profile() -> String {
    format!(
        "[main]\nbase_url = \"https://api.example.com\"\nauth_token = \"{SECRET}\"\nmodel = \"synthetic-model\"\n"
    )
}

fn subscription_profile(platform: &str) -> &'static str {
    match platform {
        "claude" => "[main]\nauth_mode = \"subscription\"\n",
        "codex" => "[main]\nauth_mode = \"openai_chatgpt\"\n",
        "grok" => "[main]\nprovider_type = \"official_relay\"\n",
        _ => unreachable!(),
    }
}

#[test]
fn validate_binary_six_state_matrix_is_read_only() {
    for platform in ["claude", "codex", "grok"] {
        for profile in [api_profile(), subscription_profile(platform).to_string()] {
            for state in [
                "valid",
                "warning",
                "invalid",
                "corrupt",
                "unreadable",
                "missing",
            ] {
                let fixture = Fixture::new();
                let (code, category) = match state {
                    "valid" => {
                        fixture.write(platform, &profile);
                        (0, "valid")
                    }
                    "warning" => {
                        fixture.write(platform, &(profile.clone() + "enabled = false\n"));
                        (0, "disabled")
                    }
                    "invalid" => {
                        fixture.write(
                            platform,
                            &if profile.contains("base_url") {
                                profile.replace("https://api.example.com", "invalid-url")
                            } else {
                                profile.clone() + "base_url = \"invalid-url\"\n"
                            },
                        );
                        (90, "invalid")
                    }
                    "corrupt" => {
                        fixture.write(
                            platform,
                            &format!("[main]\nauth_token = \"{SECRET}\"\n[broken"),
                        );
                        (14, "corrupt")
                    }
                    "unreadable" => {
                        fs::create_dir_all(fixture.path(platform)).unwrap();
                        (51, "unreadable")
                    }
                    "missing" => (0, "missing"),
                    _ => unreachable!(),
                };
                fixture.validate(code, category);
            }
        }
    }
}

#[test]
fn validate_binary_applied_api_and_subscription_profiles_follow_domain_rules() {
    for platform in ["claude", "codex", "grok"] {
        for profile in [api_profile(), subscription_profile(platform).to_string()] {
            let fixture = Fixture::new();
            fixture.write(platform, &profile);
            let home = fixture.temp.path();
            fs::create_dir_all(home.join(".claude")).unwrap();
            fs::write(home.join(".claude/.credentials.json"), serde_json::to_vec(&serde_json::json!({
                "claudeAiOauth": {"accessToken": SECRET, "refreshToken": SECRET, "expiresAt": "2099-01-01T00:00:00Z", "subscriptionType": "pro"}
            })).unwrap()).unwrap();
            fs::create_dir_all(home.join(".codex")).unwrap();
            fs::write(
                home.join(".codex/config.toml"),
                "cli_auth_credentials_store = \"file\"\n",
            )
            .unwrap();
            fs::write(home.join(".codex/auth.json"), serde_json::to_vec(&serde_json::json!({
                "tokens": {"access_token": SECRET, "refresh_token": SECRET, "id_token": "e30.eyJleHAiOjQxMDI0NDQ4MDB9.synthetic"}
            })).unwrap()).unwrap();
            let output = fixture
                .command()
                .args([platform, "profile", "switch", "main"])
                .output()
                .unwrap();
            assert_eq!(
                output.status.code(),
                Some(0),
                "apply {platform}: {}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            fixture.validate(0, "valid");
        }
    }
}

#[test]
fn validate_binary_existing_runtime_errors_are_not_unconfigured() {
    for (platform, relative, malformed, code) in [
        ("claude", ".claude/settings.json", "{broken", 40),
        ("codex", ".codex/config.toml", "[broken", 41),
        ("grok", ".grok/config.toml", "[broken", 41),
    ] {
        let fixture = Fixture::new();
        fixture.write(platform, &api_profile());
        let path = fixture.temp.path().join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, malformed).unwrap();
        fixture.validate(code, "corrupt");
        fs::remove_file(&path).unwrap();
        fs::create_dir(&path).unwrap();
        fixture.validate(51, "unreadable");
    }
}

#[test]
fn validate_binary_runtime_errors_are_independent_of_profile_presence() {
    for (platform, relative, malformed, code) in [
        ("claude", ".claude/settings.json", "{broken", 40),
        ("codex", ".codex/config.toml", "[broken", 41),
        ("grok", ".grok/config.toml", "[broken", 41),
    ] {
        for current in [false, true] {
            let fixture = Fixture::new();
            if current {
                fixture.write(
                    platform,
                    &format!(
                        "current_config = \"main\"\ndefault_config = \"main\"\n{}",
                        api_profile()
                    ),
                );
            }
            let path = fixture.temp.path().join(relative);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, malformed).unwrap();
            fixture.validate(code, "corrupt");
            fs::remove_file(&path).unwrap();
            fs::create_dir(&path).unwrap();
            fixture.validate(51, "unreadable");
        }
    }
}

#[test]
fn doctor_binary_current_conflict_names_the_authoritative_source() {
    for (platform, owner) in [
        ("claude", "profiles.toml"),
        ("codex", "registry"),
        ("grok", "registry"),
    ] {
        let fixture = Fixture::new();
        fixture.write(
            platform,
            &format!(
                "current_config = \"main\"\ndefault_config = \"main\"\n{}{}",
                api_profile(),
                api_profile().replace("[main]", "[alternate]")
            ),
        );
        fs::write(
            fixture.temp.path().join(".ccr/config.toml"),
            format!("[{platform}]\nenabled = true\ncurrent_profile = \"alternate\"\n"),
        )
        .unwrap();
        let before = inventory(fixture.temp.path());
        let output = fixture
            .command()
            .args(["doctor", "--platform", platform, "--json"])
            .output()
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        let id = format!("platform.{platform}.current_profile");
        let check = json["checks"]
            .as_array()
            .unwrap()
            .iter()
            .find(|check| check["id"] == id)
            .unwrap();
        assert!(
            check["summary"]
                .as_str()
                .unwrap()
                .contains(&format!("from {owner}")),
            "{check}"
        );
        assert_eq!(before, inventory(fixture.temp.path()));
    }
}

#[cfg(windows)]
#[test]
fn validate_binary_native_denied_read_is_distinct_from_missing() {
    use std::os::windows::fs::OpenOptionsExt;
    for platform in ["claude", "codex", "grok"] {
        let fixture = Fixture::new();
        fixture.write(platform, &api_profile());
        let before = inventory(fixture.temp.path());
        let handle = fs::OpenOptions::new()
            .read(true)
            .share_mode(0)
            .open(fixture.path(platform))
            .unwrap();
        let output = fixture.command().arg("validate").output().unwrap();
        assert_eq!(output.status.code(), Some(51));
        assert!(String::from_utf8_lossy(&output.stdout).contains("unreadable"));
        drop(handle);
        assert_eq!(before, inventory(fixture.temp.path()));
    }
}

#[cfg(unix)]
#[test]
fn validate_binary_permission_denied_is_distinct_from_missing() {
    use std::os::unix::fs::PermissionsExt;
    for platform in ["claude", "codex", "grok"] {
        let fixture = Fixture::new();
        fixture.write(platform, &api_profile());
        let path = fixture.path(platform);
        let before = inventory(fixture.temp.path());
        let permissions = fs::metadata(&path).unwrap().permissions();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o0)).unwrap();
        let output = fixture.command().arg("validate").output().unwrap();
        fs::set_permissions(&path, permissions).unwrap();
        assert_eq!(output.status.code(), Some(51));
        assert!(String::from_utf8_lossy(&output.stdout).contains("unreadable"));
        assert_eq!(before, inventory(fixture.temp.path()));
    }
}

#[test]
fn doctor_binary_default_scope_includes_grok_and_preserves_simplified_profiles() {
    let fixture = Fixture::new();
    fixture.write("grok", &api_profile());
    fs::create_dir_all(fixture.temp.path().join(".grok")).unwrap();
    fs::write(
        fixture.temp.path().join(".grok/config.toml"),
        "[ui]\ntheme = \"dark\"\n",
    )
    .unwrap();
    let before = inventory(fixture.temp.path());
    let output = fixture
        .command()
        .args(["doctor", "--json"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let checks = json["checks"].as_array().unwrap();
    assert!(
        checks
            .iter()
            .any(|c| c["id"] == "platform.grok.settings_file" && c["status"] == "ok")
    );
    assert!(
        checks
            .iter()
            .any(|c| c["id"] == "platform.grok.current_profile" && c["status"] == "skip")
    );
    assert_eq!(before, inventory(fixture.temp.path()));
}

#[test]
fn doctor_binary_corrupt_registry_and_settings_do_not_echo_source_secrets() {
    let fixture = Fixture::new();
    fs::write(
        fixture.temp.path().join(".ccr/config.toml"),
        format!("token = \"{SECRET}"),
    )
    .unwrap();
    let output = fixture
        .command()
        .args(["doctor", "--json"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(!String::from_utf8_lossy(&output.stdout).contains(SECRET));
    assert!(!String::from_utf8_lossy(&output.stderr).contains(SECRET));
    for (platform, path, body) in [
        (
            "claude",
            ".claude/settings.json",
            format!("{{\"env\": \"{SECRET}\"}}"),
        ),
        ("codex", ".codex/config.toml", format!("token = \"{SECRET}")),
        ("grok", ".grok/config.toml", format!("token = \"{SECRET}")),
    ] {
        let fixture = Fixture::new();
        fixture.write(platform, &api_profile());
        let path = fixture.temp.path().join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, body).unwrap();
        let before = inventory(fixture.temp.path());
        let output = fixture
            .command()
            .args(["doctor", "--platform", platform, "--json"])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1));
        assert!(
            !String::from_utf8_lossy(&output.stdout).contains(SECRET),
            "{platform}: {}",
            String::from_utf8_lossy(&output.stdout)
        );
        assert!(!String::from_utf8_lossy(&output.stderr).contains(SECRET));
        assert_eq!(before, inventory(fixture.temp.path()));
    }
}

#[test]
fn validate_binary_subscription_profiles_and_empty_documents_are_valid() {
    for (platform, profile) in [
        ("claude", "[main]\nauth_mode = \"subscription\"\n"),
        ("codex", "[main]\nauth_mode = \"openai_chatgpt\"\n"),
        ("grok", "[main]\nprovider_type = \"official_relay\"\n"),
    ] {
        let fixture = Fixture::new();
        fixture.write(platform, profile);
        fixture.validate(0, "valid");
        fixture.write(platform, "");
        fixture.validate(0, "inactive");
    }
}

#[test]
fn validate_binary_disabled_current_is_not_activatable() {
    for platform in ["claude", "codex", "grok"] {
        let fixture = Fixture::new();
        fixture.write(
            platform,
            &format!(
                "current_config = \"main\"\ndefault_config = \"main\"\n{}enabled = false\n",
                api_profile()
            ),
        );
        fixture.validate(90, "disabled");
    }
}

#[test]
fn validate_binary_claude_current_uses_valid_marker_precedence_without_repair() {
    for (file_current, registry_current, disabled, exit, category) in [
        ("deleted", "main", false, 0, "[WARN/invalid]"),
        ("main", "deleted", false, 0, "[WARN/invalid]"),
        ("main", "alternate", false, 0, "[WARN/invalid]"),
        ("deleted", "also-deleted", false, 62, "[ERROR/invalid]"),
        ("main", "alternate", true, 90, "[ERROR/disabled]"),
    ] {
        let fixture = Fixture::new();
        fixture.write(
            "claude",
            &format!(
                "current_config = {file_current:?}\ndefault_config = \"main\"\n{}enabled = {}\n{}",
                subscription_profile("claude"),
                !disabled,
                subscription_profile("claude").replace("[main]", "[alternate]")
            ),
        );
        fs::write(
            fixture.temp.path().join(".ccr/config.toml"),
            format!("[claude]\nenabled = true\ncurrent_profile = {registry_current:?}\n"),
        )
        .unwrap();
        fixture.validate(exit, category);
    }
}

#[test]
fn doctor_binary_grok_is_supported_and_legacy_adapters_are_explicit() {
    let fixture = Fixture::new();
    let before = inventory(fixture.temp.path());
    let output = fixture
        .command()
        .args(["doctor", "--platform", "grok", "--json"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(
        json["checks"]
            .as_array()
            .unwrap()
            .iter()
            .any(|check| check["id"] == "platform.grok.profiles_file")
    );
    assert_eq!(before, inventory(fixture.temp.path()));
    for platform in ["gemini", "droid"] {
        let result = fixture
            .command()
            .args(["doctor", "--platform", platform, "--json"])
            .output()
            .unwrap();
        assert_eq!(result.status.code(), Some(1));
        let text = String::from_utf8_lossy(&result.stdout);
        assert!(text.contains("legacy_adapter"), "{text}");
    }
}
