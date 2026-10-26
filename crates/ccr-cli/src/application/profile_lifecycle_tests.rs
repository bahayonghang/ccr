#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::*;
use ccr_core::core::write_journal::{self, fault};
use std::collections::BTreeMap;
use std::sync::{Arc, Barrier};

const SENTINEL: &str = "synthetic-gpui04-secret-never-render";

fn paths(root: &Path, platform: Platform) -> PlatformPaths {
    let platform_dir = root.join("platforms").join(platform.to_string());
    PlatformPaths {
        root: root.to_path_buf(),
        registry_file: root.join("config.toml"),
        profiles_file: platform_dir.join("profiles.toml"),
        settings_file: platform_dir.join("settings.json"),
        history_file: root.join("history").join(format!("{platform}.json")),
        backups_dir: root.join("backups").join(platform.to_string()),
        platform_dir,
    }
}

fn context(root: &Path, platform: Platform) -> ProfileLifecycleContext {
    let paths = paths(root, platform);
    let locks = root.join("locks");
    let history = root.join("history.db");
    let runtime = root.join("runtime").join(platform.to_string());
    match platform {
        Platform::Claude => ProfileLifecycleContext::for_claude(
            paths,
            ccr_config::ClaudeRuntimePaths {
                settings_file: runtime.join("settings.json"),
                credentials_file: runtime.join(".credentials.json"),
                state_file: root.join("claude-state.json"),
                backups_dir: runtime.join("backups"),
                config_dir: runtime,
            },
            locks,
            history,
        )
        .unwrap(),
        Platform::Codex => {
            ProfileLifecycleContext::for_codex(paths, runtime, locks, history).unwrap()
        }
        Platform::Grok => {
            ProfileLifecycleContext::for_grok(paths, runtime.join("config.toml"), locks, history)
                .unwrap()
        }
        _ => unreachable!(),
    }
}

fn seed(context: &ProfileLifecycleContext) {
    context
        .run(|| {
            for name in ["old", "new"] {
                let mut profile = ProfileConfig::new()
                    .with_base_url("https://fixture.invalid".into())
                    .with_auth_token(SENTINEL.into())
                    .with_model(format!("model-{name}"));
                profile.provider_type = Some("third_party_model".into());
                profile
                    .platform_data
                    .insert("fixture_unknown".into(), serde_json::json!({"keep":[1,2]}));
                context.instance().save_profile(name, &profile)?;
            }
            context.manager().mutate(|catalog| {
                catalog.sections.get_mut("old").unwrap().other.insert(
                    "fixture_datetime".into(),
                    toml::Value::Datetime("2026-10-18T12:00:00Z".parse().unwrap()),
                );
                Ok(())
            })
        })
        .unwrap();
}

fn snapshot(context: &ProfileLifecycleContext) -> BTreeMap<PathBuf, Option<Vec<u8>>> {
    operation_paths_with(context.paths(), context.platform(), context.instance())
        .unwrap()
        .into_iter()
        .map(|path| {
            let bytes = std::fs::read(&path).ok();
            (path, bytes)
        })
        .collect()
}

#[test]
fn injected_constructors_and_missing_reads_create_no_files() {
    let fixture = tempfile::tempdir().unwrap();
    for platform in [Platform::Claude, Platform::Codex, Platform::Grok] {
        let root = fixture.path().join(platform.to_string());
        let context = context(&root, platform);
        assert!(!root.exists());
        context
            .run(|| {
                assert!(context.instance().load_profiles()?.is_empty());
                assert_eq!(context.instance().get_current_profile()?, None);
                Ok(())
            })
            .unwrap();
        assert!(!root.exists());
    }
}

