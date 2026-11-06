#![allow(clippy::unwrap_used)]
use super::*;
use crate::{features::monitoring::MonitoringView, runtime::ConfigSource};
use ccr_app::runtime::RuntimeOwner;
use gpui_kit::test::{TestAppContextExt, TestWindowExt};
use gpui_kit::{AppContext, TestAppContext, WindowOptions};
use std::time::Duration;

fn synthetic() -> Report {
    Report {
        sampled_at: chrono::DateTime::parse_from_rfc3339("2026-11-06T12:00:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc),
        cpu_sample_ms: 250,
        hostname: Some("Synthetic host".into()),
        os_name: Some("Synthetic OS".into()),
        os_version: Some("fixture".into()),
        kernel_version: None,
        arch: "synthetic-arch".into(),
        cpu_brand: Some("Synthetic CPU".into()),
        logical_cpus: Some(4),
        cpu_usage_percent: Some(42.),
        memory: Memory {
            total_bytes: Some(8_589_934_592),
            used_bytes: Some(2_147_483_648),
        },
        swap: Memory {
            total_bytes: None,
            used_bytes: None,
        },
        uptime_seconds: Some(90061),
    }
}

#[gpui_kit::test]
async fn explicit_system_controls_show_units_and_refuse_fixture_then_clear_stale_sample(
    cx: &mut TestAppContext,
) {
    cx.executor().allow_parking();
    let root = tempfile::tempdir().unwrap();
    let owner = RuntimeOwner::new().unwrap();
    let services = Services::new(
        owner.handle(),
        ConfigSource::Fixture {
            profiles: root.path().join("profiles.toml"),
            registry: root.path().join("registry.toml"),
            delay: Duration::ZERO,
        },
    );
    let (handle, parent) = cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(gpui_kit::WindowBounds::Windowed(
                    gpui_kit::Bounds::centered(None, gpui_kit::size(px(1280.), px(800.)), cx),
                )),
                ..Default::default()
            },
            cx,
            |window, cx| cx.new(|cx| MonitoringView::new(services, window, cx)),
        )
        .unwrap()
    });
    let view = cx.update(|cx| parent.read(cx).system.clone());
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert_eq!(view.read(cx).generation, 0);
        assert!(view.read(cx).task.is_none());
        assert!(window.try_find("monitoring-host-facts").is_none());
        assert!(
            window
                .find("monitoring-host-status")
                .label()
                .unwrap()
                .contains("not been read")
        );
        window.click("monitoring-host-read", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        view.read(cx).error.is_some()
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert_eq!(view.read(cx).error, Some(Error::Fixture));
        assert!(
            window
                .find("monitoring-host-status")
                .label()
                .unwrap()
                .contains("benchmark")
        );
        assert!(window.try_find("monitoring-host-facts").is_none());
        view.update(cx, |view, _| {
            view.synthetic = Some((synthetic(), Duration::from_millis(100)))
        });
        window.click("monitoring-host-read", cx);
        assert!(view.read(cx).busy);
        assert!(view.read(cx).report.is_none());
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        view.read(cx).report.is_some()
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert!(
            window
                .find("monitoring-host-sampled")
                .label()
                .unwrap()
                .contains("2026-11-06 12:00:00 UTC")
        );
        assert!(
            window
                .find("monitoring-host-sampled")
                .label()
                .unwrap()
                .contains("250 ms")
        );
        assert!(
            window
                .find("monitoring-host-os")
                .label()
                .unwrap()
                .contains("Synthetic OS")
        );
        assert!(
            window
                .find("monitoring-host-os")
                .label()
                .unwrap()
                .contains("Unavailable")
        );
        assert!(
            window
                .find("monitoring-host-cpu")
                .label()
                .unwrap()
                .contains("42.0%")
        );
        assert!(
            window
                .find("monitoring-host-cpu")
                .label()
                .unwrap()
                .contains("Logical CPUs: 4")
        );
        assert!(
            window
                .find("monitoring-host-memory")
                .label()
                .unwrap()
                .contains("2.00 GiB / 8.00 GiB (25.0%)")
        );
        let swap = window
            .find("monitoring-host-swap")
            .label()
            .unwrap()
            .to_string();
        assert!(swap.contains("Unavailable"));
        assert!(!swap.contains("0.00"));
        assert!(
            window
                .find("monitoring-host-uptime")
                .label()
                .unwrap()
                .contains("90061 s")
        );
        view.update(cx, |view, _| {
            view.synthetic = Some((synthetic(), Duration::from_millis(100)))
        });
        window.click("monitoring-host-read", cx);
        let previous = view.read(cx).generation;
        assert!(view.read(cx).busy);
        parent.update(cx, |view, cx| view.suspend(cx));
        view.update(cx, |view, cx| view.finish(previous, Ok(synthetic()), cx));
        assert!(view.read(cx).report.is_none());
        assert!(view.read(cx).task.is_none());
        assert!(!view.read(cx).busy);
    })
    .unwrap();
    let generation = cx.update(|cx| view.read(cx).generation);
    cx.background_executor
        .timer(Duration::from_millis(250))
        .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert_eq!(view.read(cx).generation, generation);
        assert!(view.read(cx).report.is_none());
        assert!(window.try_find("monitoring-host-facts").is_none());
        window.remove_window();
    })
    .unwrap();
    owner.shutdown();
}

#[test]
fn memory_units_and_unknowns_are_explicit() {
    let missing = "Unavailable";
    assert_eq!(
        memory(&synthetic().memory, missing),
        "2.00 GiB / 8.00 GiB (25.0%)"
    );
    assert_eq!(
        memory(&synthetic().swap, missing),
        "Unavailable / Unavailable"
    );
}
