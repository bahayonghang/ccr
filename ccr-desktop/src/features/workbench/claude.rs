use super::*;
use crate::features::overview::{format_usd, grouped};
use gpui_kit::App;

impl WorkbenchView {
    pub(super) fn claude_status(&self, cx: &mut Context<Self>) -> gpui_kit::AnyElement {
        let reading = self.profiles.is_none()
            || self.credential.is_none()
            || self.usage.is_none()
            || (self.observer.is_none() && self.observer_error.is_none());
        let mut section = div()
            .flex_shrink_0()
            .flex()
            .flex_col()
            .gap(px(theme::CONTROL_GAP))
            .bg(cx.theme().background)
            .rounded(px(theme::PANEL_RADIUS))
            .border_1()
            .border_color(cx.theme().border)
            .p(px(theme::SECTION_GAP))
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .justify_between()
                    .items_center()
                    .gap(px(theme::SECTION_GAP))
                    .child(
                        div()
                            .min_w_0()
                            .flex_1()
                            .flex()
                            .flex_wrap()
                            .items_center()
                            .gap(px(theme::SECTION_GAP))
                            .child(self.current_profile(cx))
                            .child(self.credential_status(cx)),
                    )
                    .child(semantics::button(
                        Button::new("workbench-refresh")
                            .ghost()
                            .label(if reading {
                                text(cx, "Refreshing…", "正在刷新…")
                            } else {
                                text(cx, "Refresh status", "刷新状态")
                            })
                            .on_click(
                                cx.listener(|this, _, _, cx| this.open(Platform::Claude, cx)),
                            ),
                        reading,
                        reading,
                        cx,
                    )),
            );
        if !self.facts.is_empty() {
            let mut facts = div().flex().flex_wrap().gap(px(theme::SECTION_GAP));
            for (key, value) in &self.facts {
                let label = match key.as_str() {
                    "model" => text(cx, "Model", "模型"),
                    "effortLevel" => text(cx, "Effort", "推理强度"),
                    "theme" => text(cx, "Theme", "主题"),
                    "language" => text(cx, "Language", "语言"),
                    "autoUpdatesChannel" => text(cx, "Update channel", "更新渠道"),
                    "permissionMode" => text(cx, "Permission mode", "权限模式"),
                    _ => key.as_str(),
                };
                facts = facts.child(
                    div()
                        .flex()
                        .flex_wrap()
                        .gap(px(theme::CONTROL_GAP))
                        .child(
                            div()
                                .text_color(cx.theme().muted_foreground)
                                .child(label.to_owned()),
                        )
                        .child(value.clone()),
                );
            }
            section = section.child(facts.text_size(px(theme::META_SIZE)));
        }
        section.into_any_element()
    }

    pub(super) fn claude_insights(&self, cx: &mut Context<Self>) -> gpui_kit::AnyElement {
        let title = text(cx, "Usage insights", "用量洞察");
        let mut section = div()
            .flex_shrink_0()
            .flex()
            .flex_col()
            .gap(px(theme::CONTROL_GAP))
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .justify_between()
                    .items_center()
                    .gap(px(theme::SECTION_GAP))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(theme::NAV_ROW_GAP))
                            .child(
                                div()
                                    .id("claude-usage-heading")
                                    .role(gpui_kit::Role::Heading)
                                    .aria_label(title)
                                    .aria_level(2)
                                    .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                                    .text_size(px(theme::SECTION_TITLE_SIZE))
                                    .child(title),
                            )
                            .child(
                                div()
                                    .text_size(px(theme::META_SIZE))
                                    .text_color(cx.theme().muted_foreground)
                                    .child(text(
                                        cx,
                                        "Claude costs recorded by llmusage · USD",
                                        "llmusage 记录的 Claude 费用 · USD",
                                    )),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .gap(px(theme::CONTROL_GAP))
                            .child(self.button(
                                "workbench-usage",
                                Link::Usage,
                                "Full usage report",
                                "完整用量报告",
                                false,
                                cx,
                            ))
                            .child(self.button(
                                "workbench-observer",
                                Link::Observer,
                                "Subscription settings",
                                "订阅设置",
                                false,
                                cx,
                            )),
                    ),
            )
            .child(self.claude_metrics(cx));

        let subscription = match self.observer.as_ref().map(|report| &report.subscription) {
            Some(Ok(Some(value))) => format!(
                "{} · {} / {} · ${:.2} {}",
                text(cx, "Subscription settings", "订阅设置"),
                value.mode,
                value.plan,
                value.monthly_usd,
                text(cx, "per month", "每月")
            ),
            Some(Ok(None)) => text(cx, "Subscription not configured", "未配置订阅").into(),
            Some(Err(_)) => text(cx, "Subscription settings unavailable", "订阅设置不可用").into(),
            None => self
                .observer_error
                .map(|error| service_error(cx, error))
                .unwrap_or_else(|| text(cx, "Reading subscription…", "正在读取订阅…"))
                .into(),
        };
        let mut summary = div()
            .flex()
            .flex_wrap()
            .items_center()
            .justify_between()
            .gap(px(theme::CONTROL_GAP))
            .text_size(px(theme::META_SIZE))
            .text_color(cx.theme().muted_foreground);
        let totals = match &self.usage {
            Some(Ok(snapshot)) => match snapshot.as_ref() {
                InsightsSnapshot::Usage(usage) if usage.overview.total_events > 0 => format!(
                    "{} · {} {} · {} tokens · {}",
                    text(cx, "Last 30 days", "最近 30 天"),
                    grouped(usage.overview.total_events),
                    text(cx, "requests", "请求"),
                    grouped(usage.overview.total.total_tokens),
                    format_usd(usage.overview.total_cost_usd)
                ),
                InsightsSnapshot::Usage(_) => text(
                    cx,
                    "No usage recorded in the last 30 days",
                    "最近 30 天无用量记录",
                )
                .into(),
                _ => text(cx, "30-day usage unavailable", "30 天用量不可用").into(),
            },
            Some(Err(_)) => text(cx, "30-day usage unavailable", "30 天用量不可用").into(),
            None => text(cx, "Reading 30-day usage…", "正在读取 30 天用量…").into(),
        };
        summary = summary
            .child(
                div()
                    .id("workbench-usage-totals")
                    .test_support()
                    .role(gpui_kit::Role::Label)
                    .aria_label(totals.clone())
                    .aria_value(totals.clone())
                    .child(totals),
            )
            .child(
                div()
                    .id("claude-subscription")
                    .test_support()
                    .role(gpui_kit::Role::Label)
                    .aria_label(subscription.clone())
                    .aria_value(subscription.clone())
                    .child(subscription),
            );
        section = section.child(summary);
        section.into_any_element()
    }

    fn claude_metrics(&self, cx: &App) -> gpui_kit::AnyElement {
        let insight = self
            .observer
            .as_ref()
            .and_then(|report| report.insight.as_ref().ok());
        let unavailable = if self.observer.is_none() && self.observer_error.is_none() {
            text(cx, "Reading…", "读取中…")
        } else {
            text(cx, "Unavailable", "不可用")
        };
        let mut band = div()
            .id("workbench-claude-insight")
            .test_support()
            .w_full()
            .flex()
            .flex_wrap()
            .border_1()
            .border_color(cx.theme().border)
            .rounded(px(theme::PANEL_RADIUS))
            .bg(cx.theme().background)
            .overflow_hidden();
        for (id, label, key, detail) in [
            (
                "claude-cost-today",
                text(cx, "Today", "今日"),
                "today_value_usd",
                "today_tokens",
            ),
            (
                "claude-cost-month",
                text(cx, "This month", "本月"),
                "month_value_usd",
                "month_tokens",
            ),
            (
                "claude-cost-total",
                text(cx, "All history", "全部历史"),
                "total_value_usd",
                "total_projects",
            ),
        ] {
            let amount = insight.and_then(|v| v.get(key)?.as_f64()).map(format_usd);
            let value = amount.as_deref().unwrap_or(unavailable);
            let unit = if detail == "total_projects" {
                text(cx, "projects", "个项目")
            } else {
                "tokens"
            };
            let detail = insight
                .and_then(|v| v.get(detail)?.as_i64())
                .map(|n| format!("{} {unit}", grouped(n)));
            let description = format!("{label} · {value}");
            band = band.child(
                div()
                    .id(id)
                    .test_support()
                    .role(gpui_kit::Role::Label)
                    .aria_label(description.clone())
                    .aria_value(description)
                    .flex_1()
                    .min_w(px(160.))
                    .flex()
                    .flex_col()
                    .gap(px(theme::NAV_ROW_GAP))
                    .p(px(theme::SECTION_GAP))
                    .when(key == "month_value_usd", |cell| cell.bg(cx.theme().sidebar))
                    .child(
                        div()
                            .text_size(px(theme::META_SIZE))
                            .text_color(cx.theme().muted_foreground)
                            .child(label),
                    )
                    .child(
                        div()
                            .text_size(px(if amount.is_some() {
                                theme::DISPLAY_SIZE
                            } else {
                                theme::BODY_SIZE
                            }))
                            .font_weight(gpui_kit::FontWeight::MEDIUM)
                            .when(amount.is_some(), |number| {
                                number.font_family(cx.theme().mono_font_family.clone())
                            })
                            .child(value.to_owned()),
                    )
                    .children(detail.map(|detail| {
                        div()
                            .text_size(px(theme::META_SIZE))
                            .text_color(cx.theme().muted_foreground)
                            .child(detail)
                    })),
            );
        }
        band.into_any_element()
    }
}
