//! Saved check-in accounts. Balances appear only when a local row stores them.
mod operations;
use crate::runtime::checkin::OperationState;
use operations::{Confirmation, Editor};
use std::sync::Arc;

use crate::{
    locales::{is_chinese, service_error, text},
    runtime::{
        Services,
        checkin::{CheckinAccountRow, CheckinError, CheckinSnapshot, StoredBalance, StoredNumbers},
    },
    shell::semantics,
    theme,
};
use ccr_app::runtime::{Completion, EnvironmentId, Platform, RequestKey, ResponseGate, TaskEvent};
use gpui_kit::base::TestSupportExt;
use gpui_kit::component::{
    Selectable,
    button::{Button, ButtonVariants},
};
use gpui_kit::{
    App, Context, FocusHandle, InteractiveElement, IntoElement, ParentElement, Render,
    SharedString, StatefulInteractiveElement, Styled, Task, Window, div, px,
};

pub struct CheckinView {
    services: Services,
    snapshot: Option<CheckinSnapshot>,
    selected: Option<String>,
    generation: u64,
    gate: ResponseGate,
    task: Option<Task<()>>,
    busy: bool,
    message: Option<&'static str>,
    focus: FocusHandle,
    operation: Arc<OperationState>,
    observation: Option<Task<()>>,
    editor: Option<Editor>,
    confirmation: Option<Confirmation>,
    catalog: bool,
}

impl CheckinView {
    pub fn new(services: Services, _window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            operation: services.checkin_operation_state(),
            observation: None,
            editor: None,
            confirmation: None,
            catalog: false,
            services,
            snapshot: None,
            selected: None,
            generation: 0,
            gate: ResponseGate::default(),
            task: None,
            busy: false,
            message: None,
            focus: cx.focus_handle().tab_stop(false),
        }
    }

    pub fn focus_main(&self, window: &mut Window, cx: &mut Context<Self>) {
        window.focus(&self.focus, cx);
        window.focus_next(cx);
    }

    pub fn suspend(&mut self, cx: &mut Context<Self>) {
        self.task = None;
        self.observation = None;
        self.editor = None;
        self.confirmation = None;
        self.gate.clear();
        self.generation += 1;
        self.busy = false;
        cx.notify();
    }

    pub fn refresh(&mut self, cx: &mut Context<Self>) {
        self.observe(cx);
        if self.busy {
            return;
        }
        self.generation += 1;
        let key = RequestKey {
            environment: EnvironmentId::Local(
                if self.services.is_fixture() {
                    "fixture"
                } else {
                    "local"
                }
                .into(),
            ),
            platform: Platform::Claude,
            generation: self.generation,
            request_id: self.generation,
        };
        self.gate.replace(key.clone());
        self.busy = true;
        self.message = None;
        self.snapshot = None;
        match self.services.checkin_request(key.clone()) {
            Ok(mut subscription) => {
                self.task = Some(cx.spawn(async move |weak, cx| {
                    while let Some(event) = subscription.next().await {
                        if let TaskEvent::Terminal(result) = event {
                            let _ = weak.update(cx, |this, cx| {
                                if !this.gate.accepts(&key) {
                                    return;
                                }
                                this.busy = false;
                                match result.as_ref() {
                                    Completion::Success(snapshot) => this.accept(snapshot.clone()),
                                    Completion::Failed(error) | Completion::CleanupFailed(error) => {
                                        this.message = Some(error.message());
                                    }
                                    Completion::Cancelled => {
                                        this.message = Some(
                                            "The check-in read did not finish. / 签到读取没有完成",
                                        );
                                    }
                                    Completion::WorkerFailed => {
                                        this.message = Some(
                                            "The check-in reader stopped without a result. / 签到读取没有返回结果",
                                        );
                                    }
                                }
                                cx.notify();
                            });
                            break;
                        }
                    }
                }));
            }
            Err(_) => {
                self.busy = false;
                self.message = Some(CheckinError::Storage.message());
            }
        }
        cx.notify();
    }

    fn accept(&mut self, snapshot: CheckinSnapshot) {
        let selected = match &snapshot {
            CheckinSnapshot::Accounts(accounts) => self
                .selected
                .as_ref()
                .filter(|id| accounts.iter().any(|a| &a.id == *id))
                .cloned()
                .or_else(|| accounts.first().map(|account| account.id.clone())),
            CheckinSnapshot::NoDatabase => None,
        };
        self.snapshot = Some(snapshot);
        self.selected = selected;
    }

    fn rows(&self) -> &[CheckinAccountRow] {
        match self.snapshot.as_ref() {
            Some(CheckinSnapshot::Accounts(accounts)) => accounts,
            _ => &[],
        }
    }
}

