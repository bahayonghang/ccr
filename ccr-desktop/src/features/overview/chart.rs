//! Daily request chart, its legend and the folded exact-value table.
use super::{Node, OverviewView, grouped, heading, panel, platform_title};
use crate::{
    locales::{is_chinese, text},
    runtime::insights::{WorkbenchDay, WorkbenchReport},
    shell::semantics,
    theme,
};
use gpui_kit::base::TestSupportExt;
use gpui_kit::component::{ActiveTheme, Selectable, button::Button, tooltip::Tooltip};
use gpui_kit::{
    App, Context, FocusHandle, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};

impl OverviewView {
    /// One focus handle per displayed exact-value row (days and folded quiet runs).
    pub(super) fn sync_exact_focus(
        &mut self,
        count: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.exact_rows.truncate(count);
        while self.exact_rows.len() < count {
            let index = self.exact_rows.len();
            let focus = cx.focus_handle().tab_stop(true);
            let subscription = cx.on_focus(&focus, window, move |this, _, cx| {
                // 表头是第一个直接子项，显示行 index 对应子项 index + 1；先在表格内
                // reveal 当前行，再把表格视口 reveal 到外层页面滚动区中。
                this.exact_scroll.scroll_to_item(index + 1);
                let table = this.exact_scroll.bounds();
                let page = this.scroll.bounds();
                let mut offset = this.scroll.offset();
                if table.top() < page.top() {
                    offset.y += page.top() - table.top();
                } else if table.bottom() > page.bottom() {
                    offset.y += page.bottom() - table.bottom();
                }
                this.scroll.set_offset(offset);
                cx.notify();
            });
            self.exact_rows.push((focus, subscription));
        }
    }

    /// The chart panel never stretches. Without a report it is the column's only
    /// panel, so it takes the height difference with the rail.
    pub(super) fn usage_panel(
        &self,
        report: Option<&WorkbenchReport>,
        rows: &[ExactRow],
        cx: &mut Context<Self>,
    ) -> Node {
        let frame = panel("overview-usage", cx).min_w_0();
        let Some(report) = report else {
            return frame
                .flex_1()
                .child(heading(text(cx, "Usage and cost", "用量与成本")))
                .child(
                    div()
                        .text_color(cx.theme().muted_foreground)
                        .child(if self.loading {
                            text(cx, "Reading the selected period…", "正在读取所选时段…")
                        } else {
                            text(cx, "No report available", "暂无可用报告")
                        }),
                );
        };
        frame.flex_shrink_0().child(self.chart(report, rows, cx))
    }

