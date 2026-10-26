#![allow(clippy::unwrap_used, clippy::expect_used)]

use ccr_app::config::{backups, profile_patch, source};
use ccr_config::{ConfigManager, ConfigService, PlatformPaths};
use ccr_core::core::lock::LockManager;
use serde_json::json;
use std::{fs, path::Path, sync::Arc};

fn paths(root: &Path) -> PlatformPaths {
    PlatformPaths {
        root: root.into(),
        registry_file: root.join("config.toml"),
        platform_dir: root.join("platforms/claude"),
        profiles_file: root.join("platforms/claude/profiles.toml"),
        settings_file: root.join("runtime/settings.json"),
        history_file: root.join("history/claude.json"),
        backups_dir: root.join("backups/claude"),
    }
}

#[test]
fn source_validation_is_safe_and_does_not_create_files() {
    let temp = tempfile::tempdir().unwrap();
    let paths = paths(temp.path());
    let missing = source::read(&paths).unwrap();
    assert!(!missing.exists);
    assert!(missing.content.is_empty());
    assert!(missing.token.is_empty());
    let sentinel = "synthetic-never-in-diagnostics-secret";
    for (content, kind) in [
        (
            format!("[broken\nauth_token = '{sentinel}'"),
            source::SourceInvalidKind::Syntax,
        ),
        (
            format!("[profile]\nauth_token = ['{sentinel}']"),
            source::SourceInvalidKind::Semantic,
        ),
        (String::new(), source::SourceInvalidKind::Semantic),
    ] {
        let result = source::save(&paths, None, &content, "", false).unwrap();
        assert!(
            matches!(result, source::SourceSaveOutcome::Invalid { kind: actual, .. } if actual == kind)
        );
        assert!(!format!("{result:?}").contains(sentinel));
    }
    assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 0);
}

#[test]
fn source_canonical_round_trip_and_force_do_not_bypass_cas() {
    let temp = tempfile::tempdir().unwrap();
    let paths = paths(temp.path());
    LockManager::with_scoped_directory(&temp.path().join("locks"), || {
        source::save(
            &paths,
            None,
            "# formatting is exchange only\n[active]\nmodel='old'\n",
            "",
            false,
        )?;
        let first = source::read(&paths)?;
        assert!(first.exists);
        assert!(!first.content.contains("formatting is exchange only"));
        assert!(!paths.profiles_file.exists());
        let replacement = "[replacement]\nmodel='new'\n";
        assert!(matches!(
            source::save(&paths, Some("active"), replacement, &first.token, false)?,
            source::SourceSaveOutcome::ActivationConflict { .. }
        ));
        assert_eq!(source::read(&paths)?.token, first.token);
        assert!(matches!(
            source::save(&paths, Some("active"), replacement, &first.token, true)?,
            source::SourceSaveOutcome::Saved {
                profiles_count: 1,
                ..
            }
        ));
        assert_eq!(
            source::save(
                &paths,
                Some("active"),
                "[third]\nmodel='external-lost'\n",
                &first.token,
                true
            )?,
            source::SourceSaveOutcome::Conflict
        );
        let latest = source::read(&paths)?;
        assert!(latest.content.contains("replacement"));
        assert!(!latest.content.contains("external-lost"));
        assert!(source::export(&paths.profiles_file, false).is_err());
        assert_eq!(source::export(&paths.profiles_file, true)?, latest.content);
        Ok(())
    })
    .unwrap();
}

