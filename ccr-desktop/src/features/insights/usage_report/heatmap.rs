use super::*;
use ccr_usage::HeatmapPoint;
use chrono::{Datelike, NaiveDate};
use gpui_kit::component::tooltip::Tooltip;

const CELL: f32 = 14.;
const GAP: f32 = 3.;

pub(super) struct State {
    selected: Option<usize>,
    scroll: ScrollHandle,
    focus: FocusHandle,
}

impl State {
    pub(super) fn new(cx: &mut Context<InsightsView>) -> Self {
        Self {
            selected: None,
            scroll: ScrollHandle::new(),
            focus: cx.focus_handle(),
        }
    }

    pub(super) fn reset(&mut self) {
        self.selected = None;
        self.scroll.set_offset(gpui_kit::point(px(0.), px(0.)));
    }

    fn select(&mut self, index: usize, padding: usize, cx: &mut Context<InsightsView>) {
        self.selected = Some(index);
        self.scroll.scroll_to_item((index + padding) / 7);
        cx.notify();
    }
}

impl InsightsView {
    pub(super) fn usage_heatmap(
        &self,
        points: &[HeatmapPoint],
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        let first = points.first()?;
        let last = points.last()?;
        let start = NaiveDate::parse_from_str(&first.date, "%Y-%m-%d").ok()?;
        let zone = match self.submitted_usage.as_ref()?.timezone {
            ReportTimezone::Local => text(cx, "Local", "本地"),
            ReportTimezone::Utc => "UTC",
        };
        let range = format!(
            "{} · {} – {} · {zone}",
            text(cx, "Rolling 366 days", "滚动 366 个日历日"),
            first.date,
            last.date
        );
        let max = points
            .iter()
            .map(|point| point.event_count)
            .max()
            .unwrap_or(0);
        let title = text(cx, "Request activity", "请求活动");
        let panel = report_panel(cx)
            .child(heading(title, cx))
            .child(status_line("usage-heat-range", range, cx));
        Some(
            panel
                .child(self.heatmap_grid(points, start, max, cx))
                .child(heat_legend(max, cx))
                .children((max == 0).then(|| {
                    status_line(
                        "usage-heat-empty",
                        text(
                            cx,
                            "No matching requests in this window.",
                            "此滚动区间没有匹配的请求。",
                        ),
                        cx,
                    )
                }))
                .into_any_element(),
        )
    }

    fn heatmap_grid(
        &self,
        points: &[HeatmapPoint],
        start: NaiveDate,
        max: i64,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let state = &self.usage_report.heatmap;
        let padding = start.weekday().num_days_from_monday() as usize;
        let weeks = (points.len() + padding).div_ceil(7);
        let last = points.len() - 1;
        let selected = state.selected.unwrap_or(0).min(last);
        let mut columns = div()
            .id("usage-heat-weeks")
            .test_support()
            .flex_1()
            .min_w_0()
            .flex()
            .gap(px(GAP))
            .overflow_x_scroll()
            .restrict_scroll_to_axis()
            .track_scroll(&state.scroll);
        for week in 0..weeks {
            let slots = (week * 7..week * 7 + 7)
                .map(|slot| slot.checked_sub(padding).filter(|&index| index <= last));
            let month = slots.clone().flatten().find_map(|index| {
                let days = chrono::Days::new(index as u64);
                let date = start.checked_add_days(days)?;
                (date.day() == 1 || (index == 0 && date.day() <= 23)).then_some(date)
            });
            let mut column = div()
                .relative()
                .flex()
                .flex_col()
                .flex_shrink_0()
                .gap(px(GAP));
            column = column.child(
                div()
                    .h(px(CELL + 4.))
                    .w(px(CELL))
                    .children(month.map(|date| {
                        let label = if is_chinese(cx) {
                            format!("{}月", date.month())
                        } else {
                            date.format("%b").to_string()
                        };
                        div()
                            .absolute()
                            .top_0()
                            .left_0()
                            .min_w(px(32.))
                            .whitespace_nowrap()
                            .text_size(theme::text_size(theme::META_SIZE))
                            .text_color(cx.theme().muted_foreground)
                            .child(label)
                    })),
            );
            for index in slots {
                column = column.child(match index {
                    Some(index) => self.heatmap_cell(
                        &points[index],
                        index,
                        index == selected,
                        padding,
                        max,
                        cx,
                    ),
                    None => div().size(px(CELL)).into_any_element(),
                });
            }
            columns = columns.child(column);
        }
        let exact = exact_value(&points[selected], cx);
        div()
            .flex()
            .flex_col()
            .gap(px(theme::CONTROL_GAP))
            .child(
                div()
                    .id("usage-heatmap")
                    .test_support()
                    .role(Role::Group)
                    .aria_label(text(
                        cx,
                        "Daily requests. Up and down move by day. Left and right move by week.",
                        "每日请求。上下键按天移动，左右键按周移动。",
                    ))
                    .track_focus(&state.focus)
                    .tab_stop(true)
                    .border_2()
                    .border_color(gpui_kit::transparent_black())
                    .focus_visible(|style| style.border_color(cx.theme().ring))
                    .on_key_down(
                        cx.listener(move |this, event: &gpui_kit::KeyDownEvent, _, cx| {
                            let state = &mut this.usage_report.heatmap;
                            let current = state.selected.unwrap_or(0).min(last);
                            let next = match event.keystroke.key.as_str() {
                                "up" => current.saturating_sub(1),
                                "down" => (current + 1).min(last),
                                "left" => current.saturating_sub(7),
                                "right" => (current + 7).min(last),
                                "home" => 0,
                                "end" => last,
                                _ => return,
                            };
                            state.select(next, padding, cx);
                            cx.stop_propagation();
                        }),
                    )
                    .flex()
                    .child(weekdays(cx))
                    .child(columns),
            )
            .child(status_line("usage-heat-selected", exact, cx))
            .into_any_element()
    }

