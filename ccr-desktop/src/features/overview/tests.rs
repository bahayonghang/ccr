//! Rendered Overview contracts in Kit's headless harness; native pixels stay out of scope.
use super::activity::{
    HeatGrid, WeekChange, activity_scope, heat_bucket, heat_thresholds, week_change,
};
use super::chart::{ExactRow, latest_record, trimmed_start};
use super::insights::{Leaders, leader_rows, path_leaf, peak};
use super::kpis::format_average;
use super::*;
use crate::runtime::{
    ConfigSource, Services,
    insights::{InsightsPaths, SessionScope, SyncFreshness, WorkbenchDay, WorkbenchPlatform},
};
use ccr_app::{
    runtime::RuntimeOwner,
    usage::home::{HomeInsightsSnapshot, InsightsAgentTally, InsightsTotals, InsightsWindow},
};
use ccr_usage::{InsightsDay, InsightsTally};
use chrono::NaiveDate;
use gpui_kit::{
    AnyWindowHandle, AppContext, Entity, SharedString, TestAppContext, WindowOptions,
    test::{TestAppContextExt, TestWindowExt},
};
use std::{sync::Arc, time::Duration};

/// Runtime and fixture roots for tests that render a seeded report.
struct Stage {
    owner: RuntimeOwner,
    root: tempfile::TempDir,
}

impl Stage {
    fn boot(cx: &mut TestAppContext) -> Self {
        cx.executor().allow_parking();
        cx.update(gpui_kit::init);
        Self {
            owner: RuntimeOwner::new().unwrap_or_else(|e| panic!("runtime: {e}")),
            root: tempfile::tempdir().unwrap_or_else(|e| panic!("fixture: {e}")),
        }
    }

    fn open(
        &self,
        cx: &mut TestAppContext,
        width: f32,
        height: f32,
    ) -> (AnyWindowHandle, Entity<OverviewView>) {
        let root = self.root.path();
        let services = Services::new(
            self.owner.handle(),
            ConfigSource::Fixture {
                profiles: root.join("profiles.toml"),
                registry: root.join("config.toml"),
                delay: Duration::ZERO,
            },
        )
        .with_insights_paths(InsightsPaths {
            usage_root: root.join("missing-usage"),
            archive_db: root.join("missing-archive"),
            provider_home: root.join("home"),
        });
        cx.update(|cx| {
            gpui_kit::open_window(
                WindowOptions {
                    window_bounds: Some(gpui_kit::WindowBounds::Windowed(gpui_kit::Bounds::new(
                        gpui_kit::point(px(0.), px(0.)),
                        gpui_kit::size(px(width), px(height)),
                    ))),
                    ..WindowOptions::default()
                },
                cx,
                |_, cx| cx.new(|cx| OverviewView::new(services, cx)),
            )
            .unwrap_or_else(|e| panic!("overview window: {e}"))
        })
    }
}

fn show(
    view: &Entity<OverviewView>,
    report: WorkbenchReport,
    window: &mut Window,
    cx: &mut gpui_kit::App,
) {
    view.update(cx, |this, cx| {
        this.show_report(Arc::new(report));
        cx.notify();
    });
    window.render_frame(cx);
}

