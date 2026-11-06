#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::*;
use crate::runtime::{
    ConfigSource, LaunchMode, Services,
    insights::{InsightsQuery, InsightsRequests, InsightsSnapshot},
    platforms,
};
use ccr_app::runtime::{
    Completion, EnvironmentId, Platform, RequestKey, RuntimeOwner, TaskEvent, TaskSubscription,
};
use serde_json::json;
use std::sync::Arc;

fn key(id: u64) -> RequestKey {
    RequestKey {
        environment: EnvironmentId::Local("benchmark-test".into()),
        platform: Platform::Claude,
        generation: 0,
        request_id: id,
    }
}
async fn result<T: Send + Sync + 'static, E: Send + Sync + 'static>(
    mut subscription: TaskSubscription<T, E>,
) -> Arc<Completion<T, E>> {
    while let Some(event) = subscription.next().await {
        if let TaskEvent::Terminal(result) = event {
            return result;
        }
    }
    panic!("missing terminal");
}
fn hash_json(value: &Value) -> String {
    logical_hash(value).unwrap()
}

/// Tiny real owner-created S0 keeps ordinary tests independent of generated outputs.
fn s0() -> (tempfile::TempDir, PathBuf) {
    let pair = tempfile::tempdir().unwrap();
    let root = pair.path().join("gpui");
    for name in [
        "home",
        "ui",
        "ccr/analytics",
        "logs",
        "locks",
        "preferences",
        "crypto",
        "project",
    ] {
        fs::create_dir_all(root.join(name)).unwrap();
    }
    drop(
        ccr_db::database::create_app_pool_at(&root.join("ui/ccr-ui.db"), &root.join("home"))
            .unwrap(),
    );
    drop(
        ccr_db::database::create_usage_archive_pool_at(
            &root.join("ccr/analytics/usage.db"),
            &root.join("home"),
            &root.join("ui/ccr-ui.db"),
        )
        .unwrap(),
    );
    ccr_usage::fixtures::create_projection_db(&root.join("llmusage"));
    let root = root.canonicalize().unwrap();
    let logical = json!({"source":"synthetic-unit-s0"});
    let hash = hash_json(&logical);
    let artifacts = json!({"catalog":null,"main":file_hash(&root.join("ui/ccr-ui.db")).unwrap(),"usage":file_hash(&root.join("llmusage/llmusage.db")).unwrap(),"archive":file_hash(&root.join("ccr/analytics/usage.db")).unwrap()});
    fs::create_dir(pair.path().join("tauri")).unwrap();
    let end =
        json!({"root":root,"logical":logical,"logical_sha256":hash,"artifact_sha256":artifacts});
    let manifest = json!({"version":1,"synthetic":true,"scenario":"S0","comparison":"not_comparable","report_timezone":"UTC","as_of_utc":"2026-11-06T12:00:00Z","counts":{"claude_profiles":0,"codex_sessions":0,"usage_events":0},"logical_sha256":hash,"ends":{"gpui":end,"tauri":{"root":pair.path().join("tauri").canonicalize().unwrap(),"logical":logical,"logical_sha256":hash}}});
    fs::write(
        pair.path().join("logical-manifest.json"),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
    fs::write(
        root.join("ccr-tauri-fixture.json"),
        br#"{"version":1,"fixture_id":"benchmark-s0"}"#,
    )
    .unwrap();
    (pair, root)
}

#[test]
fn benchmark_arguments_are_explicit_exclusive_and_do_not_load_files() {
    let root = std::env::current_dir()
        .unwrap()
        .join("missing-benchmark-parser-test");
    assert!(!root.exists());
    let parse = |args: Vec<std::ffi::OsString>| LaunchMode::parse(args);
    assert!(
        matches!(parse(vec!["--benchmark-fixture-root".into(),root.clone().into()]),Ok(LaunchMode::Benchmark(path)) if path==root)
    );
    for flag in [
        "--probe",
        "--fixture-profiles",
        "--fixture-registry",
        "--fixture-read-delay-ms",
        "--insights-usage-root",
        "--insights-archive-db",
        "--insights-provider-home",
        "--benchmark-fixture-root",
    ] {
        let mut args = vec![
            "--benchmark-fixture-root".into(),
            root.clone().into(),
            flag.into(),
        ];
        if flag != "--probe" {
            args.push(if flag == "--fixture-read-delay-ms" {
                "1".into()
            } else {
                root.clone().into()
            });
        }
        assert!(parse(args).is_err(), "{flag}");
    }
    assert!(parse(vec!["--benchmark-fixture-root".into(), "relative".into()]).is_err());
    assert!(
        parse(vec![
            "--benchmark-fixture-root".into(),
            root.join("../escape").into()
        ])
        .is_err()
    );
    assert!(parse(vec!["--benchmark-fixture-root".into()]).is_err());
    assert!(matches!(
        parse(vec![]),
        Ok(LaunchMode::App(ConfigSource::Local))
    ));
    assert!(matches!(
        parse(vec![
            "--fixture-profiles".into(),
            root.join("profiles.toml").into(),
            "--fixture-registry".into(),
            root.join("registry.toml").into()
        ]),
        Ok(LaunchMode::App(ConfigSource::Fixture { .. }))
    ));
    assert!(!root.exists());
}

#[test]
fn captured_layout_refuses_changed_inputs_and_cross_root_paths() {
    let (_pair, root) = s0();
    let layout = Layout::load(&root).unwrap();
    assert!(layout.check_path(&root.join("project/new-file")).is_ok());
    let sibling = root.parent().unwrap().join("outside");
    fs::create_dir(&sibling).unwrap();
    assert!(layout.check_path(&sibling.join("new-file")).is_err());
    assert!(
        layout
            .check_path(&PathBuf::from(format!(
                "{}/project/../new-file",
                root.display()
            )))
            .is_err()
    );
    let paths = layout.platform_paths(ccr_config::Platform::Claude).unwrap();
    assert_eq!(
        paths.profiles_db(),
        root.join("ccr/platforms/claude/profiles.db")
    );
    assert_eq!(layout.roots.locks, root.join("locks"));
    assert_eq!(layout.as_of.to_rfc3339(), "2026-11-06T12:00:00+00:00");
    fs::write(root.join("ui/ccr-ui.db"), b"changed seed").unwrap();
    assert!(matches!(
        Layout::load(&root),
        Err("Benchmark seed input changed; restore the original trial data")
    ));
}

#[test]
fn poisoned_ambient_process_is_ignored_by_benchmark_domains() {
    let (_pair, root) = s0();
    let trap = root.parent().unwrap().join("host-trap");
    fs::write(&trap, b"not a directory").unwrap();
    let status = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "runtime::benchmark::tests::poisoned_ambient_child",
            "--ignored",
            "--nocapture",
            "--skip",
            "export_bindings",
        ])
        .env("CCR_NATIVE_BENCHMARK_CHILD", &root)
        .env("CCR_ROOT", &trap)
        .env("CCR_DATA_DIR", &trap)
        .env("CCR_LOCK_DIR", &trap)
        .env("CODEX_HOME", &trap)
        .env("GROK_HOME", &trap)
        .env("CLAUDE_CONFIG_DIR", &trap)
        .env("ANTHROPIC_AUTH_TOKEN", "synthetic-host-only-token")
        .env("ANTHROPIC_API_KEY", "synthetic-host-only-key")
        .status()
        .unwrap();
    assert!(status.success());
    assert_eq!(fs::read(&trap).unwrap(), b"not a directory");
}

