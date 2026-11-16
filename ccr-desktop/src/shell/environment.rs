//! The shell owns the sole current environment and explicit connection lifecycle.
use super::*;
use crate::runtime::environments::{SshAction, SshResult};
use ccr_app::environments::{
    HostKeyStatus,
    ssh::{Host, Probe},
};
use ccr_app::runtime::{Completion, EnvironmentId, RequestKey, TaskEvent};
use gpui_kit::component::FocusTrapElement;
#[cfg(test)]
mod tests;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Unreachable,
    Trust,
    Credentials,
}
impl ConnectionState {
    pub(super) fn label(self, cx: &gpui_kit::App) -> &'static str {
        match self {
            Self::Disconnected => text(cx, "Not connected", "未连接"),
            Self::Connecting => text(cx, "Connecting", "连接中"),
            Self::Connected => text(cx, "Connected", "已连接"),
            Self::Unreachable => text(cx, "Unreachable", "不可达"),
            Self::Trust => text(cx, "Host key approval required", "待信任主机密钥"),
            Self::Credentials => text(cx, "Credentials missing", "凭据缺失"),
        }
    }
}
pub(super) struct ActiveEnvironment {
    pub id: EnvironmentId,
    pub state: ConnectionState,
    pub services: Services,
    pub menu: bool,
    pub menu_press: bool,
    pub trigger_focus: FocusHandle,
    menu_focus: FocusHandle,
    menu_enter: bool,
    menu_generation: u64,
    pub pending: Option<EnvironmentId>,
    pub pending_request: Option<EnvironmentId>,
    pub probe: Option<Probe>,
    pub message: Option<&'static str>,
    generation: u64,
    task: Option<gpui_kit::Task<()>>,
    watch: Option<gpui_kit::Task<()>>,
    pub switching: bool,
    window: gpui_kit::AnyWindowHandle,
    started: bool,
    enter: bool,
    return_focus: Option<FocusHandle>,
}
impl ActiveEnvironment {
    pub fn new(services: Services, window: &Window, cx: &mut Context<Shell>) -> Self {
        Self {
            id: services.environment(),
            state: ConnectionState::Connected,
            services,
            menu: false,
            menu_press: false,
            trigger_focus: cx.focus_handle(),
            menu_focus: cx.focus_handle().tab_stop(false),
            menu_enter: false,
            menu_generation: 0,
            pending: None,
            pending_request: None,
            probe: None,
            message: None,
            generation: 0,
            task: None,
            watch: None,
            switching: false,
            window: window.window_handle(),
            started: false,
            enter: false,
            return_focus: None,
        }
    }
    pub fn is_ssh(&self) -> bool {
        matches!(self.id, EnvironmentId::Ssh(_))
    }
}
impl Shell {
    pub(super) fn close_environment_menu(
        &mut self,
        restore: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.active_environment.menu {
            return;
        }
        self.active_environment.menu = false;
        self.active_environment.menu_enter = false;
        self.active_environment.menu_generation += 1;
        if restore {
            window.focus(&self.active_environment.trigger_focus, cx);
        }
        cx.notify();
    }
    pub(super) fn toggle_environment_menu(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.preferences_loaded || self.active_environment.switching {
            return;
        }
        let was_open = if window.last_input_was_keyboard() {
            self.active_environment.menu
        } else {
            self.active_environment.menu_press
        };
        if was_open {
            self.close_environment_menu(true, window, cx);
        } else {
            self.active_environment.menu = true;
            self.active_environment.menu_enter = true;
            self.active_environment.menu_generation += 1;
            self.remote.update(cx, |view, cx| view.refresh(cx));
            cx.notify();
        }
    }
    pub(super) fn dismiss_environment(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.active_environment.menu {
            self.close_environment_menu(true, window, cx);
            return;
        }
        self.active_environment.pending = None;
        if self.active_environment.probe.take().is_some() {
            self.active_environment.state = ConnectionState::Disconnected;
        }
        window.focus(
            self.active_environment
                .return_focus
                .as_ref()
                .unwrap_or(&self.status_focus),
            cx,
        );
        cx.notify();
    }
    fn environment_has_drafts(&self, cx: &gpui_kit::App) -> bool {
        self.configurations.read(cx).has_dirty_draft(cx)
            || self.platforms.read(cx).has_environment_draft(cx)
            || self.resources.read(cx).has_environment_draft(cx)
            || self.converter.read(cx).has_environment_draft(cx)
            || self.mcp.read(cx).has_environment_draft()
            || self.sync.read(cx).has_environment_draft()
            || self.workbench.read(cx).has_environment_draft()
            || self.insights.read(cx).has_environment_draft(cx)
            || self.install.read(cx).has_environment_draft()
            || self.jobs.read(cx).has_environment_draft(cx)
            || self.remote.read(cx).has_environment_draft(cx)
    }
    pub(super) fn environment_recovery(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut actions = div().flex().flex_wrap().gap(px(8.));
        if let Some(error) = self.active_environment.message {
            let message = ccr_app::environments::ssh::authentication_message(error, crate::locales::is_chinese(cx))
                .unwrap_or_else(|| if error == "ssh_host_key_mismatch" {
                    text(cx,"Host key changed. Connection refused; the saved fingerprint was not replaced.","主机密钥已变更，已拒绝连接，未替换已保存指纹。")
                } else { text(cx,"SSH is unavailable. Check the host and retry; the current environment is retained.","SSH 当前不可用，请检查主机后重试。当前执行环境仍被保留。") });
            actions = actions.child(message);
        }
        if self.active_environment.is_ssh() {
            actions = actions.child(semantics::button(
                Button::new("environment-retry")
                    .label(
                        if self.active_environment.state == ConnectionState::Connected {
                            text(cx, "Reconnect", "重新连接")
                        } else {
                            text(cx, "Retry connection", "重试连接")
                        },
                    )
                    .ghost()
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.request_environment_switch(
                            this.active_environment.id.clone(),
                            true,
                            window,
                            cx,
                        )
                    })),
                self.active_environment.state == ConnectionState::Connecting,
                false,
                cx,
            ));
        }
        if self.active_environment.state == ConnectionState::Credentials {
            actions = actions.child(semantics::button(
                Button::new("environment-repair-credentials")
                    .label(text(cx, "Edit host credentials", "编辑主机凭据"))
                    .ghost()
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.open_environment_settings(window, cx);
                        if let EnvironmentId::Ssh(id) = this.active_environment.id.clone() {
                            this.remote
                                .update(cx, |view, cx| view.edit_saved_host(&id, window, cx));
                        }
                    })),
                false,
                false,
                cx,
            ));
        }
        actions
            .child(semantics::button(
                Button::new("environment-return-local")
                    .label(text(cx, "Return to Local", "切回本地"))
                    .ghost()
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.request_environment(EnvironmentId::Local("local".into()), window, cx)
                    })),
                self.active_environment.switching,
                false,
                cx,
            ))
            .into_any_element()
    }
    pub(super) fn independent_destination(&self) -> bool {
        matches!(
            self.destination,
            Destination::Settings | Destination::Checkin
        ) || (self.destination == Destination::McpManager && self.lens == Lens::Converter)
            || (self.destination == Destination::Commands
                && matches!(self.lens, Lens::Favorites | Lens::Recent))
    }
    pub(super) fn remote_configuration_destination(&self) -> bool {
        (matches!(
            self.destination,
            Destination::Dashboard | Destination::AgentSessions | Destination::Commands
        ) && self.lens == Lens::Primary)
            || (self.destination == Destination::Usage && self.lens == Lens::Install)
            || (self.platform().is_some() && matches!(self.lens, Lens::Primary | Lens::Sessions))
            || (self.destination == Destination::McpManager && self.lens == Lens::Primary)
            || matches!(
                (self.platform(), self.lens),
                (
                    Some(
                        Platform::Claude | Platform::Codex | Platform::Gemini | Platform::OpenCode
                    ),
                    Lens::Files | Lens::Extensions
                ) | (
                    Some(Platform::Claude | Platform::Codex | Platform::Gemini),
                    Lens::Mcp
                ) | (Some(_), Lens::Accounts)
                    | (
                        Some(Platform::Claude | Platform::Codex | Platform::Grok),
                        Lens::Profiles | Lens::Auth,
                    )
                    | (Some(Platform::Codex | Platform::OpenCode), Lens::Providers)
                    | (Some(Platform::Codex), Lens::Models)
            )
    }
    pub(super) fn environment_name(&self, cx: &gpui_kit::App) -> String {
        if !self.preferences_loaded {
            return text(cx, "Loading execution environment…", "正在加载执行环境…").into();
        }
        match &self.active_environment.id {
            EnvironmentId::Ssh(id) => self
                .remote
                .read(cx)
                .hosts()
                .iter()
                .find(|host| &host.id == id)
                .map(|host| host.name.clone())
                .unwrap_or_else(|| {
                    text(cx, "Saved SSH host unavailable", "保存的 SSH 主机不可用").into()
                }),
            _ => text(cx, "Local", "本地").into(),
        }
    }
    pub(super) fn environment_connection_label(&self, cx: &gpui_kit::App) -> &'static str {
        if !self.preferences_loaded {
            return "";
        }
        if !self.active_environment.is_ssh()
            && self.active_environment.state == ConnectionState::Connected
        {
            text(cx, "Ready", "就绪")
        } else {
            self.active_environment.state.label(cx)
        }
    }
    pub(super) fn environment_status(&self, cx: &gpui_kit::App) -> String {
        let mut label = self.environment_name(cx);
        if self.preferences_loaded {
            label.push_str(&format!(" · {}", self.environment_connection_label(cx)));
            if self.fixture {
                label.push_str(if self.slow_fixture {
                    text(
                        cx,
                        " · Demo data · Controlled slow read",
                        " · 演示数据 · 受控慢读取",
                    )
                } else {
                    text(cx, " · Demo data", " · 演示数据")
                });
            }
        }
        label
    }
    pub(super) fn restore_environment(
        &mut self,
        id: EnvironmentId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.active_environment.started {
            return;
        }
        self.active_environment.started = true;
        self.navigation_generation += 1;
        self.park_environment(window, cx);
        self.active_environment.id = id.clone();
        self.active_environment.services.select_environment(id);
        if let EnvironmentId::Ssh(id) = &self.active_environment.id {
            self.start_environment_action(SshAction::Connect(id.clone()), cx);
        } else {
            self.activate(window, cx);
        }
    }
    fn watch_environment_connection(&mut self, cx: &mut Context<Self>) {
        let EnvironmentId::Ssh(id) = self.active_environment.id.clone() else {
            return;
        };
        let generation = self.active_environment.generation;
        self.active_environment.watch = Some(cx.spawn(async move |weak, cx| {
            loop {
                cx.background_executor()
                    .timer(std::time::Duration::from_secs(1))
                    .await;
                let keep = weak
                    .update(cx, |this, cx| {
                        if this.active_environment.generation != generation
                            || this.active_environment.id != EnvironmentId::Ssh(id.clone())
                            || this.active_environment.state != ConnectionState::Connected
                        {
                            return false;
                        }
                        if !this.active_environment.services.ssh_connected(&id) {
                            this.active_environment.state = ConnectionState::Unreachable;
                            this.active_environment.message = Some("ssh_not_connected");
                            cx.notify();
                            return false;
                        }
                        true
                    })
                    .unwrap_or(false);
                if !keep {
                    break;
                }
            }
        }));
    }
    pub(super) fn request_environment(
        &mut self,
        id: EnvironmentId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.request_environment_switch(id, false, window, cx);
    }
    fn request_environment_switch(
        &mut self,
        id: EnvironmentId,
        reconnect: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.preferences_loaded || self.active_environment.switching {
            return;
        }
        self.active_environment.return_focus = if self.active_environment.menu {
            Some(self.active_environment.trigger_focus.clone())
        } else {
            window.focused(cx)
        };
        self.close_environment_menu(true, window, cx);
        let already_ready = id == self.active_environment.id
            && self.active_environment.state == ConnectionState::Connected
            && match &id {
                EnvironmentId::Local(_) => true,
                EnvironmentId::Ssh(host) => self.active_environment.services.ssh_connected(host),
                EnvironmentId::Wsl(_) => false,
            };
        if !reconnect && already_ready {
            cx.notify();
            return;
        }
        if self.environment_has_drafts(cx)
            || self
                .active_environment
                .services
                .executor
                .environment_has_jobs(&self.active_environment.id)
            || self.remote.read(cx).is_busy()
        {
            self.active_environment.pending = Some(id);
            self.active_environment.enter = true;
            cx.notify();
        } else {
            self.switch_environment(id, window, cx);
        }
    }
    fn switch_environment(
        &mut self,
        id: EnvironmentId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.navigation_generation += 1;
        self.navigation_decision = None;
        self.active_environment.watch = None;
        let previous = self.active_environment.id.clone();
        let remote_previous = self.remote.read(cx).operation_environment();
        self.park_environment(window, cx);
        self.remote.update(cx, |view, cx| {
            view.discard_for_environment_switch(window, cx)
        });
        self.jobs
            .update(cx, |view, cx| view.clear_environment(window, cx));
        self.insights
            .update(cx, |view, cx| view.clear_environment(window, cx));
        self.active_environment.pending = None;
        self.active_environment.probe = None;
        self.active_environment.message = None;
        self.active_environment.generation += 1;
        self.active_environment.state = ConnectionState::Connecting;
        self.active_environment.switching = true;
        let generation = self.active_environment.generation;
        let services = self.active_environment.services.clone();
        let key = RequestKey {
            environment: EnvironmentId::Local("environment-switch-control".into()),
            platform: Platform::Claude,
            generation,
            request_id: generation,
        };
        let remote_drain = (remote_previous != previous)
            .then(|| services.executor.begin_environment_drain(remote_previous));
        let drain = services.executor.begin_environment_drain(previous);
        let handle = window.window_handle();
        self.active_environment.task = Some(cx.spawn(async move |weak, cx| {
            if drain.recv().await.is_err() {
                return;
            }
            if let Some(remote_drain) = remote_drain
                && remote_drain.recv().await.is_err()
            {
                return;
            }
            let admitted = services.ssh_action(key, SshAction::Disconnect);
            let Ok(mut subscription) = admitted else {
                let _ = weak.update(cx, |this, cx| {
                    if this.active_environment.generation == generation {
                        this.active_environment.switching = false;
                        this.active_environment.state = ConnectionState::Unreachable;
                        this.active_environment.message = Some("ssh_busy");
                        cx.notify();
                    }
                });
                return;
            };
            while let Some(event) = subscription.next().await {
                if let TaskEvent::Terminal(result) = event {
                    let _ = cx.update_window(handle, |_, window, cx| {
                        let _ = weak.update(cx, |this, cx| {
                            if this.active_environment.generation != generation {
                                return;
                            }
                            this.active_environment.switching = false;
                            if !matches!(
                                result.as_ref(),
                                Completion::Success(SshResult::Disconnected)
                            ) {
                                this.active_environment.state = ConnectionState::Unreachable;
                                this.active_environment.message = Some("ssh_cleanup_failed");
                            } else {
                                this.active_environment.id = id.clone();
                                this.active_environment
                                    .services
                                    .select_environment(id.clone());
                                this.persist_preferences(this.preference_snapshot(cx), window, cx);
                                if let EnvironmentId::Ssh(host) = &id {
                                    this.start_environment_action(
                                        SshAction::Connect(host.clone()),
                                        cx,
                                    );
                                } else {
                                    this.active_environment.state = ConnectionState::Connected;
                                    this.activate(window, cx);
                                }
                            }
                            cx.notify();
                        });
                    });
                    break;
                }
            }
        }));
        cx.notify();
    }
    pub(super) fn start_environment_action(&mut self, action: SshAction, cx: &mut Context<Self>) {
        self.active_environment.watch = None;
        self.active_environment.generation += 1;
        let generation = self.active_environment.generation;
        let handle = self.active_environment.window;
        self.active_environment.state = ConnectionState::Connecting;
        self.active_environment.message = None;
        self.active_environment.probe = None;
        let key = RequestKey {
            environment: self.active_environment.id.clone(),
            platform: Platform::Claude,
            generation,
            request_id: generation,
        };
        match self.active_environment.services.ssh_action(key, action) {
            Err(_) => {
                self.active_environment.state = ConnectionState::Unreachable;
                self.active_environment.message = Some("ssh_busy");
            }
            Ok(mut subscription) => {
                self.active_environment.task = Some(cx.spawn(async move |weak, cx| {
                    while let Some(event) = subscription.next().await {
                        if let TaskEvent::Terminal(result) = event {
                            let _ = cx.update_window(handle, |_, window, cx| {
                                let _ = weak.update(cx, |this, cx| {
                                    if this.active_environment.generation != generation {
                                        return;
                                    }
                                    match result.as_ref() {
                                        Completion::Success(SshResult::Connected) => {
                                            this.active_environment.state =
                                                ConnectionState::Connected;
                                            this.watch_environment_connection(cx);
                                            this.remote
                                                .update(cx, |view, cx| view.clear_cli_on_leave(cx));
                                            this.activate(window, cx);
                                        }
                                        Completion::Success(SshResult::Confirmed) => {
                                            if let EnvironmentId::Ssh(id) =
                                                &this.active_environment.id
                                            {
                                                this.start_environment_action(
                                                    SshAction::Connect(id.clone()),
                                                    cx,
                                                );
                                            }
                                        }
                                        Completion::Success(SshResult::Probe(probe)) => {
                                            if probe.status == HostKeyStatus::Mismatch {
                                                this.active_environment.state =
                                                    ConnectionState::Unreachable;
                                                this.active_environment.message =
                                                    Some("ssh_host_key_mismatch");
                                            } else {
                                                this.active_environment.state =
                                                    ConnectionState::Trust;
                                                this.active_environment.probe = Some(probe.clone());
                                                this.active_environment.enter = true;
                                            }
                                        }
                                        Completion::Failed("ssh_host_key_untrusted") => {
                                            if let EnvironmentId::Ssh(id) =
                                                &this.active_environment.id
                                            {
                                                this.start_environment_action(
                                                    SshAction::Probe(id.clone()),
                                                    cx,
                                                );
                                            }
                                        }
                                        Completion::Failed(error)
                                        | Completion::CleanupFailed(error) => {
                                            this.active_environment.state =
                                                if *error == "ssh_credentials_missing" {
                                                    ConnectionState::Credentials
                                                } else {
                                                    ConnectionState::Unreachable
                                                };
                                            this.active_environment.message = Some(error);
                                        }
                                        _ => {
                                            this.active_environment.state =
                                                ConnectionState::Unreachable;
                                            this.active_environment.message = Some("ssh_cancelled");
                                        }
                                    }
                                    cx.notify();
                                });
                            });
                            break;
                        }
                    }
                }));
            }
        }
        cx.notify();
    }
    pub(super) fn environment_panel(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        if std::mem::take(&mut self.active_environment.enter) {
            let generation = self.active_environment.generation;
            let weak = cx.weak_entity();
            window.on_next_frame(move |window, cx| {
                let _ = weak.update(cx, |this, cx| {
                    if this.active_environment.generation == generation
                        && (this.active_environment.pending.is_some()
                            || this.active_environment.probe.is_some())
                    {
                        window.focus(&this.leave_focus, cx);
                        window.focus_next(cx);
                    }
                });
            });
        }
        if let Some(target) = self.active_environment.pending.clone() {
            let target_name = match &target {
                EnvironmentId::Ssh(id) => self
                    .remote
                    .read(cx)
                    .hosts()
                    .iter()
                    .find(|host| &host.id == id)
                    .map(|host| host.name.clone())
                    .unwrap_or_else(|| text(cx, "Unavailable SSH host", "SSH 主机不可用").into()),
                _ => text(cx, "Local", "本地").into(),
            };
            let direction = format!("{} → {}", self.environment_name(cx), target_name);
            return Some(div().id("environment-switch-confirmation").test_support().role(gpui_kit::Role::AlertDialog).aria_label(text(cx,"Switch execution environment","切换执行环境"))
                .p(px(24.)).flex().flex_col().gap(px(16.)).track_focus(&self.leave_focus)
                .child(div().id("environment-switch-target").test_support().aria_label(direction.clone()).child(direction))
                .child(text(cx,"Switch execution environment? Unsaved drafts will be discarded. Running jobs receive cancellation; switching waits for their cleanup.","切换执行环境？未保存草稿将被放弃。将请求取消进行中的作业，等待清理结束后切换。"))
                .child(div().flex().gap(px(8.))
                    .child(semantics::button(Button::new("environment-switch-stay").label(text(cx,"Stay here","留在当前环境")).ghost().on_click(cx.listener(|this,_,window,cx|this.dismiss_environment(window,cx))),false,false,cx))
                    .child(semantics::button(Button::new("environment-switch-discard").label(text(cx,"Cancel jobs, discard and switch","取消作业、放弃草稿并切换")).on_click(cx.listener(move |this,_,window,cx|this.switch_environment(target.clone(),window,cx))),false,false,cx)).focus_trap("environment-switch-trap", &self.leave_focus))
                .into_any_element());
        }
        if let Some(probe) = self.active_environment.probe.clone() {
            let challenge = probe.challenge_id.clone();
            return Some(
                div()
                    .id("environment-trust-confirmation")
                    .test_support()
                    .role(gpui_kit::Role::AlertDialog)
                    .aria_label(text(cx, "Review SSH host key", "确认 SSH 主机密钥"))
                    .p(px(24.))
                    .flex()
                    .flex_col()
                    .gap(px(16.))
                    .track_focus(&self.leave_focus)
                    .child(text(
                        cx,
                        "Verify the host key through a separate trusted channel before connecting.",
                        "连接前，请通过独立可信渠道核对主机密钥。",
                    ))
                    .child(format!("{}:{}", probe.host, probe.port))
                    .child(probe.fingerprint)
                    .child(format!(
                        "{}: {}",
                        text(cx, "Previously saved fingerprint", "之前保存的指纹"),
                        probe
                            .stored_fingerprint
                            .unwrap_or_else(|| text(cx, "None", "无").into())
                    ))
                    .child(
                        div()
                            .flex()
                            .gap(px(8.))
                            .child(semantics::button(
                                Button::new("environment-trust-cancel")
                                    .label(text(cx, "Keep disconnected", "保持未连接"))
                                    .ghost()
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.dismiss_environment(window, cx)
                                    })),
                                false,
                                false,
                                cx,
                            ))
                            .child(semantics::button(
                                Button::new("environment-trust-confirm")
                                    .label(text(cx, "Trust and connect", "信任并连接"))
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.start_environment_action(
                                            SshAction::Confirm(challenge.clone()),
                                            cx,
                                        )
                                    })),
                                false,
                                false,
                                cx,
                            ))
                            .focus_trap("environment-trust-trap", &self.leave_focus),
                    )
                    .into_any_element(),
            );
        }
        None
    }
    pub(super) fn environment_menu(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        if !self.active_environment.menu {
            return None;
        }
        if std::mem::take(&mut self.active_environment.menu_enter) {
            let generation = self.active_environment.menu_generation;
            let weak = cx.weak_entity();
            window.on_next_frame(move |window, cx| {
                let _ = weak.update(cx, |this, cx| {
                    if this.active_environment.menu
                        && this.active_environment.menu_generation == generation
                    {
                        window.focus(&this.active_environment.menu_focus, cx);
                        window.focus_next(cx);
                    }
                });
            });
        }
        let mut menu = div()
            .id("environment-menu")
            .test_support()
            .role(gpui_kit::Role::Menu)
            .aria_label(text(cx, "Execution environment", "执行环境"))
            .track_focus(&self.active_environment.menu_focus)
            .tab_stop(false)
            .on_mouse_down_out(
                cx.listener(|this, _, window, cx| this.close_environment_menu(true, window, cx)),
            )
            .on_key_down(
                cx.listener(|this, event: &gpui_kit::KeyDownEvent, window, cx| {
                    let count = this.remote.read(cx).hosts().len()
                        + 2
                        + usize::from(
                            this.active_environment.is_ssh()
                                && this.active_environment.state == ConnectionState::Connected,
                        );
                    match event.keystroke.key.as_str() {
                        "down" => {
                            window.focus_next(cx);
                            if !this
                                .active_environment
                                .menu_focus
                                .contains_focused(window, cx)
                            {
                                window.focus(&this.active_environment.menu_focus, cx);
                                window.focus_next(cx);
                            }
                        }
                        "up" => {
                            window.focus_prev(cx);
                            if !this
                                .active_environment
                                .menu_focus
                                .contains_focused(window, cx)
                            {
                                window.focus(&this.active_environment.menu_focus, cx);
                                for _ in 0..count {
                                    window.focus_next(cx);
                                }
                            }
                        }
                        "home" => {
                            window.focus(&this.active_environment.menu_focus, cx);
                            window.focus_next(cx);
                        }
                        "end" => {
                            window.focus(&this.active_environment.menu_focus, cx);
                            for _ in 0..count {
                                window.focus_next(cx);
                            }
                        }
                        "escape" => this.close_environment_menu(true, window, cx),
                        _ => return,
                    }
                    cx.stop_propagation();
                }),
            )
            .absolute()
            .bottom(px(theme::STATUS_HEIGHT))
            .left(px(12.))
            .w(px(360.))
            .max_h(px(360.))
            .overflow_y_scroll()
            .p(px(12.))
            .flex()
            .flex_col()
            .gap(px(8.))
            .bg(cx.theme().popover)
            .rounded(px(8.))
            .border_1()
            .border_color(cx.theme().border);
        let local = EnvironmentId::Local("local".into());
        menu = menu.child(semantics::button(
            Button::new("environment-local")
                .role(gpui_kit::Role::MenuItem)
                .label(text(cx, "Local · Ready", "本地 · 就绪"))
                .ghost()
                .on_click(cx.listener(move |this, _, window, cx| {
                    this.request_environment(local.clone(), window, cx)
                })),
            false,
            false,
            cx,
        ));
        let hosts: Vec<Host> = self.remote.read(cx).hosts().to_vec();
        if hosts.is_empty() {
            menu = menu.child(text(
                cx,
                "No SSH hosts. Add one in execution environment settings.",
                "尚无 SSH 主机，请在执行环境设置中添加。",
            ));
        }
        for host in hosts {
            let id = host.id.clone();
            let state = if self.active_environment.id == EnvironmentId::Ssh(id.clone()) {
                self.active_environment.state
            } else {
                ConnectionState::Disconnected
            };
            menu = menu.child(semantics::button(
                Button::new(gpui_kit::SharedString::from(format!(
                    "environment-host-{id}"
                )))
                .role(gpui_kit::Role::MenuItem)
                .label(format!("{} · {}", host.name, state.label(cx)))
                .ghost()
                .on_click(cx.listener(move |this, _, window, cx| {
                    this.request_environment(EnvironmentId::Ssh(id.clone()), window, cx)
                })),
                false,
                false,
                cx,
            ));
        }
        if self.active_environment.is_ssh()
            && self.active_environment.state == ConnectionState::Connected
        {
            menu = menu.child(semantics::button(
                Button::new("environment-reconnect")
                    .role(gpui_kit::Role::MenuItem)
                    .label(text(
                        cx,
                        "Reconnect current SSH host",
                        "重新连接当前 SSH 主机",
                    ))
                    .ghost()
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.request_environment_switch(
                            this.active_environment.id.clone(),
                            true,
                            window,
                            cx,
                        )
                    })),
                false,
                false,
                cx,
            ));
        }
        menu = menu.child(semantics::button(
            Button::new("environment-manage")
                .role(gpui_kit::Role::MenuItem)
                .label(text(cx, "Manage SSH hosts", "管理 SSH 主机"))
                .ghost()
                .on_click(cx.listener(|this, _, window, cx| {
                    this.close_environment_menu(true, window, cx);
                    this.open_environment_settings(window, cx);
                })),
            false,
            false,
            cx,
        ));
        Some(
            menu.focus_trap("environment-menu", &self.active_environment.menu_focus)
                .into_any_element(),
        )
    }
}
