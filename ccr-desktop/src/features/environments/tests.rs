use super::{local_line, ssh_line, ssh_trust_line, wsl_line};
use crate::runtime::ConfigSource;
use crate::shell::Shell;
use ccr_app::environments::{
    HostKeyStatus, SshNames, WslExecProbe, WslFileTarget, WslProbe, decide_host_key,
    resolve_wsl_file,
};
use ccr_app::runtime::RuntimeOwner;
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
