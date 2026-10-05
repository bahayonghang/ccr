#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::{ConfigManager, ConfigPatch, ConfigSection, FieldPatch};
use crate::models::{Platform, PlatformPaths, ProfileConfig};
use crate::platforms::base;
use crate::services::config_service::ConfigService;
use crate::test_support::TestCcrEnv;
use ccr_core::{CcrError, Validatable};
use indexmap::IndexMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};

fn section() -> ConfigSection {
    ConfigSection {
        base_url: Some("https://api.example.test".into()),
        auth_token: Some("synthetic-repository-secret".into()),
        model: Some("test-model".into()),
        ..Default::default()
    }
}

fn service() -> ConfigService {
    ConfigService::for_platform("claude").unwrap()
}

fn inventory(path: &Path) -> Vec<(PathBuf, Vec<u8>, Option<SystemTime>)> {
    fn collect(path: &Path, found: &mut Vec<(PathBuf, Vec<u8>, Option<SystemTime>)>) {
        if !path.exists() {
            return;
        }
        if path.is_file() {
            found.push((
                path.into(),
                fs::read(path).unwrap(),
                fs::metadata(path).unwrap().modified().ok(),
            ));
        } else {
            found.push((path.into(), Vec::new(), None));
            for entry in fs::read_dir(path).unwrap() {
                collect(&entry.unwrap().path(), found);
            }
        }
    }
    let mut found = Vec::new();
    collect(path, &mut found);
    found.sort_by(|a, b| a.0.cmp(&b.0));
    found
}

#[test]
fn reads_do_not_initialize_autofix_or_lock_files() {
    let env = TestCcrEnv::new();
    let before = inventory(env.root());
    let service = service();
    assert!(matches!(
        service.list_configs(),
        Err(CcrError::ConfigMissing(_))
    ));
    assert!(service.get_current().is_err());
    assert!(service.validate_all().is_err());
    assert_eq!(before, inventory(env.root()));

    assert!(
        !env.lock_dir()
            .join(format!(
                "{}.lock",
                super::repository::config_resource_name(service.config_manager().config_path())
                    .unwrap()
            ))
            .exists()
    );
    let path = service.config_manager().config_path();
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    // Missing model used to trigger auto_complete and a write from list/current.
    fs::write(
        path,
        r#"default_config = "main"
current_config = "main"
[main]
base_url = "https://api.example.test"
auth_token = "synthetic-repository-secret"
"#,
    )
    .unwrap();
    let before = inventory(env.root());
    assert_eq!(service.list_configs().unwrap().configs.len(), 1);
    assert_eq!(service.get_current().unwrap().name, "main");
    assert_eq!(service.validate_all().unwrap().valid_count, 1);
    assert!(
        service.load_config().unwrap().sections["main"]
            .model
            .is_none()
    );
    assert_eq!(before, inventory(env.root()));
}

#[test]
fn corrupt_and_readonly_queries_leave_files_unchanged() {
    let env = TestCcrEnv::new();
    let service = service();
    service.add_config("main".into(), section()).unwrap();
    let path = service.config_manager().config_path();
    let valid = fs::read(path).unwrap();
    fs::write(path, "auth_token = [secret-sentinel").unwrap();
    let before = inventory(env.root());
    for result in [
        service.list_configs().map(|_| ()),
        service.get_current().map(|_| ()),
        service.validate_all().map(|_| ()),
    ] {
        assert!(!result.unwrap_err().to_string().contains("secret-sentinel"));
    }
    assert_eq!(before, inventory(env.root()));
    fs::write(path, valid).unwrap();
    let original = fs::metadata(path).unwrap().permissions();
    let mut readonly = original.clone();
    readonly.set_readonly(true);
    fs::set_permissions(path, readonly).unwrap();
    let before = inventory(env.root());
    service.list_configs().unwrap();
    service.get_current().unwrap();
    service.validate_all().unwrap();
    assert_eq!(before, inventory(env.root()));
    fs::set_permissions(path, original).unwrap();
}

#[test]
fn no_op_mutations_preserve_content_time_and_backups() {
    let env = TestCcrEnv::new();
    let service = service();
    service.add_config("main".into(), section()).unwrap();
    let manager = service.config_manager();
    let before = inventory(env.root());
    manager.mutate(|_| Ok(())).unwrap();
    assert_eq!(inventory(env.root()), before);
}

