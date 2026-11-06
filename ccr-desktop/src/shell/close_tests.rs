#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::Shell;
use crate::runtime::{Services, profiles::tests::fixture};
use ccr_app::runtime::{Completion, EnvironmentId, Platform, RequestKey, RuntimeOwner};
use gpui_kit::test::TestWindowExt;
use gpui_kit::{
    AppContext, Bounds, TestAppContext, VisualTestContext, WindowBounds, WindowOptions, px, size,
};
use std::{
    sync::mpsc,
    time::{Duration, Instant},
};

fn wait_for(
    cx: &mut TestAppContext,
    handle: gpui_kit::AnyWindowHandle,
    mut predicate: impl FnMut(&mut gpui_kit::Window) -> bool,
) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        cx.run_until_parked();
        if cx
            .update_window(handle, |_, window, cx| {
                window.render_frame(cx);
                predicate(window)
            })
            .unwrap()
        {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "rendered fixture setup timed out"
        );
        cx.executor().advance_clock(Duration::from_millis(10));
        std::thread::yield_now();
    }
}

#[gpui_kit::test]
fn close_request_requires_confirmation_and_cancel_preserves_window(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let (root, source) = fixture();
    let owner = RuntimeOwner::new().unwrap();
    for language in [
        crate::locales::Language::English,
        crate::locales::Language::Chinese,
    ] {
        let (cancel_label, quit_label) = match language {
            crate::locales::Language::English => ("Keep working", "Quit"),
            crate::locales::Language::Chinese => ("继续使用", "退出"),
        };
        let (handle, _) = cx.update(|cx| {
            gpui_kit::init(cx);
            cx.set_reduce_motion(true);
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
                        Shell::new(Services::new(owner.handle(), source.clone()), window, cx)
                    })
                },
            )
            .unwrap()
        });
        let mut visual = VisualTestContext::from_window(handle, cx);
        visual.update(|_, cx| cx.set_global(language));
        assert!(!visual.simulate_close());
        visual.update(|window, cx| {
            window.press("enter", cx);
            assert!(
                window.try_find("dialog").is_none(),
                "root Enter must safely cancel"
            );
        });
        assert!(!visual.simulate_close());
        visual.update(|window, cx| {
            window.render_frame(cx);
            window.simulate_next_frame(cx);
            window.render_frame(cx);
            assert!(window.try_find("dialog").is_some(), "close must ask first");
            assert_eq!(window.find("cancel").label(), Some(cancel_label));
            assert_eq!(window.find("cancel").focused(), Some(true));
            window.press("tab", cx);
            assert_eq!(window.find("ok").focused(), Some(true));
            window.simulate_next_frame(cx);
            window.render_frame(cx);
            assert_eq!(
                window.find("ok").focused(),
                Some(true),
                "no late focus theft"
            );
            window.press("tab", cx);
            assert_eq!(window.find("cancel").focused(), Some(true));
            window.press("shift-tab", cx);
            assert_eq!(window.find("ok").focused(), Some(true));
            window.press("escape", cx);
            assert!(window.try_find("dialog").is_none());
        });
        // Dismiss and reopen before the old opening's next-frame callback runs.
        assert!(!visual.simulate_close());
        visual.update(|window, cx| {
            window.render_frame(cx);
            window.press("escape", cx);
        });
        assert!(!visual.simulate_close());
        visual.update(|window, cx| {
            window.render_frame(cx);
            window.press("tab", cx);
            window.press("tab", cx);
            assert_eq!(window.find("ok").focused(), Some(true));
            window.simulate_next_frame(cx);
            window.render_frame(cx);
            assert_eq!(
                window.find("ok").focused(),
                Some(true),
                "dismissed opening must not steal focus after reopen"
            );
            window.press("shift-tab", cx);
            window.press("enter", cx);
            assert!(window.try_find("dialog").is_none());
        });
        assert!(!visual.simulate_close());
        assert!(!visual.simulate_close());
        visual.update(|window, cx| {
            window.render_frame(cx);
            window.click("cancel", cx);
            assert!(window.try_find("dialog").is_none(), "close must not stack");
        });
        assert!(!visual.simulate_close());
        visual.update(|window, cx| {
            window.render_frame(cx);
            window.simulate_next_frame(cx);
            window.render_frame(cx);
            assert_eq!(window.find("cancel").label(), Some(cancel_label));
            assert_eq!(window.find("ok").label(), Some(quit_label));
            assert_eq!(window.find("cancel").focused(), Some(true));
            window.press("enter", cx);
            assert!(
                window.try_find("dialog").is_none(),
                "Enter on Cancel stays open"
            );
        });
        assert!(!visual.simulate_close());
        visual.update(|window, cx| {
            window.render_frame(cx);
            window.simulate_next_frame(cx);
            window.press("tab", cx);
            assert_eq!(window.find("ok").focused(), Some(true));
            window.press("enter", cx);
        });
        assert!(cx.update_window(handle, |_, _, _| ()).is_err());
    }
    owner.shutdown();
    drop(root);
}

