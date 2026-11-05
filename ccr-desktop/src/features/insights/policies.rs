//! Budget/custom-pricing consumer; persistence and validation remain shared domain policy.
use super::*;
use crate::runtime::insights::policies::PolicyCommand;
use gpui_kit::component::{
    FocusTrapElement,
    input::{Textarea, TextareaState},
};
use serde_json::Value;

pub(super) struct PoliciesView {
    requests: InsightsRequests,
    model: Entity<InputState>,
    draft: Entity<TextareaState>,
    output: Entity<TextareaState>,
    pending_output: Option<String>,
    generation: u64,
    gate: ResponseGate,
    task: Option<Task<()>>,
    busy: bool,
    message: String,
    decision: Option<PolicyCommand>,
    decision_label: String,
    entry_pending: bool,
    decision_generation: u64,
    confirmation_focus: FocusHandle,
    return_focus: Option<FocusHandle>,
}
impl PoliciesView {
    pub fn new(requests: InsightsRequests, window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            requests,
            model: cx.new(|cx| InputState::new(window, cx)),
            draft: cx.new(|cx| TextareaState::new(window, cx)),
            output: cx.new(|cx| TextareaState::new(window, cx)),
            pending_output: None,
            generation: 0,
            gate: ResponseGate::default(),
            task: None,
            busy: false,
            message: String::new(),
            decision: None,
            decision_label: String::new(),
            entry_pending: false,
            decision_generation: 0,
            confirmation_focus: cx.focus_handle().tab_stop(false),
            return_focus: None,
        }
    }
    pub fn suspend(&mut self, cx: &mut Context<Self>) {
        self.task = None;
        self.gate.clear();
        self.generation += 1;
        self.busy = false;
        self.decision = None;
        self.entry_pending = false;
        self.return_focus = None;
        cx.notify();
    }
    pub fn load(&mut self, command: PolicyCommand, cx: &mut Context<Self>) {
        if self.busy || self.decision.is_some() {
            return;
        }
        self.generation += 1;
        let key = RequestKey {
            environment: EnvironmentId::Local("insights-explicit".into()),
            platform: Platform::Claude,
            generation: self.generation,
            request_id: self.generation,
        };
        self.gate.replace(key.clone());
        match self.requests.policy(key.clone(), command) {
            Ok(mut subscription) => {
                self.busy = true;
                self.message = "Running on background worker / 正在后台执行".into();
                self.task = Some(cx.spawn(async move |weak, cx| {
                    while let Some(event) = subscription.next().await {
                        if let TaskEvent::Terminal(result) = event {
                            let _ = weak.update(cx, |this, cx| {
                                if !this.gate.accepts(&key) { return; }
                                this.busy = false;
                                match result.as_ref() {
                                    Completion::Success(value) => {
                                        this.pending_output = serde_json::to_string_pretty(value).ok();
                                        this.message = "Operation completed. Budget updates persist supplied fields sequentially; custom pricing does not reprice llmusage. / 操作完成。预算逐字段保存；自定义价格不改写 llmusage 用量价格。".into();
                                    }
                                    Completion::Failed(error) | Completion::CleanupFailed(error) => {
                                        this.message = format!("{} Budget changes may have persisted earlier fields; reload before retrying. / 预算可能已保存前面的字段，请先重读再重试。", error.message());
                                    }
                                    Completion::Cancelled => this.message = "Operation did not start / 操作尚未开始".into(),
                                    Completion::WorkerFailed => this.message = "Worker stopped without a result. Reload before retrying. / 后台任务未返回结果，请重读后再重试。".into(),
                                }
                                cx.notify();
                            });
                            break;
                        }
                    }
                }));
            }
            Err(_) => {
                self.message =
                    "Operation queue unavailable. Retry. / 操作队列不可用，请重试。".into()
            }
        }
        cx.notify();
    }
    fn save(&mut self, pricing: bool, cx: &mut Context<Self>) {
        let Ok(value) = serde_json::from_str::<Value>(&self.draft.read(cx).value()) else {
            self.message =
                "Enter valid JSON. Draft retained. / 请输入有效 JSON，草稿已保留。".into();
            cx.notify();
            return;
        };
        if !value.is_object() {
            self.message =
                "Enter a JSON object. Draft retained. / 请输入 JSON 对象，草稿已保留。".into();
            cx.notify();
            return;
        }
        let model = self.model.read(cx).value().trim().to_string();
        if pricing && model.is_empty() {
            self.message = "A model ID is required / 请填写模型 ID".into();
            cx.notify();
            return;
        }
        self.load(
            if pricing {
                PolicyCommand::SetPricing {
                    model,
                    pricing: value,
                }
            } else {
                PolicyCommand::UpdateBudget(value)
            },
            cx,
        );
    }
    fn decide(&mut self, reset: bool, window: &mut Window, cx: &mut Context<Self>) {
        if self.busy || self.decision.is_some() {
            return;
        }
        let model = self.model.read(cx).value().trim().to_string();
        if !reset && model.is_empty() {
            self.message = "A model ID is required / 请填写模型 ID".into();
            cx.notify();
            return;
        }
        self.decision_label = if reset {
            "Reset all custom pricing to built-in defaults? / 将全部自定义价格重置为内置默认值？"
                .into()
        } else {
            format!("Remove custom price for {model}? / 删除此模型的自定义价格？")
        };
        self.decision = Some(if reset {
            PolicyCommand::ResetPricing
        } else {
            PolicyCommand::RemovePricing(model)
        });
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
impl Render for PoliciesView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if let Some(value) = self.pending_output.take() {
            self.output
                .update(cx, |state, cx| state.set_value(value, window, cx));
        }
        let mut root = div()
            .id("insights-policies")
            .test_support()
            .flex()
            .flex_col()
            .min_h_0()
            .flex_1()
            .gap(px(theme::CONTROL_GAP))
            .overflow_y_scroll()
            .on_key_down(
                cx.listener(|this, event: &gpui_kit::KeyDownEvent, window, cx| {
                    if event.keystroke.key == "escape" && this.decision.is_some() {
                        this.dismiss(window, cx);
                        cx.stop_propagation();
                    }
                }),
            );
        if self.decision.is_some() {
            if std::mem::take(&mut self.entry_pending) {
                let generation = self.decision_generation;
                let view = cx.weak_entity();
                window.on_next_frame(move |window, cx| {
                    let _ = view.update(cx, |this, cx| {
                        if this.decision.is_some() && this.decision_generation == generation {
                            window.focus(&this.confirmation_focus, cx);
                            window.focus_next(cx);
                        }
                    });
                });
            }
            return root
                .child(
                    div()
                        .id("policy-confirmation")
                        .test_support()
                        .role(gpui_kit::Role::AlertDialog)
                        .aria_label(self.decision_label.clone())
                        .child(self.decision_label.clone()),
                )
                .child(
                    div()
                        .tab_group()
                        .tab_stop(false)
                        .flex()
                        .gap(px(theme::CONTROL_GAP))
                        .child(semantics::button(
                            Button::new("policy-cancel")
                                .label("Cancel / 取消")
                                .ghost()
                                .on_click(
                                    cx.listener(|this, _, window, cx| this.dismiss(window, cx)),
                                ),
                            false,
                            false,
                            cx,
                        ))
                        .child(semantics::button(
                            Button::new("policy-confirm")
                                .label("Confirm / 确认")
                                .on_click(cx.listener(|this, _, window, cx| {
                                    let command = this.decision.take();
                                    this.dismiss(window, cx);
                                    if let Some(command) = command {
                                        this.load(command, cx);
                                    }
                                })),
                            false,
                            false,
                            cx,
                        ))
                        .focus_trap("policy-decision-trap", &self.confirmation_focus),
                );
        }
        root = root.child(text(cx, "Budgets use the legacy CCR cost tracker; custom prices do not modify stored llmusage pricing. Reads may create the existing stats directory. Budget field updates retain sequential persistence and may partially succeed.", "预算使用 CCR 旧成本追踪器；自定义价格不改写 llmusage 已存价格。读取可能创建成本目录；预算按字段顺序保存，可能部分成功。"));
        root = root.child(
            div()
                .flex()
                .flex_wrap()
                .gap(px(theme::CONTROL_GAP))
                .child(semantics::button(
                    Button::new("policy-load-budget")
                        .label("Load budgets / 读取预算")
                        .on_click(
                            cx.listener(|this, _, _, cx| this.load(PolicyCommand::Budgets, cx)),
                        ),
                    self.busy,
                    false,
                    cx,
                ))
                .child(semantics::button(
                    Button::new("policy-load-pricing")
                        .label("List custom pricing / 读取自定义价格")
                        .ghost()
                        .on_click(
                            cx.listener(|this, _, _, cx| this.load(PolicyCommand::Pricing, cx)),
                        ),
                    self.busy,
                    false,
                    cx,
                )),
        );
        root = root.child(Input::new(&self.model).id("policy-model").aria_label("Pricing model ID / 价格模型 ID"))
            .child(text(cx, "Budget patch: enabled, dailyLimit, weeklyLimit, monthlyLimit, warnAtPercent; null clears a limit. Pricing: model, input_price, output_price, cache_read_price, cache_write_price (USD per million tokens).", "预算变更字段：enabled、dailyLimit、weeklyLimit、monthlyLimit、warnAtPercent；null 清除限额。模型价格单位为美元/百万 Token，使用现有价格字段。"))
            .child(Textarea::new(&self.draft).accessibility_id("policy-draft").aria_label("Budget patch or pricing JSON / 预算变更或价格 JSON").h(px(110.)).readonly(self.busy));
        root = root.child(
            div()
                .flex()
                .flex_wrap()
                .gap(px(theme::CONTROL_GAP))
                .child(semantics::button(
                    Button::new("policy-save-budget")
                        .label("Save budget patch / 保存预算变更")
                        .on_click(cx.listener(|this, _, _, cx| this.save(false, cx))),
                    self.busy,
                    false,
                    cx,
                ))
                .child(semantics::button(
                    Button::new("policy-save-pricing")
                        .label("Set model pricing / 保存模型价格")
                        .ghost()
                        .on_click(cx.listener(|this, _, _, cx| this.save(true, cx))),
                    self.busy,
                    false,
                    cx,
                ))
                .child(semantics::button(
                    Button::new("policy-remove-pricing")
                        .label("Remove model pricing / 删除模型价格")
                        .ghost()
                        .on_click(
                            cx.listener(|this, _, window, cx| this.decide(false, window, cx)),
                        ),
                    self.busy,
                    false,
                    cx,
                ))
                .child(semantics::button(
                    Button::new("policy-reset-pricing")
                        .label("Reset pricing / 重置价格")
                        .ghost()
                        .on_click(cx.listener(|this, _, window, cx| this.decide(true, window, cx))),
                    self.busy,
                    false,
                    cx,
                )),
        );
        root.child(
            div()
                .id("policy-status")
                .test_support()
                .role(gpui_kit::Role::Status)
                .child(self.message.clone()),
        )
        .child(
            Textarea::new(&self.output)
                .accessibility_id("policy-output")
                .aria_label("Exact policy result / 策略精确结果")
                .readonly(true)
                .h(px(180.)),
        )
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::runtime::{ConfigSource, insights::InsightsPaths};
    use ccr_app::runtime::RuntimeOwner;
    use gpui_kit::{
        TestAppContext, WindowOptions,
        test::{TestAppContextExt, TestWindowExt},
    };
    use std::time::Duration;

    #[gpui_kit::test]
    async fn native_policy_controls_persist_budget_and_pricing_and_cancel_destructive_reset(
        cx: &mut TestAppContext,
    ) {
        cx.executor().allow_parking();
        let root = tempfile::tempdir().unwrap();
        let owner = RuntimeOwner::new().unwrap();
        let services = Services::new(
            owner.handle(),
            ConfigSource::Fixture {
                profiles: root.path().join("profiles.toml"),
                registry: root.path().join("config.toml"),
                delay: Duration::ZERO,
            },
        )
        .with_insights_paths(InsightsPaths {
            usage_root: root.path().join("llmusage"),
            archive_db: root.path().join("archive.db"),
            provider_home: root.path().join("home"),
        });
        let (handle, view) = cx.update(|cx| {
            gpui_kit::init(cx);
            gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
                cx.new(|cx| PoliciesView::new(InsightsRequests::new(&services), window, cx))
            })
            .unwrap()
        });
        cx.update_window(handle, |_, window, cx| {
            view.update(cx, |this, cx| {
                this.draft.update(cx, |state, cx| {
                    state.set_value("{\"dailyLimit\":12.5,\"weeklyLimit\":30}", window, cx)
                })
            });
            window.render_frame(cx);
            window.click("policy-save-budget", cx);
        })
        .unwrap();
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| !view.read(cx).busy)
            .await;
        let budget = root.path().join("home/.claude/budget.toml");
        assert_eq!(
            ccr_store::BudgetManager::new(budget.clone())
                .unwrap()
                .get_config()
                .daily_limit,
            Some(12.5)
        );
        let before = std::fs::read(&budget).unwrap();
        cx.update_window(handle, |_, window, cx| {
            view.update(cx, |this, cx| this.draft.update(cx, |state, cx| state.set_value("{invalid", window, cx)));
            window.render_frame(cx); window.click("policy-save-budget", cx);
            assert!(!view.read(cx).busy);
            assert_eq!(view.read(cx).draft.read(cx).value().as_ref(), "{invalid");
            view.update(cx, |this, cx| {
                this.model.update(cx, |state, cx| state.set_value("native-fixture", window, cx));
                this.draft.update(cx, |state, cx| state.set_value("{\"model\":\"native-fixture\",\"input_price\":1.23456789,\"output_price\":2.5}", window, cx));
            });
            window.render_frame(cx); window.click("policy-save-pricing", cx);
        }).unwrap();
        assert_eq!(std::fs::read(budget).unwrap(), before);
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| !view.read(cx).busy)
            .await;
        let pricing = root.path().join("home/.claude/pricing.toml");
        assert_eq!(
            ccr_store::PricingManager::new(pricing.clone())
                .unwrap()
                .get_pricing("native-fixture")
                .unwrap()
                .input_price,
            1.23456789
        );
        let before = std::fs::read(&pricing).unwrap();
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            window.click("policy-reset-pricing", cx);
            window.simulate_next_frame(cx);
            window.render_frame(cx);
            assert_eq!(window.find("policy-cancel").focused(), Some(true));
            window.press("tab", cx);
            assert_eq!(window.find("policy-confirm").focused(), Some(true));
            window.press("escape", cx);
            assert!(view.read(cx).decision.is_none());
            window.remove_window();
        })
        .unwrap();
        assert_eq!(std::fs::read(pricing).unwrap(), before);
        assert!(!root.path().join("llmusage").exists());
        owner.shutdown();
    }
}