#[test]
fn no_op_mutations_reject_a_leaf_replacement() {
    let env = TestCcrEnv::new();
    let service = service();
    service.add_config("main".into(), section()).unwrap();
    let manager = service.config_manager();
    let path = manager.config_path();
    let backups = inventory(&env.root().join("backups"));
    let error = manager
        .mutate(|_| {
            ccr_core::core::guarded_write::write_guarded(
                path,
                b"external replacement",
                &ccr_core::core::WriteOptions::default(),
            )?;
            Ok(())
        })
        .unwrap_err();
    assert!(matches!(error, CcrError::ValidationError(message) if message.contains("版本冲突")));
    assert_eq!(fs::read(path).unwrap(), b"external replacement");
    assert_eq!(inventory(&env.root().join("backups")), backups);
}

#[cfg(unix)]
#[test]
fn no_op_mutations_enforce_secret_mode_without_replacing_the_file() {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};

    let env = TestCcrEnv::new();
    let service = service();
    service.add_config("main".into(), section()).unwrap();
    let manager = service.config_manager();
    let path = manager.config_path();
    for (initial, required) in [
        (0o644, 0o600),
        (0o400, 0o400),
        (0o600, 0o600),
        (0o4600, 0o600),
    ] {
        fs::set_permissions(path, fs::Permissions::from_mode(initial)).unwrap();
        let before = inventory(env.root());
        let inode = fs::metadata(path).unwrap().ino();
        let version = manager.snapshot().unwrap().version;
        service
            .patch_config(
                "main",
                None,
                &ConfigPatch::default(),
                &version,
                Validatable::validate,
            )
            .unwrap();
        let metadata = fs::metadata(path).unwrap();
        assert_eq!(metadata.permissions().mode() & 0o7777, required);
        assert_eq!(metadata.ino(), inode);
        assert_eq!(inventory(env.root()), before);
    }
}

#[cfg(unix)]
#[test]
fn rejected_mutations_and_queries_preserve_existing_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let env = TestCcrEnv::new();
    let service = service();
    service.add_config("main".into(), section()).unwrap();
    let manager = service.config_manager();
    let path = manager.config_path();
    fs::set_permissions(path, fs::Permissions::from_mode(0o644)).unwrap();
    let before = inventory(env.root());
    service.list_configs().unwrap();
    service.get_current().unwrap();
    service.validate_all().unwrap();
    let version = manager.snapshot().unwrap().version;
    let patch = ConfigPatch::default();
    assert!(
        service
            .patch_config("main", None, &patch, "stale", Validatable::validate)
            .is_err()
    );
    assert!(
        service
            .patch_config("main", None, &patch, &version, |_| {
                Err(CcrError::ValidationError("rejected fixture".into()))
            })
            .is_err()
    );
    assert_eq!(
        fs::metadata(path).unwrap().permissions().mode() & 0o777,
        0o644
    );
    assert_eq!(inventory(env.root()), before);
}

#[cfg(unix)]
#[test]
fn no_op_hardening_survives_uncommitted_journal_rollback() {
    use ccr_core::core::write_journal::WriteJournal;
    use std::os::unix::fs::PermissionsExt;

    let env = TestCcrEnv::new();
    let service = service();
    service.add_config("main".into(), section()).unwrap();
    let manager = service.config_manager();
    let path = manager.config_path();
    fs::set_permissions(path, fs::Permissions::from_mode(0o644)).unwrap();
    let before = inventory(env.root());
    let journal = WriteJournal::begin(&[path.to_path_buf()]).unwrap();
    manager.mutate(|_| Ok(())).unwrap();
    assert!(journal.changed_paths().is_empty());
    journal.verify().unwrap();
    drop(journal);
    assert_eq!(
        fs::metadata(path).unwrap().permissions().mode() & 0o777,
        0o600
    );
    assert_eq!(inventory(env.root()), before);
}