#[test]
fn benchmark_oauth_rejects_owned_and_foreign_sessions_before_claim_or_effects() {
    let (_pair, root) = s0();
    let owner = RuntimeOwner::new().unwrap();
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async {
        let services = Services::new(
            owner.handle(),
            ConfigSource::Benchmark(Arc::new(Layout::load(&root).unwrap())),
        );
        let mut request_key = key(70);
        request_key.platform = Platform::Codex;
        let reply = result(
            services
                .platform_request(
                    request_key.clone(),
                    platforms::Command::Read(platforms::Surface::Auth),
                )
                .unwrap(),
        )
        .await;
        let Completion::Success(platforms::ResultValue::Loaded(session)) = reply.as_ref() else {
            panic!("captured Codex auth read failed");
        };
        let owned = session.clone();
        drop(reply);
        // A preclaimed owner makes the old implementation fail safely with Full;
        // the regression never starts a listener even before the guard is fixed.
        services.platform_oauth.claim().unwrap();
        let denied = services.platform_oauth_start(request_key.clone(), owned.clone());
        assert!(denied.is_ok(), "source refusal must precede OAuth claim");
        assert!(matches!(
            result(denied.unwrap()).await.as_ref(),
            Completion::Failed(platforms::PlatformError::Unsupported)
        ));
        assert!(services.platform_oauth.is_active());
        services.platform_oauth.abandon_claim();
        let denied = services
            .platform_oauth_start(request_key.clone(), owned.clone())
            .unwrap();
        assert!(matches!(
            result(denied).await.as_ref(),
            Completion::Failed(platforms::PlatformError::Unsupported)
        ));
        assert!(!services.platform_oauth.is_active());
        let mut foreign_roots = owned.roots.clone();
        foreign_roots.fixture = false;
        foreign_roots.home = root.parent().unwrap().join("foreign-owned-home");
        fs::create_dir(&foreign_roots.home).unwrap();
        let foreign = platforms::PlatformSession {
            document: ccr_app::platform_settings::Document::new(
                foreign_roots.home.join("config.toml"),
                None,
                ccr_core::core::BackupPolicy::None,
                Some(foreign_roots.locks.clone()),
                ccr_app::platform_settings::Format::Toml,
            ),
            roots: foreign_roots,
            snapshot: owned.snapshot.clone(),
            accounts: Vec::new(),
            signed_in: false,
            provider_ids: Vec::new(),
            provider_report: platforms::ProviderReport::Unsupported,
            surface: platforms::Surface::Auth,
            platform: Platform::Codex,
            file_state: platforms::FileState::Missing,
            file_label: "synthetic",
            fields: Vec::new(),
            credential: platforms::CredentialState::NotDetected,
            runtime_auth_present: None,
            auth_revision: Default::default(),
        };
        let denied = services
            .platform_oauth_start(request_key.clone(), Arc::new(foreign))
            .unwrap();
        assert!(matches!(
            result(denied).await.as_ref(),
            Completion::Failed(platforms::PlatformError::Unsupported)
        ));
        for control in [
            platforms::OAuthControl::Callback {
                login_id: "synthetic".into(),
                url: "synthetic".into(),
            },
            platforms::OAuthControl::Complete {
                login_id: "synthetic".into(),
                name: None,
            },
            platforms::OAuthControl::RetryCleanup,
        ] {
            let denied = services
                .platform_oauth_control(request_key.clone(), control)
                .unwrap();
            assert!(matches!(
                result(denied).await.as_ref(),
                Completion::Failed(platforms::PlatformError::Unsupported)
            ));
        }
        assert!(!services.platform_oauth.is_active());
        assert!(services.platform_oauth.status().is_none());
        assert!(
            fs::read_dir(root.parent().unwrap().join("foreign-owned-home"))
                .unwrap()
                .next()
                .is_none()
        );
        Layout::load(&root).unwrap();
    });
    owner.shutdown();
}