/// Report date 2026-11-06 (a Friday), so the grid starts on Monday 2025-11-03.
fn activity_fixture() -> HomeInsightsSnapshot {
    let day = |date: &str, requests| InsightsDay {
        date: date.into(),
        requests,
    };
    let tally = |label: &str, requests| InsightsTally {
        key: label.into(),
        label: label.into(),
        requests,
        tokens: 1,
    };
    let mut hourly = vec![0; 24];
    hourly[9] = 10;
    hourly[14] = 40;
    HomeInsightsSnapshot {
        as_of: "2026-11-06".into(),
        first_date: Some("2024-01-01".into()),
        totals: InsightsTotals {
            requests: 1041,
            tokens: 900,
            sessions: 5,
            agents: 2,
            projects: 2,
            active_days: 6,
        },
        sessions_indexed: true,
        last_7_days: InsightsWindow {
            requests: 120,
            sessions: 4,
            active_days: 3,
        },
        previous_7_days: InsightsWindow {
            requests: 100,
            sessions: 4,
            active_days: 0,
        },
        daily: vec![
            day("2025-11-03", 1),
            day("2026-11-03", 5),
            day("2026-11-04", 20),
            day("2026-11-05", 7),
            day("2026-11-06", 9),
        ],
        current_streak: 4,
        // 全历史读数比 53 周窗口更大：摘要不能读这两个字段。
        longest_streak: 30,
        busiest_day: Some(day("2024-01-01", 999)),
        hourly,
        weekday: vec![1, 30, 2, 3, 4, 0, 0],
        monthly: vec![0; 12],
        trend_start: "2025-11-03".into(),
        trend: Vec::new(),
        agents: vec![
            InsightsAgentTally {
                source: "kimi_code".into(),
                requests: 9,
                tokens: 1,
                sessions: 1,
                unmapped: false,
            },
            InsightsAgentTally {
                source: "archive-only".into(),
                requests: 0,
                tokens: 0,
                sessions: 3,
                unmapped: true,
            },
        ],
        projects: vec![
            tally(r"D:\Documents\thesis", 12),
            tally("/home/me/site/", 3),
        ],
        models: (0..10)
            .map(|index| tally(&format!("model-{index}"), 100 - index))
            .collect(),
        generated_at: "2026-11-06T00:00:00+00:00".into(),
    }
}

