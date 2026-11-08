//! Rendered Overview contracts in Kit's headless harness; native pixels stay out of scope.
use super::chart::{ExactRow, trimmed_start};
use super::kpis::format_average;
use super::*;
use crate::runtime::{
    ConfigSource, Services,
    insights::{InsightsPaths, SessionScope, WorkbenchDay, WorkbenchPlatform},
};
use ccr_app::runtime::RuntimeOwner;
use gpui_kit::{
    AppContext, SharedString, TestAppContext, WindowOptions,
    test::{TestAppContextExt, TestWindowExt},
};
use std::{sync::Arc, time::Duration};

fn days(values: &[i64]) -> Vec<WorkbenchDay> {
    values
        .iter()
        .enumerate()
        .map(|(index, value)| WorkbenchDay {
            label: format!("d{index}"),
            claude: *value,
            codex: 0,
            antigravity: 0,
            opencode: 0,
        })
        .collect()
}

#[test]
fn exact_rows_are_newest_first_and_fold_every_quiet_run() {
    use ExactRow::{Day, Quiet};
    assert_eq!(exact_rows(&[]), Vec::new());
    assert_eq!(
        exact_rows(&days(&[0, 0, 2, 0, 0, 0, 5, 0])),
        vec![
            Quiet { first: 7, last: 7 },
            Day(6),
            Quiet { first: 3, last: 5 },
            Day(2),
            Quiet { first: 0, last: 1 },
        ]
    );
    assert_eq!(exact_rows(&days(&[1, 2])), vec![Day(1), Day(0)]);
    assert_eq!(
        exact_rows(&days(&[0, 0, 0])),
        vec![Quiet { first: 0, last: 2 }]
    );
}

#[test]
fn chart_trims_to_one_day_before_the_first_record() {
    assert_eq!(trimmed_start(&days(&[0, 0, 0, 4, 0])), 2);
    assert_eq!(trimmed_start(&days(&[0, 4])), 0);
    assert_eq!(trimmed_start(&days(&[4, 0])), 0);
    // 没有任何记录时保留完整窗口（全部虚线基线），不假装有起点。
    assert_eq!(trimmed_start(&days(&[0, 0])), 0);
    assert_eq!(format_average(1500.), "1.5K");
    assert_eq!(format_average(143.4), "143");
    assert_eq!(format_average(0.25), "0.2");
}

