//! Below-the-fold insights: the request distribution and the leader boards.
use super::{Node, OverviewView, compact_count, grouped, panel, platform_title};
use crate::{
    locales::{is_chinese, text},
    shell::{badge, page_header, semantics},
    theme,
};
use ccr_app::usage::home::HomeInsightsSnapshot;
use gpui_kit::base::TestSupportExt;
use gpui_kit::component::{
    ActiveTheme,
    button::{Button, ButtonVariants},
    tooltip::Tooltip,
};
use gpui_kit::{
    App, Context, Div, Hsla, InteractiveElement, ParentElement, Role, SharedString,
    StatefulInteractiveElement, Styled, div, px, relative, rgb,
};

const LEADER_LIMIT: usize = 8;
/// 请求数列定宽，让各行比例条的右端对齐。
const LEADER_COUNT_WIDTH: f32 = 64.;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Distribution {
    Hourly,
    Weekday,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Leaders {
    Models,
    Projects,
    Agents,
}

impl OverviewView {
    pub(super) fn insights_row(
        &self,
        activity: &HomeInsightsSnapshot,
        stacked: bool,
        cx: &mut Context<Self>,
    ) -> Node {
        let distribution = self.distribution_panel(activity, cx);
        let leaders = self.leaders_panel(activity, cx);
        let row = div()
            .id("overview-insights-row")
            .test_support()
            .flex()
            .flex_shrink_0()
            .gap(px(theme::SECTION_GAP));
        if stacked {
            row.flex_col().child(distribution).child(leaders)
        } else {
            row.items_stretch()
                .child(distribution.flex_1())
                .child(leaders.flex_1())
        }
    }

    fn distribution_panel(&self, activity: &HomeInsightsSnapshot, cx: &mut Context<Self>) -> Node {
        let selected = self.distribution;
        let tabs = page_header::capsule(cx)
            .self_center()
            .child(scope_tab(
                "overview-dist-hourly",
                text(cx, "Hourly", "按小时"),
                selected == Distribution::Hourly,
                |this| this.distribution = Distribution::Hourly,
                cx,
            ))
            .child(scope_tab(
                "overview-dist-weekday",
                text(cx, "Weekday", "按星期"),
                selected == Distribution::Weekday,
                |this| this.distribution = Distribution::Weekday,
                cx,
            ));
        let frame = panel("overview-distribution", cx)
            .min_w_0()
            .child(panel_header(
                text(cx, "Request distribution", "请求分布"),
                None,
                tabs,
                cx,
            ));
        let (values, labels) = match selected {
            Distribution::Hourly => (&activity.hourly, hour_labels()),
            Distribution::Weekday => (&activity.weekday, weekday_labels(cx)),
        };
        match peak(values) {
            Some((index, value)) => {
                let label = labels.get(index).map_or("", String::as_str);
                let caption = peak_caption(label, value, cx);
                frame
                    .child(bars(values, &labels, index, value, cx))
                    .child(ticks(&labels, selected, cx))
                    .child(caption_line(caption, cx))
            }
            None => frame.child(empty_distribution(cx)),
        }
    }

    fn leaders_panel(&self, activity: &HomeInsightsSnapshot, cx: &mut Context<Self>) -> Node {
        let selected = self.leaders;
        let tabs = page_header::capsule(cx)
            .self_center()
            .child(scope_tab(
                "overview-leaders-models",
                text(cx, "Models", "模型"),
                selected == Leaders::Models,
                |this| this.leaders = Leaders::Models,
                cx,
            ))
            .child(scope_tab(
                "overview-leaders-projects",
                text(cx, "Projects", "项目"),
                selected == Leaders::Projects,
                |this| this.leaders = Leaders::Projects,
                cx,
            ))
            .child(scope_tab(
                "overview-leaders-agents",
                text(cx, "Agents", "Agent"),
                selected == Leaders::Agents,
                |this| this.leaders = Leaders::Agents,
                cx,
            ));
        let frame = panel("overview-leaders", cx).min_w_0().child(panel_header(
            text(cx, "Leaders", "排行"),
            Some(text(cx, "All history", "全历史")),
            tabs,
            cx,
        ));
        let rows = leader_rows(activity, selected);
        let Some(top) = rows.first().map(|row| row.requests) else {
            return frame.child(
                div()
                    .id("overview-leaders-empty")
                    .test_support()
                    .role(Role::Label)
                    .aria_label(text(cx, "No records", "暂无记录"))
                    .text_color(cx.theme().muted_foreground)
                    .child(text(cx, "No records", "暂无记录")),
            );
        };
        let mut list = div()
            .id("overview-leaders-list")
            .test_support()
            .role(Role::List)
            .flex()
            .flex_col();
        for (index, row) in rows.into_iter().enumerate() {
            list = list.child(leader_row(index, row, top, cx));
        }
        frame.child(list)
    }
}

// 选项构建放在独立函数中，避免增大 render 栈帧（Windows UI 线程只有 1 MiB）。
fn scope_tab(
    id: &'static str,
    label: &'static str,
    selected: bool,
    select: impl Fn(&mut OverviewView) + 'static,
    cx: &mut Context<OverviewView>,
) -> Button {
    semantics::tab(
        semantics::button(
            Button::new(id)
                .label(label)
                .ghost()
                .on_click(cx.listener(move |this, _, _, cx| {
                    select(this);
                    cx.notify();
                })),
            false,
            false,
            cx,
        ),
        selected,
        false,
        cx,
    )
}

fn panel_header(title: &'static str, scope: Option<&'static str>, tabs: Div, cx: &App) -> Div {
    div()
        .flex()
        .flex_wrap()
        .items_center()
        .justify_between()
        .gap(px(theme::CONTROL_GAP))
        .child(badge::section_heading(title, scope.map(Into::into), cx))
        .child(tabs)
}

fn hour_labels() -> Vec<String> {
    (0..24).map(|hour| format!("{hour:02}:00")).collect()
}

fn weekday_labels(cx: &App) -> Vec<String> {
    let names = if is_chinese(cx) {
        ["周一", "周二", "周三", "周四", "周五", "周六", "周日"]
    } else {
        ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]
    };
    names.map(String::from).to_vec()
}

