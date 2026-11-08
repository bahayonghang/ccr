//! Activity panel (53-week request heat map) and the seven-day comparison.
use super::{Geometry, Node, compact_count, grouped, panel};
use crate::{
    locales::{is_chinese, text},
    runtime::insights::{SessionScope, WorkbenchReport},
    shell::badge,
    theme::{self, Tone},
};
use ccr_app::usage::home::HomeInsightsSnapshot;
use chrono::NaiveDate;
use gpui_kit::base::TestSupportExt;
use gpui_kit::component::{ActiveTheme, Icon, IconName, tooltip::Tooltip};
use gpui_kit::{
    AnyElement, App, Div, InteractiveElement, IntoElement, ParentElement, Role, ScrollHandle,
    SharedString, StatefulInteractiveElement, Styled, div, px, relative,
};
use std::collections::BTreeMap;

const HEAT_WEEKS: usize = 53;
const HEAT_DAYS: usize = 7;
/// 网格最宽时的周列总宽；用作“滚到最新一周”的偏移，超出部分由布局钳制。
pub(super) const HEAT_MAX_SCROLL: f32 =
    HEAT_WEEKS as f32 * (theme::HEATMAP_CELL_MAX + theme::HEATMAP_GAP);
/// 网格右侧放得下摘要所需的宽度。
const SUMMARY_MIN_WIDTH: f32 = 200.;
const WEEK_VALUE_WIDTH: f32 = 56.;

impl Geometry {
    /// 阅读面板内宽：主列宽减去两侧 1px 边框与 16px 内边距。
    pub(super) fn panel_inner(self) -> f32 {
        self.main_w - 2. * (1. + theme::SECTION_GAP)
    }

    /// Heat-map cell size: as wide as the panel allows within the cell bounds.
    pub(super) fn heat_cell(self) -> f32 {
        let gaps = (HEAT_WEEKS - 1) as f32 * theme::HEATMAP_GAP;
        ((self.panel_inner() - theme::HEATMAP_LABEL_WIDTH - gaps) / HEAT_WEEKS as f32)
            .floor()
            .clamp(theme::HEATMAP_CELL_MIN, theme::HEATMAP_CELL_MAX)
    }

    pub(super) fn heat_width(self) -> f32 {
        theme::HEATMAP_LABEL_WIDTH
            + HEAT_WEEKS as f32 * self.heat_cell()
            + (HEAT_WEEKS - 1) as f32 * theme::HEATMAP_GAP
    }

    pub(super) fn summary_beside(self) -> bool {
        self.heat_width() + SUMMARY_MIN_WIDTH <= self.panel_inner()
    }

    /// 取最小格仍放不下：网格在面板内横向滚动。
    pub(super) fn heat_scrolls(self) -> bool {
        self.heat_width() > self.panel_inner()
    }
}

/// Parsed heat-map dates. `None` from [`HeatGrid::new`] means the snapshot's
/// report dates are unreadable, which the panel states instead of guessing.
pub(super) struct HeatGrid {
    start: NaiveDate,
    as_of: NaiveDate,
    /// 53 周窗口内有请求的日期；摘要与网格同一口径。
    days: BTreeMap<NaiveDate, i64>,
    thresholds: [f64; 3],
}

impl HeatGrid {
    pub(super) fn new(activity: &HomeInsightsSnapshot) -> Option<Self> {
        let parse = |value: &str| NaiveDate::parse_from_str(value, "%Y-%m-%d").ok();
        let days = activity
            .daily
            .iter()
            .filter(|day| day.requests > 0)
            .map(|day| Some((parse(&day.date)?, day.requests)))
            .collect::<Option<BTreeMap<_, _>>>()?;
        let thresholds = heat_thresholds(days.values().copied().collect());
        Some(Self {
            start: parse(&activity.trend_start)?,
            as_of: parse(&activity.as_of)?,
            days,
            thresholds,
        })
    }