    fn chart(
        &self,
        report: &WorkbenchReport,
        rows: &[ExactRow],
        cx: &mut Context<Self>,
    ) -> gpui_kit::Div {
        // 自动收紧到首个有数据日的前一天；查询窗口仍写在页头状态行。
        let start = trimmed_start(&report.days);
        let visible = &report.days[start..];
        let max = visible.iter().map(day_total).max().unwrap_or(0);
        let mut title = div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap(px(theme::CONTROL_GAP))
            .child(heading(text(cx, "Daily requests", "逐日请求")));
        if let Some(first) = visible.first().filter(|_| start > 0) {
            let since = if is_chinese(cx) {
                format!("自 {} 起", first.label)
            } else {
                format!("since {}", first.label)
            };
            title = title.child(
                div()
                    .id("overview-chart-since")
                    .test_support()
                    .aria_label(since.clone())
                    .text_size(px(theme::META_SIZE))
                    .text_color(cx.theme().muted_foreground)
                    .child(since),
            );
        }
        if let Some(date) = latest_record(&report.days) {
            title = title.child(latest_suffix(date, cx));
        }
        let mut chart = div().flex().flex_col().gap(px(theme::CONTROL_GAP)).child(
            div()
                .flex()
                .flex_wrap()
                .items_center()
                .justify_between()
                .gap(px(theme::CONTROL_GAP))
                .child(title)
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .gap(px(theme::SECTION_GAP))
                        .child(legend(report, cx))
                        .child(semantics::button(
                            Button::new("overview-exact-toggle")
                                .label(text(cx, "Exact values", "精确数值"))
                                .selected(self.show_exact)
                                .toggled(self.show_exact)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.show_exact = !this.show_exact;
                                    cx.notify();
                                })),
                            false,
                            false,
                            cx,
                        )),
                ),
        );
        if visible.is_empty() {
            chart = chart.child(
                div()
                    .py(px(theme::SECTION_GAP))
                    .text_color(cx.theme().muted_foreground)
                    .child(if report.requests == 0 {
                        text(cx, "No requests in this period.", "此时段没有请求。")
                    } else {
                        text(
                            cx,
                            "No daily breakdown available.",
                            "没有可绘制的逐日明细。",
                        )
                    }),
            );
        } else {
            chart = chart
                .child(bars(visible, max, cx))
                .child(ticks(visible, cx));
        }
        chart
            .when(self.show_exact, |el| {
                el.child(self.exact_table(report, rows, cx))
            })
            .when(report.other_requests > 0, |el| {
                el.child(
                    div()
                        .text_size(px(theme::META_SIZE))
                        .text_color(cx.theme().muted_foreground)
                        .child(format!(
                            "{} {}",
                            grouped(report.other_requests),
                            text(
                                cx,
                                "requests included in the total have no bar in this chart.",
                                "条请求计入总数，未画入此图。"
                            )
                        )),
                )
            })
    }

    fn exact_table(
        &self,
        report: &WorkbenchReport,
        rows: &[ExactRow],
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let mut table = div()
            .id("overview-exact-table")
            .test_support()
            .role(gpui_kit::Role::Table)
            .aria_label(text(
                cx,
                "Daily requests by chart source",
                "图表来源逐日请求",
            ))
            .max_h(px(240.))
            .overflow_x_scroll()
            .overflow_y_scroll()
            .track_scroll(&self.exact_scroll)
            .font_features(theme::tabular_figures());
        let mut header = div().flex().min_w(px(640.)).py(px(4.));
        for (index, label) in [
            text(cx, "Date", "日期"),
            "Claude Code",
            "Codex",
            "Antigravity",
            "OpenCode",
            text(cx, "Chart total", "图中合计"),
        ]
        .into_iter()
        .enumerate()
        {
            header = header.child(
                div()
                    .flex_1()
                    .min_w_0()
                    .text_size(px(theme::META_SIZE))
                    .when(index > 0, |cell| cell.text_right())
                    .child(label),
            );
        }
        table = table.child(header);
        if report.days.is_empty() {
            table = table.child(div().text_color(cx.theme().muted_foreground).child(text(
                cx,
                "Daily breakdown unavailable",
                "逐日明细不可用",
            )));
        }
        // 最新在上；连续全零日折叠为一行。显示行与焦点句柄按同一顺序一一对应。
        for (index, row) in rows.iter().enumerate() {
            let Some((focus, _)) = self.exact_rows.get(index) else {
                break;
            };
            table = table.child(match *row {
                ExactRow::Day(day) => day_row(&report.days[day], focus, cx),
                ExactRow::Quiet { first, last } => quiet_row(
                    &report.days[first],
                    &report.days[last],
                    last - first + 1,
                    focus,
                    cx,
                ),
            });
        }
        table
    }
}

const SERIES: [&str; 4] = ["claude", "codex", "antigravity", "opencode"];

fn day_values(day: &WorkbenchDay) -> [i64; 4] {
    [day.claude, day.codex, day.antigravity, day.opencode]
}

pub(super) fn day_total(day: &WorkbenchDay) -> i64 {
    day_values(day).iter().sum()
}

/// First plotted day: one day before the first day with chart data, so a long
/// window does not open with an empty stretch. No data keeps the full window.
pub(super) fn trimmed_start(days: &[WorkbenchDay]) -> usize {
    days.iter()
        .position(|day| day_total(day) > 0)
        .map_or(0, |first| first.saturating_sub(1))
}

/// A displayed exact-value row; indices point into `WorkbenchReport::days`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ExactRow {
    Day(usize),
    /// 连续全零日（含单日），`first` 更早、`last` 更晚。
    Quiet {
        first: usize,
        last: usize,
    },
}