#[test]
fn injected_edit_cas_apply_rename_replay_and_off_use_existing_owners() {
    for platform in [Platform::Claude, Platform::Codex, Platform::Grok] {
        let fixture = tempfile::tempdir().unwrap();
        let context = context(fixture.path(), platform);
        seed(&context);
        let draft = context.manager().snapshot().unwrap().version;
        let edited = context
            .update("old", "old", Some(&draft), |profile| {
                profile.description = Some("edited".into());
                Ok(())
            })
            .unwrap();
        assert!(!edited.activation_committed);
        assert_eq!(
            context.instance().load_profiles().unwrap()["old"]
                .description
                .as_deref(),
            Some("edited")
        );
        let stale = context.update("old", "old", Some(&draft), |profile| {
            profile.description = Some("stale".into());
            Ok(())
        });
        assert!(
            matches!(stale,Err(CcrError::ValidationError(ref message)) if message == "配置版本冲突，请重新读取后重试")
        );
        let apply = ApplyProfileRequest::new(platform, "old");
        let outcome = context.apply(apply.clone()).unwrap();
        assert!(outcome.activation_committed, "{platform}: {outcome:?}");
        assert_eq!(context.apply(apply).unwrap(), outcome);
        assert!(!format!("{outcome:?}").contains(SENTINEL));
        assert_eq!(
            context.instance().get_current_profile().unwrap().as_deref(),
            Some("old")
        );
        let current = context.manager().snapshot().unwrap().version;
        let rename = context
            .update("old", "renamed", Some(&current), |_| Ok(()))
            .unwrap();
        let persisted_name = if platform == Platform::Grok {
            // The pre-existing generic lifecycle cannot delete Grok's active
            // source. Its dedicated staged adapter owns rename recovery states.
            assert_eq!(rename.status, ProfileStatus::Unchanged);
            assert!(!rename.activation_committed);
            "old"
        } else {
            assert!(rename.activation_committed, "{platform}: {rename:?}");
            "renamed"
        };
        assert_eq!(
            context.instance().get_current_profile().unwrap().as_deref(),
            Some(persisted_name)
        );
        let catalog = context.manager().load().unwrap();
        assert_eq!(
            catalog.sections.contains_key("old"),
            platform == Platform::Grok
        );
        assert_eq!(
            catalog.sections[persisted_name].other["fixture_datetime"]
                .as_datetime()
                .unwrap()
                .to_string(),
            "2026-10-18T12:00:00Z"
        );
        assert!(
            catalog.sections[persisted_name]
                .other
                .contains_key("fixture_unknown")
        );
        assert!(context.needs_login_prep().unwrap());
        let off = context.off().unwrap();
        assert!(off.changed);
        assert_eq!(context.instance().get_current_profile().unwrap(), None);
        assert!(!context.needs_login_prep().unwrap());
        assert!(context.history_file.exists());
        assert!(
            context
                .paths
                .platform_dir
                .join("profile-operations")
                .is_dir()
        );
    }
}

#[test]
fn two_independent_injected_clients_one_draft_exactly_one_save_commits() {
    for platform in [Platform::Claude, Platform::Codex, Platform::Grok] {
        let fixture = tempfile::tempdir().unwrap();
        let initial = context(fixture.path(), platform);
        seed(&initial);
        let revision = initial.manager().snapshot().unwrap().version;
        let barrier = Arc::new(Barrier::new(2));
        let workers = ["client-a", "client-b"]
            .into_iter()
            .map(|description| {
                let client = context(fixture.path(), platform);
                let revision = revision.clone();
                let barrier = barrier.clone();
                std::thread::spawn(move || {
                    barrier.wait();
                    client.update("old", "old", Some(&revision), |profile| {
                        profile.description = Some(description.into());
                        Ok(())
                    })
                })
            })
            .collect::<Vec<_>>();
        let results = workers
            .into_iter()
            .map(|worker| worker.join().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
        assert_eq!(results.iter().filter(|result|matches!(result,Err(CcrError::ValidationError(message)) if message=="配置版本冲突，请重新读取后重试")).count(),1);
        let catalog = initial.manager().load().unwrap();
        assert!(matches!(
            catalog.sections["old"].description.as_deref(),
            Some("client-a" | "client-b")
        ));
        assert!(
            catalog.sections["old"]
                .other
                .contains_key("fixture_unknown")
        );
        assert!(catalog.sections["old"].other["fixture_datetime"].is_datetime());
    }
}

#[test]
fn injected_apply_write_failure_restores_catalog_runtime_and_markers() {
    for platform in [Platform::Claude, Platform::Codex, Platform::Grok] {
        let fixture = tempfile::tempdir().unwrap();
        let context = context(fixture.path(), platform);
        seed(&context);
        assert!(
            context
                .apply(ApplyProfileRequest::new(platform, "old"))
                .unwrap()
                .activation_committed
        );
        let before = snapshot(&context);
        let revision = context.manager().snapshot().unwrap().version;
        let _fault = fault::install(|path| {
            if write_journal::contains(path) {
                Err(CcrError::FileIoError("synthetic injected failure".into()))
            } else {
                Ok(())
            }
        });
        let outcome = context
            .apply(ApplyProfileRequest::new(platform, "new"))
            .unwrap();
        assert_eq!(
            outcome.status,
            ProfileStatus::Unchanged,
            "{platform}: {outcome:?}"
        );
        assert!(!outcome.activation_committed);
        assert_eq!(snapshot(&context), before);
        assert_eq!(context.manager().snapshot().unwrap().version, revision);
        assert!(!format!("{outcome:?}").contains(SENTINEL));
    }
}

#[test]
fn injected_wrong_platform_rejects_before_any_file_access() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path().join("absent");
    let context = context(&root, Platform::Claude);
    assert!(
        context
            .apply(ApplyProfileRequest::new(Platform::Codex, "missing"))
            .is_err()
    );
    assert!(!root.exists());
}

