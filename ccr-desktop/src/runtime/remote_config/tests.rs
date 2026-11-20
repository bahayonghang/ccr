#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::*;
use crate::runtime::{
    ConfigSource,
    environments::{SshAction, SshResult},
    platforms::{AccountAction, Command, ResultValue, Surface},
    profiles::{ProfileCommand, ProfileResult},
};
use ccr_app::{
    environments::ssh::{Host, HostInput, Workspace, fixture::FixtureTransport},
    runtime::{Completion, RequestKey, RuntimeOwner, TaskEvent, TaskSubscription},
};
use std::time::Duration;

struct Fixture {
    root: tempfile::TempDir,
    runtime: RuntimeOwner,
    services: Services,
    workspace: Arc<Workspace>,
    transport: Arc<FixtureTransport>,
    host: Host,
}
async fn terminal<T: Send + Sync + 'static, E: Send + Sync + 'static>(
    mut task: TaskSubscription<T, E>,
) -> Arc<Completion<T, E>> {
    while let Some(event) = task.next().await {
        if let TaskEvent::Terminal(result) = event {
            return result;
        }
    }
    panic!("terminal missing")
}
impl Fixture {
    async fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        let runtime = RuntimeOwner::new().unwrap();
        let services = Services::new(
            runtime.handle(),
            ConfigSource::Fixture {
                profiles: root.path().join("profiles.toml"),
                registry: root.path().join("registry.toml"),
                delay: Duration::ZERO,
            },
        );
        let (workspace, transport) =
            Workspace::fixture(root.path().join("ssh.db"), root.path().join("known_hosts"));
        let workspace = Arc::new(workspace);
        services
            .ssh
            .set(workspace.clone())
            .unwrap_or_else(|_| panic!("workspace"));
        transport.set_home("/home/probed-fixture");
        let result = terminal(
            services
                .ssh_action(
                    RequestKey {
                        environment: EnvironmentId::Local("fixture".into()),
                        platform: Platform::Claude,
                        generation: 1,
                        request_id: 1,
                    },
                    SshAction::SaveHost(HostInput {
                        name: "Configuration fixture".into(),
                        host: "config.fixture.invalid".into(),
                        user: "fixture".into(),
                        port: 2222,
                        remote_home: "/srv/configuration-fixture".into(),
                        ..Default::default()
                    }),
                )
                .unwrap(),
        )
        .await;
        let Completion::Success(SshResult::HostSaved(host)) = result.as_ref() else {
            panic!("host")
        };
        let host = host.clone();
        let key = RequestKey {
            environment: EnvironmentId::Ssh(host.id.clone()),
            platform: Platform::Claude,
            generation: 2,
            request_id: 2,
        };
        let result = terminal(
            services
                .ssh_action(key.clone(), SshAction::Probe(host.id.clone()))
                .unwrap(),
        )
        .await;
        let Completion::Success(SshResult::Probe(probe)) = result.as_ref() else {
            panic!("probe")
        };
        assert!(matches!(
            terminal(
                services
                    .ssh_action(key.clone(), SshAction::Confirm(probe.challenge_id.clone()))
                    .unwrap()
            )
            .await
            .as_ref(),
            Completion::Success(SshResult::Confirmed)
        ));
        assert!(matches!(
            terminal(
                services
                    .ssh_action(key, SshAction::Connect(host.id.clone()))
                    .unwrap()
            )
            .await
            .as_ref(),
            Completion::Success(SshResult::Connected)
        ));
        services.select_environment(EnvironmentId::Ssh(host.id.clone()));
        // 本机平台根是毒化文件；任何回退或环境覆盖解析都会使本测试失败。
        std::fs::write(
            root.path().join("desktop-platform-fixture"),
            "local-root-must-stay-untouched",
        )
        .unwrap();
        std::fs::write(
            root.path().join("desktop-profile-fixture"),
            "local-profile-root-must-stay-untouched",
        )
        .unwrap();
        Self {
            root,
            runtime,
            services,
            workspace,
            transport,
            host,
        }
    }
    fn key(&self, platform: Platform) -> RequestKey {
        RequestKey {
            environment: EnvironmentId::Ssh(self.host.id.clone()),
            platform,
            generation: 1,
            request_id: 1,
        }
    }
    fn path(relative: &str) -> String {
        format!("/srv/configuration-fixture/{relative}")
    }
    fn set(&self, path: &str, content: &str) {
        self.transport
            .set_file("config.fixture.invalid", &Self::path(path), content);
    }
    fn file(&self, path: &str) -> Option<String> {
        self.transport
            .file("config.fixture.invalid", &Self::path(path))
    }
    async fn read(&self, platform: Platform, surface: Surface) -> Arc<PlatformSession> {
        let result = terminal(
            self.services
                .platform_request(self.key(platform), Command::Read(surface))
                .unwrap(),
        )
        .await;
        match result.as_ref() {
            Completion::Success(ResultValue::Loaded(session)) => session.clone(),
            _ => panic!("platform read failed"),
        }
    }
    async fn profile_catalog(&self, platform: Platform) -> super::super::profiles::ProfileCatalog {
        let terminal = terminal(
            self.services
                .profile_catalog(self.key(platform), String::new())
                .unwrap(),
        )
        .await;
        match terminal.as_ref() {
            Completion::Success(catalog) => catalog.clone(),
            _ => panic!("expected remote catalog"),
        }
    }
    async fn command(
        &self,
        platform: Platform,
        command: ProfileCommand,
    ) -> Arc<Completion<ProfileResult, crate::runtime::profiles::ProfileError>> {
        terminal(
            self.services
                .profile_command(self.key(platform), command)
                .unwrap(),
        )
        .await
    }
}