#[test]
fn two_source_clients_from_one_revision_have_exactly_one_winner() {
    let temp = tempfile::tempdir().unwrap();
    let paths = paths(temp.path());
    let locks = temp.path().join("locks");
    LockManager::with_scoped_directory(&locks, || {
        source::save(&paths, None, "[profile]\nmodel='baseline'\n", "", false).map(|_| ())
    })
    .unwrap();
    let token = source::read(&paths).unwrap().token;
    let barrier = Arc::new(std::sync::Barrier::new(2));
    let handles: Vec<_> = ["client-a", "client-b"]
        .into_iter()
        .map(|model| {
            let (paths, token, locks, barrier) =
                (paths.clone(), token.clone(), locks.clone(), barrier.clone());
            std::thread::spawn(move || {
                barrier.wait();
                LockManager::with_scoped_directory(&locks, || {
                    source::save(
                        &paths,
                        None,
                        &format!("[profile]\nmodel='{model}'\n"),
                        &token,
                        true,
                    )
                })
                .unwrap()
            })
        })
        .collect();
    let results: Vec<_> = handles
        .into_iter()
        .map(|handle| handle.join().unwrap())
        .collect();
    assert_eq!(
        results
            .iter()
            .filter(|outcome| matches!(outcome, source::SourceSaveOutcome::Saved { .. }))
            .count(),
        1
    );
    assert_eq!(
        results
            .iter()
            .filter(|outcome| **outcome == source::SourceSaveOutcome::Conflict)
            .count(),
        1
    );
    let content = source::read(&paths).unwrap().content;
    assert!(content.contains("client-a") ^ content.contains("client-b"));
}

#[test]
fn managed_restore_uses_safe_parser_and_keeps_preimage() {
    let temp = tempfile::tempdir().unwrap();
    let paths = paths(temp.path());
    LockManager::with_scoped_directory(&temp.path().join("locks"), || {
        source::save(&paths, None, "[profile]\nmodel='before'\n", "", false)?;
        let service = ConfigService::new(Arc::new(ConfigManager::new(&paths.profiles_file)));
        let bad = paths.platform_dir.join("profiles.toml.bad.bak");
        fs::write(
            &bad,
            "[profile]\nauth_token = ['secret-sentinel-never-show']\n",
        )
        .unwrap();
        let before = source::read(&paths)?;
        let error = backups::restore(&service, &bad).unwrap_err().to_string();
        assert!(!error.contains("secret-sentinel-never-show"));
        assert_eq!(source::read(&paths)?.token, before.token);
        let good = paths.platform_dir.join("profiles.toml.saved.bak");
        fs::write(&good, "[profile]\nmodel='restored'\n").unwrap();
        assert_eq!(
            backups::resolve_managed_restore_path(&paths.profiles_file, "profiles.toml.saved.bak")?,
            good
        );
        backups::restore(&service, &good)?;
        assert!(source::read(&paths)?.content.contains("restored"));
        assert!(
            fs::read_dir(&paths.platform_dir)
                .unwrap()
                .flatten()
                .any(|entry| {
                    entry.path() != good
                        && entry.path() != bad
                        && entry.path().extension().is_some_and(|ext| ext == "bak")
                        && fs::read_to_string(entry.path())
                            .is_ok_and(|content| content.contains("before"))
                })
        );
        for invalid in [
            "../profiles.toml.saved.bak",
            "nested/profiles.toml.saved.bak",
            "wrong.bak",
            "profiles.toml.backup",
        ] {
            assert!(backups::resolve_managed_restore_path(&paths.profiles_file, invalid).is_err());
        }
        Ok(())
    })
    .unwrap();
}

#[test]
fn cleanup_is_explicit_and_bounded_to_existing_extensions_depth() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("managed");
    fs::create_dir_all(root.join("a/b/c")).unwrap();
    for path in [
        "one.bak",
        "a/two.backup",
        "a/b/three.bak",
        "a/b/c/too-deep.bak",
        "keep.toml",
    ] {
        fs::write(root.join(path), "synthetic").unwrap();
    }
    let outside = temp.path().join("outside.bak");
    fs::write(&outside, "untouched").unwrap();
    let result = backups::clean(&root);
    assert_eq!(
        result,
        backups::BackupCleanup {
            root_exists: true,
            deleted: 3
        }
    );
    assert!(root.join("a/b/c/too-deep.bak").is_file());
    assert!(root.join("keep.toml").is_file());
    assert!(outside.is_file());
    assert_eq!(
        backups::clean(&temp.path().join("missing")),
        backups::BackupCleanup {
            root_exists: false,
            deleted: 0
        }
    );
}

