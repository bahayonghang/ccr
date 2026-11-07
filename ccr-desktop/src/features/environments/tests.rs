use super::{
    draft_line, environment_read_key, home_environment, job_line, local_line, local_only_line,
    ssh_line, ssh_trust_line, wsl_line,
};
use crate::runtime::ConfigSource;
use crate::shell::Shell;
use ccr_app::environments::{
    AUTH_UNSUPPORTED, CANCELLED_BEFORE_CONNECT, ConnectFacts, ConnectKind, DisconnectReport,
    EnvironmentReport, HOST_KEY_MISMATCH, HostKeyStatus, InspectOptions, LOCAL_ONLY,
    ProcessHostTrust, ReconnectGate, SshNames, WslExecProbe, WslFileIntent, WslFileTarget,
    WslProbe, allow_reconnect, allow_reconnect_target, boundary_before_connect, decide_host_key,
    disconnect_error, inspect, password_only, plan_disconnect, plan_wsl_file, read_local_source,
    reconnect_gate, resolve_wsl_file,
};
use ccr_app::runtime::{
    Completion, EnvironmentId, Platform, RequestKey, RuntimeHandle, RuntimeOwner, TaskEvent,
};
use gpui_kit::test::{TestAppContextExt, TestWindowExt};
use gpui_kit::{AppContext, TestAppContext, WindowBounds, WindowOptions, px, size};
use std::time::Duration;

fn labels(window: &gpui_kit::Window) -> String {
    gpui_kit::base::test_support::snapshots(window)
        .into_iter()
        .filter_map(|node| node.label().map(str::to_string))
        .collect::<Vec<_>>()
        .join("\n")
}

