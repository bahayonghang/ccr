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

enum ContractFixture {
    Legacy(Box<TestHome>),
    Process(ProcessFixture),
}

impl Drop for ContractFixture {
    fn drop(&mut self) {
        // Variant owners restore legacy state or clean child fixture files.
    }
}

impl ContractFixture {
    fn root(&self) -> &std::path::Path {
        match self {
            Self::Legacy(home) => home.root(),
            Self::Process(home) => &home.root,
        }
    }

    fn settings_path(&self) -> &std::path::Path {
        match self {
            Self::Legacy(home) => home.settings_path(),
            Self::Process(home) => &home.settings,
        }
    }
}

/// Test-only process fixture. The caller supplies child environment overrides;
/// no process-global HOME, USERPROFILE, or CCR variables are changed here.
struct ProcessFixture {
    base: PathBuf,
    root: PathBuf,
    settings: PathBuf,
}

impl ProcessFixture {
    fn new(base: PathBuf) -> Self {
        assert!(base.is_absolute(), "contract fixture root must be absolute");
        let base = base.canonicalize().unwrap();
        assert!(base.join(".ccr-profile-contract-fixture").is_file());
        let root = base.join(".ccr");
        let settings = base.join(".claude/settings.json");
        for (key, expected) in [
            ("CCR_ROOT", root.clone()),
            ("CCR_DATA_DIR", root.clone()),
            ("CCR_LOCK_DIR", base.join(".locks")),
            ("CLAUDE_CONFIG_DIR", base.join(".claude")),
            ("CLAUDE_JSON_PATH", base.join(".claude.json")),
            ("CCR_SETTINGS_PATH", settings.clone()),
            ("CCR_BACKUP_DIR", base.join(".claude/backups")),
            ("CCR_CODEX_DIR", base.join(".codex")),
            ("GROK_HOME", base.join(".grok")),
        ] {
            assert_eq!(std::env::var_os(key).map(PathBuf::from), Some(expected));
        }
        for dir in [".ccr", ".locks", ".claude/backups", ".codex", ".grok"] {
            std::fs::create_dir_all(base.join(dir)).unwrap();
        }
        Self {
            base,
            root,
            settings,
        }
    }
}

impl Drop for ProcessFixture {
    fn drop(&mut self) {
        // Only these fixture-owned children are reset between lifecycle rounds.
        // The parent-owned marker survives until its TempDir is dropped.
        for name in [".ccr", ".locks", ".claude", ".codex", ".grok"] {
            let path = self.base.join(name);
            if path.exists() {
                std::fs::remove_dir_all(path).unwrap();
            }
        }
        let json = self.base.join(".claude.json");
        if json.exists() {
            std::fs::remove_file(json).unwrap();
        }
    }
}

fn fixture(platform: Platform) -> ContractFixture {
    let home = if let Some(root) = std::env::var_os("CCR_PROFILE_CONTRACT_ROOT") {
        ContractFixture::Process(ProcessFixture::new(root.into()))
    } else {
        let mut home = TestHome::new_with_home_env();
        let root = home.root().as_os_str().to_owned();
        home.set_env("CCR_DATA_DIR", &root);
        let grok = home.home().join(".grok");
        home.set_env("GROK_HOME", grok.as_os_str());
        ContractFixture::Legacy(Box::new(home))
    };
    seed_fixture(platform);
    home
}

fn seed_fixture(platform: Platform) {
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
        } else {
            ccr_config::fail_next_usage_increment();
        }
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
        assert!(count >= 2, "{platform} journal writes: {count}");
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