    pub(super) fn longest_streak(&self) -> u32 {
        let (mut longest, mut run) = (0, 0);
        let mut previous: Option<NaiveDate> = None;
        for day in self.days.keys() {
            run = if previous.and_then(|day| day.succ_opt()) == Some(*day) {
                run + 1
            } else {
                1
            };
            longest = longest.max(run);
            previous = Some(*day);
        }
        longest
    }

    /// Most requested day; ties resolve to the earlier day.
    pub(super) fn busiest(&self) -> Option<(NaiveDate, i64)> {
        let mut best: Option<(NaiveDate, i64)> = None;
        for (day, requests) in &self.days {
            if best.is_none_or(|(_, top)| *requests > top) {
                best = Some((*day, *requests));
            }
        }
        best
    }
}

/// 线性插值四分位（与 Tauri 概览同一算法），只统计非零日。
pub(super) fn heat_thresholds(mut values: Vec<i64>) -> [f64; 3] {
    values.sort_unstable();
    [0.25, 0.5, 0.75].map(|quantile| {
        let last = values.len().saturating_sub(1);
        let position = last as f64 * quantile;
        let lower = position.floor() as usize;
        let upper = (lower + 1).min(last);
        match (values.get(lower), values.get(upper)) {
            (Some(low), Some(high)) => {
                *low as f64 + (*high - *low) as f64 * (position - lower as f64)
            }
            _ => 0.,
        }
    })
}

pub(super) fn heat_bucket(requests: i64, thresholds: [f64; 3]) -> u8 {
    let value = requests as f64;
    if requests <= 0 {
        0
    } else if value <= thresholds[0] {
        1
    } else if value <= thresholds[1] {
        2
    } else if value <= thresholds[2] {
        3
    } else {
        4
    }
}

/// Activity panel. The caller builds it only once a report exists.
pub(super) fn activity_panel(
    report: &WorkbenchReport,
    geometry: Geometry,
    scroll: &ScrollHandle,
    cx: &App,
) -> Node {
    let frame = panel("overview-activity", cx).flex_1();
    let grid = report.activity.as_ref().and_then(|activity| {
        let grid = HeatGrid::new(activity)?;
        Some((activity, grid))
    });
    match grid {
        Some((activity, grid)) => frame
            .child(activity_title(activity, cx))
            .child(activity_body(activity, &grid, geometry, scroll, cx)),
        None => frame
            .child(badge::section_heading(
                text(cx, "Activity", "活动"),
                None,
                cx,
            ))
            .child(activity_unavailable(cx)),
    }
}

fn activity_unavailable(cx: &App) -> Node {
    badge::empty_panel(
        "overview-activity-unavailable",
        IconName::CircleAlert,
        text(cx, "Activity unavailable", "活动不可用"),
        text(
            cx,
            "The usage database could not answer the activity query. Check the llmusage version, then refresh.",
            "用量库未能返回活动数据，请检查 llmusage 版本后刷新。",
        ),
        None,
        cx,
    )
    .test_support()
    .flex_1()
}

fn activity_title(activity: &HomeInsightsSnapshot, cx: &App) -> Div {
    let scope = activity_scope(&activity.trend_start, is_chinese(cx));
    badge::section_heading(text(cx, "Activity", "活动"), Some(scope.into()), cx)
}

/// Scope label of the heat map. The window starts a year back, so the date
/// keeps its year: "since 11-03" would read as this month.
pub(super) fn activity_scope(trend_start: &str, chinese: bool) -> String {
    if chinese {
        format!("最近 53 周 · 自 {trend_start} 起")
    } else {
        format!("Last 53 weeks · since {trend_start}")
    }
}