#[test]
fn injected_invalid_patch_and_backup_failure_leave_state_unchanged() {
    for platform in [Platform::Claude, Platform::Codex, Platform::Grok] {
        let fixture = tempfile::tempdir().unwrap();
        let context = context(fixture.path(), platform);
        seed(&context);
        let before = context.manager().canonical_toml().unwrap();
        let revision = context.manager().snapshot().unwrap().version;
        assert!(
            context
                .update("old", "old", Some(&revision), |_| Err(
                    CcrError::ValidationError("Invalid fixture patch".into())
                ))
                .is_err()
        );
        assert_eq!(context.manager().canonical_toml().unwrap(), before);
        assert!(
            context
                .apply(ApplyProfileRequest::new(platform, "old"))
                .unwrap()
                .activation_committed
        );
        if platform == Platform::Grok {
            continue;
        } // Grok's runtime owner has its own failure matrix.
        let before_runtime = snapshot(&context);
        let before_catalog = context.manager().canonical_toml().unwrap();
        let backup_root = fixture.path().join("backups/profile-off");
        std::fs::create_dir_all(backup_root.parent().unwrap()).unwrap();
        std::fs::write(&backup_root, b"fixture blocker").unwrap();
        assert!(context.off().is_err());
        assert_eq!(snapshot(&context), before_runtime);
        assert_eq!(context.manager().canonical_toml().unwrap(), before_catalog);
        assert_eq!(
            context.instance().get_current_profile().unwrap().as_deref(),
            Some("old")
        );
    }
}

#[test]
fn injected_create_preserves_platform_normalization_and_rejects_collisions_and_stale_drafts() {
    for platform in [Platform::Claude, Platform::Codex, Platform::Grok] {
        let fixture = tempfile::tempdir().unwrap();
        let context = context(fixture.path(), platform);
        let mut profile = ProfileConfig::new()
            .with_base_url("https://fixture.invalid".into())
            .with_auth_token(SENTINEL.into())
            .with_model("fixture-model".into());
        if platform == Platform::Codex {
            profile
                .platform_data
                .insert("auth_mode".into(), serde_json::json!("openai_api_key"));
        }
        assert!(
            context
                .create("settings", profile.clone(), Some(""))
                .is_err()
        );
        assert!(!context.manager().profiles_db().exists());
        context
            .create("created", profile.clone(), Some(""))
            .unwrap();
        let revision = context.manager().snapshot().unwrap().version;
        assert!(
            context
                .create("created", profile.clone(), Some(&revision))
                .is_err()
        );
        assert!(context.create("second", profile.clone(), Some("")).is_err());
        context.create("second", profile, Some(&revision)).unwrap();
        assert_eq!(context.instance().load_profiles().unwrap().len(), 2);
        if platform == Platform::Codex {
            assert!(
                context
                    .export_profile_env("created", false)
                    .unwrap()
                    .contains(SENTINEL)
            );
            assert!(
                context
                    .export_profile_env("created", true)
                    .unwrap()
                    .contains(SENTINEL)
            );
        }
    }
}