struct ShutdownOnDrop(Option<RuntimeOwner>);
impl Drop for ShutdownOnDrop {
    fn drop(&mut self) {
        if let Some(owner) = self.0.take() {
            owner.shutdown();
        }
    }
}

#[gpui_kit::test]
fn cancelling_close_preserves_nested_profile_confirmation_draft_and_running_job(
    cx: &mut TestAppContext,
) {
    cx.executor().allow_parking();
    let (_root, source) = fixture();
    let context = crate::runtime::profiles::context(&source, ccr_config::Platform::Claude).unwrap();
    let revision = context.manager().snapshot().unwrap().version;
    let mut owner = ShutdownOnDrop(Some(RuntimeOwner::new().unwrap()));
    let executor = owner.0.as_ref().unwrap().handle();
    let (handle, _) = cx.update(|cx| {
        gpui_kit::init(cx);
        cx.set_reduce_motion(true);
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
                cx.new(|cx| Shell::new(Services::new(executor.clone(), source), window, cx))
            },
        )
        .unwrap()
    });
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("nav-claude-code", cx);
    })
    .unwrap();
    wait_for(cx, handle, |window| {
        window.try_find("configuration:alpha").is_some() && executor.active_tasks() == 0
    });
    cx.update_window(handle, |_, window, cx| {
        window.click("configuration:alpha", cx)
    })
    .unwrap();
    wait_for(cx, handle, |window| {
        // Kit snapshots expose Some(true) for disabled and None when cleared.
        // Successful editor input below additionally proves input acceptance.
        window.find("profile-edit").disabled() != Some(true)
    });
    cx.update_window(handle, |_, window, cx| {
        window.click("profile-edit", cx);
        window.click("profile-description", cx);
        window.press("ctrl-a", cx);
        window.input("Unconfirmed draft", cx);
        for _ in 0..12 {
            window.press("tab", cx);
            if window.find("profile-advanced").focused() == Some(true) {
                break;
            }
        }
        assert_eq!(window.find("profile-advanced").focused(), Some(true));
        window.press("enter", cx);
        window.simulate_next_frame(cx);
        window.render_frame(cx);
        assert_eq!(window.find("profile-confirm-cancel").focused(), Some(true));
    })
    .unwrap();
    let (entered_tx, entered_rx) = mpsc::channel();
    let job = executor
        .job(
            RequestKey {
                environment: EnvironmentId::Local("close-fixture".into()),
                platform: Platform::Claude,
                generation: 0,
                request_id: 42,
            },
            move |cancel, _| {
                entered_tx.send(()).unwrap();
                tokio::runtime::Handle::current().block_on(cancel.cancelled());
                Completion::<(), ()>::Cancelled
            },
        )
        .unwrap();
    entered_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    let mut visual = VisualTestContext::from_window(handle, cx);
    for cancel in ["escape", "click"] {
        assert!(!visual.simulate_close());
        visual.update(|window, cx| {
            window.render_frame(cx);
            window.simulate_next_frame(cx);
            window.render_frame(cx);
            assert_eq!(window.find("cancel").focused(), Some(true));
            if cancel == "escape" {
                window.press("escape", cx);
            } else {
                window.click("cancel", cx);
            }
            assert!(window.try_find("dialog").is_none());
            assert!(window.try_find("profile-confirmation").is_some());
            assert_eq!(window.find("profile-confirm-cancel").focused(), Some(true));
            assert_eq!(
                window.find("profile-description").value(),
                Some("Unconfirmed draft")
            );
        });
        assert!(!job.is_cancel_requested());
        assert!(job.terminal().is_none());
        assert_eq!(context.manager().snapshot().unwrap().version, revision);
    }
    assert!(!visual.simulate_close());
    visual.update(|window, cx| {
        window.render_frame(cx);
        window.click("ok", cx);
    });
    assert!(cx.update_window(handle, |_, _, _| ()).is_err());
    // TestAppContext has no real OS event loop; final join is exercised explicitly.
    owner.0.take().unwrap().shutdown();
    assert!(matches!(
        job.terminal().as_deref(),
        Some(Completion::Cancelled)
    ));
    assert_eq!(executor.active_tasks(), 0);
    assert_eq!(context.manager().snapshot().unwrap().version, revision);
}
