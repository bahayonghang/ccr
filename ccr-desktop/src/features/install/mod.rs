//! Native installation and explicit companion-tool management.
mod tools;
use crate::{
    locales::text,
    runtime::{
        Services,
        install::{Error, InstallRequests, Query, Snapshot},
    },
    shell::semantics,
    theme,
};
use ccr_app::runtime::{
    Completion, EnvironmentId, Platform, RequestKey, TaskEvent, TaskSubscription,
};
use ccr_cli::services::install_types::{
    DetectionResult, DurationClass, InstallEvent, PlanId, PlanOutcome, ProgressStage,
    UnsupportedReason,
};
use gpui_kit::base::TestSupportExt;
use gpui_kit::component::{
    ActiveTheme, FocusTrapElement,
    button::{Button, ButtonVariants},
};
use gpui_kit::{
    AppContext, Context, Entity, FocusHandle, InteractiveElement, IntoElement, ParentElement,
    Render, StatefulInteractiveElement, Styled, Task, Window, div, px,
};
use std::sync::Arc;

pub struct InstallView {
    requests: InstallRequests,
    tools: Entity<tools::ToolsView>,
    snapshot: Arc<Snapshot>,
    observer: Option<Task<()>>,
    task: Option<Task<()>>,
    generation: u64,
    request_id: u64,
    loading: bool,
    error: Option<&'static str>,
    decision: Option<PlanId>,
    decision_generation: u64,
    decision_entry_pending: bool,
    decision_focus: FocusHandle,
    return_focus: Option<FocusHandle>,
    focus: FocusHandle,
}
impl InstallView {
    pub fn new(services: Services, _: &mut Window, cx: &mut Context<Self>) -> Self {
        let tools = cx.new(|cx| tools::ToolsView::new(&services, cx));
        let requests = InstallRequests::new(&services);
        Self {
            snapshot: requests.snapshot(),
            requests,
            tools,
            observer: None,
            task: None,
            generation: 0,
            request_id: 0,
            loading: false,
            error: None,
            decision: None,
            decision_generation: 0,
            decision_entry_pending: false,
            decision_focus: cx.focus_handle().tab_stop(false),
            return_focus: None,
            focus: cx.focus_handle().tab_stop(false),
        }
    }
    pub fn focus_main(&self, window: &mut Window, cx: &mut Context<Self>) {
        window.focus(&self.focus, cx);
        window.focus_next(cx);
    }
    pub fn refresh(&mut self, cx: &mut Context<Self>) {
        self.tools.update(cx, |this, cx| this.refresh(cx));
        if self.observer.is_none() {
            let mut updates = self.requests.observe();
            self.snapshot = updates.borrow_and_update().clone();
            let generation = self.generation;
            self.observer = Some(cx.spawn(async move |weak, cx| {
                while updates.changed().await.is_ok() {
                    let snapshot = updates.borrow_and_update().clone();
                    if weak
                        .update(cx, |this, cx| {
                            if this.generation == generation {
                                this.snapshot = snapshot;
                                cx.notify();
                            }
                        })
                        .is_err()
                    {
                        break;
                    }
                }
            }));
        } else {
            self.snapshot = self.requests.snapshot();
        }
        cx.notify();
    }
    pub fn suspend(&mut self, cx: &mut Context<Self>) {
        self.tools.update(cx, |this, cx| this.suspend(cx));
        self.observer = None;
        self.task = None;
        self.generation += 1;
        self.loading = false;
        self.error = None;
        self.decision = None;
        self.decision_generation += 1;
        self.decision_entry_pending = false;
        self.return_focus = None;
        cx.notify();
    }
    fn key(&mut self) -> RequestKey {
        self.request_id += 1;
        RequestKey {
            environment: EnvironmentId::Local(
                if self.requests.is_fixture() {
                    "fixture"
                } else {
                    "local"
                }
                .into(),
            ),
            platform: Platform::Claude,
            generation: self.generation,
            request_id: self.request_id,
        }
    }
    fn receive(
        &mut self,
        mut subscription: TaskSubscription<Arc<Snapshot>, Error>,
        cx: &mut Context<Self>,
    ) {
        self.error = None;
        self.loading = true;
        let generation = self.generation;
        let request_id = self.request_id;
        self.task = Some(cx.spawn(async move |weak, cx| {
            while let Some(event) = subscription.next().await {
                if let TaskEvent::Terminal(result) = event {
                    let _ = weak.update(cx, |this, cx| {
                        if this.generation != generation || this.request_id != request_id {
                            return;
                        }
                        this.loading = false;
                        this.snapshot = this.requests.snapshot();
                        this.error = match result.as_ref() {
                            Completion::Success(_) => None,
                            Completion::Failed(error) | Completion::CleanupFailed(error) => {
                                Some(error.message())
                            }
                            Completion::Cancelled | Completion::WorkerFailed => {
                                Some(Error::Interrupted.message())
                            }
                        };
                        cx.notify();
                    });
                    break;
                }
            }
        }));
        self.refresh(cx);
    }
    fn query(&mut self, query: Query, cx: &mut Context<Self>) {
        if self.loading || self.snapshot.busy() || self.decision.is_some() {
            return;
        }
        let key = self.key();
        match self.requests.query(key, query) {
            Ok(subscription) => self.receive(subscription, cx),
            Err(_) => {
                self.error = Some(Error::Operation.message());
                cx.notify();
            }
        }
    }
    fn decide(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.loading || self.snapshot.busy() || self.decision.is_some() {
            return;
        }
        let Some(PlanOutcome::Plan(plan)) = &self.snapshot.plan else {
            return;
        };
        self.decision = Some(plan.plan_id);
        self.return_focus = window.focused(cx);
        self.decision_generation += 1;
        self.decision_entry_pending = true;
        cx.notify();
    }
    fn dismiss(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.decision = None;
        self.decision_generation += 1;
        self.decision_entry_pending = false;
        if let Some(focus) = self.return_focus.take() {
            window.focus(&focus, cx);
        }
        cx.notify();
    }
    fn execute(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(id) = self.decision else {
            return;
        };
        self.dismiss(window, cx);
        let key = self.key();
        match self.requests.execute(key, id, true) {
            Ok(subscription) => self.receive(subscription, cx),
            Err(error) => {
                self.error = Some(error.message());
                cx.notify();
            }
        }
    }
    fn cancel(&mut self, cx: &mut Context<Self>) {
        if !self.snapshot.busy() || self.snapshot.cancel_requested {
            return;
        }
        let key = self.key();
        match self.requests.cancel(key) {
            Ok(snapshot) => {
                self.snapshot = snapshot;
                self.loading = false;
                self.task = None;
            }
            Err(error) => self.error = Some(error.message()),
        }
        cx.notify();
    }
}
impl Render for InstallView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let blocked = self.loading || self.snapshot.busy() || self.decision.is_some();
        let fixture = self.requests.is_fixture();
        let state = if self.snapshot.cancel_requested && self.snapshot.busy() {
            "Cancellation requested; waiting for cleanup / 已请求取消，等待清理"
        } else if self.snapshot.queued {
            "Queued / 等待执行"
        } else if self.snapshot.running {
            "Running / 正在安装"
        } else {
            match &self.snapshot.recent.terminal {
                Some(InstallEvent::Succeeded { .. }) => "Succeeded / 安装成功",
                Some(InstallEvent::Failed { .. }) => "Failed / 安装失败",
                Some(InstallEvent::Cancelled { .. }) => "Cancelled after cleanup / 清理后已取消",
                _ => "No active installation / 当前无安装任务",
            }
        };
        let mut root = div().id("native-install").test_support().track_focus(&self.focus).tab_stop(false)
            .flex().flex_col().size_full().overflow_y_scroll().p(px(theme::CONTENT_PADDING)).gap(px(theme::SECTION_GAP))
            .bg(cx.theme().background).text_color(cx.theme().foreground)
            .on_key_down(cx.listener(|this, event: &gpui_kit::KeyDownEvent, window, cx| {
                if event.keystroke.key == "escape" && this.decision.is_some() { this.dismiss(window, cx); cx.stop_propagation(); }
            }))
            .child(self.tools.clone())
            .child(div().text_size(px(theme::TITLE_SIZE)).child(text(cx, "Install llmusage", "安装 llmusage")))
            .child(text(cx, "Check the host, review a short-lived plan, then explicitly confirm installation for the current user.", "检测主机并核对短期安装计划，确认后为当前用户安装。"))
            .child(div().id("install-status").test_support().child(crate::locales::localized_line(cx, state)));
        if fixture {
            root = root.child(div().text_color(cx.theme().warning).child(
                crate::locales::localized_line(cx, Error::Unsupported.message()),
            ));
        }
        if self.loading && !self.snapshot.busy() {
            root = root.child(text(
                cx,
                "Checking or preparing installation…",
                "正在检测或准备安装…",
            ));
        }
        root = root.child(
            div()
                .flex()
                .flex_wrap()
                .gap(px(theme::CONTROL_GAP))
                .child(semantics::button(
                    Button::new("install-check")
                        .label(text(cx, "Check host", "检测主机"))
                        .ghost()
                        .on_click(cx.listener(|this, _, _, cx| this.query(Query::Check, cx))),
                    blocked || fixture,
                    false,
                    cx,
                ))
                .child(semantics::button(
                    Button::new("install-plan")
                        .label(text(cx, "Create plan", "生成计划"))
                        .ghost()
                        .on_click(cx.listener(|this, _, _, cx| this.query(Query::Plan, cx))),
                    blocked || fixture || self.snapshot.detection.is_none(),
                    false,
                    cx,
                ))
                .child(semantics::button(
                    Button::new("install-review")
                        .label(text(cx, "Review and install", "核对并安装"))
                        .on_click(cx.listener(|this, _, window, cx| this.decide(window, cx))),
                    blocked || fixture || !matches!(self.snapshot.plan, Some(PlanOutcome::Plan(_))),
                    false,
                    cx,
                ))
                .child(semantics::button(
                    Button::new("install-cancel")
                        .label(text(cx, "Request cancellation", "请求取消"))
                        .ghost()
                        .on_click(cx.listener(|this, _, _, cx| this.cancel(cx))),
                    !self.snapshot.busy() || self.snapshot.cancel_requested,
                    false,
                    cx,
                ))
                .child(semantics::button(
                    Button::new("install-manual")
                        .label(text(cx, "Manual instructions", "手动安装说明"))
                        .ghost()
                        .on_click(cx.listener(|this, _, _, cx| this.query(Query::Manual, cx))),
                    blocked || fixture,
                    false,
                    cx,
                )),
        );
        if let Some(detection) = &self.snapshot.detection {
            root = root.child(match detection {
                DetectionResult::Available { version, .. } => {
                    let version_label =
                        version
                            .as_deref()
                            .unwrap_or(text(cx, "version unknown", "版本未知"));
                    format!(
                        "{} · {version_label}",
                        text(cx, "llmusage available", "已安装")
                    )
                }
                DetectionResult::Absent { .. } => {
                    text(cx, "llmusage unavailable", "尚未安装").to_string()
                }
            });
        }
        if let Some(capabilities) = &self.snapshot.capabilities {
            root = root.child(format!(
                "{} · cargo={} · homebrew={} · scoop={} · winget={}",
                capabilities.platform,
                capabilities.has_cargo,
                capabilities.has_homebrew,
                capabilities.has_scoop,
                capabilities.has_winget
            ));
        }
        if let Some(plan) = &self.snapshot.plan {
            root = root.child(match plan {
                PlanOutcome::Plan(plan) => {
                    let expires = i64::try_from(plan.expires_at_ms)
                        .ok()
                        .and_then(chrono::DateTime::from_timestamp_millis)
                        .map(|date| {
                            date.with_timezone(&chrono::Local)
                                .format("%Y-%m-%d %H:%M:%S %Z")
                                .to_string()
                        })
                        .unwrap_or_else(|| text(cx, "Unknown expiry", "有效期未知").into());
                    let duration = match plan.duration_class {
                        DurationClass::Fast => text(cx, "Usually brief", "通常很快"),
                        DurationClass::Medium => {
                            text(cx, "May take a few minutes", "可能需要几分钟")
                        }
                        DurationClass::Slow => {
                            text(cx, "May take several minutes", "可能需要较长时间")
                        }
                    };
                    let elevation = if plan.elevation_required {
                        text(cx, "Administrator permission required", "需要管理员权限")
                    } else {
                        text(cx, "No administrator permission required", "无需管理员权限")
                    };
                    format!(
                        "{} / {} · {duration} · {elevation}\n{}: {expires}\n{}",
                        plan.platform,
                        plan.package_manager,
                        text(cx, "Plan expires", "计划有效期至"),
                        plan.expected_effects.join("\n")
                    )
                }
                PlanOutcome::Unsupported { reason } => match reason {
                    UnsupportedReason::NoPackageManager => text(
                        cx,
                        "Automatic installation unavailable: no supported package manager.",
                        "无法自动安装：未发现支持的软件包管理器。",
                    ),
                    UnsupportedReason::ElevationRequired => text(
                        cx,
                        "Automatic installation unavailable: administrator permission required.",
                        "无法自动安装：需要管理员权限。",
                    ),
                }
                .into(),
            });
        }
        if self.decision.is_some() {
            if std::mem::take(&mut self.decision_entry_pending) {
                let generation = self.decision_generation;
                let weak = cx.weak_entity();
                window.on_next_frame(move |window, cx| {
                    let _ = weak.update(cx, |this, cx| {
                        if this.decision.is_some() && this.decision_generation == generation {
                            window.focus(&this.decision_focus, cx);
                            window.focus_next(cx);
                        }
                    });
                });
            }
            root = root.child(div().id("install-confirmation").test_support().role(gpui_kit::Role::AlertDialog).aria_label("Confirm llmusage installation")
                .child(text(cx,"This installs llmusage for the current user using the package manager in the plan and may download packages. Continue?","将按计划使用软件包管理器为当前用户安装 llmusage，并可能下载软件包。是否继续？")))
                .child(div().tab_group().tab_stop(false).flex().gap(px(theme::CONTROL_GAP))
                    .child(semantics::button(Button::new("install-decision-cancel").label(text(cx,"Cancel","取消")).ghost().on_click(cx.listener(|this,_,window,cx| this.dismiss(window,cx))),false,false,cx))
                    .child(semantics::button(Button::new("install-decision-confirm").label(text(cx,"Confirm installation","确认安装")).on_click(cx.listener(|this,_,window,cx| this.execute(window,cx))),false,false,cx))
                    .focus_trap("install-decision-trap",&self.decision_focus));
        }
        if let Some(error) = self.error.or(self.snapshot.error.map(Error::message)) {
            root = root.child(
                div()
                    .text_color(cx.theme().danger)
                    .child(crate::locales::localized_line(cx, error)),
            );
        }
        if let Some(stage) = self.snapshot.stage {
            root = root.child(match stage {
                ProgressStage::Resolving => text(cx, "Resolving packages", "解析软件包"),
                ProgressStage::Downloading => text(cx, "Downloading", "正在下载"),
                ProgressStage::Compiling => text(cx, "Compiling", "正在编译"),
                ProgressStage::Installing => text(cx, "Installing", "正在安装"),
                ProgressStage::Finalizing => text(cx, "Finalizing", "正在完成安装"),
            });
        }
        let mut output = div()
            .id("install-output")
            .test_support()
            .flex()
            .flex_col()
            .flex_1()
            .overflow_y_scroll()
            .gap_1();
        for (index, event) in self.snapshot.recent.logs.iter().enumerate() {
            if let InstallEvent::Log { stream, line, .. } = event {
                output = output.child(
                    div()
                        .id(format!("install-log-{index}"))
                        .test_support()
                        .child(format!("{stream:?}: {line}")),
                );
            }
        }
        if let Some(manual) = &self.snapshot.manual {
            output = output.child(manual.docs_url.clone());
            for (index, entry) in manual.entries.iter().enumerate() {
                let command = entry.command_line.clone();
                output = output
                    .child(format!(
                        "{} · {}\n{}\n{}",
                        entry.platform,
                        entry.title,
                        entry.command_line,
                        entry.notes.as_deref().unwrap_or("")
                    ))
                    .child(semantics::button(
                        Button::new(format!("install-manual-copy-{index}"))
                            .label(text(cx, "Copy command", "复制命令"))
                            .ghost()
                            .on_click(cx.listener(move |_, _, _, cx| {
                                cx.write_to_clipboard(gpui_kit::ClipboardItem::new_string(
                                    command.clone(),
                                ))
                            })),
                        false,
                        false,
                        cx,
                    ));
            }
        }
        root.child(output)
    }
}

#[cfg(test)]
mod tests;