fn balance_lines(cx: &App, balance: &StoredBalance) -> Vec<String> {
    match balance {
        StoredBalance::Missing => {
            vec![text(cx, "No stored balance.", "没有已保存的余额。").to_string()]
        }
        StoredBalance::Unusable => vec![
            text(
                cx,
                "A stored balance row could not be shown.",
                "有一条已保存的余额，但无法显示其中的数字。",
            )
            .to_string(),
        ],
        StoredBalance::Numbers(numbers) => vec![numbers_line(cx, numbers)],
    }
}

fn numbers_line(cx: &App, numbers: &StoredNumbers) -> String {
    let currency = numbers
        .currency
        .as_deref()
        .unwrap_or_else(|| text(cx, "currency not stored", "未保存货币"));
    if is_chinese(cx) {
        format!(
            "已保存剩余 {} {currency}。已保存总额 {}。已保存已用 {}。记录时间 {}。",
            numbers.remaining, numbers.total, numbers.used, numbers.recorded_at
        )
    } else {
        format!(
            "Stored remaining {} {currency}. Stored total {}. Stored used {}. Recorded at {}.",
            numbers.remaining, numbers.total, numbers.used, numbers.recorded_at
        )
    }
}

fn detail_lines(cx: &App, account: &CheckinAccountRow) -> Vec<String> {
    let mut lines = vec![account.name.clone()];
    lines.push(match account.provider_name.as_deref() {
        Some(name) => {
            if is_chinese(cx) {
                format!("提供商：{name}")
            } else {
                format!("Provider: {name}")
            }
        }
        None => text(cx, "Provider name is not stored.", "未保存提供商名称。").to_string(),
    });
    lines.push(
        if account.enabled {
            text(cx, "Enabled in the saved account.", "已保存的账户为启用。")
        } else {
            text(cx, "Disabled in the saved account.", "已保存的账户为停用。")
        }
        .to_string(),
    );
    lines.push(match account.last_checkin_at.as_deref() {
        Some(time) => {
            if is_chinese(cx) {
                format!("上次签到时间：{time}")
            } else {
                format!("Last check-in at {time}.")
            }
        }
        None => text(
            cx,
            "No check-in is recorded for this account.",
            "这个账户没有签到记录。",
        )
        .to_string(),
    });
    lines.extend(balance_lines(cx, &account.balance));
    lines.push(
        text(
            cx,
            "Reading this page does not contact the provider. Use the actions to check in or refresh balance.",
            "读取此页不会联系提供商。使用操作按钮执行签到或刷新余额。",
        )
        .to_string(),
    );
    lines
}

