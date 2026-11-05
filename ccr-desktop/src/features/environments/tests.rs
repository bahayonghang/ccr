use super::{environment_read_key, local_line, ssh_line, ssh_trust_line, wsl_line};
use crate::runtime::ConfigSource;
use crate::shell::Shell;
use ccr_app::environments::{
    DisconnectReport, EnvironmentReport, HostKeyStatus, InspectOptions, ProcessHostTrust,
    ReconnectGate, SshNames, WslExecProbe, WslFileTarget, WslProbe, allow_reconnect,
    allow_reconnect_target, decide_host_key, disconnect_error, inspect, plan_disconnect,
    reconnect_gate, resolve_wsl_file,
};
use ccr_app::runtime::{Completion, RuntimeHandle, RuntimeOwner};
use gpui_kit::test::TestWindowExt;
use gpui_kit::{AppContext, TestAppContext, WindowBounds, WindowOptions, px, size};
use std::time::Duration;

fn labels(window: &gpui_kit::Window) -> String {
    gpui_kit::base::test_support::snapshots(window)
        .into_iter()
        .filter_map(|node| node.label().map(str::to_string))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn wsl_path_failure_names_the_listed_distro_without_the_stderr_path() {
    let line = wsl_line(
        false,
        &WslProbe::Listed {
            names: vec!["docker-desktop".into()],
            exec: WslExecProbe::PathNotFound,
        },
    );
    assert!(line.contains("docker-desktop"));
    assert!(line.contains("path was not found"));
    assert!(line.contains("No usable distribution is connected."));
    assert!(!line.contains("vhdx"));
    assert!(!line.contains("ext4"));
    assert!(!line.contains(" / "));
    assert!(!line.contains("Ubuntu"));
}

#[test]
fn ssh_line_prints_names_only() {
    let line = ssh_line(false, &SshNames::Names(vec!["lab".into()]));
    assert_eq!(line, "Saved SSH host names: lab.");
    assert_eq!(
        ssh_line(false, &SshNames::NotConnected),
        "No SSH host list is loaded in this desktop shell."
    );
    assert_eq!(
        ssh_line(false, &SshNames::Unreadable),
        "Saved SSH host names could not be read."
    );
    assert_eq!(
        ssh_line(true, &SshNames::Unreadable),
        "无法读取已保存的 SSH 主机名。"
    );
    let trust = ssh_trust_line(false);
    assert_eq!(
        trust,
        "Saved SSH names stay names. A new or unknown host key is not stored. A changed host key is not accepted."
    );
    assert!(!trust.to_lowercase().contains("connected"));
    assert!(!trust.to_lowercase().contains("trusted"));
    assert!(!trust.contains(" / "));
    assert!(!ssh_trust_line(true).contains(" / "));
    assert!(!line.contains("secret"));
    assert!(!line.to_lowercase().contains("connected"));
    assert!(!line.to_lowercase().contains("trusted"));
    assert!(!ssh_line(false, &SshNames::Unreadable).contains(" / "));
}

#[test]
fn reconnect_without_process_trust_is_refused_before_a_socket() {
    let trust = ProcessHostTrust::default();
    assert_eq!(
        reconnect_gate(&trust),
        ReconnectGate::Refused("ssh_host_key_untrusted")
    );
    let mut confirmed = ProcessHostTrust::default();
    confirmed.note_confirmed("Example.COM", 22);
    assert_eq!(reconnect_gate(&confirmed), ReconnectGate::CheckTarget);
    assert_eq!(allow_reconnect(&confirmed, "example.com", 22), Ok(()));
    assert_eq!(
        allow_reconnect_target(&trust, Some(("example.com", 22))),
        Err("ssh_host_key_untrusted")
    );
    assert_eq!(
        allow_reconnect_target(&confirmed, None),
        Err("ssh_host_key_untrusted")
    );
    assert_eq!(
        allow_reconnect(&confirmed, "other.example", 22),
        Err("ssh_host_key_untrusted")
    );
    let nothing = plan_disconnect(false);
    assert_eq!(nothing, DisconnectReport::NothingConnected);
    assert_eq!(disconnect_error(nothing), Some("ssh_not_connected"));
    assert_eq!(disconnect_error(plan_disconnect(true)), None);
    let rendered = format!("{trust:?}{confirmed:?}{nothing:?}");
    assert!(!rendered.contains("ssh-keyscan"));
    assert!(!rendered.contains("id_rsa"));
    assert!(!rendered.contains("password"));
    assert!(!rendered.contains(" / "));
}

#[test]
fn host_key_rule_refuses_new_unknown_and_mismatch_without_key_material() {
    let unknown = decide_host_key(None, None);
    assert!(unknown.status().is_none());
    assert!(!unknown.may_store());
    assert!(!unknown.may_trust());
    assert!(!unknown.may_accept());

    let new_key = decide_host_key(None, Some("SHA256:new"));
    assert_eq!(new_key.status(), Some(HostKeyStatus::New));
    assert!(!new_key.may_store());
    assert!(!new_key.may_trust());
    assert!(!new_key.may_accept());

    let mismatch = decide_host_key(Some("SHA256:old"), Some("SHA256:new"));
    assert_eq!(mismatch.status(), Some(HostKeyStatus::Mismatch));
    assert!(!mismatch.may_accept());
    assert!(!mismatch.may_store());
    assert!(!mismatch.may_trust());

    let padded = decide_host_key(Some(" SHA256:same "), Some("SHA256:same"));
    assert_eq!(padded.status(), Some(HostKeyStatus::Mismatch));
    assert!(!padded.may_store());
    assert!(!padded.may_trust());
    assert!(!padded.may_accept());

    let matched = decide_host_key(Some("SHA256:same"), Some("SHA256:same"));
    assert_eq!(matched.status(), Some(HostKeyStatus::Matched));
    assert!(!matched.may_store());
    assert!(!matched.may_trust());
    assert!(!matched.may_accept());
    let rendered = format!("{unknown:?}{new_key:?}{mismatch:?}{padded:?}{matched:?}");
    assert!(!rendered.contains("SHA256"));
    assert!(!rendered.contains("password"));
    assert!(!rendered.contains("id_rsa"));
    assert!(!rendered.contains("ssh-keyscan"));
}

#[test]
fn wsl_file_without_a_distro_is_not_a_windows_path() {
    let dir = tempfile::tempdir().unwrap_or_else(|error| panic!("{error}"));
    let local = dir.path().join("settings.json");
    std::fs::write(&local, b"local-secret").unwrap_or_else(|error| panic!("{error}"));
    let windows_home = r"C:\Users\example\.claude\settings.json";
    let missing = resolve_wsl_file(None, windows_home);
    assert_eq!(missing, WslFileTarget::Unavailable);
    let rendered = format!("{missing:?}");
    assert!(!rendered.contains("Users"));
    assert!(!rendered.contains("settings.json"));
    assert!(!rendered.contains("local-secret"));
    assert!(!rendered.contains(':'));
    assert!(!rendered.contains('\\'));
    let local_target = resolve_wsl_file(None, &local.display().to_string());
    assert_eq!(local_target, WslFileTarget::Unavailable);
    assert!(!format!("{local_target:?}").contains("local-secret"));
    assert_eq!(
        std::fs::read_to_string(&local).unwrap_or_else(|error| panic!("{error}")),
        "local-secret"
    );
    assert_eq!(
        resolve_wsl_file(Some("docker-desktop"), windows_home),
        WslFileTarget::Unavailable
    );
    assert_eq!(
        resolve_wsl_file(None, "/home/user/.claude/settings.json"),
        WslFileTarget::Unavailable
    );
}

#[test]
fn local_line_uses_the_host_os_from_the_caller() {
    assert_eq!(
        local_line(false, "Windows"),
        "This desktop is running in the local environment on Windows."
    );
    assert_eq!(
        local_line(true, "Windows"),
        "此桌面正在 Windows 上的本地环境中运行。"
    );
}

#[gpui_kit::test]
async fn settings_shows_the_local_desktop_without_a_sample_distro(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let owner = RuntimeOwner::new().unwrap_or_else(|error| panic!("{error}"));
    let root = tempfile::tempdir().unwrap_or_else(|error| panic!("{error}"));
    let (handle, _) = cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(gpui_kit::Bounds::centered(
                    None,
                    size(px(1280.), px(900.)),
                    cx,
                ))),
                ..Default::default()
            },
            cx,
            |window, cx| {
                cx.new(|cx| {
                    Shell::new(
                        crate::runtime::Services::new(
                            owner.handle(),
                            ConfigSource::Fixture {
                                profiles: root.path().join("profiles.toml"),
                                registry: root.path().join("registry.toml"),
                                delay: Duration::ZERO,
                            },
                        ),
                        window,
                        cx,
                    )
                })
            },
        )
        .unwrap_or_else(|error| panic!("{error}"))
    });
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("nav-settings", cx);
        window.render_frame(cx);
        let os = ccr_app::environments::host_os_name();
        let text = labels(window);
        assert!(text.contains(&local_line(false, os)));
        if cfg!(windows) {
            assert!(text.contains("WSL was not queried for demo data."));
            assert!(text.contains("Hide and restore are unsupported on Windows."));
        } else {
            assert!(text.contains("WSL is not available on this operating system."));
            assert!(text.contains("Hide and restore were not run on this operating system."));
        }
        assert!(text.contains("No SSH host list is loaded in this desktop shell."));
        assert!(text.contains(ssh_trust_line(false)));
        assert!(!text.to_lowercase().contains("connected"));
        assert!(!text.to_lowercase().contains("trusted"));
        assert!(window.try_find("env-reconnect").is_none());
        assert!(window.try_find("env-disconnect").is_none());
        assert!(text.contains("The tray is unsupported. This window stack has no tray."));
        assert!(text.contains("A system notification can be posted. Delivery is not confirmed."));
        let routes: [&str; 13] = [
            "nav-dashboard",
            "nav-agent-sessions",
            "nav-mcp-manager",
            "nav-claude-code",
            "nav-codex",
            "nav-grok",
            "nav-antigravity",
            "nav-opencode",
            "nav-commands",
            "nav-checkin",
            "nav-sync",
            "nav-usage",
            "nav-settings",
        ];
        for id in routes {
            assert!(window.try_find(id).is_some(), "{id} missing");
        }
        assert!(!text.contains(" / "));
        assert!(!text.contains("Home insights"));
        assert!(!text.contains("Apply / refresh"));
        assert!(!text.contains("Ubuntu"));
        assert!(!text.contains("docker-desktop"));
        assert!(!text.to_lowercase().contains("blur"));
        assert!(window.try_find("env-hide-action").is_none());
        window.click("language-zh", cx);
        window.render_frame(cx);
        let chinese = labels(window);
        assert!(chinese.contains(&local_line(true, os)));
        assert!(chinese.contains(ssh_trust_line(true)));
        assert!(!chinese.contains(ssh_trust_line(false)));
        assert!(!chinese.contains("This desktop is running"));
        assert!(!chinese.contains(" / "));
        assert!(!chinese.to_lowercase().contains("connected"));
        assert!(!chinese.to_lowercase().contains("trusted"));
        window.click("language-en", cx);
        window.click("env-notify", cx);
        window.render_frame(cx);
        assert!(labels(window).contains("Notification request posted. Delivery is not confirmed."));
        window.remove_window();
    })
    .unwrap_or_else(|error| panic!("{error}"));
    let shown = cx.shown_system_notifications();
    assert_eq!(shown.len(), 1);
    assert_eq!(shown[0].tag.as_ref(), "ccr-gpui-environment");
    assert!(!shown[0].body.contains(" / "));
    owner.shutdown();
}