#[test]
fn grok_staged_rename_preserves_real_apply_and_cleanup_failure_states() {
    use crate::platforms::grok::{GrokActivationState, GrokProfileUpdateOutcome};
    for cleanup_failure in [false, true] {
        let fixture = tempfile::tempdir().unwrap();
        let context = context(fixture.path(), Platform::Grok);
        seed(&context);
        assert!(
            context
                .apply(ApplyProfileRequest::new(Platform::Grok, "old"))
                .unwrap()
                .activation_committed
        );
        let revision = context.manager().snapshot().unwrap().version;
        let platform = context.grok_platform().unwrap();
        let runtime = platform.get_settings_path();
        let entry = context
            .paths()
            .platform_dir
            .join("profile_entry_config_state.json");
        let _before = if cleanup_failure {
            None
        } else {
            Some(fault::install(move |path| {
                if path == runtime {
                    Err(CcrError::FileIoError("fixture apply failure".into()))
                } else {
                    Ok(())
                }
            }))
        };
        let runtime = platform.get_settings_path();
        let _after = if cleanup_failure {
            Some(fault::install_after_publish(move |path| {
                if path == runtime {
                    std::fs::remove_file(&entry)?;
                }
                Ok(())
            }))
        } else {
            None
        };
        let outcome = context
            .run(|| {
                platform.update_profile_versioned("old", "renamed", Some(&revision), |_| Ok(()))
            })
            .unwrap();
        let profiles = platform.load_profiles().unwrap();
        assert!(profiles.contains_key("old"));
        assert!(profiles.contains_key("renamed"));
        if cleanup_failure {
            assert!(matches!(
                outcome,
                GrokProfileUpdateOutcome::RenameCleanupFailed { .. }
            ));
            assert_eq!(context.manager().load().unwrap().current_config, "renamed");
            assert!(matches!(
                platform.inspect_activation_state().unwrap(),
                GrokActivationState::UnsafeMissingEntryState { .. }
            ));
        } else {
            assert!(matches!(
                outcome,
                GrokProfileUpdateOutcome::RenameApplyFailed { .. }
            ));
            assert_eq!(
                platform.get_current_profile().unwrap().as_deref(),
                Some("old")
            );
        }
    }
}

#[test]
fn grok_staged_same_revision_clients_and_successful_active_rename() {
    use crate::platforms::grok::GrokProfileUpdateOutcome;
    let fixture = tempfile::tempdir().unwrap();
    let initial = context(fixture.path(), Platform::Grok);
    seed(&initial);
    initial
        .apply(ApplyProfileRequest::new(Platform::Grok, "old"))
        .unwrap();
    let revision = initial.manager().snapshot().unwrap().version;
    let barrier = Arc::new(Barrier::new(2));
    let workers = ["a", "b"]
        .into_iter()
        .map(|description| {
            let client = context(fixture.path(), Platform::Grok);
            let revision = revision.clone();
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                client.run(|| {
                    barrier.wait();
                    client.grok_platform()?.update_profile_versioned(
                        "old",
                        "old",
                        Some(&revision),
                        |profile| {
                            profile.description = Some(description.into());
                            Ok(())
                        },
                    )
                })
            })
        })
        .collect::<Vec<_>>();
    let results = workers
        .into_iter()
        .map(|worker| worker.join().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    assert_eq!(results.iter().filter(|result|matches!(result,Err(CcrError::ValidationError(message)) if message=="配置版本冲突，请重新读取后重试")).count(),1);
    // Same-name edits intentionally do not reapply. Restore a genuinely Active
    // baseline before testing the active-rename branch (Drifted remains distinct).
    initial
        .apply(ApplyProfileRequest::new(Platform::Grok, "old"))
        .unwrap();
    let revision = initial.manager().snapshot().unwrap().version;
    let outcome = initial
        .run(|| {
            initial.grok_platform()?.update_profile_versioned(
                "old",
                "renamed",
                Some(&revision),
                |profile| {
                    profile.description = None;
                    Ok(())
                },
            )
        })
        .unwrap();
    assert!(matches!(outcome, GrokProfileUpdateOutcome::Renamed { .. }));
    assert_eq!(
        initial.instance().get_current_profile().unwrap().as_deref(),
        Some("renamed")
    );
    assert!(
        !initial
            .instance()
            .load_profiles()
            .unwrap()
            .contains_key("old")
    );
}