#[gpui_kit::test]
async fn five_remote_settings_use_posix_home_owners_backups_and_metadata_only_sources(
    cx: &mut gpui_kit::TestAppContext,
) {
    cx.executor().allow_parking();
    let fixture = Fixture::new().await;
    for (platform, surface, path, original, patch) in [
        (
            Platform::Claude,
            Surface::Settings,
            ".claude/settings.json",
            "{\"model\":\"old\",\"env\":{\"ANTHROPIC_AUTH_TOKEN\":\"synthetic-secret\"}}",
            "{\"model\":\"updated\"}",
        ),
        (
            Platform::Codex,
            Surface::Settings,
            ".codex/config.toml",
            "model='old'\n",
            "{\"model\":\"updated\"}",
        ),
        (
            Platform::Grok,
            Surface::Settings,
            ".grok/config.toml",
            "[ui]\ntheme='light'\n",
            "{\"ui\":{\"theme\":\"dark\"}}",
        ),
        (
            Platform::OpenCode,
            Surface::Settings,
            ".config/opencode/opencode.json",
            "{\"model\":\"old\"}",
            "{\"model\":\"updated\"}",
        ),
        (
            Platform::Gemini,
            Surface::LegacySettings,
            ".gemini/antigravity-cli/settings.json",
            "{\"theme\":\"light\"}",
            "{\"theme\":\"dark\"}",
        ),
    ] {
        fixture.set(path, original);
        let session = fixture.read(platform, surface).await;
        let private = session.remote.as_ref().unwrap().root().to_path_buf();
        assert!(!format!("{:?}", session.fields).contains("synthetic-secret"));
        let command = if platform == Platform::Gemini {
            Command::Source {
                session: session.clone(),
                content: patch.into(),
            }
        } else {
            Command::Patch {
                session: session.clone(),
                content: patch.into(),
            }
        };
        assert!(matches!(
            terminal(
                fixture
                    .services
                    .platform_request(fixture.key(platform), command)
                    .unwrap()
            )
            .await
            .as_ref(),
            Completion::Success(ResultValue::Saved(
                ccr_app::platform_settings::SaveOutcome::Saved { .. }
            ))
        ));
        let saved = fixture.file(path).unwrap();
        assert_ne!(saved, original);
        assert!(!saved.contains(&private.to_string_lossy().to_string()));
        let trace = fixture.transport.batches();
        let target = Fixture::path(path);
        let backup = trace
            .iter()
            .rposition(|item| item.starts_with(&format!("rename {target}.bak.")))
            .unwrap();
        let replace = trace
            .iter()
            .rposition(|item| item == &format!("rename {target}"))
            .unwrap();
        assert!(backup < replace);
        let before = fixture.transport.batches().len();
        let layers = terminal(
            fixture
                .services
                .platform_layers_request(fixture.key(platform), None)
                .unwrap(),
        )
        .await;
        let Completion::Success(layers) = layers.as_ref() else {
            panic!("layers")
        };
        assert!(
            layers
                .iter()
                .any(|row| row.id == "user" && row.exists == Some(true))
        );
        assert!(
            fixture.transport.batches()[before..]
                .iter()
                .all(|item| item.starts_with("stat ") || item.starts_with("list "))
        );
        assert!(layers.iter().all(|row| matches!(
            row.path,
            None | Some(ccr_app::environments::io::EnvironmentPath::Ssh(_))
        )));
        drop(session);
        assert!(
            !private.exists(),
            "private secret mirror is removed after session drop"
        );
    }
    assert!(fixture.file(".opencode/opencode.json").is_none());
    assert_eq!(
        std::fs::read_to_string(fixture.root.path().join("desktop-platform-fixture")).unwrap(),
        "local-root-must-stay-untouched"
    );
    fixture.runtime.shutdown();
}