pub fn runtime_changed_before_catalog_commit(
    adapter: impl Fn(ApplyProfileRequest) -> Result<ProfileOutcome>,
) {
    let home = fixture(Platform::Claude);
    let runtime = home.settings_path().to_path_buf();
    let replacement = runtime.clone();
    let _fault = fault::install_after_publish(move |path| {
        if path.file_name().is_some_and(|name| name == "config.toml") {
            std::fs::write(&replacement, br#"{"env":{"USER_EXTERNAL":"keep"}}"#).unwrap();
        }
        Ok(())
    });
    let outcome = adapter(ApplyProfileRequest::new(Platform::Claude, "new")).unwrap();
    assert!(!outcome.activation_committed, "{outcome:?}");
    assert_eq!(outcome.status, ProfileStatus::RecoveryRequired);
    assert_eq!(
        std::fs::read(&runtime).unwrap(),
        br#"{"env":{"USER_EXTERNAL":"keep"}}"#
    );
    assert_eq!(
        create_platform(Platform::Claude)
            .unwrap()
            .get_current_profile()
            .unwrap()
            .as_deref(),
        Some("old")
    );
}

pub fn checkpoint_failure_keeps_activation(
    adapter: impl Fn(ApplyProfileRequest) -> Result<ProfileOutcome>,
) {
    let _home = fixture(Platform::Claude);
    ccr_config::fail_next_catalog_checkpoint();
    let outcome = adapter(ApplyProfileRequest::new(Platform::Claude, "new")).unwrap();
    assert!(outcome.activation_committed, "{outcome:?}");
    assert_ne!(outcome.status, ProfileStatus::Unchanged);
    assert_ne!(outcome.status, ProfileStatus::RecoveryRequired);
    assert_eq!(
        create_platform(Platform::Claude)
            .unwrap()
            .get_current_profile()
            .unwrap()
            .as_deref(),
        Some("new")
    );
}

pub fn commit_failure_replays(adapter: impl Fn(ApplyProfileRequest) -> Result<ProfileOutcome>) {
    let _home = fixture(Platform::Claude);
    let request = ApplyProfileRequest::new(Platform::Claude, "new");
    ccr_config::fail_next_catalog_commit();
    let failed = adapter(request.clone()).unwrap();
    assert!(!failed.activation_committed, "{failed:?}");
    assert_eq!(failed.status, ProfileStatus::Unchanged);
    assert_eq!(
        create_platform(Platform::Claude)
            .unwrap()
            .get_current_profile()
            .unwrap()
            .as_deref(),
        Some("old")
    );
    let replayed = adapter(request).unwrap();
    assert!(replayed.activation_committed, "{replayed:?}");
    assert_eq!(replayed.status, ProfileStatus::Applied);
    let instance = create_platform(Platform::Claude).unwrap();
    assert_eq!(
        instance.get_current_profile().unwrap().as_deref(),
        Some("new")
    );
    assert_eq!(
        instance.load_profiles().unwrap()["new"].usage_count,
        Some(1)
    );
}

pub fn external_change(adapter: impl Fn(ApplyProfileRequest) -> Result<ProfileOutcome>) {
    let home = fixture(Platform::Claude);
    let runtime = home.settings_path().to_path_buf();
    let replacement = runtime.clone();
    let _fault = fault::install(move |p| {
        if write_journal::is_active() && p.file_name().is_some_and(|n| n == "config.toml") {
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
    let database = manager.profiles_db().to_path_buf();
    let runtime = std::fs::read(home.settings_path()).unwrap();
    let _fault = fault::install(move |path| {
        if path
            .parent()
            .is_some_and(|parent| parent.ends_with("profile-operations"))
            && !write_journal::is_active()
        {
            ccr_config::ConfigManager::new(&database)
                .mutate(|config| {
                    config.sections.shift_remove("new");
                    Ok(())
                })
                .unwrap();
        }
        Ok(())
    });
    let outcome = adapter(ApplyProfileRequest::new(Platform::Claude, "new"));
    assert!(outcome.is_err(), "{outcome:?}");
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
        assert!(count >= 2, "{platform} enable journal writes: {count}");
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
        if crate::home_process::run_current_test() {
            return;
        }
        enable_lifecycle();
    }
    fn cli(request: ApplyProfileRequest) -> Result<ProfileOutcome> {
        super::super::profile_switch::switch_profile_sync(request).map(|r| r.outcome)
    }
    #[test]
    fn profile_post_publish_failure_contract() {
        if crate::home_process::run_current_test() {
            return;
        }
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
        assert!(apply.contains("self.start_profile_mutation(MutationRequest::Apply(request))"));
        assert!(!apply.contains("profile_off"));
        assert!(!apply.contains(".apply_profile("));
        assert!(!apply.contains("profile_backend::apply("));
        let handoff = tui
            .split_once("fn start_profile_mutation(")
            .expect("TUI must submit captured mutation requests")
            .1
            .split_once("fn reload_profiles(")
            .expect("TUI mutation handoff must precede reload")
            .0;
        assert!(handoff.contains("self.profile_tasks.start_mutation(request)"));

        // Scheduling may move, but the blocking worker must retain the shared owner.
        let tasks = include_str!("../../../ccr-tui/src/tui/profile_tasks.rs");
        assert!(tasks.contains("writer: Arc::new(mutate)"));
        let worker = tasks
            .split_once("fn start_mutation(")
            .expect("Profile tasks must own mutation scheduling")
            .1
            .split_once("fn drain(")
            .expect("Profile tasks must drain mutation outcomes")
            .0;
        assert!(worker.contains("let worker_request = request.clone()"));
        assert!(worker.contains("self.executor.spawn_blocking(move ||"));
        assert!(worker.contains("sender.send(writer(worker_request))"));
        let mutation = tasks
            .split_once("fn mutate(")
            .expect("Profile tasks must retain the shared mutation adapter")
            .1
            .split_once("struct ReadJob")
            .expect("Mutation adapter must precede read jobs")
            .0;
        let (apply_arm, off_arm) = mutation
            .split_once("MutationRequest::Off(platform)")
            .expect("Apply and off must have separate mutation arms");
        assert!(apply_arm.contains("super::profile_backend::apply(request)"));
        assert!(!apply_arm.contains("profile_off"));
        assert!(off_arm.contains("profile_off_for_platform(platform)"));
        assert!(!mutation.contains(".await"));

        let backend = include_str!("../../../ccr-tui/src/tui/profile_backend.rs");
        let backend_apply = backend
            .split_once("pub fn presentation(")
            .expect("TUI backend must separate mutation and presentation")
            .0;
        assert!(backend_apply.contains("ccr_cli::application::profile_lifecycle::"));
        assert!(backend_apply.contains("apply_profile(request)"));
        for adapter in [handoff, worker, mutation, backend_apply] {
            assert!(!adapter.contains(".apply_profile("));
            assert!(!adapter.contains("switch_command"));
        }
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
        if crate::home_process::run_current_test() {
            return;
        }
        deleted_after_preparation(cli);
    }
    #[test]
    fn profile_preflight_contract() {
        if crate::home_process::run_current_test() {
            return;
        }
        preflight(cli);
    }
    #[test]
    fn profile_success_replay_contract() {
        if crate::home_process::run_current_test() {
            return;
        }
        success_and_replay(cli);
    }
    #[test]
    fn profile_ancillary_contract() {
        if crate::home_process::run_current_test() {
            return;
        }
        ancillary_failures(cli);
    }
    #[test]
    fn profile_each_write_contract() {
        if crate::home_process::run_current_test() {
            return;
        }
        failure_matrix(cli);
    }
    #[test]
    fn profile_external_version_contract() {
        if crate::home_process::run_current_test() {
            return;
        }
        external_change(cli);
    }
    #[test]
    fn profile_commit_failure_replays_operation() {
        if crate::home_process::run_current_test() {
            return;
        }
        commit_failure_replays(cli);
    }
    #[test]
    fn profile_runtime_change_before_catalog_commit() {
        if crate::home_process::run_current_test() {
            return;
        }
        runtime_changed_before_catalog_commit(cli);
    }
    #[test]
    fn profile_checkpoint_failure_keeps_activation() {
        if crate::home_process::run_current_test() {
            return;
        }
        checkpoint_failure_keeps_activation(cli);
    }
    #[test]
    fn profile_rename_each_write_contract() {
        if crate::home_process::run_current_test() {
            return;
        }
        rename_failures(
            &[Platform::Claude, Platform::Codex],
            |platform, name, target| update_profile(platform, name, target, |_| Ok(())),
        );
    }
}