/// The Windows executable reserves 1 MiB for its UI thread; Rust's test
/// threads normally have more space and masked the native Settings overflow.
#[cfg(windows)]
#[test]
fn environment_views_fit_native_windows_stack_with_isolated_locks() {
    let executable = std::env::current_exe().unwrap_or_else(|error| panic!("{error}"));
    let root = tempfile::tempdir().unwrap_or_else(|error| panic!("{error}"));
    let blocked_default = root.path().join("not-a-lock-directory");
    std::fs::write(&blocked_default, b"fixture lock fallback must fail")
        .unwrap_or_else(|error| panic!("{error}"));
    for test in [
        "features::environments::tests::settings_shows_the_local_desktop_without_a_sample_distro",
        "features::environments::remote::tests::rendered_host_actions_reach_isolated_transport_and_write_remote_document",
        "features::environments::remote::detect::tests::connected_cli_controls_keep_paths_scoped_and_clear_on_error_and_navigation",
        "features::environments::remote::detect::tests::late_cli_results_cannot_rebind_a_left_or_changed_host",
        "features::insights::session_operations::tests::selected_codex_controls_export_cancel_and_clear_plaintext",
        "features::insights::session_operations::tests::session_trash_pagination_bounds_large_collections_and_keeps_controls_reachable",
        "features::insights::observer::tests::observer_setup_confirmation_cancels_then_saves_without_a_usage_database",
        "features::platforms::tests::codex_api_account_requires_confirmation_and_clears_plaintext_after_commit",
        "features::platforms::diagnostics::tests::codex_auth_reviews_warn_without_blocking_and_cleanup_never_takes_unknown_pids",
        "features::platforms::tests::configuration_sources_are_discoverable_and_managed_layers_stay_read_only",
        "features::install::tools::tests::tool_update_review_can_be_cancelled_and_fixture_controls_do_not_launch",
        "features::mcp::batch::tests::rendered_batch_review_focus_escape_confirm_and_navigation_keep_safe_receipt",
        "features::monitoring::benchmark::tests::s2_review_starts_with_safe_cancel_rejects_early_enter_and_dies_on_suspend",
        "features::monitoring::tests::monitoring_filters_real_main_events_and_suspension_detaches_without_polling",
        "features::monitoring::tests::overview_monitoring_entry_preserves_the_thirteen_destination_shell",
        "features::monitoring::system::tests::explicit_system_controls_show_units_and_refuse_fixture_then_clear_stale_sample",
        "features::checkin::credentials::tests::selected_credentials_fill_masked_editor_and_stale_results_are_discarded",
        "features::checkin::credentials::tests::missing_key_editor_requires_recovery_but_keeps_metadata_savable",
        "runtime::checkin::operations::credentials::tests::empty_fixture_transfer_keeps_crypto_creation_and_locks_captured",
        "features::workbench::rolling::tests::codex_rolling_action_distinguishes_missing_and_empty_sessions",
        "features::resources::claude_files::tests::shell_claude_resources_write_confirm_conflict_and_keep_private_fields",
        "features::platforms::forms::tests::ordinary_settings_forms_write_dirty_fields_keep_conflicts_and_clear",
        "features::platforms::forms::tests::provider_common_fields_keep_stored_keys_and_metadata_omissions",
        "features::mcp::editor::form::tests::mcp_common_form_validates_writes_http_and_retains_conflicting_private_drafts",
        "features::mcp::editor::form::tests::mcp_common_form_switches_http_to_stdio_without_stale_transport_type",
        "features::workbench::grok::tests::grok_workbench_reports_missing_baseline_and_clears_on_platform_change",
        "features::insights::capabilities::tests::capability_controls_show_missing_and_old_schema_without_host_probe_or_creation",
    ] {
        let output = std::process::Command::new(&executable)
            .env("RUST_MIN_STACK", "1048576")
            // Child-only poison path: any missed scoped lock fails without
            // accessing the real user's lock directory or changing HOME.
            .env("CCR_LOCK_DIR", &blocked_default)
            .args(["--exact", test, "--skip", "export_bindings"])
            .output()
            .unwrap_or_else(|error| panic!("{error}"));
        assert!(
            output.status.success(),
            "{test} failed on the native stack with isolated locks: {}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed"));
    }
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
        "Manage saved SSH hosts in SSH environments below."
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
    })
    .unwrap_or_else(|error| panic!("{error}"));
    cx.wait_for(handle, Duration::from_secs(5), |window, _| {
        window
            .try_find("language-zh")
            .is_some_and(|button| button.disabled() != Some(true))
    })
    .await;
    cx.run_until_parked();
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        let os = ccr_app::environments::host_os_name();
        let text = labels(window);
        assert!(text.contains(&local_line(false, os)));
        if cfg!(windows) {
            assert!(text.contains("WSL discovery was not run."));
            assert!(text.contains("Hide and restore are unsupported on Windows."));
        } else {
            assert!(text.contains("WSL is not available on this operating system."));
            assert!(text.contains("Hide and restore were not run on this operating system."));
        }
        assert!(text.contains("Manage saved SSH hosts in SSH environments below."));
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
        assert!(
            matches!(
                cx.global::<crate::locales::Language>(),
                crate::locales::Language::Chinese
            ),
            "The rendered language button did not update the language owner"
        );
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
        window.click("settings-nav-environment", cx);
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
        assert!(!text.contains("WSL discovery was not run."));
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
        assert!(!left.contains("WSL discovery was not run."));
        assert!(!left.contains(ssh_trust_line(false)));
        assert!(!left.contains("Saved SSH host names:"));
        assert!(!left.contains("Manage saved SSH hosts"));
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
        assert!(!left.contains("WSL discovery was not run."));
        assert!(!left.contains(ssh_trust_line(false)));
        assert!(!left.contains("Manage saved SSH hosts"));
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
            assert!(!this.apply_terminal(&first, &Completion::Failed("stale")));
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
        assert!(!text.contains("Manage saved SSH hosts"));
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
        assert!(!text.contains("Manage saved SSH hosts"));
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

fn boundary_facts<'a>(
    cancel_requested: bool,
    password: Option<&'a str>,
    stored: Option<&'a str>,
    observed: Option<&'a str>,
    process_trusted: bool,
) -> ConnectFacts<'a> {
    ConnectFacts {
        cancel_requested,
        password_only: password_only(password),
        stored_fingerprint: stored,
        observed_fingerprint: observed,
        process_trusted,
    }
}

