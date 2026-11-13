use super::*;
use crate::features::overview::grouped;
use crate::runtime::insights::{
    claude_workbench::{Report, Usage, cache_ratio},
    observer::{Error, Update},
};
use ccr_app::usage::{observer::subscription::SubscriptionDto, report_clock::ReportClock};
use gpui_kit::component::{
    IconName,
    input::{Input, InputState},
};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::{App, AppContext, Div};

pub(super) fn format_recorded_cost(value: f64) -> String {
    if value != 0. && value.abs() < 0.01 {
        return if value < 0. {
            ">−$0.01".into()
        } else {
            "<$0.01".into()
        };
    }
    format!("${value:.2}")
}

/// 精确值表用固定六位小数，去掉二进制浮点尾巴又不丢可核对精度。
pub(super) fn format_exact_usd(value: f64) -> String {
    let rounded = (value * 1_000_000.).round() / 1_000_000.;
    format!("${rounded}")
}

fn subscription_plan<'a>(plan: &'a str, cx: &App) -> &'a str {
    match plan {
        "free_pro" => "Free / Pro",
        "max5x" => "Max 5×",
        "max20x" => "Max 20×",
        "team" => "Team",
        "enterprise" => "Enterprise",
        "custom" => text(cx, "Custom", "自定义"),
        unknown => unknown,
    }
}

fn panel(cx: &App) -> Div {
    div()
        .flex_shrink_0()
        .min_w_0()
        .flex()
        .flex_col()
        .gap(px(8.))
        .p(px(16.))
        .rounded(px(theme::PANEL_RADIUS))
        .bg(cx.theme().background)
        .border_1()
        .border_color(theme::hairline(cx.theme().mode))
}
fn heading(label: &'static str) -> Div {
    div()
        .text_size(px(theme::SECTION_TITLE_SIZE))
        .font_weight(gpui_kit::FontWeight::SEMIBOLD)
        .child(label)
}

impl WorkbenchView {
    pub(super) fn load_claude(&mut self, clock: ReportClock, cx: &mut Context<Self>) {
        let generation = self.generation;
        let key = RequestKey {
            environment: EnvironmentId::Local("claude-workbench".into()),
            platform: Platform::Claude,
            generation,
            request_id: generation,
        };
        match self.requests.claude_workbench(key, clock) {
            Ok(mut request) => {
                self.claude_task = Some(cx.spawn(async move |weak, cx| {
                    while let Some(event) = request.next().await {
                        if let TaskEvent::Terminal(result) = event {
                            let _ = weak.update(cx, |this, cx| {
                                if this.generation != generation
                                    || this.platform != Platform::Claude
                                {
                                    return;
                                }
                                match result.as_ref() {
                                    Completion::Success(report) => {
                                        this.subscription_state = Some(report.subscription.clone());
                                        this.claude_report = Some(Ok(report.clone()));
                                    }
                                    _ => {
                                        this.claude_report = Some(Err(
                                            "Claude report unavailable / Claude 报告不可用",
                                        ))
                                    }
                                }
                                cx.notify();
                            });
                            break;
                        }
                    }
                }))
            }
            Err(_) => self.claude_report = Some(Err("Read queue unavailable / 读取队列不可用")),
        }
    }
    fn claude_report(&self) -> Option<&Report> {
        self.claude_report
            .as_ref()?
            .as_ref()
            .ok()
            .map(AsRef::as_ref)
    }
    pub(super) fn claude_usage(&self) -> Option<&Usage> {
        self.claude_report()?.usage.as_ref().ok()
    }

    pub(super) fn claude_status(&self, cx: &mut Context<Self>) -> gpui_kit::AnyElement {
        let current = self
            .profiles
            .as_ref()
            .and_then(|rows| rows.as_ref().ok())
            .and_then(|rows| rows.iter().find(|row| row.is_current));
        let mut facts = panel(cx).child(heading(text(cx, "Access facts", "接入状态")));
        if let Some(row) = current {
            for (index, (label, value)) in [
                (
                    text(cx, "Current runtime profile", "当前运行方案"),
                    row.name.clone(),
                ),
                (
                    text(cx, "Endpoint host", "端点主机"),
                    safe_host(&row.base_url).unwrap_or_else(|| text(cx, "Unknown", "未知").into()),
                ),
                (
                    text(cx, "Provider", "供应商"),
                    row.provider
                        .clone()
                        .unwrap_or_else(|| text(cx, "Unknown", "未知").into()),
                ),
                (
                    text(cx, "Model", "模型"),
                    row.model
                        .clone()
                        .unwrap_or_else(|| text(cx, "Not observed", "未观测").into()),
                ),
                (
                    text(cx, "Authentication method", "认证方式"),
                    row.auth_mode
                        .clone()
                        .unwrap_or_else(|| text(cx, "Not observed", "未观测").into()),
                ),
            ]
            .into_iter()
            .enumerate()
            {
                facts = facts.child(
                    div()
                        .id(if index == 0 {
                            "workbench-current-profile".to_string()
                        } else {
                            format!("workbench-access-fact-{index}")
                        })
                        .test_support()
                        .aria_label(format!("{label}: {value}"))
                        .text_size(px(12.))
                        .child(format!("{label}: {value}")),
                );
            }
        } else {
            facts = facts.child(self.current_profile(cx));
        }
        facts.child(self.credential_status(cx))
            .child(div().text_size(px(12.)).text_color(cx.theme().muted_foreground).child(text(cx,"Local configuration and credential observations do not verify network access. API access does not require a saved official account.","本地配置与凭据观测不证明网络可用；API 接入不要求保存官方账户。")))
            .child(self.claude_refresh(cx)).into_any_element()
    }

