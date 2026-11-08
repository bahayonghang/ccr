//! Platform band: a sparkline cell per platform with daily records, one cell for the rest.
use super::{Node, OverviewView, compact_count, grouped, platform_title};
use crate::{
    locales::{is_chinese, text},
    runtime::insights::{WorkbenchPlatform, WorkbenchReport},
    theme,
};
use gpui_kit::base::TestSupportExt;
use gpui_kit::component::{ActiveTheme, tooltip::Tooltip};
use gpui_kit::{
    App, Context, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, div, prelude::FluentBuilder, px,
};

impl OverviewView {
    /// Platforms with daily records get a sparkline cell; the rest share one
    /// compact "Other sources" cell with their reason (never zero-filled).
    pub(super) fn platforms(
        &self,
        report: &WorkbenchReport,
        cx: &mut Context<Self>,
    ) -> Option<gpui_kit::AnyElement> {
        let (recorded, quiet): (Vec<&WorkbenchPlatform>, Vec<&WorkbenchPlatform>) = report
            .platforms
            .iter()
            .partition(|platform| platform.requests_by_day.iter().any(|value| *value > 0));
        if recorded.is_empty() && quiet.is_empty() {
            return None;
        }
        let mut band = div()
            .id("overview-platform-band")
            .test_support()
            .w_full()
            .flex_shrink_0()
            .flex()
            .bg(cx.theme().background)
            .rounded(px(theme::PANEL_RADIUS))
            .border_1()
            .border_color(cx.theme().border)
            .overflow_x_scroll();
        for (index, platform) in recorded.iter().enumerate() {
            band = band.child(platform_cell(platform, index > 0, cx));
        }
        if !quiet.is_empty() {
            band = band.child(other_sources(&quiet, !recorded.is_empty(), cx));
        }
        Some(band.into_any_element())
    }
}

/// Compact label/value row used inside platform cells.
fn inline_metric(id: String, label: &'static str, value: i64, cx: &App) -> gpui_kit::Div {
    let description = format!("{label}: {}", grouped(value));
    div()
        .min_w_0()
        .flex()
        .items_center()
        .justify_between()
        .gap(px(theme::CONTROL_GAP))
        .child(
            div()
                .text_size(px(theme::META_SIZE))
                .text_color(cx.theme().muted_foreground)
                .child(label),
        )
        .child(
            div()
                .id(SharedString::from(format!("overview-{id}")))
                .test_support()
                .role(gpui_kit::Role::Label)
                .aria_label(description.clone())
                .aria_value(description.clone())
                .font_features(theme::tabular_figures())
                .font_weight(gpui_kit::FontWeight::MEDIUM)
                .text_right()
                .child(compact_count(value))
                .tooltip(move |window, cx| Tooltip::new(description.clone()).build(window, cx)),
        )
}

fn platform_cell(platform: &WorkbenchPlatform, divided: bool, cx: &App) -> gpui_kit::Div {
    let ink = theme::platform_color(&platform.id, cx.theme().mode);
    div()
        .relative()
        .min_w(px(theme::SPARK_CELL_MIN_WIDTH))
        .flex_1()
        .flex_shrink_0()
        .flex()
        .flex_col()
        .gap(px(theme::CONTROL_GAP))
        .p(px(theme::SECTION_GAP))
        .when(divided, |cell| {
            cell.border_l_1().border_color(cx.theme().border)
        })
        .child(
            div()
                .absolute()
                .top_0()
                .left_0()
                .right_0()
                .h(px(2.))
                .bg(ink),
        )
        .child(
            div()
                .text_size(px(theme::META_SIZE))
                .text_color(cx.theme().muted_foreground)
                .child(platform_title(&platform.id)),
        )
        .child(sparkline(&platform.requests_by_day, ink, cx))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(theme::CONTROL_GAP))
                .child(inline_metric(
                    format!("{}-requests", platform.id),
                    text(cx, "Requests", "请求"),
                    platform.requests,
                    cx,
                ))
                .child(inline_metric(
                    format!("{}-tokens", platform.id),
                    text(cx, "Tokens", "Token"),
                    platform.tokens,
                    cx,
                )),
        )
}

fn other_sources(quiet: &[&WorkbenchPlatform], divided: bool, cx: &App) -> Node {
    let zh = is_chinese(cx);
    let mut cell = div()
        .id("overview-other-sources")
        .test_support()
        .min_w(px(theme::SPARK_CELL_MIN_WIDTH))
        .flex_1()
        .flex_shrink_0()
        .flex()
        .flex_col()
        .gap(px(4.))
        .p(px(theme::SECTION_GAP))
        .text_size(px(theme::META_SIZE))
        .when(divided, |cell| {
            cell.border_l_1().border_color(cx.theme().border)
        })
        .child(div().text_color(cx.theme().muted_foreground).child(text(
            cx,
            "Other sources",
            "其他来源",
        )));
    for platform in quiet {
        let name = platform_title(&platform.id);
        // 有总数但无逐日明细时保留真实总数；无记录时只写原因，不铺 0。
        let description = match (platform.requests > 0, zh) {
            (true, true) => format!(
                "{name} · {} 次请求 · 无逐日明细",
                grouped(platform.requests)
            ),
            (true, false) => format!(
                "{name} · {} requests · daily split unavailable",
                grouped(platform.requests)
            ),
            (false, true) => format!("{name} · 无请求"),
            (false, false) => format!("{name} · no requests"),
        };
        cell = cell.child(
            div()
                .id(SharedString::from(format!(
                    "overview-other-{}",
                    platform.id
                )))
                .test_support()
                .aria_label(description.clone())
                .text_color(cx.theme().secondary_foreground)
                .child(description),
        );
    }
    cell
}

fn sparkline(values: &[i64], ink: gpui_kit::Hsla, cx: &App) -> gpui_kit::Div {
    let max = values.iter().copied().max().unwrap_or(0);
    let row = div()
        .h(px(theme::SPARK_HEIGHT))
        .flex()
        .min_w_0()
        .items_end()
        .border_b_1()
        .border_color(cx.theme().border);
    if max == 0 {
        // 只有有逐日记录的平台才单独成格；此分支仅作防御，绘制虚线空基线。
        return row.border_dashed();
    }
    let subtle = gpui_kit::Hsla::from(gpui_kit::rgb(theme::palette(cx.theme().mode).subtle));
    let mut row = row.gap(px(if values.len() > 30 { 0. } else { 1. }));
    for value in values {
        let height = if *value == 0 {
            0.
        } else {
            (*value as f32 / max as f32 * (theme::SPARK_HEIGHT - 2.)).max(1.)
        };
        row = row.child(div().flex_1().min_w_0().h(px(height)).bg(if *value == max {
            ink
        } else {
            subtle
        }));
    }
    row
}
