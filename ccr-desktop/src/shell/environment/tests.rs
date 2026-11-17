#![allow(clippy::unwrap_used)]
use super::*;
use ccr_app::{
    environments::ssh::{HostInput, Workspace, fixture::FixtureTransport},
    runtime::RuntimeOwner,
};
use gpui_kit::test::{TestAppContextExt, TestWindowExt};
use gpui_kit::{Bounds, SharedString, TestAppContext, WindowBounds, WindowOptions, size};
use std::{sync::Arc, time::Duration};

#[gpui_kit::test]
async fn ssh_activity_sidebar_platform_tabs_and_missing_cli_install_link_reach_real_owners(
    cx: &mut TestAppContext,
) {
    cx.executor().allow_parking();
    let fixture = Fixture::new(true).await;
    fixture.preferences(true);
    fixture.transport.set_file("build.fixture.invalid","/home/fixture/.claude/projects/project/session.jsonl","{\"type\":\"user\",\"sessionId\":\"ssh-ui\",\"timestamp\":\"2026-11-02T00:00:00Z\",\"message\":{\"role\":\"user\",\"content\":\"remote UI transcript\"}}\n");
    let (handle, shell) = fixture.open(cx);
    ready(cx, handle, &shell).await;
    wait_resource_ui(cx, handle, |window, _| {
        window.try_find("ssh-overview-claude").is_some()
    });
    for (nav, platform) in [
        ("nav-claude-code", "claude"),
        ("nav-codex", "codex"),
        ("nav-grok", "grok"),
        ("nav-antigravity", "gemini"),
        ("nav-opencode", "opencode"),
    ] {
        cx.update_window(handle, |_, window, cx| window.click(nav, cx))
            .unwrap();
        let id = SharedString::from(format!("ssh-overview-{platform}"));
        wait_resource_ui(cx, handle, |window, _| {
            window.try_find(id.clone()).is_some()
        });
        cx.update_window(handle, |_, window, _| {
            assert!(window.try_find("overview-kpis").is_none())
        })
        .unwrap();
    }
    cx.update_window(handle, |_, window, cx| {
        window.click("nav-agent-sessions", cx)
    })
    .unwrap();
    wait_resource_ui(cx, handle, |window, _| {
        window.try_find("session-listbox").is_some()
    });
    cx.update_window(handle, |_, window, cx| {
        assert!(window.try_find("session-open-trash").is_none());
        window.click("nav-claude-code", cx);
        window.click("tool-platform-sessions", cx);
    })
    .unwrap();
    wait_resource_ui(cx, handle, |window, _| {
        window.try_find("session-listbox").is_some()
    });
    cx.update_window(handle, |_, window, cx| window.click("nav-commands", cx))
        .unwrap();
    wait_resource_ui(cx, handle, |window, _| {
        window.try_find("command-foreground").is_some()
    });
    cx.update_window(handle, |_, window, cx| {
        window.click("command-foreground", cx)
    })
    .unwrap();
    wait_resource_ui(cx, handle, |window, _| {
        window.try_find("jobs-install-remote-cli").is_some()
    });
    cx.update_window(handle, |_, window, cx| {
        window.click("jobs-install-remote-cli", cx)
    })
    .unwrap();
    wait_resource_ui(cx, handle, |window, _| {
        window.try_find("install-check").is_some()
    });
    cx.update_window(handle, |_, window, _| {
        assert!(window.try_find("tools-review-update").is_some());
        assert!(window.try_find("tool-install").is_some());
    })
    .unwrap();
    assert!(
        !fixture
            .transport
            .batches()
            .iter()
            .any(|line| line.starts_with("exec ") && !line.starts_with("exec 'python3' '-c'"))
    );
    fixture
        .runtime
        .shutdown_after(fixture.services.disconnect_ssh());
}

struct Fixture {
    root: tempfile::TempDir,
    runtime: RuntimeOwner,
    services: Services,
    workspace: Arc<Workspace>,
    transport: Arc<FixtureTransport>,
    host: Host,
}
async fn action(
    services: &Services,
    environment: EnvironmentId,
    action: SshAction,
) -> Arc<Completion<SshResult, &'static str>> {
    let mut read = services
        .ssh_action(
            RequestKey {
                environment,
                platform: Platform::Claude,
                generation: 1,
                request_id: 1,
            },
            action,
        )
        .unwrap();
    while let Some(event) = read.next().await {
        if let TaskEvent::Terminal(value) = event {
            return value;
        }
    }
    panic!("missing SSH terminal");
}
impl Fixture {
    async fn new(trusted: bool) -> Self {
        let root = tempfile::tempdir().unwrap();
        let runtime = RuntimeOwner::new().unwrap();
        let services = Services::new(
            runtime.handle(),
            crate::runtime::ConfigSource::Fixture {
                profiles: root.path().join("profiles.toml"),
                registry: root.path().join("registry.toml"),
                delay: Duration::ZERO,
            },
        );
        let (workspace, transport) = Workspace::fixture(
            root.path().join("ssh.db"),
            root.path().join("ssh/known_hosts"),
        );
        let workspace = Arc::new(workspace);
        use ccr_app::environments::ssh_security::posix_single_quote as quote;
        let snapshot_command = [
            "python3",
            "-c",
            ccr_usage::snapshot::BACKUP_SCRIPT,
            "/home/fixture/.llmusage",
        ]
        .map(quote)
        .join(" ");
        transport.set_exec_output(&snapshot_command, false, "", "python3: not found");
        services
            .ssh
            .set(workspace.clone())
            .unwrap_or_else(|_| panic!("fixture owner"));
        let saved = action(
            &services,
            EnvironmentId::Local("host-management".into()),
            SshAction::SaveHost(HostInput {
                name: "Build fixture".into(),
                host: "build.fixture.invalid".into(),
                user: "fixture".into(),
                port: 2222,
                remote_home: "/home/fixture".into(),
                ..Default::default()
            }),
        )
        .await;
        let Completion::Success(SshResult::HostSaved(host)) = saved.as_ref() else {
            panic!("save host")
        };
        let host = host.clone();
        if trusted {
            let probe = action(
                &services,
                EnvironmentId::Ssh(host.id.clone()),
                SshAction::Probe(host.id.clone()),
            )
            .await;
            let Completion::Success(SshResult::Probe(probe)) = probe.as_ref() else {
                panic!("probe")
            };
            assert!(matches!(
                action(
                    &services,
                    EnvironmentId::Ssh(host.id.clone()),
                    SshAction::Confirm(probe.challenge_id.clone())
                )
                .await
                .as_ref(),
                Completion::Success(SshResult::Confirmed)
            ));
        }
        Self {
            root,
            runtime,
            services,
            workspace,
            transport,
            host,
        }
    }
    fn preferences(&self, ssh: bool) {
        let path = self.root.path().join("desktop-gpui/preferences.json");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(
            path,
            serde_json::json!({"version":1,"theme":"light","language":"en","confirm_quit":false,
            "environment":if ssh {format!("ssh:{}",self.host.id)}else{"local".into()}})
            .to_string(),
        )
        .unwrap();
    }
    fn open(&self, cx: &mut TestAppContext) -> (gpui_kit::AnyWindowHandle, Entity<Shell>) {
        cx.update(|cx| {
            gpui_kit::init(cx);
            gpui_kit::open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                        None,
                        size(px(1280.), px(800.)),
                        cx,
                    ))),
                    ..Default::default()
                },
                cx,
                |window, cx| cx.new(|cx| Shell::new(self.services.clone(), window, cx)),
            )
            .unwrap()
        })
    }
}
fn show(window: &mut Window, id: &str, cx: &mut gpui_kit::App) {
    for _ in 0..12 {
        window.render_frame(cx);
        let bounds = window
            .find(gpui_kit::SharedString::from(id.to_owned()))
            .bounds();
        let viewport = window.find("settings-body").bounds();
        if bounds.top() >= viewport.top() && bounds.bottom() <= viewport.bottom() {
            return;
        }
        window.scroll(
            "settings-body",
            gpui_kit::ScrollDelta::Pixels(gpui_kit::point(
                px(0.),
                px(if bounds.top() < viewport.top() {
                    240.
                } else {
                    -240.
                }),
            )),
            cx,
        );
    }
    panic!("control outside settings viewport");
}

fn enter_host_field(window: &mut Window, index: usize, value: &str, cx: &mut gpui_kit::App) {
    let id = format!("ssh-host-field-{index}");
    show(window, &id, cx);
    window.click(gpui_kit::SharedString::from(id), cx);
    window.press("ctrl-a", cx);
    window.input(value, cx);
}