fn activity_body(
    activity: &HomeInsightsSnapshot,
    grid: &HeatGrid,
    geometry: Geometry,
    scroll: &ScrollHandle,
    cx: &App,
) -> AnyElement {
    // 53 周内没有记录：直接写明，网格不铺 0 格。
    if grid.days.is_empty() {
        let message = text(cx, "No requests in the last 53 weeks", "最近 53 周没有请求");
        return div()
            .id("overview-activity-empty")
            .test_support()
            .role(Role::Label)
            .aria_label(message)
            .text_color(cx.theme().muted_foreground)
            .child(message)
            .into_any_element();
    }
    let readings = summary_readings(activity, grid, cx);
    let sentence = summary_sentence(&readings, cx);
    let beside = geometry.summary_beside();
    // 图例紧贴网格并与网格右缘对齐（同一列），不再沉到面板底部悬空。
    let map = div()
        .flex()
        .flex_col()
        .gap(px(theme::CONTROL_GAP))
        .child(heatmap(grid, geometry, sentence, scroll, cx))
        .child(legend(geometry.heat_cell(), cx));
    let details = summary(readings, geometry, cx);
    let block = div()
        .id("overview-activity-block")
        .test_support()
        .flex()
        .gap(px(theme::SECTION_GAP));
    let block = if beside {
        block
            .items_start()
            .child(map.flex_shrink_0().w(px(geometry.heat_width())))
            .child(details.flex_1())
    } else {
        // 网格不横滚时列宽收到网格宽，让图例仍与网格右缘对齐。
        let map = if geometry.heat_scrolls() {
            map.w_full()
        } else {
            map.flex_shrink_0()
                .w(px(geometry.heat_width()))
                .max_w_full()
        };
        block.flex_col().child(map).child(details.w_full())
    };
    // 面板为与右栏等高而吸收的余量均分在内容上下，而不是全部堆在底部形成死区。
    div()
        .id("overview-activity-body")
        .test_support()
        .flex_1()
        .flex()
        .flex_col()
        .justify_center()
        .child(block)
        .into_any_element()
}

fn heatmap(
    grid: &HeatGrid,
    geometry: Geometry,
    sentence: String,
    scroll: &ScrollHandle,
    cx: &App,
) -> Node {
    let cell = geometry.heat_cell();
    let mut weeks = div()
        .id("overview-heat-weeks")
        .test_support()
        .flex_1()
        .min_w_0()
        .flex()
        .gap(px(theme::HEATMAP_GAP))
        .overflow_x_scroll()
        // 竖向滚轮只滚整页，不顺带横移网格。
        .restrict_scroll_to_axis()
        .track_scroll(scroll);
    let mut dates = grid.start.iter_days();
    for _ in 0..HEAT_WEEKS {
        weeks = weeks.child(heat_week(grid, &mut dates, cell, cx));
    }
    // 星期标签列在滚动区之外，横滚时始终可见。
    div()
        .id("overview-heatmap")
        .test_support()
        .role(Role::Group)
        .aria_label(sentence)
        .min_w_0()
        .flex()
        .child(weekday_labels(cell, cx))
        .child(weeks)
}

fn heat_week(
    grid: &HeatGrid,
    dates: &mut impl Iterator<Item = NaiveDate>,
    cell: f32,
    cx: &App,
) -> Div {
    let mut column = div()
        .flex_shrink_0()
        .flex()
        .flex_col()
        .gap(px(theme::HEATMAP_GAP));
    for date in dates.take(HEAT_DAYS) {
        column = column.child(heat_day(grid, date, cell, cx));
    }
    column
}

// 只有有记录的格子带 id 和 tooltip：371 格里多数是静态色块，不进入无障碍树。
fn heat_day(grid: &HeatGrid, date: NaiveDate, cell: f32, cx: &App) -> AnyElement {
    let mode = cx.theme().mode;
    let base = div()
        .flex_shrink_0()
        .size(px(cell))
        .rounded(px(theme::CHART_BAR_RADIUS));
    // 晚于报告日的格子只占位，不画。
    if date > grid.as_of {
        return base.into_any_element();
    }
    let Some(requests) = grid.days.get(&date).copied() else {
        return base.bg(theme::heat_level(0, mode)).into_any_element();
    };
    let detail: SharedString = format!(
        "{date} · {} {}",
        grouped(requests),
        text(cx, "requests", "次请求")
    )
    .into();
    let level = heat_bucket(requests, grid.thresholds);
    base.id(SharedString::from(format!("overview-heat-{date}")))
        .test_support()
        .role(Role::Label)
        .aria_label(detail.clone())
        .bg(theme::heat_level(level, mode))
        .tooltip(move |window, cx| Tooltip::new(detail.clone()).build(window, cx))
        .into_any_element()
}

