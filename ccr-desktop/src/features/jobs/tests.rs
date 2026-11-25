use super::*;
use ccr_app::runtime::RuntimeOwner;
use gpui_kit::test::{TestAppContextExt, TestWindowExt};
use gpui_kit::{
    AnyWindowHandle, Role, SharedString, TestAppContext, WindowBounds, WindowOptions, px, size,
};

fn fixture_services(owner: &RuntimeOwner, root: &std::path::Path) -> Services {
    Services::new(
        owner.handle(),
        crate::runtime::ConfigSource::Fixture {
            profiles: root.join("profiles.toml"),
            registry: root.join("config.toml"),
            delay: std::time::Duration::ZERO,
        },
    )
}

/// The command page alone in a 1280×800 window.
fn open(cx: &mut TestAppContext, services: Services) -> (AnyWindowHandle, Entity<JobsView>) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        crate::theme::init(cx);
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(gpui_kit::Bounds::new(
                    gpui_kit::point(px(0.), px(0.)),
                    size(px(1280.), px(800.)),
                ))),
                ..Default::default()
            },
            cx,
            |window, cx| cx.new(|cx| JobsView::new(services, window, cx)),
        )
        .unwrap_or_else(|_| panic!("window"))
    })
}

fn set_input(
    input: &Entity<gpui_kit::component::input::InputState>,
    value: &str,
    window: &mut Window,
    cx: &mut App,
) {
    input.update(cx, |state, cx| {
        state.set_value(value.to_string(), window, cx)
    });
}

fn labels(window: &gpui_kit::Window) -> String {
    gpui_kit::base::test_support::snapshots(window)
        .into_iter()
        .filter_map(|node| node.label().map(str::to_string))
        .collect::<Vec<_>>()
        .join("\n")
}

fn label(window: &gpui_kit::Window, id: impl Into<gpui_kit::ElementId>) -> String {
    window.find(id).label().unwrap_or_default().to_string()
}

#[gpui_kit::test]
async fn revisit_reattaches_owner_and_rejects_old_generation_or_job(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let root = tempfile::tempdir().unwrap_or_else(|_| panic!("fixture"));
    let owner = RuntimeOwner::new().unwrap_or_else(|_| panic!("runtime"));
    let services = fixture_services(&owner, root.path());
    let jobs = services.command_jobs.clone();
    let admission = jobs
        .admit(
            RequestKey {
                environment: EnvironmentId::Local("fixture".into()),
                platform: Platform::Claude,
                generation: 1,
                request_id: 1,
            },
            ccr_app::jobs::policy::CommandExecutionRequest::background(
                "status".into(),
                Some(vec!["--json".into()]),
                None,
            ),
        )
        .unwrap_or_else(|_| panic!("admit"));
    let id = admission.snapshot.job_id.clone();
    let (handle, view) = cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
            cx.new(|cx| JobsView::new(services, window, cx))
        })
        .unwrap_or_else(|_| panic!("window"))
    });
    cx.update_window(handle, |_, _, cx| {
        view.update(cx, |this, cx| {
            this.job_id = Some(id.clone());
            this.refresh(cx);
            assert!(this.busy);
            assert!(this.task.is_some());
            this.suspend(cx);
            assert!(this.task.is_none());
            assert!(
                !jobs
                    .status(&id)
                    .unwrap_or_else(|_| panic!("status"))
                    .status
                    .is_terminal()
            );
            this.refresh(cx);
            assert!(this.busy);
            assert!(this.task.is_some());
            let mut stale = admission.snapshot.clone();
            stale.status = ccr_app::jobs::commands::Status::Success;
            this.accept_snapshot(this.generation - 1, stale.clone());
            assert!(this.busy);
            stale.job_id = "another-job".into();
            this.accept_snapshot(this.generation, stale);
            assert!(this.busy);
        });
    })
    .unwrap_or_else(|_| panic!("revisit"));
    jobs.request_cancel(&id)
        .unwrap_or_else(|_| panic!("cancel"));
    cx.wait_for(handle, std::time::Duration::from_secs(5), |_, cx| {
        view.read(cx)
            .snapshot
            .as_ref()
            .is_some_and(|snapshot| snapshot.cancel_requested)
    })
    .await;
    cx.update_window(handle, |_, _, cx| assert!(view.read(cx).busy))
        .unwrap_or_else(|_| panic!("nonterminal cancel"));
    jobs.abandon(admission.ticket.unwrap_or_else(|| panic!("ticket")));
    cx.wait_for(handle, std::time::Duration::from_secs(5), |_, cx| {
        !view.read(cx).busy
    })
    .await;
    cx.update_window(handle, |_, _, cx| {
        view.update(cx, |this, cx| {
            let terminal = this.snapshot.clone().unwrap_or_else(|| panic!("terminal"));
            assert_eq!(
                terminal.status,
                ccr_app::jobs::commands::Status::Unavailable
            );
            this.suspend(cx);
            this.refresh(cx);
            assert!(this.task.is_none());
            assert!(!this.busy);
            let mut late = terminal.clone();
            late.status = ccr_app::jobs::commands::Status::Running;
            this.accept_snapshot(this.generation, late);
            assert_eq!(this.snapshot.as_ref(), Some(&terminal));
        });
    })
    .unwrap_or_else(|_| panic!("terminal revisit"));
    owner.shutdown();
}

