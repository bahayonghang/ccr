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
fn observer_renders_tables_by_default_and_keeps_raw_projection_advanced(cx: &mut TestAppContext) {
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
    let mut report = Report {
        insight: Err(Error::Missing),
        usage: Ok(
            serde_json::json!({"daily":[{"date":"2026-11-06","cost_usd":1.25,"input_tokens":10,"output_tokens":4,"cache_read_tokens":3}],"models":[],"projects":[]}),
        ),
        tools: Ok(
            serde_json::json!({"tools":[{"tool_name":"Read","call_count":7,"cost_usd":null},{"tool_name":"Bash","call_count":1,"cost_usd":0.0}],"sessions":[],"heatmap":[],"timezone":"utc"}),
        ),
        subscription: Ok(None),
    };
    let (handle, view) = cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
            cx.new(|cx| ObserverView::new(requests, window, cx))
        })
        .unwrap()
    });
    for (count, complete, expected) in [
        (0, true, "0"),
        (0, false, "Projects unattributed"),
        (2, false, "2 · Attributed projects (partial coverage)"),
        (2, true, "2"),
    ] {
        report.insight =
            Ok(serde_json::json!({"total_projects":count,"project_attribution_complete":complete}));
        view.update(cx, |view, cx| view.receive(&report, cx));
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            assert_eq!(
                window.find("observer-total-total_projects").label(),
                Some(expected)
            );
        })
        .unwrap();
    }
    report.insight = Err(Error::Missing);
    view.update(cx, |view, cx| view.receive(&report, cx));
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert!(window.find("observer-structured-report").visible());
        assert!(window.find("observer-daily").visible());
        assert!(window.try_find("observer-tools").is_some());
        let cells = gpui_kit::base::test_support::snapshots(window);
        assert!(
            cells
                .iter()
                .any(|node| node.label() == Some("Not recorded"))
        );
        assert!(cells.iter().any(|node| node.label() == Some("0.0")));
        let raw_present = |window: &Window| {
            gpui_kit::base::test_support::snapshots(window)
                .iter()
                .any(|node| node.label() == Some("Stored Claude usage and tool report"))
        };
        assert!(!raw_present(window));
        window.click("observer-advanced", cx);
        assert!(raw_present(window));
        window.click("observer-advanced", cx);
        assert!(!raw_present(window));
    })
    .unwrap();
    // 已观测会话表：未记录费用与真实零分开显示；工具表留空，标签只能来自会话表
    report.tools = Ok(serde_json::json!({
        "tools": [],
        "sessions": [
            {"session_id":"unknown","project_path":null,"cost_usd":null,"tool_call_count":7},
            {"session_id":"free","project_path":null,"cost_usd":0.0,"tool_call_count":1}
        ],
        "heatmap": [],
        "timezone": "utc"
    }));
    for (language, unknown) in [
        (crate::locales::Language::English, "Not recorded"),
        (crate::locales::Language::Chinese, "未记录"),
    ] {
        cx.update(|cx| cx.set_global(language));
        view.update(cx, |view, cx| view.receive(&report, cx));
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            assert!(window.try_find("observer-tools").is_none());
            assert!(window.find("observer-sessions").visible());
            let labels = gpui_kit::base::test_support::snapshots(window)
                .iter()
                .filter_map(|node| node.label().map(str::to_owned))
                .collect::<Vec<_>>();
            assert_eq!(labels.iter().filter(|label| *label == unknown).count(), 1);
            assert_eq!(labels.iter().filter(|label| *label == "0.0").count(), 1);
        })
        .unwrap();
    }
    cx.update_window(handle, |_, window, _| window.remove_window())
        .unwrap();
    owner.shutdown();
}

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