fn weekday_labels(cell: f32, cx: &App) -> Div {
    let names = if is_chinese(cx) {
        ["周一", "周三", "周五"]
    } else {
        ["Mon", "Wed", "Fri"]
    };
    let mut column = div()
        .flex_shrink_0()
        .w(px(theme::HEATMAP_LABEL_WIDTH))
        .flex()
        .flex_col()
        .gap(px(theme::HEATMAP_GAP))
        .text_size(px(theme::META_SIZE))
        .text_color(cx.theme().muted_foreground);
    for row in 0..HEAT_DAYS {
        // 行高等于格子高，文字以格子中线为准居中；相邻行没有标签，不会重叠。
        let label = (row % 2 == 0)
            .then(|| names.get(row / 2))
            .flatten()
            .copied();
        column = column.child(
            div()
                .h(px(cell))
                .line_height(px(cell))
                .whitespace_nowrap()
                .children(label),
        );
    }
    column
}

fn legend(cell: f32, cx: &App) -> Node {
    let mode = cx.theme().mode;
    let mut row = div()
        .id("overview-heat-legend")
        .test_support()
        .role(Role::Label)
        .aria_label(text(
            cx,
            "Colour scale: fewer to more requests",
            "色阶：请求由少到多",
        ))
        .flex()
        .items_center()
        .justify_end()
        .gap(px(theme::HEATMAP_GAP))
        .text_size(px(theme::META_SIZE))
        .text_color(cx.theme().muted_foreground)
        .child(div().mr(px(4.)).child(text(cx, "Less", "少")));
    for level in 0..=4 {
        row = row.child(
            div()
                .size(px(cell))
                .rounded(px(theme::CHART_BAR_RADIUS))
                .bg(theme::heat_level(level, mode)),
        );
    }
    row.child(div().ml(px(4.)).child(text(cx, "More", "多")))
}

struct Reading {
    id: &'static str,
    label: &'static str,
    value: String,
    /// 带单位的完整读法，写入无障碍标签与摘要句。
    spoken: String,
}

fn day_count(value: u32, cx: &App) -> String {
    match (value, is_chinese(cx)) {
        (_, true) => format!("{value} 天"),
        (1, false) => "1 day".into(),
        (_, false) => format!("{value} days"),
    }
}

/// 摘要全部取 53 周内的 `daily`，与网格同一口径；当前连续本身就截止到报告日。
fn summary_readings(activity: &HomeInsightsSnapshot, grid: &HeatGrid, cx: &App) -> Vec<Reading> {
    let plain = |id, label: &'static str, value: String| Reading {
        id,
        label,
        spoken: value.clone(),
        value,
    };
    let mut readings = vec![
        plain(
            "overview-activity-active-days",
            text(cx, "Active days", "活跃天数"),
            grouped(grid.days.len() as i64),
        ),
        plain(
            "overview-activity-current-streak",
            text(cx, "Current streak", "当前连续"),
            day_count(activity.current_streak, cx),
        ),
        plain(
            "overview-activity-longest-streak",
            text(cx, "Longest streak", "最长连续"),
            day_count(grid.longest_streak(), cx),
        ),
    ];
    if let Some((day, requests)) = grid.busiest() {
        readings.push(Reading {
            id: "overview-activity-busiest",
            label: text(cx, "Busiest day", "最忙日"),
            value: format!("{day} · {}", grouped(requests)),
            spoken: format!(
                "{day} · {} {}",
                grouped(requests),
                text(cx, "requests", "次请求")
            ),
        });
    }
    readings
}

fn summary_sentence(readings: &[Reading], cx: &App) -> String {
    let body = readings
        .iter()
        .map(|reading| format!("{} {}", reading.label, reading.spoken))
        .collect::<Vec<_>>()
        .join(text(cx, "; ", "；"));
    format!("{}{body}", text(cx, "Last 53 weeks: ", "最近 53 周："))
}