#[test]
fn dedicated_patch_retains_omitted_fields_and_preserves_platform_semantics() {
    let mut claude = profile_patch::claude::build(&json!({"description":"old", "auth_token":"synthetic-token", "extra":{"unknown":1},"auth_mode":"api_key"})).unwrap();
    profile_patch::claude::patch(&mut claude, &json!({"description":null})).unwrap();
    assert!(claude.description.is_none());
    assert_eq!(
        claude.auth_token.as_ref().unwrap().expose(),
        "synthetic-token"
    );
    assert_eq!(claude.platform_data["unknown"], 1);
    profile_patch::claude::patch(&mut claude, &json!({"platform_data":null})).unwrap();
    assert!(claude.platform_data.is_empty()); // Historical whole-extra replacement.
    let codex = profile_patch::codex::build(&json!({"auth_mode":"provider_bearer_token", "preferred_auth_method":"apikey", "extra":{"auth_mode":"no_auth"}})).unwrap();
    assert_eq!(codex.platform_data["auth_mode"], "no_auth"); // Existing explicit -> extra precedence.
    assert_eq!(codex.platform_data["preferred_auth_method"], "apikey");
    for value in [
        json!({"enabled":"secret-sentinel"}),
        json!({"usage_count":-1}),
        json!({"tags":[1]}),
    ] {
        assert!(
            !profile_patch::codex::build(&value)
                .unwrap_err()
                .contains("secret-sentinel")
        );
    }
}

#[test]
fn grok_credentials_preserve_replace_and_clear_without_display_round_trip() {
    let mut profile = profile_patch::grok::build(
        &json!({"api_key":"synthetic-key", "base_url":"https://provider.invalid", "model":"grok"}),
    )
    .unwrap();
    assert_eq!(profile.platform_data["api_key"], "synthetic-key");
    profile_patch::grok::patch(
        &mut profile,
        &json!({"reasoning_effort":"high", "base_url_display":"lossy"}),
    )
    .unwrap();
    assert_eq!(
        profile.base_url.as_deref(),
        Some("https://provider.invalid")
    );
    assert_eq!(profile.platform_data["api_key"], "synthetic-key");
    profile_patch::grok::patch(
        &mut profile,
        &json!({"credential_action":"replace_env_key", "env_key":"KEY_NAME"}),
    )
    .unwrap();
    assert!(!profile.platform_data.contains_key("api_key"));
    assert_eq!(profile.platform_data["env_key"], "KEY_NAME");
    profile_patch::grok::patch(&mut profile, &json!({"credential_action":"clear"})).unwrap();
    assert!(!profile.platform_data.contains_key("env_key"));
    assert!(profile_patch::grok::build(&json!({"api_key":"one","env_key":"two"})).is_err());
}

#[test]
fn generic_patch_is_strict_typed_and_never_reports_rejected_values() {
    use ccr_app::config::mutations::parse_patch;
    use ccr_config::managers::config::FieldPatch;
    let sentinel = "synthetic-rejected-secret";
    for value in [
        json!({"unknown": sentinel}),
        json!({"auth_token": [sentinel]}),
        json!({"model": false}),
        json!({"enabled": null}),
        json!({"enabled": "false"}),
        json!({"tags": [1]}),
        json!([]),
    ] {
        let error = parse_patch(&value).err().unwrap().to_string();
        assert!(!error.contains(sentinel));
    }
    let patch = parse_patch(&json!({"description":null,"tags":["one"],"enabled":false})).unwrap();
    assert!(matches!(patch.fields["description"], FieldPatch::Remove));
    assert!(matches!(
        patch.fields["enabled"],
        FieldPatch::Set(toml::Value::Boolean(false))
    ));
    assert!(!patch.fields.contains_key("auth_token"));
}