async fn open_host_settings(
    cx: &mut TestAppContext,
    handle: gpui_kit::AnyWindowHandle,
    shell: &Entity<Shell>,
) {
    cx.update_window(handle, |_, window, cx| {
        window.click("nav-settings", cx);
        window.click("settings-nav-environment", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        !shell.read(cx).remote.read(cx).is_busy()
    })
    .await;
}

#[gpui_kit::test]
async fn settings_tabs_keep_secret_drafts_until_explicit_discard_and_persist_the_page(
    cx: &mut TestAppContext,
) {
    cx.executor().allow_parking();
    let fixture = Fixture::new(true).await;
    fixture.preferences(false);
    let (handle, shell) = fixture.open(cx);
    ready(cx, handle, &shell).await;
    open_host_settings(cx, handle, &shell).await;
    cx.update_window(handle, |_, window, cx| {
        window.click("ssh-add-host", cx);
        show(window, "ssh-auth-password", cx);
        window.click("ssh-auth-password", cx);
        enter_host_field(window, 7, "synthetic-unsaved-secret", cx);
        window.click("settings-nav-diagnostics", cx);
        window.simulate_next_frame(cx);
        window.render_frame(cx);
        assert_eq!(
            window.find("profile-navigation-cancel").focused(),
            Some(true)
        );
        window.click("profile-navigation-cancel", cx);
        assert!(window.try_find("ssh-host-form").is_some());
        assert!(shell.read(cx).remote.read(cx).has_environment_draft(cx));
        window.click("settings-nav-diagnostics", cx);
        window.click("profile-navigation-discard", cx);
        assert!(window.try_find("settings-page-diagnostics").is_some());
        assert!(window.try_find("ssh-host-form").is_none());
        assert!(!shell.read(cx).remote.read(cx).has_environment_draft(cx));
        window.click("nav-dashboard", cx);
        window.click("nav-settings", cx);
        assert!(window.try_find("settings-page-diagnostics").is_some());
        window.click("environment-switcher", cx);
        window.click("environment-manage", cx);
        window.simulate_next_frame(cx);
        window.render_frame(cx);
        assert!(window.try_find("settings-page-environment").is_some());
    })
    .unwrap();
    let path = fixture.root.path().join("desktop-gpui/preferences.json");
    cx.wait_for(handle, Duration::from_secs(10), |_, _| {
        std::fs::read(&path)
            .ok()
            .and_then(|v| serde_json::from_slice::<serde_json::Value>(&v).ok())
            .is_some_and(|v| v["settings_tab"] == "environment")
    })
    .await;
    cx.update_window(handle, |_, window, _| window.remove_window())
        .unwrap();
    let (reopened, reopened_shell) = fixture.open(cx);
    ready(cx, reopened, &reopened_shell).await;
    cx.update_window(reopened, |_, window, cx| {
        window.click("nav-settings", cx);
        assert!(window.try_find("settings-page-environment").is_some());
        window.remove_window();
    })
    .unwrap();
    fixture
        .runtime
        .shutdown_after(fixture.services.disconnect_ssh());
}

#[gpui_kit::test]
async fn settings_repairs_missing_credentials_and_requires_local_before_current_host_changes(
    cx: &mut TestAppContext,
) {
    cx.executor().allow_parking();
    let fixture = Fixture::new(true).await;
    let mut input = fixture.workspace.host_details(&fixture.host.id).unwrap();
    input.auth_method = ccr_app::environments::ssh::AuthMethod::Password;
    assert!(matches!(
        action(
            &fixture.services,
            EnvironmentId::Local("local".into()),
            SshAction::SaveHost(input)
        )
        .await
        .as_ref(),
        Completion::Success(_)
    ));
    fixture.preferences(true);
    let (handle, shell) = fixture.open(cx);
    ready(cx, handle, &shell).await;
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        shell.read(cx).active_environment.state == ConnectionState::Credentials
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.click("environment-repair-credentials", cx);
        window.simulate_next_frame(cx);
        window.render_frame(cx);
        assert!(window.try_find("settings-page-environment").is_some());
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        !shell.read(cx).remote.read(cx).is_busy()
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.click(
            SharedString::from(format!("ssh-host-{}", fixture.host.id)),
            cx,
        );
        window.click("ssh-edit-host", cx);
        window.click("ssh-delete-host", cx);
        assert!(window.try_find("ssh-confirm-delete").is_none());
        assert!(window.try_find("ssh-host-form").is_none());
        window.click("ssh-local", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        !shell.read(cx).active_environment.is_ssh() && !shell.read(cx).active_environment.switching
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        show(window, "ssh-edit-host", cx);
        window.click("ssh-edit-host", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |window, _| {
        window.try_find("ssh-host-field-7").is_some()
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        enter_host_field(window, 7, "synthetic-repaired-password", cx);
        show(window, "ssh-host-save", cx);
        window.click("ssh-host-save", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        !shell.read(cx).remote.read(cx).is_busy()
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        assert!(!shell.read(cx).remote.read(cx).hosts()[0].credentials_missing);
        assert!(window.try_find("ssh-host-field-7").is_none());
        show(window, "ssh-connect", cx);
        window.click("ssh-connect", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        shell.read(cx).active_environment.state == ConnectionState::Connected
            && shell.read(cx).active_environment.is_ssh()
    })
    .await;
    assert!(fixture.workspace.connected(&fixture.host.id));
    assert_eq!(fixture.transport.handshakes(), 1);
    fixture
        .runtime
        .shutdown_after(fixture.services.disconnect_ssh());
}

#[gpui_kit::test]
async fn settings_create_test_edit_and_delete_another_host_preserves_the_current_session(
    cx: &mut TestAppContext,
) {
    cx.executor().allow_parking();
    let fixture = Fixture::new(true).await;
    fixture.preferences(true);
    let (handle, shell) = fixture.open(cx);
    ready(cx, handle, &shell).await;
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        shell.read(cx).active_environment.state == ConnectionState::Connected
    })
    .await;
    open_host_settings(cx, handle, &shell).await;
    cx.update_window(handle, |_, window, cx| {
        window.click("ssh-add-host", cx);
        for (index, value) in [
            (0, "Deploy fixture"),
            (1, "deploy.fixture.invalid"),
            (2, "2202"),
            (3, "fixture"),
        ] {
            enter_host_field(window, index, value, cx);
        }
        show(window, "ssh-host-test", cx);
        window.click("ssh-host-test", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |window, _| {
        window.try_find("ssh-confirm-trust").is_some()
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        show(window, "ssh-confirm-trust", cx);
        window.click("ssh-confirm-trust", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        !shell.read(cx).remote.read(cx).is_busy()
    })
    .await;
    assert!(fixture.workspace.connected(&fixture.host.id));
    assert_eq!(fixture.transport.handshakes(), 2);
    cx.update_window(handle, |_, window, cx| {
        show(window, "ssh-host-save", cx);
        window.click("ssh-host-save", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        shell.read(cx).remote.read(cx).hosts().len() == 2
            && !shell.read(cx).remote.read(cx).is_busy()
    })
    .await;
    let other = cx.update(|cx| {
        shell
            .read(cx)
            .remote
            .read(cx)
            .hosts()
            .iter()
            .find(|h| h.id != fixture.host.id)
            .unwrap()
            .clone()
    });
    cx.update_window(handle, |_, window, cx| {
        show(window, "ssh-edit-host", cx);
        window.click("ssh-edit-host", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |window, _| {
        window.try_find("ssh-host-form").is_some()
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        enter_host_field(window, 0, "Deploy fixture edited", cx);
        show(window, "ssh-auth-key", cx);
        window.click("ssh-auth-key", cx);
        enter_host_field(window, 6, "C:/synthetic/fixture-key", cx);
        show(window, "ssh-host-save", cx);
        window.click("ssh-host-save", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        !shell.read(cx).remote.read(cx).is_busy()
    })
    .await;
    let details = fixture.workspace.host_details(&other.id).unwrap();
    assert_eq!(
        details.identity_file.as_deref(),
        Some("C:/synthetic/fixture-key")
    );
    assert_eq!(
        details.auth_method,
        ccr_app::environments::ssh::AuthMethod::PrivateKey
    );
    assert!(fixture.workspace.connected(&fixture.host.id));
    cx.update_window(handle, |_, window, cx| {
        show(window, "ssh-delete-host", cx);
        window.click("ssh-delete-host", cx);
        window.simulate_next_frame(cx);
        window.render_frame(cx);
        assert_eq!(window.find("ssh-cancel-delete").focused(), Some(true));
        window.press("tab", cx);
        assert_eq!(window.find("ssh-confirm-delete").focused(), Some(true));
        window.press("escape", cx);
        assert!(window.try_find("ssh-confirm-delete").is_none());
        assert_eq!(window.find("ssh-delete-host").focused(), Some(true));
        window.click("ssh-delete-host", cx);
        window.click("ssh-confirm-delete", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        shell.read(cx).remote.read(cx).hosts().len() == 1
            && !shell.read(cx).remote.read(cx).is_busy()
    })
    .await;
    assert!(fixture.workspace.host_details(&other.id).is_err());
    assert!(fixture.workspace.connected(&fixture.host.id));
    cx.update(|cx| {
        assert_eq!(
            shell.read(cx).active_environment.id,
            EnvironmentId::Ssh(fixture.host.id.clone())
        )
    });
    fixture
        .runtime
        .shutdown_after(fixture.services.disconnect_ssh());
}

#[gpui_kit::test]
async fn queued_host_management_is_drained_before_connecting_the_managed_host(
    cx: &mut TestAppContext,
) {
    cx.executor().allow_parking();
    for delete in [false, true] {
        let fixture = Fixture::new(true).await;
        fixture.preferences(false);
        let (handle, shell) = fixture.open(cx);
        ready(cx, handle, &shell).await;
        open_host_settings(cx, handle, &shell).await;
        cx.update_window(handle, |_, window, cx| {
            window.click(
                SharedString::from(format!("ssh-host-{}", fixture.host.id)),
                cx,
            );
            if delete {
                window.click("ssh-delete-host", cx);
            } else {
                window.click("ssh-edit-host", cx);
            }
        })
        .unwrap();
        if !delete {
            cx.wait_for(handle, Duration::from_secs(10), |window, _| {
                window.try_find("ssh-host-form").is_some()
            })
            .await;
            cx.update_window(handle, |_, window, cx| {
                enter_host_field(window, 0, "Queued stale edit", cx)
            })
            .unwrap();
        }
        let mut releases = Vec::new();
        let mut blockers = Vec::new();
        let (started_tx, started_rx) = std::sync::mpsc::channel();
        for index in 0..ccr_app::runtime::BLOCKING_CONCURRENCY {
            let (release_tx, release_rx) = std::sync::mpsc::channel();
            releases.push(release_tx);
            let started = started_tx.clone();
            blockers.push(
                fixture
                    .services
                    .executor
                    .read(
                        RequestKey {
                            environment: EnvironmentId::Local("isolated-pool-blocker".into()),
                            platform: Platform::Claude,
                            generation: index as u64,
                            request_id: index as u64,
                        },
                        move |_, _| {
                            started.send(()).unwrap();
                            let _ = release_rx.recv_timeout(Duration::from_secs(20));
                            Ok::<_, ()>(())
                        },
                    )
                    .unwrap(),
            );
        }
        for _ in 0..ccr_app::runtime::BLOCKING_CONCURRENCY {
            started_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        }
        cx.update_window(handle, |_, window, cx| {
            if delete {
                window.click("ssh-confirm-delete", cx);
            } else {
                show(window, "ssh-host-save", cx);
                window.click("ssh-host-save", cx);
            }
            assert!(
                fixture
                    .services
                    .executor
                    .environment_has_jobs(&EnvironmentId::Local("local".into())),
                "management work must belong to the environment being drained"
            );
            window.click("environment-switcher", cx);
            window.click(
                SharedString::from(format!("environment-host-{}", fixture.host.id)),
                cx,
            );
            window.click("environment-switch-discard", cx);
            assert!(shell.read(cx).active_environment.switching);
            assert_eq!(fixture.transport.handshakes(), 0);
        })
        .unwrap();
        for release in releases {
            let _ = release.send(());
        }
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
            shell.read(cx).active_environment.state == ConnectionState::Connected
                && shell.read(cx).active_environment.is_ssh()
        })
        .await;
        assert_eq!(
            fixture
                .workspace
                .host_details(&fixture.host.id)
                .unwrap()
                .name,
            "Build fixture"
        );
        assert!(fixture.workspace.connected(&fixture.host.id));
        assert_eq!(fixture.transport.handshakes(), 1);
        cx.update_window(handle, |_, window, _| window.remove_window())
            .unwrap();
        drop(blockers);
        fixture
            .runtime
            .shutdown_after(fixture.services.disconnect_ssh());
    }
}

#[gpui_kit::test]
async fn cancelling_or_discarding_a_saved_host_test_clears_its_challenge_without_disconnect(
    cx: &mut TestAppContext,
) {
    cx.executor().allow_parking();
    let fixture = Fixture::new(true).await;
    let other = action(
        &fixture.services,
        EnvironmentId::Local("local".into()),
        SshAction::SaveHost(HostInput {
            name: "Untrusted fixture".into(),
            host: "untrusted.fixture.invalid".into(),
            user: "fixture".into(),
            port: 22,
            remote_home: "~".into(),
            ..Default::default()
        }),
    )
    .await;
    let Completion::Success(SshResult::HostSaved(other)) = other.as_ref() else {
        panic!("saved fixture")
    };
    fixture.preferences(true);
    let (handle, shell) = fixture.open(cx);
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        shell.read(cx).preferences_loaded
            && shell.read(cx).remote.read(cx).hosts().len() == 2
            && shell.read(cx).active_environment.state == ConnectionState::Connected
    })
    .await;
    open_host_settings(cx, handle, &shell).await;
    cx.update_window(handle, |_, window, cx| {
        show(window, &format!("ssh-host-{}", other.id), cx);
        window.click(SharedString::from(format!("ssh-host-{}", other.id)), cx);
        show(window, "ssh-test-host", cx);
        window.click("ssh-test-host", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |window, _| {
        window.try_find("ssh-reject-trust").is_some()
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        show(window, "ssh-reject-trust", cx);
        window.click("ssh-reject-trust", cx);
        assert!(!shell.read(cx).remote.read(cx).has_environment_draft(cx));
        window.click("settings-nav-diagnostics", cx);
        assert!(window.try_find("settings-page-diagnostics").is_some());
        window.click("settings-nav-environment", cx);
        show(window, "ssh-test-host", cx);
        window.click("ssh-test-host", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |window, _| {
        window.try_find("ssh-reject-trust").is_some()
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.click("settings-nav-diagnostics", cx);
        window.click("profile-navigation-discard", cx);
        assert!(!shell.read(cx).remote.read(cx).has_environment_draft(cx));
        assert!(window.try_find("settings-page-diagnostics").is_some());
        window.click("nav-dashboard", cx);
        assert!(window.try_find("profile-navigation-confirmation").is_none());
    })
    .unwrap();
    assert!(fixture.workspace.connected(&fixture.host.id));
    assert_eq!(fixture.transport.handshakes(), 1);
    fixture
        .runtime
        .shutdown_after(fixture.services.disconnect_ssh());
}
async fn ready(cx: &mut TestAppContext, handle: gpui_kit::AnyWindowHandle, shell: &Entity<Shell>) {
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        shell.read(cx).preferences_loaded && shell.read(cx).remote.read(cx).hosts().len() == 1
    })
    .await;
}

#[gpui_kit::test]
async fn ssh_offline_retained_import_stays_visible_through_navigation_and_cleanup(
    cx: &mut TestAppContext,
) {
    use crate::runtime::insights::{InsightsRequests, import::Status};
    cx.executor().allow_parking();
    let fixture = Fixture::new(true).await;
    fixture.preferences(true);
    fixture
        .transport
        .set_cli_output("llmusage", "/usr/bin/llmusage");
    fixture.transport.set_exec_output(
        "'env' 'RUST_LOG=off' 'NO_COLOR=1' '/usr/bin/llmusage' '--home' '/home/fixture/.llmusage' 'sync' '--json-events' '--recent-days' '30'",
        true,
        "{\"event\":\"progress\",\"source\":\"codex\",\"files_scanned\":1,\"records_imported\":1,\"current_file\":null}\n{\"event\":\"finished\",\"summary\":{\"sources\":1,\"total_seen\":1,\"total_inserted\":1}}\n",
        "",
    );
    fixture
        .transport
        .delay_exec_terminal(Duration::from_secs(30));
    let requests = InsightsRequests::new(&fixture.services);
    let (handle, shell) = fixture.open(cx);
    ready(cx, handle, &shell).await;
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        shell.read(cx).active_environment.state == ConnectionState::Connected
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.click("nav-usage", cx);
        window.click("insights-import", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, _| {
        requests.import_snapshot().status == Status::Running
    })
    .await;
    let cleanup = fixture.transport.hold_disconnect();
    cx.update_window(handle, |_, window, cx| {
        window.click("insights-import-cancel", cx)
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(5), |_, cx| {
        shell.read(cx).active_environment.state == ConnectionState::Unreachable
    })
    .await;
    assert_eq!(requests.import_snapshot().status, Status::CancelRequested);
    let before = fixture.transport.batches();
    let cancelling_visible = cx
        .update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            window.try_find("insights-import-status").is_some()
                && window
                    .find("insights-import-status")
                    .label()
                    .is_some_and(|s| s.contains("Cancellation requested"))
                && window.try_find("environment-retry").is_some()
                && window.try_find("insights-import").is_none()
        })
        .unwrap();
    cx.update_window(handle, |_, window, cx| {
        window.click("nav-settings", cx);
        window.click("nav-usage", cx);
    })
    .unwrap();
    let after_navigation = cx
        .update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            window.try_find("insights-import-status").is_some()
                && window
                    .find("insights-import-status")
                    .label()
                    .is_some_and(|s| s.contains("Cancellation requested"))
        })
        .unwrap();
    assert_eq!(
        fixture.transport.batches(),
        before,
        "retained status must issue no SSH I/O"
    );
    cleanup.notify_one();
    cx.wait_for(handle, Duration::from_secs(10), |_, _| {
        !requests.import_snapshot().status.active()
    })
    .await;
    let terminal_visible = cx
        .update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            window.try_find("insights-import-status").is_some()
                && window
                    .find("insights-import-status")
                    .label()
                    .is_some_and(|s| s.contains("cleanup failed"))
        })
        .unwrap();
    assert_eq!(
        requests.import_snapshot().status,
        Status::Failed(crate::runtime::insights::import::Failure::CleanupFailed)
    );
    let other = EnvironmentId::Ssh("00000000-0000-4000-8000-000000000002".into());
    cx.update_window(handle, |_, window, cx| {
        shell.update(cx, |this, cx| {
            this.request_environment(other.clone(), window, cx)
        });
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        shell.read(cx).active_environment.id == other
            && shell.read(cx).active_environment.state == ConnectionState::Unreachable
    })
    .await;
    let other_hidden = cx
        .update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            window.try_find("insights-import-status").is_none()
        })
        .unwrap();
    cx.update_window(handle, |_, window, _| window.remove_window())
        .unwrap();
    fixture
        .runtime
        .shutdown_after(fixture.services.disconnect_ssh());
    assert!(
        cancelling_visible,
        "offline recovery hid the cancelling import"
    );
    assert!(
        after_navigation,
        "offline navigation hid the retained import"
    );
    assert!(terminal_visible, "offline recovery hid the import terminal");
    assert!(other_hidden, "another host must not see this import");
}

#[gpui_kit::test]
async fn ssh_offline_retained_install_stays_visible_through_navigation_and_cleanup(
    cx: &mut TestAppContext,
) {
    use crate::runtime::install::InstallRequests;
    cx.executor().allow_parking();
    let fixture = Fixture::new(true).await;
    fixture.preferences(true);
    fixture.transport.set_cli_output("cargo", "/usr/bin/cargo");
    fixture
        .transport
        .set_exec_output("'uname' '-s'", true, "Linux\n", "");
    fixture.transport.set_exec_output(
        "'/usr/bin/cargo' 'install' '--locked' 'llmusage'",
        true,
        "installed\n",
        "",
    );
    let requests = InstallRequests::new(&fixture.services);
    let (handle, shell) = fixture.open(cx);
    ready(cx, handle, &shell).await;
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        shell.read(cx).active_environment.state == ConnectionState::Connected
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.click("nav-usage", cx);
        window.click("tool-install", cx);
        window.click("install-check", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(5), |_, cx| {
        let (snapshot, loading) = shell.read(cx).install.read(cx).observed_state();
        snapshot.detection.is_some() && !loading
    })
    .await;
    cx.update_window(handle, |_, window, cx| window.click("install-plan", cx))
        .unwrap();
    cx.wait_for(handle, Duration::from_secs(5), |_, cx| {
        let (snapshot, loading) = shell.read(cx).install.read(cx).observed_state();
        snapshot.plan.is_some() && !loading
    })
    .await;
    // 只使用现有 transport 暂停点，放行执行前复核，保留真实安装 exec。
    let transport = fixture.transport.clone();
    let gate = transport.hold_next_exec();
    let mut count = transport
        .batches()
        .iter()
        .filter(|s| s.starts_with("exec "))
        .count();
    let (held_tx, held_rx) = std::sync::mpsc::channel();
    let controller = std::thread::spawn(move || {
        let mut gate = gate;
        let deadline = std::time::Instant::now() + Duration::from_secs(10);
        while std::time::Instant::now() < deadline {
            let batches = transport.batches();
            let commands = batches
                .iter()
                .filter(|s| s.starts_with("exec "))
                .collect::<Vec<_>>();
            if commands.len() > count {
                count = commands.len();
                if commands
                    .last()
                    .unwrap()
                    .contains("'/usr/bin/cargo' 'install'")
                {
                    held_tx.send(gate).unwrap();
                    return;
                }
                std::thread::sleep(Duration::from_millis(2));
                let next = transport.hold_next_exec();
                gate.notify_one();
                gate = next;
            }
            std::thread::sleep(Duration::from_millis(2));
        }
        gate.notify_one();
        panic!("fixture installation did not reach exec");
    });
    cx.update_window(handle, |_, window, cx| {
        window.click("install-review", cx);
        window.simulate_next_frame(cx);
        window.render_frame(cx);
        window.click("install-decision-confirm", cx);
    })
    .unwrap();
    let mut cargo = None;
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        cargo = held_rx.try_recv().ok().or(cargo.take());
        cargo.is_some() && shell.read(cx).install.read(cx).observed_state().0.running
    })
    .await;
    controller.join().unwrap();
    let cleanup = fixture.transport.hold_disconnect();
    cx.update_window(handle, |_, window, cx| window.click("install-cancel", cx))
        .unwrap();
    cx.wait_for(handle, Duration::from_secs(5), |_, cx| {
        shell.read(cx).active_environment.state == ConnectionState::Unreachable
    })
    .await;
    assert!(requests.snapshot().busy() && requests.snapshot().cancel_requested);
    let before = fixture.transport.batches();
    let cancelling_visible = cx
        .update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            window.try_find("install-status").is_some()
                && window.find("install-status").label()
                    == Some("Cancellation requested; waiting for cleanup")
                && window.try_find("environment-retry").is_some()
                && window.try_find("install-review").is_none()
        })
        .unwrap();
    cx.update_window(handle, |_, window, cx| {
        window.click("nav-settings", cx);
        window.click("nav-usage", cx);
        window.click("tool-install", cx);
    })
    .unwrap();
    let after_navigation = cx
        .update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            window.try_find("install-status").is_some()
                && window.find("install-status").label()
                    == Some("Cancellation requested; waiting for cleanup")
        })
        .unwrap();
    assert_eq!(
        fixture.transport.batches(),
        before,
        "retained status must issue no SSH I/O"
    );
    cleanup.notify_one();
    cargo.unwrap().notify_one();
    cx.wait_for(handle, Duration::from_secs(10), |_, _| {
        !requests.snapshot().busy()
    })
    .await;
    let terminal_visible = cx
        .update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            window.try_find("install-status").is_some()
                && window.find("install-status").label() == Some("Failed")
        })
        .unwrap();
    assert!(matches!(
        requests.snapshot().recent.terminal,
        Some(ccr_cli::services::install_types::InstallEvent::Failed { .. })
    ));
    let other = EnvironmentId::Ssh("00000000-0000-4000-8000-000000000002".into());
    cx.update_window(handle, |_, window, cx| {
        shell.update(cx, |this, cx| {
            this.request_environment(other.clone(), window, cx)
        });
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        shell.read(cx).active_environment.id == other
            && shell.read(cx).active_environment.state == ConnectionState::Unreachable
    })
    .await;
    let other_hidden = cx
        .update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            window.try_find("install-status").is_none()
        })
        .unwrap();
    cx.update_window(handle, |_, window, _| window.remove_window())
        .unwrap();
    fixture
        .runtime
        .shutdown_after(fixture.services.disconnect_ssh());
    assert!(
        cancelling_visible,
        "offline recovery hid the cancelling installation"
    );
    assert!(
        after_navigation,
        "offline navigation hid the retained installation"
    );
    assert!(
        terminal_visible,
        "offline recovery hid the installation terminal"
    );
    assert!(other_hidden, "another host must not see this installation");
}

