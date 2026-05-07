#![allow(clippy::unwrap_used)]

use ccr::managers::{ConfigManager, PlatformConfigManager, UnifiedConfig};
use std::ffi::OsString;
use std::fs;

struct EnvGuard {
    key: &'static str,
    previous: Option<OsString>,
}

impl EnvGuard {
    fn set_path(key: &'static str, value: &std::path::Path) -> Self {
        let previous = std::env::var_os(key);
        unsafe {
            std::env::set_var(key, value);
        }
        Self { key, previous }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        unsafe {
            match self.previous.take() {
                Some(value) => std::env::set_var(self.key, value),
                None => std::env::remove_var(self.key),
            }
        }
    }
}

#[test]
fn legacy_registry_fields_load_without_becoming_clean_write_routing_truth() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");
    fs::write(
        &config_path,
        r#"
default_platform = "claude"
current_platform = "codex"

[claude]
enabled = true
current_profile = "main"
description = "Claude runtime"
last_used = "2026-05-06T00:00:00Z"

[codex]
enabled = true
current_profile = "official"
description = "Codex runtime"
"#,
    )
    .unwrap();

    let manager = PlatformConfigManager::new(&config_path);
    let config = manager.load().unwrap();
    assert_eq!(config.current_platform, "codex");
    assert_eq!(config.default_platform, "claude");
    assert_eq!(config.get_current_profile("claude").unwrap(), Some("main"));
    assert_eq!(
        config.get_current_profile("codex").unwrap(),
        Some("official")
    );
    assert_eq!(
        config
            .get_platform("claude")
            .unwrap()
            .description
            .as_deref(),
        Some("Claude runtime")
    );
    assert_eq!(
        config.get_platform("claude").unwrap().last_used.as_deref(),
        Some("2026-05-06T00:00:00Z")
    );

    manager.save(&config).unwrap();
    let saved = fs::read_to_string(&config_path).unwrap();
    assert!(!saved.contains("current_platform"));
    assert!(!saved.contains("default_platform"));
    assert!(saved.contains("[claude]"));
    assert!(saved.contains("current_profile = \"main\""));
    assert!(saved.contains("description = \"Claude runtime\""));
    assert!(saved.contains("last_used = \"2026-05-06T00:00:00Z\""));
}

#[test]
fn per_platform_current_profile_helpers_round_trip() {
    let mut config = UnifiedConfig::default();
    config
        .register_platform("codex".to_string(), Default::default())
        .unwrap();

    config
        .set_current_profile("claude", Some("subscription"))
        .unwrap();
    config.set_current_profile("codex", Some("team")).unwrap();

    assert_eq!(
        config.get_current_profile("claude").unwrap(),
        Some("subscription")
    );
    assert_eq!(config.get_current_profile("codex").unwrap(), Some("team"));

    config.set_current_profile("codex", None).unwrap();
    assert_eq!(config.get_current_profile("codex").unwrap(), None);
}

#[test]
fn config_manager_default_ignores_legacy_current_platform_routing() {
    let temp_dir = tempfile::tempdir().unwrap();
    fs::write(
        temp_dir.path().join("config.toml"),
        r#"
default_platform = "claude"
current_platform = "codex"

[claude]
enabled = true
current_profile = "main"

[codex]
enabled = true
current_profile = "official"
"#,
    )
    .unwrap();

    let _ccr_root = EnvGuard::set_path("CCR_ROOT", temp_dir.path());
    let manager = ConfigManager::with_default().unwrap();

    assert!(
        manager
            .config_path()
            .ends_with("platforms/claude/profiles.toml")
    );
}
