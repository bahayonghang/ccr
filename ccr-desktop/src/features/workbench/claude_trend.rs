//! Calendar-spaced bars. Missing dates remain gaps; every observed bar is keyboard reachable.
use super::claude::format_exact_usd;
#[cfg(test)]
use super::claude::format_recorded_cost;
use super::*;
use chrono::{Duration, NaiveDate};
use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::{App, SharedString};

fn calendar(until: NaiveDate, rows: &[ccr_usage::DailyTrendDto]) -> Vec<(NaiveDate, Option<f64>)> {
    (0..30)
        .map(|index| {
            let date = until - Duration::days(29 - index);
            let value = rows
                .iter()
                .find(|row| row.date == date.to_string())
                .map(|row| row.cost_usd);
            (date, value)
        })
        .collect()
}
fn exact_fee(date: NaiveDate, value: f64, cx: &App) -> String {
    format!(
        "{date}: {} USD · {}",
        format_exact_usd(value),
        text(cx, "recorded API estimate", "API 价目估算记录")
    )
}
impl WorkbenchView {
    pub(super) fn claude_trend(&self, cx: &mut Context<Self>) -> gpui_kit::AnyElement {
        let mut section=div().id("workbench-claude-trend").test_support().flex_shrink_0().min_w_0().flex().flex_col().gap(px(8.))
            .p(px(16.)).bg(cx.theme().background).rounded(px(theme::PANEL_RADIUS)).border_1().border_color(theme::hairline(cx.theme().mode))
            .child(div().text_size(crate::theme::text_size(16.)).font_weight(gpui_kit::FontWeight::SEMIBOLD).child(text(cx,"Daily recorded cost · 30 days (USD)","每日记录费用 · 30 天（USD）")))
            .child(div().text_size(crate::theme::text_size(12.)).text_color(cx.theme().muted_foreground).child(text(cx,"Missing dates are gaps. Tab to a recorded day, then press Enter for its exact estimate in the panel; hover shows the same line.","缺失日留空。Tab 到有记录的日期后按 Enter，可在本面板读到该日精确估算；hover 显示同一行。")));
        if let Some(usage) = self.claude_usage() {
            let days = calendar(self.usage_until, &usage.recent.trends);
            let maximum = days
                .iter()
                .filter_map(|(_, v)| *v)
                .fold(0_f64, f64::max)
                .max(0.01);
            let mut bars = div()
                .id("workbench-claude-trend-plot")
                .test_support()
                .w_full()
                .h(px(216.))
                .flex()
                .items_end()
                .gap(px(3.));
            for (index, (date, value)) in days.iter().copied().enumerate() {
                let mut slot = div()
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .flex()
                    .flex_col()
                    .justify_end();
                if let Some(value) = value {
                    let description = exact_fee(date, value, cx);
                    let tip: SharedString = description.clone().into();
                    let bar = div()
                        .id(SharedString::from(format!("claude-cost-bar-{index}")))
                        .test_support()
                        .role(gpui_kit::Role::Button)
                        .aria_label(description)
                        .tab_stop(true)
                        .w_full()
                        .h(px((value / maximum * 180.) as f32).max(px(3.)))
                        .bg(theme::chart_series("claude", cx.theme().mode))
                        .border_2()
                        .border_color(gpui_kit::transparent_black())
                        .focus(|style| style.border_color(cx.theme().ring))
                        .tooltip(move |window, cx| Tooltip::new(tip.clone()).build(window, cx))
                        .on_key_down(cx.listener(
                            move |this, event: &gpui_kit::KeyDownEvent, _, cx| {
                                if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                                    this.trend_highlight = Some((date, value));
                                    this.exact_costs = true;
                                    cx.notify();
                                    cx.stop_propagation();
                                }
                            },
                        ))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.trend_highlight = Some((date, value));
                            this.exact_costs = true;
                            cx.notify();
                        }));
                    slot = slot.child(bar);
                }
                bars = bars.child(slot);
            }
            section = section.child(bars).child(
                div()
                    .flex()
                    .justify_between()
                    .text_size(crate::theme::text_size(12.))
                    .child(days[0].0.to_string())
                    .child(days[29].0.to_string()),
            );
            // 可见读数：键盘激活（或点击）某天后，日期、金额与口径同时出现在本行，不再只靠 tooltip。
            if let Some((date, value)) = self.trend_highlight {
                let description = exact_fee(date, value, cx);
                section = section.child(
                    div()
                        .id("workbench-claude-trend-readout")
                        .test_support()
                        .role(gpui_kit::Role::Status)
                        .aria_label(description.clone())
                        .text_size(crate::theme::text_size(12.))
                        .border_1()
                        .rounded(px(theme::PANEL_RADIUS))
                        .border_color(theme::hairline(cx.theme().mode))
                        .p(px(8.))
                        .child(description),
                );
            }
            section = section.child(semantics::ghost_button(
                Button::new("claude-cost-exact-toggle")
                    .label(text(
                        cx,
                        "Exact values (days and all models)",
                        "精确值（逐日及全部模型）",
                    ))
                    .ghost()
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.exact_costs = !this.exact_costs;
                        cx.notify();
                    })),
                false,
                false,
                cx,
            ));
            if self.exact_costs {
                let mut table = div()
                    .id("claude-cost-exact-table")
                    .test_support()
                    .max_h(px(240.))
                    .overflow_y_scroll()
                    .flex()
                    .flex_col()
                    .gap(px(4.))
                    .font_features(theme::tabular_figures());
                for (date, value) in days {
                    if let Some(value) = value {
                        let highlighted = self.trend_highlight.is_some_and(|(day, _)| day == date);
                        table = table.child(
                            div()
                                .text_size(crate::theme::text_size(12.))
                                .when(highlighted, |row| {
                                    row.bg(theme::chart_series("claude", cx.theme().mode))
                                        .text_color(cx.theme().background)
                                        .rounded(px(4.))
                                        .px(px(4.))
                                })
                                .child(exact_fee(date, value, cx)),
                        );
                    }
                }
                for (label, overview) in [
                    (text(cx, "Today", "今日"), &usage.today),
                    (text(cx, "Month to date", "本月截至今日"), &usage.month),
                    (
                        text(cx, "All imported history", "全部已导入历史"),
                        &usage.history,
                    ),
                ] {
                    table = table.child(div().text_size(crate::theme::text_size(12.)).child(
                        if overview.bucket_count == 0 {
                            format!(
                                "{label}: {}",
                                text(cx, "No imported records", "尚无已导入记录")
                            )
                        } else {
                            format!("{label}: {}", format_exact_usd(overview.total_cost_usd))
                        },
                    ));
                }
                section = section.child(table);
            }
            if usage.recent.trends.is_empty() {
                section = section.child(text(
                    cx,
                    "No imported records in this window",
                    "此窗口尚无已导入记录",
                ));
            }
        } else {
            section = section.child(text(
                cx,
                "Daily recorded costs unavailable",
                "逐日记录费用不可用",
            ));
        }
        section.into_any_element()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn calendar_preserves_missing_days_and_measured_zero() {
        let until = NaiveDate::from_ymd_opt(2026, 11, 10).unwrap_or_default();
        let row = ccr_usage::DailyTrendDto {
            date: until.to_string(),
            request_count: 1,
            total_tokens: 0,
            input_tokens: 0,
            output_tokens: 0,
            reasoning_output_tokens: 0,
            cache_read_tokens: 0,
            cache_creation_tokens: 0,
            cost_usd: 0.,
        };
        let days = calendar(until, &[row]);
        assert_eq!(days.len(), 30);
        assert_eq!(days[0].1, None);
        assert_eq!(days[29].1, Some(0.));
        assert_eq!(format_recorded_cost(0.004), "<$0.01");
    }
}