fn report_fixture(activity: Option<HomeInsightsSnapshot>) -> WorkbenchReport {
    WorkbenchReport {
        as_of: "2026-11-06".into(),
        window_days: 7,
        requests: 120,
        tokens: 900,
        cost_usd: Some(1.5),
        sessions_indexed: true,
        session_count: Some(4),
        session_scope: SessionScope::Window,
        other_requests: 0,
        platforms: vec![WorkbenchPlatform {
            id: "codex".into(),
            requests: 9,
            tokens: 1,
            requests_by_day: vec![0, 0, 0, 5, 2, 2, 0],
        }],
        days: [
            "10-31", "11-01", "11-02", "11-03", "11-04", "11-05", "11-06",
        ]
        .into_iter()
        .zip([0, 0, 0, 5, 2, 2, 0])
        .map(|(label, codex)| WorkbenchDay {
            label: label.into(),
            claude: 0,
            codex,
            antigravity: 0,
            opencode: 0,
        })
        .collect(),
        last_sync_at: Some("2026-11-06T02:04:00Z".into()),
        freshness: SyncFreshness {
            local: Some("11-06 02:04".into()),
            stale: false,
        },
        activity,
    }
}

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
fn latest_record_names_the_last_active_day_only_before_trailing_gaps() {
    assert_eq!(latest_record(&days(&[0, 2, 5, 0, 0])), Some("d2"));
    assert_eq!(latest_record(&days(&[0, 2, 5])), None, "no trailing gap");
    assert_eq!(latest_record(&days(&[0, 0, 0])), None, "no records at all");
    assert_eq!(latest_record(&[]), None);
    // 热力图起点在一年前：口径保留年份。
    assert_eq!(
        activity_scope("2025-11-03", false),
        "Last 53 weeks · since 2025-11-03"
    );
    assert_eq!(
        activity_scope("2025-11-03", true),
        "最近 53 周 · 自 2025-11-03 起"
    );
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

#[test]
fn geometry_follows_the_viewport_and_the_shell_chrome() {
    let at = |width, height| Geometry::new(gpui_kit::size(px(width), px(height)));
    // 1280×800：内容宽 975 = 1280 − 216 侧栏 − 40 场景边距 − 1 亮边 − 48 内边距。
    let default = at(1280., 800.);
    assert_eq!((default.rail_w, default.main_w), (280., 679.));
    assert_eq!(default.heat_cell(), 9.);
    assert!(!default.summary_beside() && !default.heat_scrolls());
    // 2000×1080：比例算出 408，收到上限 360；格子收到上限 14，摘要放到右侧。
    let wide = at(2000., 1080.);
    assert_eq!((wide.rail_w, wide.main_w), (360., 1319.));
    assert_eq!(wide.heat_cell(), 14.);
    assert!(wide.summary_beside() && !wide.heat_scrolls());
    // 比例落在上下限之间时取 8 的倍数：内容宽 1335 × 0.24 = 320.4。
    assert_eq!(at(1640., 1000.).rail_w, 320.);
    // 1040 宽：场景边距随高度余量收敛，主列放不下最小格，网格在面板内横滚。
    let tight = at(1040., 800.);
    assert_eq!((tight.rail_w, tight.main_w), (280., 459.));
    assert_eq!(tight.heat_cell(), theme::HEATMAP_CELL_MIN);
    assert!(tight.heat_scrolls());
    // 960×640：单列，内边距 16，主列就是内容宽。
    let narrow = at(960., 640.);
    assert!(narrow.stacked);
    assert_eq!((narrow.padding, narrow.main_w), (16., 743.));
    assert_eq!(narrow.heat_cell(), 10.);
    assert!(!narrow.summary_beside() && !narrow.heat_scrolls());
}

#[test]
fn heat_levels_split_active_days_by_quartile() {
    let thresholds = heat_thresholds(vec![5, 1, 4, 2, 3]);
    assert_eq!(thresholds, [2., 3., 4.]);
    assert_eq!(
        [0, 1, 2, 3, 4, 5].map(|value| heat_bucket(value, thresholds)),
        [0, 1, 1, 2, 3, 4]
    );
    // 线性插值；只有一天时所有阈值相同，落在最低一档而不是最高。
    assert_eq!(heat_thresholds(vec![1, 2, 3, 4]), [1.75, 2.5, 3.25]);
    assert_eq!(heat_bucket(7, heat_thresholds(vec![7])), 1);
    assert_eq!(heat_thresholds(Vec::new()), [0.; 3]);
}

#[test]
fn activity_summary_reads_only_the_53_week_window() {
    let grid = HeatGrid::new(&activity_fixture()).unwrap_or_else(|| panic!("fixture grid"));
    assert_eq!(grid.longest_streak(), 4);
    assert_eq!(
        grid.busiest(),
        Some((
            NaiveDate::from_ymd_opt(2026, 11, 4).unwrap_or_else(|| panic!("fixture date")),
            20
        ))
    );
    let mut unreadable = activity_fixture();
    unreadable.trend_start = "not a date".into();
    assert!(HeatGrid::new(&unreadable).is_none());
}

#[test]
fn week_change_never_divides_by_zero() {
    assert_eq!(week_change(0, 0), WeekChange::Same);
    assert_eq!(week_change(5, 5), WeekChange::Same);
    assert_eq!(week_change(5, 0), WeekChange::New);
    assert_eq!(week_change(120, 100), WeekChange::Up(20));
    assert_eq!(week_change(50, 100), WeekChange::Down(50));
    assert_eq!(week_change(0, 10), WeekChange::Down(100));
    assert_eq!(week_change(1001, 1000), WeekChange::Up(0));
}

#[test]
fn leaders_rank_real_requests_and_shorten_project_paths() {
    let activity = activity_fixture();
    let models = leader_rows(&activity, Leaders::Models);
    assert_eq!(models.len(), 8);
    assert_eq!(models[0].label, "model-0");
    let projects = leader_rows(&activity, Leaders::Projects);
    assert_eq!(projects[0].label, "thesis");
    assert_eq!(projects[0].full, r"D:\Documents\thesis");
    assert_eq!(projects[1].label, "site");
    // 只有会话、没有请求的来源不进请求排行；已知来源用平台显示名。
    let agents = leader_rows(&activity, Leaders::Agents);
    assert_eq!(agents.len(), 1);
    assert_eq!(agents[0].label, "Kimi");
    assert_eq!(path_leaf("plain"), "plain");
    assert_eq!(path_leaf("/"), "/");
    assert_eq!(peak(&[0, 3, 3, 1]), Some((1, 3)));
    assert_eq!(peak(&[0, 0]), None);
}

#[gpui_kit::test]
fn heatmap_cell_size_and_summary_side_follow_the_viewport(cx: &mut TestAppContext) {
    let stage = Stage::boot(cx);
    for (width, height, cell, beside) in [(1280., 800., 9., false), (2000., 1080., 14., true)] {
        let (handle, view) = stage.open(cx, width, height);
        cx.update_window(handle, |_, window, cx| {
            show(&view, report_fixture(Some(activity_fixture())), window, cx);
            assert_eq!(Geometry::new(window.viewport_size()).heat_cell(), cell);
            let friday = window.find("overview-heat-2026-11-06");
            assert_eq!(friday.bounds().size.width, px(cell));
            assert_eq!(friday.bounds().size.height, px(cell));
            assert_eq!(friday.label(), Some("2026-11-06 · 9 requests"));
            // 周一在上：同一周的周二到周五同列逐行向下；窗口第一格是首周周一。
            let tuesday = window.find("overview-heat-2026-11-03").bounds();
            assert_eq!(tuesday.left(), friday.bounds().left());
            assert_eq!(
                friday.bounds().top() - tuesday.top(),
                px(3. * (cell + theme::HEATMAP_GAP))
            );
            let first = window.find("overview-heat-2025-11-03").bounds();
            let map = window.find("overview-heatmap").bounds();
            assert_eq!(first.top(), map.top());
            assert_eq!(first.left(), map.left() + px(theme::HEATMAP_LABEL_WIDTH));
            // 只有记录日进入快照：5 个记录日，其余格子没有 id。
            assert_eq!(heat_cell_ids(window), 5);
            let summary = window.find("overview-activity-summary").bounds();
            if beside {
                assert!(summary.left() >= map.right(), "{summary:?} vs {map:?}");
                assert_eq!(summary.top(), map.top());
            } else {
                assert!(summary.top() >= map.bottom(), "{summary:?} vs {map:?}");
            }
            let legend = window.find("overview-heat-legend").bounds();
            // 图例紧贴网格下方并与网格右缘对齐；堆叠时摘要排在图例之后。
            assert!(map.bottom() <= legend.top(), "{map:?} vs {legend:?}");
            assert_eq!(legend.right(), map.right());
            if !beside {
                assert!(
                    summary.top() >= legend.bottom(),
                    "{summary:?} vs {legend:?}"
                );
            }
            // 摘要与无障碍句都是 53 周口径，不读全历史的 30 天和 999 次。
            assert_eq!(
                window.find("overview-activity-longest-streak").value(),
                Some("Longest streak: 4 days")
            );
            assert_eq!(
                window.find("overview-activity-busiest").value(),
                Some("Busiest day: 2026-11-04 · 20 requests")
            );
            assert_eq!(
                window.find("overview-heatmap").label(),
                Some(
                    "Last 53 weeks: Active days 5; Current streak 4 days; \
                     Longest streak 4 days; Busiest day 2026-11-04 · 20 requests"
                )
            );
            window.remove_window();
        })
        .unwrap_or_else(|e| panic!("heatmap {width}: {e}"));
    }
    stage.owner.shutdown();
}

fn heat_cell_ids(window: &Window) -> usize {
    gpui_kit::base::test_support::snapshots(window)
        .iter()
        .filter(|node| {
            node.path()
                .last()
                .is_some_and(|id| id.to_string().starts_with("overview-heat-20"))
        })
        .count()
}

#[gpui_kit::test]
fn columns_end_together_and_the_chart_keeps_no_dead_zone(cx: &mut TestAppContext) {
    let stage = Stage::boot(cx);
    for (width, height, rail_width) in [(1280., 800., 280.), (2000., 1080., 360.)] {
        let (handle, view) = stage.open(cx, width, height);
        cx.update_window(handle, |_, window, cx| {
            show(&view, report_fixture(Some(activity_fixture())), window, cx);
            let main = window.find("overview-main").bounds();
            let rail = window.find("overview-rail").bounds();
            assert_eq!(rail.size.width, px(rail_width));
            assert_eq!(main.top(), rail.top());
            assert_eq!(main.bottom(), rail.bottom());
            // 多余高度只进每列最后一个面板；图表面板不伸展。
            let usage = window.find("overview-usage").bounds();
            let axis = window.find("overview-chart-axis").bounds();
            assert!(
                usage.bottom() - axis.bottom() <= px(24.),
                "{usage:?} vs {axis:?}"
            );
            let activity = window.find("overview-activity").bounds();
            assert_eq!(activity.bottom(), main.bottom());
            assert_eq!(
                window.find("overview-data-source").bounds().bottom(),
                rail.bottom()
            );
            // 吸收的余量均分在内容上下：内容块在活动面板主体内垂直居中，不沉底也不贴顶。
            let body = window.find("overview-activity-body").bounds();
            let block = window.find("overview-activity-block").bounds();
            let (above, below) = (block.top() - body.top(), body.bottom() - block.bottom());
            assert!(
                above <= below + px(1.) && below <= above + px(1.),
                "{block:?} in {body:?}"
            );
            assert_eq!(
                activity.bottom() - body.bottom(),
                px(theme::SECTION_GAP + 1.)
            );
            // 右栏三个面板依次排列，洞察行在两列之下且 1:1 等高。
            assert!(
                window.find("overview-actions").bounds().bottom()
                    <= window.find("overview-week").bounds().top()
            );
            assert!(
                window.find("overview-week").bounds().bottom()
                    <= window.find("overview-data-source").bounds().top()
            );
            let distribution = window.find("overview-distribution").bounds();
            let leaders = window.find("overview-leaders").bounds();
            assert!(distribution.top() >= main.bottom());
            assert_eq!(distribution.top(), leaders.top());
            assert_eq!(distribution.bottom(), leaders.bottom());
            assert_eq!(distribution.size.width, leaders.size.width);
            window.remove_window();
        })
        .unwrap_or_else(|e| panic!("columns {width}: {e}"));
    }
    stage.owner.shutdown();
}

#[gpui_kit::test]
fn activity_survives_an_unavailable_archive_and_sessions_are_named(cx: &mut TestAppContext) {
    let stage = Stage::boot(cx);
    let (handle, view) = stage.open(cx, 1280., 800.);
    cx.update_window(handle, |_, window, cx| {
        let degraded = |scope| {
            let mut activity = activity_fixture();
            activity.sessions_indexed = false;
            activity.last_7_days.sessions = 0;
            activity.previous_7_days.sessions = 0;
            WorkbenchReport {
                sessions_indexed: false,
                session_count: None,
                session_scope: scope,
                ..report_fixture(Some(activity))
            }
        };
        show(&view, degraded(SessionScope::Unavailable), window, cx);
        assert!(window.find("overview-activity").visible());
        assert!(window.try_find("overview-heat-2026-11-06").is_some());
        assert!(window.try_find("overview-insights-row").is_some());
        assert_eq!(
            window.find("overview-week-sessions").value(),
            Some("Sessions: Unavailable")
        );
        assert_eq!(
            window.find("overview-total-sessions").value(),
            Some("Sessions: Session count unavailable")
        );
        show(&view, degraded(SessionScope::Unindexed), window, cx);
        assert_eq!(
            window.find("overview-week-sessions").value(),
            Some("Sessions: Not indexed")
        );
        // 读得到时才给对比；前期为 0 写 new，不做除法。
        show(&view, report_fixture(Some(activity_fixture())), window, cx);
        assert_eq!(
            window.find("overview-week-requests").value(),
            Some("Requests: last 7 days 120, prior 7 days 100, up 20%")
        );
        assert_eq!(
            window.find("overview-week-sessions").value(),
            Some("Sessions: last 7 days 4, prior 7 days 4, no change")
        );
        assert_eq!(
            window.find("overview-week-active-days").value(),
            Some("Active days: last 7 days 3, prior 7 days 0, new")
        );
        window.remove_window();
    })
    .unwrap_or_else(|e| panic!("archive unavailable: {e}"));
    stage.owner.shutdown();
}

#[gpui_kit::test]
fn missing_projection_states_its_reason_and_draws_no_insights(cx: &mut TestAppContext) {
    let stage = Stage::boot(cx);
    let (handle, view) = stage.open(cx, 1280., 800.);
    cx.update_window(handle, |_, window, cx| {
        show(&view, report_fixture(None), window, cx);
        assert_eq!(
            window.find("overview-activity-unavailable").label(),
            Some("Activity unavailable")
        );
        assert!(window.find("overview-week-unavailable").visible());
        assert!(window.try_find("overview-heatmap").is_none());
        assert!(window.try_find("overview-insights-row").is_none());
        // 主列与右栏仍然等高，KPI 与图表不受影响。
        assert_eq!(
            window.find("overview-main").bounds().bottom(),
            window.find("overview-rail").bounds().bottom()
        );
        assert!(window.find("overview-chart-axis").visible());
        window.remove_window();
    })
    .unwrap_or_else(|e| panic!("missing projection: {e}"));
    stage.owner.shutdown();
}

#[gpui_kit::test]
fn freshness_is_stated_in_the_header_chart_and_data_source(cx: &mut TestAppContext) {
    let stage = Stage::boot(cx);
    let (handle, view) = stage.open(cx, 1280., 800.);
    cx.update_window(handle, |_, window, cx| {
        show(&view, report_fixture(None), window, cx);
        assert_eq!(
            window.find("overview-last-sync").label(),
            Some("Last successful sync 11-06 02:04")
        );
        for id in ["overview-last-sync-stale", "overview-source-stale"] {
            assert!(window.try_find(id).is_none(), "fresh report shows {id}");
        }
        // 末尾无记录的日子只写“最新记录”这一事实，与同步是否过期无关。
        assert_eq!(
            window.find("overview-chart-latest").label(),
            Some("· latest record 11-05")
        );
        show(
            &view,
            WorkbenchReport {
                last_sync_at: Some("2026-11-05T14:34:00Z".into()),
                freshness: SyncFreshness {
                    local: Some("11-05 14:34".into()),
                    stale: true,
                },
                ..report_fixture(None)
            },
            window,
            cx,
        );
        assert_eq!(
            window.find("overview-last-sync").label(),
            Some(
                "Last successful sync 11-05 14:34. Newer usage may be missing until a sync succeeds"
            )
        );
        assert!(window.find("overview-last-sync-stale").visible());
        assert_eq!(
            window.find("overview-chart-latest").label(),
            Some("· latest record 11-05")
        );
        assert_eq!(
            window.find("overview-source-stale").label(),
            Some("Newer usage may be missing until a sync succeeds")
        );
        // 窗口最后一天有记录：没有末尾空白，不写“最新记录”。
        let mut through_today = report_fixture(None);
        if let Some(last) = through_today.days.last_mut() {
            last.codex = 3;
        }
        show(&view, through_today, window, cx);
        assert!(window.try_find("overview-chart-latest").is_none());
        // 没有成功记录：写明未记录并算过期，但不追加“可能缺失”的推断句。
        let unknown = |raw: Option<&str>| WorkbenchReport {
            last_sync_at: raw.map(Into::into),
            freshness: SyncFreshness {
                local: None,
                stale: true,
            },
            ..report_fixture(None)
        };
        show(&view, unknown(None), window, cx);
        assert_eq!(
            window.find("overview-last-sync").label(),
            Some("No successful sync recorded")
        );
        assert!(window.find("overview-last-sync-stale").visible());
        assert!(window.try_find("overview-source-stale").is_none());
        show(&view, unknown(Some("yesterday")), window, cx);
        assert_eq!(
            window.find("overview-last-sync").label(),
            Some("Sync time unreadable")
        );
        window.remove_window();
    })
    .unwrap_or_else(|e| panic!("freshness: {e}"));
    stage.owner.shutdown();
}

#[gpui_kit::test]
fn no_report_keeps_columns_level_without_placeholder_panels(cx: &mut TestAppContext) {
    let stage = Stage::boot(cx);
    let (handle, _view) = stage.open(cx, 1280., 800.);
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        for id in [
            "overview-activity",
            "overview-week",
            "overview-insights-row",
            "overview-last-sync",
        ] {
            assert!(window.try_find(id).is_none(), "no report shows {id}");
        }
        // 主列只有用量面板，由它吸收与右栏的高度差。
        let usage = window.find("overview-usage").bounds();
        let rail = window.find("overview-rail").bounds();
        assert_eq!(usage.bottom(), rail.bottom());
        assert_eq!(
            window.find("overview-data-source").bounds().bottom(),
            rail.bottom()
        );
        window.remove_window();
    })
    .unwrap_or_else(|e| panic!("no report: {e}"));
    stage.owner.shutdown();
}

