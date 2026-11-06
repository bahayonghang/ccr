//! Controlled DTOs only: never inspect a host PID, real callback port or identity.
#![allow(clippy::unwrap_used)]
use super::*;
use gpui_kit::{
    TestAppContext, WindowOptions,
    test::{TestAppContextExt, TestWindowExt},
};
use std::time::Duration;

#[gpui_kit::test]
async fn codex_auth_reviews_warn_without_blocking_and_cleanup_never_takes_unknown_pids(
    cx: &mut TestAppContext,
) {
    cx.executor().allow_parking();
    let root = tempfile::tempdir().unwrap();
    let roots = ccr_cli::application::platform_roots::PlatformRoots::fixture(
        &root.path().join("desktop-platform-fixture"),
    )
    .unwrap();
    std::fs::create_dir_all(&roots.codex_runtime).unwrap();
    std::fs::write(
        roots.codex_runtime.join("auth.json"),
        r#"{"OPENAI_API_KEY":"controlled-diagnostic-fixture-key"}"#,
    )
    .unwrap();
    let owner = ccr_app::runtime::RuntimeOwner::new().unwrap();
    let services = Services::new(
        owner.handle(),
        crate::runtime::ConfigSource::Fixture {
            profiles: root.path().join("profiles.toml"),
            registry: root.path().join("registry.toml"),
            delay: Duration::ZERO,
        },
    );
    let (handle, view) = cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
            cx.new(|cx| {
                let mut view = PlatformsView::new(services, window, cx);
                view.open(Platform::Codex, Surface::Auth, window, cx);
                view
            })
        })
        .unwrap()
    });
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| !view.read(cx).busy)
        .await;
    assert!(view.read_with(cx, |this, _| this.diagnostics.warning.is_none()));
    assert!(
        view.read_with(cx, |this, _| this.diagnostics.port.is_none()),
        "mounting auth must not probe the host"
    );
    cx.update_window(handle, |_, window, cx| {
        view.update(cx, |this, cx| {
            this.name.update(cx, |state, cx| {
                state.set_value("controlled-saved", window, cx)
            })
        });
        window.render_frame(cx);
        window.click("platform-account-name", cx);
        window.click("platform-account-save", cx);
        window.press("enter", cx);
        assert!(
            !roots
                .codex_store
                .join("auth/controlled-saved.json")
                .exists()
        );
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        !view.read(cx).diagnostics.busy
    })
    .await;
    assert_eq!(
        view.read_with(cx, |this, _| this.diagnostics.warning.clone()),
        Some(Err(Error::Isolated))
    );
    view.update(cx, |this, cx| {
        this.apply_diagnostic(
            Intent::SaveWarning,
            Ok(Report::Processes(ProcessWarning {
                pids: vec![41, 42],
                additional: 2,
            })),
            cx,
        )
    });
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.simulate_next_frame(cx);
        window.render_frame(cx);
        assert_eq!(window.find("codex-diagnostic-cancel").focused(), Some(true));
        assert!(
            window
                .find("codex-save-process-warning")
                .label()
                .unwrap()
                .contains("41, 42")
        );
        window.press("tab", cx);
        assert_eq!(
            window.find("codex-diagnostic-confirm").focused(),
            Some(true)
        );
        window.press("shift-tab", cx);
        window.press("escape", cx);
        window.render_frame(cx);
        assert_eq!(window.find("platform-account-name").focused(), Some(true));
        assert!(window.try_find("codex-diagnostic-review").is_none());
        window.click("platform-account-save", cx);
        window.render_frame(cx);
        // Save warning is advisory even before the isolated diagnostic completes.
        window.click("codex-diagnostic-confirm", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| !view.read(cx).busy)
        .await;
    assert!(
        roots
            .codex_store
            .join("auth/controlled-saved.json")
            .exists()
    );
    view.update(cx, |this, cx| this.load(cx));
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| !view.read(cx).busy)
        .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("codex-port-check", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        !view.read(cx).diagnostics.busy
    })
    .await;
    assert_eq!(
        view.read_with(cx, |this, _| this.diagnostics.port.clone()),
        Some(Err(Error::Isolated))
    );
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("codex-port-review-cleanup", cx);
        window.render_frame(cx);
        window.press("enter", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        !view.read(cx).diagnostics.busy
    })
    .await;
    assert_eq!(
        view.read_with(cx, |this, _| this.diagnostics.cleanup_error),
        Some(Error::Isolated)
    );
    cx.update_window(handle, |_, window, cx| {
        window.simulate_next_frame(cx);
        window.render_frame(cx);
        window.press("escape", cx);
        window.render_frame(cx);
        window.click("codex-port-review-cleanup", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        !view.read(cx).diagnostics.busy
    })
    .await;
    // Controlled captured consent tests Confirm wiring. The fixture runtime must
    // still refuse before cleanup, fixed-port bind or external tool execution.
    let plan = ccr_cli::application::codex_oauth::NativeSession::default()
        .prepare_cleanup(&roots)
        .unwrap();
    view.update(cx, |this, cx| {
        this.apply_diagnostic(
            Intent::PrepareCleanup,
            Ok(Report::CleanupPlan(Arc::new(plan))),
            cx,
        )
    });
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("codex-diagnostic-confirm", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        !view.read(cx).diagnostics.busy
    })
    .await;
    assert_eq!(
        view.read_with(cx, |this, _| this.diagnostics.release.clone()),
        Some(Err(Error::Isolated))
    );
    view.update(cx, |this, cx| {
        this.apply_diagnostic(
            Intent::CancelOwned,
            Ok(Report::Release(ReleaseReport {
                owned: vec![41],
                unknown: vec![42],
                cancel_requested: 1,
                port: PortState::Occupied,
            })),
            cx,
        )
    });
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert!(window.try_find("codex-port-status").is_some());
        window.click("platform-account-save", cx);
        view.update(cx, |this, cx| this.leave(window, cx));
        window.render_frame(cx);
    })
    .unwrap();
    cx.run_until_parked();
    assert!(view.read_with(cx, |this, _| this.diagnostics.review.is_none()));
    assert!(view.read_with(cx, |this, _| this.diagnostics.warning.is_none()));
    assert!(view.read_with(cx, |this, _| this.diagnostics.release.is_none()));
    assert!(view.read_with(cx, |this, _| this.session.is_none()));
    cx.update_window(handle, |_, window, _| window.remove_window())
        .unwrap();
    owner.shutdown();
}