#[gpui_kit::test]
fn catalog_opens_on_runnable_with_counts_before_search_and_a_clearable_empty_state(
    cx: &mut TestAppContext,
) {
    cx.executor().allow_parking();
    let fixture = tempfile::tempdir().unwrap_or_else(|_| panic!("fixture"));
    let owner = RuntimeOwner::new().unwrap_or_else(|_| panic!("runtime"));
    let (handle, view) = open(cx, fixture_services(&owner, fixture.path()));
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert_eq!(window.find("command-heading").role(), Some(Role::Heading));
        assert_eq!(label(window, "command-heading"), "Commands");
        assert_eq!(
            label(window, "command-environment"),
            "Execution environment · Local"
        );
        assert_eq!(label(window, "command-catalog-summary"), "6 runnable");
        assert!(
            window.try_find("command-job-status").is_none(),
            "an idle page shows no job tag"
        );
        assert!(label(window, "jobs-fixture-warning").contains("Synthetic owned-process"));
        // Default filter is Runnable; every chip carries its catalog count.
        assert_eq!(window.find("command-filter-runnable").checked(), Some(true));
        assert_eq!(window.find("command-filter-all").checked(), Some(false));
        let text = labels(window);
        for chip in [
            "Runnable 6",
            "All 24",
            "Preview only 14",
            "Needs its page 4",
        ] {
            assert!(text.contains(chip), "missing chip {chip}");
        }
        assert_eq!(window.find("command-listbox").role(), Some(Role::ListBox));
        assert_eq!(
            window.find("command-row:status").role(),
            Some(Role::ListBoxOption)
        );
        assert_eq!(
            label(window, "command-row:status"),
            "status · Status overview · Runnable"
        );
        assert!(window.try_find("command-row:delete").is_none());
        // The old capsule wall, foreground run and refresh status are gone.
        for removed in ["command-foreground", "command-status", "job-command-status"] {
            assert!(
                window.try_find(removed).is_none(),
                "{removed} still renders"
            );
        }
        assert!(!text.contains(" / "));

        let search = view.read(cx).search.clone();
        set_input(&search, "zzz-no-match", window, cx);
        window.render_frame(cx);
        assert!(window.try_find("command-row:status").is_none());
        assert_eq!(label(window, "command-empty"), "No matching commands");
        assert!(
            labels(window).contains("Runnable 6"),
            "counts are computed before search"
        );
        window.click("command-clear-filters", cx);
        window.render_frame(cx);
        assert_eq!(search.read(cx).value().as_str(), "");
        assert_eq!(window.find("command-filter-all").checked(), Some(true));
        assert!(window.try_find("command-empty").is_none());
        assert!(
            window.try_find("command-row:list").is_some(),
            "All lists preview-only commands too"
        );

        set_input(&search, "CONFIGURATION operations", window, cx);
        window.render_frame(cx);
        assert!(window.try_find("command-row:history").is_some());
        assert!(window.try_find("command-row:status").is_none());

        view.update(cx, |this, cx| {
            this.catalog = Catalog::new(Vec::new());
            this.selected = "missing".into();
            this.start(cx);
        });
        window.render_frame(cx);
        let empty = labels(window);
        assert!(empty.contains("The local command catalog is empty."));
        assert!(empty.contains("cannot be started from this catalog"));
        assert_eq!(label(window, "command-catalog-summary"), "0 runnable");
        assert!(label(window, "command-run-blocked").contains("not in the local catalog"));
        assert!(view.read(cx).job_id.is_none());
        assert!(!empty.contains(" / "));
        window.remove_window();
    })
    .unwrap_or_else(|_| panic!("catalog"));
    owner.shutdown();
}

#[gpui_kit::test]
fn arrow_keys_move_the_highlight_and_enter_loads_a_live_preview_with_hints(
    cx: &mut TestAppContext,
) {
    cx.executor().allow_parking();
    let fixture = tempfile::tempdir().unwrap_or_else(|_| panic!("fixture"));
    let owner = RuntimeOwner::new().unwrap_or_else(|_| panic!("runtime"));
    let (handle, view) = open(cx, fixture_services(&owner, fixture.path()));
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert_eq!(window.find("command-row:status").selected(), Some(true));
        assert_eq!(label(window, "command-preview"), "ccr status");
        let focus = view.read(cx).list_focus.clone();
        window.focus(&focus, cx);
        window.press("down", cx);
        window.render_frame(cx);
        assert_eq!(window.find("command-row:validate").selected(), Some(true));
        assert_eq!(window.find("command-row:status").selected(), Some(false));
        assert_eq!(view.read(cx).selected, "status", "moving does not load");
        window.press("down", cx);
        window.press("enter", cx);
        window.render_frame(cx);
        assert_eq!(view.read(cx).selected, "history");
        assert_eq!(label(window, "command-editor-name"), "history");
        assert_eq!(
            label(window, "command-editor-description"),
            "Inspect recent CCR configuration operations."
        );
        assert_eq!(label(window, "command-preview"), "ccr history");
        assert_eq!(
            label(window, "command-args-hints"),
            "Available arguments: --limit, -l <number> · --type, -t <text>"
        );
        assert!(window.try_find("command-run-blocked").is_none());
        assert!(!view.read(cx).has_environment_draft(cx));

        let args = view.read(cx).args.clone();
        set_input(&args, r#"-l   20 -t "two words""#, window, cx);
        window.render_frame(cx);
        assert_eq!(
            label(window, "command-preview"),
            r#"ccr history -l 20 -t "two words""#
        );
        assert!(window.try_find("command-args-error").is_none());
        assert_ne!(
            window.find("command-start").focused(),
            None,
            "Run is enabled"
        );
        assert!(view.read(cx).has_environment_draft(cx));

        window.focus(&focus, cx);
        window.press("up", cx);
        window.render_frame(cx);
        assert_eq!(window.find("command-row:validate").selected(), Some(true));
        assert_eq!(view.read(cx).selected, "history");
        window.remove_window();
    })
    .unwrap_or_else(|_| panic!("keyboard"));
    owner.shutdown();
}

#[gpui_kit::test]
fn unavailable_commands_load_with_a_reason_and_invalid_drafts_never_start(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let fixture = tempfile::tempdir().unwrap_or_else(|_| panic!("fixture"));
    let owner = RuntimeOwner::new().unwrap_or_else(|_| panic!("runtime"));
    let (handle, view) = open(cx, fixture_services(&owner, fixture.path()));
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        view.update(cx, |this, cx| {
            this.busy = true;
            cx.notify();
        });
        window.render_frame(cx);
        window.click("command-row:validate", cx);
        assert_eq!(
            view.read(cx).selected,
            "status",
            "a running job keeps the loaded command"
        );
        view.update(cx, |this, cx| {
            this.busy = false;
            cx.notify();
        });
        window.render_frame(cx);

        window.click("command-filter-dedicated", cx);
        window.render_frame(cx);
        window.click("command-row:delete", cx);
        window.render_frame(cx);
        assert_eq!(view.read(cx).selected, "delete");
        assert_eq!(
            label(window, "command-row:delete"),
            "delete · Delete configuration · Needs its page"
        );
        assert!(label(window, "command-run-blocked").contains("Needs a dedicated page"));
        assert_eq!(
            window.find("command-start").focused(),
            None,
            "Run is disabled"
        );
        assert_ne!(
            window.find("command-help").focused(),
            None,
            "Help stays available"
        );
        window.click("command-start", cx);
        view.update(cx, |this, cx| this.start(cx));
        assert!(view.read(cx).job_id.is_none());

        window.click("command-filter-preview", cx);
        window.render_frame(cx);
        window.click("command-row:list", cx);
        window.render_frame(cx);
        assert!(label(window, "command-run-blocked").contains("Run this command in a terminal"));
        assert_eq!(window.find("command-start").focused(), None);

        window.click("command-filter-runnable", cx);
        window.render_frame(cx);
        window.click("command-row:status", cx);
        window.render_frame(cx);
        let args = view.read(cx).args.clone();
        set_input(&args, r#"-t "secret-sentinel"#, window, cx);
        window.render_frame(cx);
        assert!(label(window, "command-args-error").contains("double quote is not closed"));
        assert_eq!(
            label(window, "command-preview"),
            r#"ccr status -t "secret-sentinel"#
        );
        assert_eq!(window.find("command-start").focused(), None);
        window.click("command-start", cx);
        view.update(cx, |this, cx| this.start(cx));
        assert!(view.read(cx).job_id.is_none());
        assert!(view.read(cx).snapshot.is_none());
        assert!(!view.read(cx).busy);
        assert!(
            fixture
                .path()
                .read_dir()
                .unwrap_or_else(|_| panic!("read fixture"))
                .next()
                .is_none(),
            "invalid drafts must not admit work or initialize Recent storage"
        );
        window.click("command-help", cx);
        let snapshot = view
            .read(cx)
            .snapshot
            .as_ref()
            .unwrap_or_else(|| panic!("help snapshot"));
        assert_eq!(snapshot.command, "help");
        assert_eq!(snapshot.args, ["status"]);
        assert!(
            !view
                .read(cx)
                .message
                .unwrap_or_default()
                .contains("secret-sentinel")
        );
        assert_eq!(
            view.read(cx).args.read(cx).value().as_str(),
            r#"-t "secret-sentinel"#
        );
    })
    .unwrap_or_else(|_| panic!("controls"));
    owner.shutdown();
    // The valid Help admission has a failed terminal in this unmarked fixture;
    // its retained finalizer records that outcome even after observation ends.
    assert!(fixture.path().join("ui-state.db").is_file());
    let pool = ccr_db::database::create_pool(&fixture.path().join("ui-state.db"), None)
        .unwrap_or_else(|_| panic!("Recent fixture pool"));
    let manager =
        ccr_db::managers::ui_state::UiStateManager::with_db(ccr_db::database::DbAccess::Pool(pool));
    let history = manager
        .try_get_history(None)
        .unwrap_or_else(|_| panic!("Recent history"));
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].command, "help");
    assert_eq!(history[0].full_command, "ccr help");
    assert!(history[0].args.is_empty());
    assert!(!history[0].success);
}