#[gpui_kit::test]
async fn ssh_configuration_tabs_load_remote_data_while_later_capabilities_stay_closed(
    cx: &mut TestAppContext,
) {
    cx.executor().allow_parking();
    let fixture = Fixture::new(true).await;
    fixture.preferences(true);
    fixture.transport.set_file(
        "build.fixture.invalid",
        "/home/fixture/.claude/settings.json",
        "{\"model\":\"remote-fixture\"}",
    );
    let (handle, shell) = fixture.open(cx);
    ready(cx, handle, &shell).await;
    wait_resource_ui(cx, handle, |_, cx| {
        shell.read(cx).active_environment.state == ConnectionState::Connected
    });
    cx.update_window(handle, |_, window, cx| {
        window.click("nav-claude-code", cx);
        window.click("tool-settings", cx);
    })
    .unwrap();
    wait_resource_ui(cx, handle, |window, _| {
        window.try_find("platform-source-open").is_some()
    });
    assert!(
        fixture
            .transport
            .batches()
            .iter()
            .any(|item| item == "get /home/fixture/.claude/settings.json")
    );
    cx.update_window(handle, |_, window, cx| {
        assert!(window.try_find("platform-source-open").is_some());
        window.click("tool-files", cx);
        assert!(window.try_find("platform-source-open").is_none());
    })
    .unwrap();
    wait_resource_ui(cx, handle, |window, _| {
        window
            .try_find("resource-open-claude-user-memory")
            .is_some()
    });
    cx.update_window(handle, |_, window, cx| {
        window.click("tool-auth", cx);
    })
    .unwrap();
    wait_resource_ui(cx, handle, |window, _| {
        window.try_find("platform-account-save").is_some()
    });
    cx.update_window(handle, |_, window, cx| {
        window.click("tool-profiles", cx);
    })
    .unwrap();
    wait_resource_ui(cx, handle, |window, _| {
        window.try_find("profile-create").is_some()
    });
    cx.update_window(handle, |_, window, cx| {
        for id in ["profile-import", "profile-backups", "profile-restore"] {
            assert!(window.try_find(id).is_none());
        }
        window.click("tool-settings", cx);
    })
    .unwrap();
    wait_resource_ui(cx, handle, |window, _| {
        window.try_find("platform-source-open").is_some()
    });
    action(
        &fixture.services,
        EnvironmentId::Ssh(fixture.host.id.clone()),
        SshAction::Disconnect,
    )
    .await;
    wait_resource_ui(cx, handle, |_, cx| {
        shell.read(cx).active_environment.state == ConnectionState::Unreachable
    });
    cx.update_window(handle, |_, window, _| {
        assert!(window.try_find("platform-source-open").is_none());
        assert!(window.try_find("environment-return-local").is_some());
    })
    .unwrap();
    cx.update_window(handle, |_, window, cx| {
        window.click("tool-profiles", cx);
        window.click("profile-history", cx);
    })
    .unwrap();
    wait_resource_ui(cx, handle, |window, _| {
        window.try_find("configuration-history").is_some()
    });
    wait_resource_ui(cx, handle, |_, cx| {
        !shell.read(cx).configurations.read(cx).history_busy(cx)
    });
    let before = fixture.transport.batches();
    cx.update_window(handle, |_, window, cx| {
        assert!(window.try_find("environment-return-local").is_none());
        window.click("configuration-history-back", cx);
    })
    .unwrap();
    wait_resource_ui(cx, handle, |window, _| {
        window.try_find("configuration-history").is_none()
            && window.try_find("environment-return-local").is_some()
    });
    cx.update_window(handle, |_, window, _| {
        assert!(window.try_find("configuration-history").is_none());
        assert!(window.try_find("environment-return-local").is_some());
        assert!(window.try_find("profile-create").is_none());
    })
    .unwrap();
    assert_eq!(fixture.transport.batches(), before);
    fixture
        .runtime
        .shutdown_after(fixture.services.disconnect_ssh());
}

