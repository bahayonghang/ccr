use super::*;
use ccr_app::runtime::RuntimeOwner;
use gpui_kit::test::{TestAppContextExt, TestWindowExt};
use gpui_kit::{TestAppContext, WindowOptions};

#[gpui_kit::test]
async fn revisit_reattaches_owner_and_rejects_old_generation_or_job(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let root = tempfile::tempdir().unwrap_or_else(|_| panic!("fixture"));
    let owner = RuntimeOwner::new().unwrap_or_else(|_| panic!("runtime"));
    let services = Services::new(
        owner.handle(),
        crate::runtime::ConfigSource::Fixture {
            profiles: root.path().join("profiles.toml"),
            registry: root.path().join("config.toml"),
            delay: std::time::Duration::ZERO,
        },
    );
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
fn rendered_catalog_blocks_typed_only_commands_and_invalid_draft_never_starts(
    cx: &mut TestAppContext,
) {
    cx.executor().allow_parking();
    let fixture = tempfile::tempdir().unwrap_or_else(|_| panic!("fixture"));
    let owner = RuntimeOwner::new().unwrap_or_else(|_| panic!("runtime"));
    let services = Services::new(
        owner.handle(),
        crate::runtime::ConfigSource::Fixture {
            profiles: fixture.path().join("profiles.toml"),
            registry: fixture.path().join("config.toml"),
            delay: std::time::Duration::ZERO,
        },
    );
    let (handle, view) = cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
            cx.new(|cx| JobsView::new(services, window, cx))
        })
        .unwrap_or_else(|_| panic!("window"))
    });
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("job-command-delete", cx);
        assert_eq!(
            view.read(cx).selected,
            "status",
            "typed destructive paths remain disabled in generic passthrough"
        );
        view.read(cx).args.clone().update(cx, |state, cx| {
            state.set_value("not-json-secret-sentinel", window, cx)
        });
        window.render_frame(cx);
        window.click("command-start", cx);
        assert!(view.read(cx).job_id.is_none());
        assert!(view.read(cx).snapshot.is_none());
        assert!(!view.read(cx).busy);
        window.click("command-foreground", cx);
        assert!(view.read(cx).job_id.is_none());
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
            "not-json-secret-sentinel"
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

fn labels(window: &gpui_kit::Window) -> String {
    gpui_kit::base::test_support::snapshots(window)
        .into_iter()
        .filter_map(|node| node.label().map(str::to_string))
        .collect::<Vec<_>>()
        .join("\n")
}

#[gpui_kit::test]
fn command_catalog_lists_real_entries_and_rejects_unavailable_commands(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let fixture = tempfile::tempdir().unwrap_or_else(|_| panic!("fixture"));
    let owner = RuntimeOwner::new().unwrap_or_else(|_| panic!("runtime"));
    let services = Services::new(
        owner.handle(),
        crate::runtime::ConfigSource::Fixture {
            profiles: fixture.path().join("profiles.toml"),
            registry: fixture.path().join("config.toml"),
            delay: std::time::Duration::ZERO,
        },
    );
    let count = ccr_app::jobs::policy::command_catalog().len();
    let (handle, view) = cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
            cx.new(|cx| JobsView::new(services, window, cx))
        })
        .unwrap_or_else(|_| panic!("window"))
    });
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        let text = labels(window);
        assert!(text.contains(&format!("The local command catalog has {count} commands.")));
        assert_eq!(
            window.find("job-command-status").label(),
            Some("status · Status overview · Can run")
        );
        assert!(
            window
                .find("job-command-delete")
                .label()
                .unwrap_or_default()
                .contains("Not available here")
        );
        assert!(text.contains("Status overview"));
        assert!(!text.contains("Home insights"));
        assert!(!text.contains("Apply / refresh"));
        assert!(!text.contains(" / "));
        window.click("job-command-delete", cx);
        window.render_frame(cx);
        assert_eq!(view.read(cx).selected, "status");
        assert!(
            window
                .find("command-catalog-detail")
                .label()
                .unwrap_or_default()
                .contains("Status overview")
        );
        view.update(cx, |this, cx| {
            this.catalog.clear();
            this.selected = "missing".into();
            this.start(cx);
        });
        window.render_frame(cx);
        let empty = labels(window);
        assert!(empty.contains("The local command catalog is empty."));
        assert!(empty.contains("cannot be started from this catalog"));
        assert!(view.read(cx).job_id.is_none());
        assert!(!empty.contains(" / "));
        window.remove_window();
    })
    .unwrap_or_else(|_| panic!("catalog"));
    owner.shutdown();
}
