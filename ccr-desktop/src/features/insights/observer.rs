//! Claude usage and tool projections with a separately confirmed subscription setting.
use super::*;
use crate::runtime::insights::observer::{Error, Report, Update};
use ccr_app::usage::observer::subscription::SubscriptionDto;
use gpui_kit::component::{
    FocusTrapElement,
    input::{Textarea, TextareaState},
};
mod presentation;

pub(super) struct ObserverView {
    requests: InsightsRequests,
    fields: Vec<Entity<InputState>>,
    output: Entity<TextareaState>,
    report: Option<Report>,
    advanced: bool,
    pending: Option<(String, Option<SubscriptionDto>)>,
    expected: Option<SubscriptionDto>,
    available: bool,
    days: i64,
    calls: bool,
    generation: u64,
    task: Option<Task<()>>,
    observer: Option<Task<()>>,
    loading: bool,
    message: String,
    decision: Option<SubscriptionDto>,
    decision_generation: u64,
    entry_pending: bool,
    return_focus: Option<FocusHandle>,
    confirmation_focus: FocusHandle,
}
fn error_text(error: Error, cx: &gpui_kit::App) -> &'static str {
    match error {
        Error::LocalOnly => text(
            cx,
            "Claude Observer is available in Local only.",
            "Claude 观测仅支持本地环境。",
        ),
        Error::Missing => text(
            cx,
            "No stored data for this source.",
            "此来源尚无已存数据。",
        ),
        Error::Storage => text(
            cx,
            "The existing database is unavailable or incompatible.",
            "现有数据库不可读取或不兼容。",
        ),
        Error::Invalid => text(
            cx,
            "Choose a valid mode, plan and nonnegative monthly price.",
            "请选择有效模式、套餐和非负月费。",
        ),
        Error::Conflict => text(
            cx,
            "Subscription changed elsewhere. Reload before saving.",
            "订阅已被其他操作修改，请重读后再保存。",
        ),
        Error::Busy => text(
            cx,
            "A subscription update is already running.",
            "已有订阅更新正在运行。",
        ),
        Error::Interrupted => text(
            cx,
            "Subscription update returned no result. Reload before retrying.",
            "订阅更新未返回结果，请重读后再重试。",
        ),
    }
}
impl ObserverView {
    pub fn new(requests: InsightsRequests, window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            requests,
            fields: (0..3)
                .map(|_| cx.new(|cx| InputState::new(window, cx)))
                .collect(),
            output: cx.new(|cx| TextareaState::new(window, cx)),
            report: None,
            advanced: false,
            pending: None,
            expected: None,
            available: false,
            days: 30,
            calls: true,
            generation: 0,
            task: None,
            observer: None,
            loading: false,
            message: String::new(),
            decision: None,
            decision_generation: 0,
            entry_pending: false,
            return_focus: None,
            confirmation_focus: cx.focus_handle().tab_stop(false),
        }
    }
    pub fn suspend(&mut self, cx: &mut Context<Self>) {
        self.advanced = false;
        self.report = None;
        self.task = None;
        self.observer = None;
        self.generation += 1;
        self.loading = false;
        self.decision = None;
        self.decision_generation += 1;
        self.entry_pending = false;
        self.return_focus = None;
        cx.notify();
    }
    fn key(&self) -> RequestKey {
        RequestKey {
            environment: EnvironmentId::Local("insights-explicit".into()),
            platform: Platform::Claude,
            generation: self.generation,
            request_id: self.generation,
        }
    }
    pub fn open(&mut self, cx: &mut Context<Self>) {
        let mut updates = self.requests.observe_observer();
        self.observer = Some(cx.spawn(async move |weak, cx| {
            while updates.changed().await.is_ok() {
                let state = updates.borrow_and_update().clone();
                if weak
                    .update(cx, |this, cx| {
                        if let Update::Finished(result) = state {
                            this.message = match result {
                                Ok(_) => text(cx, "Subscription saved.", "订阅已保存。").into(),
                                Err(error) => error_text(error, cx).into(),
                            };
                            this.load(cx);
                        }
                        cx.notify();
                    })
                    .is_err()
                {
                    break;
                }
            }
        }));
        if let Update::Finished(result) = self.requests.observer_update() {
            self.message = match result {
                Ok(_) => text(cx, "Subscription saved.", "订阅已保存。").into(),
                Err(error) => error_text(error, cx).into(),
            };
        }
        self.load(cx);
    }
    fn load(&mut self, cx: &mut Context<Self>) {
        if self.decision.is_some() {
            return;
        }
        self.generation += 1;
        let generation = self.generation;
        self.available = false;
        self.report = None;
        self.pending = Some((
            text(cx, "Reading stored data…", "正在读取已存数据…").into(),
            self.expected.clone(),
        ));
        match self
            .requests
            .observer_read(self.key(), self.days, self.calls)
        {
            Ok(mut subscription) => {
                self.loading = true;
                self.task = Some(cx.spawn(async move |weak, cx| {
                    while let Some(event) = subscription.next().await {
                        if let TaskEvent::Terminal(result) = event {
                            let _ = weak.update(cx, |this, cx| {
                                if this.generation != generation {
                                    return;
                                }
                                this.loading = false;
                                match result.as_ref() {
                                    Completion::Success(report) => this.receive(report, cx),
                                    Completion::Failed(error)
                                    | Completion::CleanupFailed(error) => {
                                        this.message = error_text(*error, cx).into()
                                    }
                                    _ => this.message = error_text(Error::Interrupted, cx).into(),
                                }
                                cx.notify();
                            });
                            break;
                        }
                    }
                }));
            }
            Err(_) => self.message = error_text(Error::Interrupted, cx).into(),
        }
        cx.notify();
    }
    fn receive(&mut self, report: &Report, cx: &mut Context<Self>) {
        self.report = Some(report.clone());
        let section = |name: &str, value: &Result<serde_json::Value, Error>| {
            format!(
                "{name}\n{}\n",
                match value {
                    Ok(value) => serde_json::to_string_pretty(value).unwrap_or_default(),
                    Err(error) => error_text(*error, cx).into(),
                }
            )
        };
        let output = format!(
            "{}\n{}\n{}",
            section(
                text(cx, "Claude totals and ROI", "Claude 总计与订阅比值"),
                &report.insight
            ),
            section(
                text(
                    cx,
                    "Usage: daily, projects, models and cache",
                    "用量：逐日、项目、模型与缓存"
                ),
                &report.usage
            ),
            section(
                text(
                    cx,
                    "Tools: top sessions (30 days), heatmap and ranking",
                    "工具：会话排行（30 天）、热力数据与工具排行"
                ),
                &report.tools
            )
        );
        match &report.subscription {
            Ok(value) => {
                self.available = true;
                if value.is_none() {
                    self.message = text(cx,"The CCR main database does not exist. Confirming Save initializes it; usage data is separate.","CCR 主数据库尚不存在。确认保存后将初始化主库；用量数据来自独立来源。").into();
                }
                self.pending = Some((output, value.clone()));
            }
            Err(error) => {
                self.message = error_text(*error, cx).into();
                self.pending = Some((output, None));
            }
        }
    }
    fn decide(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.available
            || self.loading
            || matches!(self.requests.observer_update(), Update::Running)
        {
            return;
        }
        let monthly = self.fields[2].read(cx).value().parse::<f64>();
        let Ok(monthly_usd) = monthly else {
            self.message = error_text(Error::Invalid, cx).into();
            cx.notify();
            return;
        };
        let proposed = SubscriptionDto {
            mode: self.fields[0].read(cx).value().trim().into(),
            plan: self.fields[1].read(cx).value().trim().into(),
            monthly_usd,
        };
        if !matches!(proposed.mode.as_str(), "auto" | "api_key" | "subscription")
            || !matches!(
                proposed.plan.as_str(),
                "free_pro" | "max5x" | "max20x" | "team" | "enterprise" | "custom"
            )
            || !monthly_usd.is_finite()
            || monthly_usd < 0.0
        {
            self.message = error_text(Error::Invalid, cx).into();
            cx.notify();
            return;
        }
        self.decision = Some(proposed);
        self.return_focus = window.focused(cx);
        self.decision_generation += 1;
        self.entry_pending = true;
        cx.notify();
    }
    fn dismiss(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.decision = None;
        self.entry_pending = false;
        if let Some(focus) = self.return_focus.take() {
            window.focus(&focus, cx);
        }
        cx.notify();
    }
}
impl Render for ObserverView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if let Some((output, expected)) = self.pending.take() {
            self.output
                .update(cx, |state, cx| state.set_value(output, window, cx));
            self.expected = expected;
            let value = self.expected.clone().unwrap_or_default();
            for (i, value) in [value.mode, value.plan, value.monthly_usd.to_string()]
                .into_iter()
                .enumerate()
            {
                self.fields[i].update(cx, |state, cx| state.set_value(value, window, cx));
            }
        }
        let busy = self.loading || matches!(self.requests.observer_update(), Update::Running);
        let mut root = div()
            .id("claude-observer")
            .test_support()
            .flex()
            .flex_col()
            .min_h_0()
            .flex_1()
            .gap(px(theme::CONTROL_GAP))
            .overflow_y_scroll()
            .track_focus(&self.confirmation_focus)
            .tab_stop(false)
            .on_key_down(
                cx.listener(|this, event: &gpui_kit::KeyDownEvent, window, cx| {
                    if this.decision.is_none() {
                        return;
                    }
                    this.decision_generation += 1;
                    if event.keystroke.key == "escape"
                        || (event.keystroke.key == "enter"
                            && window.focused(cx).as_ref() == Some(&this.confirmation_focus))
                    {
                        this.dismiss(window, cx);
                        cx.stop_propagation();
                    }
                }),
            );
        if let Some(proposed) = self.decision.as_ref() {
            if std::mem::take(&mut self.entry_pending) {
                window.focus(&self.confirmation_focus, cx);
                let generation = self.decision_generation;
                let weak = cx.weak_entity();
                window.on_next_frame(move |window, cx| {
                    let _ = weak.update(cx, |this, cx| {
                        if this.decision.is_some()
                            && this.decision_generation == generation
                            && window.focused(cx).as_ref() == Some(&this.confirmation_focus)
                        {
                            window.focus_next(cx);
                        }
                    });
                });
            }
            let label = if is_chinese(cx) {
                format!(
                    "保存订阅：{} / {}，月费 ${}？",
                    proposed.mode, proposed.plan, proposed.monthly_usd
                )
            } else {
                format!(
                    "Save subscription: {} / {}, ${} per month?",
                    proposed.mode, proposed.plan, proposed.monthly_usd
                )
            };
            return root
                .child(
                    div()
                        .id("observer-confirmation")
                        .test_support()
                        .role(gpui_kit::Role::AlertDialog)
                        .aria_label(label.clone())
                        .child(label),
                )
                .child(
                    div()
                        .tab_group()
                        .tab_stop(false)
                        .flex()
                        .gap(px(theme::CONTROL_GAP))
                        .child(semantics::button(
                            Button::new("observer-cancel")
                                .label(text(cx, "Cancel", "取消"))
                                .ghost()
                                .on_click(
                                    cx.listener(|this, _, window, cx| this.dismiss(window, cx)),
                                ),
                            false,
                            false,
                            cx,
                        ))
                        .child(semantics::button(
                            Button::new("observer-confirm")
                                .label(text(cx, "Save subscription", "保存订阅"))
                                .on_click(cx.listener(|this, _, window, cx| {
                                    let proposed = this.decision.take();
                                    this.dismiss(window, cx);
                                    if let Some(proposed) = proposed {
                                        if let Err(error) = this.requests.observer_save(
                                            this.key(),
                                            this.expected.clone(),
                                            proposed,
                                        ) {
                                            this.message = error_text(error, cx).into();
                                        }
                                        cx.notify();
                                    }
                                })),
                            false,
                            false,
                            cx,
                        ))
                        .focus_trap("observer-confirmation-trap", &self.confirmation_focus),
                )
                .into_any_element();
        }
        let mut controls = div().flex().flex_wrap().gap(px(theme::CONTROL_GAP));
        for days in [7, 30, 90] {
            controls = controls.child(semantics::button(
                Button::new(SharedString::from(format!("observer-days-{days}")))
                    .label(if is_chinese(cx) {
                        format!("{days} 天")
                    } else {
                        format!("{days} days")
                    })
                    .ghost()
                    .toggled(self.days == days)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.days = days;
                        this.load(cx);
                    })),
                busy,
                false,
                cx,
            ));
        }
        controls = controls
            .child(semantics::button(
                Button::new("observer-calls")
                    .label(text(cx, "Rank sessions by calls", "按调用数排列会话"))
                    .ghost()
                    .toggled(self.calls)
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.calls = !this.calls;
                        this.load(cx);
                    })),
                busy,
                false,
                cx,
            ))
            .child(semantics::button(
                Button::new("observer-refresh")
                    .label(text(cx, "Reload", "重读"))
                    .ghost()
                    .on_click(cx.listener(|this, _, _, cx| this.load(cx))),
                busy,
                false,
                cx,
            ));
        root = root.child(controls).child(
            div()
                .id("observer-status")
                .test_support()
                .role(gpui_kit::Role::Status)
                .child(if busy {
                    text(cx, "Working on background worker", "正在后台处理").to_string()
                } else {
                    self.message.clone()
                }),
        );
        for (i, en, zh) in [
            (
                0,
                "Mode: auto, api_key, subscription",
                "模式：auto、api_key、subscription",
            ),
            (
                1,
                "Plan: free_pro, max5x, max20x, team, enterprise, custom",
                "套餐：free_pro、max5x、max20x、team、enterprise、custom",
            ),
            (2, "Monthly price (USD)", "月费（美元）"),
        ] {
            root = root.child(
                Input::new(&self.fields[i])
                    .id(SharedString::from(format!("observer-field-{i}")))
                    .aria_label(text(cx, en, zh))
                    .disabled(busy || !self.available),
            );
        }
        root = root
            .child(semantics::button(
                Button::new("observer-save")
                    .label(text(cx, "Save subscription", "保存订阅"))
                    .on_click(cx.listener(|this, _, window, cx| this.decide(window, cx))),
                busy || !self.available,
                false,
                cx,
            ))
            .child(semantics::button(
                Button::new("observer-advanced")
                    .label(text(cx, "Advanced projection", "高级原始投影"))
                    .ghost()
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.advanced = !this.advanced;
                        cx.notify();
                    })),
                busy,
                false,
                cx,
            ));
        if self.advanced {
            root = root.child(
                Textarea::new(&self.output)
                    .accessibility_id("observer-report")
                    .aria_label(text(
                        cx,
                        "Stored Claude usage and tool report",
                        "已存 Claude 用量和工具报告",
                    ))
                    .readonly(true)
                    .h(px(360.)),
            );
        }
        root.child(presentation::report(self, cx))
            .into_any_element()
    }
}

#[cfg(test)]
mod tests;