#[gpui_kit::test]
async fn run_starts_a_background_job_and_cancel_takes_the_same_slot(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let fixture = tempfile::tempdir().unwrap_or_else(|_| panic!("fixture"));
    let owner = RuntimeOwner::new().unwrap_or_else(|_| panic!("runtime"));
    let services = fixture_services(&owner, fixture.path());
    let jobs = services.command_jobs.clone();
    let (handle, view) = open(cx, services);
    let run_width = cx
        .update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            window.find("command-start").bounds().size.width
        })
        .unwrap_or_else(|_| panic!("run width"));
    // Hold a job open so the page stays busy while it observes it.
    let admission = jobs
        .admit(
            RequestKey {
                environment: EnvironmentId::Local("fixture".into()),
                platform: Platform::Claude,
                generation: 1,
                request_id: 1,
            },
            ccr_app::jobs::policy::CommandExecutionRequest::background(
                "status".into(),
                Some(vec!["--json".into()]),
                None,
            ),
        )
        .unwrap_or_else(|_| panic!("admit"));
    let id = admission.snapshot.job_id.clone();
    cx.update_window(handle, |_, window, cx| {
        view.update(cx, |this, cx| {
            this.job_id = Some(id.clone());
            this.refresh(cx);
            assert!(this.busy);
        });
        window.render_frame(cx);
        assert!(window.try_find("command-start").is_none());
        assert_eq!(
            window.find("command-cancel").bounds().size.width,
            run_width,
            "Cancel replaces Run without moving the row"
        );
        assert!(matches!(
            label(window, "command-job-status").as_str(),
            "Queued" | "Running"
        ));
        window.click("command-cancel", cx);
    })
    .unwrap_or_else(|_| panic!("busy"));
    cx.wait_for(handle, std::time::Duration::from_secs(5), |_, cx| {
        view.read(cx)
            .snapshot
            .as_ref()
            .is_some_and(|snapshot| snapshot.cancel_requested)
    })
    .await;
    jobs.abandon(admission.ticket.unwrap_or_else(|| panic!("ticket")));
    cx.wait_for(handle, std::time::Duration::from_secs(5), |_, cx| {
        !view.read(cx).busy
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert!(window.try_find("command-cancel").is_none());
        let args = view.read(cx).args.clone();
        set_input(&args, "--json", window, cx);
        window.render_frame(cx);
        window.click("command-start", cx);
        let snapshot = view
            .read(cx)
            .snapshot
            .clone()
            .unwrap_or_else(|| panic!("run snapshot"));
        assert_eq!(snapshot.command, "status");
        assert_eq!(snapshot.args, ["--json"]);
        assert_ne!(view.read(cx).job_id.as_deref(), Some(id.as_str()));
        window.remove_window();
    })
    .unwrap_or_else(|_| panic!("run"));
    owner.shutdown();
}

