//! Shared behavioral fixtures used by the real CLI, TUI and desktop adapters.
#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::profile_lifecycle::*;
use crate::{
    models::{Platform, PlatformConfig, ProfileConfig},
    platforms::create_platform,
    test_support::TestHome,
};
use ccr_core::core::write_journal::{self, fault};
use ccr_core::{CcrError, Result};
use std::{cell::RefCell, collections::BTreeMap, path::PathBuf, rc::Rc};

#[derive(Clone, Default)]
struct LogCapture(std::sync::Arc<std::sync::Mutex<Vec<u8>>>);
impl std::io::Write for LogCapture {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for LogCapture {
    type Writer = Self;
    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}

pub const SENTINEL: &str = "synthetic-T02-secret-sentinel";

fn fixture(platform: Platform) -> TestHome {
    let mut home = TestHome::new_with_home_env();
    let root = home.root().as_os_str().to_owned();
    home.set_env("CCR_DATA_DIR", &root);
    let grok = home.home().join(".grok");
    home.set_env("GROK_HOME", grok.as_os_str());
    let instance = create_platform(platform).unwrap();
    for name in ["old", "new"] {
        let mut profile = ProfileConfig::new()
            .with_base_url("https://fixture.invalid".into())
            .with_auth_token(SENTINEL.into())
            .with_model(format!("model-{name}"));
        profile.provider_type = Some("third_party_model".into());
        profile.platform_data.insert(
            "fixture_unknown".into(),
            serde_json::json!({"keep": [1, 2]}),
        );
        instance.save_profile(name, &profile).unwrap();
    }
    ccr_config::ConfigManager::for_platform(&platform.to_string())
        .unwrap()
        .mutate(|config| {
            config.sections.get_mut("old").unwrap().other.insert(
                "fixture_datetime".into(),
                toml::Value::Datetime("2026-09-28T12:00:00Z".parse().unwrap()),
            );
            Ok(())
        })
        .unwrap();
    instance.apply_profile("old").unwrap();
    home
}

/// Holds the existing environment fixture lock for downstream adapter tests.
pub fn isolated_fixture(platform: Platform) -> impl Drop {
    fixture(platform)
}

fn snapshot(paths: &[PathBuf]) -> BTreeMap<PathBuf, Option<Vec<u8>>> {
    paths
        .iter()
        .map(|p| (p.clone(), std::fs::read(p).ok()))
        .collect()
}

fn paths(platform: Platform, instance: &dyn PlatformConfig) -> Vec<PathBuf> {
    super::profile_lifecycle::operation_paths(platform, instance).unwrap()
}

pub fn preflight(adapter: impl Fn(ApplyProfileRequest) -> Result<ProfileOutcome>) {
    for platform in [Platform::Claude, Platform::Codex, Platform::Grok] {
        let _home = fixture(platform);
        let instance = create_platform(platform).unwrap();
        let mut profile = instance.load_profiles().unwrap()["new"].clone();
        profile.enabled = Some(false);
        instance.save_profile("new", &profile).unwrap();
        let before = snapshot(&paths(platform, instance.as_ref()));
        assert!(adapter(ApplyProfileRequest::new(platform, "new")).is_err());
        assert!(adapter(ApplyProfileRequest::new(platform, "deleted")).is_err());
        assert_eq!(before, snapshot(&paths(platform, instance.as_ref())));
        ccr_config::ConfigManager::for_platform(&platform.to_string())
            .unwrap()
            .mutate(|config| {
                let target = config.sections.get_mut("new").unwrap();
                target.enabled = Some(true);
                target.base_url = Some("invalid://endpoint".into());
                Ok(())
            })
            .unwrap();
        let invalid = snapshot(&paths(platform, instance.as_ref()));
        assert!(adapter(ApplyProfileRequest::new(platform, "new")).is_err());
        assert_eq!(invalid, snapshot(&paths(platform, instance.as_ref())));
    }
}

pub fn success_and_replay(adapter: impl Fn(ApplyProfileRequest) -> Result<ProfileOutcome>) {
    for platform in [Platform::Claude, Platform::Codex, Platform::Grok] {
        let _home = fixture(platform);
        let request = ApplyProfileRequest::new(platform, "new");
        let capture = LogCapture::default();
        let subscriber = tracing_subscriber::fmt()
            .with_ansi(false)
            .without_time()
            .with_max_level(tracing::Level::TRACE)
            .with_writer(capture.clone())
            .finish();
        let first =
            tracing::subscriber::with_default(subscriber, || adapter(request.clone())).unwrap();
        assert!(!String::from_utf8_lossy(&capture.0.lock().unwrap()).contains(SENTINEL));
        assert_eq!(first.status, ProfileStatus::Applied, "{first:?}");
        assert!(first.activation_committed);
        let instance = create_platform(platform).unwrap();
        let before = snapshot(&paths(platform, instance.as_ref()));
        assert_eq!(adapter(request).unwrap(), first);
        assert_eq!(before, snapshot(&paths(platform, instance.as_ref())));
        assert_eq!(
            instance.load_profiles().unwrap()["new"].usage_count,
            Some(1)
        );
        assert_eq!(
            instance.get_current_profile().unwrap().as_deref(),
            Some("new")
        );
        let history = crate::managers::HistoryManager::with_default()
            .unwrap()
            .load()
            .unwrap();
        assert_eq!(history.len(), 1);
        assert!(!serde_json::to_string(&first).unwrap().contains(SENTINEL));
        assert!(!format!("{first:?}").contains(SENTINEL));
        // Replaying a committed operation needs no surviving source profile.
        ccr_config::ConfigManager::for_platform(&platform.to_string())
            .unwrap()
            .mutate(|config| {
                config.sections.shift_remove("new");
                config.current_config = "old".into();
                config.default_config = "old".into();
                Ok(())
            })
            .unwrap();
        let removed = snapshot(&paths(platform, instance.as_ref()));
        let replay = ApplyProfileRequest {
            platform,
            name: "new".into(),
            operation_id: first.operation_id.clone(),
        };
        assert_eq!(adapter(replay).unwrap(), first);
        assert_eq!(removed, snapshot(&paths(platform, instance.as_ref())));
    }
}

pub fn ancillary_failures(adapter: impl Fn(ApplyProfileRequest) -> Result<ProfileOutcome>) {
    for history_failure in [false, true] {
        let home = fixture(Platform::Claude);
        if history_failure {
            std::fs::create_dir(home.root().join("data.db")).unwrap();
        }
        let _fault = fault::install(move |path| {
            if !history_failure
                && !write_journal::is_active()
                && path.file_name().is_some_and(|n| n == "profiles.toml")
            {
                return Err(CcrError::FileIoError("synthetic usage failure".into()));
            }
            Ok(())
        });
        let request = ApplyProfileRequest::new(Platform::Claude, "new");
        let outcome = adapter(request.clone()).unwrap();
        assert_eq!(outcome.status, ProfileStatus::AppliedWithWarning);
        assert!(outcome.activation_committed);
        assert!(outcome.warnings.contains(&if history_failure {
            ProfileWarning::HistoryFailed
        } else {
            ProfileWarning::UsageCountFailed
        }));
        assert_eq!(adapter(request).unwrap(), outcome);
        assert_eq!(
            create_platform(Platform::Claude)
                .unwrap()
                .get_current_profile()
                .unwrap()
                .as_deref(),
            Some("new")
        );
    }
}

pub fn failure_matrix(adapter: impl Fn(ApplyProfileRequest) -> Result<ProfileOutcome>) {
    for platform in [Platform::Claude, Platform::Codex, Platform::Grok] {
        let count = {
            let _home = fixture(platform);
            let writes = Rc::new(RefCell::new(0));
            let seen = writes.clone();
            let _fault = fault::install(move |p| {
                if write_journal::contains(p) {
                    *seen.borrow_mut() += 1;
                }
                Ok(())
            });
            assert!(
                adapter(ApplyProfileRequest::new(platform, "new"))
                    .unwrap()
                    .activation_committed
            );
            *writes.borrow()
        };
        assert!(count >= 3);
        for fail_at in 1..=count {
            let _home = fixture(platform);
            let instance = create_platform(platform).unwrap();
            let before = snapshot(&paths(platform, instance.as_ref()));
            let mut index = 0;
            let _fault = fault::install(move |p| {
                if write_journal::contains(p) {
                    index += 1;
                    if index == fail_at {
                        return Err(CcrError::FileIoError("synthetic stage failure".into()));
                    }
                }
                Ok(())
            });
            let result = adapter(ApplyProfileRequest::new(platform, "new")).unwrap();
            assert_eq!(
                result.status,
                ProfileStatus::Unchanged,
                "{platform} write {fail_at}: {result:?}"
            );
            assert!(!result.activation_committed);
            assert_eq!(
                before,
                snapshot(&paths(platform, instance.as_ref())),
                "{platform} write {fail_at}"
            );
            assert!(!std::path::Path::new(&_home.root().join("data.db")).exists());
        }
    }
}

pub fn external_change(adapter: impl Fn(ApplyProfileRequest) -> Result<ProfileOutcome>) {
    let home = fixture(Platform::Claude);
    let runtime = home.settings_path().to_path_buf();
    let replacement = runtime.clone();
    let _fault = fault::install(move |p| {
        if write_journal::is_active() && p.file_name().is_some_and(|n| n == "profiles.toml") {
            std::fs::write(&replacement, br#"{"env":{"USER_EXTERNAL":"keep"}}"#).unwrap();
            return Err(CcrError::FileIoError("synthetic concurrent failure".into()));
        }
        Ok(())
    });
    let outcome = adapter(ApplyProfileRequest::new(Platform::Claude, "new")).unwrap();
    assert_eq!(outcome.status, ProfileStatus::RecoveryRequired);
    assert!(!outcome.activation_committed);
    assert_eq!(
        std::fs::read(runtime).unwrap(),
        br#"{"env":{"USER_EXTERNAL":"keep"}}"#
    );
    assert!(!outcome.recovery_paths.is_empty());
}

pub fn deleted_after_preparation(adapter: impl Fn(ApplyProfileRequest) -> Result<ProfileOutcome>) {
    let home = fixture(Platform::Claude);
    let manager = ccr_config::ConfigManager::for_platform("claude").unwrap();
    let profiles = manager.config_path().to_owned();
    let mut document: toml::Value =
        toml::from_str(&std::fs::read_to_string(&profiles).unwrap()).unwrap();
    document.as_table_mut().unwrap().remove("new");
    let replacement = toml::to_string(&document).unwrap();
    let runtime = std::fs::read(home.settings_path()).unwrap();
    let _fault = fault::install(move |path| {
        if path
            .parent()
            .is_some_and(|parent| parent.ends_with("profile-operations"))
            && !write_journal::is_active()
        {
            std::fs::write(&profiles, &replacement).unwrap();
        }
        Ok(())
    });
    let outcome = adapter(ApplyProfileRequest::new(Platform::Claude, "new")).unwrap();
    assert_eq!(outcome.status, ProfileStatus::Unchanged);
    assert!(!outcome.activation_committed);
    assert_eq!(std::fs::read(home.settings_path()).unwrap(), runtime);
    assert!(!manager.load().unwrap().sections.contains_key("new"));
    assert!(!home.root().join("data.db").exists());
}

pub fn post_publish_failure(adapter: impl Fn(ApplyProfileRequest) -> Result<ProfileOutcome>) {
    for platform in [Platform::Claude, Platform::Codex, Platform::Grok] {
        let _home = fixture(platform);
        let instance = create_platform(platform).unwrap();
        let before = snapshot(&paths(platform, instance.as_ref()));
        let mut once = true;
        let _fault = fault::install_after_publish(move |path| {
            if once && write_journal::contains(path) {
                once = false;
                return Err(CcrError::FileIoError(
                    "synthetic post-publication failure".into(),
                ));
            }
            Ok(())
        });
        let outcome = adapter(ApplyProfileRequest::new(platform, "new")).unwrap();
        assert_eq!(outcome.status, ProfileStatus::Unchanged);
        assert!(!outcome.activation_committed);
        assert_eq!(before, snapshot(&paths(platform, instance.as_ref())));
    }
}

pub fn rename_failures(
    platforms: &[Platform],
    adapter: impl Fn(Platform, &str, &str) -> Result<ProfileOutcome>,
) {
    for &platform in platforms {
        {
            let home = fixture(platform);
            let instance = create_platform(platform).unwrap();
            let before = snapshot(&paths(platform, instance.as_ref()));
            for target in ["", "settings", "default_config", "current_config", "new"] {
                assert!(adapter(platform, "old", target).is_err());
                assert_eq!(before, snapshot(&paths(platform, instance.as_ref())));
            }
            let before_count = instance.load_profiles().unwrap()["new"].usage_count;
            std::fs::create_dir(home.root().join("data.db")).unwrap();
            let updated = adapter(platform, "new", "new").unwrap();
            assert_eq!(updated.status, ProfileStatus::AppliedWithWarning);
            assert!(!updated.activation_committed);
            assert!(updated.warnings.contains(&ProfileWarning::HistoryFailed));
            assert!(!updated.message().contains("Profile applied"));
            assert_eq!(
                instance.get_current_profile().unwrap().as_deref(),
                Some("old")
            );
            assert_eq!(
                instance.load_profiles().unwrap()["new"].usage_count,
                before_count
            );
        }
        // Save-new, pointer update, delete-old, and runtime apply all cross the
        // same journal; fail each successful write in a fresh fixture.
        let count = {
            let _home = fixture(platform);
            let count = Rc::new(RefCell::new(0));
            let seen = count.clone();
            let _fault = fault::install(move |p| {
                if write_journal::contains(p) {
                    *seen.borrow_mut() += 1;
                }
                Ok(())
            });
            let result = adapter(platform, "old", "renamed").unwrap();
            assert!(result.activation_committed);
            let manager = ccr_config::ConfigManager::for_platform(&platform.to_string()).unwrap();
            let config = manager.load().unwrap();
            assert_eq!(config.current_config, "renamed");
            assert_eq!(config.default_config, "renamed");
            assert!(config.sections.contains_key("renamed"));
            assert!(!config.sections.contains_key("old"));
            assert!(
                config.sections["renamed"]
                    .other
                    .contains_key("fixture_unknown")
            );
            assert!(matches!(
                config.sections["renamed"].other.get("fixture_datetime"),
                Some(toml::Value::Datetime(_))
            ));
            *count.borrow()
        };
        for fail_at in 1..=count {
            let _home = fixture(platform);
            let instance = create_platform(platform).unwrap();
            let before = snapshot(&paths(platform, instance.as_ref()));
            let mut index = 0;
            let _fault = fault::install(move |p| {
                if write_journal::contains(p) {
                    index += 1;
                    if index == fail_at {
                        return Err(CcrError::FileIoError("synthetic rename failure".into()));
                    }
                }
                Ok(())
            });
            let outcome = adapter(platform, "old", "renamed").unwrap();
            assert_eq!(
                outcome.status,
                ProfileStatus::Unchanged,
                "{platform}: {fail_at}"
            );
            assert_eq!(before, snapshot(&paths(platform, instance.as_ref())));
        }
    }
}

/// Enabling a disabled profile persists policy and activation as one operation.
pub fn enable_lifecycle() {
    for platform in [Platform::Claude, Platform::Codex, Platform::Grok] {
        let prepare = || {
            let home = fixture(platform);
            ccr_config::ConfigManager::for_platform(&platform.to_string())
                .unwrap()
                .mutate(|config| {
                    config.get_section_mut("new")?.enabled = Some(false);
                    Ok(())
                })
                .unwrap();
            home
        };
        let count = {
            let _home = prepare();
            let writes = Rc::new(RefCell::new(0));
            let seen = writes.clone();
            let fault = fault::install(move |path| {
                if write_journal::contains(path) {
                    *seen.borrow_mut() += 1;
                }
                Ok(())
            });
            let request = ApplyProfileRequest::new(platform, "new");
            let result = enable_profile(request.clone()).unwrap();
            drop(fault);
            assert!(result.activation_committed);
            assert_eq!(result.status, ProfileStatus::Applied);
            let instance = create_platform(platform).unwrap();
            assert!(instance.load_profiles().unwrap()["new"].is_enabled());
            assert_eq!(
                instance.get_current_profile().unwrap().as_deref(),
                Some("new")
            );
            let committed = snapshot(&paths(platform, instance.as_ref()));
            assert_eq!(enable_profile(request).unwrap(), result);
            assert_eq!(committed, snapshot(&paths(platform, instance.as_ref())));
            *writes.borrow()
        };
        assert!(count >= 4);
        for fail_at in 1..=count {
            let _home = prepare();
            let instance = create_platform(platform).unwrap();
            let before = snapshot(&paths(platform, instance.as_ref()));
            let mut index = 0;
            let _fault = fault::install(move |path| {
                if write_journal::contains(path) {
                    index += 1;
                    if index == fail_at {
                        return Err(CcrError::FileIoError("synthetic enable failure".into()));
                    }
                }
                Ok(())
            });
            let result = enable_profile(ApplyProfileRequest::new(platform, "new")).unwrap();
            assert_eq!(
                result.status,
                ProfileStatus::Unchanged,
                "{platform} stage {fail_at}"
            );
            assert!(!result.activation_committed);
            assert_eq!(before, snapshot(&paths(platform, instance.as_ref())));
            assert!(!instance.load_profiles().unwrap()["new"].is_enabled());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn enable_policy_activation_and_compensation() {
        enable_lifecycle();
    }
    fn cli(request: ApplyProfileRequest) -> Result<ProfileOutcome> {
        super::super::profile_switch::switch_profile_sync(request).map(|r| r.outcome)
    }
    #[test]
    fn profile_post_publish_failure_contract() {
        post_publish_failure(cli);
    }

    #[test]
    fn profile_adapters_keep_application_boundary() {
        let lifecycle = include_str!("profile_lifecycle.rs");
        for forbidden in [
            "println!",
            "print!",
            "stdin(",
            "process::exit",
            "commands::profile",
        ] {
            assert!(
                !lifecycle.contains(forbidden),
                "Shared lifecycle must not contain {forbidden}"
            );
        }
        let tui = include_str!("../../../ccr-tui/src/tui/app.rs");
        let apply = tui
            .split("fn apply_selected(")
            .nth(1)
            .unwrap()
            .split("fn off_selected(")
            .next()
            .unwrap();
        assert!(apply.contains("profile_backend::apply("));
        assert!(!apply.contains("profile_off"));
        assert!(!apply.contains(".apply_profile("));
        for adapter in [
            include_str!("../../../../ccr-ui/src-tauri/src/commands/claude_profiles.rs"),
            include_str!("../../../../ccr-ui/src-tauri/src/commands/codex_profiles.rs"),
        ] {
            assert!(adapter.contains("apply_profile_payload("));
            assert!(adapter.contains("profile_lifecycle::update_profile("));
            assert!(!adapter.contains("switch_command"));
            // Low-level calls in legacy test fixtures are not application callers.
            let production = adapter.split("#[cfg(test)]").next().unwrap();
            assert!(!production.contains(".apply_profile("));
        }
    }
    #[test]
    fn profile_deleted_after_prepare_contract() {
        deleted_after_preparation(cli);
    }
    #[test]
    fn profile_preflight_contract() {
        preflight(cli);
    }
    #[test]
    fn profile_success_replay_contract() {
        success_and_replay(cli);
    }
    #[test]
    fn profile_ancillary_contract() {
        ancillary_failures(cli);
    }
    #[test]
    fn profile_each_write_contract() {
        failure_matrix(cli);
    }
    #[test]
    fn profile_external_version_contract() {
        external_change(cli);
    }
    #[test]
    fn profile_rename_each_write_contract() {
        rename_failures(
            &[Platform::Claude, Platform::Codex],
            |platform, name, target| update_profile(platform, name, target, |_| Ok(())),
        );
    }
}
