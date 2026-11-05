use super::*;
use crate::runtime::install::tests::{fixture, key, wait_until};
use ccr_app::runtime::RuntimeOwner;
use gpui_kit::test::{TestAppContextExt, TestWindowExt};
use gpui_kit::{AppContext, TestAppContext, WindowOptions};
use std::sync::atomic::Ordering;

#[gpui_kit::test]
fn install_confirmation_enters_cancel_escape_restores_focus_and_navigation_clears_decision(
    cx: &mut TestAppContext,
) {
    cx.executor().allow_parking();
    let owner = RuntimeOwner::new().unwrap_or_else(|_| panic!("runtime"));
    let (services, _, controls) = fixture(&owner);
    let (handle, view) = cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
            cx.new(|cx| InstallView::new(services, window, cx))
        })
        .unwrap_or_else(|_| panic!("window"))
    });
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        view.update(cx, |this, cx| this.focus_main(window, cx));
        window.press("tab", cx);
        assert_eq!(window.find("install-review").focused(), Some(true));
        window.click("install-review", cx);
        window.simulate_next_frame(cx);
        window.render_frame(cx);
        window.simulate_next_frame(cx);
        window.render_frame(cx);
        assert!(view.read(cx).decision.is_some());
        assert_eq!(window.find("install-decision-cancel").focused(), Some(true));
        window.press("tab", cx);
        assert_eq!(
            window.find("install-decision-confirm").focused(),
            Some(true)
        );
        window.press("escape", cx);
        assert!(view.read(cx).decision.is_none());
        assert_eq!(window.find("install-review").focused(), Some(true));
        window.click("install-review", cx);
        view.update(cx, |this, cx| this.suspend(cx));
        window.render_frame(cx);
        window.render_frame(cx);
        assert!(view.read(cx).decision.is_none());
        assert_eq!(controls.starts.load(Ordering::SeqCst), 0);
        window.remove_window();
    })
    .unwrap_or_else(|_| panic!("confirmation"));
    owner.shutdown();
}

#[gpui_kit::test]
fn rendered_fixture_install_controls_never_probe_or_execute(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let owner = RuntimeOwner::new().unwrap_or_else(|_| panic!("runtime"));
    let root = tempfile::tempdir().unwrap_or_else(|_| panic!("fixture"));
    let services = Services::new(
        owner.handle(),
        crate::runtime::ConfigSource::Fixture {
            profiles: root.path().join("profiles.toml"),
            registry: root.path().join("config.toml"),
            delay: std::time::Duration::ZERO,
        },
    );
    let (handle, view) = cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
            cx.new(|cx| InstallView::new(services, window, cx))
        })
        .unwrap_or_else(|_| panic!("window"))
    });
    cx.update_window(handle, |_, window, cx| {
        view.update(cx, |this, cx| this.refresh(cx));
        window.render_frame(cx);
        window.click("install-check", cx);
        window.click("install-review", cx);
        assert!(view.read(cx).snapshot.detection.is_none());
        assert!(view.read(cx).decision.is_none());
        assert!(!view.read(cx).loading);
        window.remove_window();
    })
    .unwrap_or_else(|_| panic!("fixture controls"));
    assert_eq!(
        root.path()
            .read_dir()
            .unwrap_or_else(|_| panic!("read fixture"))
            .count(),
        0
    );
    owner.shutdown();
}

#[gpui_kit::test]
fn rendered_return_observes_retained_running_attempt_without_restarting(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let owner = RuntimeOwner::new().unwrap_or_else(|_| panic!("runtime"));
    let (services, id, controls) = fixture(&owner);
    let requests = InstallRequests::new(&services);
    let (handle, view) = cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
            cx.new(|cx| InstallView::new(services, window, cx))
        })
        .unwrap_or_else(|_| panic!("window"))
    });
    let subscription = requests
        .execute(key(1), id, true)
        .unwrap_or_else(|_| panic!("execute"));
    wait_until(|| controls.started.load(Ordering::SeqCst));
    drop(subscription);
    cx.update_window(handle, |_, window, cx| {
        view.update(cx, |this, cx| this.refresh(cx));
        assert!(view.read(cx).snapshot.running);
        view.update(cx, |this, cx| this.suspend(cx));
        assert!(view.read(cx).observer.is_none());
        assert!(requests.snapshot().running);
        view.update(cx, |this, cx| this.refresh(cx));
        assert!(view.read(cx).observer.is_some());
        assert!(view.read(cx).snapshot.running);
        assert_eq!(controls.starts.load(Ordering::SeqCst), 1);
        window.remove_window();
    })
    .unwrap_or_else(|_| panic!("revisit"));
    controls.complete.notify_one();
    wait_until(|| !requests.snapshot().busy());
    owner.shutdown();
}

#[gpui_kit::test]
async fn rendered_confirm_cancel_and_revisit_receive_terminal_after_cleanup(
    cx: &mut TestAppContext,
) {
    cx.executor().allow_parking();
    let owner = RuntimeOwner::new().unwrap_or_else(|_| panic!("runtime"));
    let (services, _, controls) = fixture(&owner);
    let (handle, view) = cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
            cx.new(|cx| InstallView::new(services, window, cx))
        })
        .unwrap_or_else(|_| panic!("window"))
    });
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("install-review", cx);
        window.simulate_next_frame(cx);
        window.render_frame(cx);
        window.click("install-decision-confirm", cx);
    })
    .unwrap_or_else(|_| panic!("confirm"));
    cx.wait_for(handle, std::time::Duration::from_secs(5), |_, cx| {
        view.read(cx).snapshot.running
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("install-cancel", cx);
        assert!(view.read(cx).snapshot.cancel_requested);
        assert!(!view.read(cx).loading);
        view.update(cx, |this, cx| this.suspend(cx));
        view.update(cx, |this, cx| this.refresh(cx));
        assert!(view.read(cx).snapshot.running);
        assert!(view.read(cx).snapshot.recent.terminal.is_none());
    })
    .unwrap_or_else(|_| panic!("cancel/revisit"));
    wait_until(|| controls.cancelled.load(Ordering::SeqCst));
    controls.cleanup.notify_one();
    cx.wait_for(handle, std::time::Duration::from_secs(5), |_, cx| {
        !view.read(cx).snapshot.busy() && view.read(cx).snapshot.recent.terminal.is_some()
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        assert!(matches!(
            view.read(cx).snapshot.recent.terminal,
            Some(InstallEvent::Cancelled { .. })
        ));
        assert!(!view.read(cx).loading);
        assert_eq!(controls.starts.load(Ordering::SeqCst), 1);
        window.remove_window();
    })
    .unwrap_or_else(|_| panic!("terminal"));
    owner.shutdown();
}