#[test]
fn grok_direct_apply_and_staged_edit_share_operation_before_catalog_order() {
    let fixture = tempfile::tempdir().unwrap();
    let initial = context(fixture.path(), Platform::Grok);
    seed(&initial);
    initial
        .apply(ApplyProfileRequest::new(Platform::Grok, "old"))
        .unwrap();
    let (ready_send, ready_receive) = std::sync::mpsc::channel();
    let (release_send, release_receive) = std::sync::mpsc::channel();
    let direct = context(fixture.path(), Platform::Grok);
    let first = std::thread::spawn(move || {
        direct.run(|| {
            let runtime = direct.instance().get_settings_path();
            let mut sent = false;
            let _fault = fault::install(move |path| {
                if path == runtime && !sent {
                    sent = true;
                    ready_send.send(()).unwrap();
                    release_receive
                        .recv_timeout(Duration::from_secs(5))
                        .unwrap();
                }
                Ok(())
            });
            direct.instance().apply_profile("new")
        })
    });
    ready_receive.recv_timeout(Duration::from_secs(5)).unwrap();
    let applying = context(fixture.path(), Platform::Grok);
    let second =
        std::thread::spawn(move || applying.apply(ApplyProfileRequest::new(Platform::Grok, "old")));
    let staged = context(fixture.path(), Platform::Grok);
    let third = std::thread::spawn(move || {
        staged.run(|| {
            staged
                .grok_platform()?
                .update_profile_versioned("new", "new", None, |profile| {
                    profile.description = Some("parallel staged".into());
                    Ok(())
                })
        })
    });
    // Give both contenders an opportunity to acquire their first lock while the
    // direct owner's Grok guard is held. This is an ordering regression, not perf.
    std::thread::sleep(Duration::from_millis(100));
    release_send.send(()).unwrap();
    first.join().unwrap().unwrap();
    assert!(second.join().unwrap().unwrap().activation_committed);
    assert!(third.join().unwrap().is_ok());
    assert_eq!(
        initial.instance().load_profiles().unwrap()["new"]
            .description
            .as_deref(),
        Some("parallel staged")
    );
}

#[test]
fn grok_guarded_delete_preserves_active_drifted_and_unsafe_force_policy() {
    use crate::platforms::grok::{GrokProfileDeleteBlockedReason, GrokProfileDeleteOutcome};
    for state in ["active", "drifted", "unsafe"] {
        let fixture = tempfile::tempdir().unwrap();
        let context = context(fixture.path(), Platform::Grok);
        seed(&context);
        context
            .apply(ApplyProfileRequest::new(Platform::Grok, "old"))
            .unwrap();
        let platform = context.grok_platform().unwrap();
        if state == "unsafe" {
            std::fs::remove_file(
                context
                    .paths()
                    .platform_dir
                    .join("profile_entry_config_state.json"),
            )
            .unwrap();
        }
        if state == "drifted" {
            let runtime = platform.get_settings_path();
            let text = std::fs::read_to_string(&runtime).unwrap();
            assert!(text.contains("model-old"));
            std::fs::write(
                runtime,
                text.replace("model-old", "externally-changed-model"),
            )
            .unwrap();
        }
        let before = snapshot(&context);
        let catalog = context.manager().canonical_toml().unwrap();
        let expected = match state {
            "active" => GrokProfileDeleteBlockedReason::Active,
            "drifted" => GrokProfileDeleteBlockedReason::Drifted,
            _ => GrokProfileDeleteBlockedReason::UnsafeMissingEntryState,
        };
        let blocked = context
            .run(|| platform.delete_profile_guarded("old", false))
            .unwrap();
        assert_eq!(
            blocked,
            GrokProfileDeleteOutcome::Blocked { reason: expected }
        );
        assert_eq!(snapshot(&context), before);
        assert_eq!(context.manager().canonical_toml().unwrap(), catalog);
        let forced = context
            .run(|| platform.delete_profile_guarded("old", true))
            .unwrap();
        if state == "unsafe" {
            assert_eq!(
                forced,
                GrokProfileDeleteOutcome::Blocked { reason: expected }
            );
            assert_eq!(snapshot(&context), before);
            assert_eq!(context.manager().canonical_toml().unwrap(), catalog);
        } else {
            assert_eq!(
                forced,
                GrokProfileDeleteOutcome::Deleted { name: "old".into() }
            );
            assert!(!platform.load_profiles().unwrap().contains_key("old"));
            assert_eq!(platform.get_current_profile().unwrap(), None);
        }
    }
}

