//! Canonical sync settings and retained WebDAV operations.
mod assets;
mod folders;
use crate::{
    locales::{is_chinese, service_error, text},
    runtime::{
        Services,
        sync::{
            FolderChange, FolderPatch, Operation, OperationKind, OperationState, PasswordUpdate,
            SettingsPatch, SyncError, SyncFolderRow, SyncLocal, SyncOutcome, SyncSnapshot, SyncUrl,
        },
    },
    shell::semantics,
    theme,
};
use ccr_app::runtime::{Completion, EnvironmentId, Platform, RequestKey, ResponseGate, TaskEvent};
use folders::{FolderEditor, FolderTarget};
use gpui_kit::base::TestSupportExt;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::input::{Input, InputState};
use gpui_kit::{
    App, AppContext, Context, Entity, FocusHandle, InteractiveElement, IntoElement, ParentElement,
    Render, SharedString, StatefulInteractiveElement, Styled, Task, Window, div, px,
};
use std::sync::Arc;

struct Editor {
    url: Entity<InputState>,
    username: Entity<InputState>,
    password: Entity<InputState>,
    base: Entity<InputState>,
    enabled: bool,
    password_mode: u8,
}

pub struct SyncView {
    services: Services,
    snapshot: Option<SyncSnapshot>,
    generation: u64,
    gate: ResponseGate,
    task: Option<Task<()>>,
    busy: bool,
    message: Option<&'static str>,
    focus: FocusHandle,
    editor: Option<Editor>,
    asset_editor: Option<assets::AssetEditor>,
    folder_editor: Option<FolderEditor>,
    folder_delete: Option<FolderTarget>,
    selected: Option<usize>,
    confirmation: Option<OperationKind>,
    operation: Arc<OperationState>,
    observation: Option<Task<()>>,
}