fn tree_bytes(root: &Path) -> Vec<(PathBuf, Option<Vec<u8>>)> {
    fn visit(root: &Path, directory: &Path, rows: &mut Vec<(PathBuf, Option<Vec<u8>>)>) {
        for entry in fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                rows.push((path.strip_prefix(root).unwrap().into(), None));
                visit(root, &path, rows);
            } else {
                rows.push((
                    path.strip_prefix(root).unwrap().into(),
                    Some(fs::read(&path).unwrap()),
                ));
            }
        }
    }
    let mut rows = Vec::new();
    visit(root, root, &mut rows);
    rows.sort();
    rows
}

#[test]
fn benchmark_refuses_foreign_retained_mcp_and_resource_sessions_without_writes() {
    use crate::runtime::{mcp, resources};
    use ccr_app::mcp_document::EntryMutation;
    let (pair, root) = s0();
    let foreign_root = pair.path().join("foreign-owned");
    fs::create_dir(&foreign_root).unwrap();
    let owner = RuntimeOwner::new().unwrap();
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async {
        let services = Services::new(
            owner.handle(),
            ConfigSource::Benchmark(Arc::new(Layout::load(&root).unwrap())),
        );
        let foreign = Services::new(
            owner.handle(),
            ConfigSource::Fixture {
                profiles: foreign_root.join("profiles.toml"),
                registry: foreign_root.join("registry.toml"),
                delay: Default::default(),
            },
        );
        let reply = result(
            foreign
                .mcp_edit_request(key(80), mcp::McpTarget::Codex)
                .unwrap(),
        )
        .await;
        let Completion::Success(edit) = reply.as_ref() else {
            panic!("foreign MCP open failed");
        };
        let edit = edit.clone();
        assert!(matches!(
            result(
                foreign
                    .mcp_mutate_request(
                        key(81),
                        edit.clone(),
                        "synthetic".into(),
                        EntryMutation::Add(json!({"command":"synthetic-tool"}))
                    )
                    .unwrap()
            )
            .await
            .as_ref(),
            Completion::Success(())
        ));
        // Reopen after the seed write so the retained revision is current.
        let reply = result(
            foreign
                .mcp_edit_request(key(82), mcp::McpTarget::Codex)
                .unwrap(),
        )
        .await;
        let Completion::Success(edit) = reply.as_ref() else {
            panic!("foreign MCP reopen failed");
        };
        let edit = edit.clone();
        let entry = mcp::McpEntry {
            source: mcp::McpSource::CodexConfig,
            name: "synthetic".into(),
            scope: mcp::McpScope::User,
            command: Some("synthetic-tool".into()),
            has_url: false,
            has_args: false,
            disabled: None,
            enabled: None,
        };
        let reply = result(
            foreign
                .mcp_copy_prepare_request(key(83), entry, mcp::McpTarget::Antigravity)
                .unwrap(),
        )
        .await;
        let Completion::Success(copy) = reply.as_ref() else {
            panic!("foreign copy prepare failed");
        };
        let copy = copy.clone();
        let reply = result(
            foreign
                .mcp_batch_prepare_request(
                    key(84),
                    mcp::BatchInput::Sync {
                        source: mcp::McpTarget::Codex,
                    },
                    vec![mcp::McpTarget::ClaudeUser],
                    false,
                )
                .unwrap(),
        )
        .await;
        let Completion::Success(batch) = reply.as_ref() else {
            panic!("foreign batch prepare failed");
        };
        let batch = batch.clone();
        let reply = result(
            foreign
                .resource_request(key(85), resources::ResourceCommand::ListPrompts)
                .unwrap(),
        )
        .await;
        let Completion::Success(resources::ResourceResult::Prompts(prompts)) = reply.as_ref()
        else {
            panic!("foreign prompts list failed");
        };
        let id = prompts.first().unwrap().id;
        let reply = result(
            foreign
                .resource_request(key(86), resources::ResourceCommand::OpenPrompt(id))
                .unwrap(),
        )
        .await;
        let Completion::Success(resources::ResourceResult::Opened(prompt)) = reply.as_ref() else {
            panic!("foreign prompt open failed");
        };
        let prompt = prompt.clone();
        let reply = result(
            foreign
                .claude_resource_request(
                    key(87),
                    resources::claude_files::Command::Open {
                        kind: resources::claude_files::Kind::Rules,
                        name: "synthetic.md".into(),
                        creating: true,
                    },
                )
                .unwrap(),
        )
        .await;
        let Completion::Success(resources::claude_files::Outcome::Opened(claude)) = reply.as_ref()
        else {
            panic!("foreign Claude open failed");
        };
        let claude = claude.clone();
        let mut platform_key = key(88);
        platform_key.platform = Platform::OpenCode;
        let reply = result(
            foreign
                .platform_files_read(
                    platform_key.clone(),
                    resources::platform_files::Read::Open {
                        kind: resources::platform_files::Kind::OpenCodeAgent,
                        scope: resources::platform_files::Scope::User,
                        name: "synthetic.md".into(),
                        creating: true,
                    },
                )
                .unwrap(),
        )
        .await;
        let Completion::Success(resources::platform_files::ReadResult::Opened(platform_file)) =
            reply.as_ref()
        else {
            panic!("foreign platform file open failed");
        };
        let platform_file = platform_file.clone();
        let before = tree_bytes(pair.path());
        let mut denied = Vec::new();
        denied.push(matches!(
            result(
                services
                    .mcp_mutate_request(
                        key(90),
                        edit,
                        "blocked".into(),
                        EntryMutation::Add(json!({"command":"blocked"}))
                    )
                    .unwrap()
            )
            .await
            .as_ref(),
            Completion::Failed(mcp::McpError::Scope)
        ));
        denied.push(matches!(
            result(
                services
                    .mcp_copy_save_request(key(91), copy, "blocked".into())
                    .unwrap()
            )
            .await
            .as_ref(),
            Completion::Failed(mcp::McpError::Scope)
        ));
        let operation = services.mcp_batch_save_request(key(92), batch).unwrap();
        loop {
            let state = operation.snapshot();
            if !state.running {
                denied.push(
                    state
                        .rows
                        .iter()
                        .all(|row| row.outcome == mcp::BatchOutcome::Failed(mcp::McpError::Scope)),
                );
                break;
            }
            tokio::task::yield_now().await;
        }
        for command in [
            resources::ResourceCommand::CreatePrompt {
                session: prompt.clone(),
            },
            resources::ResourceCommand::SavePrompt {
                session: prompt,
                content: "synthetic blocked".into(),
            },
        ] {
            denied.push(matches!(
                result(services.resource_request(key(93), command).unwrap())
                    .await
                    .as_ref(),
                Completion::Failed(resources::ResourceError::Invalid)
            ));
        }
        denied.push(matches!(
            result(
                services
                    .claude_resource_request(
                        key(94),
                        resources::claude_files::Command::Commit {
                            session: claude,
                            content: Some("synthetic blocked".into())
                        }
                    )
                    .unwrap()
            )
            .await
            .as_ref(),
            Completion::Failed(resources::claude_files::Error::UnsafePath)
        ));
        let committed = services.commit_platform_file(
            platform_key,
            platform_file,
            Some("synthetic blocked".into()),
        );
        denied.push(committed == Err(resources::platform_files::Error::UnsafePath));
        if committed.is_ok() {
            while services.platform_files_state().busy {
                tokio::task::yield_now().await;
            }
        }
        assert!(
            denied.iter().all(|denied| *denied),
            "mutate/copy/batch/create/save/Claude refusals: {denied:?}"
        );
        assert_eq!(
            tree_bytes(pair.path()),
            before,
            "both owned source trees must retain all directories and bytes"
        );
    });
    owner.shutdown();
}