#[test]
fn ssh_boundaries_refuse_without_storing_a_key_or_opening_a_socket() {
    let password = "secret-password";
    let identity = "C:/Keys/id_rsa";
    let fingerprint = "SHA256:example.com";
    let Err(cancelled) = boundary_before_connect(
        ConnectKind::Again,
        boundary_facts(
            true,
            Some(password),
            Some(fingerprint),
            Some(fingerprint),
            true,
        ),
    ) else {
        panic!("cancel must refuse before connect");
    };
    assert_eq!(cancelled, CANCELLED_BEFORE_CONNECT);
    let Err(auth) = boundary_before_connect(
        ConnectKind::Open,
        boundary_facts(false, Some(password), None, None, false),
    ) else {
        panic!("password authentication must be refused");
    };
    assert_eq!(auth, AUTH_UNSUPPORTED);
    let Err(first) = boundary_before_connect(
        ConnectKind::Open,
        boundary_facts(false, None, None, Some(fingerprint), false),
    ) else {
        panic!("first trust must not accept a new key");
    };
    assert_eq!(first, "ssh_host_key_untrusted");
    let Err(changed) = boundary_before_connect(
        ConnectKind::Again,
        boundary_facts(false, None, Some("SHA256:old"), Some("SHA256:new"), true),
    ) else {
        panic!("a changed key must not be accepted");
    };
    assert_eq!(changed, HOST_KEY_MISMATCH);
    let Err(matched) = boundary_before_connect(
        ConnectKind::Again,
        boundary_facts(false, None, Some(fingerprint), Some(fingerprint), false),
    ) else {
        panic!("a matching fingerprint is not process trust");
    };
    assert_eq!(matched, "ssh_host_key_untrusted");
    let Err(reconnect) = boundary_before_connect(
        ConnectKind::Again,
        boundary_facts(false, None, None, None, false),
    ) else {
        panic!("reconnect without process trust must refuse");
    };
    assert_eq!(reconnect, "ssh_host_key_untrusted");
    assert_eq!(
        boundary_before_connect(
            ConnectKind::Open,
            boundary_facts(false, None, None, None, false)
        ),
        Ok(())
    );
    let nothing = plan_disconnect(false);
    assert_eq!(disconnect_error(nothing), Some("ssh_not_connected"));
    let rendered = format!("{cancelled}{auth}{first}{changed}{matched}{reconnect}");
    assert!(!rendered.contains(password));
    assert!(!rendered.contains(identity));
    assert!(!rendered.contains(fingerprint));
    assert!(!rendered.contains("ssh-keyscan"));
    assert!(!rendered.contains(" / "));
    let dir = tempfile::tempdir().unwrap_or_else(|error| panic!("{error}"));
    let hosts = dir.path().join("known_hosts");
    std::fs::write(&hosts, b"original").unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(
        boundary_before_connect(
            ConnectKind::Again,
            boundary_facts(false, None, Some(fingerprint), Some(fingerprint), true),
        ),
        Ok(())
    );
    assert_eq!(
        std::fs::read_to_string(&hosts).unwrap_or_else(|error| panic!("{error}")),
        "original"
    );
}