impl SyncView {
    pub fn new(services: Services, _window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            operation: services.sync_operation_state(),
            services,
            snapshot: None,
            generation: 0,
            gate: ResponseGate::default(),
            task: None,
            busy: false,
            message: None,
            focus: cx.focus_handle().tab_stop(false),
            editor: None,
            asset_editor: None,
            folder_editor: None,
            folder_delete: None,
            selected: None,
            confirmation: None,
            observation: None,
        }
    }

    pub fn focus_main(&self, window: &mut Window, cx: &mut Context<Self>) {
        window.focus(&self.focus, cx);
        window.focus_next(cx);
    }

    pub fn suspend(&mut self, cx: &mut Context<Self>) {
        self.task = None;
        self.gate.clear();
        self.generation += 1;
        self.busy = false;
        self.observation = None;
        self.editor = None;
        self.asset_editor = None;
        self.confirmation = None;
        self.folder_editor = None;
        self.folder_delete = None;
        cx.notify();
    }

    pub fn refresh(&mut self, cx: &mut Context<Self>) {
        self.observe(cx);
        if self.busy {
            return;
        }
        self.confirmation = None;
        self.asset_editor = None;
        self.folder_delete = None;
        self.folder_editor = None;
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
        match self.services.sync_request(key.clone()) {
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
                                    Completion::Success(snapshot) => {
                                        this.selected = match snapshot {
                                            SyncSnapshot::Present(local) => local.folders.iter().position(|folder| folder.enabled),
                                            SyncSnapshot::Absent => None,
                                        };
                                        this.snapshot = Some(snapshot.clone());
                                    }
                                    Completion::Failed(error) | Completion::CleanupFailed(error) => {
                                        this.message = Some(error.message());
                                    }
                                    Completion::Cancelled => {
                                        this.message = Some(
                                            "The sync config read did not finish. / 同步配置读取没有完成",
                                        );
                                    }
                                    Completion::WorkerFailed => {
                                        this.message = Some(
                                            "The sync config reader stopped without a result. / 同步配置读取没有返回结果",
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
                self.message = Some(SyncError::Storage.message());
            }
        }
        cx.notify();
    }

    fn observe(&mut self, cx: &mut Context<Self>) {
        self.operation = self.services.sync_operation_state();
        if self.observation.is_some() {
            return;
        }
        let mut updates = self.services.observe_sync();
        self.operation = updates.borrow_and_update().clone();
        self.observation = Some(cx.spawn(async move |weak, cx| {
            while updates.changed().await.is_ok() {
                let state = updates.borrow_and_update().clone();
                if weak
                    .update(cx, |this, cx| {
                        let saved = matches!(
                            state.result,
                            Some(Ok(SyncOutcome::Saved | SyncOutcome::FolderSaved))
                        );
                        this.operation = state;
                        if saved {
                            this.editor = None;
                            this.folder_editor = None;
                            this.folder_delete = None;
                            this.refresh(cx);
                        }
                        cx.notify();
                    })
                    .is_err()
                {
                    break;
                }
            }
        }));
    }

    fn revision(&self) -> Option<String> {
        match self.snapshot.as_ref()? {
            SyncSnapshot::Absent => Some(String::new()),
            SyncSnapshot::Present(local) => Some(local.revision.clone()),
        }
    }

    fn edit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.busy
            || self.operation.busy
            || self.folder_editor.is_some()
            || self.snapshot.is_none()
        {
            return;
        }
        self.confirmation = None;
        self.asset_editor = None;
        self.editor = Some(Editor {
            url: cx.new(|cx| InputState::new(window, cx)),
            username: cx.new(|cx| InputState::new(window, cx)),
            password: cx.new(|cx| InputState::new(window, cx).masked(true)),
            base: cx.new(|cx| InputState::new(window, cx)),
            enabled: matches!(&self.snapshot, Some(SyncSnapshot::Present(local)) if local.webdav_enabled),
            password_mode: 0,
        });
        cx.notify();
    }

    fn start(&mut self, operation: Operation, cx: &mut Context<Self>) {
        if self.busy || self.operation.busy {
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
        self.message = self
            .services
            .start_sync(key, operation)
            .err()
            .map(SyncError::message);
        self.confirmation = None;
        self.folder_delete = None;
        self.observe(cx);
        cx.notify();
    }

    fn save(&mut self, cx: &mut Context<Self>) {
        let (Some(editor), Some(revision)) = (&self.editor, self.revision()) else {
            return;
        };
        let value = |input: &Entity<InputState>| {
            let value = input.read(cx).value().to_string();
            (!value.trim().is_empty()).then(|| value.trim().to_string())
        };
        let password = match editor.password_mode {
            1 => PasswordUpdate::Replace(editor.password.read(cx).value().to_string().into()),
            2 => PasswordUpdate::Clear,
            _ => PasswordUpdate::Keep,
        };
        let patch = SettingsPatch {
            revision,
            enabled: editor.enabled,
            url: value(&editor.url),
            username: value(&editor.username),
            base_remote_path: value(&editor.base),
            password,
        };
        self.start(Operation::Save(patch), cx);
    }

    fn transfer(&mut self, kind: OperationKind, cx: &mut Context<Self>) {
        let (Some(revision), Some(folder)) = (self.revision(), self.selected) else {
            return;
        };
        self.start(
            match kind {
                OperationKind::Push => Operation::Push { revision, folder },
                OperationKind::Pull => Operation::Pull { revision, folder },
                _ => return,
            },
            cx,
        );
    }
}

fn url_line(cx: &App, url: &SyncUrl) -> String {
    match url {
        SyncUrl::Missing => text(cx, "No WebDAV URL is stored.", "未保存 WebDAV URL。").to_string(),
        SyncUrl::Stored => text(cx, "A WebDAV URL is stored.", "已保存 WebDAV URL。").to_string(),
        SyncUrl::Host(host) => {
            if is_chinese(cx) {
                format!("WebDAV 主机：{host}")
            } else {
                format!("WebDAV host: {host}")
            }
        }
    }
}

