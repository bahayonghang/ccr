//! Kit headless interaction evidence; OS IME, accessibility and pixels are separate checks.
use ccr_gpui::probe::Probe;
use gpui_kit::test::TestWindowExt;
use gpui_kit::{AppContext, TestAppContext, WindowOptions};

use ccr_app::runtime::{BLOCKING_CONCURRENCY, EnvironmentId, Platform, RequestKey, RuntimeOwner};
use ccr_gpui::{
    app::ConfigurationApp,
    runtime::{ConfigSource, Services},
};
use std::{
    sync::{Arc, Barrier, mpsc},
    time::Duration,
};

#[gpui_kit::test]
fn input_retains_unicode_across_redraw_and_window_closes(cx: &mut TestAppContext) {
    let handle = cx.update(|cx| {
        gpui_kit::init(cx);
        let (handle, _) = gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
            cx.new(|cx| Probe::new(window, cx))
        })
        .unwrap_or_else(|error| panic!("test window: {error}"));
        handle
    });
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("probe-input", cx);
        window.input("CCR 中文🦀", cx);
        assert_eq!(window.find("probe-input").value(), Some("CCR 中文🦀"));
        assert_eq!(window.find("probe-input").focused(), Some(true));
        window.render_frame(cx);
        assert_eq!(window.find("probe-input").value(), Some("CCR 中文🦀"));
        window.press("backspace", cx);
        assert_eq!(window.find("probe-input").value(), Some("CCR 中文"));
        window.remove_window();
    })
    .unwrap_or_else(|error| panic!("test interaction: {error}"));
    assert!(cx.update_window(handle, |_, _, _| ()).is_err());
}

#[gpui_kit::test]
fn pending_catalog_read_keeps_real_filter_interactive_and_close_reclaims_it(
    cx: &mut TestAppContext,
) {
    // This integration test deliberately mixes GPUI with the actual Tokio
    // worker pool. Barriers control work; permit cross-runtime wakeups instead
    // of claiming the OS threads run on GPUI's deterministic test scheduler.
    cx.executor().allow_parking();
    let owner = RuntimeOwner::new().unwrap_or_else(|error| panic!("runtime: {error}"));
    let executor = owner.handle();
    let barrier = Arc::new(Barrier::new(BLOCKING_CONCURRENCY + 1));
    let mut release = Vec::new();
    let mut workers = Vec::new();
    // Saturate the finite blocking workers with controlled work, never sleep.
    for request_id in 0..BLOCKING_CONCURRENCY {
        let barrier = barrier.clone();
        let (tx, rx) = mpsc::channel();
        release.push(tx);
        workers.push(
            executor
                .read(
                    RequestKey {
                        environment: EnvironmentId::Local("fixture".into()),
                        platform: Platform::Claude,
                        generation: 0,
                        request_id: request_id as u64,
                    },
                    move |_, _| {
                        barrier.wait();
                        rx.recv().unwrap_or_else(|error| panic!("release: {error}"));
                        Ok::<_, ()>(())
                    },
                )
                .unwrap_or_else(|error| panic!("admission: {error:?}")),
        );
    }
    barrier.wait();
    let (handle, view) = cx.update(|cx| {
        gpui_kit::init(cx);
        let services = Services::new(
            executor.clone(),
            ConfigSource::Fixture {
                profiles: "never-read-before-cancel/profiles.toml".into(),
                registry: "never-read-before-cancel/config.toml".into(),
                delay: Duration::ZERO,
            },
        );
        let (handle, view) = gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
            cx.new(|cx| ConfigurationApp::new(services, window, cx))
        })
        .unwrap_or_else(|error| panic!("test window: {error}"));
        (handle, view)
    });
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        // Kit's disabled button does not expose an AccessKit disabled flag in
        // 0.7.3. This test checks the real loading surface; native semantics
        // remain an explicit shell/a11y acceptance item.
        assert_eq!(
            window.find("configuration-refresh").label(),
            Some("Reading")
        );
        let generation = view.read(cx).read_generation();
        window.click("configuration-refresh", cx);
        assert_eq!(
            view.read(cx).read_generation(),
            generation,
            "disabled refresh must not admit another read"
        );
        window.click("configuration-filter", cx);
        window.input("原生读取", cx);
        assert_eq!(
            window.find("configuration-filter").value(),
            Some("原生读取")
        );
        assert_eq!(window.find("configuration-filter").focused(), Some(true));
        window.press("backspace", cx);
        assert_eq!(window.find("configuration-filter").value(), Some("原生读"));
        window.remove_window();
    })
    .unwrap_or_else(|error| panic!("test interaction: {error}"));
    assert!(cx.update_window(handle, |_, _, _| ()).is_err());
    drop(view);
    cx.run_until_parked();
    for tx in release {
        tx.send(())
            .unwrap_or_else(|error| panic!("release: {error}"));
    }
    drop(workers);
    owner.shutdown();
    assert_eq!(executor.active_tasks(), 0);
}