#[test]
fn native_advanced_fields_match_dedicated_patch_capabilities() {
    for validate in [
        profile_patch::claude::validate_fields,
        profile_patch::codex::validate_fields,
        profile_patch::grok::validate_fields,
    ] {
        assert_eq!(
            validate(&json!({"unknown_secret":"never-print-sentinel"})).unwrap_err(),
            "unknown_profile_patch_field"
        );
        assert!(validate(&json!([])).is_err());
        assert!(validate(&json!({"name":"renamed","description":null})).is_ok());
    }
    assert!(
        profile_patch::claude::validate_fields(
            &json!({"default_fable_model_name":"Fable","auth_mode":"api_key"})
        )
        .is_ok()
    );
    assert!(profile_patch::codex::validate_fields(&json!({"auth_mode":"provider_bearer_token","auth_token":"token","model_catalog_json":"catalog.json","requires_openai_auth":false})).is_ok());
    assert!(profile_patch::grok::validate_fields(&json!({"credential_action":"replace_api_key","api_key":"token","context_window":500000})).is_ok());
    assert!(
        profile_patch::grok::validate_fields(&json!({"auth_token":"ignored-legacy-field"}))
            .is_err()
    );
    // This stricter native gate must not change existing Tauri unknown-field behavior.
    assert!(profile_patch::claude::build(&json!({"legacy_unknown":true})).is_ok());
}

/// Evidence tool, invoked directly from its prebuilt test executable while CU runs.
/// It cannot target arbitrary user catalogs, even if supplied a misleading path.
#[test]
#[ignore = "explicit ownership-marked native fixture second-client mutation only"]
fn mutate_native_profile_fixture() {
    fn required_path(key: &str) -> std::path::PathBuf {
        let value = std::env::var_os(key).expect("explicit fixture path required");
        let path = std::path::PathBuf::from(value);
        assert!(path.is_absolute(), "fixture paths must be absolute");
        path
    }
    let root = required_path("CCR_APP_PROFILE_MUTATION_ROOT")
        .canonicalize()
        .expect("fixture root must exist");
    let permitted = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../outputs/desktop/profiles")
        .canonicalize()
        .expect("native evidence root must exist");
    assert!(
        root.starts_with(permitted),
        "fixture root outside evidence tree"
    );
    let marker = root.join(".ccr-gpui-profile-fixture");
    assert_eq!(
        fs::read_to_string(marker).expect("fixture ownership marker required"),
        "ccr-gpui-profiles-fixture-v1",
        "unexpected fixture owner"
    );
    let profiles = required_path("CCR_APP_PROFILE_MUTATION_PROFILES");
    assert_eq!(
        profiles.file_name().and_then(|name| name.to_str()),
        Some("profiles.toml")
    );
    assert!(
        profiles
            .parent()
            .unwrap()
            .canonicalize()
            .unwrap()
            .starts_with(&root)
    );
    let manager = ConfigManager::new(&profiles);
    assert!(
        manager
            .profiles_db()
            .canonicalize()
            .unwrap()
            .starts_with(&root)
    );
    // Exact native ConfigSource fixture lock directory: cooperating resource locks.
    let locks = profiles
        .parent()
        .unwrap()
        .join("desktop-profile-fixture/locks");
    let mut ancestor = locks.as_path();
    while !ancestor.exists() {
        ancestor = ancestor.parent().expect("fixture lock ancestor required");
    }
    assert!(ancestor.canonicalize().unwrap().starts_with(&root));
    let name =
        std::env::var("CCR_APP_PROFILE_MUTATION_NAME").expect("fixture profile name required");
    assert!(!name.trim().is_empty());
    LockManager::with_scoped_directory(&locks, || {
        let before = manager.snapshot()?;
        manager.mutate_versioned(Some(&before.version), false, |config| {
            config.get_section_mut(&name)?.description =
                Some("External native CAS fixture update".into());
            Ok(())
        })?;
        let after = manager.snapshot()?;
        assert_ne!(
            before.version, after.version,
            "fixture description must actually change"
        );
        println!(
            "{}",
            json!({
                "status": "updated", "name": name,
                "before_version": before.version, "after_version": after.version,
            })
        );
        Ok(())
    })
    .unwrap_or_else(|_| panic!("native fixture mutation failed"));
}