fn presence_line(
    cx: &App,
    present: bool,
    en_yes: &'static str,
    en_no: &'static str,
    zh_yes: &'static str,
    zh_no: &'static str,
) -> String {
    text(
        cx,
        if present { en_yes } else { en_no },
        if present { zh_yes } else { zh_no },
    )
    .to_string()
}

fn local_lines(cx: &App, local: &SyncLocal) -> Vec<String> {
    let mut lines = vec![
        if local.webdav_enabled {
            text(
                cx,
                "WebDAV is enabled in the local file.",
                "本地文件里启用了 WebDAV。",
            )
            .to_string()
        } else {
            text(
                cx,
                "WebDAV is turned off in the local file.",
                "本地文件里关闭了 WebDAV。",
            )
            .to_string()
        },
        url_line(cx, &local.url),
        presence_line(
            cx,
            local.username_present,
            "A WebDAV username is stored.",
            "No WebDAV username is stored.",
            "已保存 WebDAV 用户名。",
            "未保存 WebDAV 用户名。",
        ),
        presence_line(
            cx,
            local.password_present,
            "A WebDAV password is stored in the local file.",
            "No WebDAV password is stored.",
            "本地文件里保存了 WebDAV 密码。",
            "未保存 WebDAV 密码。",
        ),
        text(
            cx,
            "Remote access was not tested. Use Test connection to check the saved account.",
            "未测试远程访问。使用测试连接检查已保存的账户。",
        )
        .to_string(),
    ];
    if local.folders.is_empty() {
        lines.push(
            text(
                cx,
                "The canonical file has no folders.",
                "规范配置文件里没有文件夹。",
            )
            .to_string(),
        );
    }
    lines
}

fn folder_label(cx: &App, folder: &SyncFolderRow) -> String {
    let state = if folder.enabled {
        text(cx, "enabled", "已启用")
    } else {
        text(cx, "disabled", "已停用")
    };
    if is_chinese(cx) {
        format!(
            "{} · {state} · 本地 {} · 远程 {}",
            folder.name, folder.local_path, folder.remote_path
        )
    } else {
        format!(
            "{} · {state} · local {} · remote {}",
            folder.name, folder.local_path, folder.remote_path
        )
    }
}

