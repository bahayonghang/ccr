//! Native saved-host selection, explicit trust and per-host SFTP documents.
use crate::{
    locales::text,
    runtime::{
        Services,
        environments::{SshAction, SshResult},
    },
    shell::semantics,
    theme,
};
use ccr_app::{
    environments::ssh::{ConfigFile, Document, Host, HostInput, Probe},
    runtime::{CancellationHandle, Completion, EnvironmentId, Platform, RequestKey, TaskEvent},
};
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::input::{Input, InputState, Textarea, TextareaState};
use gpui_kit::{
    AppContext, Context, Entity, IntoElement, ParentElement, Render, SharedString, Styled, Task,
    Window, div, px,
};
use std::collections::HashMap;

#[derive(Clone)]
struct Draft {
    document: Document,
    text: String,
}
pub struct RemoteView {
    services: Services,
    hosts: Vec<Host>,
    selected: Option<Host>,
    connected: bool,
    file: ConfigFile,
    drafts: HashMap<(String, &'static str), Draft>,
    editor: Entity<TextareaState>,
    probe: Option<Probe>,
    busy: bool,
    generation: u64,
    task: Option<Task<()>>,
    cancel: Option<CancellationHandle>,
    message: Option<&'static str>,
    loaded: bool,
    save_confirmation: bool,
    read_confirmation: bool,
    host_form: Option<HostInput>,
    host_fields: [Entity<InputState>; 5],
    host_form_pending: bool,
    delete_confirmation: bool,
}
impl RemoteView {
    pub fn new(services: Services, window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            services,
            hosts: Vec::new(),
            selected: None,
            connected: false,
            file: ConfigFile::Claude,
            drafts: HashMap::new(),
            editor: cx.new(|cx| TextareaState::new(window, cx)),
            probe: None,
            busy: false,
            generation: 0,
            task: None,
            cancel: None,
            message: None,
            loaded: false,
            save_confirmation: false,
            read_confirmation: false,
            host_form: None,
            host_fields: std::array::from_fn(|_| cx.new(|cx| InputState::new(window, cx))),
            host_form_pending: false,
            delete_confirmation: false,
        }
    }
    pub fn is_remote(&self) -> bool {
        self.selected.is_some()
    }
    pub fn status(&self) -> String {
        self.selected
            .as_ref()
            .map(|h| format!("SSH · {}", h.name))
            .unwrap_or_else(|| "Local".into())
    }
    pub fn refresh(&mut self, cx: &mut Context<Self>) {
        if !self.loaded && !self.busy {
            self.start(SshAction::Hosts, cx);
        }
    }
    fn remember(&mut self, cx: &Context<Self>) {
        if self.message == Some("ssh_document_ready") {
            return;
        }
        if let Some(host) = &self.selected
            && let Some(draft) = self.drafts.get_mut(&(host.id.clone(), self.file.path()))
        {
            draft.text = self.editor.read(cx).value().to_string();
        }
    }
    fn restore(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let value = self
            .selected
            .as_ref()
            .and_then(|h| self.drafts.get(&(h.id.clone(), self.file.path())))
            .map(|d| d.text.clone())
            .unwrap_or_default();
        self.editor
            .update(cx, |state, cx| state.set_value(value, window, cx));
    }
    fn select(&mut self, host: Host, window: &mut Window, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        self.remember(cx);
        self.host_form = None;
        self.delete_confirmation = false;
        self.read_confirmation = false;
        self.selected = Some(host);
        self.connected = false;
        self.probe = None;
        self.message = None;
        self.save_confirmation = false;
        self.generation += 1;
        self.restore(window, cx);
        cx.notify();
    }
    fn select_file(&mut self, file: ConfigFile, window: &mut Window, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        self.remember(cx);
        self.file = file;
        self.message = None;
        self.save_confirmation = false;
        self.read_confirmation = false;
        self.generation += 1;
        self.restore(window, cx);
        cx.notify();
    }
    fn host_form(&mut self, input: HostInput, window: &mut Window, cx: &mut Context<Self>) {
        let values = [
            input.name.clone(),
            input.host.clone(),
            input.port.to_string(),
            input.user.clone(),
            input.remote_home.clone(),
        ];
        for (field, value) in self.host_fields.iter().zip(values) {
            field.update(cx, |state, cx| state.set_value(value, window, cx));
        }
        self.host_form = Some(input);
        self.host_form_pending = false;
        cx.notify();
    }
    fn save_host(&mut self, cx: &mut Context<Self>) {
        let Some(form) = &self.host_form else { return };
        let fields = self
            .host_fields
            .iter()
            .map(|field| field.read(cx).value().to_string())
            .collect::<Vec<_>>();
        let Ok(port) = fields[2].parse::<u16>() else {
            self.message = Some("ssh_invalid_target");
            cx.notify();
            return;
        };
        self.start(
            SshAction::SaveHost(HostInput {
                id: form.id.clone(),
                name: fields[0].clone(),
                host: fields[1].clone(),
                port,
                user: fields[3].clone(),
                remote_home: fields[4].clone(),
            }),
            cx,
        );
    }
    fn start(&mut self, action: SshAction, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        self.remember(cx);
        self.generation += 1;
        let generation = self.generation;
        let environment = self
            .selected
            .as_ref()
            .map(|h| EnvironmentId::Ssh(h.id.clone()))
            .unwrap_or(EnvironmentId::Local("local".into()));
        let key = RequestKey {
            environment,
            platform: Platform::Claude,
            generation,
            request_id: generation,
        };
        self.message = None;
        self.busy = true;
        self.save_confirmation = false;
        self.read_confirmation = false;
        self.delete_confirmation = false;
        match self.services.ssh_action(key, action) {
            Err(_) => {
                self.busy = false;
                self.message = Some("ssh_busy");
            }
            Ok(mut subscription) => {
                self.cancel = Some(subscription.cancellation_handle());
                self.task = Some(cx.spawn(async move |weak, cx| {
                    while let Some(event) = subscription.next().await {
                        if let TaskEvent::Terminal(result) = event {
                            let _ = weak.update(cx, |this, cx| {
                                if this.generation != generation {
                                    return;
                                }
                                this.busy = false;
                                this.cancel = None;
                                match result.as_ref() {
                                    Completion::Success(SshResult::Hosts(hosts)) => {
                                        this.hosts = hosts.clone();
                                        this.loaded = true;
                                    }
                                    Completion::Success(SshResult::HostDetails(input)) => {
                                        this.host_form = Some(input.clone());
                                        this.host_form_pending = true;
                                    }
                                    Completion::Success(SshResult::HostSaved(host)) => {
                                        this.hosts.retain(|item| item.id != host.id);
                                        this.hosts.push(host.clone());
                                        this.selected = Some(host.clone());
                                        this.connected = false;
                                        this.host_form = None;
                                        this.probe = None;
                                        this.message = Some("ssh_document_ready");
                                    }
                                    Completion::Success(SshResult::HostDeleted) => {
                                        if let Some(host) = this.selected.take() {
                                            this.hosts.retain(|item| item.id != host.id);
                                            this.drafts.retain(|(id, _), _| id != &host.id);
                                        }
                                        this.connected = false;
                                        this.delete_confirmation = false;
                                        this.probe = None;
                                    }
                                    Completion::Success(SshResult::Probe(probe)) => {
                                        this.probe = Some(probe.clone());
                                        this.connected = false;
                                    }
                                    Completion::Success(SshResult::Confirmed) => {
                                        this.probe = None;
                                        this.connected = false;
                                        this.message = Some("ssh_trusted");
                                    }
                                    Completion::Success(SshResult::Connected) => {
                                        this.connected = true;
                                        this.message = Some("ssh_connected");
                                    }
                                    Completion::Success(SshResult::Disconnected) => {
                                        this.selected = None;
                                        this.connected = false;
                                        this.probe = None;
                                        this.message = None;
                                    }
                                    Completion::Success(SshResult::Document(document)) => {
                                        this.drafts.insert(
                                            (document.environment.clone(), document.file.path()),
                                            Draft {
                                                document: document.clone(),
                                                text: document.content.clone(),
                                            },
                                        );
                                        this.message = Some("ssh_document_ready");
                                    }
                                    Completion::Failed(error) => {
                                        this.message = Some(*error);
                                        if !matches!(
                                            *error,
                                            "ssh_conflict" | "ssh_invalid_document"
                                        ) {
                                            this.connected = false;
                                        }
                                    }
                                    Completion::Cancelled => {
                                        this.message = Some("ssh_cancelled");
                                        this.connected = false;
                                    }
                                    Completion::CleanupFailed(error) => {
                                        this.message = Some(*error);
                                        this.connected = false;
                                    }
                                    _ => {
                                        this.message = Some("ssh_failed");
                                        this.connected = false;
                                    }
                                }
                                cx.notify();
                            });
                            break;
                        }
                    }
                }));
            }
        }
        cx.notify();
    }
    fn read(&mut self, cx: &mut Context<Self>) {
        self.remember(cx);
        if let Some(host) = &self.selected
            && let Some(draft) = self.drafts.get(&(host.id.clone(), self.file.path()))
            && draft.text != draft.document.content
        {
            self.read_confirmation = true;
            cx.notify();
            return;
        }
        self.load_document(cx);
    }
    fn load_document(&mut self, cx: &mut Context<Self>) {
        self.read_confirmation = false;
        if let Some(host) = &self.selected {
            self.start(SshAction::Read(host.id.clone(), self.file), cx);
        }
    }