#[test]
fn wsl_file_plans_do_not_fall_back_to_a_local_windows_file() {
    let dir = tempfile::tempdir().unwrap_or_else(|error| panic!("{error}"));
    let local = dir.path().join("settings.json");
    std::fs::write(&local, b"local-secret").unwrap_or_else(|error| panic!("{error}"));
    let local_text = local.display().to_string();
    let before = std::fs::read_dir(dir.path())
        .unwrap_or_else(|error| panic!("{error}"))
        .count();
    let posix = "/home/user/.claude/settings.json";
    for intent in [
        WslFileIntent::Detect,
        WslFileIntent::Read,
        WslFileIntent::Write,
    ] {
        for (distro, path) in [
            (None, local_text.as_str()),
            (Some(""), r"C:\Users\example\.claude\settings.json"),
            (Some("   "), r"\\server\share\settings.json"),
            (Some("docker-desktop"), "//server/share/settings.json"),
            (Some("docker-desktop"), "/home/../etc/passwd"),
        ] {
            let target = plan_wsl_file(intent, distro, path);
            assert_eq!(target, WslFileTarget::Unavailable);
            let rendered = format!("{target:?}");
            assert!(!rendered.contains("local-secret"));
            assert!(!rendered.contains("settings.json"));
            assert!(!rendered.contains("server"));
            assert!(!rendered.contains(':'));
        }
        match plan_wsl_file(intent, Some("docker-desktop"), posix) {
            WslFileTarget::Remote { distro, path } => {
                assert_eq!(distro, "docker-desktop");
                assert_eq!(path, posix);
                assert!(!path.contains(".."));
            }
            WslFileTarget::Unavailable => panic!("posix path should stay on the named distro"),
        }
    }
    assert_eq!(
        resolve_wsl_file(None, &local_text),
        WslFileTarget::Unavailable
    );
    let after = std::fs::read_dir(dir.path())
        .unwrap_or_else(|error| panic!("{error}"))
        .count();
    assert_eq!(before, after);
    assert_eq!(
        std::fs::read_to_string(&local).unwrap_or_else(|error| panic!("{error}")),
        "local-secret"
    );
    assert!(!local_only_line(false).contains(" / "));
    assert!(!local_only_line(true).contains(" / "));
    assert_ne!(local_only_line(false), local_only_line(true));
    assert!(!draft_line(false, "local-draft").contains(" / "));
    assert!(!job_line(false, "local-job").contains(" / "));
}

#[gpui_kit::test]
async fn non_local_environment_load_returns_local_only(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let owner = RuntimeOwner::new().unwrap_or_else(|error| panic!("{error}"));
    let root = tempfile::tempdir().unwrap_or_else(|error| panic!("{error}"));
    let services = crate::runtime::Services::new(
        owner.handle(),
        ConfigSource::Fixture {
            profiles: root.path().join("profiles.toml"),
            registry: root.path().join("registry.toml"),
            delay: Duration::ZERO,
        },
    );
    let key = RequestKey {
        environment: EnvironmentId::Ssh("fixture-ssh".into()),
        platform: Platform::Claude,
        generation: 1,
        request_id: 1,
    };
    let mut read = crate::runtime::environments::load(&services, key)
        .unwrap_or_else(|error| panic!("{error:?}"));
    let event = read.next().await;
    match event {
        Some(TaskEvent::Terminal(value)) => match value.as_ref() {
            Completion::Failed(reason) => {
                assert_eq!(*reason, LOCAL_ONLY);
                assert!(!reason.contains("fixture-ssh"));
                assert!(!reason.contains("ssh-keyscan"));
                assert!(!reason.contains(" / "));
            }
            other => panic!("expected local-only refusal, got {other:?}"),
        },
        other => panic!("missing terminal: {other:?}"),
    }
    owner.shutdown();
}

