#![allow(clippy::unwrap_used)]
use super::*;
use crate::{runtime::ConfigSource, shell::Shell};
use ccr_app::{
    environments::ssh::{CliStatus, Workspace},
    runtime::{RuntimeHandle, RuntimeOwner},
};
use gpui_kit::test::{TestAppContextExt, TestWindowExt};
use gpui_kit::{
    AppContext, Bounds, ScrollDelta, TestAppContext, WindowBounds, WindowOptions, point, size,
};
use std::{sync::Arc, time::Duration};

async fn result(
    services: &Services,
    key: RequestKey,
    action: SshAction,
) -> Arc<Completion<SshResult, &'static str>> {
    let mut subscription = services.ssh_action(key, action).unwrap();
    while let Some(event) = subscription.next().await {
        if let TaskEvent::Terminal(result) = event {
            return result;
        }
    }
    panic!("missing SSH terminal");
}
fn show(window: &mut Window, id: SharedString, cx: &mut gpui_kit::App) {
    for _ in 0..10 {
        window.render_frame(cx);
        let bounds = window.find(id.clone()).bounds();
        let viewport = window
            .try_find("settings-body")
            .map(|node| node.bounds())
            .unwrap_or_else(|| window.find("page-settings").bounds());
        if bounds.top() >= viewport.top() && bounds.bottom() <= viewport.bottom() {
            return;
        }
        window.scroll(
            "settings-body",
            ScrollDelta::Pixels(point(
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
    panic!("SSH control is not reachable in the Settings scroll owner");
}

fn wait_until_runtime_idle(handle: &RuntimeHandle) {
    // CLI fixture 使用 Tokio 真实时钟延迟，GPUI timer 推进的是虚拟时间。
    let started = std::time::Instant::now();
    while handle.active_tasks() > 0 {
        assert!(
            started.elapsed() < Duration::from_secs(10),
            "CLI fixture operation did not reach its terminal state"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[gpui_kit::test]
async fn connected_cli_controls_keep_paths_scoped_and_clear_on_error_and_navigation(
    cx: &mut TestAppContext,
) {
    cx.executor().allow_parking();
    let root = tempfile::tempdir().unwrap();
    let base = root.path().canonicalize().unwrap();
    let runtime = RuntimeOwner::new().unwrap();
    let (workspace, transport) =
        Workspace::fixture(base.join("ssh.db"), base.join("ssh/known_hosts"));
    let workspace = Arc::new(workspace);
    let services = Services::new(
        runtime.handle(),
        ConfigSource::Fixture {
            profiles: base.join("profiles.toml"),
            registry: base.join("registry.toml"),
            delay: Duration::ZERO,
        },
    );
    services
        .ssh
        .set(workspace.clone())
        .unwrap_or_else(|_| panic!("fixture owner"));
    let key = RequestKey {
        environment: EnvironmentId::Local("fixture".into()),
        platform: Platform::Claude,
        generation: 1,
        request_id: 1,
    };
    let saved = result(
        &services,
        key,
        SshAction::SaveHost(HostInput {
            id: None,
            name: "Synthetic remote".into(),
            host: "synthetic.example".into(),
            port: 22,
            user: "fixture".into(),
            remote_home: "/home/fixture".into(),
            ..Default::default()
        }),
    )
    .await;
    let Completion::Success(SshResult::HostSaved(host)) = saved.as_ref() else {
        panic!("save fixture host")
    };
    let host = host.clone();
    transport.set_cli_output("claude", "/usr/local/bin/claude\n");
    transport.set_cli_output("agy", "/opt/antigravity/agy\n");
    transport.set_file(
        "synthetic.example",
        "/home/fixture/.claude/settings.json",
        r#"{"fixture":1}"#,
    );
    transport.delay_cli(Duration::from_millis(100));
    let (handle, _) = cx.update(|cx| {
        gpui_kit::init(cx);
        cx.set_global(crate::locales::Language::English);
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
            |window, cx| {
                cx.new(|cx| {
                    Shell::new_with_preferences(services, Ok(Default::default()), window, cx)
                })
            },
        )
        .unwrap()
    });
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("nav-settings", cx);
        window.click("settings-nav-environment", cx);
    })
    .unwrap();
    let host_id = SharedString::from(format!("ssh-host-{}", host.id));
    cx.wait_for(handle, Duration::from_secs(10), |window, _| {
        window.try_find(host_id.clone()).is_some()
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        show(window, host_id.clone(), cx);
        window.click(host_id, cx);
        if window.try_find("ssh-cli-detect").is_none() {
            window.click("ssh-advanced", cx);
        }
        show(window, "ssh-cli-detect".into(), cx);
        window.click("ssh-cli-detect", cx);
        assert!(transport.cli_requests().is_empty());
        assert!(
            window
                .find("ssh-cli-status")
                .label()
                .unwrap()
                .contains("Connect and verify")
        );
        show(window, "ssh-connect".into(), cx);
        window.click("ssh-connect", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |window, _| {
        window.try_find("environment-trust-confirm").is_some()
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        show(window, "environment-trust-confirm".into(), cx);
        window.click("environment-trust-confirm", cx);
    })
    .unwrap();
    cx.update_window(handle, |_, window, cx| {
        if window.try_find("ssh-cli-detect").is_none() {
            window.click("ssh-advanced", cx);
        }
    })
    .unwrap();
    // 信任确认会自动发起连接，弹窗消失不代表连接已到终态。
    cx.wait_for(handle, Duration::from_secs(10), |window, _| {
        window
            .find("environment-switcher")
            .label()
            .is_some_and(|label| label.contains(" · Connected · "))
            && window
                .try_find("ssh-cli-status")
                .and_then(|node| node.label().map(|v| v.contains("have not been checked")))
                .unwrap_or(false)
    })
    .await;
    assert!(workspace.connected(&host.id));
    cx.update_window(handle, |_, window, cx| {
        assert!(window.try_find("environment-trust-confirm").is_none());
        if window.try_find("ssh-cli-detect").is_none() {
            window.click("ssh-advanced", cx);
        }
        show(window, "ssh-cli-detect".into(), cx);
        window.click("ssh-cli-detect", cx);
        assert_eq!(
            window.find("ssh-cli-detect").label(),
            Some("Detecting remote CLIs…")
        );
    })
    .unwrap();
    wait_until_runtime_idle(&runtime.handle());
    cx.wait_for(handle, Duration::from_secs(10), |window, _| {
        window.try_find("ssh-cli-claude").is_some()
    })
    .await;
    cx.update_window(handle, |_, window, _| {
        assert!(
            window
                .find("ssh-cli-claude")
                .label()
                .unwrap()
                .contains("/usr/local/bin/claude")
        );
        assert!(
            window
                .find("ssh-cli-codex")
                .label()
                .unwrap()
                .contains("Not found")
        );
        assert!(
            window
                .find("ssh-cli-gemini")
                .label()
                .unwrap()
                .contains("/opt/antigravity/agy")
        );
        assert!(window.try_find("ssh-cli-grok").is_none());
    })
    .unwrap();
    transport.fail("detect");
    cx.update_window(handle, |_, window, cx| {
        if window.try_find("ssh-cli-detect").is_none() {
            window.click("ssh-advanced", cx);
        }
        show(window, "ssh-cli-detect".into(), cx);
        window.click("ssh-cli-detect", cx);
        assert!(window.try_find("ssh-cli-claude").is_none());
    })
    .unwrap();
    wait_until_runtime_idle(&runtime.handle());
    cx.wait_for(handle, Duration::from_secs(10), |window, _| {
        window
            .try_find("ssh-cli-status")
            .and_then(|node| node.label().map(|v| v.contains("SSH operation failed")))
            .unwrap_or(false)
    })
    .await;
    assert!(!workspace.connected(&host.id));
    cx.update_window(handle, |_, window, cx| {
        show(window, "ssh-connect".into(), cx);
        window.click("ssh-connect", cx);
        if window.try_find("ssh-cli-detect").is_none() {
            window.click("ssh-advanced", cx);
        }
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |window, _| {
        window
            .find("environment-switcher")
            .label()
            .is_some_and(|label| label.contains(" · Connected · "))
            && window
                .try_find("ssh-cli-status")
                .and_then(|node| node.label().map(|v| v.contains("have not been checked")))
                .unwrap_or(false)
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        if window.try_find("ssh-cli-detect").is_none() {
            window.click("ssh-advanced", cx);
        }
        show(window, "ssh-cli-detect".into(), cx);
        window.click("ssh-cli-detect", cx);
    })
    .unwrap();
    wait_until_runtime_idle(&runtime.handle());
    cx.wait_for(handle, Duration::from_secs(10), |window, _| {
        window.try_find("ssh-cli-claude").is_some()
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.click("nav-dashboard", cx);
        assert!(window.try_find("ssh-cli-claude").is_none());
        window.click("environment-switcher", cx);
        window.click("environment-manage", cx);
        window.simulate_next_frame(cx);
        window.render_frame(cx);
        assert_eq!(
            window.find("settings-nav-environment").checked(),
            Some(true)
        );
        assert!(window.try_find("ssh-cli-claude").is_none());
        if window.try_find("ssh-cli-detect").is_none() {
            window.click("ssh-advanced", cx);
        }
        assert!(
            window
                .find("ssh-cli-status")
                .label()
                .unwrap()
                .contains("have not been checked")
        );
        if window.try_find("ssh-cli-detect").is_none() {
            window.click("ssh-advanced", cx);
        }
        show(window, "ssh-cli-detect".into(), cx);
        window.click("ssh-cli-detect", cx);
        window.click("nav-dashboard", cx);
        window.click("nav-settings", cx);
        window.click("settings-nav-environment", cx);
    })
    .unwrap();
    wait_until_runtime_idle(&runtime.handle());
    cx.run_until_parked();
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert!(window.try_find("ssh-cli-claude").is_none());
        window.remove_window();
    })
    .unwrap();
    assert!(!base.join("profiles.toml").exists());
    runtime.shutdown();
}

#[gpui_kit::test]
fn late_cli_results_cannot_rebind_a_left_or_changed_host(cx: &mut TestAppContext) {
    let runtime = RuntimeOwner::new().unwrap();
    let root = tempfile::tempdir().unwrap();
    let services = Services::new(
        runtime.handle(),
        ConfigSource::Fixture {
            profiles: root.path().join("profiles.toml"),
            registry: root.path().join("registry.toml"),
            delay: Duration::ZERO,
        },
    );
    let (handle, view) = cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
            cx.new(|cx| RemoteView::new(services, window, cx))
        })
        .unwrap()
    });
    cx.update_window(handle, |_, window, cx| {
        view.update(cx, |view, cx| {
            view.selected = Some(Host {
                id: "old".into(),
                name: "Synthetic".into(),
                ..Default::default()
            });
            view.generation = 10;
            view.detecting = true;
            view.busy = true;
            view.connected = true;
            view.editor.update(cx, |editor, cx| {
                editor.set_value("synthetic unsaved remote draft", window, cx);
            });
            view.clear_cli_on_leave(cx);
            assert_eq!(view.selected.as_ref().unwrap().id, "old");
            assert_eq!(
                view.editor.read(cx).value().as_ref(),
                "synthetic unsaved remote draft"
            );
            let rows = Completion::Success(SshResult::CliDetected(vec![CliStatus {
                name: "claude",
                installed: true,
                path: Some("/synthetic/late".into()),
            }]));
            view.cli_finished(10, "old", &rows, cx);
            assert!(view.cli.is_none());
            assert!(!view.busy);
            view.selected = Some(Host {
                id: "other".into(),
                name: "Synthetic other".into(),
                ..Default::default()
            });
            view.cli_finished(view.generation, "old", &rows, cx);
            assert!(view.cli.is_none());
        });
        window.remove_window();
    })
    .unwrap();
    runtime.shutdown();
}