fn summary(readings: Vec<Reading>, geometry: Geometry, cx: &App) -> Node {
    // 网格右侧或很窄时 2×2，其余一行四格。
    let share = if geometry.summary_beside() || geometry.heat_scrolls() {
        0.5
    } else {
        0.25
    };
    let mut summary = div()
        .id("overview-activity-summary")
        .test_support()
        .min_w_0()
        .flex()
        .flex_wrap()
        .gap_y(px(12.));
    for reading in readings {
        summary = summary.child(summary_reading(reading, share, cx));
    }
    summary
}

fn summary_reading(reading: Reading, share: f32, cx: &App) -> Node {
    let description = format!("{}: {}", reading.label, reading.spoken);
    div()
        .id(reading.id)
        .test_support()
        .role(Role::Label)
        .aria_label(description.clone())
        .aria_value(description)
        .w(relative(share))
        .flex()
        .flex_col()
        .gap(px(2.))
        .child(
            div()
                .text_size(px(theme::META_SIZE))
                .text_color(cx.theme().muted_foreground)
                .child(reading.label),
        )
        .child(
            div()
                .font_features(theme::tabular_figures())
                .font_weight(gpui_kit::FontWeight::MEDIUM)
                .child(reading.value),
        )
}

/// Period-over-period change, never a division by zero.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum WeekChange {
    /// 前期为 0、本期不为 0：没有可比的基数。
    New,
    Same,
    Up(i64),
    Down(i64),
}

pub(super) fn week_change(current: i64, previous: i64) -> WeekChange {
    if current == previous {
        return WeekChange::Same;
    }
    if previous <= 0 {
        return WeekChange::New;
    }
    let percent = ((current - previous).abs() as f64 / previous as f64 * 100.).round() as i64;
    if current > previous {
        WeekChange::Up(percent)
    } else {
        WeekChange::Down(percent)
    }
}

// 四舍五入到 0 的非零变化写“<1%”，不能和持平混淆。
fn percent_text(percent: i64) -> String {
    if percent == 0 {
        "<1%".into()
    } else {
        format!("{}%", grouped(percent))
    }
}

fn change_spoken(change: WeekChange, cx: &App) -> String {
    match change {
        WeekChange::New => text(cx, "new", "新增").into(),
        WeekChange::Same => text(cx, "no change", "持平").into(),
        WeekChange::Up(percent) => format!("{} {}", text(cx, "up", "上升"), percent_text(percent)),
        WeekChange::Down(percent) => {
            format!("{} {}", text(cx, "down", "下降"), percent_text(percent))
        }
    }
}

/// Seven-day comparison. Sessions follow the honesty rule: an unknown count is
/// named as unknown and never drawn as zero.
pub(super) fn week_panel(report: &WorkbenchReport, cx: &App) -> Node {
    let frame = panel("overview-week", cx)
        .flex_shrink_0()
        .gap(px(theme::CONTROL_GAP))
        .text_size(px(theme::META_SIZE))
        .font_features(theme::tabular_figures())
        .child(badge::section_heading(
            text(cx, "7-day comparison", "近 7 日对比"),
            None,
            cx,
        ));
    let Some(activity) = report.activity.as_ref() else {
        return frame.child(
            div()
                .id("overview-week-unavailable")
                .test_support()
                .text_color(cx.theme().muted_foreground)
                .child(text(cx, "Comparison unavailable", "对比不可用")),
        );
    };
    let (current, previous) = (&activity.last_7_days, &activity.previous_7_days);
    let sessions = session_state(report, activity, cx).map_or_else(
        || {
            week_row(
                "overview-week-sessions",
                text(cx, "Sessions", "会话"),
                current.sessions,
                previous.sessions,
                cx,
            )
        },
        |state| {
            week_state_row(
                "overview-week-sessions",
                text(cx, "Sessions", "会话"),
                state,
                cx,
            )
        },
    );
    let table = div()
        .id("overview-week-table")
        .test_support()
        .role(Role::Table)
        .aria_label(text(cx, "7-day comparison", "近 7 日对比"))
        .flex()
        .flex_col()
        .gap(px(theme::CONTROL_GAP))
        .child(week_header(cx))
        .child(week_row(
            "overview-week-requests",
            text(cx, "Requests", "请求"),
            current.requests,
            previous.requests,
            cx,
        ))
        .child(sessions)
        .child(week_row(
            "overview-week-active-days",
            text(cx, "Active days", "活跃天数"),
            current.active_days,
            previous.active_days,
            cx,
        ));
    frame.child(table)
}

