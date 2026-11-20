use super::*;
use crate::runtime::platforms::{QuotaRow, QuotaTarget};
use gpui_kit::{AnyElement, App, Div, Role, Stateful};
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Workspace {
    Login,
    Add,
    Quota,
    Diagnostics,
}

impl Workspace {
    fn entry(self, cx: &App) -> (&'static str, &'static str) {
        match self {
            Self::Login => ("platform-tools-login", text(cx, "Log in", "登录")),
            Self::Add => (
                "platform-tools-add",
                text(cx, "Add / import accounts", "添加／导入账户"),
            ),
            Self::Quota => ("platform-tools-quota", text(cx, "Quota", "配额")),
            Self::Diagnostics => (
                "platform-tools-diagnostics",
                text(cx, "Callback diagnostics", "回调诊断"),
            ),
        }
    }
}

pub(super) struct CodexTools {
    pub active: Option<Workspace>,
    pub name: Entity<InputState>,
    pub api: Entity<InputState>,
    pub import: Entity<InputState>,
    pub callback: Entity<InputState>,
    pub clear_callback: bool,
    pub cancelling: bool,
    pub login_feedback: Option<String>,
    pub quota_target: Option<QuotaTarget>,
    pub quota_error: Option<String>,
    pub quota_history: HashMap<QuotaTarget, QuotaRow>,
    pub revision: u64,
    pub quota_pending: bool,
    focus: FocusHandle,
    entries: [FocusHandle; 4],
}
impl CodexTools {
    pub fn new(window: &mut Window, cx: &mut Context<PlatformsView>) -> Self {
        Self {
            active: None,
            name: cx.new(|cx| InputState::new(window, cx).validate(|v, _| v.len() <= 32)),
            api: cx.new(|cx| InputState::new(window, cx).masked(true)),
            import: cx.new(|cx| InputState::new(window, cx).masked(true)),
            callback: cx.new(|cx| InputState::new(window, cx).masked(true)),
            clear_callback: false,
            cancelling: false,
            login_feedback: None,
            quota_target: None,
            quota_error: None,
            quota_history: HashMap::new(),
            revision: 0,
            quota_pending: false,
            focus: cx.focus_handle().tab_stop(false),
            entries: std::array::from_fn(|_| cx.focus_handle()),
        }
    }
    pub fn clear(&mut self, window: &mut Window, cx: &mut Context<PlatformsView>) {
        // Drop the old inputs, including their selection and undo buffers.
        self.name = cx.new(|cx| InputState::new(window, cx).validate(|v, _| v.len() <= 32));
        self.api = cx.new(|cx| InputState::new(window, cx).masked(true));
        self.import = cx.new(|cx| InputState::new(window, cx).masked(true));
        self.callback = cx.new(|cx| InputState::new(window, cx).masked(true));
        self.clear_callback = false;
    }
}
#[inline(never)]
fn status(id: impl Into<SharedString>, label: impl Into<SharedString>) -> AnyElement {
    let label = label.into();
    div()
        .id(id.into())
        .test_support()
        .role(Role::Status)
        .aria_label(label.clone())
        .child(label)
        .into_any_element()
}
#[inline(never)]
fn field(
    id: &'static str,
    label: &'static str,
    input: &Entity<InputState>,
    disabled: bool,
) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .gap(px(3.))
        .child(label)
        .child(
            Input::new(input)
                .id(id)
                .accessibility_id(id)
                .aria_label(label)
                .readonly(disabled),
        )
        .into_any_element()
}
#[inline(never)]
fn workspace_panel(
    label: &'static str,
    focus: &FocusHandle,
    cx: &App,
) -> impl ParentElement + IntoElement + use<> {
    div()
        .id("platform-tools-workspace")
        .test_support()
        .role(Role::Region)
        .aria_label(label)
        .track_focus(focus)
        .tab_stop(false)
        .flex()
        .flex_col()
        .w_full()
        .max_w(px(680.))
        .gap(px(8.))
        .p(px(12.))
        .rounded(px(12.))
        .border_1()
        .border_color(cx.theme().border)
        .child(badge::section_heading(label, None, cx))
}
impl PlatformsView {
    pub(super) fn workspace_dirty(&self, cx: &App) -> bool {
        match self.tools.active {
            Some(Workspace::Login) => [&self.tools.name, &self.tools.callback]
                .iter()
                .any(|i| !i.read(cx).value().is_empty()),
            Some(Workspace::Add) => [&self.tools.name, &self.tools.api, &self.tools.import]
                .iter()
                .any(|i| !i.read(cx).value().is_empty()),
            _ => false,
        }
    }
    pub(super) fn change_workspace(
        &mut self,
        next: Option<Workspace>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.decision.is_some()
            || self.account_form.is_some()
            || (self.busy && !self.tools.quota_pending)
            || self.diagnostic_navigation_blocked()
        {
            return;
        }
        if self.workspace_dirty(cx) {
            self.decide(Decision::DiscardCodexTools(next), window, cx);
        } else {
            self.show_workspace(next, window, cx);
        }
    }
    pub(super) fn show_workspace(
        &mut self,
        next: Option<Workspace>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let previous = self.tools.active;
        if matches!(previous, Some(Workspace::Diagnostics | Workspace::Login)) {
            self.clear_workspace_diagnostic();
        }
        self.tools.revision += 1;
        if self.tools.quota_pending {
            self.tools.quota_target = None;
        }
        self.tools.clear(window, cx);
        self.tools.active = next;
        self.message = None;
        if next == Some(Workspace::Login) {
            self.restore_login_status(cx);
        }
        if next.is_some() {
            window.focus(&self.tools.focus, cx);
        } else if let Some(previous) = previous {
            window.focus(&self.tools.entries[previous as usize], cx);
        }
        cx.notify();
    }
    fn restore_login_status(&mut self, cx: &mut Context<Self>) {
        self.oauth_status = self.services.platform_oauth_status();
        self.tools.cancelling &= self.services.platform_oauth_active();
        if self.oauth_task.is_some()
            || self.oauth_watch.is_some()
            || !self.services.platform_oauth_active()
        {
            return;
        }
        // Leaving the page requests cancellation. Reopening may precede owner cleanup.
        let generation = self.oauth_generation;
        self.oauth_watch = Some(cx.spawn(async move |weak, cx| {
            loop {
                cx.background_executor()
                    .timer(std::time::Duration::from_millis(60))
                    .await;
                let again = weak
                    .update(cx, |v, cx| {
                        if v.oauth_generation != generation {
                            return false;
                        }
                        v.oauth_status = v.services.platform_oauth_status();
                        let active = v.services.platform_oauth_active();
                        if !active {
                            v.tools.cancelling = false;
                            v.oauth_watch = None;
                        }
                        cx.notify();
                        active
                    })
                    .unwrap_or(false);
                if !again {
                    break;
                }
            }
        }));
    }
    #[inline(never)]
    pub(super) fn render_workspaces(
        &mut self,
        mut root: Stateful<Div>,
        window: &mut Window,
        cx: &mut Context<Self>,
        blocked: bool,
        oauth_active: bool,
    ) -> Stateful<Div> {
        root.extend([self.render_tool_entries(oauth_active, cx)]);
        let Some(task) = self.tools.active else {
            if self.diagnostic_blocked() {
                root = root.child(self.render_codex_diagnostics(window, cx));
            }
            return root;
        };
        let (_, label) = task.entry(cx);
        let mut panel = workspace_panel(label, &self.tools.focus, cx);
        let content = match task {
            Workspace::Login => self.render_login(blocked, oauth_active, cx),
            Workspace::Add => self.render_add(blocked, cx),
            Workspace::Quota => self.render_manual_quota(blocked, cx),
            Workspace::Diagnostics => self.render_codex_diagnostics(window, cx),
        };
        panel.extend([content, self.render_back(cx)]);
        root.extend([panel.into_any_element()]);
        root
    }