#[gpui_kit::test]
async fn overview_ranges_hide_stale_totals_and_preserve_exact_metrics(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let root = tempfile::tempdir().unwrap_or_else(|e| panic!("fixture: {e}"));
    let owner = RuntimeOwner::new().unwrap_or_else(|e| panic!("runtime: {e}"));
    let services = Services::new(
        owner.handle(),
        ConfigSource::Fixture {
            profiles: root.path().join("profiles.toml"),
            registry: root.path().join("config.toml"),
            delay: Duration::ZERO,
        },
    )
    .with_insights_paths(InsightsPaths {
        usage_root: root.path().join("missing-usage"),
        archive_db: root.path().join("missing-archive"),
        provider_home: root.path().join("home"),
    });
    let (handle, view) = cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(gpui_kit::WindowBounds::Windowed(gpui_kit::Bounds::new(
                    gpui_kit::point(px(0.), px(0.)),
                    gpui_kit::size(px(1280.), px(800.)),
                ))),
                ..WindowOptions::default()
            },
            cx,
            |_, cx| cx.new(|cx| OverviewView::new(services, cx)),
        )
        .unwrap_or_else(|e| panic!("overview window: {e}"))
    });
    cx.update_window(handle, |_, window, _| window.activate_window())
        .unwrap_or_else(|e| panic!("activate overview: {e}"));
    cx.run_until_parked();
    cx.update_window(handle, |_, window, cx| {
        view.update(cx, |this, cx| {
            this.report = Some(Arc::new(WorkbenchReport {
                as_of: "2026-11-06".into(),
                window_days: 7,
                requests: 1002,
                tokens: 338926579,
                cost_usd: None,
                sessions_indexed: true,
                session_count: Some(3),
                session_scope: SessionScope::Window,
                other_requests: 1000,
                platforms: vec![WorkbenchPlatform {
                    id: "codex".into(),
                    requests: 2,
                    tokens: 10,
                    requests_by_day: vec![2],
                }],
                days: vec![WorkbenchDay {
                    label: "11-06".into(),
                    claude: 0,
                    codex: 2,
                    antigravity: 0,
                    opencode: 0,
                }],
            }));
            cx.notify();
        });
        window.render_frame(cx);
        assert_eq!(
            window.find("overview-total-tokens").value(),
            Some("Tokens: 338,926,579")
        );
        assert_eq!(
            window.find("overview-codex-requests").value(),
            Some("Requests: 2")
        );
        assert_eq!(
            window.find("overview-total-sessions").value(),
            Some("Sessions · 7D: 3")
        );
        assert_eq!(
            window.find("overview-total-cost").value(),
            Some("Cost: Cost unavailable")
        );
        assert!(window.try_find("overview-exact-table").is_none());
        view.update(cx, |this, cx| this.focus_main(window, cx));
        for _ in 0..10 {
            window.render_frame(cx);
            if window.find("overview-exact-toggle").focused() == Some(true) {
                break;
            }
            window.press("tab", cx);
        }
        assert_eq!(window.find("overview-exact-toggle").focused(), Some(true));
        window.press("enter", cx);
        assert!(window.try_find("overview-exact-table").is_some());
        window.press("tab", cx);
        assert_eq!(window.find("overview-exact-11-06").focused(), Some(true));
        assert_eq!(
            window.find("overview-exact-11-06").value(),
            Some("11-06: Claude Code 0; Codex 2; Antigravity 0; OpenCode 0; Chart total 2"),
        );
        window.press("shift-tab", cx);
        assert_eq!(window.find("overview-exact-toggle").focused(), Some(true));
        window.press("enter", cx);
        assert!(window.try_find("overview-exact-table").is_none());
    })
    .unwrap_or_else(|e| panic!("overview exact values: {e}"));
    let seeded = cx.update(|cx| {
        view.read(cx)
            .report
            .clone()
            .unwrap_or_else(|| panic!("seeded report"))
    });
    for day_count in [30_u16, 90] {
        let start =
            chrono::NaiveDate::from_ymd_opt(2026, 8, 1).unwrap_or_else(|| panic!("fixture date"));
        cx.update_window(handle, |_, window, cx| {
            view.update(cx, |this, cx| {
                let mut report = seeded.as_ref().clone();
                report.window_days = day_count;
                report.days = (0..day_count)
                    .map(|index| WorkbenchDay {
                        label: (start + chrono::Duration::days(i64::from(index)))
                            .format("%m-%d")
                            .to_string(),
                        claude: 0,
                        codex: 1,
                        antigravity: 0,
                        opencode: 0,
                    })
                    .collect();
                this.report = Some(Arc::new(report));
                this.scroll.set_offset(Default::default());
                this.exact_scroll.set_offset(Default::default());
                cx.notify();
            });
            window.render_frame(cx);
            view.update(cx, |this, cx| this.focus_main(window, cx));
            for _ in 0..10 {
                window.render_frame(cx);
                if window.find("overview-exact-toggle").focused() == Some(true) {
                    break;
                }
                window.press("tab", cx);
            }
            assert_eq!(window.find("overview-exact-toggle").focused(), Some(true));
            window.press("enter", cx);
        })
        .unwrap_or_else(|e| panic!("long report: {e}"));
        for (index, key) in (0..day_count)
            .map(|index| (index, "tab"))
            .chain((0..day_count - 1).rev().map(|index| (index, "shift-tab")))
        {
            cx.update_window(handle, |_, window, cx| {
                window.press(key, cx);
                // Focus observers run after paint; render their requested
                // scroll before checking the focused row's visible bounds.
                window.render_frame(cx);
                // 精确值表最新在上：第 index 个表行是倒数第 index 个日期。
                let id = format!(
                    "overview-exact-{}",
                    (start + chrono::Duration::days(i64::from(day_count - 1 - index)))
                        .format("%m-%d")
                );
                let row = window.find(SharedString::from(id.clone()));
                assert_eq!(row.focused(), Some(true), "{day_count}D {key}: {id}");
                let row = row.bounds();
                let table = window.find("overview-exact-table").bounds();
                let page = window.find("overview-root").bounds();
                assert!(
                    row.top() >= table.top().max(page.top())
                        && row.bottom() <= table.bottom().min(page.bottom()),
                    "{day_count}D {key}: {id} {row:?} outside table {table:?} / page {page:?}",
                );
            })
            .unwrap_or_else(|e| panic!("exact row reveal: {e}"));
        }
        cx.update_window(handle, |_, window, cx| {
            window.press("shift-tab", cx);
            assert_eq!(window.find("overview-exact-toggle").focused(), Some(true));
            window.press("enter", cx);
            assert!(window.try_find("overview-exact-table").is_none());
        })
        .unwrap_or_else(|e| panic!("close long report: {e}"));
    }
    cx.update_window(handle, |_, window, cx| {
        view.update(cx, |this, cx| {
            let mut report = seeded.as_ref().clone();
            report.window_days = 7;
            report.platforms = vec![
                WorkbenchPlatform {
                    id: "codex".into(),
                    requests: 4,
                    tokens: 10,
                    requests_by_day: vec![0, 0, 1, 0, 0, 3, 0],
                },
                WorkbenchPlatform {
                    id: "grok".into(),
                    requests: 9,
                    tokens: 3,
                    requests_by_day: Vec::new(),
                },
                WorkbenchPlatform {
                    id: "pi".into(),
                    requests: 0,
                    tokens: 0,
                    requests_by_day: vec![0; 7],
                },
            ];
            report.days = [
                "10-31", "11-01", "11-02", "11-03", "11-04", "11-05", "11-06",
            ]
            .into_iter()
            .zip([0, 0, 1, 0, 0, 3, 0])
            .map(|(label, codex)| WorkbenchDay {
                label: label.into(),
                claude: 0,
                codex,
                antigravity: 0,
                opencode: 0,
            })
            .collect();
            this.report = Some(Arc::new(report));
            this.show_exact = false;
            this.scroll.set_offset(Default::default());
            this.exact_scroll.set_offset(Default::default());
            cx.notify();
        });
        window.render_frame(cx);
        // 页头：标题、范围切换与刷新同一行。
        let heading = window.find("overview-heading").bounds();
        for id in ["overview-window-7", "overview-refresh"] {
            let control = window.find(id).bounds();
            assert!(
                control.top() < heading.bottom() && heading.top() < control.bottom(),
                "{id} is not on the heading row: {control:?} vs {heading:?}"
            );
        }
        // KPI 四栏同一行；右栏与主列等高。
        assert_eq!(
            window.find("overview-total-cost").bounds().top(),
            window.find("overview-total-requests").bounds().top()
        );
        assert_eq!(
            window.find("overview-rail").bounds().bottom(),
            window.find("overview-usage").bounds().bottom()
        );
        assert!(window.find("overview-insights").visible());
        // 只有有逐日记录的平台单独成格；其余合并到“其他来源”并写明原因。
        assert_eq!(
            window.find("overview-codex-requests").value(),
            Some("Requests: 4")
        );
        assert!(window.try_find("overview-grok-requests").is_none());
        assert_eq!(
            window.find("overview-other-grok").label(),
            Some("Grok · 9 requests · daily split unavailable")
        );
        assert_eq!(
            window.find("overview-other-pi").label(),
            Some("Pi · no requests")
        );
        // 趋势图收紧到首个有数据日（11-02）的前一天。
        assert_eq!(
            window.find("overview-chart-since").label(),
            Some("since 11-01")
        );
        assert!(window.try_find("overview-day-10-31").is_none());
        assert!(window.try_find("overview-day-11-01").is_some());
        view.update(cx, |this, cx| this.focus_main(window, cx));
        for _ in 0..10 {
            window.render_frame(cx);
            if window.find("overview-exact-toggle").focused() == Some(true) {
                break;
            }
            window.press("tab", cx);
        }
        assert_eq!(window.find("overview-exact-toggle").focused(), Some(true));
        window.press("enter", cx);
        // 最新在上，连续全零日折叠为一行，折叠行同样可聚焦。
        let rows = [
            ("overview-exact-quiet-11-06", "11-06 · no records"),
            (
                "overview-exact-11-05",
                "11-05: Claude Code 0; Codex 3; Antigravity 0; OpenCode 0; Chart total 3",
            ),
            (
                "overview-exact-quiet-11-03",
                "11-03 – 11-04 · no records (2 days)",
            ),
            (
                "overview-exact-11-02",
                "11-02: Claude Code 0; Codex 1; Antigravity 0; OpenCode 0; Chart total 1",
            ),
            (
                "overview-exact-quiet-10-31",
                "10-31 – 11-01 · no records (2 days)",
            ),
        ];
        for id in [
            "overview-exact-11-03",
            "overview-exact-11-04",
            "overview-exact-11-06",
        ] {
            assert!(window.try_find(id).is_none(), "{id} should be folded");
        }
        for (id, value) in rows {
            window.press("tab", cx);
            window.render_frame(cx);
            let row = window.find(id);
            assert_eq!(row.focused(), Some(true), "tab: {id}");
            assert_eq!(row.value(), Some(value));
        }
        for (id, _) in rows.iter().rev().skip(1) {
            window.press("shift-tab", cx);
            window.render_frame(cx);
            assert_eq!(window.find(*id).focused(), Some(true), "shift-tab: {id}");
        }
        window.press("shift-tab", cx);
        assert_eq!(window.find("overview-exact-toggle").focused(), Some(true));
        window.press("enter", cx);
        assert!(window.try_find("overview-exact-table").is_none());
    })
    .unwrap_or_else(|e| panic!("overview folded exact rows: {e}"));
    cx.update_window(handle, |_, window, cx| {
        view.update(cx, |this, cx| {
            this.report = Some(seeded);
            this.scroll.set_offset(Default::default());
            cx.notify();
        });
        window.render_frame(cx);
        for (scope, description) in [
            (SessionScope::Unindexed, "Sessions: Archive not indexed"),
            (
                SessionScope::Unavailable,
                "Sessions: Session count unavailable",
            ),
        ] {
            view.update(cx, |this, cx| {
                let Some(report) = this.report.as_mut() else {
                    panic!("expected the seeded overview report");
                };
                let report = Arc::make_mut(report);
                report.session_scope = scope;
                report.session_count = None;
                report.sessions_indexed = false;
                cx.notify();
            });
            window.render_frame(cx);
            assert_eq!(
                window.find("overview-total-sessions").value(),
                Some(description)
            );
        }
        for (id, link) in [
            ("overview-open-usage", OverviewLink::Usage),
            ("overview-open-sessions", OverviewLink::Sessions),
            ("overview-open-commands", OverviewLink::Commands),
            ("overview-open-install", OverviewLink::Install),
        ] {
            window.click(id, cx);
            assert!(view.update(cx, |v, _| v.take_pending()) == Some(link));
        }
        window.click("overview-window-30", cx);
        assert_eq!(window.find("overview-window-30").checked(), Some(true));
        assert!(window.try_find("overview-total-tokens").is_none());
        let old = RequestKey {
            environment: EnvironmentId::Local("overview".into()),
            platform: Platform::Claude,
            generation: view.read(cx).generation,
            request_id: view.read(cx).generation,
        };
        window.click("overview-window-90", cx);
        assert!(!view.read(cx).gate.accepts(&old));
        assert_eq!(window.find("overview-window-90").checked(), Some(true));
    })
    .unwrap_or_else(|e| panic!("overview interaction: {e}"));
    cx.wait_for(handle, Duration::from_secs(5), |_, cx| {
        view.read(cx).error.is_some()
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert!(window.try_find("overview-total-tokens").is_none());
        assert!(
            window
                .find("overview-status")
                .label()
                .is_some_and(|s| !s.is_empty())
        );
        window.remove_window();
    })
    .unwrap_or_else(|e| panic!("overview failed read: {e}"));
    owner.shutdown();
}