#[gpui_kit::test]
async fn application_history_control_is_available_before_a_first_successful_ssh_connection(
    cx: &mut TestAppContext,
) {
    cx.executor().allow_parking();
    let fixture = Fixture::new(true).await;
    fixture.preferences(true);
    fixture.transport.fail("agent");
    let (handle, shell) = fixture.open(cx);
    ready(cx, handle, &shell).await;
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        shell.read(cx).active_environment.state == ConnectionState::Unreachable
    })
    .await;
    assert_eq!(fixture.transport.handshakes(), 0);
    let before = fixture.transport.batches();
    cx.update_window(handle, |_, window, cx| {
        window.click("nav-claude-code", cx);
        window.click("tool-profiles", cx);
        window.click("profile-history", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |window, _| {
        window.try_find("configuration-history").is_some()
    })
    .await;
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        !shell.read(cx).configurations.read(cx).history_busy(cx)
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.click("configuration-history-back", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |window, _| {
        window.try_find("configuration-history").is_none()
            && window.try_find("environment-return-local").is_some()
    })
    .await;
    cx.update_window(handle, |_, window, _| {
        assert!(window.try_find("environment-return-local").is_some());
        assert!(window.try_find("profile-create").is_none());
    })
    .unwrap();
    assert_eq!(fixture.transport.batches(), before);
    fixture
        .runtime
        .shutdown_after(fixture.services.disconnect_ssh());
}

#[gpui_kit::test]
async fn delayed_preference_resolution_never_starts_or_renders_local_destination_reads(
    cx: &mut TestAppContext,
) {
    cx.executor().allow_parking();
    let fixture = Fixture::new(true).await;
    fixture.preferences(true);
    fixture.transport.fail("agent");
    let gate = Arc::new(std::sync::Barrier::new(
        ccr_app::runtime::BLOCKING_CONCURRENCY + 1,
    ));
    let (started_tx, started_rx) = std::sync::mpsc::channel();
    let mut subscriptions = Vec::new();
    for id in 0..ccr_app::runtime::BLOCKING_CONCURRENCY {
        let gate = gate.clone();
        let started = started_tx.clone();
        subscriptions.push(
            fixture
                .services
                .executor
                .read(
                    RequestKey {
                        environment: EnvironmentId::Local("startup-delay".into()),
                        platform: Platform::Claude,
                        generation: id as u64,
                        request_id: id as u64,
                    },
                    move |_, _| {
                        started.send(()).unwrap();
                        gate.wait();
                        Ok::<_, ()>(())
                    },
                )
                .unwrap(),
        );
    }
    for _ in 0..ccr_app::runtime::BLOCKING_CONCURRENCY {
        started_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    }
    let (handle, shell) = fixture.open(cx);
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert!(!shell.read(cx).preferences_loaded);
        assert_eq!(shell.read(cx).configurations.read(cx).read_generation(), 0);
        assert!(window.try_find("overview-refresh").is_none());
        window.click("nav-claude-code", cx);
        assert_eq!(
            shell.read(cx).configurations.read(cx).read_generation(),
            1,
            "navigation only invalidates; it cannot start a local read before preferences"
        );
        assert!(
            window
                .find("shell-status")
                .label()
                .unwrap()
                .contains("Loading execution environment")
        );
    })
    .unwrap();
    gate.wait();
    ready(cx, handle, &shell).await;
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        shell.read(cx).active_environment.state == ConnectionState::Unreachable
    })
    .await;
    cx.update_window(handle, |_, window, _| {
        assert!(window.try_find("environment-return-local").is_some())
    })
    .unwrap();
    drop(subscriptions);
    fixture
        .runtime
        .shutdown_after(fixture.services.disconnect_ssh());
}
async fn selected(cx: &mut TestAppContext, handle: gpui_kit::AnyWindowHandle, host: &Host) {
    let id = gpui_kit::SharedString::from(format!("environment-host-{}", host.id));
    cx.update_window(handle, |_, window, cx| {
        window.click("environment-switcher", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |window, _| {
        window.try_find(id.clone()).is_some()
    })
    .await;
    cx.update_window(handle, |_, window, cx| window.click(id, cx))
        .unwrap();
}

#[gpui_kit::test]
async fn persisted_ssh_starts_once_stays_on_failure_and_retries_without_local_data(
    cx: &mut TestAppContext,
) {
    cx.executor().allow_parking();
    let fixture = Fixture::new(true).await;
    fixture.preferences(true);
    fixture.transport.fail("agent");
    let (handle, shell) = fixture.open(cx);
    ready(cx, handle, &shell).await;
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        shell.read(cx).active_environment.state == ConnectionState::Unreachable
    })
    .await;
    assert_eq!(fixture.transport.handshakes(), 0);
    cx.update_window(handle, |_, window, cx| {
        assert!(window.try_find("overview-refresh").is_none());
        assert!(window.try_find("environment-return-local").is_some());
        assert!(
            shell.read(cx).active_environment.id == EnvironmentId::Ssh(fixture.host.id.clone())
        );
        window.click("nav-checkin", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |window, _| {
        window.try_find("checkin-absent").is_some()
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.click("nav-commands", cx);
        window.click("tool-recent", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |window, _| {
        window.try_find("remote-open-settings").is_none()
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.click("nav-settings", cx);
        window.click("theme-dark", cx);
        window.click("nav-dashboard", cx);
    })
    .unwrap();
    cx.executor().advance_clock(Duration::from_secs(4));
    cx.run_until_parked();
    assert_eq!(
        fixture.transport.handshakes(),
        0,
        "navigation and preference saves never reconnect"
    );
    cx.update_window(handle, |_, window, cx| {
        window.click("environment-retry", cx)
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        shell.read(cx).active_environment.state == ConnectionState::Connected
    })
    .await;
    assert_eq!(fixture.transport.handshakes(), 1);
    assert!(fixture.workspace.connected(&fixture.host.id));
    let prefs = fixture.root.path().join("desktop-gpui/preferences.json");
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        !shell.read(cx).preferences_busy
    })
    .await;
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&std::fs::read_to_string(prefs).unwrap())
            .unwrap()["environment"],
        format!("ssh:{}", fixture.host.id)
    );
    cx.update_window(handle, |_, window, _| window.remove_window())
        .unwrap();
    let (reopened, reloaded) = fixture.open(cx);
    ready(cx, reopened, &reloaded).await;
    cx.wait_for(reopened, Duration::from_secs(10), |_, cx| {
        reloaded.read(cx).active_environment.state == ConnectionState::Connected
    })
    .await;
    assert_eq!(
        fixture.transport.handshakes(),
        2,
        "restart connects exactly once"
    );
    cx.update(|cx| {
        assert!(
            reloaded.read(cx).active_environment.id == EnvironmentId::Ssh(fixture.host.id.clone())
        );
        assert!(reloaded.read(cx).preference == Preference::Dark);
    });
    fixture
        .runtime
        .shutdown_after(fixture.services.disconnect_ssh());
}

