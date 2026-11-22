use super::InsightsView;
use crate::{locales::text, shell::semantics, theme};
use ccr_app::usage::read::UsageSnapshot;
use ccr_usage::{DailyTrendDto, QueryFilter, RecordedCost};
use chrono::{Days, NaiveDate};
use gpui_kit::base::TestSupportExt;
use gpui_kit::component::{ActiveTheme, button::Button, tooltip::Tooltip};
use gpui_kit::{
    App, Bounds, Context, Div, FocusHandle, FontWeight, InteractiveElement, IntoElement,
    ParentElement, PathBuilder, Pixels, Role, ScrollHandle, SharedString,
    StatefulInteractiveElement, Styled, Subscription, Window, canvas, div, point,
    prelude::FluentBuilder, px, relative,
};

const DAYS_PER_PAGE: usize = 30;
const PLOT_HEIGHT: f32 = 184.;
const PLOT_PAD: f32 = 12.;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum Metric {
    #[default]
    Cost,
    Tokens,
    Requests,
}

impl Metric {
    const OPTIONS: [(Self, &'static str); 3] = [
        (Self::Cost, "usage-trend-metric-cost"),
        (Self::Tokens, "usage-trend-metric-tokens"),
        (Self::Requests, "usage-trend-metric-requests"),
    ];

    fn label(self, cx: &App) -> &'static str {
        match self {
            Self::Cost => text(cx, "Recorded cost", "记录费用"),
            Self::Tokens => text(cx, "Tokens", "Token"),
            Self::Requests => text(cx, "Requests", "请求"),
        }
    }

    fn unit(self, cx: &App) -> &'static str {
        match self {
            Self::Cost => "USD",
            Self::Tokens => text(cx, "tokens", "Token"),
            Self::Requests => text(cx, "requests", "请求"),
        }
    }

    fn read(self, day: &Day<'_>) -> Reading {
        let Some(trend) = day.trend else {
            return Reading::NoRecords;
        };
        match self {
            Self::Cost => Reading::Cost(day.cost),
            Self::Tokens => Reading::Count(trend.total_tokens),
            Self::Requests => Reading::Count(trend.request_count),
        }
    }
}

#[derive(Clone, Copy)]
enum Reading {
    NoRecords,
    Cost(RecordedCost),
    Count(i64),
}

impl Reading {
    fn coordinate(self) -> Option<f64> {
        match self {
            Self::NoRecords | Self::Cost(RecordedCost::Unavailable) => None,
            Self::Cost(
                RecordedCost::Known(value)
                | RecordedCost::Partial(value)
                | RecordedCost::Mixed(value),
            ) => value.is_finite().then_some(value),
            Self::Count(value) => Some(value as f64),
        }
    }

    fn exact(self, metric: Metric, cx: &App) -> String {
        let unavailable = text(cx, "Estimate unavailable", "估算不可用");
        let cost = |value: f64| {
            if value.is_finite() {
                format!("${value} USD")
            } else {
                unavailable.into()
            }
        };
        match self {
            Self::NoRecords => text(cx, "No records", "无记录").into(),
            Self::Cost(RecordedCost::Unavailable) => unavailable.into(),
            Self::Cost(RecordedCost::Known(value)) => cost(value),
            Self::Cost(RecordedCost::Partial(value)) => {
                format!(
                    "{} · {}",
                    cost(value),
                    text(cx, "Partial estimate", "部分记录可估算")
                )
            }
            Self::Cost(RecordedCost::Mixed(value)) => {
                format!(
                    "{} · {}",
                    cost(value),
                    text(cx, "Mixed pricing coverage", "估算覆盖范围待确认")
                )
            }
            Self::Count(value) => format!(
                "{} {}",
                crate::features::overview::grouped(value),
                metric.unit(cx)
            ),
        }
    }
}

struct Day<'a> {
    date: NaiveDate,
    trend: Option<&'a DailyTrendDto>,
    cost: RecordedCost,
}

struct CalendarPage<'a> {
    start: NaiveDate,
    end: NaiveDate,
    number: usize,
    count: usize,
    days: Vec<Day<'a>>,
}

