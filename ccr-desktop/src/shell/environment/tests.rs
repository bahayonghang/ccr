#![allow(clippy::unwrap_used)]
use super::*;
use ccr_app::{
    environments::ssh::{HostInput, Workspace, fixture::FixtureTransport},
    runtime::RuntimeOwner,
};
use gpui_kit::test::{TestAppContextExt, TestWindowExt};
use gpui_kit::{Bounds, TestAppContext, WindowBounds, WindowOptions, size};
use std::{sync::Arc, time::Duration};

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
        let viewport = window.find("page-settings").bounds();
        if bounds.top() >= viewport.top() && bounds.bottom() <= viewport.bottom() {
            return;
        }
        window.scroll(
            "page-settings",
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
async fn ready(cx: &mut TestAppContext, handle: gpui_kit::AnyWindowHandle, shell: &Entity<Shell>) {
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        shell.read(cx).preferences_loaded && shell.read(cx).remote.read(cx).hosts().len() == 1
    })
    .await;
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
        window.click("environment-retry", cx)
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
        window.click("environment-retry", cx)
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
    assert!(fixture.transport.cli_requests().is_empty());
    gate.wait();
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        shell.read(cx).active_environment.state == ConnectionState::Connected
    })
    .await;
    assert_eq!(fixture.transport.handshakes(), 2);
    assert!(
        fixture.transport.cli_requests().is_empty(),
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