#[cfg(unix)]
#[test]
fn no_op_hardening_rejects_an_active_journal_version_conflict() {
    use ccr_core::core::write_journal::WriteJournal;
    use std::os::unix::fs::PermissionsExt;

    let env = TestCcrEnv::new();
    let service = service();
    service.add_config("main".into(), section()).unwrap();
    let manager = service.config_manager();
    let path = manager.config_path();
    fs::set_permissions(path, fs::Permissions::from_mode(0o644)).unwrap();
    let journal = WriteJournal::begin(&[path.to_path_buf()]).unwrap();
    journal.expect_version(path, "stale".into()).unwrap();
    let before = inventory(env.root());
    assert!(manager.mutate(|_| Ok(())).is_err());
    drop(journal);
    assert_eq!(
        fs::metadata(path).unwrap().permissions().mode() & 0o777,
        0o644
    );
    assert_eq!(inventory(env.root()), before);
}

#[test]
fn explicit_platform_and_legacy_claude_ignore_registry_order() {
    let env = TestCcrEnv::new();
    for registry in [
        "[codex]\nenabled=true\n[claude]\nenabled=true\n",
        "[claude]\nenabled=true\n[codex]\nenabled=true\n",
    ] {
        fs::write(env.root().join("config.toml"), registry).unwrap();
        let before = inventory(env.root());
        assert!(
            ConfigManager::for_platform("codex")
                .unwrap()
                .config_path()
                .ends_with("platforms/codex/profiles.toml")
        );
        assert!(
            ConfigManager::with_default()
                .unwrap()
                .config_path()
                .ends_with("platforms/claude/profiles.toml")
        );
        assert!(ConfigManager::for_platform("../unknown").is_err());
        assert_eq!(before, inventory(env.root()));
    }
}

#[test]
fn strict_patch_rejects_invalid_missing_duplicate_and_stale_without_writes() {
    let env = TestCcrEnv::new();
    let service = service();
    service.add_config("main".into(), section()).unwrap();
    service.add_config("other".into(), section()).unwrap();
    let manager = service.config_manager();
    let version = manager.snapshot().unwrap().version;
    let invalid = ConfigPatch {
        fields: IndexMap::from([(
            "enabled".into(),
            FieldPatch::Set(toml::Value::String("secret-sentinel".into())),
        )]),
    };
    let empty = ConfigPatch::default();
    for (name, rename, patch, version) in [
        ("main", None, &invalid, version.as_str()),
        ("missing", None, &empty, version.as_str()),
        ("main", Some("other"), &empty, version.as_str()),
        ("main", Some("settings"), &empty, version.as_str()),
        ("main", None, &empty, "stale-token"),
    ] {
        let before = inventory(env.root());
        let error = service
            .patch_config(name, rename, patch, version, Validatable::validate)
            .unwrap_err();
        assert!(!error.to_string().contains("secret-sentinel"));
        assert_eq!(before, inventory(env.root()));
    }
    for (old, new) in [("missing", "missing"), ("main", "other")] {
        let before = inventory(env.root());
        assert!(service.update_config(old, new.into(), section()).is_err());
        assert_eq!(before, inventory(env.root()));
    }
}

#[test]
fn patch_preserves_unedited_fields_and_renames_current_and_default() {
    let _env = TestCcrEnv::new();
    let service = service();
    service.add_config("main".into(), section()).unwrap();
    let manager = service.config_manager();
    let path = manager.config_path();
    let mut value: toml::Value = toml::from_str(&fs::read_to_string(path).unwrap()).unwrap();
    value["main"]
        .as_table_mut()
        .unwrap()
        .insert("future_field".into(), toml::Value::String("keep".into()));
    value.as_table_mut().unwrap().insert(
        "settings".into(),
        toml::Value::Table(toml::map::Map::from_iter([(
            "future_setting".into(),
            toml::Value::String("keep".into()),
        )])),
    );
    fs::write(path, toml::to_string(&value).unwrap()).unwrap();
    let before = manager.snapshot().unwrap();
    let patch = ConfigPatch {
        fields: IndexMap::from([
            (
                "description".into(),
                FieldPatch::Set(toml::Value::String("changed".into())),
            ),
            ("model".into(), FieldPatch::Remove),
        ]),
    };
    service
        .patch_config(
            "main",
            Some("renamed"),
            &patch,
            &before.version,
            Validatable::validate,
        )
        .unwrap();
    let after = manager.snapshot().unwrap();
    let section = &after.config.sections["renamed"];
    assert_eq!(section.description.as_deref(), Some("changed"));
    assert!(section.model.is_none());
    assert_eq!(
        section.auth_token,
        before.config.sections["main"].auth_token
    );
    assert_eq!(section.other["future_field"].as_str(), Some("keep"));
    assert_eq!(after.config.current_config, "renamed");
    assert_eq!(after.config.default_config, "renamed");
    let raw: toml::Value = toml::from_str(&fs::read_to_string(path).unwrap()).unwrap();
    assert_eq!(raw["settings"]["future_setting"].as_str(), Some("keep"));
    let disk = inventory(path.parent().unwrap());
    assert!(
        service
            .patch_config(
                "renamed",
                None,
                &patch,
                &before.version,
                Validatable::validate
            )
            .is_err()
    );
    assert_eq!(disk, inventory(path.parent().unwrap()));
}