#[gpui_kit::test]
async fn three_remote_profile_owners_create_apply_off_and_keep_catalogs_and_secrets_remote(
    cx: &mut gpui_kit::TestAppContext,
) {
    cx.executor().allow_parking();
    let fixture = Fixture::new().await;
    for platform in [Platform::Claude, Platform::Codex, Platform::Grok] {
        let mut patch = ccr_config::ConfigPatch::default();
        patch.fields.insert(
            "auth_token".into(),
            ccr_config::FieldPatch::Set("synthetic-profile-secret".into()),
        );
        patch.fields.insert(
            "base_url".into(),
            ccr_config::FieldPatch::Set("https://profile.fixture.invalid".into()),
        );
        patch.fields.insert(
            "model".into(),
            ccr_config::FieldPatch::Set("fixture-model".into()),
        );
        if platform == Platform::Grok {
            patch.fields.shift_remove("auth_token");
            for (key, value) in [
                ("credential_action", "replace_api_key"),
                ("api_key", "synthetic-profile-secret"),
                ("api_backend", "responses"),
            ] {
                patch
                    .fields
                    .insert(key.into(), ccr_config::FieldPatch::Set(value.into()));
            }
        }
        let created = fixture
            .command(
                platform,
                ProfileCommand::Create {
                    name: "fixture-profile".into(),
                    patch,
                },
            )
            .await;
        assert!(
            matches!(created.as_ref(), Completion::Success(_)),
            "create failed: {}",
            platform.short_name()
        );
        let rows = terminal(
            fixture
                .services
                .profile_catalog(fixture.key(platform), String::new())
                .unwrap(),
        )
        .await;
        let Completion::Success(rows) = rows.as_ref() else {
            panic!("rows")
        };
        assert_eq!(rows.rows.len(), 1);
        assert!(!rows.rows[0].auth_token.contains("synthetic-profile-secret"));
        let applied = fixture
            .command(
                platform,
                ProfileCommand::Apply {
                    name: "fixture-profile".into(),
                    enable: false,
                },
            )
            .await;
        assert!(
            matches!(applied.as_ref(), Completion::Success(ProfileResult::Outcome(outcome)) if outcome.activation_committed),
            "apply failed: {}",
            platform.short_name()
        );
        use ccr_cli::application::profile_observation::RuntimeState;
        let catalog = fixture.profile_catalog(platform).await;
        assert_eq!(catalog.observation.name.as_deref(), Some("fixture-profile"));
        assert_eq!(catalog.observation.state, RuntimeState::Matched);
        assert!(
            catalog
                .runtime_file
                .starts_with("/srv/configuration-fixture/")
        );
        if platform == Platform::Claude {
            let registry = ".ccr/config.toml";
            let settings = ".claude/settings.json";
            fixture
                .transport
                .fail_path("get", &Fixture::path(registry), "ssh_file_access_denied");
            let file_marker = fixture.profile_catalog(platform).await;
            assert_eq!(
                file_marker.observation.name.as_deref(),
                Some("fixture-profile")
            );
            assert_eq!(file_marker.observation.state, RuntimeState::Matched);

            let original_registry = fixture.file(registry).unwrap();
            let original_settings = fixture.file(settings).unwrap();
            assert!(matches!(
                fixture
                    .command(platform, ProfileCommand::Off)
                    .await
                    .as_ref(),
                Completion::Success(_)
            ));
            fixture.set(registry, &original_registry);
            fixture.set(settings, &original_settings);
            fixture
                .transport
                .fail_path("get", &Fixture::path(settings), "ssh_file_access_denied");
            let registry_marker = fixture.profile_catalog(platform).await;
            assert_eq!(registry_marker.rows.len(), 1);
            assert_eq!(
                registry_marker.observation.name.as_deref(),
                Some("fixture-profile")
            );
            assert_eq!(registry_marker.observation.state, RuntimeState::Unreadable);
            assert_eq!(fixture.file(registry).as_ref(), Some(&original_registry));
            assert_eq!(fixture.file(settings).as_ref(), Some(&original_settings));
        }
        assert!(matches!(
            fixture
                .command(platform, ProfileCommand::Status)
                .await
                .as_ref(),
            Completion::Success(ProfileResult::Status { can_off: true })
        ));
        assert!(matches!(
            fixture
                .command(platform, ProfileCommand::Off)
                .await
                .as_ref(),
            Completion::Success(_)
        ));
        let directory = format!(".ccr/platforms/{}", platform.short_name());
        let bytes = fixture
            .transport
            .bytes(
                "config.fixture.invalid",
                &Fixture::path(&format!("{directory}/profiles.db")),
            )
            .unwrap();
        assert!(bytes.starts_with(b"SQLite format 3"));
        assert!(!String::from_utf8_lossy(&bytes).contains("ccr-configuration-"));
        for suffix in ["-wal", "-shm", "-journal"] {
            assert!(
                fixture
                    .file(&format!("{directory}/profiles.db{suffix}"))
                    .is_none()
            );
        }
    }
    for platform in [Platform::Gemini, Platform::OpenCode] {
        assert!(matches!(
            fixture
                .command(platform, ProfileCommand::Off)
                .await
                .as_ref(),
            Completion::Failed(crate::runtime::profiles::ProfileError::Unsupported)
        ));
    }
    assert_eq!(
        std::fs::read_to_string(fixture.root.path().join("desktop-profile-fixture")).unwrap(),
        "local-profile-root-must-stay-untouched"
    );
    fixture.transport.fail_path(
        "get",
        &Fixture::path(".claude/settings.json"),
        "ssh_network_error",
    );
    let disconnected = terminal(
        fixture
            .services
            .profile_catalog(fixture.key(Platform::Claude), String::new())
            .unwrap(),
    )
    .await;
    assert!(matches!(disconnected.as_ref(), Completion::Failed(_)));
    fixture.runtime.shutdown();
}