impl<'a> CalendarPage<'a> {
    fn new(report: &'a UsageSnapshot, filter: &QueryFilter, page: usize) -> Option<Self> {
        let date = |row: &DailyTrendDto| row.date.parse::<NaiveDate>().ok();
        let start = filter
            .since
            .or_else(|| report.trends.first().and_then(date))?;
        let end = filter
            .until
            .or_else(|| report.trends.last().and_then(date))?;
        let span = usize::try_from((end - start).num_days() + 1).ok()?;
        if span == 0 {
            return None;
        }
        let count = span.div_ceil(DAYS_PER_PAGE);
        let number = page.min(count - 1);
        let end = end.checked_sub_days(Days::new((number * DAYS_PER_PAGE) as u64))?;
        let start = end
            .checked_sub_days(Days::new((DAYS_PER_PAGE - 1) as u64))
            .map_or(start, |page_start| page_start.max(start));
        let first = start.to_string();
        let index = report.trends.partition_point(|row| row.date < first);
        let mut rows = report.trends[index..].iter().peekable();
        let mut days = Vec::with_capacity(DAYS_PER_PAGE);
        for date in start.iter_days().take_while(|date| *date <= end) {
            let key = date.to_string();
            let trend = rows.next_if(|row| row.date == key);
            let cost = report
                .daily_recorded_cost
                .get(&key)
                .copied()
                .unwrap_or(RecordedCost::Unavailable);
            days.push(Day { date, trend, cost });
        }
        Some(Self {
            start,
            end,
            number,
            count,
            days,
        })
    }
}

pub(super) struct State {
    metric: Metric,
    page: usize,
    show_exact: bool,
    selected: usize,
    return_focus: Option<FocusHandle>,
    chart_focus: FocusHandle,
    rows: Vec<(FocusHandle, Subscription)>,
    _chart_focus_listener: Subscription,
    pub(super) page_scroll: ScrollHandle,
    header_bounds: ScrollHandle,
    exact_scroll: ScrollHandle,
    composition: [i128; 4],
}

impl State {
    pub(super) fn new(
        window: &mut Window,
        cx: &mut Context<InsightsView>,
        page_scroll: ScrollHandle,
    ) -> Self {
        let chart_focus = cx.focus_handle().tab_stop(false);
        let subscription = cx.on_focus_in(&chart_focus, window, |this, _, cx| {
            let trend = &this.usage_trend;
            reveal(&trend.page_scroll, trend.header_bounds.bounds());
            cx.notify();
        });
        Self {
            metric: Metric::default(),
            page: 0,
            show_exact: false,
            selected: 0,
            return_focus: None,
            chart_focus,
            rows: Vec::new(),
            _chart_focus_listener: subscription,
            page_scroll,
            header_bounds: ScrollHandle::new(),
            exact_scroll: ScrollHandle::new(),
            composition: [0; 4],
        }
    }

    fn prepare_rows(&mut self, window: &mut Window, cx: &mut Context<InsightsView>) {
        if !self.rows.is_empty() {
            return;
        }
        self.rows = (0..DAYS_PER_PAGE)
            .map(|index| {
                let focus = cx.focus_handle();
                let subscription = cx.on_focus(&focus, window, move |this, _, cx| {
                    let trend = &mut this.usage_trend;
                    trend.selected = index;
                    trend.reveal_row(index);
                    cx.notify();
                });
                (focus, subscription)
            })
            .collect();
    }

    pub(super) fn reset(&mut self) {
        self.metric = Metric::default();
        self.reset_range();
        self.page_scroll.set_offset(point(px(0.), px(0.)));
    }

    pub(super) fn reset_range(&mut self) {
        self.page = 0;
        self.show_exact = false;
        self.selected = 0;
        self.return_focus = None;
        self.exact_scroll.set_offset(point(px(0.), px(0.)));
    }

    pub(super) fn read_composition(&mut self, report: &UsageSnapshot) {
        self.composition = [0; 4];
        for day in &report.trends {
            // The compatible daily output field already includes reasoning.
            for (total, value) in self.composition.iter_mut().zip([
                day.input_tokens,
                day.output_tokens,
                day.cache_read_tokens,
                day.cache_creation_tokens,
            ]) {
                *total += i128::from(value);
            }
        }
    }

    fn go_to_page(&mut self, page: usize) {
        self.page = page;
        self.selected = 0;
        self.exact_scroll.scroll_to_top_of_item(0);
    }