#[test]
#[ignore = "closed subprocess for poisoned_ambient_process_is_ignored_by_benchmark_domains"]
fn poisoned_ambient_child() {
    let root = PathBuf::from(
        std::env::var_os("CCR_NATIVE_BENCHMARK_CHILD").expect("controlled fixture root"),
    );
    let owner = RuntimeOwner::new().unwrap();
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async {
        let services = load_services(owner.handle(), root.clone()).await.unwrap();
        let layout = services.source.benchmark().unwrap().clone();
        for surface in [platforms::Surface::Settings, platforms::Surface::Auth] {
            let reply = result(
                services
                    .platform_request(key(1), platforms::Command::Read(surface))
                    .unwrap(),
            )
            .await;
            let Completion::Success(platforms::ResultValue::Loaded(session)) = reply.as_ref()
            else {
                panic!("captured platform read failed");
            };
            assert_eq!(session.roots.home, layout.roots.home);
            assert_eq!(session.roots.ccr, layout.roots.ccr);
            assert!(!session.signed_in);
            assert_ne!(session.credential, platforms::CredentialState::Detected);
            if surface == platforms::Surface::Settings {
                let reply = result(
                    services
                        .platform_request(
                            key(20),
                            platforms::Command::Patch {
                                session: session.clone(),
                                content: r#"{"language":"zh-CN"}"#.into(),
                            },
                        )
                        .unwrap(),
                )
                .await;
                assert!(matches!(
                    reply.as_ref(),
                    Completion::Success(platforms::ResultValue::Saved(_))
                ));
            }
        }
        let requests = InsightsRequests::new(&services);
        assert!(!requests.capability_cli_probe_enabled());
        assert_eq!(
            requests.start_import(key(2)),
            Err(crate::runtime::insights::import::Failure::FixtureDisabled)
        );
        assert!(
            crate::runtime::jobs::CommandRequests::new(&services)
                .start(key(3), "status".into(), vec![])
                .is_err()
        );
        let prefs = crate::runtime::preferences::PreferencesRequests::new(&services);
        assert!(prefs.path().unwrap().starts_with(root.join("preferences")));
        let reply = result(prefs.save(Default::default(), 1).unwrap()).await;
        assert!(matches!(reply.as_ref(), Completion::Success(_)));
        let context = layout
            .profile_context(ccr_config::Platform::Claude)
            .unwrap();
        context
            .run(|| {
                let manager = ccr_config::ConfigManager::new(context.paths().profiles_file.clone());
                let mut config = ccr_config::CcsConfig {
                    default_config: "synthetic".into(),
                    current_config: String::new(),
                    settings: Default::default(),
                    sections: Default::default(),
                };
                config.sections.insert(
                    "synthetic".into(),
                    ccr_config::ConfigSection {
                        base_url: Some("https://synthetic.invalid".into()),
                        auth_token: Some(ccr_core::Secret::new("synthetic-fixture-token")),
                        model: Some("fixture-model".into()),
                        ..Default::default()
                    },
                );
                manager.save(&config)
            })
            .unwrap();
        let rows = result(services.profile_rows(key(4), String::new()).unwrap()).await;
        assert!(matches!(rows.as_ref(),Completion::Success(rows) if rows.len()==1));
        for command in [
            crate::runtime::profiles::ProfileCommand::Apply {
                name: "synthetic".into(),
                enable: true,
            },
            crate::runtime::profiles::ProfileCommand::Off,
        ] {
            let reply = result(services.profile_command(key(40), command).unwrap()).await;
            assert!(
                matches!(reply.as_ref(), Completion::Success(_)),
                "profile lifecycle must use captured locks and environment policy"
            );
        }
        if let Ok(settings) = fs::read_to_string(layout.roots.home.join(".claude/settings.json")) {
            assert!(!settings.contains("synthetic-host-only-token"));
            assert!(!settings.contains("synthetic-host-only-key"));
        }
        let reply = result(services.system_info(key(42)).unwrap()).await;
        assert!(matches!(
            reply.as_ref(),
            Completion::Failed(crate::runtime::monitoring::system::Error::Fixture)
        ));
        let reply = result(
            services
                .ssh_action(key(41), crate::runtime::environments::SshAction::Hosts)
                .unwrap(),
        )
        .await;
        assert!(matches!(
            reply.as_ref(),
            Completion::Failed("ssh_fixture_offline")
        ));
        let monitoring = result(
            crate::runtime::monitoring::Requests::new(&services)
                .refresh(key(5), Default::default())
                .unwrap(),
        )
        .await;
        assert!(
            matches!(monitoring.as_ref(),Completion::Success(report) if report.storage.is_ok())
        );
    });
    owner.shutdown();
}