#[gpui_kit::test]
async fn three_remote_file_account_owners_save_switch_off_delete_without_local_fallback(
    cx: &mut gpui_kit::TestAppContext,
) {
    cx.executor().allow_parking();
    let fixture = Fixture::new().await;
    for (platform, path, content, save) in [
        (
            Platform::Claude,
            ".claude/.credentials.json",
            r#"{"claudeAiOauth":{"accessToken":"synthetic-access","refreshToken":"synthetic-refresh","expiresAt":4102444800000,"scopes":["user:inference"],"subscriptionType":"pro"}}"#,
            AccountAction::Save,
        ),
        (
            Platform::Codex,
            ".codex/auth.json",
            r#"{"OPENAI_API_KEY":"synthetic-codex-key"}"#,
            AccountAction::Save,
        ),
        (
            Platform::Grok,
            ".grok/auth.json",
            r#"{"https://auth.x.ai::fixture-client":{"auth_mode":"oidc","oidc_issuer":"https://auth.x.ai","oidc_client_id":"fixture-client","key":"synthetic-grok-key","user_id":"fixture-user","create_time":"2026-01-01T00:00:00Z"}}"#,
            AccountAction::SaveFrom("https://auth.x.ai::fixture-client".into()),
        ),
    ] {
        fixture.set(path, content);
        for action in [
            save,
            AccountAction::Off,
            AccountAction::Switch,
            AccountAction::Delete,
        ] {
            let session = fixture.read(platform, Surface::Auth).await;
            assert!(
                matches!(
                    terminal(
                        fixture
                            .services
                            .platform_request(
                                fixture.key(platform),
                                Command::Account {
                                    session,
                                    name: "fixture-account".into(),
                                    action
                                }
                            )
                            .unwrap()
                    )
                    .await
                    .as_ref(),
                    Completion::Success(ResultValue::AccountChanged { .. })
                ),
                "account failed: {}",
                platform.short_name()
            );
        }
        assert!(
            fixture
                .read(platform, Surface::Auth)
                .await
                .accounts
                .is_empty()
        );
    }
    for platform in [Platform::Gemini, Platform::OpenCode] {
        let result = terminal(
            fixture
                .services
                .platform_request(fixture.key(platform), Command::Read(Surface::Auth))
                .unwrap(),
        )
        .await;
        assert!(matches!(
            result.as_ref(),
            Completion::Failed(PlatformError::Unsupported)
        ));
    }
    fixture.runtime.shutdown();
}

#[gpui_kit::test]
async fn remote_cas_sidecars_missing_files_and_reconnect_are_fail_closed(
    cx: &mut gpui_kit::TestAppContext,
) {
    cx.executor().allow_parking();
    let fixture = Fixture::new().await;
    let session = fixture.read(Platform::Claude, Surface::Settings).await;
    assert!(!session.snapshot.exists);
    fixture.set(".claude/settings.json", "{\"external\":true}");
    let result = terminal(
        fixture
            .services
            .platform_request(
                fixture.key(Platform::Claude),
                Command::Source {
                    session,
                    content: "{}".into(),
                },
            )
            .unwrap(),
    )
    .await;
    assert!(matches!(
        result.as_ref(),
        Completion::Failed(PlatformError::Conflict)
    ));
    assert_eq!(
        fixture.file(".claude/settings.json").as_deref(),
        Some("{\"external\":true}")
    );
    for suffix in ["-wal", "-shm", "-journal"] {
        fixture.set(&format!(".ccr/platforms/codex/profiles.db{suffix}"), "");
        let result = terminal(
            fixture
                .services
                .profile_catalog(fixture.key(Platform::Codex), String::new())
                .unwrap(),
        )
        .await;
        assert!(matches!(
            result.as_ref(),
            Completion::Failed(crate::runtime::profiles::ProfileError::RemoteBusy)
        ));
    }
    let session = fixture.read(Platform::Claude, Surface::Settings).await;
    assert!(matches!(
        terminal(
            fixture
                .services
                .ssh_action(
                    fixture.key(Platform::Claude),
                    SshAction::Connect(fixture.host.id.clone())
                )
                .unwrap()
        )
        .await
        .as_ref(),
        Completion::Success(SshResult::Connected)
    ));
    assert!(matches!(
        terminal(
            fixture
                .services
                .platform_request(
                    fixture.key(Platform::Claude),
                    Command::Source {
                        session,
                        content: "{}".into()
                    }
                )
                .unwrap()
        )
        .await
        .as_ref(),
        Completion::Failed(PlatformError::Storage)
    ));
    assert_eq!(
        fixture.file(".claude/settings.json").as_deref(),
        Some("{\"external\":true}")
    );
    assert!(fixture.workspace.connected(&fixture.host.id));
    fixture.runtime.shutdown();
}