    // 读取在途时保持禁用与忙碌语义，并用文字陈述正在刷新。
    fn claude_refresh(&self, cx: &mut Context<Self>) -> Button {
        let reading = self.claude_report.is_none();
        semantics::ghost_button(
            Button::new("workbench-refresh")
                .icon(IconName::RefreshCw)
                .label(if reading {
                    text(cx, "Refreshing…", "正在刷新…")
                } else {
                    text(cx, "Refresh status", "刷新状态")
                })
                .ghost()
                .on_click(cx.listener(|this, _, _, cx| this.open(Platform::Claude, cx))),
            reading,
            reading,
            cx,
        )
    }

    pub(super) fn claude_insights(
        &self,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> gpui_kit::AnyElement {
        let report = self.claude_report();
        let usage = self.claude_usage();
        let clock = report.map(|report| report.clock);
        let failure = match &self.claude_report {
            Some(Err(message)) => Some(*message),
            _ => None,
        };
        let context = clock
            .map(|clock| {
                format!(
                    "llmusage · {} · USD · {}: {} · {}: {}",
                    text(
                        cx,
                        "API list-price estimate, not a bill",
                        "API 价目估算，非账单"
                    ),
                    text(cx, "Report timezone", "报告时区"),
                    match clock.timezone {
                        ccr_usage::ReportTimezone::Local => text(cx, "Local", "本地"),
                        ccr_usage::ReportTimezone::Utc => "UTC",
                    },
                    text(cx, "Through", "截止日"),
                    clock.date()
                )
            })
            .unwrap_or_else(|| {
                if failure.is_some() {
                    text(cx, "Report context unavailable", "报告上下文不可用").into()
                } else {
                    text(cx, "Reading report context…", "正在读取报告上下文…").into()
                }
            });
        let mut section = div().flex_shrink_0().flex().flex_col().gap(px(8.)).child(
            div()
                .text_size(px(12.))
                .text_color(cx.theme().muted_foreground)
                .child(context),
        );
        if let Some(message) = failure {
            section = section.child(
                div()
                    .id("workbench-claude-report-error")
                    .test_support()
                    .role(gpui_kit::Role::Status)
                    .text_color(cx.theme().danger)
                    .child(crate::locales::localized_line(cx, message)),
            );
        }
        if let Some(usage) = usage {
            let sync = usage
                .history
                .last_sync_at
                .as_deref()
                .unwrap_or(text(cx, "Unknown", "未知"));
            let latest = usage
                .latest
                .as_deref()
                .unwrap_or(text(cx, "Unknown", "未知"));
            let freshness = format!(
                "{}: {sync} · {}: {latest}",
                text(cx, "Global last successful sync", "全局上次成功同步"),
                text(cx, "Latest imported record date", "最新导入记录日")
            );
            section = section.child(
                div()
                    .id("claude-usage-freshness")
                    .test_support()
                    .aria_label(freshness.clone())
                    .text_size(px(12.))
                    .child(freshness),
            );
        }
        if let Some(report) = report
            && let Err(error) = report.usage
        {
            section = section.child(
                div()
                    .text_color(cx.theme().danger)
                    .child(service_error(cx, error.message())),
            );
        }
        let narrow = window.viewport_size().width < px(1040.);
        let mut band = panel(cx).grid().grid_cols(if narrow { 2 } else { 3 }).p_0();
        for (index, id, en, zh) in [
            (
                0,
                "claude-cost-today",
                "Today recorded cost",
                "今日记录费用",
            ),
            (1, "claude-cost-month", "Month to date", "本月截至今日"),
            (
                2,
                "claude-cost-total",
                "All imported history",
                "已导入全部历史",
            ),
        ] {
            let value = usage.map(|u| match index {
                0 => &u.today,
                1 => &u.month,
                _ => &u.history,
            });
            let measured = value.filter(|v| v.bucket_count > 0);
            let amount = measured
                .map(|value| format_recorded_cost(value.total_cost_usd))
                .unwrap_or_else(|| {
                    if value.is_some() {
                        if index == 0 {
                            text(cx, "No imported records today", "今日尚无已导入记录")
                        } else {
                            text(cx, "No imported records", "尚无已导入记录")
                        }
                    } else {
                        text(cx, "Unavailable", "不可用")
                    }
                    .into()
                });
            let detail = match (index, usage, clock) {
                (0, _, Some(clock)) => clock.date().to_string(),
                (1, _, Some(clock)) => {
                    format!("{} – {}", clock.date().format("%Y-%m-01"), clock.date())
                }
                (2, Some(usage), _) => format!(
                    "{}: {}",
                    text(cx, "Since", "起点"),
                    usage
                        .first
                        .as_deref()
                        .unwrap_or(text(cx, "Unknown", "未知"))
                ),
                _ => String::new(),
            };
            band = band.child(
                div()
                    .id(id)
                    .test_support()
                    .role(gpui_kit::Role::Label)
                    .aria_label(format!("{}: {amount}; {detail}", text(cx, en, zh)))
                    .when(narrow && index == 2, |cell| cell.col_span(2))
                    .flex_1()
                    .min_w_0()
                    .p(px(16.))
                    // 最后一格不再向右画线；窄窗第三格跨两列，用上边线分隔行。
                    .when(index != 2, |cell| {
                        cell.border_r_1()
                            .border_color(theme::hairline(cx.theme().mode))
                    })
                    .when(narrow && index == 2, |cell| {
                        cell.border_t_1()
                            .border_color(theme::hairline(cx.theme().mode))
                    })
                    .child(
                        div()
                            .text_size(px(12.))
                            .text_color(cx.theme().muted_foreground)
                            .child(text(cx, en, zh)),
                    )
                    .child(
                        div()
                            .text_size(px(if measured.is_none() {
                                16.
                            } else if index == 1 {
                                32.
                            } else {
                                24.
                            }))
                            .font_features(theme::tabular_figures())
                            .font_weight(if index == 1 {
                                gpui_kit::FontWeight::SEMIBOLD
                            } else {
                                gpui_kit::FontWeight::MEDIUM
                            })
                            .child(amount),
                    )
                    .child(
                        div()
                            .text_size(px(12.))
                            .text_color(cx.theme().muted_foreground)
                            .child(detail),
                    ),
            );
        }
        section.child(band).into_any_element()
    }

    pub(super) fn claude_columns(
        &self,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> gpui_kit::AnyElement {
        if window.viewport_size().width < px(1200.) {
            return div()
                .flex_shrink_0()
                .flex()
                .flex_col()
                .gap(px(16.))
                .child(self.claude_trend(cx))
                .child(self.claude_models(cx))
                .child(self.claude_subscription(cx))
                .child(self.claude_cache(cx))
                .into_any_element();
        }
        // 与运行概览同一套几何：侧栏、场景边距、纸面亮边都按外壳实际使用的主题函数扣除。
        let viewport = window.viewport_size();
        let available = viewport.width.as_f32()
            - theme::sidebar_width(viewport.width)
            - 2. * theme::scene_inset(viewport)
            - theme::SHEET_RIM
            - 2. * theme::content_padding(viewport.width);
        let rail = ((available * 0.24 / 8.).round() * 8.).clamp(280., 360.);
        div()
            .flex_shrink_0()
            .flex()
            .items_start()
            .gap(px(16.))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap(px(16.))
                    .child(self.claude_trend(cx))
                    .child(self.claude_cache(cx)),
            )
            .child(
                div()
                    .w(px(rail))
                    .flex_shrink_0()
                    .flex()
                    .flex_col()
                    .gap(px(16.))
                    .child(self.claude_models(cx))
                    .child(self.claude_subscription(cx)),
            )
            .into_any_element()
    }

    fn claude_models(&self, cx: &mut Context<Self>) -> gpui_kit::AnyElement {
        let mut pane = panel(cx).child(heading(text(
            cx,
            "Model recorded costs · 30 days",
            "模型记录费用 · 30 天",
        )));
        if let Some(usage) = self.claude_usage() {
            let mut rows = usage.recent.models.iter().collect::<Vec<_>>();
            rows.sort_by(|a, b| {
                b.cost_with_cache_usd
                    .total_cmp(&a.cost_with_cache_usd)
                    .then_with(|| a.model.cmp(&b.model))
            });
            for row in rows.iter().take(5) {
                pane = pane.child(
                    div()
                        .flex()
                        .justify_between()
                        .gap(px(8.))
                        .child(div().min_w_0().truncate().child(row.model.clone()))
                        .child(
                            div()
                                .font_features(theme::tabular_figures())
                                .child(format_recorded_cost(row.cost_with_cache_usd)),
                        ),
                );
            }
            if rows.len() > 5 {
                pane = pane.child(format!(
                    "{}: {}",
                    text(cx, "Other", "其他"),
                    format_recorded_cost(rows[5..].iter().map(|r| r.cost_with_cache_usd).sum())
                ));
            }
            if rows.is_empty() {
                pane = pane.child(text(cx, "No imported records", "尚无已导入记录"));
            }
            if self.exact_costs {
                for row in rows {
                    pane = pane.child(div().text_size(px(12.)).child(format!(
                        "{}: {}",
                        row.model,
                        format_exact_usd(row.cost_with_cache_usd)
                    )));
                }
            } else {
                // 精确值表在趋势面板；这里给同一面板内的入口，避免宽窗下要去另一列找开关。
                pane = pane.child(semantics::ghost_button(
                    Button::new("claude-model-exact-toggle")
                        .label(text(cx, "Exact model values", "模型精确值"))
                        .ghost()
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.exact_costs = true;
                            cx.notify();
                        })),
                    false,
                    false,
                    cx,
                ));
            }
        } else {
            pane = pane.child(text(cx, "Unavailable", "不可用"));
        }
        pane.into_any_element()
    }

    fn claude_cache(&self, cx: &mut Context<Self>) -> gpui_kit::AnyElement {
        let mut pane = panel(cx).child(heading(text(
            cx,
            "Tokens and cache · 30 days",
            "Token 与缓存 · 30 天",
        )));
        if let Some(usage) = self.claude_usage() {
            let total = &usage.recent.overview.total;
            let ratio = cache_ratio(total.input_tokens, total.cache_read_tokens)
                .map(|v| format!("{:.2}%", v * 100.))
                .unwrap_or_else(|| {
                    text(
                        cx,
                        "Not calculable (zero denominator)",
                        "不可计算（分母为零）",
                    )
                    .into()
                });
            let output = usage
                .recent
                .trends
                .iter()
                .map(|row| row.output_tokens)
                .sum::<i64>();
            let creation = usage
                .recent
                .trends
                .iter()
                .map(|row| row.cache_creation_tokens)
                .sum::<i64>();
            for (index, (label, value)) in [
                (text(cx, "Input", "输入"), grouped(total.input_tokens)),
                (
                    text(cx, "Cache read", "缓存读取"),
                    grouped(total.cache_read_tokens),
                ),
                (text(cx, "Cache creation", "缓存创建"), grouped(creation)),
                (
                    text(cx, "Output (includes reasoning)", "输出（含推理）"),
                    grouped(output),
                ),
                (
                    text(cx, "Total (owner projection)", "总量（领域投影）"),
                    grouped(total.total_tokens),
                ),
                (
                    text(cx, "Cache token hit ratio", "缓存 Token 命中率"),
                    ratio,
                ),
            ]
            .into_iter()
            .enumerate()
            {
                pane = pane.child(
                    div()
                        .id(format!("claude-cache-{index}"))
                        .test_support()
                        .aria_label(format!("{label}: {value}"))
                        .flex()
                        .justify_between()
                        .gap(px(8.))
                        .child(label)
                        .child(div().font_features(theme::tabular_figures()).child(value)),
                );
            }
        } else {
            pane = pane.child(text(cx, "Unavailable", "不可用"));
        }
        pane.into_any_element()
    }

    fn edit_subscription(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let value = self
            .subscription_state
            .as_ref()
            .and_then(|v| v.as_ref().ok())
            .and_then(|v| v.clone())
            .unwrap_or_default();
        self.subscription_fields = Some(
            [value.mode, value.plan, value.monthly_usd.to_string()]
                .into_iter()
                .map(|value| cx.new(|cx| InputState::new(window, cx).default_value(value)))
                .collect(),
        );
        cx.notify();
    }
    fn save_subscription(&mut self, cx: &mut Context<Self>) {
        let Some(fields) = &self.subscription_fields else {
            return;
        };
        // 存储版本比草稿新时必须先重读；否则这次提交会拿旧 revision 覆盖别人的修改。
        if self.subscription_stale {
            self.subscription_message = Some(
                "Stored settings changed; reload before saving / 存储设置已变化，请先重读再保存"
                    .into(),
            );
            cx.notify();
            return;
        }
        let Ok(monthly_usd) = fields[2].read(cx).value().parse::<f64>() else {
            self.subscription_message = Some("Invalid monthly price / 月费无效".into());
            cx.notify();
            return;
        };
        let proposed = SubscriptionDto {
            mode: fields[0].read(cx).value().trim().into(),
            plan: fields[1].read(cx).value().trim().into(),
            monthly_usd,
        };
        let Some(Ok(expected)) = self.subscription_state.clone() else {
            return;
        };
        let generation = self.generation;
        let key = RequestKey {
            environment: EnvironmentId::Local("claude-workbench-subscription".into()),
            platform: Platform::Claude,
            generation,
            request_id: generation,
        };
        let mut updates = self.requests.observe_observer();
        match self.requests.observer_save(key,expected,proposed) {
            Err(error)=>{self.subscription_message=Some(subscription_error(error).into());}
            Ok(())=>self.subscription_task=Some(cx.spawn(async move|weak,cx|{
                loop {
                    let update=updates.borrow_and_update().clone();
                    if let Update::Finished(result)=update {
                        let _=weak.update(cx,|this,cx|{
                            if this.generation!=generation || this.platform!=Platform::Claude{return;}
                            match result {
                                Ok(value) => {
                                    this.subscription_state = Some(Ok(Some(value)));
                                    this.subscription_fields = None;
                                    this.subscription_stale = false;
                                    this.subscription_message = Some(
                                        "Saved; no sync or account switch / 已保存，未触发同步或账户切换"
                                            .into(),
                                    );
                                }
                                Err(Error::Conflict) => {
                                    // 冲突：保留草稿，并把存储版本读回来，下一次保存用新 revision。
                                    this.subscription_stale = true;
                                    this.subscription_message = Some(
                                        "Changed elsewhere; draft retained. Reload, then save again / 设置已在别处变化，草稿保留；重读后再保存"
                                            .into(),
                                    );
                                }
                                Err(error) => {
                                    this.subscription_message =
                                        Some(subscription_error(error).into());
                                }
                            }
                            cx.notify();
                        });break;
                    }
                    if updates.changed().await.is_err(){break;}
                }
            })),
        }
        cx.notify();
    }
    /// 冲突后重读存储版本；只刷新 CAS 期望值，不覆盖用户草稿。
    fn reload_subscription(&mut self, cx: &mut Context<Self>) {
        let generation = self.generation;
        let clock = self.requests.report_clock();
        let key = RequestKey {
            environment: EnvironmentId::Local("claude-workbench-subscription-reload".into()),
            platform: Platform::Claude,
            generation,
            request_id: generation,
        };
        match self.requests.claude_workbench(key, clock) {
            Ok(mut request) => {
                self.subscription_task = Some(cx.spawn(async move |weak, cx| {
                    while let Some(event) = request.next().await {
                        if let TaskEvent::Terminal(result) = event {
                            let _ = weak.update(cx, |this, cx| {
                                if this.generation != generation
                                    || this.platform != Platform::Claude
                                {
                                    return;
                                }
                                if let Completion::Success(report) = result.as_ref() {
                                    this.subscription_state = Some(report.subscription.clone());
                                    this.subscription_stale = false;
                                    this.subscription_message =
                                        Some("Stored settings reloaded / 已重读存储设置".into());
                                } else {
                                    this.subscription_message = Some(
                                        "Reload failed; draft retained / 重读失败，草稿保留".into(),
                                    );
                                }
                                cx.notify();
                            });
                            break;
                        }
                    }
                }))
            }
            Err(_) => {
                self.subscription_message = Some("Read queue unavailable / 读取队列不可用".into())
            }
        }
        cx.notify();
    }
    fn claude_subscription(&self, cx: &mut Context<Self>) -> gpui_kit::AnyElement {
        let mut pane = panel(cx).child(heading(text(cx, "Subscription settings", "订阅设置")));
        let busy = matches!(self.requests.observer_update(), Update::Running);
        if let Some(fields) = &self.subscription_fields {
            pane = pane.child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap(px(8.))
                    .child(semantics::button(
                        Button::new("workbench-subscription-save")
                            .label(text(cx, "Save", "保存"))
                            .primary()
                            .on_click(cx.listener(|this, _, _, cx| this.save_subscription(cx))),
                        busy,
                        busy,
                        cx,
                    ))
                    .child(semantics::ghost_button(
                        Button::new("workbench-subscription-cancel")
                            .label(text(cx, "Cancel", "取消"))
                            .ghost()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.subscription_fields = None;
                                this.subscription_message = None;
                                // 放弃草稿时也不能留着过期的存储版本。
                                if this.subscription_stale {
                                    this.reload_subscription(cx);
                                }
                                cx.notify();
                            })),
                        busy,
                        false,
                        cx,
                    )),
            );
            for (index, label) in [
                text(
                    cx,
                    "Mode: auto / api_key / subscription",
                    "模式：auto / api_key / subscription",
                ),
                text(cx, "Plan", "套餐"),
                text(cx, "Monthly price (USD)", "整月月费（USD）"),
            ]
            .into_iter()
            .enumerate()
            {
                pane = pane.child(div().text_size(px(12.)).child(label)).child(
                    Input::new(&fields[index])
                        .id(format!("workbench-subscription-field-{index}"))
                        .aria_label(label)
                        .disabled(busy),
                );
            }
        } else {
            let summary = match &self.subscription_state {
                Some(Ok(Some(value))) => {
                    let mode = match value.mode.as_str() {
                        "auto" => text(cx, "Auto detect", "自动判断"),
                        "api_key" => text(cx, "API key", "API Key 计费"),
                        "subscription" => text(cx, "Subscription", "订阅计费"),
                        unknown => unknown,
                    };
                    if value.mode == "subscription" {
                        format!(
                            "{mode} · {} · ${:.2} / {}",
                            subscription_plan(&value.plan, cx),
                            value.monthly_usd,
                            text(cx, "month", "月")
                        )
                    } else {
                        mode.into()
                    }
                }
                Some(Ok(None)) => text(cx, "Not configured", "未设置").into(),
                Some(Err(_)) => text(cx, "Settings unreadable", "设置读取失败").into(),
                None => text(cx, "Reading…", "读取中…").into(),
            };
            pane = pane.child(
                div()
                    .id("claude-subscription")
                    .test_support()
                    .aria_label(summary.clone())
                    .child(summary),
            );
            if let (Some(Ok(Some(value))), Some(usage)) =
                (&self.subscription_state, self.claude_usage())
                && value.mode == "subscription"
                && value.monthly_usd.is_finite()
                && value.monthly_usd > 0.
                && usage.month.bucket_count > 0
            {
                pane = pane.child(format!(
                    "{}: {:.2}×",
                    text(cx, "Subscription value ratio", "订阅价值比"),
                    usage.month.total_cost_usd / value.monthly_usd
                ));
            }
            pane = pane.child(semantics::ghost_button(
                Button::new("workbench-subscription-edit")
                    .label(text(cx, "Edit settings", "编辑设置"))
                    .ghost()
                    .on_click(
                        cx.listener(|this, _, window, cx| this.edit_subscription(window, cx)),
                    ),
                !matches!(self.subscription_state, Some(Ok(_))) || busy,
                false,
                cx,
            ));
        }
        if self.subscription_stale {
            pane = pane.child(semantics::ghost_button(
                Button::new("workbench-subscription-reload")
                    .label(text(cx, "Reload saved settings", "重读存储设置"))
                    .ghost()
                    .on_click(cx.listener(|this, _, _, cx| this.reload_subscription(cx))),
                busy,
                false,
                cx,
            ));
        }
        if let Some(message) = &self.subscription_message {
            pane = pane.child(
                div()
                    .id("workbench-subscription-status")
                    .role(gpui_kit::Role::Status)
                    .child(crate::locales::localized_line(cx, message)),
            );
        }
        pane.child(div().text_size(px(12.)).text_color(cx.theme().muted_foreground).child(text(cx,"Month-to-date recorded cost / full monthly price. No proration; this is not savings or a bill.","本月截至今日记录费用 ÷ 整月月费。不按天折算，不代表节省或账单。"))).into_any_element()
    }
}

