#![allow(clippy::unwrap_used)]
use super::*;
use gpui_kit::{
    AppContext, TestAppContext, WindowBounds, WindowOptions, px, size, test::TestWindowExt,
};
#[gpui_kit::test]
fn s2_review_starts_with_safe_cancel_rejects_early_enter_and_dies_on_suspend(
    cx: &mut TestAppContext,
) {
    cx.executor().allow_parking();
    let root = tempfile::tempdir().unwrap();
    let runtime = ccr_app::runtime::RuntimeOwner::new().unwrap();
    let services = Services::new(
        runtime.handle(),
        crate::runtime::benchmark::stream::tests::source(root.path()),
    );
    let check = services.clone();
    let (handle, monitor) = cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(gpui_kit::Bounds::centered(
                    None,
                    size(px(1200.), px(1000.)),
                    cx,
                ))),
                ..Default::default()
            },
            cx,
            |window, cx| cx.new(|cx| super::super::MonitoringView::new(services, window, cx)),
        )
        .unwrap()
    });
    cx.update_window(handle, |_, window, cx| {
        let view = monitor.read(cx).benchmark.clone().unwrap();
        window.render_frame(cx);
        assert!(window.try_find("benchmark-s2-start").is_some());
        window.click("benchmark-s2-start", cx);
        window.render_frame(cx);
        let generation = view.read(cx).review.as_ref().unwrap().generation;
        assert!(!view.read(cx).review.as_ref().unwrap().armed);
        // Calling a stale/early button handler cannot admit the run either.
        view.update(cx, |view, cx| view.confirm_start(generation, window, cx));
        window.press("enter", cx);
        assert_eq!(
            check.benchmark_s2_status().unwrap().phase,
            Phase::NotStarted
        );
        assert!(!view.read(cx).error);
        window.simulate_next_frame(cx);
        window.render_frame(cx);
        assert_eq!(
            window.find("benchmark-s2-review-cancel").focused(),
            Some(true)
        );
        window.press("enter", cx);
        window.render_frame(cx);
        assert!(view.read(cx).review.is_none());
        assert_eq!(
            check.benchmark_s2_status().unwrap().phase,
            Phase::NotStarted
        );
        window.click("benchmark-s2-start", cx);
        window.render_frame(cx);
        let generation = view.read(cx).review.as_ref().unwrap().generation;
        view.update(cx, |view, cx| view.suspend(cx));
        window.simulate_next_frame(cx);
        window.render_frame(cx);
        view.update(cx, |view, cx| view.confirm_start(generation, window, cx));
        assert!(view.read(cx).review.is_none());
        assert!(!view.read(cx).error);
        assert_eq!(
            check.benchmark_s2_status().unwrap().phase,
            Phase::NotStarted
        );
        assert!(window.try_find("benchmark-s2-review-confirm").is_none());
        window.remove_window();
    })
    .unwrap();
    runtime.shutdown();
}