#[gpui_kit::test]
async fn queued_remote_mutations_cannot_acquire_a_reconnected_sessions_authority(
    cx: &mut gpui_kit::TestAppContext,
) {
    cx.executor().allow_parking();
    let fixture = Fixture::new().await;
    let session = fixture.read(Platform::Claude, Surface::Settings).await;
    let entered = Arc::new(std::sync::Barrier::new(
        ccr_app::runtime::BLOCKING_CONCURRENCY + 1,
    ));
    let release = Arc::new((std::sync::Mutex::new(false), std::sync::Condvar::new()));
    let mut blockers = Vec::new();
    for _ in 0..ccr_app::runtime::BLOCKING_CONCURRENCY {
        let entered = entered.clone();
        let release = release.clone();
        blockers.push(
            fixture
                .services
                .executor
                .read(fixture.key(Platform::Claude), move |_, _| {
                    entered.wait();
                    let (lock, wake) = &*release;
                    let _guard = wake
                        .wait_while(lock.lock().unwrap(), |released| !*released)
                        .unwrap();
                    Ok::<_, ()>(())
                })
                .unwrap(),
        );
    }
    entered.wait();
    let settings = fixture
        .services
        .platform_request(
            fixture.key(Platform::Claude),
            Command::Source {
                session,
                content: "{}".into(),
            },
        )
        .unwrap();
    let profile = fixture
        .services
        .profile_command(
            fixture.key(Platform::Claude),
            ProfileCommand::Create {
                name: "queued".into(),
                patch: Default::default(),
            },
        )
        .unwrap();
    let reconnect_runtime = RuntimeOwner::new().unwrap();
    let reconnect = Services::new(reconnect_runtime.handle(), fixture.services.source.clone());
    reconnect
        .ssh
        .set(fixture.workspace.clone())
        .unwrap_or_else(|_| panic!("workspace"));
    assert!(matches!(
        terminal(
            reconnect
                .ssh_action(
                    fixture.key(Platform::Claude),
                    SshAction::Connect(fixture.host.id.clone())
                )
                .unwrap()
        )
        .await
        .as_ref(),
        Completion::Success(SshResult::Connected)
    ));
    let before = fixture.transport.batches().len();
    {
        let (lock, wake) = &*release;
        *lock.lock().unwrap() = true;
        wake.notify_all();
    }
    assert!(matches!(
        terminal(settings).await.as_ref(),
        Completion::Failed(PlatformError::Storage)
    ));
    assert!(matches!(
        terminal(profile).await.as_ref(),
        Completion::Failed(crate::runtime::profiles::ProfileError::Storage)
    ));
    for task in blockers {
        assert!(matches!(
            terminal(task).await.as_ref(),
            Completion::Success(())
        ));
    }
    assert!(
        fixture.transport.batches()[before..]
            .iter()
            .all(|item| !item.starts_with("put ")
                && !item.starts_with("rename ")
                && !item.starts_with("remove "))
    );
    assert!(fixture.file(".claude/settings.json").is_none());
    reconnect_runtime.shutdown();
    fixture.runtime.shutdown();
}

#[gpui_kit::test]
async fn remote_posix_account_aliases_are_rejected_before_materialization_or_mutation(
    cx: &mut gpui_kit::TestAppContext,
) {
    cx.executor().allow_parking();
    for names in [["One.json", "one.json"], ["CON.json", "ordinary.json"]] {
        let fixture = Fixture::new().await;
        for name in names {
            fixture.set(
                &format!(".ccr/platforms/claude/auth/{name}"),
                "{\"synthetic\":true}",
            );
        }
        let before = fixture.transport.batches().len();
        let result = terminal(
            fixture
                .services
                .platform_request(fixture.key(Platform::Claude), Command::Read(Surface::Auth))
                .unwrap(),
        )
        .await;
        assert!(matches!(
            result.as_ref(),
            Completion::Failed(PlatformError::RemotePathUnsupported)
        ));
        assert!(
            fixture.transport.batches()[before..]
                .iter()
                .all(|item| item.starts_with("list "))
        );
        for name in names {
            assert_eq!(
                fixture
                    .file(&format!(".ccr/platforms/claude/auth/{name}"))
                    .as_deref(),
                Some("{\"synthetic\":true}")
            );
        }
        fixture.runtime.shutdown();
    }
}

