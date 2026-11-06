#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::*;
use ccr_app::runtime::RuntimeOwner;
use ccr_types::MonitoringLevel;
use gpui_kit::{
    TestAppContext, WindowOptions,
    test::{TestAppContextExt, TestWindowExt},
};
use std::time::Duration;

#[gpui_kit::test]
async fn monitoring_filters_real_main_events_and_suspension_detaches_without_polling(
    cx: &mut TestAppContext,
) {
    cx.executor().allow_parking();
    let root = tempfile::tempdir().unwrap();
    let source = crate::runtime::monitoring::tests::source(root.path());
    crate::runtime::monitoring::tests::seed(
        root.path(),
        "observer.db",
        &MonitoringEntry::new(
            MonitoringLevel::Error,
            "commands",
            "command.failed",
            "fixture",
            "api_key=fixture-secret-sentinel",
        ),
    );
    crate::runtime::monitoring::tests::seed(
        root.path(),
        "observer.db",
        &MonitoringEntry::new(
            MonitoringLevel::Info,
            "tools",
            "tools.operation.completed",
            "fixture",
            "tool checked",
        ),
    );
    let owner = RuntimeOwner::new().unwrap();
    let paths = ccr_usage::fixtures::create_projection_db(&root.path().join("llmusage"));
    let services = Services::new(owner.handle(), source).with_insights_paths(
        crate::runtime::insights::InsightsPaths {
            usage_root: paths.root_dir,
            archive_db: root.path().join("archive.db"),
            provider_home: root.path().join("home"),
        },
    );
    let latency = services.usage_report_latency.clone();
    let monitor = services.monitoring.clone();
    let (handle, view) = cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
            cx.new(|cx| {
                let mut view = MonitoringView::new(services, window, cx);
                view.refresh(cx);
                view
            })
        })
        .unwrap()
    });
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        view.read(cx).report.is_some()
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        let labels = gpui_kit::base::test_support::snapshots(window)
            .into_iter()
            .filter_map(|node| node.label().map(str::to_string))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(!labels.contains("fixture-secret-sentinel"));
        assert!(window.try_find("global-monitoring").is_some());
        assert!(labels.contains("0 / 2048"));
        assert!(labels.contains("Unavailable") || labels.contains("不可用"));
        assert_eq!(latency.lock().unwrap().snapshot().sample_count, 0);
        assert_eq!(
            view.read(cx)
                .report
                .as_ref()
                .unwrap()
                .metrics
                .retained_ssh_sessions,
            0
        );
        window.click("monitoring-level-error", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| !view.read(cx).busy)
        .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        let report = view.read(cx).report.as_ref().unwrap().clone();
        assert_eq!(report.entries.len(), 1);
        assert_eq!(report.entries[0].channel, "commands");
        window.click(
            SharedString::from(format!("monitoring-entry:{}", report.entries[0].id)),
            cx,
        );
        window.render_frame(cx);
        assert!(window.try_find("monitoring-detail").is_some());
        latency.lock().unwrap().record(Duration::ZERO);
        view.update(cx, |this, cx| this.refresh(cx));
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| !view.read(cx).busy)
        .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        let labels = gpui_kit::base::test_support::snapshots(window)
            .into_iter()
            .filter_map(|node| node.label().map(str::to_string))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(labels.contains("0.000 ms"));
        assert!(labels.contains("1 / 2048"));
        view.update(cx, |this, cx| {
            this.refresh(cx);
            this.suspend(cx);
        });
        assert!(view.read(cx).task.is_none());
        assert!(view.read(cx).report.is_none());
    })
    .unwrap();
    let reads = monitor.read_count();
    cx.background_executor
        .timer(Duration::from_millis(250))
        .await;
    cx.update_window(handle, |_, _, cx| {
        assert!(view.read(cx).report.is_none());
        assert!(monitor.read_count() <= reads + 1);
    })
    .unwrap();
    let reads = monitor.read_count();
    cx.background_executor
        .timer(Duration::from_millis(250))
        .await;
    assert_eq!(monitor.read_count(), reads);
    assert_eq!(latency.lock().unwrap().snapshot().sample_count, 1);
    owner.shutdown();
}

#[gpui_kit::test]
async fn overview_monitoring_entry_preserves_the_thirteen_destination_shell(
    cx: &mut TestAppContext,
) {
    cx.executor().allow_parking();
    let root = tempfile::tempdir().unwrap();
    let owner = RuntimeOwner::new().unwrap();
    let services = Services::new(
        owner.handle(),
        crate::runtime::monitoring::tests::source(root.path()),
    );
    let (handle, _shell) = cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
            cx.new(|cx| crate::shell::Shell::new(services, window, cx))
        })
        .unwrap()
    });
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("tool-monitoring", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |window, _| {
        window.try_find("monitoring-storage-error").is_some()
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert!(window.try_find("monitoring-storage-error").is_some());
        assert!(window.try_find("nav-settings").is_some());
        window.click("nav-claude-code", cx);
        window.render_frame(cx);
        assert!(window.try_find("global-monitoring").is_none());
    })
    .unwrap();
    owner.shutdown();
}