impl Render for CheckinView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut root = div()
            .id("checkin-page")
            .test_support()
            .track_focus(&self.focus)
            .tab_stop(false)
            .flex()
            .flex_col()
            .size_full()
            .overflow_y_scroll()
            .p(px(theme::CONTENT_PADDING))
            .gap(px(theme::SECTION_GAP))
            .child(
                div()
                    .id("checkin-heading")
                    .test_support()
                    .role(gpui_kit::Role::Heading)
                    .aria_level(1)
                    .text_size(px(theme::TITLE_SIZE))
                    .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                    .child(text(cx, "Check-ins", "签到管理")),
            )
            .child({
                let intro = text(
                    cx,
                    "Reading this page does not create a database or contact providers. Management and account actions write to the local check-in store.",
                    "读取此页不会创建数据库或联系提供商。管理及账户操作会写入本地签到存储。",
                );
                div()
                    .id("checkin-intro")
                    .test_support()
                    .aria_label(intro)
                    .child(intro)
            })
            .child(semantics::button(
                Button::new("checkin-read")
                    .label(if self.busy {
                        text(cx, "Reading", "读取中")
                    } else {
                        text(cx, "Read saved accounts", "读取已保存的账户")
                    })
                    .ghost()
                    .on_click(cx.listener(|this, _, _, cx| this.refresh(cx))),
                self.busy,
                self.busy,
                cx,
            ));
        if self.busy {
            root = root.child(
                div()
                    .id("checkin-status")
                    .test_support()
                    .role(gpui_kit::Role::Status)
                    .child(text(
                        cx,
                        "Reading saved check-in accounts.",
                        "正在读取已保存的签到账户。",
                    )),
            );
        }
        if let Some(message) = self.message {
            root = root.child(
                div()
                    .id("checkin-status")
                    .test_support()
                    .role(gpui_kit::Role::Status)
                    .child(service_error(cx, message)),
            );
        }
        if let Some(snapshot) = self.snapshot.clone() {
            match snapshot {
                CheckinSnapshot::NoDatabase => {
                    let empty = text(
                        cx,
                        "No local check-in database was found. Use management to create providers and accounts.",
                        "没有找到本地签到数据库。使用管理入口新增提供商和账户。",
                    );
                    root = root.child(
                        div()
                            .id("checkin-absent")
                            .test_support()
                            .aria_label(empty)
                            .child(empty),
                    );
                }
                CheckinSnapshot::Accounts(accounts) if accounts.is_empty() => {
                    let empty = text(
                        cx,
                        "The local database has no saved check-in accounts.",
                        "本地数据库没有已保存的签到账户。",
                    );
                    root = root.child(
                        div()
                            .id("checkin-empty")
                            .test_support()
                            .aria_label(empty)
                            .child(empty),
                    );
                }
                CheckinSnapshot::Accounts(_) => {
                    for (index, account) in self.rows().iter().enumerate() {
                        let selected = self.selected.as_deref() == Some(account.id.as_str());
                        let provider = account.provider_name.as_deref().unwrap_or("");
                        let label = if provider.is_empty() {
                            account.name.clone()
                        } else {
                            format!("{} · {provider}", account.name)
                        };
                        let clicked = account.id.clone();
                        root = root.child(
                            div().h(px(theme::ROW_HEIGHT)).flex().items_center().child(
                                semantics::button(
                                    Button::new(format!("checkin-account-{index}"))
                                        .label(label)
                                        .ghost()
                                        .selected(selected)
                                        .on_click(cx.listener(move |this, _, _, cx| {
                                            this.selected = Some(clicked.clone());
                                            cx.notify();
                                        })),
                                    false,
                                    false,
                                    cx,
                                ),
                            ),
                        );
                    }
                    if let Some(selected) = self.selected.clone()
                        && let Some(account) =
                            self.rows().iter().find(|account| account.id == selected)
                    {
                        let lines = detail_lines(cx, account);
                        let summary = lines.join(" ");
                        let mut detail = div()
                            .id("checkin-detail")
                            .test_support()
                            .aria_label(SharedString::from(summary))
                            .flex()
                            .flex_col()
                            .gap(px(theme::CONTROL_GAP));
                        for line in lines {
                            detail = detail.child(line);
                        }
                        root = root.child(detail);
                    }
                }
            }
        }
        root.child(self.render_operations(cx))
    }
}

#[cfg(test)]
mod tests;