#[test]
fn platform_validation_runs_inside_patch_before_persistence() {
    let env = TestCcrEnv::new();
    let service = service();
    service.add_config("main".into(), section()).unwrap();
    let manager = service.config_manager();
    let version = manager.snapshot().unwrap().version;
    let patch = ConfigPatch {
        fields: IndexMap::from([(
            "auth_mode".into(),
            FieldPatch::Set(toml::Value::String("invalid".into())),
        )]),
    };
    let before = inventory(env.root());
    let error = service
        .patch_config("main", None, &patch, &version, |section| {
            assert_eq!(section.other["auth_mode"].as_str(), Some("invalid"));
            Err(CcrError::ValidationError("auth_mode 无效".into()))
        })
        .unwrap_err();
    assert!(error.to_string().contains("auth_mode"));
    assert_eq!(before, inventory(env.root()));
}

#[test]
fn leaf_cas_detects_nonparticipating_replacement() {
    let env = TestCcrEnv::new();
    let service = service();
    service.add_config("main".into(), section()).unwrap();
    let manager = service.config_manager();
    let path = manager.config_path();
    let error = manager
        .mutate(|config| {
            config.get_section_mut("main")?.description = Some("CCR update".into());
            // A process that does not use the repository resource lock.
            fs::write(path, b"external writer bytes").unwrap();
            Ok(())
        })
        .unwrap_err();
    assert!(error.to_string().contains("版本冲突"));
    assert_eq!(fs::read(path).unwrap(), b"external writer bytes");
    assert!(!env.root().join("backups").exists());
}

#[test]
#[ignore = "child process used by the two cross-process tests"]
fn subprocess_adapter_worker() {
    let kind = std::env::var("CCR_REPOSITORY_TEST_ADAPTER").unwrap();
    let name = std::env::var("CCR_REPOSITORY_TEST_NAME").unwrap();
    let barrier = PathBuf::from(std::env::var_os("CCR_REPOSITORY_TEST_BARRIER").unwrap());
    fs::write(barrier.join(format!("{name}.ready")), []).unwrap();
    wait_for(&barrier.join("start"));
    match kind.as_str() {
        "platform" => {
            let paths = PlatformPaths::new(Platform::Claude).unwrap();
            base::mutate_profiles(&paths.profiles_file, "claude", &paths, |profiles| {
                profiles.insert(name.clone(), base::section_to_profile(&section()));
                Ok(())
            })
            .unwrap();
        }
        "desktop" => {
            let path = ConfigManager::for_platform("claude")
                .unwrap()
                .config_path()
                .to_path_buf();
            let alias = path.parent().unwrap().join(".").join("profiles.toml");
            ConfigService::new(Arc::new(ConfigManager::new(alias)))
                .add_config(name.clone(), section())
                .unwrap();
        }
        #[cfg(windows)]
        "verbatim" => {
            let path = ConfigManager::for_platform("claude")
                .unwrap()
                .config_path()
                .canonicalize()
                .unwrap();
            ConfigService::new(Arc::new(ConfigManager::new(path)))
                .add_config(name.clone(), section())
                .unwrap();
        }
        "service" => service().add_config(name.clone(), section()).unwrap(),
        _ => panic!("unknown test adapter"),
    }
    fs::write(barrier.join(format!("{name}.done")), []).unwrap();
}