#[gpui_kit::test]
async fn missing_credentials_and_disconnection_stay_ssh_without_reconnect(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let fixture = Fixture::new(true).await;
    let mut host = fixture.workspace.host_details(&fixture.host.id).unwrap();
    host.auth_method = ccr_app::environments::ssh::AuthMethod::Password;
    assert!(matches!(
        action(
            &fixture.services,
            EnvironmentId::Local("host-management".into()),
            SshAction::SaveHost(host)
        )
        .await
        .as_ref(),
        Completion::Success(_)
    ));
    fixture.preferences(true);
    let (handle, shell) = fixture.open(cx);
    ready(cx, handle, &shell).await;
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        shell.read(cx).active_environment.state == ConnectionState::Credentials
    })
    .await;
    cx.update_window(handle, |_, window, _| {
        assert!(window.try_find("environment-repair-credentials").is_some());
        assert!(window.try_find("overview-refresh").is_none());
    })
    .unwrap();
    let mut host = fixture.workspace.host_details(&fixture.host.id).unwrap();
    host.auth_method = ccr_app::environments::ssh::AuthMethod::Agent;
    action(
        &fixture.services,
        EnvironmentId::Local("host-management".into()),
        SshAction::SaveHost(host),
    )
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.click("environment-retry", cx)
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        shell.read(cx).active_environment.state == ConnectionState::Connected
    })
    .await;
    action(
        &fixture.services,
        EnvironmentId::Ssh(fixture.host.id.clone()),
        SshAction::Disconnect,
    )
    .await;
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        shell.read(cx).active_environment.state == ConnectionState::Unreachable
    })
    .await;
    assert_eq!(fixture.transport.handshakes(), 1);
    cx.update(|cx| {
        assert!(shell.read(cx).active_environment.id == EnvironmentId::Ssh(fixture.host.id.clone()))
    });
    fixture
        .runtime
        .shutdown_after(fixture.services.disconnect_ssh());
}