/// 会话计数未知的原因；`None` 表示计数可信。
fn session_state(
    report: &WorkbenchReport,
    activity: &HomeInsightsSnapshot,
    cx: &App,
) -> Option<&'static str> {
    match report.session_scope {
        SessionScope::Unavailable => Some(text(cx, "Unavailable", "不可用")),
        SessionScope::Unindexed => Some(text(cx, "Not indexed", "未索引")),
        SessionScope::Window | SessionScope::AllHistory if !activity.sessions_indexed => {
            Some(text(cx, "Not indexed", "未索引"))
        }
        SessionScope::Window | SessionScope::AllHistory => None,
    }
}

fn value_cell(content: impl IntoElement) -> Div {
    div()
        .flex_shrink_0()
        .w(px(WEEK_VALUE_WIDTH))
        .flex()
        .items_center()
        .justify_end()
        .gap(px(2.))
        .whitespace_nowrap()
        .child(content)
}

fn week_header(cx: &App) -> Div {
    div()
        .flex()
        .gap(px(4.))
        .text_color(cx.theme().muted_foreground)
        .child(div().flex_1().min_w_0())
        .child(value_cell(text(cx, "Last 7", "近 7 日")))
        .child(value_cell(text(cx, "Prior 7", "前 7 日")))
        .child(value_cell(text(cx, "Change", "变化")))
}

// 七位数以上才压缩，避免撑破 56px 列；精确值保留在行的无障碍标签里。
fn week_count(value: i64) -> String {
    if value.abs() >= 1_000_000 {
        compact_count(value)
    } else {
        grouped(value)
    }
}

fn week_row(id: &'static str, label: &'static str, current: i64, previous: i64, cx: &App) -> Node {
    let change = week_change(current, previous);
    let description = format!(
        "{label}: {} {}, {} {}, {}",
        text(cx, "last 7 days", "近 7 日"),
        grouped(current),
        text(cx, "prior 7 days", "前 7 日"),
        grouped(previous),
        change_spoken(change, cx),
    );
    div()
        .id(id)
        .test_support()
        .role(Role::Row)
        .aria_label(description.clone())
        .aria_value(description)
        .flex()
        .items_center()
        .gap(px(4.))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .text_color(cx.theme().muted_foreground)
                .child(label),
        )
        .child(value_cell(week_count(current)))
        .child(value_cell(week_count(previous)))
        .child(change_cell(change, cx))
}

fn week_state_row(id: &'static str, label: &'static str, state: &'static str, cx: &App) -> Node {
    let description = format!("{label}: {state}");
    div()
        .id(id)
        .test_support()
        .role(Role::Row)
        .aria_label(description.clone())
        .aria_value(description)
        .flex()
        .items_center()
        .justify_between()
        .gap(px(4.))
        .child(div().text_color(cx.theme().muted_foreground).child(label))
        .child(
            div()
                .text_color(cx.theme().secondary_foreground)
                .child(state),
        )
}

// 变化方向只用形状表达，墨色保持中性，不用红绿暗示好坏。
fn change_cell(change: WeekChange, cx: &App) -> Div {
    let ink = theme::tone_colors(Tone::Neutral, cx.theme().mode).ink;
    let arrow = |name: IconName| Icon::new(name).size(px(theme::META_SIZE)).text_color(ink);
    let cell = value_cell(div()).text_color(ink);
    match change {
        WeekChange::New => cell.child(text(cx, "new", "新增")),
        WeekChange::Same => cell.child("= 0%"),
        WeekChange::Up(percent) => cell
            .child(arrow(IconName::ArrowUp))
            .child(percent_text(percent)),
        WeekChange::Down(percent) => cell
            .child(arrow(IconName::ArrowDown))
            .child(percent_text(percent)),
    }
}