    fn focus_row(&mut self, index: usize, window: &mut Window, cx: &mut Context<InsightsView>) {
        self.selected = index;
        self.reveal_row(index);
        window.focus(&self.rows[index].0, cx);
        cx.notify();
    }

    fn close_exact(&mut self, window: &mut Window, cx: &mut Context<InsightsView>) {
        self.show_exact = false;
        reveal(&self.page_scroll, self.header_bounds.bounds());
        let focus = self
            .return_focus
            .take()
            .unwrap_or_else(|| self.chart_focus.clone());
        window.focus(&focus, cx);
        cx.notify();
    }

    fn reveal_row(&self, index: usize) {
        self.exact_scroll.scroll_to_item(index);
        reveal(&self.page_scroll, self.exact_scroll.bounds());
    }
}

fn reveal(scroll: &ScrollHandle, bounds: Bounds<Pixels>) {
    let viewport = scroll.bounds();
    let mut offset = scroll.offset();
    if bounds.top() < viewport.top() {
        offset.y += viewport.top() - bounds.top();
    } else if bounds.bottom() > viewport.bottom() {
        offset.y += viewport.bottom() - bounds.bottom();
    } else {
        return;
    }
    scroll.set_offset(offset);
}

impl InsightsView {
    pub(super) fn usage_trend(
        &self,
        report: &UsageSnapshot,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let trend = &self.usage_trend;
        let page = self
            .submitted_usage
            .as_ref()
            .and_then(|filter| CalendarPage::new(report, filter, trend.page));
        let panel = section("usage-trend", cx);
        let Some(page) = page else {
            return panel
                .child(section_title(text(cx, "Daily trend", "逐日趋势")))
                .child(text(cx, "No daily breakdown available", "暂无逐日明细"));
        };
        let metric = trend.metric;
        let last_index = page.days.len() - 1;
        let reading = page
            .days
            .iter()
            .map(|day| metric.read(day))
            .collect::<Vec<_>>();
        let range = format!("{} – {}", page.start, page.end);
        let metrics = Metric::OPTIONS
            .into_iter()
            .map(|(option, id)| {
                semantics::tab(
                    Button::new(id)
                        .label(option.label(cx))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.usage_trend.metric = option;
                            cx.notify();
                        })),
                    metric == option,
                    false,
                    cx,
                )
            })
            .collect::<Vec<_>>();
        let next = page.number.saturating_sub(1);
        let previous = (page.number + 1).min(page.count - 1);
        let range_row = div()
            .flex()
            .items_center()
            .flex_wrap()
            .gap(px(theme::CONTROL_GAP))
            .child(label("usage-trend-range", range, cx))
            .when(page.count > 1, |row| {
                row.when(page.count > 2, |row| {
                    row.child(page_button(
                        "usage-trend-earliest",
                        text(cx, "Oldest dates", "最早日期"),
                        page.number,
                        page.count - 1,
                        cx,
                    ))
                })
                .child(page_button(
                    "usage-trend-previous",
                    text(cx, "Earlier dates", "较早日期"),
                    page.number,
                    previous,
                    cx,
                ))
                .child(page_button(
                    "usage-trend-next",
                    text(cx, "Later dates", "较晚日期"),
                    page.number,
                    next,
                    cx,
                ))
                .when(page.count > 2, |row| {
                    row.child(page_button(
                        "usage-trend-latest",
                        text(cx, "Newest dates", "最新日期"),
                        page.number,
                        0,
                        cx,
                    ))
                })
            });
        let controls = div()
            .flex()
            .items_center()
            .flex_wrap()
            .gap(px(theme::CONTROL_GAP))
            .child(section_title(text(cx, "Daily trend", "逐日趋势")))
            .child(div().flex().flex_wrap().gap(px(4.)).children(metrics))
            .child(semantics::disclosure(
                semantics::ghost_button(
                    Button::new("usage-trend-exact-toggle")
                        .label(if trend.show_exact {
                            text(cx, "Hide exact values", "收起精确值")
                        } else {
                            text(cx, "Show exact values", "查看精确值")
                        })
                        .on_click(cx.listener(
                            move |this, event: &gpui_kit::ClickEvent, window, cx| {
                                this.toggle_usage_exact(last_index, event, window, cx);
                            },
                        )),
                    false,
                    false,
                    cx,
                ),
                trend.show_exact,
            ));
        let header = div()
            .id("usage-trend-chart-focus")
            .test_support()
            .role(Role::Group)
            .aria_label(text(cx, "Daily trend controls", "逐日趋势控件"))
            .track_focus(&trend.chart_focus)
            .tab_stop(false)
            .track_scroll(&trend.header_bounds)
            .flex()
            .flex_col()
            .gap(px(4.))
            .border_2()
            .border_color(gpui_kit::transparent_black())
            .focus(|el| el.border_color(cx.theme().ring))
            .child(controls)
            .child(range_row);
        let table = trend
            .show_exact
            .then(|| self.usage_exact_table(&page, &reading, cx));
        panel
            .child(header)
            .child(plot(&page, &reading, metric, cx))
            .children(table)
    }

    fn toggle_usage_exact(
        &mut self,
        last_index: usize,
        event: &gpui_kit::ClickEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let trend = &mut self.usage_trend;
        if trend.show_exact {
            trend.close_exact(window, cx);
            return;
        }
        trend.show_exact = true;
        trend.prepare_rows(window, cx);
        trend.return_focus = event.is_keyboard().then(|| window.focused(cx)).flatten();
        let generation = self.generation;
        let view = cx.weak_entity();
        window.on_next_frame(move |window, cx| {
            let _ = view.update(cx, |this, cx| {
                if this.generation == generation && this.usage_trend.show_exact {
                    let trend = &mut this.usage_trend;
                    trend.focus_row(trend.selected.min(last_index), window, cx);
                }
            });
        });
        cx.notify();
    }

    fn usage_exact_table(
        &self,
        page: &CalendarPage<'_>,
        readings: &[Reading],
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let trend = &self.usage_trend;
        let metric = trend.metric;
        let table_label = text(cx, "Daily exact values", "逐日精确值");
        let header = div()
            .flex()
            .justify_between()
            .gap(px(theme::CONTROL_GAP))
            .pb(px(4.))
            .text_color(cx.theme().muted_foreground)
            .child(text(cx, "Date", "日期"))
            .child(format!("{} ({})", metric.label(cx), metric.unit(cx)));
        let count = page.days.len();
        div()
            .id("usage-trend-exact-table")
            .test_support()
            .role(Role::Table)
            .aria_label(table_label)
            .font_features(theme::tabular_figures())
            .child(header)
            .child(
                div()
                    .id("usage-trend-exact-scroll")
                    .test_support()
                    .max_h(px(192.))
                    .overflow_y_scroll()
                    .track_scroll(&trend.exact_scroll)
                    .children(page.days.iter().zip(readings).enumerate().map(
                        |(index, (day, reading))| {
                            let date = day.date.to_string();
                            let value = reading.exact(metric, cx);
                            let accessible_value = format!("{date} · {value}");
                            div()
                                .id(SharedString::from(format!("usage-trend-day-{date}")))
                                .test_support()
                                .role(Role::Row)
                                .aria_label(accessible_value.clone())
                                .aria_value(accessible_value)
                                .track_focus(&trend.rows[index].0)
                                .tab_stop(index == trend.selected.min(count - 1))
                                .flex()
                                .justify_between()
                                .items_center()
                                .gap(px(theme::CONTROL_GAP))
                                .px(px(4.))
                                .py(px(3.))
                                .border_2()
                                .border_color(gpui_kit::transparent_black())
                                .focus(|el| el.border_color(cx.theme().ring))
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    this.usage_trend.focus_row(index, window, cx);
                                }))
                                .on_key_down(cx.listener(
                                    move |this, event: &gpui_kit::KeyDownEvent, window, cx| {
                                        let trend = &mut this.usage_trend;
                                        let target = match event.keystroke.key.as_str() {
                                            "up" => index.saturating_sub(1),
                                            "down" => (index + 1).min(count - 1),
                                            "home" => 0,
                                            "end" => count - 1,
                                            "escape" => {
                                                trend.close_exact(window, cx);
                                                cx.stop_propagation();
                                                return;
                                            }
                                            _ => return,
                                        };
                                        cx.stop_propagation();
                                        trend.focus_row(target, window, cx);
                                    },
                                ))
                                .child(div().flex_shrink_0().child(date))
                                .child(div().text_right().min_w_0().child(value))
                        },
                    )),
            )
    }
}