impl Render for SyncView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut root = div()
            .id("sync-page")
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
                    .id("sync-heading")
                    .test_support()
                    .role(gpui_kit::Role::Heading)
                    .aria_level(1)
                    .text_size(px(theme::TITLE_SIZE))
                    .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                    .child(text(cx, "Sync", "同步")),
            )
            .child({
                let intro = text(
                    cx,
                    "Manage the canonical local WebDAV account and saved folders. Sensitive assets have a separate encrypted transfer below. Automatic sync is not enabled by this page.",
                    "管理本地规范 WebDAV 账户和已保存的文件夹。敏感资产使用下方独立加密传输，不会启用自动同步。",
                );
                div()
                    .id("sync-intro")
                    .test_support()
                    .aria_label(intro)
                    .child(intro)
            })
            .child(semantics::button(
                Button::new("sync-read")
                    .label(if self.busy {
                        text(cx, "Reading", "读取中")
                    } else {
                        text(cx, "Read local config", "读取本地配置")
                    })
                    .ghost()
                    .on_click(cx.listener(|this, _, _, cx| this.refresh(cx))),
                self.busy || self.editor.is_some() || self.folder_editor.is_some() || self.operation.busy,
                self.busy,
                cx,
            ));
        if self.busy {
            root = root.child(
                div()
                    .id("sync-status")
                    .test_support()
                    .role(gpui_kit::Role::Status)
                    .child(text(
                        cx,
                        "Reading the local sync config.",
                        "正在读取本地同步配置。",
                    )),
            );
        }
        if let Some(message) = self.message {
            let line = service_error(cx, message);
            root = root.child(
                div()
                    .id("sync-status")
                    .test_support()
                    .role(gpui_kit::Role::Status)
                    .aria_label(line)
                    .child(line),
            );
        }
        if let Some(snapshot) = self.snapshot.clone() {
            match snapshot {
                SyncSnapshot::Absent => {
                    let empty = text(
                        cx,
                        "No canonical sync config file was found. Legacy sync config is not read. Remote access was not tested.",
                        "没有找到规范同步配置文件。不会读取旧的同步配置。未测试远程访问。",
                    );
                    root = root.child(
                        div()
                            .id("sync-absent")
                            .test_support()
                            .aria_label(empty)
                            .child(empty),
                    );
                }
                SyncSnapshot::Present(local) => {
                    let lines = local_lines(cx, &local);
                    let summary = lines.join(" ");
                    let mut detail = div()
                        .id("sync-local")
                        .test_support()
                        .aria_label(SharedString::from(summary))
                        .flex()
                        .flex_col()
                        .gap(px(theme::CONTROL_GAP));
                    for line in lines {
                        detail = detail.child(line);
                    }
                    root = root.child(detail);
                    for (index, folder) in local.folders.iter().enumerate() {
                        let label = folder_label(cx, folder);
                        root = root.child(
                            div()
                                .id(SharedString::from(format!("sync-folder-{index}")))
                                .test_support()
                                .h(px(theme::ROW_HEIGHT))
                                .flex()
                                .items_center()
                                .aria_label(SharedString::from(label.clone()))
                                .child(label)
                                .child(semantics::button(
                                    Button::new(("sync-select", index))
                                        .label(if self.selected == Some(index) {
                                            text(cx, "Selected", "已选择")
                                        } else {
                                            text(cx, "Select", "选择")
                                        })
                                        .ghost()
                                        .on_click(cx.listener(move |this, _, _, cx| {
                                            this.generation += 1;
                                            this.selected = Some(index);
                                            this.confirmation = None;
                                            this.folder_editor = None;
                                            this.folder_delete = None;
                                            cx.notify();
                                        })),
                                    self.busy || self.operation.busy,
                                    false,
                                    cx,
                                )),
                        );
                    }
                }
            }
        }
        root.child(self.render_folder_controls(cx))
            .child(self.render_operations(cx))
            .child(self.render_assets(cx))
    }
}

