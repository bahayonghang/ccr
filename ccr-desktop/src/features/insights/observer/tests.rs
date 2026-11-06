#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::*;
use crate::runtime::ConfigSource;
use ccr_app::runtime::RuntimeOwner;
use gpui_kit::{
    TestAppContext, WindowOptions,
    test::{TestAppContextExt, TestWindowExt},
};
use std::time::Duration;

#[gpui_kit::test]
async fn observer_setup_confirmation_cancels_then_saves_without_a_usage_database(
    cx: &mut TestAppContext,
) {
    cx.executor().allow_parking();
    let owner = RuntimeOwner::new().unwrap();
    let root = tempfile::tempdir().unwrap();
    let services = Services::new(
        owner.handle(),
        ConfigSource::Fixture {
            registry: root.path().join("registry.toml"),
            profiles: root.path().join("profiles.toml"),
            delay: Duration::ZERO,
        },
    );
    let requests = InsightsRequests::new(&services);
    let (handle, _) = cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
            cx.new(|cx| crate::shell::Shell::new(services, window, cx))
        })
        .unwrap()
    });
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("nav-usage", cx);
        window.render_frame(cx);
        window.click("insights-observer-tab", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, _| {
        owner.handle().active_tasks() == 0
    })
    .await;
    cx.run_until_parked();
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert!(window.try_find("claude-observer").is_some());
        let input = gpui_kit::base::test_support::snapshots(window)
            .into_iter()
            .find(|node| node.label() == Some("Monthly price (USD)"))
            .unwrap()
            .path()
            .last()
            .unwrap()
            .clone();
        window.click(input, cx);
        window.press("ctrl-a", cx);
        window.input("17.5", cx);
        window.click("observer-save", cx);
        window.render_frame(cx);
        assert!(window.try_find("observer-confirmation").is_some());
        window.press("escape", cx);
        window.render_frame(cx);
        assert!(window.try_find("observer-confirmation").is_none());
    })
    .unwrap();
    assert!(!root.path().join("observer.db").exists());
    cx.update_window(handle, |_, window, cx| {
        window.click("observer-save", cx);
        window.render_frame(cx);
        window.click("observer-confirm", cx);
        window.click("insights-usage", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, _| {
        matches!(requests.observer_update(), Update::Finished(_))
    })
    .await;
    assert!(
        matches!(requests.observer_update(),Update::Finished(Ok(value)) if value.monthly_usd==17.5)
    );
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("insights-observer-tab", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, _| {
        owner.handle().active_tasks() == 0
    })
    .await;
    cx.run_until_parked();
    let conn = rusqlite::Connection::open_with_flags(
        root.path().join("observer.db"),
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .unwrap();
    assert_eq!(
        ccr_app::usage::observer::subscription::read(&conn)
            .unwrap()
            .monthly_usd,
        17.5
    );
    assert!(!root.path().join("llmusage").exists());
    assert!(!root.path().join("archive.db").exists());
    cx.update_window(handle, |_, window, _| window.remove_window())
        .unwrap();
    owner.shutdown();
}