#[gpui_kit::test]
async fn remote_source_save_keeps_canonical_preimage_before_catalog_and_records_only_final_results(
    cx: &mut gpui_kit::TestAppContext,
) {
    cx.executor().allow_parking();
    let fixture = Fixture::new().await;
    let mut patch = ccr_config::ConfigPatch::default();
    for (key, value) in [
        ("auth_token", "synthetic-profile-secret"),
        ("base_url", "https://profile.fixture.invalid"),
        ("model", "original-model"),
    ] {
        patch
            .fields
            .insert(key.into(), ccr_config::FieldPatch::Set(value.into()));
    }
    assert!(matches!(
        fixture
            .command(
                Platform::Claude,
                ProfileCommand::Create {
                    name: "fixture-profile".into(),
                    patch
                }
            )
            .await
            .as_ref(),
        Completion::Success(_)
    ));
    let source = fixture
        .command(Platform::Claude, ProfileCommand::Source)
        .await;
    let Completion::Success(ProfileResult::Source {
        content,
        revision,
        path,
        ..
    }) = source.as_ref()
    else {
        panic!("source")
    };
    assert_eq!(
        path,
        "/srv/configuration-fixture/.ccr/platforms/claude/profiles.db"
    );
    let before = fixture.transport.batches().len();
    assert!(matches!(
        fixture
            .command(
                Platform::Claude,
                ProfileCommand::SaveSource {
                    content: content.replace("original-model", "updated-model"),
                    revision: revision.clone()
                }
            )
            .await
            .as_ref(),
        Completion::Success(ProfileResult::SourceSave(
            ccr_app::config::source::SourceSaveOutcome::Saved { .. }
        ))
    ));
    let trace = fixture.transport.batches();
    let trace = &trace[before..];
    let canonical = trace.iter().position(|item| item.starts_with("rename /srv/configuration-fixture/.ccr/platforms/claude/profiles.toml.ssh_source_preimage_") && item.ends_with(".bak")).unwrap();
    let catalog = trace
        .iter()
        .position(|item| {
            item == "rename /srv/configuration-fixture/.ccr/platforms/claude/profiles.db"
        })
        .unwrap();
    assert!(canonical < catalog);
    let backup_path = trace[canonical].strip_prefix("rename ").unwrap();
    assert!(
        fixture
            .transport
            .file("config.fixture.invalid", backup_path)
            .unwrap()
            .contains("original-model")
    );
    let before = fixture.transport.batches().len();
    assert!(matches!(
        fixture
            .command(
                Platform::Claude,
                ProfileCommand::SaveSource {
                    content: content.clone(),
                    revision: revision.clone()
                }
            )
            .await
            .as_ref(),
        Completion::Success(ProfileResult::SourceSave(
            ccr_app::config::source::SourceSaveOutcome::Conflict
        ))
    ));
    assert!(
        fixture.transport.batches()[before..]
            .iter()
            .all(|item| !item.starts_with("put ") && !item.starts_with("rename "))
    );
    let history = ccr_store::HistoryManager::new(
        ccr_store::Database::init(&fixture.root.path().join("history.db")).unwrap(),
    )
    .load()
    .unwrap();
    assert_eq!(history.len(), 3);
    assert_eq!(
        history
            .iter()
            .filter(|entry| matches!(entry.result, ccr_store::OperationResult::Success))
            .count(),
        2
    );
    assert_eq!(
        history
            .iter()
            .filter(|entry| matches!(entry.result, ccr_store::OperationResult::Failure(_)))
            .count(),
        1
    );
    assert!(
        serde_json::to_string(&history)
            .unwrap()
            .contains("remote_configuration:claude")
    );
    assert!(
        !serde_json::to_string(&history)
            .unwrap()
            .contains("ccr-configuration-")
    );
    fixture.runtime.shutdown();
}