fn page_button(
    id: &'static str,
    label: &'static str,
    current: usize,
    target: usize,
    cx: &mut Context<InsightsView>,
) -> Button {
    semantics::ghost_button(
        Button::new(id)
            .label(label)
            .on_click(cx.listener(move |this, _, window, cx| {
                let trend = &mut this.usage_trend;
                trend.go_to_page(target);
                if trend.rows.iter().any(|(focus, _)| focus.is_focused(window)) {
                    window.focus(&trend.chart_focus, cx);
                }
                cx.notify();
            })),
        current == target,
        false,
        cx,
    )
}

fn section(id: &'static str, cx: &App) -> gpui_kit::base::ObservedElement<gpui_kit::Stateful<Div>> {
    div()
        .id(id)
        .test_support()
        .flex()
        .flex_col()
        .flex_shrink_0()
        .min_w_0()
        .gap(px(theme::CONTROL_GAP))
        .p(px(theme::CONTROL_GAP))
        .border_t_1()
        .border_color(cx.theme().border)
}

fn section_title(label: &'static str) -> Div {
    div()
        .text_size(theme::text_size(theme::SECTION_TITLE_SIZE))
        .font_weight(FontWeight::SEMIBOLD)
        .child(label)
}

fn label(
    id: &'static str,
    value: String,
    cx: &App,
) -> gpui_kit::base::ObservedElement<gpui_kit::Stateful<Div>> {
    div()
        .id(id)
        .test_support()
        .role(Role::Label)
        .aria_label(value.clone())
        .aria_value(value.clone())
        .text_size(theme::text_size(theme::META_SIZE))
        .text_color(cx.theme().muted_foreground)
        .font_features(theme::tabular_figures())
        .child(value)
}