#[gpui_kit::test]
fn chinese_copy_keeps_scopes_and_freshness(cx: &mut TestAppContext) {
    let stage = Stage::boot(cx);
    cx.update(|cx| cx.set_global(crate::locales::Language::Chinese));
    let (handle, view) = stage.open(cx, 1280., 1600.);
    cx.update_window(handle, |_, window, cx| {
        show(
            &view,
            WorkbenchReport {
                freshness: SyncFreshness {
                    local: Some("11-05 14:34".into()),
                    stale: true,
                },
                ..report_fixture(Some(activity_fixture()))
            },
            window,
            cx,
        );
        assert_eq!(
            window.find("overview-last-sync").label(),
            Some("上次成功同步 11-05 14:34。同步成功前，较新的用量可能缺失")
        );
        assert_eq!(
            window.find("overview-chart-latest").label(),
            Some("· 最新记录 11-05")
        );
        assert_eq!(
            window.find("overview-heat-2026-11-06").label(),
            Some("2026-11-06 · 9 次请求")
        );
        assert_eq!(
            window.find("overview-week-active-days").value(),
            Some("活跃天数: 近 7 日 3, 前 7 日 0, 新增")
        );
        assert_eq!(
            window.find("overview-dist-caption").label(),
            Some("峰值 14:00 · 40 次请求 · 全历史")
        );
        window.remove_window();
    })
    .unwrap_or_else(|e| panic!("chinese copy: {e}"));
    stage.owner.shutdown();
}