#[gpui_kit::test]
async fn switch_confirmation_waits_for_job_cleanup_before_new_connection(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let fixture = Fixture::new(true).await;
    fixture.preferences(false);
    let (handle, shell) = fixture.open(cx);
    ready(cx, handle, &shell).await;
    let (entered_tx, entered_rx) = std::sync::mpsc::channel();
    let (release_tx, release_rx) = std::sync::mpsc::channel();
    let mut job = fixture
        .services
        .executor
        .job(
            RequestKey {
                environment: fixture.services.environment(),
                platform: Platform::Claude,
                generation: 1,
                request_id: 1,
            },
            move |cancel, _| {
                entered_tx.send(()).unwrap();
                while !cancel.is_cancelled() {
                    std::thread::sleep(Duration::from_millis(1));
                }
                release_rx.recv().unwrap();
                Completion::<(), ()>::Cancelled
            },
        )
        .unwrap();
    entered_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    selected(cx, handle, &fixture.host).await;
    cx.wait_for(handle, Duration::from_secs(10), |window, _| {
        window.try_find("environment-switch-discard").is_some()
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.click("environment-switch-discard", cx)
    })
    .unwrap();
    cx.update(|cx| {
        assert!(shell.read(cx).active_environment.switching);
        assert_eq!(
            fixture.transport.handshakes(),
            0,
            "new session waits for old process cleanup"
        );
    });
    release_tx.send(()).unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        shell.read(cx).active_environment.state == ConnectionState::Connected
    })
    .await;
    assert!(matches!(job.next().await,Some(TaskEvent::Terminal(v)) if *v==Completion::Cancelled));
    fixture
        .runtime
        .shutdown_after(fixture.services.disconnect_ssh());
}

#[gpui_kit::test]
async fn disconnect_failure_keeps_previous_ssh_and_requires_an_explicit_retry(
    cx: &mut TestAppContext,
) {
    cx.executor().allow_parking();
    let fixture = Fixture::new(true).await;
    fixture.preferences(true);
    let (handle, shell) = fixture.open(cx);
    ready(cx, handle, &shell).await;
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        shell.read(cx).active_environment.state == ConnectionState::Connected
    })
    .await;
    fixture.transport.fail("disconnect");
    cx.update_window(handle, |_, window, cx| {
        window.click("environment-switcher", cx);
        window.click("environment-local", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        shell.read(cx).active_environment.message == Some("ssh_cleanup_failed")
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        assert!(
            shell.read(cx).active_environment.id == EnvironmentId::Ssh(fixture.host.id.clone())
        );
        assert!(window.try_find("overview-refresh").is_none());
        assert!(window.try_find("environment-return-local").is_some());
    })
    .unwrap();
    assert_eq!(
        fixture.transport.handshakes(),
        1,
        "cleanup failure cannot start another connection"
    );
    assert_eq!(
        fixture.workspace.retained_session_count(),
        0,
        "old authority is still revoked"
    );
    let saved: serde_json::Value = serde_json::from_slice(
        &std::fs::read(fixture.root.path().join("desktop-gpui/preferences.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(saved["environment"], format!("ssh:{}", fixture.host.id));
    cx.update_window(handle, |_, window, cx| {
        window.click("environment-return-local", cx)
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        !shell.read(cx).active_environment.is_ssh() && !shell.read(cx).active_environment.switching
    })
    .await;
    fixture
        .runtime
        .shutdown_after(fixture.services.disconnect_ssh());
}

#[gpui_kit::test]
async fn switcher_trust_cancel_confirm_and_changed_key_use_native_controls(
    cx: &mut TestAppContext,
) {
    cx.executor().allow_parking();
    let fixture = Fixture::new(false).await;
    fixture.preferences(false);
    let (handle, shell) = fixture.open(cx);
    ready(cx, handle, &shell).await;
    selected(cx, handle, &fixture.host).await;
    cx.wait_for(handle, Duration::from_secs(10), |window, _| {
        window.try_find("environment-trust-confirmation").is_some()
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.simulate_next_frame(cx);
        window.render_frame(cx);
        assert_eq!(
            window.find("environment-trust-cancel").focused(),
            Some(true)
        );
        window.press("shift-tab", cx);
        assert_eq!(
            window.find("environment-trust-confirm").focused(),
            Some(true)
        );
        window.press("tab", cx);
        assert_eq!(
            window.find("environment-trust-cancel").focused(),
            Some(true)
        );
    })
    .unwrap();
    cx.update_window(handle, |_, window, cx| {
        assert!(!fixture.root.path().join("ssh/known_hosts").exists());
        window.click("environment-trust-cancel", cx);
        assert!(
            shell.read(cx).active_environment.id == EnvironmentId::Ssh(fixture.host.id.clone())
        );
        window.click("environment-retry", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |window, _| {
        window.try_find("environment-trust-confirm").is_some()
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.click("environment-trust-confirm", cx)
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        shell.read(cx).active_environment.state == ConnectionState::Connected
    })
    .await;
    let trusted = std::fs::read(fixture.root.path().join("ssh/known_hosts")).unwrap();
    fixture.transport.change_key();
    cx.update_window(handle, |_, window, cx| {
        window.click("environment-switcher", cx);
        window.click("environment-reconnect", cx)
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        shell.read(cx).active_environment.message == Some("ssh_host_key_mismatch")
    })
    .await;
    cx.update_window(handle, |_, window, _| {
        assert!(window.try_find("environment-trust-confirm").is_none())
    })
    .unwrap();
    assert_eq!(
        trusted,
        std::fs::read(fixture.root.path().join("ssh/known_hosts")).unwrap()
    );
    fixture
        .runtime
        .shutdown_after(fixture.services.disconnect_ssh());
}

#[gpui_kit::test]
async fn host_form_drafts_require_stay_or_discard_and_a_b_a_invalidates_reads(
    cx: &mut TestAppContext,
) {
    cx.executor().allow_parking();
    let fixture = Fixture::new(true).await;
    fixture.preferences(false);
    let (handle, shell) = fixture.open(cx);
    ready(cx, handle, &shell).await;
    cx.update_window(handle, |_, window, cx| {
        window.click("nav-settings", cx);
        window.click("settings-nav-environment", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |window, _| {
        window.try_find("ssh-add-host").is_some()
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        show(window, "ssh-add-host", cx);
        window.click("ssh-add-host", cx);
    })
    .unwrap();
    selected(cx, handle, &fixture.host).await;
    cx.wait_for(handle, Duration::from_secs(10), |window, _| {
        window.try_find("environment-switch-stay").is_some()
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        assert_eq!(
            window.find("environment-switch-target").label(),
            Some("Local → Build fixture")
        );
        window.click("environment-switch-stay", cx);
        assert!(shell.read(cx).active_environment.id == EnvironmentId::Local("local".into()));
        assert!(shell.read(cx).remote.read(cx).has_environment_draft(cx));
    })
    .unwrap();
    selected(cx, handle, &fixture.host).await;
    cx.update_window(handle, |_, window, cx| {
        window.click("environment-switch-discard", cx)
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        shell.read(cx).active_environment.state == ConnectionState::Connected
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        assert!(!shell.read(cx).remote.read(cx).has_environment_draft(cx));
        window.click("environment-switcher", cx);
        window.click("environment-local", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        shell.read(cx).active_environment.id == EnvironmentId::Local("local".into())
            && !shell.read(cx).active_environment.switching
    })
    .await;
    let before = cx.update(|cx| shell.read(cx).configurations.read(cx).read_generation());
    let old_focus_generation = cx.update(|cx| shell.read(cx).navigation_generation);
    // 再次切换同一主机并返回，旧本地 generation 不能复活。
    selected(cx, handle, &fixture.host).await;
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        shell.read(cx).active_environment.state == ConnectionState::Connected
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.click("environment-switcher", cx);
        window.click("environment-local", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        !shell.read(cx).active_environment.is_ssh() && !shell.read(cx).active_environment.switching
    })
    .await;
    cx.update(|cx| {
        assert!(shell.read(cx).configurations.read(cx).read_generation() > before);
        assert!(
            shell.read(cx).navigation_generation > old_focus_generation,
            "A→B→A invalidates delayed navigation focus as well as data"
        );
    });
    fixture
        .runtime
        .shutdown_after(fixture.services.disconnect_ssh());
}

#[gpui_kit::test]
async fn settings_host_selection_waits_for_delayed_preference_resolution(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let fixture = Fixture::new(true).await;
    fixture.preferences(true);
    fixture.transport.fail("agent");
    let before = std::fs::read(fixture.root.path().join("desktop-gpui/preferences.json")).unwrap();
    let (handle, shell) = cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
            cx.new(|cx| {
                Shell::new_with_preferences(
                    fixture.services.clone(),
                    Err(crate::runtime::preferences::PreferencesError::Storage),
                    window,
                    cx,
                )
            })
        })
        .unwrap()
    });
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        shell.read(cx).remote.read(cx).hosts().len() == 1
    })
    .await;
    let gate = Arc::new(std::sync::Barrier::new(
        ccr_app::runtime::BLOCKING_CONCURRENCY + 1,
    ));
    let (tx, rx) = std::sync::mpsc::channel();
    let mut held = Vec::new();
    for id in 0..ccr_app::runtime::BLOCKING_CONCURRENCY {
        let gate = gate.clone();
        let tx = tx.clone();
        held.push(
            fixture
                .services
                .executor
                .read(
                    RequestKey {
                        environment: EnvironmentId::Local("delay-preferences".into()),
                        platform: Platform::Claude,
                        generation: id as u64,
                        request_id: id as u64,
                    },
                    move |_, _| {
                        tx.send(()).unwrap();
                        gate.wait();
                        Ok::<_, ()>(())
                    },
                )
                .unwrap(),
        );
    }
    for _ in 0..ccr_app::runtime::BLOCKING_CONCURRENCY {
        rx.recv_timeout(Duration::from_secs(5)).unwrap();
    }
    cx.update_window(handle, |_, window, cx| {
        shell.update(cx, |this, cx| this.load_preferences(window, cx));
        window.click("nav-settings", cx);
        window.click("settings-nav-environment", cx);
        let host_id = format!("ssh-host-{}", fixture.host.id);
        show(window, &host_id, cx);
        window.click(gpui_kit::SharedString::from(host_id), cx);
        show(window, "ssh-connect", cx);
        window.click("ssh-connect", cx);
        window.render_frame(cx);
        assert!(!shell.read(cx).preferences_loaded);
        assert!(!shell.read(cx).active_environment.switching);
        assert_eq!(shell.read(cx).active_environment.generation, 0);
        assert!(fixture.services.environment() == EnvironmentId::Local("local".into()));
    })
    .unwrap();
    assert_eq!(
        std::fs::read(fixture.root.path().join("desktop-gpui/preferences.json")).unwrap(),
        before
    );
    gate.wait();
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        shell.read(cx).active_environment.state == ConnectionState::Unreachable
    })
    .await;
    cx.update(|cx| {
        assert!(shell.read(cx).active_environment.id == EnvironmentId::Ssh(fixture.host.id.clone()))
    });
    assert_eq!(
        fixture
            .transport
            .batches()
            .iter()
            .filter(|line| line.starts_with("agent "))
            .count(),
        1
    );
    drop(held);
    fixture
        .runtime
        .shutdown_after(fixture.services.disconnect_ssh());
}

