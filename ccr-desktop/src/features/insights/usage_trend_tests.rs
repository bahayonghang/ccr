#![allow(clippy::unwrap_used)]

use super::*;
use crate::runtime::{ConfigSource, insights::InsightsPaths};
use ccr_app::runtime::RuntimeOwner;
use ccr_usage::fixtures::{SeedBucket, create_projection_db, seed_bucket};
use chrono::{Days, Local, NaiveDate};
use gpui_kit::{
    TestAppContext, WindowOptions,
    test::{TestAppContextExt, TestWindowExt},
};
use std::{path::Path, time::Duration};

fn services(owner: &RuntimeOwner, root: &Path) -> Services {
    Services::new(
        owner.handle(),
        ConfigSource::Fixture {
            profiles: root.join("profiles.toml"),
            registry: root.join("config.toml"),
            delay: Duration::ZERO,
        },
    )
    .with_insights_paths(InsightsPaths {
        usage_root: root.join("llmusage"),
        archive_db: root.join("archive.db"),
        provider_home: root.join("home"),
    })
}

fn seed(root: &Path, buckets: &[SeedBucket]) {
    let paths = create_projection_db(&root.join("llmusage"));
    let connection = rusqlite::Connection::open(&paths.db_path).unwrap();
    for bucket in buckets {
        seed_bucket(&connection, bucket);
    }
}

fn day_before(today: NaiveDate, days: u64) -> NaiveDate {
    today.checked_sub_days(Days::new(days)).unwrap()
}

#[gpui_kit::test]
async fn usage_trend_switches_metric_and_keeps_zero_and_gap_distinct(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let owner = RuntimeOwner::new().unwrap();
    let executor = owner.handle();
    let root = tempfile::tempdir().unwrap();
    let today = Local::now().date_naive();
    let first = day_before(today, 4);
    let missing = day_before(today, 3);
    let zero = day_before(today, 2);
    let unknown = day_before(today, 1);
    seed(
        root.path(),
        &[
            SeedBucket {
                hour_start: bucket_time(first),
                cost_with_cache_usd: 0.123456789,
                ..Default::default()
            },
            SeedBucket {
                hour_start: bucket_time(zero),
                cost_with_cache_usd: 0.,
                ..Default::default()
            },
            SeedBucket {
                hour_start: bucket_time(unknown),
                cost_with_cache_usd: 0.,
                pricing_status: "missing".into(),
                ..Default::default()
            },
        ],
    );
    let (handle, _) = cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
            cx.new(|cx| {
                crate::shell::Shell::new_with_preferences(
                    services(&owner, root.path()),
                    Ok(Default::default()),
                    window,
                    cx,
                )
            })
        })
        .unwrap()
    });
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("nav-usage", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, _| {
        executor.active_tasks() == 0
    })
    .await;
    cx.run_until_parked();
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert_eq!(window.find("usage-trend-metric-cost").checked(), Some(true));
        for (date, expected) in [
            (first, "$0.123456789 USD"),
            (missing, "No records"),
            (zero, "$0 USD"),
            (unknown, "Estimate unavailable"),
        ] {
            let id = SharedString::from(format!("usage-trend-slot-{date}"));
            let label = format!("{date} · {expected}");
            assert_eq!(window.find(id).value(), Some(label.as_str()), "{date}");
        }
        let slot = |date| {
            window
                .find(SharedString::from(format!("usage-trend-slot-{date}")))
                .bounds()
                .center()
                .x
        };
        let first_interval = slot(missing) - slot(first);
        let second_interval = slot(zero) - slot(missing);
        assert!(f32::from(first_interval - second_interval).abs() <= 1.);
        let first_point = window.find(SharedString::from(format!("usage-trend-point-{first}")));
        let zero_point = window.find(SharedString::from(format!("usage-trend-point-{zero}")));
        assert!(first_point.bounds().top() < zero_point.bounds().top());
        assert!(
            window
                .try_find(SharedString::from(format!("usage-trend-point-{missing}")))
                .is_none()
        );
        assert_eq!(
            window.find("usage-cache-hit-rate").value(),
            Some("Cache hit rate · 20.00%")
        );
        window.click("usage-trend-metric-tokens", cx);
        assert_eq!(
            window.find("usage-trend-metric-tokens").checked(),
            Some(true)
        );
        assert_eq!(
            window
                .find(SharedString::from(format!("usage-trend-slot-{unknown}")))
                .value(),
            Some(format!("{unknown} · 100 tokens").as_str())
        );
        window.click("usage-trend-metric-requests", cx);
        assert_eq!(
            window
                .find(SharedString::from(format!("usage-trend-slot-{unknown}")))
                .value(),
            Some(format!("{unknown} · 2 requests").as_str())
        );
        window.click("usage-trend-exact-toggle", cx);
        window.simulate_next_frame(cx);
        window.render_frame(cx);
        assert!(window.find("usage-trend-exact-table").visible());
        scroll_usage_to_top(window, cx);
        window.click("usage-trend-exact-toggle", cx);
        assert!(window.try_find("usage-trend-exact-table").is_none());
        assert_eq!(window.find("usage-trend-chart-focus").focused(), Some(true));
        assert_eq!(executor.active_tasks(), 0);
        window.remove_window();
    })
    .unwrap();
    owner.shutdown();
}