#[gpui_kit::test]
async fn remote_unconnected_collection_and_network_actions_are_rejected_by_runtime(
    cx: &mut gpui_kit::TestAppContext,
) {
    cx.executor().allow_parking();
    let fixture = Fixture::new().await;
    for command in [
        ProfileCommand::Backups,
        ProfileCommand::Restore {
            leaf: "profiles.toml.fixture.bak".into(),
        },
        ProfileCommand::CleanBackups,
        ProfileCommand::Import {
            content: "synthetic".into(),
            replace: false,
        },
        ProfileCommand::SaveExport {
            content: "synthetic".into(),
            destination: fixture.root.path().join("must-not-save.toml"),
        },
    ] {
        assert!(matches!(
            fixture.command(Platform::Claude, command).await.as_ref(),
            Completion::Failed(crate::runtime::profiles::ProfileError::Unsupported)
        ));
    }
    let session = fixture.read(Platform::Codex, Surface::Auth).await;
    assert!(matches!(
        terminal(
            fixture
                .services
                .platform_request(
                    fixture.key(Platform::Codex),
                    Command::Quota {
                        session: session.clone(),
                        target: crate::runtime::platforms::QuotaTarget::AllSaved
                    }
                )
                .unwrap()
        )
        .await
        .as_ref(),
        Completion::Failed(PlatformError::Unsupported)
    ));
    assert!(matches!(
        terminal(
            fixture
                .services
                .platform_oauth_start(fixture.key(Platform::Codex), session)
                .unwrap()
        )
        .await
        .as_ref(),
        Completion::Failed(PlatformError::Unsupported)
    ));
    assert!(!fixture.root.path().join("must-not-save.toml").exists());
    assert!(
        fixture
            .transport
            .batches()
            .iter()
            .all(|item| !item.starts_with("put ")
                && !item.starts_with("rename ")
                && !item.starts_with("exec "))
    );
    fixture.runtime.shutdown();
}

#[gpui_kit::test]
async fn remote_settings_ignore_unrelated_catalog_and_account_failures_but_grok_write_keeps_its_guard(
    cx: &mut gpui_kit::TestAppContext,
) {
    cx.executor().allow_parking();
    for platform in [
        Platform::Claude,
        Platform::Codex,
        Platform::Grok,
        Platform::Gemini,
        Platform::OpenCode,
    ] {
        let fixture = Fixture::new().await;
        fixture.set(
            &format!(".ccr/platforms/{}/profiles.db", platform.short_name()),
            "damaged-catalog",
        );
        fixture.set(
            &format!(".ccr/platforms/{}/profiles.db-wal", platform.short_name()),
            "active-wal",
        );
        fixture.transport.fail_with("list", "ssh_read_failed");
        let (surface, path, content) = match platform {
            Platform::Claude => (Surface::Settings, ".claude/settings.json", "{}"),
            Platform::Codex => (Surface::Settings, ".codex/config.toml", "model='fixture'\n"),
            Platform::Grok => (
                Surface::Settings,
                ".grok/config.toml",
                "[ui]\ntheme='light'\n",
            ),
            Platform::Gemini => (
                Surface::LegacySettings,
                ".gemini/antigravity-cli/settings.json",
                "{}",
            ),
            Platform::OpenCode => (Surface::Settings, ".config/opencode/opencode.json", "{}"),
            _ => unreachable!(),
        };
        fixture.set(path, content);
        let before = fixture.transport.batches().len();
        let session = fixture.read(platform, surface).await;
        assert!(
            fixture.transport.batches()[before..]
                .iter()
                .all(|item| item == &format!("get {}", Fixture::path(path)))
        );
        let replacement = match platform {
            Platform::Codex => "model='updated'\n",
            Platform::Grok => "[ui]\ntheme='dark'\n",
            Platform::Gemini => "{\"theme\":\"dark\"}",
            _ => "{\"model\":\"updated\"}",
        };
        let result = terminal(
            fixture
                .services
                .platform_request(
                    fixture.key(platform),
                    Command::Source {
                        session,
                        content: replacement.into(),
                    },
                )
                .unwrap(),
        )
        .await;
        if platform == Platform::Grok {
            assert!(matches!(
                result.as_ref(),
                Completion::Failed(PlatformError::RemoteBusy)
            ));
        } else {
            assert!(matches!(
                result.as_ref(),
                Completion::Success(ResultValue::Saved(
                    ccr_app::platform_settings::SaveOutcome::Saved { .. }
                ))
            ));
            assert_ne!(fixture.file(path).as_deref(), Some(content));
        }
        assert!(
            fixture
                .transport
                .batches()
                .iter()
                .all(|item| !item.starts_with("list "))
        );
        fixture.runtime.shutdown();
    }
}

#[gpui_kit::test]
async fn application_history_is_available_and_clearable_without_any_ssh_authority(
    cx: &mut gpui_kit::TestAppContext,
) {
    cx.executor().allow_parking();
    let fixture = Fixture::new().await;
    assert!(super::record(
        Some(&fixture.root.path().join("history.db")),
        Platform::Claude,
        true
    ));
    terminal(
        fixture
            .services
            .ssh_action(fixture.key(Platform::Claude), SshAction::Disconnect)
            .unwrap(),
    )
    .await;
    let before = fixture.transport.batches();
    assert!(
        matches!(fixture.command(Platform::Codex, ProfileCommand::History).await.as_ref(), Completion::Success(ProfileResult::Lines(lines)) if lines.len() == 1)
    );
    let clear = fixture
        .services
        .profile_command(fixture.key(Platform::Codex), ProfileCommand::ClearHistory)
        .unwrap();
    assert_eq!(
        clear.key().environment,
        EnvironmentId::Local("application".into())
    );
    fixture
        .services
        .executor
        .begin_environment_drain(EnvironmentId::Ssh(fixture.host.id.clone()))
        .recv()
        .await
        .unwrap();
    assert!(!clear.is_cancel_requested());
    assert!(matches!(
        terminal(clear).await.as_ref(),
        Completion::Success(ProfileResult::Done(_))
    ));
    assert!(
        matches!(fixture.command(Platform::Grok, ProfileCommand::History).await.as_ref(), Completion::Success(ProfileResult::Lines(lines)) if lines.is_empty())
    );
    assert_eq!(fixture.transport.batches(), before);
    fixture.runtime.shutdown();
}

