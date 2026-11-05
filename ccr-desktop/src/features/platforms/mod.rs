//! Native platform settings, scoped source, accounts and OpenCode providers.
use crate::{
    locales::text,
    runtime::{
        Services,
        platforms::{
            AccountAction, Command, DESCRIPTORS, OAuthControl, OAuthResult, PlatformSession,
            ResultValue, Surface,
        },
    },
    shell::semantics,
    theme,
};
use ccr_app::{
    platform_settings::SaveOutcome,
    runtime::{Completion, EnvironmentId, Platform, RequestKey, ResponseGate, TaskEvent},
};
use ccr_cli::application::codex_oauth::{NativePhase, NativeStatus};
use gpui_kit::base::TestSupportExt;
use gpui_kit::component::{
    ActiveTheme, FocusTrapElement, Selectable,
    button::{Button, ButtonVariants},
    input::{Input, InputState, Textarea, TextareaState},
};
use gpui_kit::{
    AppContext, Context, Entity, FocusHandle, Focusable, InteractiveElement, IntoElement,
    ParentElement, Render, StatefulInteractiveElement, Styled, Task, Window, div, px,
};
use std::sync::Arc;

#[derive(Clone, Copy)]
enum Decision {
    Source,
    DeleteAccount,
    DeleteProvider,
}
pub struct PlatformsView {
    services: Services,
    platform: Platform,
    surface: Surface,
    session: Option<Arc<PlatformSession>>,
    source_mode: bool,
    generation: u64,
    gate: ResponseGate,
    task: Option<Task<()>>,
    busy: bool,
    message: Option<String>,
    name: Entity<InputState>,
    content: Entity<TextareaState>,
    pending_content: Option<String>,
    decision: Option<Decision>,
    decision_name: String,
    decision_generation: u64,
    decision_entry_pending: bool,
    return_focus: Option<FocusHandle>,
    confirmation_focus: FocusHandle,
    oauth_task: Option<Task<()>>,
    oauth_control_task: Option<Task<()>>,
    oauth_generation: u64,
    oauth_status: Option<NativeStatus>,
    oauth_control_busy: bool,
    oauth_url: Entity<InputState>,
}
impl PlatformsView {
    pub fn new(services: Services, window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            services,
            platform: Platform::Claude,
            surface: Surface::Settings,
            session: None,
            source_mode: false,
            generation: 0,
            gate: ResponseGate::default(),
            task: None,
            busy: false,
            message: None,
            name: cx.new(|cx| InputState::new(window, cx)),
            content: cx.new(|cx| TextareaState::new(window, cx)),
            pending_content: None,
            decision: None,
            decision_name: String::new(),
            decision_generation: 0,
            decision_entry_pending: false,
            return_focus: None,
            confirmation_focus: cx.focus_handle().tab_stop(false),
            oauth_task: None,
            oauth_control_task: None,
            oauth_generation: 0,
            oauth_status: None,
            oauth_control_busy: false,
            oauth_url: cx.new(|cx| InputState::new(window, cx)),
        }
    }
    pub fn focus_main(&self, window: &mut Window, cx: &mut Context<Self>) {
        if matches!(self.surface, Surface::Auth | Surface::Providers) {
            window.focus(&self.name.read(cx).focus_handle(cx), cx);
        } else {
            window.focus(&self.content.read(cx).focus_handle(cx), cx);
        }
    }
    pub fn leave(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.oauth_task.is_some() {
            self.services.platform_oauth_cancel();
        }
        self.oauth_task = None;
        self.oauth_control_task = None;
        self.oauth_generation += 1;
        self.oauth_status = None;
        self.oauth_control_busy = false;
        self.oauth_url
            .update(cx, |state, cx| state.set_value("", window, cx));
        self.task = None;
        self.gate.clear();
        self.generation += 1;
        self.session = None;
        self.source_mode = false;
        self.pending_content = None;
        self.decision = None;
        self.decision_entry_pending = false;
        self.decision_name.clear();
        self.return_focus = None;
        self.busy = false;
        self.message = None;
        self.name
            .update(cx, |state, cx| state.set_value("", window, cx));
        self.content
            .update(cx, |state, cx| state.set_value("", window, cx));
        cx.notify();
    }
    fn select(
        &mut self,
        platform: Platform,
        surface: Surface,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.leave(window, cx);
        self.platform = platform;
        self.surface = surface;
        self.load(cx);
    }
    pub fn load(&mut self, cx: &mut Context<Self>) {
        self.request(Command::Read(self.surface), cx);
    }
    fn request(&mut self, command: Command, cx: &mut Context<Self>) {
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
            platform: self.platform,
            generation: self.generation,
            request_id: self.generation,
        };
        self.gate.replace(key.clone());
        self.busy = true;
        self.message = None;
        match self.services.platform_request(key.clone(), command) {
            Ok(mut subscription) => self.task = Some(cx.spawn(async move |weak, cx| {
                while let Some(event) = subscription.next().await {
                    if let TaskEvent::Terminal(result) = event {
                        let _ = weak.update(cx, |this, cx| {
                            if !this.gate.accepts(&key) { return; }
                            this.busy = false;
                            match result.as_ref() {
                                Completion::Success(ResultValue::Loaded(session)) => {
                                    this.session = Some(session.clone());
                                    if this.source_mode { this.pending_content = Some(session.snapshot.content.clone()); }
                                    this.message = Some("Loaded from captured local roots. / 已从固定本地路径读取".into());
                                }
                                Completion::Success(ResultValue::Saved(SaveOutcome::Conflict)) => this.message = Some("Source changed. Draft and original revision retained. Discard and reload before saving again. / 文件已变化，保留草稿和原始版本，请丢弃并重读".into()),
                                Completion::Success(ResultValue::Saved(SaveOutcome::Saved { .. }) | ResultValue::AccountChanged { .. }) => {
                                    // 已提交后必须重新读取，不能携旧版本重复提交。
                                    this.session = None;
                                    this.source_mode = false;
                                    this.pending_content = Some(String::new());
                                    this.message = Some(if matches!(result.as_ref(), Completion::Success(ResultValue::AccountChanged { warning: true })) {
                                        "Account operation committed, but authentication suppressors or diagnosis warnings remain. Reload and inspect existing CCR diagnosis before retrying. / 账户操作已提交，但仍有认证抑制源或诊断警告，请重读并检查 CCR 诊断。"
                                    } else {
                                        "Operation committed. Reload to start a new edit session. / 操作已提交，请重新读取以开始新编辑"
                                    }.into());
                                }
                                Completion::Failed(error) | Completion::CleanupFailed(error) => this.message = Some(error.message().into()),
                                Completion::Cancelled => this.message = Some("Read cancelled. / 已取消读取".into()),
                                Completion::WorkerFailed => this.message = Some("Platform worker stopped. Retry. / 平台线程已停止，请重试".into()),
                            }
                            cx.notify();
                        });
                        break;
                    }
                }
            })),
            Err(_) => { self.busy = false; self.message = Some("Queue busy or application shutting down. Retry. / 队列繁忙或应用正在退出".into()); }
        }
        cx.notify();
    }
    fn save(&mut self, cx: &mut Context<Self>) {
        let Some(session) = self.session.clone() else {
            return;
        };
        let content = self.content.read(cx).value().to_string();
        let command = if self.surface == Surface::Providers {
            Command::Provider {
                session,
                id: self.name.read(cx).value().to_string(),
                patch: Some(content),
            }
        } else if self.source_mode {
            Command::Source { session, content }
        } else {
            Command::Patch { session, content }
        };
        self.request(command, cx);
    }
    fn oauth_key(&self) -> RequestKey {
        RequestKey {
            environment: EnvironmentId::Local(
                if self.services.is_fixture() {
                    "fixture"
                } else {
                    "local"
                }
                .into(),
            ),
            platform: Platform::Codex,
            generation: self.oauth_generation,
            request_id: self.oauth_generation,
        }
    }
    fn start_oauth(&mut self, cx: &mut Context<Self>) {
        let Some(session) = self.session.clone() else {
            return;
        };
        if self.platform != Platform::Codex
            || self.surface != Surface::Auth
            || self.services.platform_oauth_active()
        {
            return;
        }
        self.oauth_generation += 1;
        let generation = self.oauth_generation;
        match self
            .services
            .platform_oauth_start(self.oauth_key(), session)
        {
            Ok(mut subscription) => {
                self.oauth_status = None;
                self.message = Some("Starting owned OAuth listener. Callback reception is separate from account commit. / 正在启动 OAuth，收到回调不代表账户已提交。".into());
                self.oauth_task = Some(cx.spawn(async move |weak, cx| {
                    while let Some(event) = subscription.next().await {
                        let terminal = matches!(event, TaskEvent::Terminal(_));
                        let _ = weak.update(cx, |this, cx| {
                            if this.oauth_generation != generation { return; }
                            this.oauth_status = this.services.platform_oauth_status();
                            if let TaskEvent::Terminal(result) = event {
                                this.pending_content = Some(String::new());
                                this.message = Some(match result.as_ref() {
                                    Completion::Success(OAuthResult::Finished) => {
                                        this.session = None;
                                        "OAuth account committed and owner cleanup finished. Reload accounts. / OAuth 账户已提交并完成清理，请重读账户。"
                                    }
                                    Completion::Cancelled => "OAuth cancelled after listener and pending-secret cleanup. / OAuth 已取消并清理监听器与临时凭据。",
                                    Completion::CleanupFailed(_) => "OAuth cleanup failed. New login remains blocked; retry cancellation cleanup. / OAuth 清理失败，新登录被阻止，请重试清理。",
                                    _ if this.oauth_status.as_ref().is_some_and(|status| status.phase == NativePhase::TimedOut) => "OAuth timed out and cleaned up; no account success claimed. / OAuth 已超时并清理，未声称账户登录成功。",
                                    _ => "OAuth failed; no account success claimed. Reload before retrying. / OAuth 失败，请重读后重试。",
                                }.into());
                            }
                            cx.notify();
                        });
                        if terminal { break; }
                    }
                }));
            }
            Err(_) => {
                self.message = Some(
                    "OAuth admission busy or application closing. / OAuth 队列繁忙或应用正在退出。"
                        .into(),
                )
            }
        }
        cx.notify();
    }
    fn control_oauth(&mut self, complete: bool, cx: &mut Context<Self>) {
        let Some(status) = &self.oauth_status else {
            return;
        };
        if self.oauth_control_busy {
            return;
        }
        let control = if status.phase == NativePhase::CleanupFailed {
            OAuthControl::RetryCleanup
        } else if complete {
            OAuthControl::Complete {
                login_id: status.login_id.clone(),
                name: Some(self.name.read(cx).value().to_string()),
            }
        } else {
            OAuthControl::Callback {
                login_id: status.login_id.clone(),
                url: self.content.read(cx).value().to_string(),
            }
        };
        let generation = self.oauth_generation;
        match self
            .services
            .platform_oauth_control(self.oauth_key(), control)
        {
            Ok(mut subscription) => {
                self.oauth_control_busy = true;
                self.oauth_control_task = Some(cx.spawn(async move |weak, cx| {
                    while let Some(event) = subscription.next().await {
                        if let TaskEvent::Terminal(result) = event {
                            let _ = weak.update(cx, |this, cx| {
                                if this.oauth_generation != generation { return; }
                                this.oauth_control_busy = false;
                                this.oauth_status = this.services.platform_oauth_status();
                                this.message = Some(match result.as_ref() {
                                    Completion::Success(OAuthResult::CallbackAccepted) => "Callback accepted. Account has not been committed; choose Complete. / 已接受回调，账户尚未提交，请选择完成。",
                                    Completion::Success(OAuthResult::AccountCommitted) => "Account committed. Reload accounts. / 账户已提交，请重读账户。",
                                    Completion::Success(OAuthResult::CleanupFinished) => "Pending OAuth secrets cleaned; login can be retried. / OAuth 临时凭据已清理，可重新登录。",
                                    _ => "OAuth control rejected or failed. No success claimed; current owner remains authoritative. / OAuth 操作被拒绝或失败，以当前任务状态为准。",
                                }.into());
                                if matches!(result.as_ref(), Completion::Success(_)) { this.pending_content = Some(String::new()); }
                                cx.notify();
                            });
                            break;
                        }
                    }
                }));
            }
            Err(_) => {
                self.message =
                    Some("OAuth control queue unavailable. / OAuth 操作队列不可用。".into())
            }
        }
        cx.notify();
    }
    fn account(&mut self, action: AccountAction, cx: &mut Context<Self>) {
        if let Some(session) = self.session.clone() {
            self.request(
                Command::Account {
                    session,
                    name: self.name.read(cx).value().to_string(),
                    action,
                },
                cx,
            );
        }
    }
    fn decide(&mut self, decision: Decision, window: &mut Window, cx: &mut Context<Self>) {
        if self.busy || self.decision.is_some() {
            return;
        }
        self.decision = Some(decision);
        self.decision_name = self.name.read(cx).value().to_string();
        self.decision_generation += 1;
        self.decision_entry_pending = true;
        self.return_focus = window.focused(cx);
        cx.notify();
    }
    fn dismiss(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.decision = None;
        self.decision_entry_pending = false;
        if let Some(focus) = self.return_focus.take() {
            window.focus(&focus, cx);
        }
        cx.notify();
    }
    fn confirm(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let decision = self.decision;
        let name = self.decision_name.clone();
        self.dismiss(window, cx);
        match decision {
            Some(Decision::Source) => {
                if let Some(session) = &self.session {
                    self.source_mode = true;
                    self.pending_content = Some(session.snapshot.content.clone());
                }
            }
            Some(Decision::DeleteAccount) => {
                if let Some(session) = self.session.clone() {
                    self.request(
                        Command::Account {
                            session,
                            name,
                            action: AccountAction::Delete,
                        },
                        cx,
                    );
                }
            }
            Some(Decision::DeleteProvider) => {
                if let Some(session) = self.session.clone() {
                    self.request(
                        Command::Provider {
                            session,
                            id: name,
                            patch: None,
                        },
                        cx,
                    );
                }
            }
            None => {}
        }
        cx.notify();
    }
}
impl Render for PlatformsView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if let Some(content) = self.pending_content.take() {
            self.content
                .update(cx, |state, cx| state.set_value(content, window, cx));
        }
        let oauth_active = self.services.platform_oauth_active();
        let authorization_url = self
            .oauth_status
            .as_ref()
            .map(|status| status.authorization_url.expose())
            .unwrap_or("");
        if self.oauth_url.read(cx).value().as_ref() != authorization_url {
            self.oauth_url.update(cx, |state, cx| {
                state.set_value(authorization_url, window, cx)
            });
        }
        let blocked = self.busy || self.decision.is_some() || oauth_active;
        let mut tabs = div().flex().flex_wrap().gap(px(theme::CONTROL_GAP));
        for descriptor in DESCRIPTORS {
            let platform = descriptor.platform;
            tabs = tabs.child(semantics::button(
                Button::new(platform.short_name())
                    .label(descriptor.name)
                    .ghost()
                    .selected(self.platform == platform)
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.select(platform, Surface::Settings, window, cx)
                    })),
                blocked,
                false,
                cx,
            ));
        }
        let mut root = div()
            .id("platform-workspace")
            .flex()
            .flex_col()
            .size_full()
            .min_h_0()
            .overflow_y_scroll()
            .p(px(theme::CONTENT_PADDING))
            .gap(px(theme::CONTROL_GAP))
            .bg(cx.theme().background)
            .rounded_tr(px(theme::SHEET_CONTENT_RADIUS))
            .on_key_down(
                cx.listener(|this, event: &gpui_kit::KeyDownEvent, window, cx| {
                    if event.keystroke.key == "escape" && this.decision.is_some() {
                        this.dismiss(window, cx);
                        cx.stop_propagation();
                    }
                }),
            )
            .child(div().text_size(px(theme::TITLE_SIZE)).child(text(
                cx,
                "Platform settings and accounts",
                "平台设置与账户",
            )))
            .child(tabs);
        let mut surfaces = div().flex().flex_wrap().gap(px(theme::CONTROL_GAP));
        let descriptor = DESCRIPTORS
            .iter()
            .find(|descriptor| descriptor.platform == self.platform);
        for (id, surface, label, enabled) in [
            (
                "platform-settings",
                Surface::Settings,
                "Settings / 设置",
                descriptor.is_some_and(|value| value.settings),
            ),
            (
                "platform-auth",
                Surface::Auth,
                "Accounts / 账户",
                descriptor.is_some_and(|value| value.auth),
            ),
            (
                "platform-providers",
                Surface::Providers,
                "Providers / 供应商",
                descriptor.is_some_and(|value| value.providers),
            ),
            (
                "platform-tui",
                Surface::Tui,
                "TUI / 终端界面",
                self.platform == Platform::OpenCode,
            ),
        ] {
            if enabled {
                surfaces = surfaces.child(semantics::button(
                    Button::new(id)
                        .label(label)
                        .ghost()
                        .selected(self.surface == surface)
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.select(this.platform, surface, window, cx)
                        })),
                    blocked,
                    false,
                    cx,
                ));
            }
        }
        surfaces = surfaces.child(semantics::button(
            Button::new("platform-legacy-antigravity")
                .label("Advanced: Antigravity source / 高级源文件")
                .ghost()
                .on_click(cx.listener(|this, _, window, cx| {
                    this.select(Platform::Gemini, Surface::LegacySettings, window, cx)
                })),
            blocked,
            false,
            cx,
        ));
        root = root.child(surfaces);
        if let Some(decision) = self.decision {
            if std::mem::take(&mut self.decision_entry_pending) {
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
            let message = match decision { Decision::Source => "Open plaintext source in this scoped editor? It may contain credentials. Do not include it in logs or screenshots. / 打开可能含明文凭据的临时源编辑区？请勿记录或截图。".to_owned(), Decision::DeleteAccount => format!("Delete saved account {}? This removes the saved snapshot, not an official logout. / 删除保存账户，不等于官方退出登录。", self.decision_name), Decision::DeleteProvider => format!("Delete OpenCode provider {}? Other providers remain unchanged. / 删除此供应商，保留其他供应商。", self.decision_name) };
            root = root
                .child(
                    div()
                        .id("platform-confirmation")
                        .test_support()
                        .role(gpui_kit::Role::AlertDialog)
                        .aria_label("Confirm sensitive platform action")
                        .child(message),
                )
                .child(
                    div()
                        .tab_group()
                        .tab_stop(false)
                        .flex()
                        .gap(px(theme::CONTROL_GAP))
                        .child(semantics::button(
                            Button::new("platform-decision-cancel")
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
                            Button::new("platform-decision-confirm")
                                .label("Confirm / 确认")
                                .on_click(
                                    cx.listener(|this, _, window, cx| this.confirm(window, cx)),
                                ),
                            false,
                            false,
                            cx,
                        ))
                        .focus_trap("platform-decision-trap", &self.confirmation_focus),
                );
        }
        root = root.child(
            div()
                .flex()
                .gap(px(theme::CONTROL_GAP))
                .child(semantics::button(
                    Button::new("platform-load")
                        .label(text(cx, "Discard and reload", "丢弃并重读"))
                        .ghost()
                        .on_click(cx.listener(|this, _, window, cx| {
                            let platform = this.platform;
                            let surface = this.surface;
                            this.select(platform, surface, window, cx);
                        })),
                    blocked,
                    self.busy,
                    cx,
                ))
                .child(semantics::button(
                    Button::new("platform-clear")
                        .label(text(cx, "Close sensitive session", "关闭敏感会话"))
                        .ghost()
                        .on_click(cx.listener(|this, _, window, cx| this.leave(window, cx))),
                    self.busy || self.decision.is_some(),
                    false,
                    cx,
                )),
        );
        if let Some(session) = &self.session {
            if self.surface == Surface::Auth {
                if self.platform == Platform::Codex {
                    let phase = self.oauth_status.as_ref().map(|status| status.phase);
                    root = root.child(div().flex().flex_wrap().gap(px(theme::CONTROL_GAP))
                        .child(semantics::button(Button::new("platform-oauth-start")
                            .label("Start OAuth / 开始 OAuth")
                            .on_click(cx.listener(|this, _, _, cx| this.start_oauth(cx))),
                            blocked || self.oauth_control_busy, false, cx))
                        .child(semantics::button(Button::new("platform-oauth-cancel")
                            .label("Cancel OAuth / 取消 OAuth").ghost()
                            .on_click(cx.listener(|this, _, _, cx| {
                                if this.oauth_status.as_ref().is_some_and(|status| status.phase == NativePhase::CleanupFailed) { this.control_oauth(false, cx); }
                                else { this.services.platform_oauth_cancel(); this.message = Some("Cancellation requested; waiting for listener/secret cleanup or an already started commit. / 已请求取消，等待清理或已开始的账户提交。".into()); cx.notify(); }
                            })),
                            (!oauth_active && phase != Some(NativePhase::CleanupFailed)) || self.oauth_control_busy, false, cx)));
                    if let Some(status) = &self.oauth_status {
                        root = root.child(if status.fixture {
                            "Isolated fixture: external authorization and token exchange disabled. Callback reception never proves authentication. / 隔离 fixture：禁用外部授权与令牌交换，收到回调不等于认证成功。"
                        } else { "Authorization URL stays in this scoped session. After callback, explicitly complete account commit. / 授权链接仅保留在本会话，收到回调后请显式完成账户提交。" })
                        .child(Input::new(&self.oauth_url).readonly(true).aria_label("Scoped OAuth authorization URL / 临时 OAuth 授权链接"))
                        .child(Textarea::new(&self.content).h(px(72.)).readonly(self.oauth_control_busy)
                            .aria_label("Paste OAuth callback URL / 粘贴 OAuth 回调地址"))
                        .child(div().flex().flex_wrap().gap(px(theme::CONTROL_GAP))
                            .child(semantics::button(Button::new("platform-oauth-callback")
                                .label("Submit callback / 提交回调").ghost()
                                .on_click(cx.listener(|this, _, _, cx| this.control_oauth(false, cx))),
                                !oauth_active || self.oauth_control_busy || phase != Some(NativePhase::Waiting), false, cx))
                            .child(semantics::button(Button::new("platform-oauth-complete")
                                .label("Complete account commit / 完成账户提交")
                                .on_click(cx.listener(|this, _, _, cx| this.control_oauth(true, cx))),
                                !oauth_active || self.oauth_control_busy || phase != Some(NativePhase::Callback), self.oauth_control_busy, cx)));
                    }
                }
                root = root.child(if session.signed_in {
                    "Runtime credential detected / 检测到运行凭据"
                } else {
                    "No usable runtime credential / 未检测到可用运行凭据"
                });
                for account in &session.accounts {
                    root = root.child(div().h(px(theme::ROW_HEIGHT)).child(format!(
                        "{}{}",
                        account.name,
                        if account.current {
                            " · current / 当前"
                        } else {
                            ""
                        }
                    )));
                }
                if self.platform != Platform::Grok {
                    root = root
                        .child(Input::new(&self.name).aria_label("Account name / 账户名"))
                        .child(
                            div()
                                .flex()
                                .gap(px(theme::CONTROL_GAP))
                                .child(semantics::button(
                                    Button::new("platform-account-save")
                                        .label("Save runtime as account / 保存运行账户")
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.account(AccountAction::Save, cx)
                                        })),
                                    blocked,
                                    false,
                                    cx,
                                ))
                                .child(semantics::button(
                                    Button::new("platform-account-switch")
                                        .label("Switch account / 切换账户")
                                        .ghost()
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.account(AccountAction::Switch, cx)
                                        })),
                                    blocked,
                                    false,
                                    cx,
                                ))
                                .child(semantics::button(
                                    Button::new("platform-account-delete")
                                        .label("Delete saved account / 删除保存账户")
                                        .ghost()
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            this.decide(Decision::DeleteAccount, window, cx)
                                        })),
                                    blocked,
                                    false,
                                    cx,
                                )),
                        );
                }
            } else {
                root = root.child(text(cx, "Only entered dirty keys are patched. Unknown siblings survive. Source editing uses the original loaded revision; conflicts retain the draft. JSON null clearing is unsupported here.", "仅提交输入的变更字段，保留未知字段。源编辑使用原始读取版本，冲突保留草稿。此处 JSON 不支持 null 清除。"));
                if self.surface == Surface::Providers {
                    root = root
                        .child(format!(
                            "Provider IDs / 供应商: {}",
                            session.provider_ids.join(", ")
                        ))
                        .child(Input::new(&self.name).aria_label("Provider ID / 供应商标识"))
                        .child(semantics::button(
                            Button::new("platform-provider-delete")
                                .label("Delete provider / 删除供应商")
                                .ghost()
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.decide(Decision::DeleteProvider, window, cx)
                                })),
                            blocked,
                            false,
                            cx,
                        ));
                } else if !self.source_mode {
                    root = root.child(semantics::button(
                        Button::new("platform-source-open")
                            .label("Open scoped plaintext source / 打开临时明文源文件")
                            .ghost()
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.decide(Decision::Source, window, cx)
                            })),
                        blocked,
                        false,
                        cx,
                    ));
                }
                let can_patch = self.platform != Platform::Grok || self.source_mode;
                root = root
                    .child(
                        Textarea::new(&self.content)
                            .accessibility_id("platform-content")
                            .aria_label(if self.source_mode {
                                "Plaintext settings source / 明文设置源文件"
                            } else {
                                "Dirty settings JSON object / 变更设置 JSON 对象"
                            })
                            .h(px(180.))
                            .readonly(blocked || !can_patch),
                    )
                    .child(semantics::button(
                        Button::new("platform-save")
                            .label(if self.source_mode {
                                "Save source with original revision / 按原始版本保存源文件"
                            } else {
                                "Save dirty patch / 保存变更字段"
                            })
                            .on_click(cx.listener(|this, _, _, cx| this.save(cx))),
                        blocked || !can_patch,
                        self.busy,
                        cx,
                    ));
            }
        } else if !self.busy {
            root = root.child(text(cx, "Load the selected local surface to begin. Fixture roots are isolated and visibly labeled by the shell.", "读取所选本地功能后开始。演示路径已隔离并由外壳标记。"));
        }
        if let Some(message) = &self.message {
            root = root.child(
                div()
                    .id("platform-operation-status")
                    .role(gpui_kit::Role::Status)
                    .child(message.clone()),
            );
        }
        root
    }
}

#[cfg(test)]
mod tests;