#[gpui_kit::test]
async fn long_host_name_keeps_status_visible_at_the_supported_minimum_width(
    cx: &mut TestAppContext,
) {
    cx.executor().allow_parking();
    let fixture = Fixture::new(false).await;
    let mut host = fixture.workspace.host_details(&fixture.host.id).unwrap();
    host.name = "Build fixture ".repeat(9).trim().to_owned();
    assert!(matches!(
        action(
            &fixture.services,
            EnvironmentId::Local("host-management".into()),
            SshAction::SaveHost(host.clone()),
        )
        .await
        .as_ref(),
        Completion::Success(SshResult::HostSaved(_))
    ));
    fixture.preferences(true);
    let (handle, shell) = cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(960.), px(800.)),
                    cx,
                ))),
                ..Default::default()
            },
            cx,
            |window, cx| cx.new(|cx| Shell::new(fixture.services.clone(), window, cx)),
        )
        .unwrap()
    });
    ready(cx, handle, &shell).await;
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        shell.read(cx).active_environment.state == ConnectionState::Trust
    })
    .await;
    cx.update_window(handle, |_, window, _| {
        let control = window.find("environment-switcher").bounds();
        let name = window.find("environment-current-name").bounds();
        let state = window.find("environment-current-state").bounds();
        assert!(name.right() <= state.left());
        assert!(state.right() <= control.right());
        assert!(state.bottom() <= window.find("shell-status").bounds().bottom());
        assert_eq!(
            window.find("environment-current-state").label(),
            Some("Host key approval required")
        );
        assert!(
            window
                .find("environment-switcher")
                .label()
                .unwrap()
                .contains(&host.name)
        );
        assert!(
            window
                .find("shell-status")
                .label()
                .unwrap()
                .contains("Demo data")
        );
    })
    .unwrap();
    fixture
        .runtime
        .shutdown_after(fixture.services.disconnect_ssh());
}

#[gpui_kit::test]
async fn reconnect_cancels_queued_old_authority_before_a_new_session(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let fixture = Fixture::new(true).await;
    fixture.preferences(true);
    let (handle, shell) = fixture.open(cx);
    ready(cx, handle, &shell).await;
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        shell.read(cx).active_environment.state == ConnectionState::Connected
    })
    .await;
    open_host_settings(cx, handle, &shell).await;
    let gate = Arc::new(std::sync::Barrier::new(
        ccr_app::runtime::BLOCKING_CONCURRENCY + 1,
    ));
    let (tx, rx) = std::sync::mpsc::channel();
    let mut held = Vec::new();
    for id in 0..ccr_app::runtime::BLOCKING_CONCURRENCY {
        let gate = gate.clone();
        let tx = tx.clone();
        held.push(
            fixture
                .services
                .executor
                .read(
                    RequestKey {
                        environment: EnvironmentId::Local("hold-workers".into()),
                        platform: Platform::Claude,
                        generation: id as u64,
                        request_id: id as u64,
                    },
                    move |_, _| {
                        tx.send(()).unwrap();
                        gate.wait();
                        Ok::<_, ()>(())
                    },
                )
                .unwrap(),
        );
    }
    for _ in 0..ccr_app::runtime::BLOCKING_CONCURRENCY {
        rx.recv_timeout(Duration::from_secs(5)).unwrap();
    }
    let before = fixture.transport.cli_requests();
    let mut old = fixture
        .services
        .ssh_action(
            RequestKey {
                environment: EnvironmentId::Ssh(fixture.host.id.clone()),
                platform: Platform::Claude,
                generation: 10,
                request_id: 10,
            },
            SshAction::DetectCli(fixture.host.id.clone()),
        )
        .unwrap();
    cx.update_window(handle, |_, window, cx| {
        window.click("environment-switcher", cx);
        window.click("environment-reconnect", cx)
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |window, _| {
        window.try_find("environment-switch-discard").is_some()
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.click("environment-switch-discard", cx)
    })
    .unwrap();
    assert!(
        matches!(old.next().await,Some(TaskEvent::Terminal(value)) if matches!(value.as_ref(),Completion::Cancelled))
    );
    assert_eq!(fixture.transport.handshakes(), 1);
    assert_eq!(fixture.transport.cli_requests(), before);
    gate.wait();
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        shell.read(cx).active_environment.state == ConnectionState::Connected
    })
    .await;
    assert_eq!(fixture.transport.handshakes(), 2);
    assert!(
        fixture.transport.cli_requests() == before,
        "a queued old capability never retargets the new session"
    );
    drop(held);
    fixture
        .runtime
        .shutdown_after(fixture.services.disconnect_ssh());
}