#[test]
#[ignore = "explicit generated S0/S1/S2 acceptance; no UI or performance sampling"]
fn generated_pairs_load_through_native_domain_consumers() {
    let pair = PathBuf::from(
        std::env::var_os("CCR_NATIVE_BENCHMARK_PAIR_ROOT")
            .expect("explicit paired fixture directory"),
    );
    let owner = RuntimeOwner::new().unwrap();
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async {
        for scenario in ["S0","S1","S2"] {
            let root=pair.join(scenario).join("gpui");let started=std::time::Instant::now();let layout=Arc::new(Layout::load(&root).unwrap());
            println!("{scenario} startup validation {:?}, source {}, logical {}",started.elapsed(),layout.root.display(),layout.logical_sha256);
            let manifest=json(&root.parent().unwrap().join("logical-manifest.json")).unwrap();let logical=&manifest["ends"]["gpui"]["logical"];
            let services=Services::new(owner.handle(),ConfigSource::Benchmark(layout.clone()));
            assert!(services.is_fixture());assert_eq!(services.benchmark_clock().unwrap().0,layout.as_of);
            // An alternate adapter path cannot rebind this immutable source.
            let services=services.with_insights_paths(crate::runtime::insights::InsightsPaths {usage_root:pair.join("not-a-source"),archive_db:pair.join("not-an-archive"),provider_home:pair.join("not-a-home")});
            assert_eq!(services.insights_paths().unwrap().provider_home,layout.roots.home);
            let profile_rows=result(services.profile_rows(key(10),String::new()).unwrap()).await;
            if layout.profile_count==0 {assert!(matches!(profile_rows.as_ref(),Completion::Failed(crate::runtime::profiles::ProfileError::Missing)));}
            else {assert!(matches!(profile_rows.as_ref(),Completion::Success(rows) if rows.len()==layout.profile_count));}
            let context=layout.profile_context(ccr_config::Platform::Claude).unwrap();
            let projected=if layout.profile_count==0 {json!({"source":"missing","rows":[]})} else {
                let snapshot=context.run(||ccr_config::ConfigManager::new(context.paths().profiles_file.clone()).snapshot()).unwrap();
                let rows=snapshot.config.sections.iter().map(|(name,s)|json!({"name":name,"description":s.description,"base_url":s.base_url,"model":s.model,"enabled":s.enabled,"has_synthetic_credential":s.auth_token.is_some()})).collect::<Vec<_>>();
                json!({"source":"catalog","default":snapshot.config.default_config,"current":snapshot.config.current_config,"rows":rows})
            };
            assert_eq!(hash_json(&projected),logical["profiles_sha256"]);
            for surface in [platforms::Surface::Settings,platforms::Surface::Auth] {
                let reply=result(services.platform_request(key(15),platforms::Command::Read(surface)).unwrap()).await;
                assert!(matches!(reply.as_ref(),Completion::Success(platforms::ResultValue::Loaded(session)) if session.roots.home==layout.roots.home && session.roots.ccr==layout.roots.ccr && !session.signed_in));
            }
            let requests=InsightsRequests::new(&services);
            let reply=result(requests.load(key(11),InsightsQuery::Usage(ccr_usage::QueryFilter {timezone:ReportTimezone::Utc,..Default::default()})).unwrap()).await;
            let Completion::Success(snapshot)=reply.as_ref() else {panic!("usage failed");};let InsightsSnapshot::Usage(usage)=snapshot.as_ref() else {panic!("wrong report");};
            assert_eq!(usage.overview.total_events,layout.usage_count as i64);
            for (name,value) in [("tokens",serde_json::to_value(&usage.overview.total).unwrap()),("models",serde_json::to_value(&usage.models).unwrap()),("providers",serde_json::to_value(&usage.providers).unwrap()),("projects",serde_json::to_value(&usage.projects).unwrap()),("sources",serde_json::to_value(&usage.sources).unwrap()),("trends",serde_json::to_value(&usage.trends).unwrap())] {assert_eq!(value,logical["usage"][name],"{scenario}/{name}");}
            let logs=result(requests.load(key(16),InsightsQuery::Logs {filter:ccr_usage::QueryFilter {timezone:ReportTimezone::Utc,..Default::default()},cursor:None}).unwrap()).await;
            assert!(matches!(logs.as_ref(),Completion::Success(snapshot) if matches!(snapshot.as_ref(),InsightsSnapshot::Logs(page) if page.total==Some(layout.usage_count as i64) && page.records.iter().all(|record|record.record_json.is_empty()))));
            let mut rows=Vec::new();let mut cursor=None;let mut first_id=None;
            loop {
                let reply=result(requests.load(key(12),InsightsQuery::Sessions(ccr_app::sessions::SessionListRequest {cursor,limit:Some(200),..Default::default()})).unwrap()).await;
                let Completion::Success(snapshot)=reply.as_ref() else {panic!("sessions failed");};let InsightsSnapshot::Sessions {page,..}=snapshot.as_ref() else {panic!("wrong session report");};
                for row in &page.items {if first_id.is_none(){first_id=Some(row.archive_id.clone());}rows.push(json!({"id":row.session_id,"platform":row.platform,"variant":row.source_variant,"cwd":row.cwd,"title":row.title,"messages":row.message_count,"user_messages":row.user_message_count,"assistant_messages":row.assistant_message_count,"tool_uses":row.tool_use_count,"created_at":row.created_at,"updated_at":row.updated_at,"state":row.source_state.as_str(),"fidelity":row.source_fidelity}));}
                cursor=page.next_cursor.clone();if cursor.is_none(){break;}
            }
            assert_eq!(rows.len(),layout.session_count);rows.sort_by(|a,b|a["id"].as_str().cmp(&b["id"].as_str()));
            assert_eq!(hash_json(&json!(rows)),logical["sessions"]["summaries_sha256"]);
            if let Some(archive_id)=first_id {
                let reply=result(requests.load(key(13),InsightsQuery::Detail {archive_id,before:None}).unwrap()).await;
                assert!(matches!(reply.as_ref(),Completion::Success(snapshot) if matches!(snapshot.as_ref(),InsightsSnapshot::Detail(detail) if detail.page.messages.iter().any(|m|m.content.contains("Synthetic benchmark session")))));
            }
            let reply=result(crate::runtime::monitoring::Requests::new(&services).refresh(key(14),Default::default()).unwrap()).await;
            assert!(matches!(reply.as_ref(),Completion::Success(report) if report.storage.is_ok() && report.entries.is_empty()));
            // Main/catalog/archive hashes must still match after these actual read paths.
            Layout::load(&root).unwrap();
        }
    });
    owner.shutdown();
}