/// Newest-first rows with every run of all-zero days folded into one row.
pub(super) fn exact_rows(days: &[WorkbenchDay]) -> Vec<ExactRow> {
    let mut rows = Vec::new();
    let mut index = days.len();
    while index > 0 {
        index -= 1;
        if day_total(&days[index]) > 0 {
            rows.push(ExactRow::Day(index));
            continue;
        }
        let last = index;
        while index > 0 && day_total(&days[index - 1]) == 0 {
            index -= 1;
        }
        rows.push(ExactRow::Quiet { first: index, last });
    }
    rows
}

fn exact_row(id: SharedString, description: String, focus: &FocusHandle, cx: &App) -> Node {
    div()
        .id(id)
        .test_support()
        .role(gpui_kit::Role::Row)
        .aria_label(description.clone())
        .aria_value(description)
        .track_focus(focus)
        .tab_stop(true)
        .flex()
        .min_w(px(640.))
        .py(px(4.))
        .border_2()
        .border_color(gpui_kit::transparent_black())
        .focus(|style| style.border_color(cx.theme().ring))
}

fn day_row(day: &WorkbenchDay, focus: &FocusHandle, cx: &App) -> gpui_kit::AnyElement {
    let values = day_values(day);
    let total: i64 = values.iter().sum();
    let description = format!(
        "{}: Claude Code {}; Codex {}; Antigravity {}; OpenCode {}; {} {}",
        day.label,
        grouped(values[0]),
        grouped(values[1]),
        grouped(values[2]),
        grouped(values[3]),
        text(cx, "Chart total", "图中合计"),
        grouped(total),
    );
    let mut row = exact_row(
        SharedString::from(format!("overview-exact-{}", day.label)),
        description,
        focus,
        cx,
    );
    for (index, value) in std::iter::once(day.label.clone())
        .chain(values.into_iter().chain([total]).map(grouped))
        .enumerate()
    {
        row = row.child(
            div()
                .flex_1()
                .min_w_0()
                .when(index > 0, |cell| cell.text_right())
                .child(value),
        );
    }
    row.into_any_element()
}

fn quiet_row(
    first: &WorkbenchDay,
    last: &WorkbenchDay,
    count: usize,
    focus: &FocusHandle,
    cx: &App,
) -> gpui_kit::AnyElement {
    let description = match (count, is_chinese(cx)) {
        (1, true) => format!("{} · 无记录", first.label),
        (1, false) => format!("{} · no records", first.label),
        (_, true) => format!("{} – {} · {count} 日无记录", first.label, last.label),
        (_, false) => format!(
            "{} – {} · no records ({count} days)",
            first.label, last.label
        ),
    };
    exact_row(
        SharedString::from(format!("overview-exact-quiet-{}", first.label)),
        description.clone(),
        focus,
        cx,
    )
    .text_color(cx.theme().muted_foreground)
    .child(div().flex_1().min_w_0().child(description))
    .into_any_element()
}

fn bars(visible: &[WorkbenchDay], max: i64, cx: &App) -> gpui_kit::Div {
    let mut bars = div()
        .relative()
        .flex()
        .items_end()
        .h(px(theme::OVERVIEW_CHART_HEIGHT))
        .border_b_1()
        .border_color(cx.theme().border)
        .gap(px(if visible.len() > 30 { 1. } else { 3. }));
    for fraction in [0.25, 0.5, 0.75] {
        bars = bars.child(
            div()
                .absolute()
                .top(px(theme::OVERVIEW_CHART_HEIGHT * fraction))
                .w_full()
                .h(px(1.))
                .bg(cx.theme().border),
        );
    }
    for day in visible {
        bars = bars.child(day_stack(day, max.max(1), cx));
    }
    bars
}

/// Label of the last day with records when the window ends in empty days.
///
/// 只陈述事实：末尾的空白可能是没有用量，也可能是同步失败后尚未导入，这里不推断原因。
pub(super) fn latest_record(days: &[WorkbenchDay]) -> Option<&str> {
    let last = days.iter().rposition(|day| day_total(day) > 0)?;
    if last + 1 == days.len() {
        return None;
    }
    days.get(last).map(|day| day.label.as_str())
}