/// The highest bucket; ties resolve to the earlier one. `None` when all are zero.
pub(super) fn peak(values: &[i64]) -> Option<(usize, i64)> {
    let mut best: Option<(usize, i64)> = None;
    for (index, value) in values.iter().copied().enumerate() {
        if value > 0 && best.is_none_or(|(_, top)| value > top) {
            best = Some((index, value));
        }
    }
    best
}

fn peak_caption(label: &str, value: i64, cx: &App) -> String {
    if is_chinese(cx) {
        format!("峰值 {label} · {} 次请求 · 全历史", grouped(value))
    } else {
        format!("Peak {label} · {} requests · All history", grouped(value))
    }
}

fn caption_line(caption: String, cx: &App) -> Node {
    div()
        .id("overview-dist-caption")
        .test_support()
        .role(Role::Label)
        .aria_label(caption.clone())
        .mt_auto()
        .text_size(px(theme::META_SIZE))
        .text_color(cx.theme().muted_foreground)
        .font_features(theme::tabular_figures())
        .child(caption)
}

fn bars(values: &[i64], labels: &[String], peak_index: usize, max: i64, cx: &App) -> Node {
    let mut plot = div()
        .id("overview-dist-bars")
        .test_support()
        .flex()
        .items_end()
        .gap(px(2.))
        .h(px(theme::DISTRIBUTION_HEIGHT))
        .border_b_1()
        .border_color(cx.theme().border);
    for (index, value) in values.iter().copied().enumerate() {
        let label = labels.get(index).map_or("", String::as_str);
        plot = plot.child(bar(index, label, value, max, index == peak_index, cx));
    }
    plot
}

// 整列都是悬停目标，矮柱也能读到 tooltip。
fn bar(index: usize, label: &str, value: i64, max: i64, is_peak: bool, cx: &App) -> Node {
    let detail: SharedString = format!(
        "{label} · {} {}",
        grouped(value),
        text(cx, "requests", "次请求")
    )
    .into();
    let column = div()
        .id(SharedString::from(format!("overview-dist-bar-{index}")))
        .test_support()
        .role(Role::Label)
        .aria_label(detail.clone())
        .flex_1()
        .min_w_0()
        .h_full()
        .flex()
        .flex_col()
        .justify_end()
        .tooltip(move |window, cx| Tooltip::new(detail.clone()).build(window, cx));
    if value <= 0 {
        return column;
    }
    let neutral = Hsla::from(rgb(theme::palette(cx.theme().mode).subtle));
    column.child(
        div()
            .w_full()
            .h(px(
                (value as f32 / max as f32 * theme::DISTRIBUTION_HEIGHT).max(2.)
            ))
            .rounded_t(px(theme::CHART_BAR_RADIUS))
            .bg(if is_peak { cx.theme().primary } else { neutral }),
    )
}