#[gpui_kit::test]
async fn settings_history_failure_preserves_the_saved_document_outcome(
    cx: &mut gpui_kit::TestAppContext,
) {
    cx.executor().allow_parking();
    let fixture = Fixture::new().await;
    std::fs::create_dir(fixture.root.path().join("history.db")).unwrap();
    let session = fixture.read(Platform::Claude, Surface::Settings).await;
    let result = terminal(
        fixture
            .services
            .platform_request(
                fixture.key(Platform::Claude),
                Command::Source {
                    session,
                    content: "{\"model\":\"saved\"}".into(),
                },
            )
            .unwrap(),
    )
    .await;
    assert!(matches!(
        result.as_ref(),
        Completion::Success(ResultValue::SavedWithWarning {
            outcome: ccr_app::platform_settings::SaveOutcome::Saved { .. }
        })
    ));
    assert_eq!(
        fixture.file(".claude/settings.json").as_deref(),
        Some("{\"model\":\"saved\"}")
    );
    fixture.runtime.shutdown();
}

#[gpui_kit::test]
async fn remote_provider_owners_write_their_own_documents_and_models_merge_remote_custom_data(
    cx: &mut gpui_kit::TestAppContext,
) {
    cx.executor().allow_parking();
    let fixture = Fixture::new().await;
    for (platform, path, patch) in [
        (
            Platform::Codex,
            ".ccr/platforms/codex/model_providers.json",
            r#"{"name":"Fixture provider","base_url":"https://provider.fixture.invalid","api_key":"synthetic-provider-secret"}"#,
        ),
        (
            Platform::OpenCode,
            ".config/opencode/opencode.json",
            r#"{"name":"Fixture provider","options":{"baseURL":"https://provider.fixture.invalid","apiKey":"synthetic-provider-secret"}}"#,
        ),
    ] {
        let session = fixture.read(platform, Surface::Providers).await;
        let saved = terminal(
            fixture
                .services
                .platform_request(
                    fixture.key(platform),
                    Command::Provider {
                        session,
                        id: "fixture-provider".into(),
                        patch: Some(patch.into()),
                    },
                )
                .unwrap(),
        )
        .await;
        assert!(
            matches!(
                saved.as_ref(),
                Completion::Success(
                    ResultValue::Saved(ccr_app::platform_settings::SaveOutcome::Saved { .. })
                        | ResultValue::AccountChanged { .. }
                )
            ),
            "provider save failed"
        );
        assert!(
            fixture
                .file(path)
                .unwrap()
                .contains("synthetic-provider-secret")
        );
        let session = fixture.read(platform, Surface::Providers).await;
        assert_eq!(session.provider_ids, vec!["fixture-provider"]);
        assert!(
            !format!("{:?}", session.provider_fields("fixture-provider"))
                .contains("synthetic-provider-secret")
        );
        let deleted = terminal(
            fixture
                .services
                .platform_request(
                    fixture.key(platform),
                    Command::Provider {
                        session,
                        id: "fixture-provider".into(),
                        patch: None,
                    },
                )
                .unwrap(),
        )
        .await;
        assert!(matches!(deleted.as_ref(), Completion::Success(_)));
        assert!(
            fixture
                .read(platform, Surface::Providers)
                .await
                .provider_ids
                .is_empty()
        );
    }
    fixture.set(
        ".ccr/platforms/codex/custom-models.toml",
        "models=['remote-fixture-model']\n",
    );
    let session = fixture.read(Platform::Codex, Surface::Models).await;
    assert!(
        session
            .fields
            .iter()
            .any(|field| field.value == "remote-fixture-model")
    );
    assert!(session.fields.len() > 1);
    assert!(matches!(
        terminal(
            fixture
                .services
                .platform_request(
                    fixture.key(Platform::Codex),
                    Command::Source {
                        session,
                        content: "models=[]".into()
                    }
                )
                .unwrap()
        )
        .await
        .as_ref(),
        Completion::Failed(PlatformError::Unsupported)
    ));
    fixture.runtime.shutdown();
}