fn latest_suffix(date: &str, cx: &App) -> Node {
    let suffix = if is_chinese(cx) {
        format!("· 最新记录 {date}")
    } else {
        format!("· latest record {date}")
    };
    div()
        .id("overview-chart-latest")
        .test_support()
        .aria_label(suffix.clone())
        .text_size(px(theme::META_SIZE))
        .text_color(cx.theme().muted_foreground)
        .child(suffix)
}

// 与绘图区等宽日槽对齐；长窗口标注每段中点，使两端日期落在绘图区内。
fn ticks(visible: &[WorkbenchDay], cx: &App) -> Node {
    let mut ticks = div()
        .id("overview-chart-axis")
        .test_support()
        .flex()
        .gap(px(if visible.len() > 30 { 1. } else { 3. }))
        .text_size(px(theme::META_SIZE))
        .text_color(cx.theme().muted_foreground);
    let count = visible.len();
    let buckets = if count <= 7 { count } else { 6 };
    for (index, day) in visible.iter().enumerate() {
        let marked = (0..buckets).any(|i| (2 * i + 1) * count / (2 * buckets) == index);
        ticks = ticks.child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .justify_center()
                .when(marked, |slot| {
                    slot.child(div().flex_shrink_0().child(day.label.clone()))
                }),
        );
    }
    ticks
}

fn day_stack(day: &WorkbenchDay, max: i64, cx: &App) -> impl IntoElement {
    let column = div()
        .id(SharedString::from(format!("overview-day-{}", day.label)))
        .test_support()
        .flex_1()
        .min_w_0()
        .h(px(theme::OVERVIEW_CHART_HEIGHT))
        .flex()
        .flex_col()
        .justify_end();
    let values = day_values(day);
    let detail = format!(
        "{} · {} {}",
        day.label,
        grouped(values.iter().sum()),
        text(cx, "requests", "请求")
    );
    if values.iter().all(|value| *value == 0) {
        return column
            .child(
                div()
                    .w_full()
                    .border_b_1()
                    .border_dashed()
                    .border_color(cx.theme().border),
            )
            .tooltip(move |window, cx| Tooltip::new(detail.clone()).build(window, cx));
    }
    let mut column = column;
    let mut top = true;
    for (id, value) in SERIES.into_iter().zip(values).rev() {
        if value > 0 {
            let segment_detail = format!(
                "{} · {}: {} {}",
                day.label,
                platform_title(id),
                grouped(value),
                text(cx, "requests", "请求")
            );
            column = column.child(
                div()
                    .id(SharedString::from(format!(
                        "overview-day-{}-{id}",
                        day.label
                    )))
                    .w_full()
                    // 段间保留 1px 阅读面间隙，其下至少 1px 墨色；仅最上段做小圆角。
                    .h(px((value as f32 / max as f32
                        * theme::OVERVIEW_CHART_HEIGHT)
                        .max(2.)))
                    .border_t_1()
                    .border_color(cx.theme().background)
                    .when(top, |segment| {
                        segment.rounded_t(px(theme::CHART_BAR_RADIUS))
                    })
                    .bg(theme::chart_series(id, cx.theme().mode))
                    .tooltip(move |window, cx| {
                        Tooltip::new(segment_detail.clone()).build(window, cx)
                    }),
            );
            top = false;
        }
    }
    column.tooltip(move |window, cx| Tooltip::new(detail.clone()).build(window, cx))
}

fn legend(report: &WorkbenchReport, cx: &App) -> impl IntoElement {
    let mut row = div()
        .flex()
        .flex_wrap()
        .gap(px(theme::CONTROL_GAP))
        .text_size(px(theme::META_SIZE));
    for (index, id) in SERIES.into_iter().enumerate() {
        if report.days.iter().any(|d| day_values(d)[index] > 0) {
            row = row.child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(4.))
                    .child(
                        div()
                            .w(px(8.))
                            .h(px(8.))
                            .rounded(px(theme::CHART_BAR_RADIUS))
                            .bg(theme::chart_series(id, cx.theme().mode)),
                    )
                    .child(platform_title(id)),
            );
        }
    }
    row
}