fn held_services(owner: &RuntimeOwner, root: &std::path::Path) -> crate::runtime::Services {
    crate::runtime::Services::new(
        owner.handle(),
        ConfigSource::Fixture {
            profiles: root.join("profiles.toml"),
            registry: root.join("registry.toml"),
            delay: Duration::from_secs(30),
        },
    )
}

fn report_named(name: &str) -> EnvironmentReport {
    let mut report = inspect(InspectOptions::fixture(), &|| false);
    report.ssh = SshNames::Names(vec![name.into()]);
    report
}

fn wait_until_runtime_idle(handle: &RuntimeHandle) {
    let started = std::time::Instant::now();
    while handle.active_tasks() > 0 {
        assert!(
            started.elapsed() < Duration::from_secs(5),
            "held environment read was still running"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn routes() -> [&'static str; 13] {
    [
        "nav-dashboard",
        "nav-agent-sessions",
        "nav-mcp-manager",
        "nav-claude-code",
        "nav-codex",
        "nav-grok",
        "nav-antigravity",
        "nav-opencode",
        "nav-commands",
        "nav-checkin",
        "nav-sync",
        "nav-usage",
        "nav-settings",
    ]
}

#[gpui_kit::test]
fn leaving_settings_suspends_the_in_flight_environment_read(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let owner = RuntimeOwner::new().unwrap_or_else(|error| panic!("{error}"));
    let executor = owner.handle();
    let root = tempfile::tempdir().unwrap_or_else(|error| panic!("{error}"));
    let (handle, shell) = cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(gpui_kit::Bounds::centered(
                    None,
                    size(px(1280.), px(900.)),
                    cx,
                ))),
                ..Default::default()
            },
            cx,
            |window, cx| cx.new(|cx| Shell::new(held_services(&owner, root.path()), window, cx)),
        )
        .unwrap_or_else(|error| panic!("{error}"))
    });
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("nav-settings", cx);
        window.render_frame(cx);
        let probe = shell.update(cx, |this, cx| this.environment_probe(cx));
        assert!(probe.task);
        assert!(probe.loading);
        assert!(!probe.failed);
        assert!(probe.ssh.is_none());
        let text = labels(window);
        assert!(text.contains("Checking WSL."));
        assert!(text.contains("Checking saved SSH host names."));
        assert!(!text.contains("WSL was not queried for demo data."));
        assert!(!text.contains("正在检查 WSL。"));
        assert!(!text.contains(" / "));
        assert!(!text.contains("Home insights"));
        assert!(!text.contains("Apply / refresh"));
        assert!(!text.to_lowercase().contains("blur"));
        assert!(window.try_find("env-wsl").is_some());
        assert!(window.try_find("env-hide-action").is_none());
        for id in routes() {
            assert!(window.try_find(id).is_some(), "{id} missing");
        }
        window.click("nav-dashboard", cx);
        window.render_frame(cx);
        let probe = shell.update(cx, |this, cx| this.environment_probe(cx));
        assert!(!probe.task);
        assert!(!probe.loading);
        assert!(!probe.failed);
        assert!(probe.ssh.is_none());
        assert!(window.try_find("page-dashboard").is_some());
        assert!(window.try_find("page-settings").is_none());
        assert!(window.try_find("env-wsl").is_none());
        assert!(window.try_find("env-ssh").is_none());
        let left = labels(window);
        assert!(!left.contains("Checking WSL."));
        assert!(!left.contains("Checking saved SSH host names."));
        assert!(!left.contains("WSL was not queried for demo data."));
        assert!(!left.contains(ssh_trust_line(false)));
        assert!(!left.contains("Saved SSH host names:"));
        assert!(!left.contains("No SSH host list is loaded"));
        assert!(!left.contains(" / "));
        assert!(!left.contains("Home insights"));
        assert!(!left.contains("Apply / refresh"));
    })
    .unwrap_or_else(|error| panic!("{error}"));
    cx.run_until_parked();
    wait_until_runtime_idle(&executor);
    cx.run_until_parked();
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        let probe = shell.update(cx, |this, cx| this.environment_probe(cx));
        assert!(!probe.task);
        assert!(!probe.loading);
        assert!(!probe.failed);
        assert!(probe.ssh.is_none());
        assert!(window.try_find("env-wsl").is_none());
        let left = labels(window);
        assert!(!left.contains("Checking WSL."));
        assert!(!left.contains("WSL was not queried for demo data."));
        assert!(!left.contains(ssh_trust_line(false)));
        assert!(!left.contains("No SSH host list is loaded"));
        window.remove_window();
    })
    .unwrap_or_else(|error| panic!("{error}"));
    owner.shutdown();
}