struct Axis {
    min: f64,
    max: f64,
    ticks: Vec<f64>,
    decimals: usize,
}

impl Axis {
    fn new(readings: &[Reading], metric: Metric) -> Self {
        let (min, max) = readings
            .iter()
            .filter_map(|reading| reading.coordinate())
            .fold((0.0f64, 0.0f64), |(min, max), value| {
                (min.min(value), max.max(value))
            });
        if min == max {
            return Self {
                min: 0.,
                max: 1.,
                ticks: vec![0.],
                decimals: 0,
            };
        }
        let ideal = (max - min) / 4.;
        let magnitude = 10.0f64.powf(ideal.log10().floor());
        let step = [1., 2., 5., 10.]
            .into_iter()
            .map(|digit| digit * magnitude)
            .find(|candidate| *candidate >= ideal)
            .unwrap_or(ideal);
        let step = match metric {
            Metric::Cost => step,
            Metric::Tokens | Metric::Requests => step.max(1.).ceil(),
        };
        let lower = (min / step).floor();
        let upper = (max / step).ceil();
        if step == 0. || !(lower * step).is_finite() || !(upper * step).is_finite() {
            return Self {
                min,
                max,
                ticks: vec![min, max],
                decimals: 2,
            };
        }
        let ticks = (lower as i32..=upper as i32)
            .map(|index| f64::from(index) * step)
            .collect();
        Self {
            min: lower * step,
            max: upper * step,
            ticks,
            decimals: (-step.log10().floor()).clamp(0., 6.) as usize,
        }
    }

    fn y(&self, value: f64) -> f32 {
        let fraction = if (self.max - self.min).is_finite() {
            (self.max - value) / (self.max - self.min)
        } else {
            (self.max / 2. - value / 2.) / (self.max / 2. - self.min / 2.)
        };
        PLOT_PAD + fraction.clamp(0., 1.) as f32 * (PLOT_HEIGHT - PLOT_PAD * 2.)
    }
}

fn tick_value(value: f64, metric: Metric, decimals: usize) -> String {
    match metric {
        Metric::Cost if value != 0. && !(0.001..1_000_000.).contains(&value.abs()) => {
            format!("${value:.1e}")
        }
        Metric::Cost => format!("${value:.decimals$}"),
        Metric::Tokens | Metric::Requests => crate::features::overview::compact_count(value as i64),
    }
}

