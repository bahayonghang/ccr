//! KPI band: cost, requests, tokens and sessions in one ruled panel.
use super::{Node, OverviewView, compact_count, format_usd, grouped};
use crate::{
    locales::{is_chinese, text},
    runtime::insights::{SessionScope, WorkbenchReport},
    theme,
};
use gpui_kit::base::TestSupportExt;
use gpui_kit::component::{ActiveTheme, tooltip::Tooltip};
use gpui_kit::{
    App, Context, InteractiveElement, ParentElement, SharedString, StatefulInteractiveElement,
    Styled, div, px,
};

impl OverviewView {
    /// One panel, four columns separated by hairlines; cost is the primary reading.
    pub(super) fn kpis(
        &self,
        report: &WorkbenchReport,
        stacked: bool,
        cx: &mut Context<Self>,
    ) -> Node {
        let zh = is_chinese(cx);
        let days = f64::from(report.window_days.max(1));
        let span = if zh {
            format!("{} 日", report.window_days)
        } else {
            format!("{} days", report.window_days)
        };
        // 派生值（日均 = 总量 ÷ 窗口天数）必须带“日均 / avg”标签。
        let average = |value: String| {
            if zh {
                format!("{span} · 日均 {value}")
            } else {
                format!("{span} · avg {value}/day")
            }
        };
        let cost = match report.cost_usd {
            Some(value) => kpi_cell(
                text(cx, "Cost", "成本"),
                reading(
                    "total-cost",
                    format!("{}: USD {value}", text(cx, "Cost", "成本")),
                    format_usd(value),
                    theme::KPI_PRIMARY_SIZE,
                    gpui_kit::FontWeight::SEMIBOLD,
                ),
                // 日均只是辅助说明：≥$1 时两位小数即可，小额仍保留精度。
                average({
                    let daily = value / days;
                    if daily.abs() >= 1.0 {
                        format!("${daily:.2}")
                    } else {
                        format_usd(daily)
                    }
                }),
                cx,
            ),
            None => kpi_cell(
                text(cx, "Cost", "成本"),
                reading(
                    "total-cost",
                    format!(
                        "{}: {}",
                        text(cx, "Cost", "成本"),
                        text(cx, "Cost unavailable", "成本不可用")
                    ),
                    "–".into(),
                    theme::KPI_PRIMARY_SIZE,
                    gpui_kit::FontWeight::SEMIBOLD,
                ),
                text(cx, "Cost unavailable", "成本不可用").into(),
                cx,
            ),
        };
        let requests = kpi_cell(
            text(cx, "Requests", "请求"),
            count_reading(
                "total-requests",
                text(cx, "Requests", "请求"),
                report.requests,
            ),
            average(format_average(report.requests as f64 / days)),
            cx,
        );
        let tokens = kpi_cell(
            text(cx, "Tokens", "Token"),
            count_reading("total-tokens", text(cx, "Tokens", "Token"), report.tokens),
            span.clone(),
            cx,
        );
        let sessions = self.session_cell(report, &span, cx);
        let mut panel = div()
            .id("overview-kpis")
            .test_support()
            .flex_shrink_0()
            .flex()
            .bg(cx.theme().background)
            .border_1()
            .border_color(cx.theme().border)
            .rounded(px(theme::PANEL_RADIUS));
        let hairline = cx.theme().border;
        if stacked {
            // 窄宽 2×2：两行，每行第二格左侧 hairline，第二行顶部 hairline。
            panel = panel
                .flex_col()
                .child(
                    div()
                        .flex()
                        .child(cost)
                        .child(requests.border_l_1().border_color(hairline)),
                )
                .child(
                    div()
                        .flex()
                        .border_t_1()
                        .border_color(hairline)
                        .child(tokens)
                        .child(sessions.border_l_1().border_color(hairline)),
                );
        } else {
            panel = panel
                .child(cost)
                .child(requests.border_l_1().border_color(hairline))
                .child(tokens.border_l_1().border_color(hairline))
                .child(sessions.border_l_1().border_color(hairline));
        }
        panel
    }