    fn save(&mut self, cx: &mut Context<Self>) {
        self.remember(cx);
        if let Some(host) = &self.selected
            && let Some(draft) = self.drafts.get(&(host.id.clone(), self.file.path()))
        {
            self.start(
                SshAction::Save(draft.document.clone(), draft.text.clone()),
                cx,
            );
        }
    }
}
fn message(code: &str, chinese: bool) -> &'static str {
    match (code, chinese) {
        ("ssh_connected", false) => "Connected. Remote configuration access uses SFTP.",
        ("ssh_connected", true) => "已连接，远程配置通过 SFTP 访问。",
        ("ssh_trusted", false) => "Host key saved. Connect to verify this host again.",
        ("ssh_trusted", true) => "已保存主机密钥。连接时会再次验证。",
        ("ssh_document_ready", false) => "Remote document loaded or saved.",
        ("ssh_document_ready", true) => "远程文件已读取或保存。",
        ("ssh_session_changed", false) => {
            "This document belongs to a previous connection. Reload before saving; your draft is retained."
        }
        ("ssh_session_changed", true) => "文件属于之前的连接。请重新读取后保存；草稿已保留。",
        ("ssh_write_outcome_unknown", false) => {
            "The remote write outcome could not be confirmed. Reconnect and reload before retrying; your draft is retained."
        }
        ("ssh_write_outcome_unknown", true) => {
            "无法确认远程写入结果。请重新连接并读取后再决定是否重试；草稿已保留。"
        }
        ("ssh_conflict", false) => {
            "Remote content changed. Your draft is retained; reload before saving."
        }
        ("ssh_conflict", true) => "远程文件已更改。草稿已保留，请重新读取后保存。",
        ("ssh_host_key_mismatch", false) => {
            "Host key changed. Connection refused. Verify the new fingerprint independently."
        }
        ("ssh_host_key_mismatch", true) => "主机密钥已变更，连接被拒绝。请通过独立渠道核验新指纹。",
        ("ssh_host_key_untrusted", false) => {
            "Host key is not trusted. Inspect and explicitly confirm its fingerprint first."
        }
        ("ssh_host_key_untrusted", true) => "主机密钥尚未受信任。请先检查并明确确认指纹。",
        ("ssh_auth_unsupported", false) => {
            "Authentication failed. Use a saved key or an SSH agent; interactive passwords are unsupported."
        }
        ("ssh_auth_unsupported", true) => {
            "认证失败。请使用保存的密钥或 SSH agent，不支持交互式密码。"
        }
        ("ssh_cancelled", false) => {
            "Operation cancelled after cleanup. Reconnect before accessing remote files."
        }
        ("ssh_cancelled", true) => "操作已取消并完成清理。访问远程文件前请重新连接。",
        ("ssh_cleanup_failed", false) => {
            "Process cleanup could not be confirmed. The remote write outcome may be unknown. Reconnect and reload before retrying."
        }
        ("ssh_cleanup_failed", true) => {
            "无法确认进程清理完成，远程写入结果可能未知。请重新连接并读取后再决定是否重试。"
        }
        ("ssh_invalid_document", false) => "Invalid JSON or TOML. The remote file was not changed.",
        ("ssh_invalid_document", true) => "JSON 或 TOML 无效，远程文件未更改。",
        (_, false) => {
            "SSH operation failed. Check the saved host, trust, connection and file permissions; no local data was used."
        }
        (_, true) => "SSH 操作失败。请检查保存的主机、信任、连接和文件权限；没有使用本地数据。",
    }
}
impl Render for RemoteView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.host_form_pending
            && let Some(form) = self.host_form.clone()
        {
            self.host_form(form, window, cx);
        }
        if self.message == Some("ssh_document_ready") {
            self.restore(window, cx);
            self.message = None;
        }
        let chinese = crate::locales::is_chinese(cx);
        let mut root = div()
            .flex()
            .flex_col()
            .gap(px(theme::CONTROL_GAP))
            .child(text(cx, "SSH environments", "SSH 环境"));
        let local_busy = self.busy;
        root = root.child(semantics::button(
            Button::new("ssh-local")
                .label(text(cx, "Use Local", "使用本地环境"))
                .ghost()
                .on_click(cx.listener(|this, _, _, cx| {
                    this.remember(cx);
                    this.start(SshAction::Disconnect, cx);
                })),
            local_busy,
            false,
            cx,
        ));
        for host in self.hosts.clone() {
            let selected = self.selected.as_ref().is_some_and(|h| h.id == host.id);
            let label = format!("{}{}", if selected { "● " } else { "" }, host.name);
            root = root.child(semantics::button(
                Button::new(SharedString::from(format!("ssh-host-{}", host.id)))
                    .label(label)
                    .ghost()
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.select(host.clone(), window, cx)
                    })),
                self.busy,
                false,
                cx,
            ));
        }
        if self.loaded && self.hosts.is_empty() {
            root = root.child(text(
                cx,
                "No saved SSH hosts. Add a host to get started.",
                "没有保存的 SSH 主机，请先添加主机。",
            ));
        }
        root = root.child(semantics::button(
            Button::new("ssh-hosts-reload")
                .label(text(cx, "Reload hosts", "重新加载主机"))
                .ghost()
                .on_click(cx.listener(|this, _, _, cx| this.start(SshAction::Hosts, cx))),
            self.busy,
            false,
            cx,
        ));
        root = root.child(semantics::button(
            Button::new("ssh-add-host")
                .label(text(cx, "Add SSH host", "添加 SSH 主机"))
                .ghost()
                .on_click(cx.listener(|this, _, window, cx| {
                    this.host_form(
                        HostInput {
                            id: None,
                            name: String::new(),
                            host: String::new(),
                            port: 22,
                            user: String::new(),
                            remote_home: "~".into(),
                        },
                        window,
                        cx,
                    )
                })),
            self.busy,
            false,
            cx,
        ));
        if self.host_form.is_some() {
            root=root.child(text(cx,"New hosts use an SSH agent. Editing keeps an existing identity-file setting. Passwords are unsupported.","新主机使用 SSH agent；编辑时保留已有身份文件设置。不支持密码认证。"));
            for (index, (en, zh)) in [
                ("Name", "名称"),
                ("Host", "主机"),
                ("Port", "端口"),
                ("User", "用户"),
                ("Remote home", "远程主目录"),
            ]
            .into_iter()
            .enumerate()
            {
                root = root.child(text(cx, en, zh)).child(
                    Input::new(&self.host_fields[index])
                        .accessibility_id(SharedString::from(format!("ssh-host-field-{index}")))
                        .aria_label(text(cx, en, zh))
                        .disabled(self.busy),
                );
            }
            root = root.child(semantics::button(
                Button::new("ssh-host-save")
                    .label(text(cx, "Save host", "保存主机"))
                    .on_click(cx.listener(|this, _, _, cx| this.save_host(cx))),
                self.busy,
                false,
                cx,
            ));
            root = root.child(semantics::button(
                Button::new("ssh-host-cancel")
                    .label(text(cx, "Cancel", "取消"))
                    .ghost()
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.host_form = None;
                        cx.notify();
                    })),
                self.busy,
                false,
                cx,
            ));
        }
        if let Some(host) = self.selected.clone() {
            let edit_host = host.clone();
            root = root.child(semantics::button(
                Button::new("ssh-edit-host")
                    .label(text(cx, "Edit host", "编辑主机"))
                    .ghost()
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.start(SshAction::HostDetails(edit_host.id.clone()), cx)
                    })),
                self.busy,
                false,
                cx,
            ));
            root = root.child(semantics::button(
                Button::new("ssh-delete-host")
                    .label(text(cx, "Delete host", "删除主机"))
                    .ghost()
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.delete_confirmation = true;
                        cx.notify();
                    })),
                self.busy,
                false,
                cx,
            ));
            if self.delete_confirmation {
                let deleted = host.clone();
                root = root.child(text(
                    cx,
                    "Delete this saved host? Remote files are retained.",
                    "删除此保存的主机？远程文件将保留。",
                ));
                root = root.child(semantics::button(
                    Button::new("ssh-cancel-delete")
                        .label(text(cx, "Cancel", "取消"))
                        .ghost()
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.delete_confirmation = false;
                            cx.notify();
                        })),
                    self.busy,
                    false,
                    cx,
                ));
                root = root.child(semantics::button(
                    Button::new("ssh-confirm-delete")
                        .label(text(cx, "Confirm deletion", "确认删除"))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.start(SshAction::DeleteHost(deleted.id.clone()), cx)
                        })),
                    self.busy,
                    false,
                    cx,
                ));
            }
            root = root.child(if self.connected {
                text(cx, "Connected", "已连接")
            } else {
                text(cx, "Selected · Not connected", "已选择 · 未连接")
            });
            for (id, en, zh, probe) in [
                ("ssh-probe", "Inspect host key", "检查主机密钥", true),
                ("ssh-connect", "Connect", "连接", false),
            ] {
                let host = host.clone();
                root = root.child(semantics::button(
                    Button::new(id)
                        .label(text(cx, en, zh))
                        .ghost()
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.start(
                                if probe {
                                    SshAction::Probe(host.id.clone())
                                } else {
                                    SshAction::Connect(host.id.clone())
                                },
                                cx,
                            )
                        })),
                    self.busy,
                    false,
                    cx,
                ));
            }
            if let Some(probe) = self.probe.clone() {
                root = root.child(format!(
                    "{}:{} · {}",
                    probe.host, probe.port, probe.fingerprint
                ));
                if let Some(stored) = probe.stored_fingerprint {
                    root = root.child(format!(
                        "{}: {}",
                        text(cx, "Previously saved", "之前保存"),
                        stored
                    ));
                }
                root = root.child(text(
                    cx,
                    "The connection was closed for trust review. Verify this fingerprint through a separate trusted channel before saving it, then reconnect.",
                    "连接已断开以重新检查信任。保存前请通过独立可信渠道核对指纹，然后重新连接。",
                ));
                root = root.child(semantics::button(
                    Button::new("ssh-confirm-trust")
                        .label(text(cx, "Trust this host key", "信任此主机密钥"))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.start(SshAction::Confirm(probe.challenge_id.clone()), cx)
                        })),
                    self.busy,
                    false,
                    cx,
                ));
                root = root.child(semantics::button(
                    Button::new("ssh-reject-trust")
                        .label(text(cx, "Cancel", "取消"))
                        .ghost()
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.probe = None;
                            cx.notify();
                        })),
                    self.busy,
                    false,
                    cx,
                ));
            }
            root=root.child(text(cx,"Remote source files may contain credentials. Load only when you intend to view plaintext. Other pages are Local-only in this release.","远程源文件可能包含凭据。仅在需要查看明文时读取；当前版本的其他页面仅支持本地环境。"));
            for (index, file) in ConfigFile::ALL.into_iter().enumerate() {
                root = root.child(semantics::button(
                    Button::new(SharedString::from(format!("ssh-file-{index}")))
                        .label(file.path())
                        .ghost()
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.select_file(file, window, cx)
                        })),
                    self.busy,
                    false,
                    cx,
                ));
            }
            root = root.child(format!(
                "{}: {}",
                text(cx, "Selected file", "选中文件"),
                self.file.path()
            ));
            let draft = self.drafts.get(&(host.id.clone(), self.file.path()));
            root = root.child(semantics::button(
                Button::new("ssh-read")
                    .label(text(cx, "Load remote plaintext", "读取远程明文"))
                    .ghost()
                    .on_click(cx.listener(|this, _, _, cx| this.read(cx))),
                self.busy || !self.connected,
                false,
                cx,
            ));
            if draft.is_some() {
                root = root.child(
                    Textarea::new(&self.editor)
                        .accessibility_id("ssh-document")
                        .aria_label(text(cx, "Remote configuration source", "远程配置源文件"))
                        .h(px(200.))
                        .readonly(self.busy),
                );
                root = root.child(semantics::button(
                    Button::new("ssh-save")
                        .label(text(cx, "Save with remote backup", "备份后保存到远程"))
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.save_confirmation = true;
                            cx.notify();
                        })),
                    self.busy || !self.connected,
                    false,
                    cx,
                ));
            }
            if self.read_confirmation {
                root = root.child(text(
                    cx,
                    "Reloading will replace this unsaved draft with remote content.",
                    "重新读取会用远程内容替换当前未保存草稿。",
                ));
                root = root.child(semantics::button(
                    Button::new("ssh-cancel-reload")
                        .label(text(cx, "Keep draft", "保留草稿"))
                        .ghost()
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.read_confirmation = false;
                            cx.notify();
                        })),
                    false,
                    false,
                    cx,
                ));
                root = root.child(semantics::button(
                    Button::new("ssh-confirm-reload")
                        .label(text(cx, "Discard draft and reload", "放弃草稿并重新读取"))
                        .on_click(cx.listener(|this, _, _, cx| this.load_document(cx))),
                    false,
                    false,
                    cx,
                ));
            }
            if self.save_confirmation {
                root=root.child(text(cx,"Replace this remote file after checking its loaded revision? A remote backup is required before replacement.","检查读取时的版本并替换此远程文件？替换前必须成功创建远程备份。"));
                root = root.child(semantics::button(
                    Button::new("ssh-cancel-save")
                        .label(text(cx, "Cancel", "取消"))
                        .ghost()
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.save_confirmation = false;
                            cx.notify();
                        })),
                    false,
                    false,
                    cx,
                ));
                root = root.child(semantics::button(
                    Button::new("ssh-confirm-save")
                        .label(text(cx, "Confirm remote save", "确认远程保存"))
                        .on_click(cx.listener(|this, _, _, cx| this.save(cx))),
                    false,
                    false,
                    cx,
                ));
            }
        }
        if self.busy {
            root = root.child(semantics::button(
                Button::new("ssh-cancel")
                    .label(text(cx, "Cancel operation", "取消操作"))
                    .ghost()
                    .on_click(cx.listener(|this, _, _, cx| {
                        if let Some(cancel) = &this.cancel {
                            cancel.request_cancel();
                        }
                        cx.notify();
                    })),
                false,
                false,
                cx,
            ));
        }
        if let Some(code) = self.message {
            root = root.child(message(code, chinese));
        }
        root
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use ccr_app::{environments::ssh::Workspace, runtime::RuntimeOwner};
    use gpui_kit::test::{TestAppContextExt, TestWindowExt};
    use gpui_kit::{Bounds, TestAppContext, WindowBounds, WindowOptions, size};
    use std::{sync::Arc, time::Duration};

    #[gpui_kit::test]
    async fn rendered_host_actions_reach_isolated_transport_and_write_remote_document(
        cx: &mut TestAppContext,
    ) {
        cx.executor().allow_parking();
        let root = tempfile::tempdir().unwrap();
        let owner = RuntimeOwner::new().unwrap();
        let (ssh, transport) = Workspace::fixture(
            root.path().join("ssh.db"),
            root.path().join("ssh/known_hosts"),
        );
        let ssh = Arc::new(ssh);
        let setup = ssh.clone();
        let first = owner.handle();
        let host = first
            .read(
                RequestKey {
                    environment: EnvironmentId::Local("fixture".into()),
                    platform: Platform::Claude,
                    generation: 0,
                    request_id: 0,
                },
                move |_, _| {
                    let handle = tokio::runtime::Handle::current();
                    let input = HostInput {
                        id: None,
                        name: "First".into(),
                        host: "one.example".into(),
                        port: 22,
                        user: "fixture".into(),
                        remote_home: "/home/fixture".into(),
                    };
                    handle.block_on(setup.save_host(input))
                },
            )
            .unwrap();
        let mut host = host;
        let Some(TaskEvent::Terminal(result)) = host.next().await else {
            panic!("host result")
        };
        let Completion::Success(host) = result.as_ref() else {
            panic!("host save")
        };
        // Transfer the real owner, with its explicitly synthetic transport, to Services.
        let host = host.clone();
        let services = Services::new(
            owner.handle(),
            crate::runtime::ConfigSource::Fixture {
                profiles: root.path().join("profiles.toml"),
                registry: root.path().join("registry.toml"),
                delay: Duration::ZERO,
            },
        );
        transport.set_file(
            "one.example",
            "/home/fixture/.claude/settings.json",
            r#"{"original":true}"#,
        );
        services
            .ssh
            .set(ssh)
            .unwrap_or_else(|_| panic!("owner set"));
        let (handle, view) = cx.update(|cx| {
            gpui_kit::init(cx);
            cx.set_global(crate::locales::Language::English);
            gpui_kit::open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                        None,
                        size(px(1200.), px(1800.)),
                        cx,
                    ))),
                    ..Default::default()
                },
                cx,
                |window, cx| cx.new(|cx| RemoteView::new(services, window, cx)),
            )
            .unwrap()
        });
        cx.update_window(handle, |_, window, cx| {
            view.update(cx, |this, cx| this.refresh(cx));
            window.render_frame(cx);
        })
        .unwrap();
        cx.wait_for(handle, Duration::from_secs(5), |_, cx| !view.read(cx).busy)
            .await;
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            window.click(SharedString::from(format!("ssh-host-{}", host.id)), cx);
            window.render_frame(cx);
            window.click("ssh-connect", cx);
        })
        .unwrap();
        cx.wait_for(handle, Duration::from_secs(5), |_, cx| !view.read(cx).busy)
            .await;
        cx.update_window(handle, |_, window, cx| {
            assert!(!view.read(cx).connected);
            assert_eq!(view.read(cx).message, Some("ssh_host_key_untrusted"));
            window.render_frame(cx);
            window.click("ssh-probe", cx);
        })
        .unwrap();
        cx.wait_for(handle, Duration::from_secs(5), |_, cx| !view.read(cx).busy)
            .await;
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            window.click("ssh-confirm-trust", cx);
        })
        .unwrap();
        cx.wait_for(handle, Duration::from_secs(5), |_, cx| !view.read(cx).busy)
            .await;
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            window.click("ssh-connect", cx);
        })
        .unwrap();
        cx.wait_for(handle, Duration::from_secs(5), |_, cx| !view.read(cx).busy)
            .await;
        cx.update_window(handle, |_, window, cx| {
            assert!(view.read(cx).connected);
            window.render_frame(cx);
            window.click("ssh-read", cx);
        })
        .unwrap();
        cx.wait_for(handle, Duration::from_secs(5), |_, cx| !view.read(cx).busy)
            .await;
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            view.update(cx, |this, cx| {
                this.editor.update(cx, |state, cx| {
                    state.set_value(r#"{"edited":true}"#, window, cx)
                })
            });
            window.render_frame(cx);
            window.click("ssh-save", cx);
            window.render_frame(cx);
            window.click("ssh-confirm-save", cx);
        })
        .unwrap();
        cx.wait_for(handle, Duration::from_secs(5), |_, cx| !view.read(cx).busy)
            .await;
        assert_eq!(
            transport
                .file("one.example", "/home/fixture/.claude/settings.json")
                .as_deref(),
            Some(r#"{"edited":true}"#)
        );
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            view.update(cx, |this, cx| {
                this.editor.update(cx, |state, cx| {
                    state.set_value(r#"{"unsaved":true}"#, window, cx)
                })
            });
            window.render_frame(cx);
            window.click("ssh-read", cx);
            assert!(view.read(cx).read_confirmation);
            window.render_frame(cx);
            window.click("ssh-file-1", cx);
            assert!(!view.read(cx).read_confirmation);
            window.render_frame(cx);
            window.click("ssh-file-0", cx);
            window.render_frame(cx);
            window.click("ssh-add-host", cx);
            view.update(cx, |this, cx| {
                for (field, value) in this.host_fields.iter().zip([
                    "Second",
                    "two.example",
                    "22",
                    "fixture",
                    "/home/fixture",
                ]) {
                    field.update(cx, |state, cx| state.set_value(value, window, cx));
                }
            });
            window.render_frame(cx);
            window.click("ssh-host-save", cx);
        })
        .unwrap();
        cx.wait_for(handle, Duration::from_secs(5), |_, cx| !view.read(cx).busy)
            .await;
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            assert_eq!(view.read(cx).selected.as_ref().unwrap().name, "Second");
            assert!(view.read(cx).editor.read(cx).value().is_empty());
            window.click(SharedString::from(format!("ssh-host-{}", host.id)), cx);
            window.render_frame(cx);
            assert_eq!(
                view.read(cx).editor.read(cx).value().as_str(),
                r#"{"unsaved":true}"#
            );
        })
        .unwrap();
        assert!(!root.path().join("profiles.toml").exists());
        owner.shutdown();
    }
}
