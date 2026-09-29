#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;
use ccr_cli::application::profile_lifecycle::ProfileStatus;
use ccr_cli::platforms::create_platform;
use ccr_config::Platform;
use serde_json::json;
use std::fs;

fn fixture() -> (impl Drop, std::sync::MutexGuard<'static, ()>) {
    let desktop = crate::test_support::lock_env();
    (
        ccr_cli::application::profile_contract::isolated_fixture(Platform::Claude),
        desktop,
    )
}

fn patch(value: serde_json::Value) -> ConfigPatchInput {
    serde_json::from_value(value).unwrap()
}

fn manager() -> ConfigManager {
    ConfigManager::for_platform("claude").unwrap()
}

#[tokio::test]
async fn actual_handlers_switch_and_enable_commit_runtime_and_policy() {
    let _home = fixture();
    let switched = switch_config("new".into(), Some("claude".into()), None)
        .await
        .unwrap();
    assert!(switched.outcome.unwrap().activation_committed);
    manager()
        .mutate(|config| {
            config.get_section_mut("old")?.enabled = Some(false);
            Ok(())
        })
        .unwrap();
    assert!(
        switch_config("old".into(), Some("claude".into()), None)
            .await
            .is_err()
    );
    let enabled = switch_config("old".into(), Some("claude".into()), Some(true))
        .await
        .unwrap();
    assert!(enabled.outcome.unwrap().activation_committed);
    let config = manager().load().unwrap();
    assert_eq!(config.current_config, "old");
    assert!(config.get_section("old").unwrap().is_enabled());
    assert_eq!(
        create_platform(Platform::Claude)
            .unwrap()
            .get_current_profile()
            .unwrap()
            .as_deref(),
        Some("old")
    );
    let settings: serde_json::Value = serde_json::from_slice(
        &fs::read(
            create_platform(Platform::Claude)
                .unwrap()
                .get_settings_path(),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(settings["env"]["ANTHROPIC_MODEL"], "model-old");
    let listed = list_configs(Some("claude".into())).await.unwrap();
    assert!(listed.iter().all(|row| {
        !row.auth_token
            .contains(ccr_cli::application::profile_contract::SENTINEL)
    }));
}

#[tokio::test]
async fn legacy_and_unsupported_platform_requests_never_write() {
    let _home = fixture();
    let before = fs::read(manager().config_path()).unwrap();
    for platform in [
        None,
        Some("codex".into()),
        Some("grok".into()),
        Some("unknown".into()),
    ] {
        let expected = if platform.is_none() {
            "platform_required"
        } else {
            "config_platform_unsupported"
        };
        assert!(
            switch_config("new".into(), platform.clone(), None)
                .await
                .unwrap_err()
                .contains(expected)
        );
        assert!(
            add_config(
                "added".into(),
                platform.clone(),
                Some(ConfigPatchInput::default())
            )
            .await
            .unwrap_err()
            .contains(expected)
        );
        assert!(
            update_config(
                "new".into(),
                platform.clone(),
                ConfigPatchInput::default(),
                None
            )
            .await
            .unwrap_err()
            .contains(expected)
        );
        assert!(
            rename_config("new".into(), "renamed".into(), platform.clone())
                .await
                .unwrap_err()
                .contains(expected)
        );
        assert!(
            duplicate_config("new".into(), "copy".into(), platform.clone())
                .await
                .unwrap_err()
                .contains(expected)
        );
        assert!(
            delete_config(
                "new".into(),
                platform,
                Some("desktop-confirm:delete_config".into())
            )
            .await
            .unwrap_err()
            .contains(expected)
        );
    }
    assert_eq!(before, fs::read(manager().config_path()).unwrap());
}

#[test]
fn strict_patch_rejects_unknown_wrong_types_missing_collision_without_write() {
    let _home = fixture();
    let before = fs::read(manager().config_path()).unwrap();
    for value in [
        json!({"unknown": "secret"}),
        json!({"model": false}),
        json!({"enabled": "false"}),
        json!({"enabled": null}),
        json!({"tags": [2]}),
        json!([]),
    ] {
        assert!(serde_json::from_value::<ConfigPatchInput>(value).is_err());
    }
    assert!(
        adapter::update(
            Platform::Claude,
            "missing",
            patch(json!({"description":"x"})),
            None
        )
        .is_err()
    );
    assert!(
        adapter::update(
            Platform::Claude,
            "new",
            patch(json!({"provider_type":"invalid"})),
            None
        )
        .is_err()
    );
    assert!(
        adapter::update(
            Platform::Claude,
            "old",
            patch(json!({"enabled":false})),
            None
        )
        .is_err()
    );
    assert!(adapter::rename(Platform::Claude, "new", "old").is_err());
    assert!(adapter::duplicate(Platform::Claude, "new", "old").is_err());
    assert!(adapter::duplicate(Platform::Claude, "missing", "copy").is_err());
    assert!(adapter::add(Platform::Claude, "new", ConfigPatchInput::default()).is_err());
    assert!(adapter::delete(Platform::Claude, "missing").is_err());
    assert_eq!(before, fs::read(manager().config_path()).unwrap());
}

#[test]
fn patch_retains_secret_unknown_fields_inactive_marker_and_checks_version() {
    let _home = fixture();
    manager()
        .mutate(|config| {
            config.current_config.clear();
            let section = config.get_section_mut("new")?;
            let mut extension: ccr_config::ConfigSection =
                toml::from_str("extension_time = 2026-09-28T12:00:00Z").unwrap();
            section.other.insert(
                "extension_time".into(),
                extension.other.shift_remove("extension_time").unwrap(),
            );
            section.account = Some("account".into());
            Ok(())
        })
        .unwrap();
    let version = manager().snapshot().unwrap().version;
    adapter::update(
        Platform::Claude,
        "new",
        patch(json!({"description":"changed", "model":null})),
        Some(&version),
    )
    .unwrap();
    let saved = manager().load().unwrap();
    let section = saved.get_section("new").unwrap();
    assert_eq!(section.account.as_deref(), Some("account"));
    assert_eq!(
        section.auth_token.as_ref().unwrap().expose(),
        ccr_cli::application::profile_contract::SENTINEL
    );
    assert!(section.model.is_none());
    assert!(section.other["extension_time"].is_datetime());
    assert_eq!(saved.current_config, "");
    let before = fs::read(manager().config_path()).unwrap();
    assert!(
        adapter::update(
            Platform::Claude,
            "new",
            patch(json!({"description":"stale"})),
            Some(&version)
        )
        .is_err()
    );
    assert_eq!(before, fs::read(manager().config_path()).unwrap());
    adapter::duplicate(Platform::Claude, "new", "copy").unwrap();
    assert_eq!(manager().load().unwrap().current_config, "");
    assert!(
        manager().load().unwrap().get_section("copy").unwrap().other["extension_time"]
            .is_datetime()
    );
}

#[test]
fn active_rename_uses_application_and_preserves_runtime() {
    let _home = fixture();
    let renamed = adapter::rename(Platform::Claude, "old", "renamed").unwrap();
    let outcome = renamed.outcome.unwrap();
    assert!(matches!(
        outcome.status,
        ProfileStatus::Applied | ProfileStatus::AppliedWithWarning
    ));
    assert!(outcome.activation_committed);
    let saved = manager().load().unwrap();
    assert!(!saved.sections.contains_key("old"));
    assert_eq!(saved.current_config, "renamed");
    assert_eq!(
        create_platform(Platform::Claude)
            .unwrap()
            .get_current_profile()
            .unwrap()
            .as_deref(),
        Some("renamed")
    );
}

#[test]
fn enable_stage_failure_and_replay_contract() {
    let _desktop = crate::test_support::lock_env();
    ccr_cli::application::profile_contract::enable_lifecycle();
}

#[tokio::test]
async fn actual_handlers_protect_registry_fallback_active_profile() {
    let _home = fixture();
    create_platform(Platform::Claude)
        .unwrap()
        .apply_profile("new")
        .unwrap();
    manager()
        .mutate(|config| {
            config.current_config = "ghost".into();
            Ok(())
        })
        .unwrap();
    let before = fs::read(manager().config_path()).unwrap();
    let rows = list_configs(Some("claude".into())).await.unwrap();
    assert!(
        rows.iter()
            .find(|row| row.name == "new")
            .unwrap()
            .is_current
    );
    assert!(
        update_config(
            "new".into(),
            Some("claude".into()),
            patch(json!({"enabled":false})),
            None
        )
        .await
        .unwrap_err()
        .contains("active_config_cannot_be_disabled")
    );
    assert!(
        delete_config(
            "new".into(),
            Some("claude".into()),
            Some("desktop-confirm:delete_config".into())
        )
        .await
        .unwrap_err()
        .contains("active_config_cannot_be_deleted")
    );
    assert_eq!(before, fs::read(manager().config_path()).unwrap());
}

#[tokio::test]
async fn actual_handlers_crud_preserve_typed_fields_and_confirmation() {
    let _home = fixture();
    let platform = Some("claude".into());
    add_config("added".into(), platform.clone(), Some(patch(json!({"base_url":"https://fixture.invalid", "auth_token":"synthetic-add", "description":"added"})))).await.unwrap();
    update_config(
        "added".into(),
        platform.clone(),
        patch(json!({"account":"account"})),
        None,
    )
    .await
    .unwrap();
    duplicate_config("added".into(), "copy".into(), platform.clone())
        .await
        .unwrap();
    assert_eq!(
        manager()
            .load()
            .unwrap()
            .get_section("copy")
            .unwrap()
            .account
            .as_deref(),
        Some("account")
    );
    let renamed = rename_config("copy".into(), "renamed".into(), platform.clone())
        .await
        .unwrap();
    assert!(!renamed.outcome.unwrap().activation_committed);
    assert!(
        delete_config("renamed".into(), platform.clone(), None)
            .await
            .is_err()
    );
    delete_config(
        "renamed".into(),
        platform,
        Some("desktop-confirm:delete_config".into()),
    )
    .await
    .unwrap();
    assert!(!manager().load().unwrap().sections.contains_key("renamed"));
}

#[test]
fn direct_mutation_waits_for_application_lock_and_rechecks_activation() {
    use std::sync::mpsc;
    use std::time::Duration;
    let _home = fixture();
    let lock = ccr_cli::application::profile_lifecycle::operation_lock(Platform::Claude).unwrap();
    let (started, waiting) = mpsc::channel();
    let (completed, result) = mpsc::channel();
    let worker = std::thread::spawn(move || {
        started.send(()).unwrap();
        completed
            .send(adapter::update(
                Platform::Claude,
                "new",
                patch(json!({"enabled":false})),
                None,
            ))
            .unwrap();
    });
    waiting.recv_timeout(Duration::from_secs(2)).unwrap();
    assert!(matches!(
        result.recv_timeout(Duration::from_millis(100)),
        Err(mpsc::RecvTimeoutError::Timeout)
    ));
    // Model a lifecycle commit while its operation guard remains held.
    create_platform(Platform::Claude)
        .unwrap()
        .apply_profile("new")
        .unwrap();
    drop(lock);
    assert!(
        result
            .recv_timeout(Duration::from_secs(3))
            .unwrap()
            .is_err()
    );
    worker.join().unwrap();
    assert!(
        manager()
            .load()
            .unwrap()
            .get_section("new")
            .unwrap()
            .is_enabled()
    );
}

fn wait_for(path: &std::path::Path) {
    let end = std::time::Instant::now() + std::time::Duration::from_secs(15);
    while !path.exists() {
        assert!(std::time::Instant::now() < end, "fixture barrier timeout");
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

#[test]
#[ignore = "isolated child process for the repository concurrency fixture"]
fn config_subprocess() {
    let role = std::env::var("CCR_T03_ROLE").unwrap();
    let barrier = std::path::PathBuf::from(std::env::var_os("CCR_T03_BARRIER").unwrap());
    fs::write(barrier.join(format!("{role}.ready")), []).unwrap();
    wait_for(&barrier.join("start"));
    if role == "desktop" {
        adapter::update(
            Platform::Claude,
            "new",
            patch(json!({"description":"desktop-change"})),
            None,
        )
        .unwrap();
    } else {
        let service = ConfigService::for_platform("claude").unwrap();
        service
            .config_manager()
            .mutate(|config| {
                config.get_section_mut("new")?.account = Some("service-change".into());
                Ok(())
            })
            .unwrap();
    }
}

#[test]
fn desktop_and_service_processes_preserve_independent_changes() {
    use std::process::{Command, Stdio};
    let _home = fixture();
    for round in 0..3 {
        manager()
            .mutate(|config| {
                let section = config.get_section_mut("new")?;
                section.description = None;
                section.account = None;
                Ok(())
            })
            .unwrap();
        let barrier = ccr_config::PlatformPaths::new(Platform::Claude)
            .unwrap()
            .root
            .join(format!("t03-barrier-{round}"));
        fs::create_dir(&barrier).unwrap();
        let lock = manager().lock_mutation().unwrap();
        let mut children: Vec<_> = ["desktop", "service"]
            .into_iter()
            .map(|role| {
                Command::new(std::env::current_exe().unwrap())
                    .args([
                        "--exact",
                        "commands::config::contract_tests::config_subprocess",
                        "--ignored",
                    ])
                    .env("CCR_T03_ROLE", role)
                    .env("CCR_T03_BARRIER", &barrier)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                    .unwrap()
            })
            .collect();
        for role in ["desktop", "service"] {
            wait_for(&barrier.join(format!("{role}.ready")));
        }
        fs::write(barrier.join("start"), []).unwrap();
        let end = std::time::Instant::now() + std::time::Duration::from_millis(200);
        while std::time::Instant::now() < end {
            for child in &mut children {
                assert!(child.try_wait().unwrap().is_none());
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        drop(lock);
        for child in children {
            let output = child.wait_with_output().unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        let config = manager().load().unwrap();
        assert_eq!(
            config.get_section("new").unwrap().description.as_deref(),
            Some("desktop-change")
        );
        assert_eq!(
            config.get_section("new").unwrap().account.as_deref(),
            Some("service-change")
        );
    }
}