#[gpui_kit::test]
async fn ledger_shows_empty_state_readouts_grouped_channels_copy_clear_and_polite_status(
    cx: &mut TestAppContext,
) {
    cx.executor().allow_parking();
    let fixture = tempfile::tempdir().unwrap_or_else(|_| panic!("fixture"));
    let owner = RuntimeOwner::new().unwrap_or_else(|_| panic!("runtime"));
    let services = fixture_services(&owner, fixture.path());
    let jobs = services.command_jobs.clone();
    let (handle, view) = open(cx, services);

    // 1. Idle state: dashed empty panel, no readouts, copy & clear disabled.
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert!(window.try_find("command-ledger-empty").is_some());
        assert_eq!(
            label(window, "command-ledger-empty"),
            "Ready to run a command"
        );
        assert!(window.try_find("command-ledger-readouts").is_none());
        assert!(
            label(window, "command-ledger-subtitle").contains("Select a command and run it"),
            "idle subtitle explains when output appears"
        );
        assert_eq!(window.find("command-ledger-copy").focused(), None);
        assert_eq!(window.find("command-ledger-clear").focused(), None);
    })
    .unwrap_or_else(|_| panic!("idle ledger"));

    // 2. Running job: readouts with unknown dashes, polite Running status, Clear disabled, Copy works.
    let admission = jobs
        .admit(
            RequestKey {
                environment: EnvironmentId::Local("fixture".into()),
                platform: Platform::Claude,
                generation: 1,
                request_id: 1,
            },
            ccr_app::jobs::policy::CommandExecutionRequest::background(
                "status".into(),
                Some(vec!["--json".into()]),
                None,
            ),
        )
        .unwrap_or_else(|_| panic!("admit"));
    let id = admission.snapshot.job_id.clone();
    let short_id: String = id
        .strip_prefix("ccr-command-")
        .unwrap_or(&id)
        .chars()
        .take(8)
        .collect();
    cx.update_window(handle, |_, window, cx| {
        view.update(cx, |this, cx| {
            this.job_id = Some(id.clone());
            this.refresh(cx);
        });
        window.render_frame(cx);
        assert!(window.try_find("command-ledger-empty").is_none());
        assert!(window.try_find("command-ledger-readouts").is_some());
        assert_eq!(
            label(window, "command-ledger-subtitle"),
            format!("ccr status --json · job {short_id} · grouped by channel")
        );
        assert!(label(window, "command-ledger-readout-duration").contains('–'));
        assert!(label(window, "command-ledger-readout-exit").contains('–'));
        assert!(label(window, "command-ledger-readout-lines").contains('1'));
        assert_eq!(
            window.find("command-ledger-status").role(),
            Some(Role::Status)
        );
        assert!(label(window, "command-ledger-status").contains("Running…"));
        assert_eq!(
            window.find("command-ledger-clear").focused(),
            None,
            "Clear stays disabled while a job runs"
        );
        window.click("command-ledger-clear", cx);
        window.render_frame(cx);
        assert!(
            window.try_find("command-ledger-readouts").is_some(),
            "clicking disabled Clear while running keeps the active job"
        );
        assert_eq!(window.find("command-ledger-copy").focused(), Some(false));
        window.click("command-ledger-copy", cx);
        let copied = cx
            .read_from_clipboard()
            .and_then(|item| item.text())
            .unwrap_or_default();
        assert_eq!(copied, "[system] Job queued");
    })
    .unwrap_or_else(|_| panic!("running ledger"));

    // 3. Terminal state: polite terminal status, Clear enabled and only clears view snapshot.
    jobs.abandon(admission.ticket.unwrap_or_else(|| panic!("ticket")));
    cx.wait_for(handle, std::time::Duration::from_secs(5), |_, cx| {
        !view.read(cx).busy
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert_eq!(
            window.find("command-ledger-status").role(),
            Some(Role::Status)
        );
        assert!(
            label(window, "command-ledger-status").contains("Unavailable"),
            "terminal state announced politely"
        );
        assert_eq!(window.find("command-ledger-clear").focused(), Some(false));
        window.click("command-ledger-clear", cx);
        window.render_frame(cx);
        assert!(window.try_find("command-ledger-empty").is_some());
        assert!(window.try_find("command-ledger-readouts").is_none());
        assert!(
            jobs.status(&id).is_ok(),
            "clearing the ledger does not evict the app-owned job"
        );
    })
    .unwrap_or_else(|_| panic!("terminal and clear"));

    // 4. Truncated snapshot with system/stdout/stderr and >8KiB line.
    cx.update_window(handle, |_, window, cx| {
        let long_line = "x".repeat(9000);
        let snap = Snapshot {
            job_id: "ccr-command-deadbeef-1234".into(),
            command: "history".into(),
            args: vec!["-l".into(), "20".into()],
            status: ccr_app::jobs::commands::Status::Success,
            cancel_requested: false,
            started_at: "2026-11-25T00:00:00Z".into(),
            finished_at: Some("2026-11-25T00:00:01Z".into()),
            duration_ms: Some(1200),
            exit_code: Some(0),
            system_lines: vec!["Job queued".into()].into(),
            stdout_lines: vec!["first out".into(), long_line.clone()].into(),
            stderr_lines: vec!["warn err".into()].into(),
            truncated: true,
            dropped_lines: 4,
            error: None,
        };
        view.update(cx, |this, cx| {
            this.job_id = Some(snap.job_id.clone());
            this.accept_snapshot(this.generation, snap);
            cx.notify();
        });
        window.render_frame(cx);
        assert_eq!(
            label(window, "command-ledger-subtitle"),
            "ccr history -l 20 · job deadbeef · grouped by channel"
        );
        assert!(label(window, "command-ledger-readout-duration").contains("1.2 s"));
        assert!(label(window, "command-ledger-readout-exit").contains('0'));
        assert!(label(window, "command-ledger-readout-lines").contains("4 dropped"));
        assert!(label(window, "command-ledger-truncated").contains("4"));
        assert_eq!(
            label(window, "command-log-ccr-command-deadbeef-1234-0"),
            "[system] Job queued"
        );
        assert_eq!(
            label(window, "command-log-ccr-command-deadbeef-1234-1"),
            "[stdout] first out"
        );
        let clipped = label(window, "command-log-ccr-command-deadbeef-1234-2");
        assert!(clipped.starts_with("[stdout] "));
        assert!(clipped.ends_with('…'));
        assert_eq!(
            label(window, "command-log-ccr-command-deadbeef-1234-3"),
            "[stderr] warn err"
        );
        window.click("command-ledger-copy", cx);
        let copied = cx
            .read_from_clipboard()
            .and_then(|item| item.text())
            .unwrap_or_default();
        assert_eq!(
            copied,
            format!(
                "[system] Job queued\n[stdout] first out\n[stdout] {long_line}\n[stderr] warn err"
            )
        );
        window.remove_window();
    })
    .unwrap_or_else(|_| panic!("grouped truncated ledger"));
    owner.shutdown();
}