#[test]
fn injected_catalog_commit_failure_restores_secret_store_and_catalog() {
    for platform in [Platform::Claude, Platform::Codex, Platform::Grok] {
        let fixture = tempfile::tempdir().unwrap();
        let context = context(fixture.path(), platform);
        seed(&context);
        let catalog = context.manager().canonical_toml().unwrap();
        let before = snapshot(&context);
        let mut profile = ProfileConfig::new()
            .with_base_url("https://fixture.invalid".into())
            .with_auth_token(SENTINEL.into())
            .with_model("fixture-model".into());
        if platform == Platform::Codex {
            profile
                .platform_data
                .insert("auth_mode".into(), serde_json::json!("openai_api_key"));
        }
        ccr_config::fail_next_catalog_commit();
        assert!(context.create("commit-failure", profile, None).is_err());
        assert_eq!(context.manager().canonical_toml().unwrap(), catalog);
        assert_eq!(snapshot(&context), before);
        assert!(
            !context
                .instance()
                .load_profiles()
                .unwrap()
                .contains_key("commit-failure")
        );
    }
}

#[test]
fn injected_history_preparation_matches_existing_success_warning_and_replay_semantics() {
    for blocked in [false, true] {
        let fixture = tempfile::tempdir().unwrap();
        let mut context = context(fixture.path(), Platform::Claude);
        seed(&context);
        let history_parent = fixture.path().join("explicit-history-parent");
        if blocked {
            std::fs::write(&history_parent, b"fixture blocker").unwrap();
        }
        context.history_file = history_parent.join("data.db");
        let request = ApplyProfileRequest::new(Platform::Claude, "new");
        let outcome = context.apply(request.clone()).unwrap();
        assert!(outcome.activation_committed);
        assert_eq!(
            outcome.warnings.contains(&ProfileWarning::HistoryFailed),
            blocked
        );
        assert_eq!(context.apply(request).unwrap(), outcome);
        assert_eq!(context.history_file.is_file(), !blocked);
        assert_eq!(
            context.instance().get_current_profile().unwrap().as_deref(),
            Some("new")
        );
    }
}

#[test]
fn grok_staged_save_rechecks_original_revision_at_commit() {
    let fixture = tempfile::tempdir().unwrap();
    let context = context(fixture.path(), Platform::Grok);
    seed(&context);
    let revision = context.manager().snapshot().unwrap().version;
    let result = context.run(|| {
        context.grok_platform()?.update_profile_versioned(
            "old",
            "old",
            Some(&revision),
            |profile| {
                // A nested owner mutation represents an intervening catalog
                // version after the editor baseline was checked.
                context.manager().mutate(|catalog| {
                    catalog.sections.get_mut("old").unwrap().description =
                        Some("intervening writer".into());
                    Ok(())
                })?;
                profile.description = Some("stale editor".into());
                Ok(())
            },
        )
    });
    assert!(
        matches!(result, Err(CcrError::ValidationError(message)) if message == "配置版本冲突，请重新读取后重试")
    );
    assert_eq!(
        context.manager().load().unwrap().sections["old"]
            .description
            .as_deref(),
        Some("intervening writer")
    );
}

#[test]
fn grok_staged_rename_retains_exact_unknown_toml_values() {
    let fixture = tempfile::tempdir().unwrap();
    let context = context(fixture.path(), Platform::Grok);
    seed(&context);
    let revision = context.manager().snapshot().unwrap().version;
    context
        .run(|| {
            context.grok_platform()?.update_profile_versioned(
                "old",
                "renamed",
                Some(&revision),
                |_| Ok(()),
            )
        })
        .unwrap();
    let catalog = context.manager().load().unwrap();
    assert!(catalog.sections["renamed"].other["fixture_datetime"].is_datetime());
}

