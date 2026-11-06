//! Codex auth-only process warning and fixed callback-port controls.
use super::*;
use crate::runtime::platforms::diagnostics::{
    Error, PortState, ProcessWarning, ReleaseReport, Report, Request,
};

enum Review {
    Save {
        session: Arc<PlatformSession>,
        name: String,
    },
    CancelOwned {
        plan: Option<Arc<ccr_cli::application::codex_oauth::NativeCleanupPlan>>,
    },
}
#[derive(Default)]
pub(super) struct ViewState {
    task: Option<Task<()>>,
    busy: bool,
    generation: u64,
    review: Option<Review>,
    warning: Option<Result<ProcessWarning, Error>>,
    port: Option<Result<PortState, Error>>,
    release: Option<Result<ReleaseReport, Error>>,
    cleanup_error: Option<Error>,
    entry_pending: bool,
    focus: Option<FocusHandle>,
    return_focus: Option<FocusHandle>,
}
#[derive(Clone, Copy)]
enum Intent {
    SaveWarning,
    Port,
    Start,
    PrepareCleanup,
    CancelOwned,
}
impl PlatformsView {
    fn diagnostic_key(&self) -> RequestKey {
        RequestKey {
            environment: EnvironmentId::Local(
                if self.services.is_fixture() {
                    "fixture"
                } else {
                    "local"
                }
                .into(),
            ),
            platform: self.platform,
            generation: self.generation,
            request_id: self.diagnostics.generation,
        }
    }
    fn request_diagnostic(&mut self, request: Request, intent: Intent, cx: &mut Context<Self>) {
        self.diagnostics.task = None;
        self.diagnostics.busy = true;
        self.diagnostics.generation += 1;
        let generation = self.generation;
        let decision = self.diagnostics.generation;
        match self
            .services
            .codex_auth_diagnostic(self.diagnostic_key(), request)
        {
            Ok(mut request) => {
                self.diagnostics.task = Some(cx.spawn(async move |weak, cx| {
                    while let Some(event) = request.next().await {
                        if let TaskEvent::Terminal(value) = event {
                            let _ = weak.update(cx, |this, cx| {
                                if this.generation != generation
                                    || this.diagnostics.generation != decision
                                    || this.platform != Platform::Codex
                                    || this.surface != Surface::Auth
                                {
                                    return;
                                }
                                this.diagnostics.busy = false;
                                let result = match value.as_ref() {
                                    Completion::Success(value) => Ok(value.clone()),
                                    Completion::Failed(error)
                                    | Completion::CleanupFailed(error) => Err(*error),
                                    _ => Err(Error::Unavailable),
                                };
                                this.apply_diagnostic(intent, result, cx);
                                cx.notify();
                            });
                            break;
                        }
                    }
                }))
            }
            Err(_) => {
                self.diagnostics.busy = false;
                self.apply_diagnostic(intent, Err(Error::Unavailable), cx);
            }
        }
        cx.notify();
    }
    fn apply_diagnostic(
        &mut self,
        intent: Intent,
        result: Result<Report, Error>,
        cx: &mut Context<Self>,
    ) {
        match intent {
            Intent::SaveWarning => {
                self.diagnostics.warning = Some(match result {
                    Ok(Report::Processes(value)) => Ok(value),
                    Err(error) => Err(error),
                    _ => Err(Error::Unavailable),
                })
            }
            Intent::Port | Intent::Start => {
                let state = match result {
                    Ok(Report::Port(value)) => Ok(value),
                    Err(error) => Err(error),
                    _ => Err(Error::Unavailable),
                };
                let start = matches!(intent, Intent::Start) && state == Ok(PortState::Available);
                self.diagnostics.port = Some(state);
                if start {
                    self.start_oauth(cx);
                }
            }
            Intent::CancelOwned => {
                self.diagnostics.release = Some(match result {
                    Ok(Report::Release(value)) => Ok(value),
                    Err(error) => Err(error),
                    _ => Err(Error::Unavailable),
                })
            }
            Intent::PrepareCleanup => match result {
                Ok(Report::CleanupPlan(prepared)) => {
                    self.diagnostics.cleanup_error = None;
                    if let Some(Review::CancelOwned { plan }) = &mut self.diagnostics.review {
                        *plan = Some(prepared);
                    }
                }
                Err(error) => self.diagnostics.cleanup_error = Some(error),
                _ => self.diagnostics.cleanup_error = Some(Error::Unavailable),
            },
        }
    }
    pub(super) fn review_codex_save(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.platform != Platform::Codex
            || self.surface != Surface::Auth
            || self.busy
            || self.diagnostics.busy
            || self.diagnostics.review.is_some()
            || self.services.platform_oauth_active()
        {
            return;
        }
        let Some(session) = self.session.clone() else {
            return;
        };
        self.diagnostics.warning = None;
        self.diagnostics.review = Some(Review::Save {
            session: session.clone(),
            name: self.name.read(cx).value().to_string(),
        });
        self.diagnostics.return_focus = window.focused(cx);
        self.diagnostics.entry_pending = true;
        self.request_diagnostic(
            Request::SaveProcesses(Box::new(session.roots.clone())),
            Intent::SaveWarning,
            cx,
        );
    }
    pub(super) fn preflight_oauth_start(&mut self, cx: &mut Context<Self>) {
        if self.platform != Platform::Codex
            || self.surface != Surface::Auth
            || self.diagnostics.busy
            || self.services.platform_oauth_active()
        {
            return;
        }
        self.request_diagnostic(Request::CallbackPort, Intent::Start, cx);
    }
    fn review_owned_cleanup(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.diagnostics.review.is_some() || self.diagnostics.busy {
            return;
        }
        let Some(session) = self.session.clone() else {
            return;
        };
        self.diagnostics.review = Some(Review::CancelOwned { plan: None });
        self.diagnostics.cleanup_error = None;
        self.diagnostics.entry_pending = true;
        self.diagnostics.return_focus = window.focused(cx);
        self.request_diagnostic(
            Request::PrepareCleanup(Box::new(session.roots.clone())),
            Intent::PrepareCleanup,
            cx,
        );
    }
    fn dismiss_diagnostic_review(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.diagnostics.review = None;
        self.diagnostics.entry_pending = false;
        self.diagnostics.generation += 1;
        self.diagnostics.task = None;
        self.diagnostics.busy = false;
        if let Some(focus) = self.diagnostics.return_focus.take() {
            window.focus(&focus, cx);
        }
        cx.notify();
    }
    fn confirm_diagnostic_review(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if matches!(
            self.diagnostics.review,
            Some(Review::CancelOwned { plan: None })
        ) {
            return;
        }
        let review = self.diagnostics.review.take();
        self.dismiss_diagnostic_review(window, cx);
        match review {
            Some(Review::Save { session, name }) => self.request(
                Command::Account {
                    session,
                    name,
                    action: AccountAction::Save,
                },
                cx,
            ),
            Some(Review::CancelOwned { plan: Some(plan) }) => self.request_diagnostic(
                Request::CancelOwnedListeners(plan),
                Intent::CancelOwned,
                cx,
            ),
            Some(Review::CancelOwned { plan: None }) => (),
            None => (),
        }
    }
    pub(super) fn diagnostic_blocked(&self) -> bool {
        self.diagnostics.busy || self.diagnostics.review.is_some()
    }
    pub(super) fn render_codex_diagnostics(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> gpui_kit::AnyElement {
        if self.diagnostics.review.is_some() {
            return self.render_diagnostic_review(window, cx);
        }
        div().id("codex-auth-diagnostics").test_support().flex().flex_col().gap(px(theme::CONTROL_GAP))
            .child(text(cx,"OAuth callback: 127.0.0.1:1455. Preflight is a snapshot; starting OAuth always performs the final bind.","OAuth 回调：127.0.0.1:1455。预检只是瞬时状态，开始 OAuth 时仍会最终绑定校验。"))
            .child(self.render_port_diagnostic(cx))
            .child(self.render_diagnostic_actions(cx)).into_any_element()
    }
    fn render_diagnostic_actions(&self, cx: &mut Context<Self>) -> gpui_kit::AnyElement {
        div()
            .flex()
            .flex_wrap()
            .gap(px(theme::CONTROL_GAP))
            .child(semantics::button(
                Button::new("codex-port-check")
                    .label(text(cx, "Check callback port", "检查回调端口"))
                    .ghost()
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.request_diagnostic(Request::CallbackPort, Intent::Port, cx)
                    })),
                self.diagnostics.busy,
                false,
                cx,
            ))
            .child(semantics::button(
                Button::new("codex-port-review-cleanup")
                    .label(text(
                        cx,
                        "Review owned-listener cleanup…",
                        "核对自有监听器清理…",
                    ))
                    .ghost()
                    .on_click(
                        cx.listener(|this, _, window, cx| this.review_owned_cleanup(window, cx)),
                    ),
                self.diagnostics.busy,
                false,
                cx,
            ))
            .into_any_element()
    }
    fn render_port_diagnostic(&self, cx: &mut Context<Self>) -> gpui_kit::AnyElement {
        let mut root = div()
            .id("codex-port-status")
            .test_support()
            .role(gpui_kit::Role::Status)
            .flex()
            .flex_col()
            .gap(px(theme::CONTROL_GAP));
        root = root.child(if self.diagnostics.busy {
            text(cx, "Diagnostic is running.", "诊断进行中。").to_string()
        } else {
            match &self.diagnostics.port {
                None => text(
                    cx,
                    "Callback port not checked; availability is unknown.",
                    "尚未检查回调端口，可用状态未知。",
                )
                .to_string(),
                Some(Ok(state)) => port_text(state, cx).to_string(),
                Some(Err(error)) => error.message().to_string(),
            }
        });
        if let Some(result) = &self.diagnostics.release {
            root = root.child(match result {
                Err(error) => error.message().to_string(),
                Ok(report) => format!(
                    "{} {}. {} {}. {} {}. {} {}",
                    text(
                        cx,
                        "Cancellation requested for CCR-owned handles:",
                        "已请求取消 CCR 自有句柄："
                    ),
                    report.cancel_requested,
                    text(cx, "Owned PIDs:", "自有 PID："),
                    pid_text(&report.owned),
                    text(
                        cx,
                        "Unknown PIDs; close manually:",
                        "未知 PID，请手动关闭："
                    ),
                    pid_text(&report.unknown),
                    text(
                        cx,
                        "A cancellation request does not prove exit or port release.",
                        "请求取消不代表进程已退出或端口已释放。"
                    ),
                    port_text(&report.port, cx)
                ),
            });
        }
        root.into_any_element()
    }
    fn render_diagnostic_review(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> gpui_kit::AnyElement {
        let focus = self
            .diagnostics
            .focus
            .get_or_insert_with(|| cx.focus_handle().tab_stop(false))
            .clone();
        if std::mem::take(&mut self.diagnostics.entry_pending) {
            let decision = self.diagnostics.generation;
            let generation = self.generation;
            let weak = cx.weak_entity();
            window.on_next_frame(move |window, cx| {
                let _ = weak.update(cx, |this, cx| {
                    if this.generation == generation
                        && this.diagnostics.generation == decision
                        && this.diagnostics.review.is_some()
                        && let Some(focus) = &this.diagnostics.focus
                    {
                        window.focus(focus, cx);
                        window.focus_next(cx);
                    }
                });
            });
        }
        let mut root = div()
            .id("codex-diagnostic-review")
            .test_support()
            .role(gpui_kit::Role::AlertDialog)
            .track_focus(&focus)
            .tab_stop(false)
            .flex()
            .flex_col()
            .gap(px(theme::CONTROL_GAP))
            .on_key_down(
                cx.listener(|this, event: &gpui_kit::KeyDownEvent, window, cx| {
                    if event.keystroke.key == "escape" {
                        this.dismiss_diagnostic_review(window, cx);
                        cx.stop_propagation();
                    }
                }),
            );
        match &self.diagnostics.review {
            Some(Review::Save { name, .. }) => {
                root = root
                    .child(format!(
                        "{} {name}",
                        text(
                            cx,
                            "Save current runtime as account:",
                            "保存当前运行为账户："
                        )
                    ))
                    .child(self.render_save_warning(cx));
            }
            Some(Review::CancelOwned { plan }) => {
                root=root.child(text(cx,"Request current CCR OAuth cleanup, then inspect only callback port 1455 and request cancellation of matching CCR-owned handles? Unknown processes are never killed or taken over. Saved accounts are unchanged.","请求清理当前 CCR OAuth，再核对固定回调端口 1455，仅请求取消匹配的 CCR 自有句柄？不会终止或接管未知进程，保留已保存账户。"));
                if plan.is_none() {
                    root=root.child(self.diagnostics.cleanup_error.map(Error::message).unwrap_or(text(cx,"Preparing captured cleanup consent; no cancellation requested yet.","正在准备指定范围的清理确认，尚未请求取消。")));
                }
            }
            None => (),
        }
        root.child(
            div()
                .tab_group()
                .tab_stop(false)
                .flex()
                .gap(px(theme::CONTROL_GAP))
                .child(semantics::button(
                    Button::new("codex-diagnostic-cancel")
                        .label(text(cx, "Cancel", "取消"))
                        .ghost()
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.dismiss_diagnostic_review(window, cx)
                        })),
                    false,
                    false,
                    cx,
                ))
                .child(semantics::button(
                    Button::new("codex-diagnostic-confirm")
                        .label(text(cx, "Confirm", "确认"))
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.confirm_diagnostic_review(window, cx)
                        })),
                    matches!(
                        self.diagnostics.review,
                        Some(Review::CancelOwned { plan: None })
                    ),
                    false,
                    cx,
                ))
                .focus_trap("codex-diagnostic-focus", &focus),
        )
        .into_any_element()
    }
    fn render_save_warning(&self, cx: &gpui_kit::App) -> gpui_kit::AnyElement {
        let message=match &self.diagnostics.warning {
            None=>text(cx,"Process warning not checked yet. This warning is non-blocking; saving does not stop Codex.","尚未完成进程检查。此警告不阻止保存，保存不会停止 Codex。").to_string(),
            Some(Err(error))=>error.message().to_string(),
            Some(Ok(warning)) if warning.pids.is_empty()=>text(cx,"The cached best-effort owner observed no matching PID; this does not prove that no Codex process is running.","领域服务的缓存尽力检查未观察到匹配 PID，不代表没有 Codex 进程运行。").to_string(),
            Some(Ok(warning))=>format!("{} {} (+{}). {}",text(cx,"Codex process PIDs observed:","观察到 Codex 进程 PID："),pid_text(&warning.pids),warning.additional,text(cx,"Close Codex before switching accounts. Saving remains available.","切换账户前请先关闭 Codex；仍可保存账户。")),
        };
        div()
            .id("codex-save-process-warning")
            .test_support()
            .role(gpui_kit::Role::Status)
            .aria_label(message.clone())
            .child(message)
            .into_any_element()
    }
}
fn pid_text(pids: &[u32]) -> String {
    if pids.is_empty() {
        return "—".into();
    }
    let mut text = pids
        .iter()
        .take(8)
        .map(u32::to_string)
        .collect::<Vec<_>>()
        .join(", ");
    if pids.len() > 8 {
        text.push_str(&format!(" (+{})", pids.len() - 8));
    }
    text
}
fn port_text(state: &PortState, cx: &gpui_kit::App) -> &'static str {
    match state {
        PortState::Available => text(
            cx,
            "Callback port was available at preflight; final bind is still required.",
            "预检时回调端口可用，仍需最终绑定校验。",
        ),
        PortState::Occupied => text(
            cx,
            "Callback port is occupied. Review owned cleanup or close the unknown listener manually.",
            "回调端口被占用。请核对自有清理，或手动关闭未知监听器。",
        ),
        PortState::Unavailable => text(
            cx,
            "Callback port could not be checked; availability is unknown.",
            "无法检查回调端口，可用状态未知。",
        ),
    }
}

#[cfg(test)]
mod tests;