    fn heatmap_cell(
        &self,
        point: &HeatmapPoint,
        index: usize,
        selected: bool,
        padding: usize,
        max: i64,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let label: SharedString = exact_value(point, cx).into();
        let tooltip = label.clone();
        let weak = cx.weak_entity();
        div()
            .id(SharedString::from(format!("usage-heat-{}", point.date)))
            .test_support()
            .role(Role::Button)
            .aria_value(label.clone())
            .aria_label(label)
            .aria_selected(selected)
            .size(px(CELL))
            .flex_shrink_0()
            .rounded(px(theme::CHART_BAR_RADIUS))
            .bg(theme::heat_level(
                heat_level(point.event_count, max),
                cx.theme().mode,
            ))
            .border_2()
            .border_color(if selected {
                cx.theme().ring
            } else if point.event_count > 0 {
                cx.theme().primary
            } else {
                gpui_kit::transparent_black()
            })
            .tooltip(move |window, cx| Tooltip::new(tooltip.clone()).build(window, cx))
            .on_click(cx.listener(move |this, _, window, cx| {
                window.focus(&this.usage_report.heatmap.focus, cx);
                this.usage_report.heatmap.select(index, padding, cx);
            }))
            .on_a11y_action(gpui_kit::AccessibleAction::Click, move |_, window, cx| {
                let _ = weak.update(cx, |this, cx| {
                    window.focus(&this.usage_report.heatmap.focus, cx);
                    this.usage_report.heatmap.select(index, padding, cx);
                });
            })
            .into_any_element()
    }
}

fn exact_value(point: &HeatmapPoint, cx: &App) -> String {
    let unit = if point.event_count == 1 {
        "request"
    } else {
        "requests"
    };
    format!(
        "{} · {} {}",
        point.date,
        point.event_count,
        text(cx, unit, "次请求")
    )
}

fn weekdays(cx: &App) -> Div {
    let captions = if is_chinese(cx) {
        ["周一", "", "周三", "", "周五", "", ""]
    } else {
        ["Mon", "", "Wed", "", "Fri", "", ""]
    };
    div()
        .flex()
        .flex_col()
        .flex_shrink_0()
        .pt(px(CELL + 4. + GAP))
        .pr(px(theme::CONTROL_GAP))
        .gap(px(GAP))
        .text_size(theme::text_size(theme::META_SIZE))
        .text_color(cx.theme().muted_foreground)
        .children(
            captions
                .into_iter()
                .map(|name| div().h(px(CELL)).line_height(px(CELL)).child(name)),
        )
}

fn heat_level(count: i64, max: i64) -> u8 {
    if count <= 0 || max <= 0 {
        0
    } else {
        (4. * count as f64 / max as f64).ceil() as u8
    }
}

fn heat_legend(max: i64, cx: &App) -> AnyElement {
    let exact = format!("0 – {max} {}", text(cx, "requests per day", "次请求 / 天"));
    div()
        .id("usage-heat-legend")
        .test_support()
        .role(Role::Label)
        .aria_label(exact.clone())
        .aria_value(exact.clone())
        .flex()
        .items_center()
        .gap(px(GAP))
        .text_size(theme::text_size(theme::META_SIZE))
        .text_color(cx.theme().muted_foreground)
        .children((0..=4).map(|level| {
            div()
                .size(px(CELL))
                .rounded(px(theme::CHART_BAR_RADIUS))
                .bg(theme::heat_level(level, cx.theme().mode))
        }))
        .child(div().ml(px(theme::CONTROL_GAP)).child(exact))
        .into_any_element()
}