#[gpui_kit::test]
fn empty_daily_draws_no_zero_cells(cx: &mut TestAppContext) {
    let stage = Stage::boot(cx);
    let (handle, view) = stage.open(cx, 1280., 800.);
    cx.update_window(handle, |_, window, cx| {
        let mut activity = activity_fixture();
        activity.daily = Vec::new();
        show(&view, report_fixture(Some(activity)), window, cx);
        assert_eq!(
            window.find("overview-activity-empty").label(),
            Some("No requests in the last 53 weeks")
        );
        assert_eq!(heat_cell_ids(window), 0);
        for id in [
            "overview-heatmap",
            "overview-heat-weeks",
            "overview-heat-legend",
            "overview-activity-summary",
        ] {
            assert!(window.try_find(id).is_none(), "empty daily shows {id}");
        }
        window.remove_window();
    })
    .unwrap_or_else(|e| panic!("empty daily: {e}"));
    stage.owner.shutdown();
}

#[gpui_kit::test]
fn narrow_window_is_a_single_column_without_page_overflow(cx: &mut TestAppContext) {
    let stage = Stage::boot(cx);
    let (handle, view) = stage.open(cx, 960., 640.);
    cx.update_window(handle, |_, window, cx| {
        show(&view, report_fixture(Some(activity_fixture())), window, cx);
        let page = window.find("overview-root").bounds();
        assert_eq!(
            window.find("overview-heading").bounds().left() - page.left(),
            px(16.)
        );
        let main = window.find("overview-main").bounds();
        let rail = window.find("overview-rail").bounds();
        assert_eq!(main.left(), rail.left());
        assert_eq!(main.size.width, rail.size.width);
        assert!(rail.top() >= main.bottom(), "{rail:?} vs {main:?}");
        // KPI 2×2。
        let cost = window.find("overview-total-cost").bounds();
        assert_eq!(
            cost.top(),
            window.find("overview-total-requests").bounds().top()
        );
        assert!(window.find("overview-total-tokens").bounds().top() >= cost.bottom());
        // 洞察行也堆叠。
        let distribution = window.find("overview-distribution").bounds();
        let leaders = window.find("overview-leaders").bounds();
        assert_eq!(distribution.left(), leaders.left());
        assert!(leaders.top() >= distribution.bottom());
        // 整页没有可横滚的量，所有面板都在页面内边距之内。
        assert_eq!(view.read(cx).scroll.max_offset().x, px(0.));
        // 窄宽没有确定高度：吸收余量的面板仍要按内容撑开，不能被压扁或互相重叠。
        let activity = window.find("overview-activity").bounds();
        assert!(activity.top() >= window.find("overview-usage").bounds().bottom());
        let legend = window.find("overview-heat-legend").bounds();
        let summary = window.find("overview-activity-summary").bounds();
        assert!(
            legend.bottom() <= summary.top(),
            "{legend:?} vs {summary:?}"
        );
        assert_eq!(
            activity.bottom() - summary.bottom(),
            px(theme::SECTION_GAP + 1.)
        );
        let source = window.find("overview-data-source").bounds();
        assert!(source.top() >= window.find("overview-week").bounds().bottom());
        assert!(window.find("overview-source-state").bounds().bottom() <= source.bottom());
        for id in [
            "overview-kpis",
            "overview-platform-band",
            "overview-usage",
            "overview-activity",
            "overview-week",
            "overview-data-source",
            "overview-distribution",
            "overview-leaders",
        ] {
            let bounds = window.find(id).bounds();
            assert!(
                bounds.right() <= page.right() - px(16.),
                "{id} overflows the page: {bounds:?} vs {page:?}"
            );
        }
        window.remove_window();
    })
    .unwrap_or_else(|e| panic!("narrow: {e}"));
    stage.owner.shutdown();
}