fn wait_for(path: &Path) {
    let deadline = Instant::now() + Duration::from_secs(15);
    while !path.exists() {
        assert!(
            Instant::now() < deadline,
            "barrier timeout: {}",
            path.display()
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn assert_blocked(child: &mut Child) {
    match child.try_wait() {
        Ok(None) => {}
        Ok(Some(status)) => {
            let mut stdout = Vec::new();
            let mut stderr = Vec::new();
            if let Some(mut pipe) = child.stdout.take() {
                let _ = std::io::Read::read_to_end(&mut pipe, &mut stdout);
            }
            if let Some(mut pipe) = child.stderr.take() {
                let _ = std::io::Read::read_to_end(&mut pipe, &mut stderr);
            }
            panic!(
                "child exited with {status}\nstdout:\n{}\nstderr:\n{}",
                String::from_utf8_lossy(&stdout),
                String::from_utf8_lossy(&stderr)
            );
        }
        Err(error) => panic!("failed to poll child: {error}"),
    }
}

fn spawn_adapter(env: &TestCcrEnv, barrier: &Path, kind: &str, name: &str) -> Child {
    Command::new(std::env::current_exe().unwrap())
        .args([
            "--ignored",
            "--exact",
            "managers::config::repository_tests::subprocess_adapter_worker",
            "--nocapture",
        ])
        .env("CCR_ROOT", env.root())
        .env("CCR_LOCK_DIR", env.lock_dir())
        .env("CCR_REPOSITORY_TEST_ADAPTER", kind)
        .env("CCR_REPOSITORY_TEST_NAME", name)
        .env("CCR_REPOSITORY_TEST_BARRIER", barrier)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap()
}

fn concurrent_adapters(first_kind: &str) {
    let env = TestCcrEnv::new();
    let manager = ConfigManager::for_platform("claude").unwrap();
    if first_kind == "verbatim" {
        manager.ensure_initialized().unwrap();
    }
    for round in 0..3 {
        let barrier = env.home().join(format!("barrier-{round}"));
        fs::create_dir(&barrier).unwrap();
        let first_name = format!("first-{round}");
        let second_name = format!("second-{round}");
        let lock = manager.lock_mutation().unwrap();
        let mut first = spawn_adapter(&env, &barrier, first_kind, &first_name);
        let mut second = spawn_adapter(&env, &barrier, "service", &second_name);
        wait_for(&barrier.join(format!("{first_name}.ready")));
        wait_for(&barrier.join(format!("{second_name}.ready")));
        fs::write(barrier.join("start"), []).unwrap();
        // Both independent processes must wait for the same resource lock.
        let deadline = Instant::now() + Duration::from_millis(200);
        while Instant::now() < deadline {
            assert_blocked(&mut first);
            assert_blocked(&mut second);
            std::thread::sleep(Duration::from_millis(10));
        }
        drop(lock);
        for child in [first, second] {
            let output = child.wait_with_output().unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        let config = manager.load().unwrap();
        assert!(config.sections.contains_key(&first_name));
        assert!(config.sections.contains_key(&second_name));
        assert_eq!(config.sections.len(), (round + 1) * 2);
    }
}

#[test]
fn platform_and_service_processes_keep_both_updates() {
    concurrent_adapters("platform");
}

#[test]
fn desktop_and_service_processes_keep_both_updates() {
    concurrent_adapters("desktop");
}

#[cfg(windows)]
#[test]
fn windows_verbatim_and_service_processes_keep_both_updates() {
    concurrent_adapters("verbatim");
}

#[test]
fn current_resolution_reports_repair_without_changing_input() {
    let profiles = IndexMap::from([("main".into(), ProfileConfig::new())]);
    let resolution = base::resolve_file_current_profile(&profiles, Some("main"), Some("stale"));
    assert_eq!(resolution.current.as_deref(), Some("main"));
    assert_eq!(
        resolution.repair,
        Some(base::CurrentProfileRepair::Registry(Some("main".into())))
    );
    let fallback = base::resolve_file_current_profile(&profiles, Some("stale"), Some("main"));
    assert_eq!(
        fallback.repair,
        Some(base::CurrentProfileRepair::Profiles("main".into()))
    );
}

#[test]
fn current_marker_reads_only_declared_marker_without_modifying_profiles() {
    let env = TestCcrEnv::new();
    let manager = ConfigManager::for_platform("claude").unwrap();
    let path = manager.config_path();
    assert_eq!(base::load_current_profile_marker(path).unwrap(), None);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, "[main]\nmodel = \"test-model\"\n").unwrap();
    let before = inventory(env.root());
    assert!(
        base::load_profiles_from_toml(path)
            .unwrap()
            .contains_key("main")
    );
    assert_eq!(base::load_current_profile_marker(path).unwrap(), None);
    assert_eq!(before, inventory(env.root()));

    fs::write(
        path,
        "default_config = \"main\"\ncurrent_config = \"main\"\n[main]\n",
    )
    .unwrap();
    let before = inventory(env.root());
    assert_eq!(
        base::load_current_profile_marker(path).unwrap().as_deref(),
        Some("main")
    );
    assert_eq!(before, inventory(env.root()));
}

#[test]
fn service_queries_do_not_infer_current_from_simplified_profile_order() {
    let env = TestCcrEnv::new();
    let service = service();
    let manager = service.config_manager();
    let path = manager.config_path();
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let source = toml::to_string(&IndexMap::from([("main", section())])).unwrap();
    fs::write(path, &source).unwrap();
    let before = inventory(env.root());
    // The low-level format adapter keeps its documented compatibility default.
    assert_eq!(
        base::parse_config_from_str(&source).unwrap().current_config,
        "main"
    );
    assert!(manager.snapshot().unwrap().config.current_config.is_empty());
    assert!(manager.load().unwrap().current_config.is_empty());
    let list = service.list_configs().unwrap();
    assert!(list.current_config.is_empty());
    assert!(list.configs.iter().all(|config| !config.is_current));
    assert!(!service.get_config("main").unwrap().is_current);
    assert!(matches!(
        service.get_current(),
        Err(CcrError::ConfigSectionNotFound(_))
    ));
    assert_eq!(base::load_current_profile_marker(path).unwrap(), None);
    assert_eq!(before, inventory(env.root()));
}

#[test]
fn unrelated_service_edits_preserve_absent_current_until_explicit_activation() {
    let _env = TestCcrEnv::new();
    let service = service();
    let manager = service.config_manager();
    let path = manager.config_path();
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        path,
        toml::to_string(&IndexMap::from([("main", section())])).unwrap(),
    )
    .unwrap();
    let version = manager.snapshot().unwrap().version;
    let patch = ConfigPatch {
        fields: IndexMap::from([(
            "description".into(),
            FieldPatch::Set(toml::Value::String("patched".into())),
        )]),
    };
    service
        .patch_config("main", None, &patch, &version, Validatable::validate)
        .unwrap();
    let raw: toml::Value = toml::from_str(&fs::read_to_string(path).unwrap()).unwrap();
    assert!(raw.get("current_config").is_none());
    assert!(raw.get("default_config").is_none());
    assert_eq!(raw["main"]["description"].as_str(), Some("patched"));
    service
        .update_config("main", "main".into(), section())
        .unwrap();
    let raw: toml::Value = toml::from_str(&fs::read_to_string(path).unwrap()).unwrap();
    assert!(raw.get("current_config").is_none());
    assert_eq!(base::load_current_profile_marker(path).unwrap(), None);
    service.set_current("main").unwrap();
    assert_eq!(service.get_current().unwrap().name, "main");
    assert_eq!(
        base::load_current_profile_marker(path).unwrap().as_deref(),
        Some("main")
    );
}

#[test]
fn clear_current_keeps_missing_absent_and_propagates_corrupt_or_unreadable() {
    let env = TestCcrEnv::new();
    let manager = ConfigManager::for_platform("claude").unwrap();
    let before = inventory(env.root());
    manager.clear_current_if_present().unwrap();
    base::update_current_config(manager.config_path(), "main").unwrap();
    assert_eq!(before, inventory(env.root()));
    let path = manager.config_path();
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, "invalid TOML [").unwrap();
    let before = inventory(env.root());
    assert!(matches!(
        manager.clear_current_if_present(),
        Err(CcrError::ConfigFormatInvalid(_))
    ));
    assert!(matches!(
        base::update_current_config(path, "main"),
        Err(CcrError::ConfigFormatInvalid(_))
    ));
    assert!(matches!(
        base::load_current_profile_marker(path),
        Err(CcrError::ConfigFormatInvalid(_))
    ));
    assert_eq!(before, inventory(env.root()));
    fs::remove_file(path).unwrap();
    fs::create_dir(path).unwrap();
    assert!(matches!(
        manager.clear_current_if_present(),
        Err(CcrError::FileIoError(_))
    ));
    assert!(matches!(
        base::update_current_config(path, "main"),
        Err(CcrError::FileIoError(_))
    ));
    assert!(matches!(
        base::load_current_profile_marker(path),
        Err(CcrError::FileIoError(_))
    ));
    assert!(path.is_dir());
}

#[test]
fn platform_adapter_keeps_unedited_toml_datetime_extensions() {
    let _env = TestCcrEnv::new();
    let service = service();
    service.add_config("main".into(), section()).unwrap();
    let manager = service.config_manager();
    let path = manager.config_path();
    let source = fs::read_to_string(path).unwrap() + "\ncustom_timestamp = 2026-01-01T00:00:00Z\n";
    fs::write(path, source).unwrap();
    let original = manager.load().unwrap().sections["main"].other["custom_timestamp"].clone();
    let paths = PlatformPaths::new(Platform::Claude).unwrap();
    base::mutate_profiles(&paths.profiles_file, "claude", &paths, |profiles| {
        profiles.get_mut("main").unwrap().description = Some("updated".into());
        Ok(())
    })
    .unwrap();
    let after = manager.load().unwrap();
    assert_eq!(
        after.sections["main"].description.as_deref(),
        Some("updated")
    );
    assert_eq!(after.sections["main"].other["custom_timestamp"], original);
}

#[test]
fn resource_identity_uses_normalized_path_and_separates_roots() {
    use super::repository::config_resource_name;
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("a/profiles.toml");
    assert_eq!(
        config_resource_name(&path).unwrap(),
        config_resource_name(&temp.path().join("a/../a/./profiles.toml")).unwrap()
    );
    assert_ne!(
        config_resource_name(&path).unwrap(),
        config_resource_name(&temp.path().join("b/profiles.toml")).unwrap()
    );
    #[cfg(windows)]
    assert_eq!(
        config_resource_name(&path).unwrap(),
        config_resource_name(&PathBuf::from(path.to_string_lossy().to_uppercase())).unwrap()
    );
}

#[cfg(windows)]
#[test]
fn resource_identity_matches_windows_verbatim_paths() {
    use super::repository::config_resource_name;
    let temp = tempfile::tempdir().unwrap();
    let canonical = temp.path().canonicalize().unwrap().join("profiles.toml");
    let plain = PathBuf::from(canonical.to_str().unwrap().strip_prefix(r"\\?\").unwrap());
    assert_eq!(
        config_resource_name(&plain).unwrap(),
        config_resource_name(&canonical).unwrap()
    );
    assert_eq!(
        config_resource_name(Path::new(r"\\server\share\profiles.toml")).unwrap(),
        config_resource_name(Path::new(r"\\?\UNC\server\share\profiles.toml")).unwrap()
    );
}

#[test]
fn adding_to_empty_inactive_document_preserves_current_marker() {
    let _env = TestCcrEnv::new();
    let service = service();
    service.config_manager().ensure_initialized().unwrap();
    service.config_manager().clear_current_if_present().unwrap();
    service.add_config("main".into(), section()).unwrap();
    let config = service.load_config().unwrap();
    assert_eq!(config.default_config, "main");
    assert!(config.current_config.is_empty());
}

#[cfg(windows)]
#[test]
fn uppercase_profile_path_keeps_standard_backup_policy() {
    let env = TestCcrEnv::new();
    let service = service();
    service.add_config("main".into(), section()).unwrap();
    let path = service.config_manager().config_path();
    let alias = PathBuf::from(path.to_str().unwrap().to_uppercase());
    ConfigManager::new(alias)
        .mutate(|config| {
            config.get_section_mut("main")?.description = Some("updated".into());
            Ok(())
        })
        .unwrap();
    let backup_dir = env.root().join("backups/claude");
    assert!(backup_dir.is_dir());
    assert_eq!(fs::read_dir(backup_dir).unwrap().count(), 1);
}