#[gpui_kit::test]
async fn keyboard_menu_selects_hosts_esc_restores_trigger_and_outside_or_navigation_dismisses(
    cx: &mut TestAppContext,
) {
    cx.executor().allow_parking();
    let fixture = Fixture::new(true).await;
    fixture.preferences(false);
    let (handle, shell) = fixture.open(cx);
    ready(cx, handle, &shell).await;
    let host_id = gpui_kit::SharedString::from(format!("environment-host-{}", fixture.host.id));
    cx.update_window(handle, |_, window, cx| {
        window.click("nav-dashboard", cx);
        for _ in 0..3 {
            if window.find("shell-status").focused() == Some(true) {
                break;
            }
            window.press("f6", cx);
        }
        assert_eq!(window.find("shell-status").focused(), Some(true));
        window.press("tab", cx);
        assert_eq!(window.find("environment-switcher").focused(), Some(true));
        window.press("enter", cx);
        window.simulate_next_frame(cx);
        window.render_frame(cx);
        assert_eq!(
            window.find("environment-menu").role(),
            Some(gpui_kit::Role::Menu)
        );
        assert_eq!(
            window.find("environment-local").role(),
            Some(gpui_kit::Role::MenuItem)
        );
        assert_eq!(window.find("environment-local").focused(), Some(true));
        window.press("down", cx);
        assert_eq!(window.find(host_id.clone()).focused(), Some(true));
        assert_eq!(
            window.find(host_id.clone()).role(),
            Some(gpui_kit::Role::MenuItem)
        );
        window.press("escape", cx);
        assert!(window.try_find("environment-menu").is_none());
        assert_eq!(window.find("environment-switcher").focused(), Some(true));
        window.press("enter", cx);
        window.simulate_next_frame(cx);
        window.render_frame(cx);
        window.press("end", cx);
        assert_eq!(window.find("environment-manage").focused(), Some(true));
        window.press("tab", cx);
        assert_eq!(window.find("environment-local").focused(), Some(true));
        window.press("up", cx);
        assert_eq!(window.find("environment-manage").focused(), Some(true));
        window.press("up", cx);
        assert_eq!(window.find(host_id.clone()).focused(), Some(true));
        window.press("enter", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        shell.read(cx).active_environment.state == ConnectionState::Connected
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        assert!(
            shell.read(cx).active_environment.id == EnvironmentId::Ssh(fixture.host.id.clone())
        );
        window.click("environment-switcher", cx);
        assert!(window.try_find("environment-menu").is_some());
        window.click("nav-checkin", cx);
        assert!(window.try_find("environment-menu").is_none());
        window.click("environment-switcher", cx);
        assert!(window.try_find("environment-menu").is_some());
        window.click("checkin-heading", cx);
        assert!(window.try_find("environment-menu").is_none());
        window.click("environment-switcher", cx);
        assert!(window.try_find("environment-menu").is_some());
        window.click("environment-switcher", cx);
        assert!(
            window.try_find("environment-menu").is_none(),
            "pointer toggle does not reopen after outside dismissal"
        );
        window.click("environment-switcher", cx);
        assert!(window.try_find("environment-menu").is_some());
        window.click("nav-checkin", cx);
        assert!(
            window.try_find("environment-menu").is_none(),
            "same-route navigation also dismisses the menu"
        );
    })
    .unwrap();
    fixture
        .runtime
        .shutdown_after(fixture.services.disconnect_ssh());
}
#[track_caller]
fn wait_resource_ui(
    cx: &mut TestAppContext,
    handle: gpui_kit::AnyWindowHandle,
    mut predicate: impl FnMut(&mut Window, &mut gpui_kit::App) -> bool,
) {
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    loop {
        cx.run_until_parked();
        let ready = cx
            .update_window(handle, |_, window, cx| {
                window.render_frame(cx);
                predicate(window, cx)
            })
            .unwrap();
        if ready {
            return;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "resource UI condition timed out"
        );
        cx.executor().advance_clock(Duration::from_millis(10));
        std::thread::yield_now();
    }
}
#[gpui_kit::test]
async fn ssh_offline_mcp_keeps_converter_navigation_input_and_focus(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let fixture = Fixture::new(true).await;
    fixture.preferences(true);
    fixture.transport.fail("agent");
    let (handle, shell) = fixture.open(cx);
    ready(cx, handle, &shell).await;
    wait_resource_ui(cx, handle, |_, cx| {
        shell.read(cx).active_environment.state == ConnectionState::Unreachable
    });
    let before = fixture.transport.batches();
    cx.update_window(handle, |_, window, cx| {
        window.click("nav-mcp-manager", cx);
        assert!(window.try_find("environment-return-local").is_some());
        assert!(window.try_find("tool-converter").is_some());
        window.click("tool-converter", cx);
    })
    .unwrap();
    wait_resource_ui(cx, handle, |window, _| {
        window.try_find("converter-acknowledge").is_some()
    });
    cx.update_window(handle, |_, window, cx| {
        window.click("converter-acknowledge", cx);
    })
    .unwrap();
    wait_resource_ui(cx, handle, |window, _| {
        gpui_kit::base::test_support::snapshots(window)
            .iter()
            .any(|node| node.label() == Some("Source configuration"))
    });
    cx.update_window(handle, |_, window, cx| {
        assert!(window.try_find("environment-return-local").is_none());
        let input = gpui_kit::base::test_support::snapshots(window)
            .into_iter()
            .find(|node| node.label() == Some("Source configuration"))
            .unwrap();
        window.click(input.path().last().cloned().unwrap(), cx);
        window.input(
            r#"{"mcpServers":{"fixture":{"command":"fixture-mcp"}}}"#,
            cx,
        );
        window.click("converter-run", cx);
    })
    .unwrap();
    wait_resource_ui(cx, handle, |window, _| {
        gpui_kit::base::test_support::snapshots(window)
            .iter()
            .any(|node| {
                node.label() == Some("Converted preview")
                    && node.value().is_some_and(|value| !value.is_empty())
            })
    });
    cx.update_window(handle, |_, window, cx| {
        let output = gpui_kit::base::test_support::snapshots(window)
            .into_iter()
            .find(|node| node.label() == Some("Converted preview"))
            .unwrap();
        assert!(output.value().is_some_and(|value| !value.is_empty()));
        let input_id = gpui_kit::base::test_support::snapshots(window)
            .into_iter()
            .find(|node| node.label() == Some("Source configuration"))
            .unwrap()
            .path()
            .last()
            .cloned()
            .unwrap();
        for _ in 0..3 {
            window.press("f6", cx);
            if window.find(input_id.clone()).focused() == Some(true) {
                break;
            }
        }
        assert_eq!(window.find(input_id).focused(), Some(true));
        window.click("tool-mcp", cx);
        window.render_frame(cx);
        assert!(
            !gpui_kit::base::test_support::snapshots(window)
                .iter()
                .any(|node| node.label() == Some("Source configuration"))
        );
        assert!(window.try_find("environment-return-local").is_some());
    })
    .unwrap();
    assert_eq!(fixture.transport.batches(), before);
    fixture
        .runtime
        .shutdown_after(fixture.services.disconnect_ssh());
}
#[gpui_kit::test]
async fn ssh_resource_and_mcp_tabs_reach_current_environment_controls(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let fixture = Fixture::new(true).await;
    fixture.preferences(true);
    let (handle, shell) = fixture.open(cx);
    ready(cx, handle, &shell).await;
    wait_resource_ui(cx, handle, |_, cx| {
        shell.read(cx).active_environment.state == ConnectionState::Connected
    });
    for (nav, id, path) in [
        ("nav-claude-code", "claude-user-memory", ".claude/CLAUDE.md"),
        ("nav-codex", "codex-agents", ".codex/AGENTS.md"),
        ("nav-antigravity", "gemini-md", ".gemini/GEMINI.md"),
        (
            "nav-opencode",
            "opencode-agents",
            ".config/opencode/AGENTS.md",
        ),
    ] {
        fixture.transport.set_file(
            "build.fixture.invalid",
            &format!("/home/fixture/{path}"),
            "remote instruction fixture",
        );
        cx.update_window(handle, |_, window, cx| {
            window.click(nav, cx);
            window.click("tool-files", cx);
        })
        .unwrap();
        let open = gpui_kit::SharedString::from(format!("resource-open-{id}"));
        wait_resource_ui(cx, handle, |window, _| {
            window.try_find(open.clone()).is_some()
        });
        cx.update_window(handle, |_, window, cx| {
            window.click(open.clone(), cx);
            window.click("resource-decision-confirm", cx);
        })
        .unwrap();
        wait_resource_ui(cx, handle, |window, _| {
            gpui_kit::base::test_support::snapshots(window)
                .iter()
                .any(|node| node.label() == Some("Scoped instruction file"))
        });
        cx.update_window(handle, |_, window, cx| {
            assert_eq!(
                gpui_kit::base::test_support::snapshots(window)
                    .iter()
                    .find(|node| node.label() == Some("Scoped instruction file"))
                    .and_then(|node| node.value()),
                Some("remote instruction fixture")
            );
            window.click("resource-editor-close", cx);
            window.click("tool-extensions", cx);
        })
        .unwrap();
        wait_resource_ui(cx, handle, |window, _| match nav {
            "nav-claude-code" => window.try_find("claude-extension-source-0").is_some(),
            "nav-codex" => window.try_find("codex-agent-new").is_some(),
            _ => gpui_kit::base::test_support::snapshots(window)
                .iter()
                .any(|node| {
                    node.label() == Some("Absolute POSIX project directory on current SSH host")
                }),
        });
    }
    cx.update_window(handle, |_, window, cx| {
        window.click("nav-mcp-manager", cx);
    })
    .unwrap();
    wait_resource_ui(cx, handle, |window, _| {
        window.try_find("mcp-manager").is_some()
    });
    cx.update_window(handle, |_, window, _| {
        assert!(window.try_find("tool-converter").is_some());
        assert!(window.try_find("environment-return-local").is_none());
    })
    .unwrap();
    fixture
        .runtime
        .shutdown_after(fixture.services.disconnect_ssh());
}