#[gpui_kit::test]
fn distribution_and_leader_tabs_switch_in_place(cx: &mut TestAppContext) {
    let stage = Stage::boot(cx);
    // 洞察行在首屏之下；窗口够高才能直接点到胶囊。
    let (handle, view) = stage.open(cx, 1280., 1600.);
    cx.update_window(handle, |_, window, cx| {
        show(&view, report_fixture(Some(activity_fixture())), window, cx);
        // 分布：默认按小时，24 柱，峰值写进说明。
        assert_eq!(window.find("overview-dist-hourly").checked(), Some(true));
        assert_eq!(window.find("overview-dist-weekday").checked(), Some(false));
        assert_eq!(
            window.find("overview-dist-caption").label(),
            Some("Peak 14:00 · 40 requests · All history")
        );
        assert_eq!(
            window.find("overview-dist-bar-14").label(),
            Some("14:00 · 40 requests")
        );
        window.click("overview-dist-weekday", cx);
        assert_eq!(window.find("overview-dist-hourly").checked(), Some(false));
        assert_eq!(window.find("overview-dist-weekday").checked(), Some(true));
        assert!(window.try_find("overview-dist-bar-23").is_none());
        assert_eq!(
            window.find("overview-dist-bar-1").label(),
            Some("Tue · 30 requests")
        );
        assert_eq!(
            window.find("overview-dist-caption").label(),
            Some("Peak Tue · 30 requests · All history")
        );
        // 排行：默认模型，只取前 8。
        assert_eq!(window.find("overview-leaders-models").checked(), Some(true));
        assert_eq!(
            window.find("overview-leader-0").label(),
            Some("model-0 · 100 requests")
        );
        assert!(window.try_find("overview-leader-7").is_some());
        assert!(window.try_find("overview-leader-8").is_none());
        window.click("overview-leaders-projects", cx);
        assert_eq!(
            window.find("overview-leaders-models").checked(),
            Some(false)
        );
        assert_eq!(
            window.find("overview-leaders-projects").checked(),
            Some(true)
        );
        // 名称显示末段，无障碍标签保留完整路径。
        assert_eq!(
            window.find("overview-leader-0").label(),
            Some(r"D:\Documents\thesis · 12 requests")
        );
        assert!(window.try_find("overview-leader-2").is_none());
        window.click("overview-leaders-agents", cx);
        assert_eq!(window.find("overview-leaders-agents").checked(), Some(true));
        assert_eq!(
            window.find("overview-leader-0").label(),
            Some("Kimi · 9 requests")
        );
        assert!(window.try_find("overview-leader-1").is_none());
        // 空榜与全零分布写原因，不铺 0。
        let mut empty = activity_fixture();
        empty.agents = Vec::new();
        empty.hourly = vec![0; 24];
        empty.weekday = vec![0; 7];
        show(&view, report_fixture(Some(empty)), window, cx);
        assert_eq!(
            window.find("overview-leaders-empty").label(),
            Some("No records")
        );
        assert_eq!(
            window.find("overview-dist-empty").label(),
            Some("No requests recorded · All history")
        );
        assert!(window.try_find("overview-dist-bars").is_none());
        window.remove_window();
    })
    .unwrap_or_else(|e| panic!("tabs: {e}"));
    stage.owner.shutdown();
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
                last_sync_at: None,
                freshness: SyncFreshness {
                    local: None,
                    stale: true,
                },
                activity: None,
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
            // 没有“未画入”说明时，轴标签就是图表面板的最后一行。
            report.other_requests = 0;
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
        // KPI 四栏同一行；右栏与主列等高，图表面板内没有死区。
        assert_eq!(
            window.find("overview-total-cost").bounds().top(),
            window.find("overview-total-requests").bounds().top()
        );
        assert_eq!(
            window.find("overview-rail").bounds().bottom(),
            window.find("overview-main").bounds().bottom()
        );
        assert!(
            window.find("overview-usage").bounds().bottom()
                - window.find("overview-chart-axis").bounds().bottom()
                <= px(24.)
        );
        assert!(window.find("overview-data-source").visible());
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