#[gpui_kit::test]
fn ledger_follows_bottom_stops_when_scrolled_up_and_resumes_at_bottom(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let fixture = tempfile::tempdir().unwrap_or_else(|_| panic!("fixture"));
    let owner = RuntimeOwner::new().unwrap_or_else(|_| panic!("runtime"));
    let (handle, view) = open(cx, fixture_services(&owner, fixture.path()));
    let make_snap = |count: usize| Snapshot {
        job_id: "ccr-command-feedface-0001".into(),
        command: "status".into(),
        args: vec![],
        status: ccr_app::jobs::commands::Status::Running,
        cancel_requested: false,
        started_at: "2026-11-25T00:00:00Z".into(),
        finished_at: None,
        duration_ms: None,
        exit_code: None,
        system_lines: vec!["Job queued".into()].into(),
        stdout_lines: (0..count).map(|i| format!("line-{i}")).collect(),
        stderr_lines: Default::default(),
        truncated: false,
        dropped_lines: 0,
        error: None,
    };

    cx.update_window(handle, |_, window, cx| {
        view.update(cx, |this, cx| {
            this.job_id = Some("ccr-command-feedface-0001".into());
            this.accept_snapshot(this.generation, make_snap(40));
            cx.notify();
        });
        window.render_frame(cx);
        assert!(
            window
                .try_find("command-log-ccr-command-feedface-0001-40")
                .is_some_and(|row| row.visible()),
            "initial long log pins to the last row"
        );

        // Append more lines while pinned -> still follows to bottom.
        view.update(cx, |this, cx| {
            this.accept_snapshot(this.generation, make_snap(50));
            cx.notify();
        });
        window.render_frame(cx);
        assert!(
            window
                .try_find("command-log-ccr-command-feedface-0001-50")
                .is_some_and(|row| row.visible()),
            "new lines keep the view pinned to the bottom"
        );

        // User scrolls up to the top -> follow stops.
        let top_offset = {
            let state = view.read(cx).log_scroll.0.borrow();
            state
                .base_handle
                .set_offset(gpui_kit::point(px(0.), px(0.)));
            state.base_handle.offset().y
        };
        window.render_frame(cx);
        assert_eq!(top_offset, px(0.));

        view.update(cx, |this, cx| {
            this.accept_snapshot(this.generation, make_snap(60));
            cx.notify();
        });
        window.render_frame(cx);
        let offset_after_append = view.read(cx).log_scroll.0.borrow().base_handle.offset().y;
        assert_eq!(
            offset_after_append,
            px(0.),
            "appending lines while scrolled up must not jump back to the bottom"
        );

        // User scrolls back to the bottom -> follow resumes on next update.
        {
            let state = view.read(cx).log_scroll.0.borrow();
            let max_y = state.base_handle.max_offset().y;
            assert!(max_y > px(0.), "log must overflow to test follow-bottom");
            state
                .base_handle
                .set_offset(gpui_kit::point(px(0.), -max_y));
        }
        window.render_frame(cx);
        view.update(cx, |this, cx| {
            this.accept_snapshot(this.generation, make_snap(70));
            cx.notify();
        });
        window.render_frame(cx);
        assert!(
            window
                .try_find("command-log-ccr-command-feedface-0001-70")
                .is_some_and(|row| row.visible()),
            "returning to the bottom resumes auto-follow"
        );
        window.remove_window();
    })
    .unwrap_or_else(|_| panic!("follow bottom"));
    owner.shutdown();
}

#[gpui_kit::test]
async fn ledger_observation_failure_offers_retry_and_reattaches(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let fixture = tempfile::tempdir().unwrap_or_else(|_| panic!("fixture"));
    let owner = RuntimeOwner::new().unwrap_or_else(|_| panic!("runtime"));
    let services = fixture_services(&owner, fixture.path());
    let jobs = services.command_jobs.clone();
    let (handle, view) = open(cx, services);

    // Trigger a genuine observation error by observing an unknown job id.
    cx.update_window(handle, |_, window, cx| {
        view.update(cx, |this, cx| {
            this.job_id = Some("ccr-command-missing00".into());
            this.refresh(cx);
        });
        window.render_frame(cx);
        assert!(window.try_find("command-ledger-observe-error").is_some());
        assert!(window.try_find("command-ledger-retry").is_some());
    })
    .unwrap_or_else(|_| panic!("observe failure"));

    // Admit a live job and verify clicking Retry re-attaches and restores live readouts.
    let admission = jobs
        .admit(
            RequestKey {
                environment: EnvironmentId::Local("fixture".into()),
                platform: Platform::Claude,
                generation: 2,
                request_id: 2,
            },
            ccr_app::jobs::policy::CommandExecutionRequest::background(
                "status".into(),
                Some(vec!["--json".into()]),
                None,
            ),
        )
        .unwrap_or_else(|_| panic!("admit retry"));
    let id = admission.snapshot.job_id.clone();
    cx.update_window(handle, |_, window, cx| {
        view.update(cx, |this, _| {
            this.job_id = Some(id.clone());
        });
        window.click("command-ledger-retry", cx);
        window.render_frame(cx);
        assert!(window.try_find("command-ledger-observe-error").is_none());
        assert!(window.try_find("command-ledger-retry").is_none());
        assert!(window.try_find("command-ledger-readouts").is_some());
        assert!(view.read(cx).busy);
        assert!(view.read(cx).task.is_some());
    })
    .unwrap_or_else(|_| panic!("retry reattach"));

    jobs.abandon(admission.ticket.unwrap_or_else(|| panic!("ticket")));
    cx.wait_for(handle, std::time::Duration::from_secs(5), |_, cx| {
        !view.read(cx).busy
    })
    .await;
    cx.update_window(handle, |_, window, _| window.remove_window())
        .unwrap_or_else(|_| panic!("close"));
    owner.shutdown();
}

