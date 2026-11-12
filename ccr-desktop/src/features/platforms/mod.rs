//! Native platform settings, scoped source, accounts and OpenCode providers.
mod diagnostics;
mod forms;
mod layers;
use crate::{
    locales::{is_chinese, text},
    runtime::{
        Services,
        platforms::{
            AccountAction, Command, CredentialState, DESCRIPTORS, FileState, OAuthControl,
            OAuthResult, PlatformSession, ProviderReport, ResultValue, Surface,
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
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::{
    AppContext, Context, Entity, FocusHandle, Focusable, InteractiveElement, IntoElement,
    ParentElement, Render, SharedString, StatefulInteractiveElement, Styled, Task, Window, div, px,
};
use std::sync::Arc;

fn open_authorization_url(raw: &str, cx: &gpui_kit::App) -> bool {
    let Ok(url) = ccr_app::jobs::process::ProcessGateway::validate_oauth_url(raw) else {
        return false;
    };
    // The shared allowlist also supports callback URLs. This button opens only
    // the provider authorization endpoint, never the local callback listener.
    if url.scheme() != "https" {
        return false;
    }
    cx.open_url(url.as_str());
    true
}

#[derive(Clone, Copy)]
enum Decision {
    Source,
    DeleteAccount,
    DeleteProvider,
    AuthOff,
    RenameAccount,
    AddApiKey,
    ImportAccounts,
}
pub struct PlatformsView {
    form: forms::SettingsForm,
    pending_form: bool,
    advanced: bool,
    layers: Entity<layers::LayersView>,
    layers_open: bool,
    services: Services,
    platform: Platform,
    surface: Surface,
    pinned: bool,
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
    decision_content: String,
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
    quotas: Vec<crate::runtime::platforms::QuotaRow>,
    diagnostics: diagnostics::ViewState,
}
impl PlatformsView {
    pub fn new(services: Services, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let layers = cx.new(|cx| layers::LayersView::new(services.clone(), window, cx));
        Self {
            form: forms::SettingsForm::new(),
            pending_form: false,
            advanced: false,
            layers,
            layers_open: false,
            services,
            platform: Platform::Claude,
            surface: Surface::Settings,
            pinned: false,
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
            decision_content: String::new(),
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
            quotas: Vec::new(),
            diagnostics: Default::default(),
        }
    }
    pub fn focus_main(&self, window: &mut Window, cx: &mut Context<Self>) {
        if matches!(self.surface, Surface::Auth | Surface::Providers) {
            window.focus(&self.name.read(cx).focus_handle(cx), cx);
        } else {
            if self.source_mode || self.advanced {
                window.focus(&self.content.read(cx).focus_handle(cx), cx);
            } else {
                self.form.focus(window, cx);
            }
        }
    }
    pub fn leave(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.diagnostics = Default::default();
        self.form.clear();
        self.pending_form = false;
        self.advanced = false;
        self.layers.update(cx, |view, cx| view.suspend(cx));
        self.layers_open = false;
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
        self.decision_content.clear();
        self.return_focus = None;
        self.busy = false;
        self.message = None;
        self.quotas.clear();
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
    /// One sidebar destination owns this platform. Hide the cross-platform switcher.
    pub fn open(
        &mut self,
        platform: Platform,
        surface: Surface,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.pinned = true;
        self.select(platform, surface, window, cx);
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
        self.message = matches!(&command, Command::Quota { .. }).then(|| {
            text(
                cx,
                "Fetching quota. Credential refresh and history writes remain owned by CCR.",
                "正在查询配额。凭据刷新和历史写入由 CCR 管理。",
            )
            .to_string()
        });
        match self.services.platform_request(key.clone(), command) {
            Ok(mut subscription) => self.task = Some(cx.spawn(async move |weak, cx| {
                while let Some(event) = subscription.next().await {
                    if let TaskEvent::Terminal(result) = event {
                        let _ = weak.update(cx, |this, cx| {
                            if !this.gate.accepts(&key) { return; }
                            this.busy = false;
                            match result.as_ref() {
                                Completion::Success(ResultValue::Quotas(rows)) => {
                                    this.quotas = rows.clone();
                                    this.message = rows.is_empty().then(||text(cx,"No saved accounts to query.","没有可查询的已保存账户。").to_string());
                                }
                                Completion::Success(ResultValue::Loaded(session)) => {
                                    this.pending_form = true;
                                    this.session = Some(session.clone());
                                    if this.source_mode { this.pending_content = Some(session.snapshot.content.clone()); }
                                    this.message = None;
                                }
                                Completion::Success(ResultValue::Saved(SaveOutcome::Conflict)) => this.message = Some("Source changed. Draft and original revision retained. Discard and reload before saving again. / 文件已变化，保留草稿和原始版本，请丢弃并重读".into()),
                                Completion::Success(ResultValue::Saved(SaveOutcome::Saved { .. }) | ResultValue::AccountChanged { .. }) => {
                                    // 已提交后必须重新读取，不能携旧版本重复提交。
                                    this.session = None;
                                    this.pending_form = true;
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
    fn choose_opencode_theme(
        &mut self,
        theme: &'static str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.busy
            || self.source_mode
            || self.platform != Platform::OpenCode
            || self.surface != Surface::Tui
        {
            return;
        }
        let raw = self.content.read(cx).value().to_string();
        let value = if raw.trim().is_empty() {
            Ok(serde_json::json!({}))
        } else {
            serde_json::from_str::<serde_json::Value>(&raw)
        };
        let Ok(serde_json::Value::Object(mut patch)) = value else {
            self.message = Some(
                text(
                    cx,
                    "Fix the JSON object draft before choosing a theme.",
                    "请先修正 JSON 对象草稿，再选择主题。",
                )
                .into(),
            );
            cx.notify();
            return;
        };
        patch.insert("theme".into(), serde_json::Value::String(theme.into()));
        self.content.update(cx, |input, cx| {
            input.set_value(serde_json::Value::Object(patch).to_string(), window, cx)
        });
        self.message = None;
        cx.notify();
    }
    fn save(&mut self, cx: &mut Context<Self>) {
        let Some(session) = self.session.clone() else {
            return;
        };
        let content = if self.source_mode || self.advanced {
            self.content.read(cx).value().to_string()
        } else {
            match self.form.patch(&self.content.read(cx).value(), cx) {
                Ok(value) => value,
                Err(error) => {
                    self.message = Some(error);
                    cx.notify();
                    return;
                }
            }
        };
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
    fn open_oauth_browser(&mut self, cx: &mut Context<Self>) {
        let current = self.services.platform_oauth_status();
        let current = current.filter(|status| {
            self.platform == Platform::Codex
                && self.surface == Surface::Auth
                && !self.services.is_fixture()
                && !status.fixture
                && status.phase == NativePhase::Waiting
                && self
                    .oauth_status
                    .as_ref()
                    .is_some_and(|shown| shown.login_id == status.login_id)
        });
        self.message = Some(
            if current
                .is_some_and(|status| open_authorization_url(status.authorization_url.expose(), cx))
            {
                text(
                    cx,
                    "Browser open requested. Authorization and account commit are not complete.",
                    "已请求在浏览器打开授权页。授权与账户提交尚未完成。",
                )
            } else {
                text(
                    cx,
                    "Browser open refused: no current live authorization URL is available.",
                    "无可用的当前真实授权链接，已拒绝打开浏览器。",
                )
            }
            .into(),
        );
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
        self.decision_content = self.content.read(cx).value().to_string();
        self.decision_generation += 1;
        self.decision_entry_pending = true;
        self.return_focus = window.focused(cx);
        cx.notify();
    }
    fn dismiss(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.decision = None;
        self.decision_entry_pending = false;
        self.decision_content.clear();
        if let Some(focus) = self.return_focus.take() {
            window.focus(&focus, cx);
        }
        cx.notify();
    }
    fn confirm(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let decision = self.decision;
        let name = self.decision_name.clone();
        let content = self.decision_content.clone();
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
            Some(
                Decision::AuthOff
                | Decision::RenameAccount
                | Decision::AddApiKey
                | Decision::ImportAccounts,
            ) => {
                let action = match decision {
                    Some(Decision::AuthOff) => AccountAction::Off,
                    Some(Decision::RenameAccount) => AccountAction::Rename(content.trim().into()),
                    Some(Decision::ImportAccounts) => AccountAction::Import(content),
                    Some(Decision::AddApiKey) => {
                        let parsed = serde_json::from_str::<serde_json::Value>(&content).ok();
                        let valid =
                            parsed
                                .as_ref()
                                .and_then(|v| v.as_object())
                                .is_some_and(|object| {
                                    object
                                        .keys()
                                        .all(|key| matches!(key.as_str(), "api_key" | "base_url"))
                                        && object
                                            .get("api_key")
                                            .and_then(|value| value.as_str())
                                            .is_some_and(|key| !key.trim().is_empty())
                                        && object.get("base_url").is_none_or(|value| {
                                            value.is_null() || value.is_string()
                                        })
                                });
                        if !valid {
                            self.message = Some(
                                text(
                                    cx,
                                    "Invalid API account JSON. Draft retained.",
                                    "API 账户 JSON 无效，保留草稿。",
                                )
                                .into(),
                            );
                            cx.notify();
                            return;
                        }
                        let key = parsed
                            .as_ref()
                            .and_then(|v| v.get("api_key"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let base_url = parsed
                            .as_ref()
                            .and_then(|v| v.get("base_url"))
                            .and_then(|v| v.as_str())
                            .map(str::to_string);
                        AccountAction::ApiKey { key, base_url }
                    }
                    _ => return,
                };
                if let Some(session) = self.session.clone() {
                    self.request(
                        Command::Account {
                            session,
                            name,
                            action,
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
fn ui_line(cx: &gpui_kit::App, value: &str) -> String {
    crate::locales::localized_line(cx, value)
}

fn file_status(cx: &gpui_kit::App, session: &PlatformSession) -> String {
    let label = session.file_label;
    let chinese = is_chinese(cx);
    match (
        session.file_state,
        session.platform == Platform::Gemini,
        chinese,
    ) {
        (FileState::Missing, _, false) => format!("{label} was not found. Nothing was created."),
        (FileState::Missing, _, true) => format!("{label} 不存在。没有新建文件。"),
        (FileState::Invalid, _, false) => {
            format!("{label} is not a valid document. No values are shown.")
        }
        (FileState::Invalid, _, true) => format!("{label} 不是有效文档。不显示任何值。"),
        (FileState::Present, true, false) => {
            format!("{label} was found. Typed settings are not supported, so values stay hidden.")
        }
        (FileState::Present, true, true) => {
            format!("{label} 已找到。没有类型化设置，因此不显示值。")
        }
        (FileState::Present, false, false) if session.fields.is_empty() => {
            format!("{label} was found. No supported fields are set.")
        }
        (FileState::Present, false, true) if session.fields.is_empty() => {
            format!("{label} 已找到。没有设置受支持的字段。")
        }
        (FileState::Present, false, false) => {
            format!("{label} was found. Only supported non-secret fields are shown.")
        }
        (FileState::Present, false, true) => {
            format!("{label} 已找到。只显示受支持的非秘密字段。")
        }
    }
}

fn local_projection(
    session: &PlatformSession,
    show_fields: bool,
    cx: &gpui_kit::App,
) -> impl IntoElement {
    let file = file_status(cx, session);
    let auth = auth_status(cx, session);
    let mut block = div()
        .flex()
        .flex_col()
        .gap(px(theme::CONTROL_GAP))
        .text_size(px(theme::BODY_SIZE))
        .child(
            div()
                .id("platform-file-status")
                .test_support()
                .role(gpui_kit::Role::Status)
                .aria_label(file.clone())
                .child(file),
        )
        .child(
            div()
                .id("platform-auth-status")
                .test_support()
                .role(gpui_kit::Role::Status)
                .aria_label(auth.clone())
                .child(auth),
        );
    for (index, field) in session.fields.iter().enumerate().filter(|_| show_fields) {
        let label = format!("{}: {}", field.key, field.value);
        block = block.child(
            div()
                .id(SharedString::from(if session.surface == Surface::Models {
                    format!("platform-model-{index}")
                } else if field.key == "oauth_source" {
                    format!("platform-oauth-source-{index}")
                } else {
                    format!("platform-field-{}", field.key)
                }))
                .test_support()
                .h(px(theme::ROW_HEIGHT))
                .flex()
                .items_center()
                .overflow_hidden()
                .aria_label(label.clone())
                .child(label),
        );
    }
    if show_saved_accounts(session) {
        if session.accounts.is_empty() {
            let label = text(cx, "No saved accounts.", "没有已保存账户。").to_string();
            block = block.child(
                div()
                    .id("platform-account-status")
                    .test_support()
                    .aria_label(label.clone())
                    .child(label),
            );
        } else {
            for account in &session.accounts {
                let label = if account.current {
                    format!("{} · {}", account.name, text(cx, "current", "当前"))
                } else {
                    account.name.clone()
                };
                block = block.child(
                    div()
                        .id(SharedString::from(format!(
                            "platform-account-{}",
                            account.name
                        )))
                        .test_support()
                        .h(px(theme::ROW_HEIGHT))
                        .flex()
                        .items_center()
                        .overflow_hidden()
                        .aria_label(label.clone())
                        .child(label),
                );
            }
        }
    }
    if !matches!(session.provider_report, ProviderReport::Unsupported) {
        let label = provider_status(cx, session);
        block = block.child(
            div()
                .id("platform-provider-status")
                .test_support()
                .role(gpui_kit::Role::Status)
                .aria_label(label.clone())
                .child(label),
        );
    }
    block
}

fn auth_status(cx: &gpui_kit::App, session: &PlatformSession) -> String {
    match session.credential {
        CredentialState::PlatformUnsupported => text(
            cx,
            "Accounts are not supported. Login state is not reported.",
            "不支持账户。不报告登录状态。",
        )
        .to_string(),
        CredentialState::StoreUnsupported => text(
            cx,
            "Account credentials are not read for this store. No login state is shown.",
            "此凭据存储不读取账户凭据。不显示登录状态。",
        )
        .to_string(),
        CredentialState::Unreadable => text(
            cx,
            "Accounts could not be read. No login state is shown.",
            "无法读取账户。不显示登录状态。",
        )
        .to_string(),
        CredentialState::NotDetected => text(
            cx,
            "No runtime credential was detected.",
            "未检测到运行凭据。",
        )
        .to_string(),
        CredentialState::Detected if session.platform == Platform::Grok => text(
            cx,
            "Grok runtime auth file is present. This is not a remote login check.",
            "Grok 运行认证文件存在。这不是远程登录检查。",
        )
        .to_string(),
        CredentialState::Detected => text(
            cx,
            "Runtime credential detected. This is not a remote login check.",
            "检测到运行凭据。这不是远程登录检查。",
        )
        .to_string(),
        CredentialState::Invalid => text(
            cx,
            "Runtime credential is not valid. No login success is claimed.",
            "运行凭据无效。不声称登录成功。",
        )
        .to_string(),
    }
}

fn provider_status(cx: &gpui_kit::App, session: &PlatformSession) -> String {
    match &session.provider_report {
        ProviderReport::Unsupported => String::new(),
        ProviderReport::Skipped => text(
            cx,
            "Providers were not read because the settings file was not found.",
            "设置文件不存在，因此没有读取供应商。",
        )
        .to_string(),
        ProviderReport::Invalid => text(
            cx,
            "Providers were not read because the provider entry is not a valid object.",
            "供应商条目不是有效对象，因此没有读取供应商。",
        )
        .to_string(),
        ProviderReport::Ready { hidden: 0 } if session.provider_ids.is_empty() => {
            text(cx, "No providers are configured.", "没有配置供应商。").to_string()
        }
        ProviderReport::Ready { hidden } => {
            let ids = session.provider_ids.join(", ");
            if is_chinese(cx) {
                if *hidden == 0 {
                    format!("供应商：{ids}")
                } else {
                    format!("供应商：{ids}。另有 {hidden} 个标识已隐藏。")
                }
            } else if *hidden == 0 {
                format!("Providers: {ids}")
            } else {
                format!("Providers: {ids}. {hidden} more ids are hidden.")
            }
        }
    }
}

fn show_saved_accounts(session: &PlatformSession) -> bool {
    if !session.accounts.is_empty() {
        return !matches!(
            session.credential,
            CredentialState::PlatformUnsupported | CredentialState::Unreadable
        );
    }
    matches!(
        session.credential,
        CredentialState::NotDetected | CredentialState::Detected | CredentialState::Invalid
    )
}

// Bound debug builder temporaries without changing the native thread stack.
impl PlatformsView {
    #[inline(never)]
    fn render_decision(
        &mut self,
        mut root: gpui_kit::Stateful<gpui_kit::Div>,
        cx: &mut Context<Self>,
        window: &mut Window,
    ) -> gpui_kit::Stateful<gpui_kit::Div> {
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
            let message = match decision {
                Decision::Source => text(cx,"Open plaintext source in this scoped editor? It may contain credentials.","打开可能含明文凭据的临时源编辑区？").to_string(),
                Decision::DeleteAccount => text(cx,"Delete the selected saved account? This does not log out the runtime.","删除所选已保存账户？这不会退出运行账户。").to_string(),
                Decision::DeleteProvider => text(cx,"Delete the selected provider? Other providers remain unchanged.","删除所选供应商？保留其他供应商。").to_string(),
                Decision::AuthOff => text(cx,"Clear runtime authentication? Saved accounts and the selected profile stay unchanged. Active profiles or environment credentials may still provide authentication.","清除运行认证？保留已保存账户和当前配置。活动配置或环境凭据仍可能提供认证。").to_string(),
                Decision::RenameAccount => text(cx,"Rename this saved account to the name entered in the draft? Existing names are never overwritten.","将此保存账户重命名为草稿中的名称？不覆盖已有账户。").to_string(),
                Decision::AddApiKey => text(cx,"Save the entered API key as a new account? It will not switch the runtime account.","将输入的 API 密钥保存为新账户？不会切换运行账户。").to_string(),
                Decision::ImportAccounts => text(cx,"Merge the entered authentication JSON? Existing names are preserved and the runtime account stays unchanged.","合并输入的认证 JSON？保留已有名称，不切换运行账户。").to_string(),
            };
            root = root
                .child(
                    div()
                        .id("platform-confirmation")
                        .test_support()
                        .role(gpui_kit::Role::AlertDialog)
                        .aria_label(text(
                            cx,
                            "Confirm sensitive platform action",
                            "确认敏感平台操作",
                        ))
                        .child(ui_line(cx, &message)),
                )
                .child(
                    div()
                        .tab_group()
                        .tab_stop(false)
                        .flex()
                        .gap(px(theme::CONTROL_GAP))
                        .child(semantics::button(
                            Button::new("platform-decision-cancel")
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
                            Button::new("platform-decision-confirm")
                                .label(text(cx, "Confirm", "确认"))
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

        root
    }

    #[inline(never)]
    fn render_oauth(
        &mut self,
        mut root: gpui_kit::Stateful<gpui_kit::Div>,
        blocked: bool,
        cx: &mut Context<Self>,
        oauth_active: bool,
    ) -> gpui_kit::Stateful<gpui_kit::Div> {
        if self.platform == Platform::Codex {
            let phase = self.oauth_status.as_ref().map(|status| status.phase);
            root = root.child(div().flex().flex_wrap().gap(px(theme::CONTROL_GAP))
                        .child(semantics::button(Button::new("platform-oauth-start")
                            .label(text(cx, "Start OAuth", "开始 OAuth"))
                            .on_click(cx.listener(|this, _, _, cx| this.preflight_oauth_start(cx))),
                            blocked || self.oauth_control_busy, false, cx))
                        .child(semantics::button(Button::new("platform-oauth-cancel")
                            .label(text(cx, "Cancel OAuth", "取消 OAuth")).ghost()
                            .on_click(cx.listener(|this, _, _, cx| {
                                if this.oauth_status.as_ref().is_some_and(|status| status.phase == NativePhase::CleanupFailed) { this.control_oauth(false, cx); }
                                else { this.services.platform_oauth_cancel(); this.message = Some("Cancellation requested; waiting for listener/secret cleanup or an already started commit. / 已请求取消，等待清理或已开始的账户提交。".into()); cx.notify(); }
                            })),
                            (!oauth_active && phase != Some(NativePhase::CleanupFailed)) || self.oauth_control_busy, false, cx)));
            if let Some(status) = &self.oauth_status {
                root = root.child(ui_line(cx, if status.fixture {
                            "Isolated fixture: external authorization and token exchange disabled. Callback reception never proves authentication. / 隔离 fixture：禁用外部授权与令牌交换，收到回调不等于认证成功。"
                        } else { "Authorization URL stays in this scoped session. After callback, explicitly complete account commit. / 授权链接仅保留在本会话，收到回调后请显式完成账户提交。" }))
                        .child(Input::new(&self.oauth_url).readonly(true).aria_label(text(cx, "Scoped OAuth authorization URL", "临时 OAuth 授权链接")))
                        .child(semantics::button(Button::new("platform-oauth-browser")
                            .label(text(cx,"Open authorization in browser","在浏览器中打开授权页")).ghost()
                            .on_click(cx.listener(|this,_,_,cx|this.open_oauth_browser(cx))),
                            status.fixture || self.services.is_fixture() || phase != Some(NativePhase::Waiting) || self.oauth_control_busy,false,cx))
                        .child(Textarea::new(&self.content).h(px(72.)).readonly(self.oauth_control_busy)
                            .aria_label(text(cx, "Paste OAuth callback URL", "粘贴 OAuth 回调地址")))
                        .child(div().flex().flex_wrap().gap(px(theme::CONTROL_GAP))
                            .child(semantics::button(Button::new("platform-oauth-callback")
                                .label(text(cx, "Submit callback", "提交回调")).ghost()
                                .on_click(cx.listener(|this, _, _, cx| this.control_oauth(false, cx))),
                                !oauth_active || self.oauth_control_busy || phase != Some(NativePhase::Waiting), false, cx))
                            .child(semantics::button(Button::new("platform-oauth-complete")
                                .label(text(cx, "Complete account commit", "完成账户提交"))
                                .on_click(cx.listener(|this, _, _, cx| this.control_oauth(true, cx))),
                                !oauth_active || self.oauth_control_busy || phase != Some(NativePhase::Callback), self.oauth_control_busy, cx)));
            }
        }

        root
    }

    #[inline(never)]
    fn render_accounts(
        &mut self,
        mut root: gpui_kit::Stateful<gpui_kit::Div>,
        blocked: bool,
        cx: &mut Context<Self>,
    ) -> gpui_kit::Stateful<gpui_kit::Div> {
        {
            root = root
                .child(
                    Input::new(&self.name)
                        .id("platform-account-name")
                        .aria_label(text(cx, "Account name", "账户名")),
                )
                .child(
                    div()
                        .flex()
                        .gap(px(theme::CONTROL_GAP))
                        .child(semantics::button(
                            Button::new("platform-account-save")
                                .label(text(cx, "Save runtime as account", "保存运行账户"))
                                .on_click(cx.listener(|this, _, window, cx| {
                                    if this.platform == Platform::Codex {
                                        this.review_codex_save(window, cx);
                                        return;
                                    }
                                    let action = if this.platform == Platform::Grok {
                                        AccountAction::SaveFrom(
                                            this.content.read(cx).value().trim().into(),
                                        )
                                    } else {
                                        AccountAction::Save
                                    };
                                    this.account(action, cx)
                                })),
                            blocked,
                            false,
                            cx,
                        ))
                        .child(semantics::button(
                            Button::new("platform-account-switch")
                                .label(text(cx, "Switch account", "切换账户"))
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
                                .label(text(cx, "Delete saved account", "删除保存账户"))
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

        if matches!(
            self.platform,
            Platform::Claude | Platform::Codex | Platform::Grok
        ) {
            root = root.child(semantics::button(
                Button::new("platform-auth-off")
                    .label(text(cx, "Clear runtime authentication", "清除运行认证"))
                    .ghost()
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.decide(Decision::AuthOff, window, cx)
                    })),
                blocked,
                false,
                cx,
            ));
        }
        if self.platform == Platform::Grok {
            root = root.child(text(cx,"To save a Grok account, enter its official OAuth source scope below (issuer::client). Saved accounts use the existing rotation-preservation policy.","保存 Grok 账户时，在下方输入官方 OAuth 来源范围（issuer::client）。保存账户沿用现有轮换凭据保留策略。"))
                .child(Textarea::new(&self.content).aria_label(text(cx,"Grok OAuth source scope","Grok OAuth 来源范围")).h(px(80.)).readonly(blocked));
        }
        if self.platform == Platform::Codex {
            root = self.render_codex_accounts(root, blocked, cx);
        }
        root
    }

    #[inline(never)]
    fn render_codex_accounts(
        &mut self,
        mut root: gpui_kit::Stateful<gpui_kit::Div>,
        blocked: bool,
        cx: &mut Context<Self>,
    ) -> gpui_kit::Stateful<gpui_kit::Div> {
        root = root.child(text(cx,"For rename, enter the new name below. For an API account, enter JSON with api_key and optional base_url. Import accepts auth.json or an account export. Credentials are never prefilled.","重命名时在下方输入新名称。API 账户输入含 api_key 和可选 base_url 的 JSON。导入接受 auth.json 或账户导出。不会预填凭据。"))
            .child(Textarea::new(&self.content).accessibility_id("platform-account-content").aria_label(text(cx,"Account operation draft","账户操作草稿")).h(px(120.)).readonly(blocked));
        let mut actions = div().flex().flex_wrap().gap(px(theme::CONTROL_GAP));
        for (id, en, zh, decision) in [
            (
                "platform-account-rename",
                "Rename account",
                "重命名账户",
                Decision::RenameAccount,
            ),
            (
                "platform-account-api-key",
                "Add API account",
                "添加 API 账户",
                Decision::AddApiKey,
            ),
            (
                "platform-account-import",
                "Import accounts",
                "导入账户",
                Decision::ImportAccounts,
            ),
        ] {
            actions = actions.child(semantics::button(
                Button::new(id).label(text(cx, en, zh)).ghost().on_click(
                    cx.listener(move |this, _, window, cx| this.decide(decision, window, cx)),
                ),
                blocked,
                false,
                cx,
            ));
        }
        root = root.child(actions);
        self.render_quotas(root, blocked, cx)
    }

    #[inline(never)]
    fn render_quotas(
        &mut self,
        mut root: gpui_kit::Stateful<gpui_kit::Div>,
        blocked: bool,
        cx: &mut Context<Self>,
    ) -> gpui_kit::Stateful<gpui_kit::Div> {
        root = root.child(text(cx,"Quota fetch contacts the provider and may refresh saved credentials and quota history. Demo mode disables this action.","配额查询会连接供应方，并可能刷新已保存凭据和配额历史。演示模式禁用此操作。"));
        let mut actions = div().flex().flex_wrap().gap(px(theme::CONTROL_GAP));
        for (id, en, zh, mode) in [
            (
                "platform-quota-selected",
                "Fetch selected quota",
                "查询所选配额",
                0,
            ),
            (
                "platform-quota-current",
                "Fetch runtime quota",
                "查询运行账户配额",
                1,
            ),
            (
                "platform-quota-all",
                "Fetch all saved quotas",
                "查询所有保存账户配额",
                2,
            ),
        ] {
            actions = actions.child(semantics::button(
                Button::new(id)
                    .label(text(cx, en, zh))
                    .ghost()
                    .on_click(cx.listener(move |this, _, _, cx| {
                        if let Some(session) = this.session.clone() {
                            let account = match mode {
                                0 => Some(this.name.read(cx).value().trim().to_string()),
                                1 => Some("default".into()),
                                _ => None,
                            };
                            this.request(Command::Quota { session, account }, cx);
                        }
                    })),
                blocked || self.services.is_fixture(),
                false,
                cx,
            ));
        }
        root = root.child(actions);
        for row in &self.quotas {
            let missing = text(cx, "unavailable", "不可用");
            let hourly = row
                .hourly
                .map(|v| format!("{v}%"))
                .unwrap_or_else(|| missing.into());
            let weekly = row
                .weekly
                .map(|v| format!("{v}%"))
                .unwrap_or_else(|| missing.into());
            let status = if row.failed {
                text(
                    cx,
                    "Quota unavailable; inspect the existing CCR diagnosis.",
                    "配额不可用，请检查现有 CCR 诊断。",
                )
            } else if row.history_warning {
                text(
                    cx,
                    "Quota fetched; history persistence needs attention.",
                    "配额已取得，历史持久化需要检查。",
                )
            } else {
                text(cx, "Quota fetched", "已取得配额")
            };
            root = root.child(
                div()
                    .child(format!("{} · {} · {}", row.account, status, row.fetched_at))
                    .child(format!(
                        "{}: {hourly} · {}: {weekly}",
                        text(cx, "Primary window remaining", "主窗口剩余"),
                        text(cx, "Weekly window remaining", "周窗口剩余")
                    )),
            );
        }
        root
    }

    #[inline(never)]
    fn render_editor(
        &mut self,
        mut root: gpui_kit::Stateful<gpui_kit::Div>,
        blocked: bool,
        cx: &mut Context<Self>,
    ) -> gpui_kit::Stateful<gpui_kit::Div> {
        root = root.child(if self.platform==Platform::Codex && self.surface==Surface::Providers {
            text(cx,"Only submitted provider metadata changes. Credentials remain hidden; blank replacement keys retain them. Source conflicts keep the draft.","仅修改提交的供应商元数据。已有凭据保持隐藏；替换密钥留空会保留原值。来源冲突会保留草稿。")
        } else {
            text(cx, "Only entered dirty keys are patched. Unknown siblings survive. Source editing uses the original loaded revision; conflicts retain the draft. JSON null clearing is unsupported here.", "仅提交输入的变更字段，保留未知字段。源编辑使用原始读取版本，冲突保留草稿。此处 JSON 不支持 null 清除。")
        });
        if self.platform == Platform::Codex && self.surface == Surface::Providers {
            root = root.child(text(cx,"Provider metadata library. Blank replacement keys preserve saved credentials; this does not switch the active runtime provider.","供应商元数据库。替换密钥留空会保留已有凭据；此操作不会切换当前运行供应商。"));
        }
        if self.surface == Surface::Providers {
            root = root
                .child(
                    Input::new(&self.name)
                        .id("platform-provider-id")
                        .aria_label(text(cx, "Provider ID", "供应商标识")),
                )
                .child(semantics::button(
                    Button::new("platform-provider-load-fields")
                        .label(text(cx, "Load provider fields", "载入供应商字段"))
                        .ghost()
                        .on_click(cx.listener(|this, _, window, cx| {
                            let facts = this
                                .session
                                .as_ref()
                                .map(|s| s.provider_fields(&this.name.read(cx).value()))
                                .unwrap_or_default();
                            this.form
                                .load(this.platform, this.surface, &facts, window, cx);
                            this.content
                                .update(cx, |state, cx| state.set_value("", window, cx));
                            this.advanced = false;
                            cx.notify();
                        })),
                    blocked,
                    false,
                    cx,
                ))
                .child(semantics::button(
                    Button::new("platform-provider-delete")
                        .label(text(cx, "Delete provider", "删除供应商"))
                        .ghost()
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.decide(Decision::DeleteProvider, window, cx)
                        })),
                    blocked,
                    false,
                    cx,
                ));
        } else if !self.source_mode {
            root =
                root.child(semantics::button(
                    Button::new("platform-source-open")
                        .label(text(
                            cx,
                            "Open scoped plaintext source",
                            "打开临时明文源文件",
                        ))
                        .ghost()
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.decide(Decision::Source, window, cx)
                        })),
                    blocked,
                    false,
                    cx,
                ));
        }
        if self.platform == Platform::OpenCode && self.surface == Surface::Tui && !self.source_mode
        {
            let mut catalog = div().flex().flex_wrap().gap(px(theme::CONTROL_GAP));
            for (index, &(id, name, _)) in ccr_app::resources::platform_files::opencode::THEMES
                .iter()
                .enumerate()
            {
                catalog = catalog.child(semantics::button(
                    Button::new(("opencode-theme", index))
                        .label(name)
                        .ghost()
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.choose_opencode_theme(id, window, cx)
                        })),
                    blocked,
                    false,
                    cx,
                ));
            }
            root = root.child(
                div()
                    .child(text(
                        cx,
                        "OpenCode TUI theme — choose, then save the draft",
                        "OpenCode TUI 主题：选择后保存草稿",
                    ))
                    .child(catalog),
            );
        }
        if !self.source_mode {
            root = root.child(semantics::button(
                Button::new("platform-advanced-patch")
                    .label(text(cx, "Advanced JSON patch", "高级 JSON 补丁"))
                    .ghost()
                    .toggled(self.advanced)
                    .on_click(cx.listener(|this, _, window, cx| {
                        if !this.advanced {
                            match this.form.patch(&this.content.read(cx).value(), cx) {
                                Ok(value) => this
                                    .content
                                    .update(cx, |state, cx| state.set_value(value, window, cx)),
                                Err(error) => {
                                    this.message = Some(error);
                                    cx.notify();
                                    return;
                                }
                            }
                        } else {
                            let facts = this
                                .session
                                .as_ref()
                                .map(|session| {
                                    if this.surface == Surface::Providers {
                                        session.provider_fields(&this.name.read(cx).value())
                                    } else {
                                        session.fields.clone()
                                    }
                                })
                                .unwrap_or_default();
                            let mut form = forms::SettingsForm::new();
                            form.load(this.platform, this.surface, &facts, window, cx);
                            if let Err(error) =
                                form.sync_draft(&this.content.read(cx).value(), window, cx)
                            {
                                this.message = Some(error);
                                cx.notify();
                                return;
                            }
                            this.form = form;
                        }
                        this.advanced = !this.advanced;
                        cx.notify();
                    })),
                blocked,
                false,
                cx,
            ));
        }
        let editor = if !self.source_mode && !self.advanced {
            self.form.render(blocked, cx)
        } else {
            div()
                .child(
                    Textarea::new(&self.content)
                        .accessibility_id("platform-content")
                        .aria_label(if self.source_mode {
                            text(cx, "Plaintext settings source", "明文设置源文件")
                        } else {
                            text(cx, "Dirty settings JSON object", "变更设置 JSON 对象")
                        })
                        .h(px(180.))
                        .readonly(blocked),
                )
                .into_any_element()
        };
        root = root
            .child(semantics::button(
                Button::new("platform-save")
                    .label(if self.source_mode {
                        text(
                            cx,
                            "Save source with original revision",
                            "按原始版本保存源文件",
                        )
                    } else {
                        text(cx, "Save dirty patch", "保存变更字段")
                    })
                    .on_click(cx.listener(|this, _, _, cx| this.save(cx))),
                blocked,
                self.busy,
                cx,
            ))
            .child(editor);

        root
    }
}
impl Render for PlatformsView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.pending_form {
            self.pending_form = false;
            let facts = self
                .session
                .as_ref()
                .map(|s| s.fields.clone())
                .unwrap_or_default();
            self.form
                .load(self.platform, self.surface, &facts, window, cx);
        }
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
        let blocked =
            self.busy || self.decision.is_some() || oauth_active || self.diagnostic_blocked();
        let mut tabs = div().flex().flex_wrap().gap(px(theme::CONTROL_GAP));
        if !self.pinned {
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
        }
        let title = if self.pinned {
            match self.platform {
                Platform::Claude => text(cx, "Claude Code", "Claude Code"),
                Platform::Codex => text(cx, "Codex", "Codex"),
                Platform::Grok => text(cx, "Grok", "Grok"),
                Platform::Gemini => text(cx, "Antigravity", "Antigravity"),
                Platform::OpenCode => text(cx, "OpenCode", "OpenCode"),
                Platform::Qwen | Platform::Droid => {
                    text(cx, "Platform settings and accounts", "平台设置与账户")
                }
            }
        } else {
            text(cx, "Platform settings and accounts", "平台设置与账户")
        };
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
            .when(!self.pinned, |root| {
                root.child(div().text_size(px(theme::TITLE_SIZE)).child(title))
            })
            .child(tabs);
        let mut surfaces = div().flex().flex_wrap().gap(px(theme::CONTROL_GAP));
        let descriptor = DESCRIPTORS
            .iter()
            .find(|descriptor| descriptor.platform == self.platform);
        for (id, surface, en, zh, enabled) in [
            (
                "platform-settings",
                Surface::Settings,
                "Settings",
                "设置",
                descriptor.is_some_and(|value| value.settings),
            ),
            (
                "platform-auth",
                Surface::Auth,
                "Accounts",
                "账户",
                descriptor.is_some_and(|value| value.auth),
            ),
            (
                "platform-providers",
                Surface::Providers,
                "Providers",
                "供应商",
                descriptor.is_some_and(|value| value.providers),
            ),
            (
                "platform-models",
                Surface::Models,
                "Models",
                "模型目录",
                self.platform == Platform::Codex,
            ),
            (
                "platform-tui",
                Surface::Tui,
                "TUI",
                "终端界面",
                self.platform == Platform::OpenCode,
            ),
        ] {
            if enabled {
                surfaces = surfaces.child(semantics::button(
                    Button::new(id)
                        .label(text(cx, en, zh))
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
        if !self.pinned {
            surfaces = surfaces.child(semantics::button(
                Button::new("platform-legacy-antigravity")
                    .label(text(cx, "Advanced: Antigravity source", "高级源文件"))
                    .ghost()
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.select(Platform::Gemini, Surface::LegacySettings, window, cx)
                    })),
                blocked,
                false,
                cx,
            ));
        }
        if !self.pinned {
            root = root.child(surfaces);
        } else if self.platform == Platform::OpenCode {
            // 平台 tab 已覆盖设置与供应商，但 OpenCode 的 tui.json 没有对应 tab：
            // 嵌入时保留一个子入口，避免这次布局改动删掉已有能力。
            let mut tui = div().flex().flex_wrap().gap(px(theme::CONTROL_GAP));
            for (id, surface, en, zh) in [
                ("platform-tui", Surface::Tui, "TUI", "终端界面"),
                ("platform-settings", Surface::Settings, "Settings", "设置"),
            ] {
                if self.surface == surface {
                    continue;
                }
                tui = tui.child(semantics::button(
                    Button::new(id)
                        .label(text(cx, en, zh))
                        .ghost()
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.select(this.platform, surface, window, cx)
                        })),
                    blocked,
                    false,
                    cx,
                ));
            }
            root = root.child(tui);
        }
        if matches!(
            self.platform,
            Platform::Claude | Platform::Codex | Platform::Grok
        ) {
            root = root.child(semantics::button(
                Button::new("platform-show-layers")
                    .label(text(cx, "Configuration sources", "配置来源"))
                    .ghost()
                    .selected(self.layers_open)
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.layers_open = !this.layers_open;
                        let platform = this.platform;
                        let open = this.layers_open;
                        this.layers.update(cx, |view, cx| {
                            if open {
                                view.open(platform, cx);
                            } else {
                                view.suspend(cx);
                            }
                        });
                        cx.notify();
                    })),
                blocked,
                false,
                cx,
            ));
        }
        if self.layers_open {
            return root.child(self.layers.clone());
        }
        root = self.render_decision(root, cx, window);
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
            root = root.child(local_projection(
                session,
                matches!(self.surface, Surface::Models | Surface::Auth)
                    || self.source_mode
                    || self.advanced,
                cx,
            ));
            if self.surface == Surface::Auth {
                root = self.render_oauth(root, blocked, cx, oauth_active);
                root = self.render_accounts(root, blocked, cx);
                if self.platform == Platform::Codex {
                    root = root.child(self.render_codex_diagnostics(window, cx));
                }
            } else if self.surface == Surface::Models {
                root = root.child(text(cx,"Local built-in and custom model catalog. This list does not establish provider availability.","本地内置与自定义模型目录。此列表不代表供应方可用性。"));
            } else if self.platform != Platform::Gemini {
                root = self.render_editor(root, blocked, cx);
            }
        } else if self.busy {
            root = root.child(text(
                cx,
                "Reading local settings. No result yet.",
                "正在读取本地设置。还没有结果。",
            ));
        } else {
            root = root.child(text(cx, "Load the selected local surface to begin. Fixture roots are isolated and visibly labeled by the shell.", "读取所选本地功能后开始。演示路径已隔离并由外壳标记。"));
        }
        if let Some(message) = &self.message {
            root = root.child(
                div()
                    .id("platform-operation-status")
                    .role(gpui_kit::Role::Status)
                    .child(ui_line(cx, message)),
            );
        }
        root
    }
}

#[cfg(test)]
mod tests;