fn subscription_error(error: Error) -> &'static str {
    match error {
        Error::Conflict => {
            "Changed elsewhere; draft retained. Reload before saving / 设置已变化，草稿保留；重读后再保存"
        }
        Error::Invalid => "Invalid mode, plan or monthly price / 模式、套餐或月费无效",
        Error::Busy => "Another save is running / 已有保存正在进行",
        _ => "Save failed; draft retained / 保存失败，草稿保留",
    }
}
fn safe_host(raw: &str) -> Option<String> {
    reqwest::Url::parse(raw)
        .ok()
        .and_then(|url| url.host_str().map(str::to_owned))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[gpui_kit::test]
    async fn claude_snapshot_subscription_drafts_and_leave_remain_scoped(
        cx: &mut gpui_kit::TestAppContext,
    ) {
        use crate::runtime::{ConfigSource, insights::InsightsPaths};
        use ccr_app::runtime::RuntimeOwner;
        use gpui_kit::test::{TestAppContextExt, TestWindowExt};
        use gpui_kit::{AppContext, WindowOptions};
        use std::time::Duration;
        cx.executor().allow_parking();
        let root = tempfile::tempdir().unwrap_or_else(|e| panic!("fixture: {e}"));
        let paths = ccr_usage::fixtures::create_projection_db(&root.path().join("usage"));
        let conn = rusqlite::Connection::open(paths.db_path).unwrap_or_else(|e| panic!("db: {e}"));
        ccr_usage::fixtures::seed_bucket(
            &conn,
            &ccr_usage::fixtures::SeedBucket {
                source: "claude".into(),
                hour_start: format!("{}T12:00:00Z", chrono::Local::now().date_naive()),
                ..Default::default()
            },
        );
        drop(conn);
        let owner = RuntimeOwner::new().unwrap_or_else(|e| panic!("runtime: {e}"));
        let services = Services::new(
            owner.handle(),
            ConfigSource::Fixture {
                profiles: root.path().join("profiles.toml"),
                registry: root.path().join("registry.toml"),
                delay: Duration::ZERO,
            },
        )
        .with_insights_paths(InsightsPaths {
            usage_root: root.path().join("usage"),
            archive_db: root.path().join("absent.db"),
            provider_home: root.path().join("home"),
        });
        let (handle, view) = cx.update(|cx| {
            gpui_kit::init(cx);
            gpui_kit::open_window(WindowOptions::default(), cx, |_, cx| {
                cx.new(|cx| {
                    let mut view = WorkbenchView::new(services, cx);
                    view.open(Platform::Claude, cx);
                    view
                })
            })
            .unwrap_or_else(|e| panic!("window: {e}"))
        });
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
            view.read(cx).claude_report.is_some()
        })
        .await;
        cx.update_window(handle, |_, window, cx| {
            assert!(view.read(cx).claude_usage().is_some());
            assert_eq!(
                window.find("claude-subscription").label(),
                Some("Not configured")
            );
            view.update(cx, |this, cx| {
                this.subscription_state = Some(Ok(Some(SubscriptionDto::default())));
                cx.notify();
            });
            window.render_frame(cx);
            assert_eq!(
                window.find("claude-subscription").label(),
                Some("Auto detect")
            );
            for (mode, plan, expected) in [
                ("unknown-mode", "team", "unknown-mode"),
                (
                    "subscription",
                    "max20x",
                    "Subscription · Max 20× · $200.00 / month",
                ),
                (
                    "subscription",
                    "unknown-plan",
                    "Subscription · unknown-plan · $200.00 / month",
                ),
                ("auto", "team", "Auto detect"),
            ] {
                view.update(cx, |this, cx| {
                    this.subscription_state = Some(Ok(Some(SubscriptionDto {
                        mode: mode.into(),
                        plan: plan.into(),
                        monthly_usd: 200.0,
                    })));
                    cx.notify();
                });
                window.render_frame(cx);
                assert_eq!(window.find("claude-subscription").label(), Some(expected));
            }
            // 键盘/点击激活日期条后，日期、金额与口径必须出现在可见读数行，不能只靠 hover。
            assert!(window.try_find("workbench-claude-trend-readout").is_none());
            let label = {
                let bar = window.find("claude-cost-bar-29");
                bar.label()
                    .map(str::to_owned)
                    .unwrap_or_else(|| panic!("seeded day bar label"))
            };
            assert!(!label.is_empty());
            window.click("claude-cost-bar-29", cx);
            window.render_frame(cx);
            let readout = {
                let node = window.find("workbench-claude-trend-readout");
                node.label().map(str::to_owned)
            };
            assert_eq!(readout.as_deref(), Some(label.as_str()));
            assert!(window.try_find("claude-cost-exact-table").is_some());
            view.update(cx, |this, cx| {
                this.exact_costs = false;
                cx.notify();
            });
            window.render_frame(cx);
            assert!(
                window.try_find("claude-model-exact-toggle").is_some(),
                "the models panel offers its own exact-values entry"
            );
            window.click("claude-model-exact-toggle", cx);
            assert!(view.read(cx).exact_costs);
            view.update(cx, |this, cx| {
                this.exact_costs = true;
                this.edit_subscription(window, cx);
            });
            window.render_frame(cx);
            assert!(window.try_find("claude-cost-exact-table").is_some());
            assert!(window.try_find("workbench-subscription-save").is_some());
            view.update(cx, |this, cx| {
                // 克隆实体句柄：后面还要在保存之后继续改这个输入框。
                let price = this
                    .subscription_fields
                    .as_ref()
                    .unwrap_or_else(|| panic!("fields"))[2]
                    .clone();
                price.update(cx, |state, cx| state.set_value("invalid price", window, cx));
                this.save_subscription(cx);
                assert!(this.subscription_fields.is_some());
                assert!(this.subscription_message.is_some());
                // 冲突后存储版本过期：再次提交必须被拒绝并保留草稿，不能拿旧 revision 覆盖。
                price.update(cx, |state, cx| state.set_value("200", window, cx));
                this.subscription_stale = true;
                this.subscription_message = None;
                this.save_subscription(cx);
                assert!(this.subscription_fields.is_some());
                assert_eq!(
                    this.subscription_message.as_deref(),
                    Some("Stored settings changed; reload before saving / 存储设置已变化，请先重读再保存")
                );
                this.subscription_stale = false;
                let generation = this.generation;
                this.suspend(cx);
                assert!(this.generation > generation);
                assert!(this.subscription_fields.is_none());
                assert!(this.subscription_task.is_none());
                assert!(this.subscription_state.is_none());
                assert!(this.claude_report.is_none());
            });
            window.remove_window();
        })
        .unwrap_or_else(|e| panic!("subscription: {e}"));
        owner.shutdown();
    }

    /// 用固定 UTC 时钟渲染一个 Claude 工作台，读回给定 ID 的无障碍标签（不存在为 None）。
    async fn claude_home_labels(
        cx: &mut gpui_kit::TestAppContext,
        usage_root: std::path::PathBuf,
        ids: &[&'static str],
    ) -> Vec<Option<String>> {
        use crate::runtime::{ConfigSource, insights::InsightsPaths};
        use ccr_app::runtime::RuntimeOwner;
        use gpui_kit::test::{TestAppContextExt, TestWindowExt};
        use gpui_kit::{AppContext, WindowOptions};
        use std::time::Duration;
        let root = tempfile::tempdir().unwrap_or_else(|e| panic!("fixture: {e}"));
        let owner = RuntimeOwner::new().unwrap_or_else(|e| panic!("runtime: {e}"));
        let services = Services::new(
            owner.handle(),
            ConfigSource::Fixture {
                profiles: root.path().join("profiles.toml"),
                registry: root.path().join("registry.toml"),
                delay: Duration::ZERO,
            },
        )
        .with_insights_paths(InsightsPaths {
            archive_db: usage_root.join("absent.db"),
            provider_home: usage_root.join("home"),
            usage_root,
        });
        let clock = ReportClock::fixed_utc(
            chrono::DateTime::parse_from_rfc3339("2026-11-10T12:00:00Z")
                .unwrap_or_else(|e| panic!("clock: {e}"))
                .with_timezone(&chrono::Utc),
        );
        let (handle, view) = cx.update(|cx| {
            gpui_kit::init(cx);
            gpui_kit::open_window(
                WindowOptions {
                    window_bounds: Some(gpui_kit::WindowBounds::Windowed(gpui_kit::Bounds::new(
                        gpui_kit::point(px(0.), px(0.)),
                        gpui_kit::size(px(1600.), px(2400.)),
                    ))),
                    ..Default::default()
                },
                cx,
                |_, cx| {
                    cx.new(|cx| {
                        let mut view = WorkbenchView::new(services, cx);
                        view.load_claude(clock, cx);
                        view
                    })
                },
            )
            .unwrap_or_else(|e| panic!("window: {e}"))
        });
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
            view.read(cx).claude_report.is_some()
        })
        .await;
        let labels = cx
            .update_window(handle, |_, window, cx| {
                window.render_frame(cx);
                let labels = ids
                    .iter()
                    .map(|id| {
                        window
                            .try_find(*id)
                            .and_then(|node| node.label().map(str::to_owned))
                    })
                    .collect();
                window.remove_window();
                labels
            })
            .unwrap_or_else(|e| panic!("labels: {e}"));
        owner.shutdown();
        labels
    }

    #[gpui_kit::test]
    async fn claude_home_states_unavailable_unknown_and_window_scoped_cache(
        cx: &mut gpui_kit::TestAppContext,
    ) {
        use ccr_usage::fixtures::{SeedBucket, create_projection_db, seed_bucket, seed_run_log};
        cx.executor().allow_parking();
        const IDS: [&str; 6] = [
            "claude-cost-today",
            "claude-cost-total",
            "claude-usage-freshness",
            "claude-cache-3",
            "claude-cache-4",
            "claude-cache-5",
        ];
        let root = tempfile::tempdir().unwrap_or_else(|e| panic!("fixture: {e}"));
        // 无库与不可读库：KPI 写“不可用”，缓存面板不出现任何数字，绝不显示 $0.00。
        let missing = root.path().join("missing");
        let unreadable = root.path().join("unreadable");
        std::fs::create_dir_all(&unreadable).unwrap_or_else(|e| panic!("dir: {e}"));
        std::fs::write(unreadable.join("llmusage.db"), b"not a sqlite database")
            .unwrap_or_else(|e| panic!("write: {e}"));
        for usage_root in [missing, unreadable] {
            let labels = claude_home_labels(cx, usage_root, &IDS).await;
            assert_eq!(
                labels[0].as_deref(),
                Some("Today recorded cost: Unavailable; 2026-11-10")
            );
            assert_eq!(
                labels[1].as_deref(),
                Some("All imported history: Unavailable; ")
            );
            assert_eq!(labels[2], None, "no freshness claim without a report");
            assert!(labels[3..].iter().all(Option::is_none), "{labels:?}");
        }
        // 空库：今日尚无记录、起点与同步未知、分母为零不可计算。
        let empty = root.path().join("empty");
        create_projection_db(&empty);
        let labels = claude_home_labels(cx, empty, &IDS).await;
        assert_eq!(
            labels,
            vec![
                Some("Today recorded cost: No imported records today; 2026-11-10".into()),
                Some("All imported history: No imported records; Since: Unknown".into()),
                Some(
                    "Global last successful sync: Unknown · Latest imported record date: Unknown"
                        .into()
                ),
                Some("Output (includes reasoning): 0".into()),
                Some("Total (owner projection): 0".into()),
                Some("Cache token hit ratio: Not calculable (zero denominator)".into()),
            ]
        );
        // 窗口外有缓存、窗口内只有含推理的输出：命中率仍不可计算，输出 45 不重复加推理。
        let scoped = root.path().join("scoped");
        let db = create_projection_db(&scoped);
        let conn = rusqlite::Connection::open(db.db_path).unwrap_or_else(|e| panic!("db: {e}"));
        for (hour, input, cache, output, reasoning, total, cost) in [
            ("2026-09-26T12:00:00Z", 100, 500, 10, 0, 610, 8.),
            ("2026-11-05T12:00:00Z", 0, 0, 30, 15, 45, 0.5),
        ] {
            seed_bucket(
                &conn,
                &SeedBucket {
                    source: "claude".into(),
                    hour_start: hour.into(),
                    input_tokens: input,
                    cache_read_tokens: cache,
                    cache_creation_tokens: 0,
                    output_tokens: output,
                    reasoning_output_tokens: reasoning,
                    total_tokens: total,
                    cost_with_cache_usd: cost,
                    ..Default::default()
                },
            );
        }
        // 只有 Codex 的同步记录：页面仍按“全局”陈述这个旧时间。
        seed_run_log(&conn, "codex", "recent", "2026-10-01T00:00:00Z");
        drop(conn);
        let labels = claude_home_labels(cx, scoped, &IDS).await;
        assert_eq!(
            labels,
            vec![
                Some("Today recorded cost: No imported records today; 2026-11-10".into()),
                Some("All imported history: $8.50; Since: 2026-09-26".into()),
                Some(
                    "Global last successful sync: 2026-10-01T00:00:00Z · Latest imported record date: 2026-11-05"
                        .into()
                ),
                Some("Output (includes reasoning): 45".into()),
                Some("Total (owner projection): 45".into()),
                Some("Cache token hit ratio: Not calculable (zero denominator)".into()),
            ]
        );
    }

    #[test]
    fn recorded_amounts_keep_zero_and_subcent_cost_distinct() {
        for (value, expected) in [
            (0., "$0.00"),
            (0.004, "<$0.01"),
            (0.0123, "$0.01"),
            (135.74, "$135.74"),
        ] {
            assert_eq!(format_recorded_cost(value), expected);
        }
    }
    #[test]
    fn endpoint_facts_never_expose_url_credentials_or_query() {
        assert_eq!(
            safe_host("https://user:secret@example.org/path?token=secret#private").as_deref(),
            Some("example.org")
        );
        // 解析失败交给调用点按界面语言显示“未知”，不回落到原始字符串。
        assert_eq!(safe_host("not a url with secret"), None);
    }
}