impl SyncView {
    fn render_operations(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let locked = self.busy || self.operation.busy;
        let mut section = div().flex().flex_col().gap(px(theme::CONTROL_GAP));
        let status = if self.operation.busy {
            Some(text(cx, "Sync operation is running. It continues when you leave this page; closing waits for it to finish. Cancellation is unavailable during transfer.", "同步操作正在运行。离开页面后会继续，关闭应用将等待完成。传输期间不提供取消。").to_string())
        } else {
            self.operation.result.as_ref().map(|result| match result {
                Ok(SyncOutcome::Assets(results)) => format!(
                    "{}: {}/{}",
                    text(cx, "Assets completed", "资产已完成"),
                    results.iter().filter(|r| r.result.is_ok()).count(),
                    results.len()
                ),
                Ok(SyncOutcome::AssetInventory(_)) => {
                    text(cx, "Asset inventory read.", "资产状态已读取。").to_string()
                }
                Ok(SyncOutcome::FolderSaved) => {
                    text(cx, "Sync folders saved.", "同步文件夹已保存。").to_string()
                }
                Ok(SyncOutcome::Saved) => {
                    text(cx, "WebDAV settings saved.", "WebDAV 设置已保存。").to_string()
                }
                Ok(SyncOutcome::Connected) => {
                    text(cx, "Connection test succeeded.", "连接测试成功。").to_string()
                }
                Ok(SyncOutcome::Pushed) => {
                    text(cx, "Selected folder uploaded.", "已上传所选文件夹。").to_string()
                }
                Ok(SyncOutcome::Pulled { backup }) => {
                    let done = text(cx, "Selected folder downloaded.", "已下载所选文件夹。");
                    backup
                        .as_ref()
                        .map(|path| format!("{done} {}: {path}", text(cx, "Backup", "备份")))
                        .unwrap_or_else(|| done.to_string())
                }
                Err(error) => sync_error(cx, *error).to_string(),
            })
        };
        if let Some(status) = status {
            section = section.child(
                div()
                    .id("sync-operation-status")
                    .test_support()
                    .role(gpui_kit::Role::Status)
                    .aria_label(SharedString::from(status.clone()))
                    .child(status),
            );
        }
        section = section.child(
            div()
                .flex()
                .gap(px(theme::CONTROL_GAP))
                .child(semantics::button(
                    Button::new("sync-edit")
                        .label(text(cx, "Edit WebDAV settings", "编辑 WebDAV 设置"))
                        .on_click(cx.listener(|this, _, window, cx| this.edit(window, cx))),
                    locked || self.folder_editor.is_some() || self.snapshot.is_none(),
                    false,
                    cx,
                ))
                .child(semantics::button(
                    Button::new("sync-test")
                        .label(text(cx, "Test connection", "测试连接"))
                        .ghost()
                        .on_click(cx.listener(|this, _, _, cx| {
                            if let Some(revision) = this.revision() {
                                this.start(Operation::Test { revision }, cx);
                            }
                        })),
                    locked
                        || self.editor.is_some()
                        || self.folder_editor.is_some()
                        || !matches!(self.snapshot, Some(SyncSnapshot::Present(_))),
                    false,
                    cx,
                )),
        );
        if let Some(editor) = &self.editor {
            section = section.child(div().id("sync-editor").test_support().flex().flex_col().gap(px(theme::CONTROL_GAP))
                .child(text(cx, "Blank URL, username or base path keeps the saved value. Password handling is explicit. Saving does not contact the server.", "URL、用户名或基础路径留空会保留原值。密码需明确选择处理方式。保存不会连接服务器。"))
                .child(Input::new(&editor.url).aria_label(text(cx, "WebDAV URL", "WebDAV URL")).accessibility_id("sync-url"))
                .child(Input::new(&editor.username).aria_label(text(cx, "WebDAV username", "WebDAV 用户名")).accessibility_id("sync-username"))
                .child(Input::new(&editor.base).aria_label(text(cx, "Remote base path", "远程基础路径")).accessibility_id("sync-base"))
                .child(semantics::button(Button::new("sync-enabled").label(if editor.enabled { text(cx, "WebDAV enabled", "WebDAV 已启用") } else { text(cx, "WebDAV disabled", "WebDAV 已停用") })
                    .ghost().on_click(cx.listener(|this, _, _, cx| { if let Some(editor) = &mut this.editor { editor.enabled = !editor.enabled; } cx.notify(); })), locked, false, cx))
                .child(div().flex().gap(px(theme::CONTROL_GAP))
                    .children([(0, text(cx, "Keep password", "保留密码")), (1, text(cx, "Replace password", "替换密码")), (2, text(cx, "Clear password", "清除密码"))].into_iter().map(|(mode, label)| {
                        semantics::button(Button::new(("sync-password-mode", mode as usize)).label(label).ghost()
                            .on_click(cx.listener(move |this, _, _, cx| { if let Some(editor) = &mut this.editor { editor.password_mode = mode; } cx.notify(); })), locked, false, cx)
                    })))
                .child(text(cx, match editor.password_mode { 1 => "New password will replace the saved password.", 2 => "Saved password will be cleared. Disable WebDAV first.", _ => "Saved password will be kept." },
                    match editor.password_mode { 1 => "新密码将替换已保存的密码。", 2 => "将清除已保存的密码，请先停用 WebDAV。", _ => "将保留已保存的密码。" }))
                .child(Input::new(&editor.password).aria_label(text(cx, "New WebDAV password", "新 WebDAV 密码")).accessibility_id("sync-password"))
                .child(div().flex().gap(px(theme::CONTROL_GAP))
                    .child(semantics::button(Button::new("sync-save").label(text(cx, "Save settings", "保存设置")).on_click(cx.listener(|this, _, _, cx| this.save(cx))), locked, false, cx))
                    .child(semantics::button(Button::new("sync-edit-cancel").label(text(cx, "Cancel editing", "取消编辑")).ghost().on_click(cx.listener(|this, _, _, cx| { this.editor = None; cx.notify(); })), locked, false, cx))));
        }
        section = section.child(
            div().flex().gap(px(theme::CONTROL_GAP)).children(
                [
                    (
                        OperationKind::Push,
                        "sync-push",
                        text(cx, "Upload selected folder", "上传所选文件夹"),
                    ),
                    (
                        OperationKind::Pull,
                        "sync-pull",
                        text(cx, "Download selected folder", "下载所选文件夹"),
                    ),
                ]
                .into_iter()
                .map(|(kind, id, label)| {
                    semantics::button(
                        Button::new(id).label(label).ghost().on_click(cx.listener(
                            move |this, _, _, cx| {
                                this.confirmation = Some(kind);
                                cx.notify();
                            },
                        )),
                        locked
                            || self.editor.is_some()
                            || self.folder_editor.is_some()
                            || !self
                                .selected_folder()
                                .is_some_and(|folder| folder.enabled && folder.identity.is_some()),
                        false,
                        cx,
                    )
                }),
            ),
        );
        if let Some(kind) = self.confirmation {
            let warning = if kind == OperationKind::Push {
                text(
                    cx,
                    "Upload the selected folder as stored over HTTPS and overwrite matching remote content? Files can contain credentials. This ordinary folder transfer does not apply the encrypted asset envelope.",
                    "通过 HTTPS 上传所选文件夹的原始内容并覆盖对应远程内容？文件可能包含凭据。普通文件夹传输不会应用加密资产封装。",
                )
            } else {
                text(
                    cx,
                    "Replace the selected local folder with remote content? The sync service stages the download and preserves a backup before replacement.",
                    "用远程内容替换所选本地文件夹？同步服务会先暂存下载内容，并在替换前保留备份。",
                )
            };
            section = section.child(
                div()
                    .id("sync-confirmation")
                    .test_support()
                    .flex()
                    .flex_col()
                    .gap(px(theme::CONTROL_GAP))
                    .child(warning)
                    .child(
                        div()
                            .flex()
                            .gap(px(theme::CONTROL_GAP))
                            .child(semantics::button(
                                Button::new("sync-transfer-cancel")
                                    .label(text(cx, "Cancel", "取消"))
                                    .ghost()
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.confirmation = None;
                                        cx.notify();
                                    })),
                                false,
                                false,
                                cx,
                            ))
                            .child(semantics::button(
                                Button::new("sync-transfer-confirm")
                                    .label(text(cx, "Confirm transfer", "确认传输"))
                                    .on_click(
                                        cx.listener(move |this, _, _, cx| this.transfer(kind, cx)),
                                    ),
                                locked,
                                false,
                                cx,
                            )),
                    ),
            );
        }
        section
    }
}

fn sync_error(cx: &App, error: SyncError) -> &'static str {
    match error {
        SyncError::LegacyFolders => text(
            cx,
            error.message(),
            "旧格式文件夹条目只读，请通过 CCR 迁移为规范 folders 格式后再编辑。",
        ),
        SyncError::Invalid => text(cx, error.message(), "WebDAV 设置无效，或所选文件夹不可用。"),
        SyncError::Conflict => text(
            cx,
            error.message(),
            "同步设置已被其他客户端修改，请重新读取后重试。",
        ),
        SyncError::Busy => text(cx, error.message(), "已有同步操作排队或运行。"),
        SyncError::FixtureNetwork => text(cx, error.message(), "演示数据禁用远程同步。"),
        SyncError::Operation => text(
            cx,
            error.message(),
            "WebDAV 操作失败，请检查账户、权限和所选路径。",
        ),
        SyncError::Interrupted => text(cx, error.message(), "同步执行器停止，尚未确认结果。"),
        _ => service_error(cx, error.message()),
    }
}

#[cfg(test)]
mod tests;