fn plot(page: &CalendarPage<'_>, readings: &[Reading], metric: Metric, cx: &App) -> Div {
    let axis = Axis::new(readings, metric);
    let mut y_labels = div()
        .relative()
        .w(px(72.))
        .h(px(PLOT_HEIGHT))
        .flex_shrink_0();
    let mut grid = div().absolute().size_full();
    for tick in axis.ticks.iter().copied() {
        let y = axis.y(tick);
        y_labels = y_labels.child(
            div()
                .absolute()
                .right(px(7.))
                .top(px(y - 8.))
                .child(tick_value(tick, metric, axis.decimals)),
        );
        grid = grid.child(
            div()
                .absolute()
                .top(px(y))
                .w_full()
                .h(px(1.))
                .bg(cx.theme().border),
        );
    }
    let ys = readings
        .iter()
        .map(|reading| reading.coordinate().map(|value| axis.y(value)))
        .collect::<Vec<_>>();
    let ink = cx.theme().primary;
    let mut slots = div()
        .id("usage-trend-plot")
        .test_support()
        .role(Role::Group)
        .aria_label(format!("{} · {}", metric.label(cx), metric.unit(cx)))
        .relative()
        .h(px(PLOT_HEIGHT))
        .flex()
        .child(grid)
        .child(series_line(ys.clone(), ink));
    let count = page.days.len();
    let mut dates = div().flex().pl(px(72.));
    for (index, ((day, reading), y)) in page.days.iter().zip(readings).zip(ys).enumerate() {
        let date = day.date;
        let exact = reading.exact(metric, cx);
        let detail = format!("{date} · {exact}");
        slots = slots.child(
            div()
                .id(SharedString::from(format!("usage-trend-slot-{date}")))
                .test_support()
                .role(Role::Label)
                .aria_label(detail.clone())
                .aria_value(detail.clone())
                .flex_1()
                .min_w_0()
                .relative()
                .h_full()
                .hover(|style| style.bg(cx.theme().muted.opacity(0.35)))
                .tooltip(move |window, cx| Tooltip::new(detail.clone()).build(window, cx))
                .when_some(y, |slot, y| {
                    slot.child(
                        div()
                            .id(SharedString::from(format!("usage-trend-point-{date}")))
                            .test_support()
                            .absolute()
                            .left(relative(0.5))
                            .ml(px(-3.))
                            .top(px(y - 3.))
                            .size(px(6.))
                            .rounded(px(3.))
                            .bg(ink),
                    )
                }),
        );
        let marks = count.min(5);
        dates = dates.child(div().flex_1().min_w_0().flex().justify_center().when(
            (0..marks).any(|mark| (2 * mark + 1) * count / (2 * marks) == index),
            |slot| {
                slot.child(
                    div()
                        .flex_shrink_0()
                        .child(date.format("%m-%d").to_string()),
                )
            },
        ));
    }
    div()
        .flex()
        .flex_col()
        .text_size(theme::text_size(theme::META_SIZE))
        .text_color(cx.theme().muted_foreground)
        .font_features(theme::tabular_figures())
        .child(
            div()
                .flex()
                .justify_between()
                .gap(px(4.))
                .pl(px(7.))
                .child(metric.unit(cx))
                .children(
                    if readings
                        .iter()
                        .all(|reading| matches!(reading, Reading::NoRecords))
                    {
                        Some(text(cx, "No records in these dates", "这些日期没有记录"))
                    } else if readings
                        .iter()
                        .all(|reading| reading.coordinate().is_none())
                    {
                        Some(text(
                            cx,
                            "Estimate unavailable for these dates",
                            "这些日期的估算不可用",
                        ))
                    } else {
                        None
                    },
                ),
        )
        .child(
            div()
                .flex()
                .child(y_labels)
                .child(div().flex_1().min_w_0().child(slots)),
        )
        .child(dates)
}

fn series_line(ys: Vec<Option<f32>>, ink: gpui_kit::Hsla) -> gpui_kit::Canvas<()> {
    canvas(
        |_, _, _| (),
        move |bounds, (), window, _| {
            let spacing = bounds.size.width / ys.len() as f32;
            let position = |index: usize, y| {
                point(
                    bounds.left() + spacing * (index as f32 + 0.5),
                    bounds.top() + px(y),
                )
            };
            let mut line = PathBuilder::stroke(px(2.));
            for (index, pair) in ys.windows(2).enumerate() {
                if let [Some(from), Some(to)] = pair {
                    line.move_to(position(index, *from));
                    line.line_to(position(index + 1, *to));
                }
            }
            if let Ok(path) = line.build() {
                window.paint_path(path, ink);
            }
        },
    )
    .absolute()
    .size_full()
}