    #[inline(never)]
    fn render_back(&self, cx: &mut Context<Self>) -> AnyElement {
        semantics::button(
            Button::new("platform-tools-back")
                .label(text(cx, "Back to accounts", "返回账户"))
                .ghost()
                .on_click(cx.listener(|v, _, w, cx| v.change_workspace(None, w, cx))),
            (self.busy && !self.tools.quota_pending)
                || self.decision.is_some()
                || self.diagnostic_navigation_blocked(),
            false,
            cx,
        )
        .into_any_element()
    }
    #[inline(never)]
    fn render_tool_entries(&self, oauth_active: bool, cx: &mut Context<Self>) -> AnyElement {
        let remote = matches!(
            self.services.environment(),
            ccr_app::runtime::EnvironmentId::Ssh(_)
        );
        let mut actions = div()
            .id("platform-tools")
            .test_support()
            .role(Role::Group)
            .aria_label(text(cx, "Codex account tools", "Codex 账户工具"))
            .flex()
            .flex_wrap()
            .gap(px(6.));
        for task in [
            Workspace::Login,
            Workspace::Add,
            Workspace::Quota,
            Workspace::Diagnostics,
        ] {
            if remote && task != Workspace::Add {
                continue;
            }
            let (id, label) = task.entry(cx);
            let selected = self.tools.active == Some(task);
            let label = if task == Workspace::Login && oauth_active {
                format!("{label} · {}", text(cx, "in progress", "进行中"))
            } else {
                label.to_owned()
            };
            let disabled = self.account_form.is_some()
                || self.decision.is_some()
                || (self.busy && !self.tools.quota_pending)
                || self.diagnostic_navigation_blocked();
            let button = gpui_kit::base::Button::new(id)
                .track_focus(&self.tools.entries[task as usize])
                .tab_stop(!disabled)
                .disabled(disabled)
                .accessibility_label(label.clone())
                .aria_expanded(selected)
                .a11y_synthetic_children(move |tree| {
                    if disabled {
                        tree.parent_node().set_disabled();
                    }
                })
                .selected(selected)
                .px(px(10.))
                .py(px(5.))
                .rounded(px(theme::PILL_RADIUS))
                .border_1()
                .border_color(cx.theme().border)
                .focus_visible(|s| s.border_color(cx.theme().ring))
                .bg(if selected {
                    cx.theme().secondary
                } else {
                    cx.theme().background
                })
                .styles(|s| s.disabled(|s| s.text_color(cx.theme().muted_foreground)))
                .child(label)
                .on_click(cx.listener(move |v, _, w, cx| {
                    v.change_workspace((!selected).then_some(task), w, cx)
                }));
            actions.extend([button.into_any_element()]);
        }
        actions.into_any_element()
    }
    #[inline(never)]
    fn render_add(&self, blocked: bool, cx: &mut Context<Self>) -> AnyElement {
        let target = if matches!(
            self.services.environment(),
            ccr_app::runtime::EnvironmentId::Ssh(_)
        ) {
            text(
                cx,
                "Saved accounts on the selected SSH host",
                "所选 SSH 主机的已保存账户",
            )
        } else {
            text(cx, "Local saved accounts", "本机已保存账户")
        };
        let mut panel = div()
            .flex()
            .flex_col()
            .gap(px(6.))
            .child(status("platform-add-target", target))
            .child(text(
                cx,
                "Create a saved account. Import accepts auth.json or an account export.",
                "创建已保存账户；可导入 auth.json 或账户导出文件的内容。",
            ))
            .child(field(
                "platform-tools-name",
                text(cx, "New account name", "新建账户名"),
                &self.tools.name,
                blocked,
            ));
        for (id, label, input, button, action, decision) in [
            (
                "platform-tools-api",
                text(
                    cx,
                    "API account JSON (api_key, optional base_url)",
                    "API 账户 JSON（api_key，可选 base_url）",
                ),
                &self.tools.api,
                "platform-account-api-key",
                text(cx, "Add API account", "添加 API 账户"),
                Decision::AddApiKey,
            ),
            (
                "platform-tools-import",
                text(cx, "Account import JSON", "要导入的账户 JSON"),
                &self.tools.import,
                "platform-account-import",
                text(cx, "Import accounts", "导入账户"),
                Decision::ImportAccounts,
            ),
        ] {
            panel = panel
                .child(field(id, label, input, blocked))
                .child(semantics::button(
                    Button::new(button)
                        .label(action)
                        .on_click(cx.listener(move |v, _, w, cx| v.decide(decision, w, cx))),
                    blocked,
                    false,
                    cx,
                ));
        }
        panel.into_any_element()
    }
    #[inline(never)]
    fn render_login(&self, blocked: bool, active: bool, cx: &mut Context<Self>) -> AnyElement {
        let phase = self.oauth_status.as_ref().map(|s| s.phase);
        let busy = self.oauth_control_phase.is_some() || self.tools.cancelling;
        let mut panel = div().flex().flex_col().gap(px(8.));
        if self.services.is_fixture() {
            panel = panel.child(text(
                cx,
                "Demo mode disables browser authorization and token exchange.",
                "演示模式已禁用浏览器授权和令牌交换。",
            ));
        }
        let stage = if self.tools.cancelling {
            text(
                cx,
                "Cancelling · waiting for cleanup",
                "取消中 · 等待清理完成",
            )
        } else if self.oauth_control_phase == Some(NativePhase::Callback) {
            text(cx, "Submitting the account", "正在提交账户")
        } else {
            match phase {
                Some(NativePhase::Waiting) => {
                    text(cx, "Waiting for the browser callback", "等待浏览器回调")
                }
                Some(NativePhase::Callback) => text(
                    cx,
                    "Callback received · ready for account commit",
                    "已收到回调 · 等待提交账户",
                ),
                Some(NativePhase::Completing) => text(cx, "Committing account", "正在提交账户"),
                Some(NativePhase::Succeeded) => text(cx, "Account committed", "账户已提交"),
                Some(NativePhase::Cancelled) => text(cx, "Login cancelled", "登录已取消"),
                Some(NativePhase::TimedOut) => text(cx, "Login timed out", "登录已超时"),
                Some(NativePhase::Failed) => text(cx, "Login failed", "登录失败"),
                Some(NativePhase::CleanupFailed) => {
                    text(cx, "Listener cleanup failed", "监听器清理失败")
                }
                None if active => text(cx, "Starting login", "正在启动登录"),
                None => text(
                    cx,
                    "Start OAuth to sign in to a local Codex account.",
                    "开始 OAuth 以登录本机 Codex 账户。",
                ),
            }
        };
        panel = panel.child(status("platform-oauth-stage", stage));
        if !active && phase != Some(NativePhase::CleanupFailed) {
            panel = panel.child(semantics::button(
                Button::new("platform-oauth-start")
                    .label(text(cx, "Start OAuth", "开始 OAuth"))
                    .on_click(cx.listener(|v, _, _, cx| v.preflight_oauth_start(cx))),
                blocked || busy || !self.services.manual_provider_actions_available(),
                false,
                cx,
            ));
        }
        if active && phase == Some(NativePhase::Waiting) && !self.tools.cancelling {
            if self.oauth_status.as_ref().is_some_and(|s| !s.fixture) && !self.services.is_fixture()
            {
                panel = panel.child(semantics::button(
                    Button::new("platform-oauth-browser")
                        .ghost()
                        .label(text(
                            cx,
                            "Open authorization in browser",
                            "在浏览器打开授权页",
                        ))
                        .on_click(cx.listener(|v, _, _, cx| v.open_oauth_browser(cx))),
                    busy,
                    false,
                    cx,
                ));
            }
            panel = panel
                .child(field(
                    "platform-oauth-url",
                    text(cx, "Manual callback URL", "手动回调地址"),
                    &self.tools.callback,
                    busy,
                ))
                .child(semantics::button(
                    Button::new("platform-oauth-callback")
                        .label(text(cx, "Submit callback", "提交回调"))
                        .on_click(cx.listener(|v, _, _, cx| v.control_oauth(false, cx))),
                    busy,
                    false,
                    cx,
                ));
        }
        if active && phase == Some(NativePhase::Callback) && !self.tools.cancelling {
            panel = panel
                .child(field(
                    "platform-oauth-account",
                    text(cx, "Optional login account name", "可选的登录账户名"),
                    &self.tools.name,
                    busy,
                ))
                .child(semantics::button(
                    Button::new("platform-oauth-complete")
                        .label(text(cx, "Complete account commit", "完成账户提交"))
                        .on_click(cx.listener(|v, _, _, cx| v.control_oauth(true, cx))),
                    busy,
                    false,
                    cx,
                ));
        }
        if active
            && matches!(
                phase,
                None | Some(NativePhase::Waiting | NativePhase::Callback)
            )
        {
            panel = panel.child(semantics::button(
                Button::new("platform-oauth-cancel")
                    .label(text(cx, "Cancel OAuth", "取消 OAuth"))
                    .ghost()
                    .on_click(cx.listener(|v, _, _, cx| {
                        if !v.tools.cancelling {
                            v.tools.cancelling = true;
                            v.services.platform_oauth_cancel();
                            cx.notify();
                        }
                    })),
                busy,
                self.tools.cancelling,
                cx,
            ));
        } else if phase == Some(NativePhase::CleanupFailed) {
            panel = panel.child(semantics::button(
                Button::new("platform-oauth-retry-cleanup")
                    .label(text(cx, "Retry cleanup", "重试清理"))
                    .on_click(cx.listener(|v, _, _, cx| v.control_oauth(false, cx))),
                busy,
                false,
                cx,
            ));
        }
        if let Some(feedback) = &self.tools.login_feedback {
            panel = panel.child(status("platform-login-feedback", feedback.clone()));
        }
        panel.into_any_element()
    }
    #[inline(never)]
    fn render_manual_quota(&self, blocked: bool, cx: &mut Context<Self>) -> AnyElement {
        let mut panel = div().flex().flex_col().gap(px(8.)).child(text(
            cx,
            "Each query contacts the provider and may update credentials and quota history.",
            "手动查询会连接供应商，并可能更新凭据和配额历史。",
        ));
        if self.services.is_fixture() {
            panel = panel.child(text(
                cx,
                "Demo mode disables provider requests.",
                "演示模式已禁用供应商请求。",
            ));
        }
        let selected = self.selected_account();
        let mut actions = div().flex().flex_wrap().gap(px(6.));
        for (id, label, target) in [
            (
                "platform-quota-current",
                text(cx, "Current runtime", "当前运行").to_string(),
                Some(QuotaTarget::Runtime),
            ),
            (
                "platform-quota-selected",
                selected
                    .as_ref()
                    .map(|n| format!("{} · {n}", text(cx, "Selected account", "所选账户")))
                    .unwrap_or_else(|| {
                        text(cx, "Selected account · none", "所选账户 · 未选择").into()
                    }),
                selected.map(QuotaTarget::Saved),
            ),
            (
                "platform-quota-all",
                text(cx, "All saved accounts", "所有已保存账户").into(),
                Some(QuotaTarget::AllSaved),
            ),
        ] {
            let disabled =
                blocked || !self.services.manual_provider_actions_available() || target.is_none();
            actions = actions.child(semantics::button(
                Button::new(id)
                    .label(label)
                    .ghost()
                    .on_click(cx.listener(move |v, _, _, cx| {
                        let (Some(session), Some(target)) = (v.session.clone(), target.clone())
                        else {
                            return;
                        };
                        if v.busy || v.tools.active != Some(Workspace::Quota) {
                            return;
                        }
                        v.tools.quota_target = Some(target.clone());
                        v.tools.quota_error = None;
                        v.quotas.clear();
                        v.request(Command::Quota { session, target }, cx);
                    })),
                disabled,
                false,
                cx,
            ));
        }
        panel = panel.child(actions);
        let Some(target) = &self.tools.quota_target else {
            return panel
                .child(status(
                    "platform-quota-state",
                    text(cx, "No quota queried", "尚未查询配额"),
                ))
                .into_any_element();
        };
        let label = match target {
            QuotaTarget::Runtime => text(cx, "Current runtime", "当前运行"),
            QuotaTarget::Saved(name) => name.as_str(),
            QuotaTarget::AllSaved => text(cx, "All saved accounts", "所有已保存账户"),
        };
        let failures = self.quotas.iter().filter(|r| r.failed).count();
        let state = if self.tools.quota_pending {
            text(cx, "Querying", "正在查询")
        } else if self.tools.quota_error.is_some()
            || (failures > 0 && failures == self.quotas.len())
        {
            text(cx, "Query failed", "查询失败")
        } else if failures > 0 {
            text(cx, "Partial results", "部分查询失败")
        } else if self.quotas.is_empty() {
            text(cx, "No accounts returned", "未返回账户")
        } else {
            text(cx, "Query complete", "查询完成")
        };
        panel = panel.child(status("platform-quota-state", format!("{label} · {state}")));
        if let Some(e) = &self.tools.quota_error {
            panel = panel.child(status("platform-quota-error", e.clone()));
        }
        if self.tools.quota_error.is_some() {
            let mut previous = self
                .tools
                .quota_history
                .iter()
                .filter(|(scope, _)| {
                    *scope == target
                        || matches!(
                            (target, scope),
                            (QuotaTarget::AllSaved, QuotaTarget::Saved(_))
                        )
                })
                .map(|(_, row)| row)
                .collect::<Vec<_>>();
            previous.sort_by(|a, b| a.account.cmp(&b.account));
            for (index, row) in previous.into_iter().enumerate() {
                panel = panel.child(status(
                    format!("platform-quota-history-{index}"),
                    quota_facts(row, true, cx),
                ));
            }
        }
        for (index, row) in self.quotas.iter().enumerate() {
            panel = panel.child(status(
                format!("platform-quota-result-{index}"),
                quota_facts(row, false, cx),
            ));
            let scope = if *target == QuotaTarget::Runtime {
                QuotaTarget::Runtime
            } else {
                QuotaTarget::Saved(row.account.clone())
            };
            if row.failed
                && let Some(history) = self.tools.quota_history.get(&scope)
            {
                panel = panel.child(status(
                    format!("platform-quota-history-{index}"),
                    quota_facts(history, true, cx),
                ));
            }
        }
        panel.into_any_element()
    }
}
fn quota_facts(row: &QuotaRow, history: bool, cx: &App) -> String {
    let value = |p: Option<i32>| {
        p.map(|n| format!("{n}%"))
            .unwrap_or_else(|| text(cx, "unavailable", "不可用").into())
    };
    let reset = |t: Option<i64>| {
        t.and_then(|n| chrono::DateTime::from_timestamp(n, 0))
            .map(|d| d.to_rfc3339())
            .unwrap_or_else(|| text(cx, "unavailable", "不可用").into())
    };
    let source = if history {
        text(cx, "Last successful query", "上次成功查询")
    } else if row.failed {
        text(cx, "Failed attempt", "查询失败")
    } else {
        text(cx, "Fetched", "获取时间")
    };
    let warning = if row.history_warning {
        text(cx, " · History write failed", " · 历史写入失败")
    } else {
        ""
    };
    format!(
        "{} · {source} {}\n{} {} · {} {}\n{} {} · {} {}{warning}",
        row.account,
        row.fetched_at,
        text(cx, "Primary remaining", "主窗口剩余"),
        value(row.hourly),
        text(cx, "Reset", "重置"),
        reset(row.hourly_reset),
        text(cx, "Weekly remaining", "周窗口剩余"),
        value(row.weekly),
        text(cx, "Reset", "重置"),
        reset(row.weekly_reset)
    )
}