    fn session_cell(&self, report: &WorkbenchReport, span: &str, cx: &App) -> gpui_kit::Div {
        let title = match report.session_scope {
            SessionScope::Window if is_chinese(cx) => format!("会话 · {} 日", report.window_days),
            SessionScope::Window => format!("Sessions · {}D", report.window_days),
            SessionScope::AllHistory => text(cx, "Sessions · all time", "会话 · 全历史").into(),
            SessionScope::Unindexed | SessionScope::Unavailable => {
                text(cx, "Sessions", "会话").into()
            }
        };
        let (short, exact, size, note): (String, String, f32, String) =
            match (report.session_scope, report.session_count) {
                (SessionScope::Window, Some(count)) => (
                    compact_count(count),
                    grouped(count),
                    theme::KPI_SIZE,
                    span.into(),
                ),
                (SessionScope::AllHistory, Some(count)) => (
                    compact_count(count),
                    grouped(count),
                    theme::KPI_SIZE,
                    text(cx, "all time", "全历史").into(),
                ),
                (SessionScope::Unindexed, _) => (
                    text(cx, "Unindexed", "未索引").into(),
                    text(cx, "Archive not indexed", "归档未索引").into(),
                    theme::KPI_STATE_SIZE,
                    text(cx, "Archive not indexed", "归档未索引").into(),
                ),
                _ => (
                    text(cx, "Unavailable", "不可用").into(),
                    text(cx, "Session count unavailable", "会话数不可用").into(),
                    theme::KPI_STATE_SIZE,
                    text(cx, "Session count unavailable", "会话数不可用").into(),
                ),
            };
        kpi_cell(
            text(cx, "Sessions", "会话"),
            reading(
                "total-sessions",
                format!("{title}: {exact}"),
                short,
                size,
                gpui_kit::FontWeight::MEDIUM,
            ),
            note,
            cx,
        )
    }
}

/// A KPI column: 12px muted label, the reading, and a 12px note from real data.
fn kpi_cell(label: &'static str, value: Node, note: String, cx: &App) -> gpui_kit::Div {
    div()
        .flex_1()
        .min_w_0()
        .flex()
        .flex_col()
        .gap(px(4.))
        .p(px(theme::SECTION_GAP))
        .child(
            div()
                .text_size(crate::theme::text_size(theme::META_SIZE))
                .text_color(cx.theme().muted_foreground)
                .child(label),
        )
        .child(value)
        .child(
            div()
                .text_size(crate::theme::text_size(theme::META_SIZE))
                .text_color(cx.theme().muted_foreground)
                .child(note),
        )
}

fn count_reading(id: &'static str, label: &'static str, value: i64) -> Node {
    reading(
        id,
        format!("{label}: {}", grouped(value)),
        compact_count(value),
        theme::KPI_SIZE,
        gpui_kit::FontWeight::MEDIUM,
    )
}

/// A labelled reading whose exact value stays in its accessible value and tooltip.
fn reading(
    id: &str,
    description: String,
    short: String,
    size: f32,
    weight: gpui_kit::FontWeight,
) -> Node {
    div()
        .id(SharedString::from(format!("overview-{id}")))
        .test_support()
        .role(gpui_kit::Role::Label)
        .aria_label(description.clone())
        .aria_value(description.clone())
        .font_features(theme::tabular_figures())
        .font_weight(weight)
        .text_size(theme::text_size(size))
        .line_height(gpui_kit::relative(1.2))
        .child(short)
        .tooltip(move |window, cx| Tooltip::new(description.clone()).build(window, cx))
}

/// Labelled derived averages: compact from 1K, whole numbers from 10, one decimal below.
pub(super) fn format_average(value: f64) -> String {
    if value >= 1_000. {
        compact_count(value.round() as i64)
    } else if value >= 10. {
        format!("{value:.0}")
    } else {
        format!("{value:.1}")
    }
}