impl State {
    pub(super) fn token_composition(&self, narrow: bool, cx: &App) -> impl IntoElement + use<> {
        let [input, _, cache_read, _] = self.composition;
        let rate_label = text(cx, "Cache hit rate", "缓存命中率");
        let rate = cache_hit_rate(input, cache_read, cx);
        let accessible_rate = format!("{rate_label} · {rate}");
        let header = div()
            .flex()
            .items_center()
            .justify_between()
            .flex_wrap()
            .gap(px(theme::CONTROL_GAP))
            .child(section_title(text(cx, "Token composition", "Token 构成")))
            .child(
                div()
                    .id("usage-cache-hit-rate")
                    .test_support()
                    .role(Role::Label)
                    .aria_label(accessible_rate.clone())
                    .aria_value(accessible_rate)
                    .child(rate_label)
                    .child(div().font_weight(FontWeight::SEMIBOLD).child(rate))
                    .flex()
                    .gap(px(6.)),
            );
        let colors = ["codex", "opencode", "antigravity", "claude"]
            .map(|id| theme::chart_series(id, cx.theme().mode));
        let component_labels = [
            ("usage-token-input", text(cx, "Input", "输入")),
            ("usage-token-output", text(cx, "Output", "输出")),
            (
                "usage-token-cache-read",
                text(cx, "Cache reads", "缓存读取"),
            ),
            (
                "usage-token-cache-write",
                text(cx, "Cache writes", "缓存写入"),
            ),
        ];
        let total: i128 = self.composition.iter().map(|value| (*value).max(0)).sum();
        let strip = div()
            .h(px(9.))
            .flex()
            .gap(px(1.))
            .overflow_hidden()
            .rounded(px(theme::CHART_BAR_RADIUS))
            .bg(cx.theme().muted)
            .children(
                self.composition
                    .into_iter()
                    .zip(colors)
                    .filter(|(value, _)| *value > 0)
                    .map(|(value, color)| {
                        div()
                            .flex_basis(px(0.))
                            .flex_grow(value as f32 / total as f32)
                            .min_w_0()
                            .bg(color)
                    }),
            );
        let columns = if narrow { 2 } else { 4 };
        let mut legend = div().flex().flex_col().gap(px(4.));
        for indices in (0..4).step_by(columns) {
            legend = legend.child(div().flex().gap(px(theme::CONTROL_GAP)).children(
                (indices..indices + columns).map(|index| {
                    let (id, label) = component_labels[index];
                    let value = self.composition[index];
                    let exact = i64::try_from(value)
                        .map_or_else(|_| value.to_string(), crate::features::overview::grouped);
                    let accessible_value = format!("{label} · {exact}");
                    div()
                        .id(id)
                        .test_support()
                        .role(Role::Label)
                        .aria_label(accessible_value.clone())
                        .aria_value(accessible_value)
                        .flex_1()
                        .min_w_0()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .justify_between()
                        .gap(px(4.))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(5.))
                                .child(div().size(px(8.)).rounded(px(2.)).bg(colors[index]))
                                .child(label),
                        )
                        .child(
                            div()
                                .text_right()
                                .font_weight(FontWeight::SEMIBOLD)
                                .child(exact),
                        )
                }),
            ));
        }
        section("usage-token-composition", cx)
            .font_features(theme::tabular_figures())
            .child(header)
            .child(strip)
            .child(legend)
            .child(div()
                .text_size(theme::text_size(theme::META_SIZE))
                .text_color(cx.theme().muted_foreground)
                .child(text(cx,
                    "Output includes reasoning. Cache hit rate = cache reads ÷ (input + cache reads).",
                    "输出包含推理。缓存命中率 = 缓存读取 ÷（输入 + 缓存读取）。",
                )))
    }
}

fn cache_hit_rate(input: i128, cache_read: i128, cx: &App) -> String {
    let denominator = input + cache_read;
    if denominator == 0 {
        return text(cx, "Not calculable", "不可计算").into();
    }
    let percent = (cache_read as f64 / denominator as f64) * 100.;
    if cache_read > 0 && percent < 0.01 {
        "<0.01%".into()
    } else if input > 0 && percent > 99.99 {
        ">99.99%".into()
    } else {
        format!("{percent:.2}%")
    }
}