#[gpui_kit::test]
fn switching_environment_keeps_cache_draft_and_job_apart(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let owner = RuntimeOwner::new().unwrap_or_else(|error| panic!("{error}"));
    let root = tempfile::tempdir().unwrap_or_else(|error| panic!("{error}"));
    let secret = root.path().join("source.txt");
    std::fs::write(&secret, b"local-secret").unwrap_or_else(|error| panic!("{error}"));
    let services = crate::runtime::Services::new(
        owner.handle(),
        ConfigSource::Fixture {
            profiles: root.path().join("profiles.toml"),
            registry: root.path().join("registry.toml"),
            delay: Duration::ZERO,
        },
    );
    let (handle, view) = cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(WindowOptions::default(), cx, |_window, cx| {
            cx.new(|cx| super::EnvironmentView::new(services, cx))
        })
        .unwrap_or_else(|error| panic!("{error}"))
    });
    cx.update_window(handle, |_, window, cx| {
        view.update(cx, |this, cx| this.refresh(cx));
        window.render_frame(cx);
        view.update(cx, |this, cx| {
            this.surface.note_draft("local-draft".into());
            this.surface.note_job("local-job".into());
            cx.notify();
        });
        window.render_frame(cx);
        let local = labels(window);
        assert!(local.contains("Manage saved SSH hosts in SSH environments below."));
        assert!(local.contains(&draft_line(false, "local-draft")));
        assert!(local.contains(&job_line(false, "local-job")));
        assert!(!local.contains(local_only_line(false)));
        assert!(!local.contains(" / "));
        assert!(!local.to_lowercase().contains("blur"));
        view.update(cx, |this, cx| {
            this.task = None;
            this.gate.clear();
            this.surface
                .switch(EnvironmentId::Ssh("fixture-ssh".into()));
            this.generation = this.surface.generation();
            this.loading = false;
            this.failed = false;
            this.local_only = None;
            this.report = this.surface.cache();
            this.refresh(cx);
        });
        window.render_frame(cx);
        let calls = std::cell::Cell::new(0u32);
        let refused = view.update(cx, |this, _cx| {
            read_local_source(this.surface.active(), &home_environment(), || {
                calls.set(calls.get() + 1);
                std::fs::read_to_string(&secret).unwrap_or_else(|error| panic!("{error}"))
            })
        });
        assert_eq!(refused, Err(LOCAL_ONLY));
        assert_eq!(calls.get(), 0);
        assert_eq!(
            std::fs::read_to_string(&secret).unwrap_or_else(|error| panic!("{error}")),
            "local-secret"
        );
        let Err(reason) = view.update(cx, |this, _cx| {
            this.plan_connection(boundary_facts(
                true,
                Some("secret-password"),
                Some("SHA256:example.com"),
                Some("SHA256:example.com"),
                true,
            ))
        }) else {
            panic!("cancel on a remote environment must refuse before connect");
        };
        assert_eq!(reason, CANCELLED_BEFORE_CONNECT);
        assert!(!reason.contains("secret-password"));
        assert!(!reason.contains("example.com"));
        assert!(!reason.contains("ssh-keyscan"));
        let unknown = view.update(cx, |this, _cx| {
            this.plan_connection(boundary_facts(false, None, None, None, true))
        });
        assert_eq!(unknown, Err("ssh_host_key_untrusted"));
        view.update(cx, |this, cx| {
            this.surface.note_draft("ssh-draft".into());
            this.surface.note_job("ssh-job".into());
            cx.notify();
        });
        window.render_frame(cx);
        let remote = labels(window);
        assert!(remote.contains(local_only_line(false)));
        assert!(remote.contains(&draft_line(false, "ssh-draft")));
        assert!(remote.contains(&job_line(false, "ssh-job")));
        assert!(!remote.contains("Manage saved SSH hosts"));
        assert!(!remote.contains("local-draft"));
        assert!(!remote.contains("local-job"));
        assert!(!remote.contains("local-secret"));
        assert!(!remote.contains("fixture-ssh"));
        assert!(!remote.contains("ssh-keyscan"));
        assert!(!remote.contains(" / "));
        assert!(!remote.to_lowercase().contains("blur"));
        assert!(window.try_find("env-reconnect").is_none());
        assert!(window.try_find("env-disconnect").is_none());
        view.update(cx, |this, cx| {
            this.task = None;
            this.gate.clear();
            this.surface.switch(home_environment());
            this.generation = this.surface.generation();
            this.loading = false;
            this.failed = false;
            this.local_only = None;
            this.report = this.surface.cache();
            cx.notify();
        });
        window.render_frame(cx);
        let restored = labels(window);
        assert!(restored.contains("Manage saved SSH hosts in SSH environments below."));
        assert!(restored.contains(&draft_line(false, "local-draft")));
        assert!(restored.contains(&job_line(false, "local-job")));
        assert!(!restored.contains("ssh-draft"));
        assert!(!restored.contains("ssh-job"));
        assert!(!restored.contains(local_only_line(false)));
        assert!(!restored.contains("local-secret"));
        assert!(!restored.contains(" / "));
        assert!(!restored.contains("Home insights"));
        assert!(!restored.contains("Apply / refresh"));
        window.remove_window();
    })
    .unwrap_or_else(|error| panic!("{error}"));
    owner.shutdown();
}