#[test]
fn injected_codex_off_registry_failure_retains_entry_snapshot() {
    let fixture = tempfile::tempdir().unwrap();
    let context = context(fixture.path(), Platform::Codex);
    seed(&context);
    context
        .apply(ApplyProfileRequest::new(Platform::Codex, "old"))
        .unwrap();
    let entry = context
        .paths
        .platform_dir
        .join("profile_entry_auth_state.json");
    std::fs::write(&entry, b"synthetic legacy entry snapshot").unwrap();
    let before = snapshot(&context);
    let catalog = context.manager().canonical_toml().unwrap();
    let registry = context.paths.registry_file.clone();
    let mut failed = false;
    let _fault = fault::install(move |path| {
        if path == registry && !failed {
            failed = true;
            return Err(CcrError::FileIoError(
                "synthetic off registry failure".into(),
            ));
        }
        Ok(())
    });
    assert!(context.off().is_err());
    assert_eq!(snapshot(&context), before);
    assert_eq!(context.manager().canonical_toml().unwrap(), catalog);
}

#[test]
fn injected_first_create_external_change_does_not_publish_orphan_catalog() {
    let fixture = tempfile::tempdir().unwrap();
    let context = context(fixture.path(), Platform::Codex);
    let secrets = context.paths.platform_dir.join("profile_secrets.json");
    let runtime = context.instance().get_settings_path();
    let _fault = fault::install_after_publish(move |path| {
        if path == secrets {
            std::fs::create_dir_all(runtime.parent().unwrap())?;
            std::fs::write(&runtime, b"external runtime version")?;
        }
        Ok(())
    });
    let mut profile = ProfileConfig::new()
        .with_base_url("https://fixture.invalid".into())
        .with_auth_token(SENTINEL.into())
        .with_model("fixture-model".into());
    profile
        .platform_data
        .insert("auth_mode".into(), serde_json::json!("openai_api_key"));
    assert!(context.create("created", profile, Some("")).is_err());
    assert!(!context.manager().profiles_db().exists());
}

#[test]
fn injected_first_create_markers_match_platform_and_existing_inactive_stays_inactive() {
    for platform in [Platform::Claude, Platform::Codex, Platform::Grok] {
        let fixture = tempfile::tempdir().unwrap();
        let direct = context(&fixture.path().join("direct"), platform);
        let created = context(&fixture.path().join("created"), platform);
        let inactive = context(&fixture.path().join("inactive"), platform);
        let profile = ProfileConfig::new()
            .with_base_url("https://fixture.invalid".into())
            .with_auth_token(SENTINEL.into())
            .with_model("fixture-model".into());
        direct
            .run(|| direct.instance().save_profile("first", &profile))
            .unwrap();
        created.create("first", profile.clone(), Some("")).unwrap();
        let direct_catalog = direct.manager().load().unwrap();
        let created_catalog = created.manager().load().unwrap();
        assert_eq!(
            created_catalog.current_config,
            direct_catalog.current_config
        );
        assert_eq!(
            created_catalog.default_config,
            direct_catalog.default_config
        );
        inactive
            .run(|| {
                inactive.manager().ensure_initialized()?;
                inactive.manager().mutate(|catalog| {
                    catalog.current_config.clear();
                    Ok(())
                })
            })
            .unwrap();
        let revision = inactive.manager().snapshot().unwrap().version;
        inactive.create("first", profile, Some(&revision)).unwrap();
        assert!(inactive.manager().load().unwrap().current_config.is_empty());
    }
}

#[test]
fn injected_first_create_commit_failure_leaves_no_secret_or_catalog() {
    for platform in [Platform::Claude, Platform::Codex, Platform::Grok] {
        let fixture = tempfile::tempdir().unwrap();
        let context = context(fixture.path(), platform);
        let before = snapshot(&context);
        let profile = ProfileConfig::new()
            .with_base_url("https://fixture.invalid".into())
            .with_auth_token(SENTINEL.into())
            .with_model("fixture-model".into());
        ccr_config::fail_next_catalog_commit();
        assert!(context.create("created", profile, Some("")).is_err());
        assert!(!context.manager().profiles_db().exists());
        assert_eq!(snapshot(&context), before);
    }
}