#[gpui_kit::test]
fn stale_environment_terminal_does_not_replace_the_visible_report(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let owner = RuntimeOwner::new().unwrap_or_else(|error| panic!("{error}"));
    let executor = owner.handle();
    let root = tempfile::tempdir().unwrap_or_else(|error| panic!("{error}"));
    let services = held_services(&owner, root.path());
    assert!(services.is_slow_fixture());
    let (handle, view) = cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(WindowOptions::default(), cx, |_window, cx| {
            cx.new(|cx| super::EnvironmentView::new(services, cx))
        })
        .unwrap_or_else(|error| panic!("{error}"))
    });
    cx.update_window(handle, |_, window, cx| {
        view.update(cx, |this, cx| {
            this.refresh(cx);
            assert!(this.task.is_some());
            assert!(this.loading);
            assert!(this.report.is_none());
            let first = environment_read_key(this.generation);
            assert!(this.apply_terminal(&first, &Completion::Success(report_named("late-host"))));
            this.suspend(cx);
            assert!(this.task.is_none());
            assert!(!this.loading);
            assert!(
                !this.apply_terminal(&first, &Completion::Success(report_named("after-suspend")))
            );
            assert!(!this.apply_terminal(&first, &Completion::Failed(())));
            this.refresh(cx);
            assert!(this.task.is_some());
            assert_eq!(this.generation, first.generation + 2);
            assert!(!this.apply_terminal(
                &first,
                &Completion::Success(report_named("stale-generation"))
            ));
            let current = environment_read_key(this.generation);
            this.suspend(cx);
            assert!(this.task.is_none());
            assert!(!this.apply_terminal(
                &current,
                &Completion::Success(report_named("after-second-suspend"))
            ));
            assert!(!this.failed);
            assert!(!this.loading);
            assert_eq!(
                this.report.as_ref().map(|report| report.ssh.clone()),
                Some(SshNames::Names(vec!["late-host".into()]))
            );
        });
        window.render_frame(cx);
        let text = labels(window);
        assert!(text.contains("Saved SSH host names: late-host."));
        assert!(text.contains(ssh_trust_line(false)));
        assert!(!text.contains("after-suspend"));
        assert!(!text.contains("stale-generation"));
        assert!(!text.contains("after-second-suspend"));
        assert!(!text.contains("No SSH host list is loaded"));
        assert!(!text.contains("Checking WSL."));
        assert!(!text.contains(" / "));
    })
    .unwrap_or_else(|error| panic!("{error}"));
    cx.run_until_parked();
    wait_until_runtime_idle(&executor);
    cx.run_until_parked();
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        let text = labels(window);
        assert!(text.contains("Saved SSH host names: late-host."));
        assert!(!text.contains("after-suspend"));
        assert!(!text.contains("stale-generation"));
        assert!(!text.contains("after-second-suspend"));
        assert!(!text.contains("No SSH host list is loaded"));
        assert!(!text.contains("docker-desktop"));
        assert!(!text.contains("Ubuntu"));
        view.update(cx, |this, _cx| {
            assert!(this.task.is_none());
            assert!(!this.loading);
            assert!(!this.failed);
            assert_eq!(
                this.report.as_ref().map(|report| report.ssh.clone()),
                Some(SshNames::Names(vec!["late-host".into()]))
            );
        });
        window.remove_window();
    })
    .unwrap_or_else(|error| panic!("{error}"));
    owner.shutdown();
}