#[test]
fn canonical_manifest_hash_keeps_float_and_key_order_contract() {
    let value: Value = serde_json::from_str(r#"{"source":"missing","rows":[]}"#).unwrap();
    assert_eq!(
        logical_hash(&value).unwrap(),
        "be94d16d450d4625d292b120c577dfa3491ef5dfff4134f0609bf5ac94a126a0"
    );
    let value: Value = serde_json::from_str("0.09999999999999999").unwrap();
    assert_eq!(
        serde_json::to_string(&value).unwrap(),
        "0.09999999999999999"
    );
}

#[test]
fn captured_default_reports_and_explicit_home_dates_remain_distinct() {
    let (_pair, root) = s0();
    let layout = Arc::new(Layout::load(&root).unwrap());
    let owner = RuntimeOwner::new().unwrap();
    let services = Services::new(owner.handle(), ConfigSource::Benchmark(layout.clone()));
    let old = Services::new(
        owner.handle(),
        ConfigSource::Fixture {
            profiles: root.join("legacy.toml"),
            registry: root.join("legacy-registry.toml"),
            delay: std::time::Duration::ZERO,
        },
    );
    assert_eq!(old.report_clock().timezone, ReportTimezone::Local);
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async {
        let requests = InsightsRequests::new(&services);
        let reply = result(requests.load_workbench(key(92), 7).unwrap()).await;
        let Completion::Success(report) = reply.as_ref() else {
            panic!("default report failed");
        };
        assert_eq!(report.as_of, "2026-11-06");
        assert_eq!(report.days.len(), 7);
        assert_eq!(report.days.last().unwrap().label, "11-06");
        let explicit = chrono::DateTime::parse_from_rfc3339("2000-01-01T00:30:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let reply = result(
            requests
                .load(
                    key(93),
                    InsightsQuery::Home {
                        as_of: explicit.date_naive(),
                        generated_at: explicit,
                    },
                )
                .unwrap(),
        )
        .await;
        let Completion::Success(snapshot) = reply.as_ref() else {
            panic!("explicit report failed");
        };
        let InsightsSnapshot::Home(report) = snapshot.as_ref() else {
            panic!("wrong report");
        };
        assert_eq!(report.as_of, "2000-01-01");
        assert_eq!(report.generated_at, explicit.to_rfc3339());
    });
    owner.shutdown();
}

