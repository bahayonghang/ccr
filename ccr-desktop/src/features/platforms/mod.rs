//! Native platform settings, scoped source, accounts and OpenCode providers.
#[cfg(test)]
use ccr_app::runtime::EnvironmentId;
mod accounts;
use accounts::AccountForm;
mod codex_tools;
use codex_tools::{CodexTools, Workspace};
mod diagnostics;
mod forms;
mod layers;
mod scope;
use crate::{
    locales::{is_chinese, text},
    runtime::{
        Services,
        platforms::{
            AccountAction, AccountRow, Command, CredentialState, DESCRIPTORS, FileState,
            OAuthControl, OAuthResult, PlatformSession, ProviderReport, ResultValue, Surface,
        },
    },
    shell::{badge, semantics},
    theme,
};
use ccr_app::{
    platform_settings::SaveOutcome,
    runtime::{Completion, Platform, RequestKey, ResponseGate, TaskEvent},
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
    DiscardAccountForm,
    DiscardCodexTools(Option<Workspace>),
    LeaveAccountPage(Platform, Surface),
}
pub struct PlatformsView {
    form: forms::SettingsForm,
    pending_form: bool,
    advanced: bool,
    layers: Entity<layers::LayersView>,
    layers_open: bool,
    /// 键盘激活配置来源开关时记下的开关焦点；收起展开区后焦点回到这里。
    layers_toggle_focus: Option<FocusHandle>,
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
    oauth_watch: Option<Task<()>>,
    oauth_control_task: Option<Task<()>>,
    oauth_generation: u64,
    oauth_status: Option<NativeStatus>,
    oauth_control_phase: Option<NativePhase>,
    quotas: Vec<crate::runtime::platforms::QuotaRow>,
    diagnostics: diagnostics::ViewState,
    /// 已保存账户列表中的选中项；切换、重命名、删除只作用于它。
    selected_account: Option<String>,
    account_form: Option<AccountForm>,
    account_input: Entity<InputState>,
    account_scope: Entity<InputState>,
    account_return: Option<FocusHandle>,
    account_focus: std::collections::HashMap<String, FocusHandle>,
    account_summary_focus: FocusHandle,
    tools: CodexTools,
    account_feedback: Option<String>,
    clear_account_draft: bool,
}
impl PlatformsView {
    pub fn has_environment_draft(&self, cx: &gpui_kit::App) -> bool {
        self.account_draft_dirty(cx)
            || self.form.has_changes(cx)
            || self.source_mode
            || self.decision.is_some()
            || !self.name.read(cx).value().is_empty()
            || (self.advanced && self.content.read(cx).value().as_str() != "{}")
    }
    pub fn new(services: Services, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let layers = cx.new(|cx| layers::LayersView::new(services.clone(), window, cx));
        Self {
            form: forms::SettingsForm::new(),
            pending_form: false,
            advanced: false,
            layers,
            layers_open: false,
            layers_toggle_focus: None,
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
            oauth_watch: None,
            oauth_control_task: None,
            oauth_generation: 0,
            oauth_status: None,
            oauth_control_phase: None,
            quotas: Vec::new(),
            diagnostics: Default::default(),
            selected_account: None,
            account_form: None,
            account_input: cx.new(|cx| InputState::new(window, cx).validate(|v, _| v.len() <= 32)),
            account_scope: cx.new(|cx| InputState::new(window, cx)),
            account_return: None,
            account_focus: Default::default(),
            account_summary_focus: cx.focus_handle().tab_stop(false),
            tools: CodexTools::new(window, cx),
            account_feedback: None,
            clear_account_draft: false,
        }
    }
    pub fn focus_main(&self, window: &mut Window, cx: &mut Context<Self>) {
        if self.surface == Surface::Auth {
            window.focus(&self.account_summary_focus, cx);
        } else if self.surface == Surface::Providers {
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
        self.reset_account_form(window, cx);
        self.account_return = None;
        self.account_feedback = None;
        self.account_focus.clear();
        self.tools.clear(window, cx);
        self.tools.active = None;
        self.tools.login_feedback = None;
        self.tools.quota_target = None;
        self.tools.quota_error = None;
        self.tools.quota_history.clear();
        self.tools.quota_pending = false;
        self.clear_account_draft = false;
        self.diagnostics = Default::default();
        self.form.clear();
        self.pending_form = false;
        self.advanced = false;
        self.layers.update(cx, |view, cx| view.suspend(cx));
        self.layers_open = false;
        self.layers_toggle_focus = None;
        if self.oauth_task.is_some() && self.services.platform_oauth_active() {
            self.tools.cancelling = true;
            self.services.platform_oauth_cancel();
        }
        self.oauth_task = None;
        self.oauth_watch = None;
        self.oauth_control_task = None;
        self.oauth_generation += 1;
        self.oauth_status = None;
        self.oauth_control_phase = None;
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
        self.selected_account = None;
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
    fn select_from_page(
        &mut self,
        platform: Platform,
        surface: Surface,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.platform == platform && self.surface == surface {
            return;
        }
        if self.account_draft_dirty(cx) {
            self.decide(Decision::LeaveAccountPage(platform, surface), window, cx);
        } else {
            self.select(platform, surface, window, cx);
        }
    }
    pub fn load(&mut self, cx: &mut Context<Self>) {
        if self.surface == Surface::Auth && !self.busy {
            self.tools.quota_history.clear();
            self.tools.quota_target = None;
            self.tools.quota_error = None;
            self.quotas.clear();
        }
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
        let account_result = self.account_result(&command, cx);
        let is_quota = matches!(&command, Command::Quota { .. });
        let workspace_revision = self.tools.revision;
        self.tools.quota_pending = is_quota;
        self.generation += 1;
        let key = RequestKey {
            environment: self.services.environment(),
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
                            this.tools.quota_pending = false;
                            if is_quota && workspace_revision != this.tools.revision {
                                cx.notify();
                                return;
                            }
                            match result.as_ref() {
                                Completion::Success(ResultValue::Quotas(rows)) => {
                                    let runtime = matches!(this.tools.quota_target,Some(crate::runtime::platforms::QuotaTarget::Runtime));
                                    for row in rows.iter().filter(|row| !row.failed && row.account != "[hidden]") {
                                        use crate::runtime::platforms::QuotaTarget;
                                        let target = if runtime {QuotaTarget::Runtime} else {QuotaTarget::Saved(row.account.clone())};
                                        this.tools.quota_history.insert(target,row.clone());
                                    }
                                    this.quotas = rows.clone();
                                    this.tools.quota_error = None;
                                }
                                Completion::Success(ResultValue::Loaded(session)) => {
                                    this.pending_form = true;
                                    // 重读后只保留仍存在的选中账户，不自动选第一行。
                                    this.selected_account = this.selected_account.take().filter(|name| {
                                        session.accounts.iter().any(|account| &account.name == name)
                                    });
                                    this.account_focus.retain(|name, _| session.accounts.iter().any(|row| &row.name == name));
                                    this.session = Some(session.clone());
                                    if this.source_mode { this.pending_content = Some(session.snapshot.content.clone()); }
                                    this.message = this.account_feedback.take();
                                }
                                Completion::Success(ResultValue::Saved(SaveOutcome::Conflict)) => this.message = Some("Source changed. Draft and original revision retained. Discard and reload before saving again. / 文件已变化，保留草稿和原始版本，请丢弃并重读".into()),
                                Completion::Success(ResultValue::SavedWithWarning { outcome: SaveOutcome::Conflict }) => this.message = Some("Source changed. Reload before saving again. / 文件已变化，请重新读取后保存。".into()),
                                Completion::Success(ResultValue::Saved(SaveOutcome::Saved { .. }) | ResultValue::SavedWithWarning { outcome: SaveOutcome::Saved { .. } } | ResultValue::AccountChanged { .. }) => {
                                    // 已提交后必须重新读取，不能携旧版本重复提交。
                                    this.session = None;
                                    // 账户集合可能已变化（重命名、删除），旧选中项不再可信。
                                    this.selected_account = None;
                                    this.pending_form = true;
                                    this.source_mode = false;
                                    this.pending_content = Some(String::new());
                                    this.message = Some(if matches!(result.as_ref(), Completion::Success(ResultValue::SavedWithWarning { .. })) {
                                        "Settings saved. The application operation record could not be saved; reload before another edit. / 设置已保存，但应用操作记录未保存，请重新读取后再编辑。"
                                    } else if matches!(result.as_ref(), Completion::Success(ResultValue::AccountChanged { warning: true })) {
                                        if matches!(this.services.environment(), ccr_app::runtime::EnvironmentId::Ssh(_)) {
                                            "Operation committed, but authentication suppressors or operation-record warnings remain. Reload and inspect CCR before retrying. / 操作已提交，但仍有认证抑制源或操作记录警告，请重读并检查 CCR。"
                                        } else {
                                            "Account operation committed, but authentication suppressors or diagnosis warnings remain. Reload and inspect existing CCR diagnosis before retrying. / 账户操作已提交，但仍有认证抑制源或诊断警告，请重读并检查 CCR 诊断。"
                                        }
                                    } else {
                                        "Operation committed. Reload to start a new edit session. / 操作已提交，请重新读取以开始新编辑"
                                    }.into());
                                    if let Some(feedback) = &account_result {
                                        this.tools.quota_history.clear();
                                        this.tools.quota_target = None;
                                        this.quotas.clear();
                                        let warned = matches!(result.as_ref(), Completion::Success(ResultValue::AccountChanged { warning: true }));
                                        this.account_feedback = Some(if warned {format!("{} · {}",feedback, text(cx,"Review authentication suppressors and the operation record before retrying.","再次操作前请检查认证抑制源及操作记录。"))} else {feedback.clone()});
                                        this.clear_account_draft = true;
                                        this.load(cx);
                                    }
                                }
                                Completion::Failed(error) | Completion::CleanupFailed(error) if is_quota => this.tools.quota_error = Some(error.message().into()),
                                Completion::WorkerFailed | Completion::Cancelled if is_quota => this.tools.quota_error = Some(text(cx,"Query did not complete","查询未完成").into()),
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
            Err(_) => {
                self.busy = false;
                self.tools.quota_pending = false;
                let error = "Queue busy or application shutting down. Retry. / 队列繁忙或应用正在退出";
                if is_quota {self.tools.quota_error = Some(error.into());} else {self.message = Some(error.into());}
            }
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
            environment: self.services.environment(),
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
            || self.tools.active != Some(Workspace::Login)
            || session.credential.file_account_write_access().is_err()
            || self.services.platform_oauth_active()
        {
            return;
        }
        self.tools.cancelling = false;
        self.oauth_watch = None;
        self.oauth_generation += 1;
        let generation = self.oauth_generation;
        match self
            .services
            .platform_oauth_start(self.oauth_key(), session)
        {
            Ok(mut subscription) => {
                self.oauth_status = None;
                self.tools.login_feedback = Some("Starting owned OAuth listener. Callback reception is separate from account commit. / 正在启动 OAuth，收到回调不代表账户已提交。".into());
                self.oauth_task = Some(cx.spawn(async move |weak, cx| {
                    while let Some(event) = subscription.next().await {
                        let terminal = matches!(event, TaskEvent::Terminal(_));
                        let _ = weak.update(cx, |this, cx| {
                            if this.oauth_generation != generation { return; }
                            this.oauth_status = this.services.platform_oauth_status();
                            if let TaskEvent::Terminal(result) = event {
                                this.oauth_task = None;
                                this.tools.cancelling = false;
                                this.tools.clear_callback = true;
                                this.tools.login_feedback = Some(match result.as_ref() {
                                    Completion::Success(OAuthResult::Finished) => {
                                        this.load(cx);
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
                self.tools.login_feedback = Some(
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
        self.tools.login_feedback = Some(
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
        if self.tools.active != Some(Workspace::Login)
            || self.oauth_control_phase.is_some()
            || self.tools.cancelling
            || !matches!(
                (complete, status.phase),
                (true, NativePhase::Callback)
                    | (false, NativePhase::Waiting | NativePhase::CleanupFailed)
            )
        {
            return;
        }
        if status.phase == NativePhase::Waiting
            && self.tools.callback.read(cx).value().trim().is_empty()
        {
            self.tools.login_feedback =
                Some(text(cx, "Paste the callback URL first.", "请先粘贴回调地址。").into());
            cx.notify();
            return;
        }
        let control = if status.phase == NativePhase::CleanupFailed {
            OAuthControl::RetryCleanup
        } else if complete {
            OAuthControl::Complete {
                login_id: status.login_id.clone(),
                name: Some(self.tools.name.read(cx).value().to_string())
                    .filter(|n| !n.trim().is_empty()),
            }
        } else {
            OAuthControl::Callback {
                login_id: status.login_id.clone(),
                url: self.tools.callback.read(cx).value().to_string(),
            }
        };
        let generation = self.oauth_generation;
        match self
            .services
            .platform_oauth_control(self.oauth_key(), control)
        {
            Ok(mut subscription) => {
                self.oauth_control_phase = Some(status.phase);
                self.oauth_control_task = Some(cx.spawn(async move |weak, cx| {
                    while let Some(event) = subscription.next().await {
                        if let TaskEvent::Terminal(result) = event {
                            let _ = weak.update(cx, |this, cx| {
                                if this.oauth_generation != generation { return; }
                                this.oauth_control_phase = None;
                                this.oauth_status = this.services.platform_oauth_status();
                                this.tools.login_feedback = Some(match result.as_ref() {
                                    Completion::Success(OAuthResult::CallbackAccepted) => "Callback accepted. Account has not been committed; choose Complete. / 已接受回调，账户尚未提交，请选择完成。",
                                    Completion::Success(OAuthResult::AccountCommitted) => "Account committed. Reload accounts. / 账户已提交，请重读账户。",
                                    Completion::Success(OAuthResult::CleanupFinished) => "Pending OAuth secrets cleaned; login can be retried. / OAuth 临时凭据已清理，可重新登录。",
                                    _ => "OAuth control rejected or failed. No success claimed; current owner remains authoritative. / OAuth 操作被拒绝或失败，以当前任务状态为准。",
                                }.into());
                                if matches!(result.as_ref(), Completion::Success(_)) { this.tools.clear_callback = true; }
                                cx.notify();
                            });
                            break;
                        }
                    }
                }));
            }
            Err(_) => {
                self.tools.login_feedback =
                    Some("OAuth control queue unavailable. / OAuth 操作队列不可用。".into())
            }
        }
        cx.notify();
    }
    fn account(&mut self, name: String, action: AccountAction, cx: &mut Context<Self>) {
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
    /// 与渲染时的禁用条件一致：忙碌、确认中、OAuth 或诊断复核期间不接受新操作。
    fn blocked(&self) -> bool {
        self.busy
            || self.decision.is_some()
            || self.services.platform_oauth_active()
            || self.diagnostic_blocked()
    }
    /// 只返回仍在当前会话账户列表中的选中项，避免对过期对象操作。
    fn selected_account(&self) -> Option<String> {
        let name = self.selected_account.as_ref()?;
        self.session
            .as_ref()?
            .accounts
            .iter()
            .any(|account| &account.name == name)
            .then(|| name.clone())
    }
    /// 配置来源展开区开关：只读清单在页内展开，收起时回收请求并把焦点还给开关。
    fn toggle_layers(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.layers_open = !self.layers_open;
        let platform = self.platform;
        let open = self.layers_open;
        self.layers.update(cx, |view, cx| {
            if open {
                view.open(platform, cx);
            } else {
                view.suspend(cx);
            }
        });
        if !open && let Some(focus) = self.layers_toggle_focus.clone() {
            window.focus(&focus, cx);
        }
        cx.notify();
    }
    fn select_account(&mut self, name: String, cx: &mut Context<Self>) {
        if self.blocked() {
            return;
        }
        self.selected_account = Some(name);
        cx.notify();
    }
    fn decide(&mut self, decision: Decision, window: &mut Window, cx: &mut Context<Self>) {
        if self.busy || self.decision.is_some() {
            return;
        }
        // 针对已保存账户的确认捕获列表选中项；新建类动作才读取新账户名输入。
        let name = match decision {
            Decision::DeleteAccount | Decision::RenameAccount => {
                let Some(name) = self.selected_account() else {
                    return;
                };
                name
            }
            Decision::AuthOff => self.matched_account().unwrap_or_default().to_owned(),
            Decision::AddApiKey | Decision::ImportAccounts => {
                self.tools.name.read(cx).value().to_string()
            }
            _ => self.name.read(cx).value().to_string(),
        };
        self.decision = Some(decision);
        self.decision_name = name;
        self.decision_content = match decision {
            Decision::RenameAccount => self.account_input.read(cx).value().to_string(),
            Decision::AddApiKey => self.tools.api.read(cx).value().to_string(),
            Decision::ImportAccounts => self.tools.import.read(cx).value().to_string(),
            _ => String::new(),
        };
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
            Some(Decision::LeaveAccountPage(platform, surface)) => {
                self.select(platform, surface, window, cx)
            }
            Some(Decision::DiscardCodexTools(next)) => self.show_workspace(next, window, cx),
            Some(Decision::DiscardAccountForm) => self.close_account_form(window, cx),
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

/// 每个 tab 只显示自己那部分：账户 tab 只陈述凭据观测与账户来源；设置类 tab 只陈述设置文件，
/// 不显示凭据观测或已保存账户。设置文档仍在后台作为账户 CAS 令牌来源，只是不在账户 tab 上渲染。
/// `providers`：供应商状态是否随本段输出；运行设置页把它放进单独的供应商目录分组。
fn local_projection(
    session: &PlatformSession,
    show_fields: bool,
    providers: bool,
    cx: &gpui_kit::App,
) -> impl IntoElement {
    let accounts = session.surface == Surface::Auth;
    let (id, status) = if accounts {
        ("platform-auth-status", auth_status(cx, session))
    } else {
        ("platform-file-status", file_status(cx, session))
    };
    let mut block = div()
        .flex()
        .flex_col()
        .gap(px(theme::CONTROL_GAP))
        .text_size(crate::theme::text_size(theme::BODY_SIZE))
        .child(
            div()
                .id(id)
                .test_support()
                .role(gpui_kit::Role::Status)
                .aria_label(status.clone())
                .child(status),
        );
    // 账户 tab 只保留账户来源（Grok oauth_source），运行设置字段属于设置 tab。
    let fields = session
        .fields
        .iter()
        .enumerate()
        .filter(|(_, field)| show_fields && (!accounts || field.key == "oauth_source"));
    for (index, field) in fields {
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
                .h(crate::theme::text_size(theme::ROW_HEIGHT))
                .flex()
                .items_center()
                .overflow_hidden()
                .aria_label(label.clone())
                .child(label),
        );
    }
    if providers && let Some(line) = provider_line(session, cx) {
        block = block.child(line);
    }
    block
}

fn no_saved_accounts(cx: &gpui_kit::App) -> gpui_kit::AnyElement {
    let label = text(cx, "No saved accounts.", "没有已保存账户。").to_string();
    div()
        .id("platform-account-status")
        .test_support()
        .aria_label(label.clone())
        .child(label)
        .into_any_element()
}

fn account_label(account: &AccountRow, cx: &gpui_kit::App) -> String {
    if account.current {
        format!("{} · {}", account.name, text(cx, "current", "当前"))
    } else {
        account.name.clone()
    }
}

fn provider_line(session: &PlatformSession, cx: &gpui_kit::App) -> Option<gpui_kit::AnyElement> {
    if matches!(session.provider_report, ProviderReport::Unsupported) {
        return None;
    }
    let label = provider_status(cx, session);
    Some(
        div()
            .id("platform-provider-status")
            .test_support()
            .role(gpui_kit::Role::Status)
            .aria_label(label.clone())
            .child(label)
            .into_any_element(),
    )
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
    !session.accounts.is_empty()
        || !matches!(
            session.credential,
            CredentialState::PlatformUnsupported | CredentialState::Unreadable
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
                Decision::DiscardAccountForm | Decision::DiscardCodexTools(_) | Decision::LeaveAccountPage(_,_) => text(cx,"Discard the account form entries?","丢弃尚未提交的账户表单？").to_owned(),
                Decision::AuthOff if !self.decision_name.is_empty() => if is_chinese(cx) {
                    format!("清除运行账户「{}」的认证？保留已保存的账户和配置方案。活动配置或环境变量仍可能提供凭据。",self.decision_name)
                } else {
                    format!("Clear runtime authentication for “{}”? Saved accounts and profiles are retained. An active profile or environment may still supply credentials.",self.decision_name)
                },
                Decision::Source => text(cx,"Open plaintext source in this scoped editor? It may contain credentials.","打开可能含明文凭据的临时源编辑区？").to_string(),
                // 确认列明对象：名称来自已过滤的安全账户名，不含凭据。
                Decision::DeleteAccount if is_chinese(cx) => format!("删除已保存账户「{}」？这不会退出运行账户。", self.decision_name),
                Decision::DeleteAccount => format!("Delete saved account “{}”? This does not log out the runtime.", self.decision_name),
                Decision::DeleteProvider => text(cx,"Delete the selected provider? Other providers remain unchanged.","删除所选供应商？保留其他供应商。").to_string(),
                Decision::AuthOff => text(cx,"Clear runtime authentication? Saved accounts and the selected profile stay unchanged. Active profiles or environment credentials may still provide authentication.","清除运行认证？保留已保存账户和当前配置。活动配置或环境凭据仍可能提供认证。").to_string(),
                Decision::RenameAccount if is_chinese(cx) => format!("将已保存账户「{}」重命名为草稿中的名称？不覆盖已有账户。", self.decision_name),
                Decision::RenameAccount => format!("Rename saved account “{}” to the name entered in the draft? Existing names are never overwritten.", self.decision_name),
                Decision::AddApiKey => text(cx,"Save the entered API key as a new account? It will not switch the runtime account.","将输入的 API 密钥保存为新账户？不会切换运行账户。").to_string(),
                Decision::ImportAccounts => text(cx,"Merge the entered authentication JSON? Existing names are preserved and the runtime account stays unchanged.","合并输入的认证 JSON？保留已有名称，不切换运行账户。").to_string(),
            };
            root = root
                .child(
                    div()
                        .id("platform-confirmation")
                        .test_support()
                        .role(gpui_kit::Role::AlertDialog)
                        .aria_label(message.clone())
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
        if self.platform == Platform::Gemini && !self.source_mode {
            return root;
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
        if std::mem::take(&mut self.clear_account_draft) {
            self.reset_account_form(window, cx);
            self.tools.clear(window, cx);
        }
        if std::mem::take(&mut self.tools.clear_callback) {
            self.tools.callback = cx.new(|cx| InputState::new(window, cx).masked(true));
            if self.tools.active == Some(Workspace::Login)
                && self.oauth_status.as_ref().is_some_and(|s| {
                    matches!(
                        s.phase,
                        NativePhase::Succeeded
                            | NativePhase::Failed
                            | NativePhase::Cancelled
                            | NativePhase::TimedOut
                            | NativePhase::CleanupFailed
                    )
                })
            {
                self.tools.name =
                    cx.new(|cx| InputState::new(window, cx).validate(|v, _| v.len() <= 32));
            }
        }

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
        let blocked = self.blocked();
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
                            this.select_from_page(platform, Surface::Settings, window, cx)
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
                    if event.keystroke.key == "escape" {
                        if this.decision.is_some() {
                            this.dismiss(window, cx);
                        } else if this.account_form.is_some() && !this.diagnostic_blocked() {
                            this.cancel_account_form(window, cx);
                        } else if this.tools.active.is_some()
                            && !this.diagnostic_navigation_blocked()
                        {
                            this.change_workspace(None, window, cx);
                        } else {
                            return;
                        }
                        cx.stop_propagation();
                    }
                }),
            )
            .when(!self.pinned, |root| {
                root.child(
                    div()
                        .text_size(crate::theme::text_size(theme::TITLE_SIZE))
                        .child(title),
                )
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
                            this.select_from_page(this.platform, surface, window, cx)
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
        // 配置来源是设置 tab 的页内开关，账户 tab 不显示。
        if matches!(self.surface, Surface::Settings | Surface::LegacySettings)
            && (matches!(
                self.platform,
                Platform::Claude | Platform::Codex | Platform::Grok
            ) || matches!(
                self.services.environment(),
                ccr_app::runtime::EnvironmentId::Ssh(_)
            ))
        {
            root = root.child(semantics::button(
                Button::new("platform-show-layers")
                    .label(text(cx, "Configuration sources", "配置来源"))
                    .ghost()
                    .selected(self.layers_open)
                    .on_click(
                        cx.listener(|this, event: &gpui_kit::ClickEvent, window, cx| {
                            // 键盘激活时焦点就在开关上；鼠标按下不移动焦点，所以只在键盘激活时记录。
                            if event.is_keyboard() {
                                this.layers_toggle_focus = window.focused(cx);
                            }
                            this.toggle_layers(window, cx);
                        }),
                    ),
                blocked,
                false,
                cx,
            ));
        }
        root = self.render_decision(root, cx, window);
        if self.surface != Surface::Auth {
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
        }
        if let Some(session) = self.session.clone() {
            // 运行设置、供应商目录、CCR 元数据各自成组，分别陈述对象、来源与实际作用域。
            let category = scope::category(self.platform, self.surface);
            if let Some(category) = category {
                root = root.child(scope::group(category, &session, None, cx));
            }
            if self.surface != Surface::Auth {
                root = root.child(local_projection(
                    &session,
                    self.surface == Surface::Models || self.source_mode || self.advanced,
                    category != Some(scope::Category::Runtime),
                    cx,
                ));
            }
            if self.surface == Surface::Auth {
                if self.platform == Platform::Codex && self.tools.active.is_some() {
                    let blocked =
                        blocked || session.credential.file_account_write_access().is_err();
                    root = self.render_workspaces(root, window, cx, blocked, oauth_active);
                } else {
                    root = self.render_accounts(root, blocked, window, cx, oauth_active);
                }
            } else if self.surface == Surface::Models {
                root = root.child(if session.remote.is_some() {
                    text(cx,"Built-in models and this SSH host's custom model catalog. This list does not establish provider availability.","内置模型与此 SSH 主机的自定义模型目录。此列表不代表供应方可用性。")
                } else { text(cx,"Local built-in and custom model catalog. This list does not establish provider availability.","本地内置与自定义模型目录。此列表不代表供应方可用性。") });
            } else if self.platform != Platform::Gemini || session.remote.is_some() {
                root = self.render_editor(root, blocked, cx);
            }
            // 运行设置文件里的供应商条目（OpenCode）单独归入供应商目录，不与运行设置合并。
            if category == Some(scope::Category::Runtime)
                && let Some(line) = provider_line(&session, cx)
            {
                root = root.child(scope::group(
                    scope::Category::ProviderCatalog,
                    &session,
                    Some(line),
                    cx,
                ));
            }
        } else if self.surface == Surface::Auth {
            root = root
                .child(badge::section_heading(
                    text(cx, "Runtime credentials", "运行凭据"),
                    None,
                    cx,
                ))
                .child(text(
                    cx,
                    if self.busy {
                        "Reading account files…"
                    } else {
                        "Account files have not been loaded."
                    },
                    if self.busy {
                        "正在读取账户文件…"
                    } else {
                        "尚未读取账户文件。"
                    },
                ))
                .child(self.reload_accounts(self.busy, cx));
        } else if self.busy {
            root = root.child(text(
                cx,
                "Reading current environment settings. No result yet.",
                "正在读取当前执行环境的设置。还没有结果。",
            ));
        } else {
            root = root.child(text(
                cx,
                "Load settings from the current execution environment to begin.",
                "读取当前执行环境的设置后开始。",
            ));
        }
        // 配置来源是设置表单之后的页内只读展开区；表单与保存仍在原位可用。
        // 复用同一个 LayersView 实体，展开与收起不重建焦点所有者。
        if self.layers_open && matches!(self.surface, Surface::Settings | Surface::LegacySettings) {
            root = root.child(self.layers.clone());
        }
        if let Some(message) = &self.message {
            root = root.child(
                div()
                    .id("platform-operation-status")
                    .aria_label(message.clone())
                    .role(gpui_kit::Role::Status)
                    .child(ui_line(cx, message)),
            );
        }
        root
    }
}

#[cfg(test)]
mod tests;