#[gpui_kit::test]
async fn favorite_toggle_persists_across_reopen_confirms_removal_and_tracks_draft(
    cx: &mut TestAppContext,
) {
    cx.executor().allow_parking();
    let fixture = tempfile::tempdir().unwrap_or_else(|_| panic!("fixture"));
    let owner = RuntimeOwner::new().unwrap_or_else(|_| panic!("runtime"));
    let (handle, view) = open(cx, fixture_services(&owner, fixture.path()));
    cx.update_window(handle, |_, _, cx| {
        view.update(cx, |this, cx| this.open(cx));
    })
    .unwrap_or_else(|_| panic!("open records"));
    cx.wait_for(handle, std::time::Duration::from_secs(10), |_, cx| {
        let state = view.read(cx);
        state.records.loaded && !state.records.loading
    })
    .await;

    // Save a favorite from the editor without creating execution history.
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert_eq!(label(window, "command-collection-favorites"), "Favorites 0");
        assert_eq!(label(window, "command-collection-recent"), "Recent 0");
        assert!(window.try_find("favorite-add").is_none());
        assert!(window.try_find("tool-favorites").is_none());
        assert!(window.try_find("tool-recent").is_none());
        assert!(window.try_find("tool-catalog").is_none());

        window.click("command-row:history", cx);
        let args = view.read(cx).args.clone();
        set_input(&args, "-l 20", window, cx);
        window.render_frame(cx);
        assert_eq!(label(window, "command-favorite-toggle"), "Favorite");
        window.click("command-favorite-toggle", cx);
    })
    .unwrap_or_else(|_| panic!("save favorite"));

    cx.wait_for(handle, std::time::Duration::from_secs(10), |_, cx| {
        let state = view.read(cx);
        !state.records.saving
            && !state.records.loading
            && state.records.snapshot.favorites.len() == 1
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert_eq!(label(window, "command-favorite-toggle"), "Unfavorite");
        assert_eq!(label(window, "command-collection-favorites"), "Favorites 1");
        assert!(
            view.read(cx).records.snapshot.history.is_empty(),
            "saving a favorite must never invent execution history"
        );
        window.remove_window();
    })
    .unwrap_or_else(|_| panic!("close before reopen"));
    assert!(fixture.path().join("ui-state.db").exists());

    // Reopen a brand new Services + JobsView against the same database.
    let (handle, view) = open(cx, fixture_services(&owner, fixture.path()));
    cx.update_window(handle, |_, _, cx| {
        view.update(cx, |this, cx| this.open(cx));
    })
    .unwrap_or_else(|_| panic!("reopen records"));
    cx.wait_for(handle, std::time::Duration::from_secs(10), |_, cx| {
        let state = view.read(cx);
        state.records.loaded && !state.records.loading
    })
    .await;

    let fav_id = cx.read(|cx| view.read(cx).records.snapshot.favorites[0].id.clone());
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert_eq!(label(window, "command-collection-favorites"), "Favorites 1");
        window.click("command-collection-favorites", cx);
        window.render_frame(cx);
        assert!(
            label(window, "command-favorites-note").contains("Do not save credentials"),
            "favorites segment warns against storing credentials"
        );
        let row_id = SharedString::from(format!("favorite:{fav_id}"));
        assert!(label(window, row_id.clone()).contains("ccr history -l 20"));

        // Loading a favorite populates editor + args, keeps the segment, and is not a dirty draft.
        window.click(row_id, cx);
        window.render_frame(cx);
        assert_eq!(view.read(cx).selected, "history");
        assert_eq!(view.read(cx).args.read(cx).value().as_str(), "-l 20");
        assert_eq!(
            window.find("command-collection-favorites").checked(),
            Some(true),
            "loading a favorite keeps the Favorites segment active"
        );
        assert!(
            !view.read(cx).has_environment_draft(cx),
            "loaded favorite args are not treated as an environment draft"
        );
        assert_eq!(label(window, "command-favorite-toggle"), "Unfavorite");

        // Modifying args makes it a draft and flips the toggle back to Favorite.
        let args = view.read(cx).args.clone();
        set_input(&args, "-l 10", window, cx);
        window.render_frame(cx);
        assert!(view.read(cx).has_environment_draft(cx));
        assert_eq!(label(window, "command-favorite-toggle"), "Favorite");

        // Inline removal confirmation: initial focus on Cancel, Esc restores trigger, Confirm removes.
        let remove_id = SharedString::from(format!("remove-favorite:{fav_id}"));
        window.click(remove_id.clone(), cx);
        window.render_frame(cx);
        window.simulate_next_frame(cx);
        window.render_frame(cx);
        assert!(window.find("command-confirm-apply").visible());
        assert_eq!(
            window.find("command-confirm-cancel").focused(),
            Some(true),
            "confirmation starts keyboard focus on Cancel"
        );
        window.press("escape", cx);
        window.render_frame(cx);
        assert!(view.read(cx).records.confirmation.is_none());
        assert_eq!(view.read(cx).records.snapshot.favorites.len(), 1);

        window.click(remove_id.clone(), cx);
        window.render_frame(cx);
        window.click("command-confirm-cancel", cx);
        window.render_frame(cx);
        assert!(view.read(cx).records.confirmation.is_none());
        assert_eq!(view.read(cx).records.snapshot.favorites.len(), 1);

        window.click(remove_id, cx);
        window.render_frame(cx);
        window.click("command-confirm-apply", cx);
    })
    .unwrap_or_else(|_| panic!("reopen and remove"));

    cx.wait_for(handle, std::time::Duration::from_secs(10), |_, cx| {
        let state = view.read(cx);
        state.records.snapshot.favorites.is_empty()
            && !state.records.loading
            && !state.records.saving
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert!(view.read(cx).records.region_focus.is_focused(window));
        assert!(window.try_find("command-favorites-empty").is_some());
        window.remove_window();
    })
    .unwrap_or_else(|_| panic!("empty favorites"));
    owner.shutdown();
}

#[gpui_kit::test]
async fn recent_refreshes_on_terminal_loads_unavailable_records_and_confirms_clear(
    cx: &mut TestAppContext,
) {
    cx.executor().allow_parking();
    let fixture = tempfile::tempdir().unwrap_or_else(|_| panic!("fixture"));
    // Seed favorites (including an obsolete command) and history through the DB owner.
    let (legacy_fav_id, preview_hist_id, runnable_hist_id) = {
        use ccr_db::{
            database, managers::ui_state::UiStateManager, models::ui_state::AddFavoriteRequest,
        };
        let home = fixture.path().join("seed-home");
        std::fs::create_dir(&home).unwrap_or_else(|e| panic!("seed home: {e}"));
        let pool = database::create_pool(&fixture.path().join("ui-state.db"), None)
            .unwrap_or_else(|e| panic!("seed pool: {e}"));
        {
            let conn = pool.get().unwrap_or_else(|e| panic!("seed conn: {e}"));
            database::migrations::run_all_migrations(&conn, &home)
                .unwrap_or_else(|e| panic!("seed migrations: {e}"));
        }
        let manager = UiStateManager::with_db(database::DbAccess::Pool(pool));
        let fav = manager
            .add_favorite(AddFavoriteRequest {
                command: "legacy-gone".into(),
                args: vec!["--old".into()],
                display_name: Some("Removed command".into()),
                module: "commands".into(),
            })
            .unwrap_or_else(|e| panic!("seed favorite: {e}"));
        let preview_hist = manager
            .add_history("list".into(), vec![], false, 42)
            .unwrap_or_else(|e| panic!("seed preview history: {e}"));
        let runnable_hist = manager
            .add_history("status".into(), vec![], true, 12)
            .unwrap_or_else(|e| panic!("seed runnable history: {e}"));
        (fav.id, preview_hist.id, runnable_hist.id)
    };

    let owner = RuntimeOwner::new().unwrap_or_else(|_| panic!("runtime"));
    let (handle, view) = open(cx, fixture_services(&owner, fixture.path()));
    cx.update_window(handle, |_, _, cx| {
        view.update(cx, |this, cx| this.open(cx));
    })
    .unwrap_or_else(|_| panic!("open seeded"));
    cx.wait_for(handle, std::time::Duration::from_secs(10), |_, cx| {
        let state = view.read(cx);
        state.records.loaded && !state.records.loading
    })
    .await;

    // 1. Stale/unknown favorite loads with reason and keeps Favorites segment.
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert_eq!(label(window, "command-collection-favorites"), "Favorites 1");
        assert_eq!(label(window, "command-collection-recent"), "Recent 2");
        window.click("command-collection-favorites", cx);
        window.render_frame(cx);
        window.click(SharedString::from(format!("favorite:{legacy_fav_id}")), cx);
        window.render_frame(cx);
        assert_eq!(view.read(cx).selected, "legacy-gone");
        assert_eq!(view.read(cx).args.read(cx).value().as_str(), "--old");
        assert_eq!(
            window.find("command-collection-favorites").checked(),
            Some(true)
        );
        assert!(window.try_find("command-run-blocked").is_some());
        assert_eq!(window.find("command-start").focused(), None);

        // 2. Preview-only Recent row loads with empty args (D1), shows Failed tag, keeps Recent segment.
        window.click("command-collection-recent", cx);
        window.render_frame(cx);
        assert!(
            window
                .try_find(SharedString::from(format!(
                    "history-failed:{preview_hist_id}"
                )))
                .is_some()
        );
        window.click(SharedString::from(format!("history:{preview_hist_id}")), cx);
        window.render_frame(cx);
        assert_eq!(view.read(cx).selected, "list");
        assert_eq!(view.read(cx).args.read(cx).value().as_str(), "");
        assert_eq!(
            window.find("command-collection-recent").checked(),
            Some(true)
        );
        assert!(label(window, "command-run-blocked").contains("Run this command in a terminal"));

        // 3. Load runnable Recent row and run Help; terminal refreshes Recent count automatically.
        window.click(
            SharedString::from(format!("history:{runnable_hist_id}")),
            cx,
        );
        window.render_frame(cx);
        assert_eq!(view.read(cx).selected, "status");
        window.click("command-help", cx);
    })
    .unwrap_or_else(|_| panic!("load stale and trigger help"));

    cx.wait_for(handle, std::time::Duration::from_secs(5), |_, cx| {
        let state = view.read(cx);
        !state.busy && !state.records.loading && state.records.snapshot.history.len() == 3
    })
    .await;

    // 4. Clear recent: keyboard focus on Clear recent -> Enter -> initial focus Cancel -> Esc restores trigger -> Confirm clears & keeps focus in list.
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert_eq!(label(window, "command-collection-recent"), "Recent 3");
        window.focus_next(cx);
        window.render_frame(cx);
        for _ in 0..20 {
            if window.find("command-history-clear").focused() == Some(true) {
                break;
            }
            window.press("tab", cx);
        }
        assert_eq!(window.find("command-history-clear").focused(), Some(true));
        window.press("enter", cx);
        window.render_frame(cx);
        window.simulate_next_frame(cx);
        window.render_frame(cx);
        assert!(window.find("command-confirm-apply").visible());
        assert_eq!(
            window.find("command-confirm-cancel").focused(),
            Some(true),
            "Clear recent confirmation puts initial focus on Cancel"
        );
        window.press("escape", cx);
        window.render_frame(cx);
        assert!(view.read(cx).records.confirmation.is_none());
        assert_eq!(
            window.find("command-history-clear").focused(),
            Some(true),
            "Esc returns focus to Clear recent"
        );
        assert_eq!(view.read(cx).records.snapshot.history.len(), 3);

        window.press("enter", cx);
        window.render_frame(cx);
        window.click("command-confirm-cancel", cx);
        window.render_frame(cx);
        assert_eq!(view.read(cx).records.snapshot.history.len(), 3);

        window.press("enter", cx);
        window.render_frame(cx);
        window.click("command-confirm-apply", cx);
    })
    .unwrap_or_else(|_| panic!("clear recent confirmation"));

    cx.wait_for(handle, std::time::Duration::from_secs(10), |_, cx| {
        let state = view.read(cx);
        !state.records.loading && !state.records.saving && state.records.snapshot.history.is_empty()
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert!(!view.read(cx).records.error);
        assert!(view.read(cx).records.region_focus.is_focused(window));
        assert!(window.try_find("command-recent-empty").is_some());
        window.remove_window();
    })
    .unwrap_or_else(|_| panic!("empty recent"));
    owner.shutdown();
}