#[test]
#[ignore = "explicit generated pairs; default report readback without UI or samples"]
fn generated_default_workbench_uses_the_same_manifest_utc_window() {
    let pair = PathBuf::from(std::env::var_os("CCR_NATIVE_BENCHMARK_PAIR_ROOT").unwrap());
    let owner = RuntimeOwner::new().unwrap();
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async {
        for scenario in ["S0","S1","S2"] {
            let layout=Arc::new(Layout::load(&pair.join(scenario).join("gpui")).unwrap());
            let services=Services::new(owner.handle(),ConfigSource::Benchmark(layout.clone()));let clock=services.report_clock();
            let manifest=json(&layout.root.parent().unwrap().join("logical-manifest.json")).unwrap();
            let start=clock.window(7).0.to_string();let end=clock.date().to_string();
            let expected=manifest["ends"]["gpui"]["logical"]["usage"]["trends"].as_array().unwrap().iter()
                .filter(|row|row["date"].as_str().is_some_and(|date|date>=start.as_str() && date<=end.as_str()))
                .map(|row|row["request_count"].as_i64().unwrap()).sum::<i64>();
            let requests=InsightsRequests::new(&services);let reply=result(requests.load_workbench(key(100),7).unwrap()).await;
            let Completion::Success(report)=reply.as_ref() else {panic!("workbench failed");};
            assert_eq!(report.as_of,end);assert_eq!(report.requests,expected);
            if layout.session_count>0 {assert_eq!(report.session_count,Some(layout.session_count as i64));}
            let reply=result(requests.observer_read(key(101),7,false).unwrap()).await;
            assert!(matches!(reply.as_ref(),Completion::Success(report) if report.usage.is_ok() && report.insight.is_ok()));
            println!("{scenario}: native default window {start}..{end} UTC, requests {expected}, sessions {:?}",report.session_count);
        }
    });
    owner.shutdown();
}