#[gpui_kit::test]
async fn platform_usage_reaches_exact_dates_and_composition_from_the_same_report(
    cx: &mut TestAppContext,
) {
    cx.executor().allow_parking();
    let owner = RuntimeOwner::new().unwrap();
    let executor = owner.handle();
    let root = tempfile::tempdir().unwrap();
    let today = Local::now().date_naive();
    let newest = day_before(today, 2);
    let oldest = NaiveDate::from_ymd_opt(2000, 1, 1).unwrap();
    seed(
        root.path(),
        &[
            SeedBucket {
                hour_start: bucket_time(newest),
                input_tokens: 1,
                output_tokens: 30,
                reasoning_output_tokens: 11,
                cache_read_tokens: 9_999_999,
                cache_creation_tokens: 5,
                total_tokens: 10_000_046,
                ..Default::default()
            },
            SeedBucket {
                hour_start: bucket_time(oldest),
                ..Default::default()
            },
            SeedBucket {
                source: SourceKind::Claude.as_str().into(),
                hour_start: bucket_time(newest),
                input_tokens: 0,
                cache_read_tokens: 0,
                cache_creation_tokens: 9,
                output_tokens: 3,
                reasoning_output_tokens: 2,
                total_tokens: 14,
                ..Default::default()
            },
        ],
    );
    let options = WindowOptions {
        window_bounds: Some(gpui_kit::WindowBounds::Windowed(gpui_kit::Bounds::new(
            gpui_kit::point(px(0.), px(0.)),
            gpui_kit::size(px(960.), px(640.)),
        ))),
        ..Default::default()
    };
    let (handle, _) = cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(options, cx, |window, cx| {
            cx.new(|cx| {
                crate::shell::Shell::new_with_preferences(
                    services(&owner, root.path()),
                    Ok(Default::default()),
                    window,
                    cx,
                )
            })
        })
        .unwrap()
    });
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("nav-codex", cx);
        open_platform_usage(window, cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, _| {
        executor.active_tasks() == 0
    })
    .await;
    cx.run_until_parked();
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        for (id, expected) in [
            ("usage-token-input", "Input · 1"),
            ("usage-token-output", "Output · 41"),
            ("usage-token-cache-read", "Cache reads · 9,999,999"),
            ("usage-token-cache-write", "Cache writes · 5"),
            ("usage-cache-hit-rate", "Cache hit rate · >99.99%"),
        ] {
            assert_eq!(window.find(id).value(), Some(expected), "{id}");
        }
        let input = window.find("usage-token-input").bounds();
        let output = window.find("usage-token-output").bounds();
        let reads = window.find("usage-token-cache-read").bounds();
        let writes = window.find("usage-token-cache-write").bounds();
        assert_eq!(input.top(), output.top());
        assert!(reads.top() > input.bottom());
        assert_eq!(reads.top(), writes.top());
        assert!(writes.right() <= window.find("usage-token-composition").bounds().right());
        let point_id = SharedString::from(format!("usage-trend-point-{newest}"));
        assert!(
            window.try_find(point_id).is_some(),
            "one isolated day has a plotted marker"
        );
        for _ in 0..45 {
            if window.find("usage-trend-exact-toggle").focused() == Some(true) {
                break;
            }
            window.press("tab", cx);
        }
        let toggle = window.find("usage-trend-exact-toggle");
        assert_eq!(toggle.focused(), Some(true));
        assert!(toggle.visible());
        window.press("enter", cx);
    })
    .unwrap();
    cx.run_until_parked();
    cx.update_window(handle, |_, window, cx| {
        window.simulate_next_frame(cx);
        window.render_frame(cx);
        let first = day_before(today, 29);
        let first_id = SharedString::from(format!("usage-trend-day-{first}"));
        assert_eq!(window.find(first_id).focused(), Some(true));
        assert_eq!(
            window.find("usage-trend-exact-table").label(),
            Some("Daily exact values")
        );
        window.press("end", cx);
        let last_id = SharedString::from(format!("usage-trend-day-{today}"));
        let last = window.find(last_id);
        let scroll = window.find("usage-trend-exact-scroll");
        let page = window.find("insights-root");
        assert_eq!(last.focused(), Some(true));
        assert!(last.bounds().bottom() <= scroll.bounds().bottom());
        assert!(last.bounds().bottom() <= page.bounds().bottom());
        window.press("up", cx);
        window.press("up", cx);
        let newest_id = SharedString::from(format!("usage-trend-day-{newest}"));
        let latest_record = window.find(newest_id);
        assert_eq!(latest_record.focused(), Some(true));
        assert_eq!(
            latest_record.value(),
            Some(format!("{newest} · $0.1 USD").as_str())
        );
        window.press("escape", cx);
        assert!(window.try_find("usage-trend-exact-table").is_none());
        assert_eq!(
            window.find("usage-trend-exact-toggle").focused(),
            Some(true)
        );
        assert!(window.find("usage-trend-exact-toggle").visible());
        assert_eq!(executor.active_tasks(), 0);

        scroll_usage_to_top(window, cx);
        window.click("usage-range-menu", cx);
        window.click("usage-range-all", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, _| {
        executor.active_tasks() == 0
    })
    .await;
    cx.run_until_parked();
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        let range = format!("{} – {newest}", day_before(newest, 29));
        assert_eq!(
            window.find("usage-trend-range").value(),
            Some(range.as_str())
        );
        window.click("usage-trend-previous", cx);
        let older = format!("{} – {}", day_before(newest, 59), day_before(newest, 30));
        assert_eq!(
            window.find("usage-trend-range").value(),
            Some(older.as_str())
        );
        window.click("usage-trend-earliest", cx);
        let expected_end = oldest
            .checked_add_days(Days::new(((newest - oldest).num_days() % 30) as u64))
            .unwrap();
        let first_page = format!("{oldest} – {expected_end}");
        assert_eq!(
            window.find("usage-trend-range").value(),
            Some(first_page.as_str())
        );
        assert_eq!(
            window
                .find(SharedString::from(format!("usage-trend-slot-{oldest}")))
                .value(),
            Some(format!("{oldest} · $0.1 USD").as_str())
        );
        assert!(
            window
                .try_find(SharedString::from(format!("usage-trend-slot-{newest}")))
                .is_none()
        );
        assert_eq!(
            window.find("usage-token-output").value(),
            Some("Output · 86")
        );
        assert_eq!(executor.active_tasks(), 0);

        window.click("nav-claude-code", cx);
        open_platform_usage(window, cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, _| {
        executor.active_tasks() == 0
    })
    .await;
    cx.run_until_parked();
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert_eq!(
            window.find("usage-token-output").value(),
            Some("Output · 5")
        );
        assert_eq!(
            window.find("usage-cache-hit-rate").value(),
            Some("Cache hit rate · Not calculable")
        );
        window.remove_window();
    })
    .unwrap();
    owner.shutdown();
}

fn bucket_time(date: NaiveDate) -> String {
    date.and_hms_opt(12, 0, 0)
        .unwrap()
        .and_local_timezone(Local)
        .single()
        .unwrap()
        .with_timezone(&chrono::Utc)
        .to_rfc3339()
}

fn scroll_usage_to_top(window: &mut Window, cx: &mut gpui_kit::App) {
    window.scroll(
        "insights-root",
        gpui_kit::ScrollDelta::Pixels(gpui_kit::point(px(0.), px(2000.))),
        cx,
    );
}

fn open_platform_usage(window: &mut Window, cx: &mut gpui_kit::App) {
    if !window.find("tool-platform-usage").visible() {
        window.scroll(
            "platform-tab-scroll",
            gpui_kit::ScrollDelta::Pixels(gpui_kit::point(px(-2000.), px(0.))),
            cx,
        );
    }
    window.click("tool-platform-usage", cx);
}