fn ticks(labels: &[String], selected: Distribution, cx: &App) -> Div {
    let mut row = div()
        .flex()
        .gap(px(2.))
        .text_size(px(theme::META_SIZE))
        .text_color(cx.theme().muted_foreground);
    for (index, label) in labels.iter().enumerate() {
        // 24 小时每 6 小时标一次，星期逐日标注。
        let marked = selected == Distribution::Weekday || index % 6 == 0;
        let slot = div().flex_1().min_w_0().flex().justify_center();
        row = row.child(if marked {
            slot.child(div().flex_shrink_0().child(label.clone()))
        } else {
            slot
        });
    }
    row
}

fn empty_distribution(cx: &App) -> Node {
    let message = text(
        cx,
        "No requests recorded · All history",
        "没有请求记录 · 全历史",
    );
    div()
        .id("overview-dist-empty")
        .test_support()
        .role(Role::Label)
        .aria_label(message)
        .h(px(theme::DISTRIBUTION_HEIGHT))
        .flex()
        .items_center()
        .justify_center()
        .border_b_1()
        .border_dashed()
        .border_color(cx.theme().border)
        .text_color(cx.theme().muted_foreground)
        .child(message)
}

/// One ranked row. `label` is what the row shows and `full` is the unshortened
/// name for its tooltip and accessible label.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct LeaderRow {
    pub(super) label: String,
    pub(super) full: String,
    pub(super) requests: i64,
}

/// Top rows for one board. Rows without requests (a session-only agent, or a
/// tokens-only model) are not ranked here instead of showing a zero.
pub(super) fn leader_rows(activity: &HomeInsightsSnapshot, board: Leaders) -> Vec<LeaderRow> {
    let rows: Vec<LeaderRow> = match board {
        Leaders::Models => activity
            .models
            .iter()
            .map(|row| LeaderRow {
                label: row.label.clone(),
                full: row.label.clone(),
                requests: row.requests,
            })
            .collect(),
        Leaders::Projects => activity
            .projects
            .iter()
            .map(|row| LeaderRow {
                label: path_leaf(&row.label).to_string(),
                full: row.label.clone(),
                requests: row.requests,
            })
            .collect(),
        Leaders::Agents => activity
            .agents
            .iter()
            .map(|row| {
                let title = platform_title(&row.source);
                LeaderRow {
                    label: title.clone(),
                    full: title,
                    requests: row.requests,
                }
            })
            .collect(),
    };
    rows.into_iter()
        .filter(|row| row.requests > 0)
        .take(LEADER_LIMIT)
        .collect()
}

/// Last path segment of a project label, for either separator.
pub(super) fn path_leaf(label: &str) -> &str {
    let trimmed = label.trim_end_matches(['/', '\\']);
    trimmed
        .rsplit(['/', '\\'])
        .next()
        .filter(|leaf| !leaf.is_empty())
        .unwrap_or(label)
}

fn leader_row(index: usize, row: LeaderRow, top: i64, cx: &App) -> Node {
    let detail: SharedString = format!(
        "{} · {} {}",
        row.full,
        grouped(row.requests),
        text(cx, "requests", "次请求")
    )
    .into();
    let count = if row.requests.abs() >= 10_000_000 {
        compact_count(row.requests)
    } else {
        grouped(row.requests)
    };
    let share = (row.requests as f32 / top.max(1) as f32).clamp(0.01, 1.);
    div()
        .id(SharedString::from(format!("overview-leader-{index}")))
        .test_support()
        .role(Role::ListItem)
        .aria_label(detail.clone())
        .tooltip(move |window, cx| Tooltip::new(detail.clone()).build(window, cx))
        .h(px(theme::OVERVIEW_LEADER_ROW_HEIGHT))
        .flex()
        .items_center()
        .gap(px(theme::CONTROL_GAP))
        .child(div().w(relative(0.4)).min_w_0().truncate().child(row.label))
        .child(ratio_bar(share, cx))
        .child(
            div()
                .flex_shrink_0()
                .w(px(LEADER_COUNT_WIDTH))
                .text_right()
                .font_features(theme::tabular_figures())
                .child(count),
        )
}

fn ratio_bar(share: f32, cx: &App) -> Div {
    div()
        .flex_1()
        .min_w_0()
        .h(px(6.))
        .rounded(px(3.))
        .bg(cx.theme().muted)
        .child(
            div()
                .h_full()
                .w(relative(share))
                .rounded(px(3.))
                .bg(cx.theme().primary),
        )
}