/// Opens the command page alone at a given logical size.
fn open_at(
    cx: &mut TestAppContext,
    services: Services,
    width: f32,
    height: f32,
) -> (AnyWindowHandle, Entity<JobsView>) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        crate::theme::init(cx);
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(gpui_kit::Bounds::new(
                    gpui_kit::point(px(0.), px(0.)),
                    size(px(width), px(height)),
                ))),
                ..Default::default()
            },
            cx,
            |window, cx| cx.new(|cx| JobsView::new(services, window, cx)),
        )
        .unwrap_or_else(|_| panic!("window"))
    })
}

#[gpui_kit::test]
async fn narrow_window_drills_in_restores_focus_shows_running_banner_and_cancels_stale_callbacks(
    cx: &mut TestAppContext,
) {
    cx.executor().allow_parking();
    let fixture = tempfile::tempdir().unwrap_or_else(|_| panic!("fixture"));
    let owner = RuntimeOwner::new().unwrap_or_else(|_| panic!("runtime"));

    // 1. At exact 1200px split width: list and work column both render, no back button.
    let (wide_handle, _) = open_at(cx, fixture_services(&owner, fixture.path()), 1200., 640.);
    cx.update_window(wide_handle, |_, window, cx| {
        window.render_frame(cx);
        assert!(window.try_find("command-list").is_some());
        assert!(window.try_find("command-editor").is_some());
        assert!(window.try_find("command-ledger").is_some());
        assert!(window.try_find("command-back-to-list").is_none());
        let list_w = f32::from(window.find("command-list").bounds().size.width);
        assert!(
            (320.0..=400.0).contains(&list_w) && (list_w as i32) % 8 == 0,
            "wide list width {list_w} must be a multiple of 8 within 320..=400"
        );
        window.remove_window();
    })
    .unwrap_or_else(|_| panic!("wide 1200"));

    // 2. Below 1200px (960x640): mutually exclusive list -> drill-in work -> back to original row.
    let services = fixture_services(&owner, fixture.path());
    let jobs = services.command_jobs.clone();
    let (handle, view) = open_at(cx, services, 960., 640.);
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert!(window.try_find("command-list").is_some());
        assert!(window.try_find("command-editor").is_none());
        assert!(window.try_find("command-ledger").is_none());
        assert!(window.try_find("command-back-to-list").is_none());
        assert!(window.try_find("command-running-banner").is_none());

        // Drill in by clicking `history`.
        window.click("command-row:history", cx);
        window.render_frame(cx);
        assert!(window.try_find("command-list").is_none());
        assert!(window.try_find("command-editor").is_some());
        assert!(window.try_find("command-ledger").is_some());
        assert!(window.try_find("command-back-to-list").is_some());
        window.simulate_next_frame(cx);
        window.render_frame(cx);
        assert_eq!(
            window.find("command-back-to-list").focused(),
            Some(true),
            "drill-in moves next-frame focus to first control in work column"
        );

        // Return to list restores focus to the listbox with the original row selected.
        window.click("command-back-to-list", cx);
        window.render_frame(cx);
        assert!(window.try_find("command-list").is_some());
        assert!(window.try_find("command-editor").is_none());
        window.simulate_next_frame(cx);
        window.render_frame(cx);
        assert_eq!(
            window.find("command-listbox").focused(),
            Some(true),
            "Back to list restores focus to the list"
        );
        assert_eq!(
            window.find("command-row:history").selected(),
            Some(true),
            "original row stays selected after returning"
        );
    })
    .unwrap_or_else(|_| panic!("narrow drill-in"));

    // 3. Running job + back to list shows focusable status banner that re-opens work column.
    let admission = jobs
        .admit(
            RequestKey {
                environment: EnvironmentId::Local("fixture".into()),
                platform: Platform::Claude,
                generation: 1,
                request_id: 1,
            },
            ccr_app::jobs::policy::CommandExecutionRequest::background(
                "status".into(),
                Some(vec!["--json".into()]),
                None,
            ),
        )
        .unwrap_or_else(|_| panic!("admit"));
    let job_id = admission.snapshot.job_id.clone();
    cx.update_window(handle, |_, window, cx| {
        window.click("command-row:status", cx);
        view.update(cx, |this, cx| {
            this.job_id = Some(job_id.clone());
            this.refresh(cx);
        });
        window.render_frame(cx);
        window.click("command-back-to-list", cx);
        window.render_frame(cx);
        assert!(window.try_find("command-list").is_some());
        let banner = window.find("command-running-banner");
        assert!(banner.visible());
        assert!(banner.focused().is_some(), "running banner is focusable");
        assert!(label(window, "command-running-banner").contains("ccr status"));

        window.click("command-running-banner", cx);
        window.render_frame(cx);
        assert!(window.try_find("command-list").is_none());
        assert!(window.try_find("command-ledger").is_some());

        // 4. Stale focus callback after leaving page (suspend) must NOT steal focus.
        let outside = cx.focus_handle();
        window.click("command-back-to-list", cx);
        view.update(cx, |this, cx| this.suspend(cx));
        window.focus(&outside, cx);
        window.simulate_next_frame(cx);
        window.render_frame(cx);
        assert!(
            outside.is_focused(window),
            "stale focus callback after suspend must not run"
        );

        // 5. Stale focus callback after environment switch must NOT steal focus.
        window.click("command-row:status", cx);
        view.update(cx, |this, cx| this.clear_environment(window, cx));
        window.focus(&outside, cx);
        window.simulate_next_frame(cx);
        window.render_frame(cx);
        assert!(
            outside.is_focused(window),
            "stale focus callback after clear_environment must not run"
        );
        window.remove_window();
    })
    .unwrap_or_else(|_| panic!("running banner and stale callbacks"));
    jobs.abandon(admission.ticket.unwrap_or_else(|| panic!("ticket")));
    owner.shutdown();
}

#[gpui_kit::test]
fn ctrl_enter_in_arguments_runs_only_when_command_is_ready(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let fixture = tempfile::tempdir().unwrap_or_else(|_| panic!("fixture"));
    let owner = RuntimeOwner::new().unwrap_or_else(|_| panic!("runtime"));
    let (handle, view) = open(cx, fixture_services(&owner, fixture.path()));
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        // 1. Unclosed quote: Ctrl+Enter must not start.
        let args = view.read(cx).args.clone();
        set_input(&args, r#"-t "unclosed"#, window, cx);
        window.render_frame(cx);
        window.click("command-args", cx);
        window.press("ctrl-enter", cx);
        assert!(view.read(cx).job_id.is_none());

        // 2. Non-runnable command: Ctrl+Enter must not start.
        window.click("command-filter-dedicated", cx);
        window.render_frame(cx);
        window.click("command-row:delete", cx);
        window.render_frame(cx);
        window.click("command-args", cx);
        window.press("ctrl-enter", cx);
        assert!(view.read(cx).job_id.is_none());

        // 3. Runnable command with valid args: Ctrl+Enter runs it.
        window.click("command-filter-runnable", cx);
        window.render_frame(cx);
        window.click("command-row:status", cx);
        set_input(&args, "--json", window, cx);
        window.render_frame(cx);
        window.click("command-args", cx);
        window.press("ctrl-enter", cx);
        let snapshot = view
            .read(cx)
            .snapshot
            .clone()
            .unwrap_or_else(|| panic!("ctrl-enter should start job"));
        assert_eq!(snapshot.command, "status");
        assert_eq!(snapshot.args, ["--json"]);
        window.remove_window();
    })
    .unwrap_or_else(|_| panic!("ctrl-enter"));
    owner.shutdown();
}

#[gpui_kit::test]
async fn f6_region_cycle_covers_command_list_and_work_column(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let fixture = tempfile::tempdir().unwrap_or_else(|_| panic!("fixture"));
    let owner = RuntimeOwner::new().unwrap_or_else(|_| panic!("runtime"));
    let services = fixture_services(&owner, fixture.path());
    let (handle, shell) = cx.update(|cx| {
        gpui_kit::init(cx);
        crate::theme::init(cx);
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(gpui_kit::Bounds::new(
                    gpui_kit::point(px(0.), px(0.)),
                    size(px(1280.), px(800.)),
                ))),
                ..Default::default()
            },
            cx,
            |window, cx| cx.new(|cx| crate::shell::Shell::new(services, window, cx)),
        )
        .unwrap_or_else(|_| panic!("shell window"))
    });
    cx.wait_for(handle, std::time::Duration::from_secs(10), |window, _| {
        window.try_find("nav-commands").is_some()
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.click("nav-commands", cx);
    })
    .unwrap_or_else(|_| panic!("open commands"));
    cx.wait_for(handle, std::time::Duration::from_secs(10), |window, _| {
        window.try_find("command-listbox").is_some()
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        // Start from sidebar nav -> F6 -> command list -> F6 -> work column -> F6 -> status.
        window.click("nav-commands", cx);
        window.render_frame(cx);
        let _ = &shell;
        window.press("f6", cx);
        assert_eq!(
            window.find("command-listbox").focused(),
            Some(true),
            "first F6 from nav focuses command list"
        );
        window.press("f6", cx);
        assert!(
            window.find("command-help").focused() == Some(true)
                || window.find("command-favorite-toggle").focused() == Some(true)
                || window.find("command-start").focused() == Some(true)
                || window.find("command-args").focused() == Some(true),
            "second F6 moves from command list into work column"
        );
        window.press("f6", cx);
        assert!(
            window.find("command-listbox").focused() != Some(true)
                && window.find("command-help").focused() != Some(true),
            "third F6 leaves work column to status bar"
        );
        // Backward Shift+F6: status -> work column -> command list -> nav.
        window.press("shift-f6", cx);
        assert!(
            window.find("command-help").focused() == Some(true)
                || window.find("command-favorite-toggle").focused() == Some(true)
                || window.find("command-start").focused() == Some(true)
                || window.find("command-args").focused() == Some(true),
            "Shift+F6 from status enters work column"
        );
        window.press("shift-f6", cx);
        assert_eq!(
            window.find("command-listbox").focused(),
            Some(true),
            "Shift+F6 from work column returns to command list"
        );
        window.remove_window();
    })
    .unwrap_or_else(|_| panic!("f6 cycle"));
    owner.shutdown();
}
