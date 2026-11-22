//! Canonical sync settings and retained WebDAV operations.
mod assets;
mod folders;
mod layout;
mod management;
mod webdav;
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
use layout::{reading_panel, section_heading, sync_actions, sync_status};
use std::sync::Arc;

struct Editor {
    revision: String,
    url: Entity<InputState>,
    username: Entity<InputState>,
    password: Entity<InputState>,
    base: Entity<InputState>,
    enabled: bool,
    password_mode: PasswordMode,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PasswordMode {
    Keep,
    Replace,
    Clear,
}

pub struct SyncView {
    services: Services,
    environment: EnvironmentId,
    environment_name: Option<String>,
    snapshot: Option<SyncSnapshot>,
    generation: u64,
    gate: ResponseGate,
    task: Option<Task<()>>,
    busy: bool,
    message: Option<&'static str>,
    focus: FocusHandle,
    edit_anchor: FocusHandle,
    disconnect_anchor: FocusHandle,
    management_anchor: FocusHandle,
    editor: Option<Editor>,
    asset_editor: Option<assets::AssetEditor>,
    folder_editor: Option<FolderEditor>,
    folder_delete: Option<FolderTarget>,
    selected: Option<usize>,
    confirmation: Option<OperationKind>,
    management_confirmation: Option<management::Confirmation>,
    management_focus: bool,
    preparing_disconnect: bool,
    operation: Arc<OperationState>,
    observation: Option<Task<()>>,
}

impl SyncView {
    fn is_ssh(&self) -> bool {
        matches!(self.environment, EnvironmentId::Ssh(_))
    }
    pub fn has_environment_draft(&self) -> bool {
        self.editor.is_some()
            || self.asset_editor.is_some()
            || self.folder_editor.is_some()
            || self.confirmation.is_some()
            || self.folder_delete.is_some()
            || self.management_confirmation.is_some()
    }
    pub fn new(services: Services, _window: &mut Window, cx: &mut Context<Self>) -> Self {
        let environment = services.environment();
        Self {
            operation: services.sync_operation_state().visible_in(&environment),
            services,
            environment,
            environment_name: None,
            snapshot: None,
            generation: 0,
            gate: ResponseGate::default(),
            task: None,
            busy: false,
            message: None,
            focus: cx.focus_handle().tab_stop(false),
            edit_anchor: cx.focus_handle().tab_stop(false),
            disconnect_anchor: cx.focus_handle().tab_stop(false),
            management_anchor: cx.focus_handle().tab_stop(false),
            editor: None,
            asset_editor: None,
            folder_editor: None,
            folder_delete: None,
            selected: None,
            confirmation: None,
            management_confirmation: None,
            management_focus: false,
            preparing_disconnect: false,
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
        self.snapshot = None;
        self.selected = None;
        self.message = None;
        self.observation = None;
        self.editor = None;
        self.asset_editor = None;
        self.confirmation = None;
        self.management_confirmation = None;
        self.management_focus = false;
        self.preparing_disconnect = false;
        self.folder_editor = None;
        self.folder_delete = None;
        cx.notify();
    }

    pub fn open(&mut self, environment_name: String, cx: &mut Context<Self>) {
        self.refresh(cx);
        self.environment_name = Some(environment_name);
    }

    pub fn refresh(&mut self, cx: &mut Context<Self>) {
        let environment = self.services.environment();
        if self.environment != environment {
            self.suspend(cx);
            self.environment = environment;
            self.environment_name = None;
        }
        self.observe(cx);
        if self.busy {
            return;
        }
        self.confirmation = None;
        self.management_confirmation = None;
        self.management_focus = false;
        self.preparing_disconnect = false;
        self.asset_editor = None;
        self.folder_delete = None;
        self.folder_editor = None;
        self.generation += 1;
        let key = RequestKey {
            environment: self.environment.clone(),
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
        self.operation = self
            .services
            .sync_operation_state()
            .visible_in(&self.environment);
        if self.observation.is_some() {
            return;
        }
        let mut updates = self.services.observe_sync();
        self.operation = updates
            .borrow_and_update()
            .clone()
            .visible_in(&self.environment);
        self.observation = Some(cx.spawn(async move |weak, cx| {
            while updates.changed().await.is_ok() {
                let state = updates.borrow_and_update().clone();
                if weak
                    .update(cx, |this, cx| {
                        if this.environment != this.services.environment() {
                            return;
                        }
                        let state = state.visible_in(&this.environment);
                        let saved = matches!(
                            state.result,
                            Some(Ok(SyncOutcome::Saved
                                | SyncOutcome::FolderSaved
                                | SyncOutcome::Cleared))
                                | Some(Err(SyncError::LegacyRemaining))
                        );
                        if let Some(Ok(SyncOutcome::ClearPrepared(session))) = &state.result
                            && this.preparing_disconnect
                            && let Some(revision) = this.revision()
                        {
                            this.management_confirmation = Some(management::Confirmation::Clear {
                                revision,
                                session: session.clone(),
                                generation: this.generation,
                            });
                            this.management_focus = true;
                        }
                        if !state.busy {
                            this.preparing_disconnect = false;
                        }
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
        if self.environment != self.services.environment() {
            return None;
        }
        match self.snapshot.as_ref()? {
            SyncSnapshot::Absent => Some(String::new()),
            SyncSnapshot::Present(local) => Some(local.revision.clone()),
        }
    }

    fn start(&mut self, operation: Operation, cx: &mut Context<Self>) {
        if self.busy || self.operation.busy || self.environment != self.services.environment() {
            return;
        }
        self.generation += 1;
        let key = RequestKey {
            environment: self.environment.clone(),
            platform: Platform::Claude,
            generation: self.generation,
            request_id: self.generation,
        };
        let prepare_disconnect = matches!(operation, Operation::PrepareClear { .. });
        let error = self.services.start_sync(key, operation).err();
        self.preparing_disconnect = prepare_disconnect && error.is_none();
        self.message = error.map(SyncError::message);
        self.confirmation = None;
        self.management_confirmation = None;
        self.folder_delete = None;
        self.observe(cx);
        cx.notify();
    }

    fn restore_action_focus(
        &self,
        anchor: FocusHandle,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        window.focus(&anchor, cx);
        let (view, generation) = (cx.weak_entity(), self.generation);
        window.on_next_frame(move |window, cx| {
            let _ = view.update(cx, |this, cx| {
                if this.generation == generation && anchor.is_focused(window) {
                    window.focus_next(cx);
                }
            });
        });
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

impl SyncView {
    fn render_operation_status(&self, cx: &App) -> Option<layout::Panel> {
        if self.operation.kind == Some(OperationKind::Test)
            || matches!(
                self.operation.result,
                Some(Ok(SyncOutcome::ClearPrepared(_)))
            )
        {
            return None;
        }
        let status = if self.operation.busy {
            Some(text(cx, "Sync operation is running. It continues when you leave this page; closing waits for it to finish. Cancellation is unavailable during transfer.", "同步操作正在运行。离开页面后会继续，关闭应用将等待完成。传输期间不提供取消。").to_string())
        } else {
            self.operation.result.as_ref().map(|result| match result {
                Ok(SyncOutcome::ClearPrepared(_)) => text(
                    cx,
                    "Disconnection is ready for confirmation.",
                    "断开连接已准备就绪，等待确认。",
                )
                .to_string(),
                Ok(SyncOutcome::Cleared) => text(
                    cx,
                    "WebDAV account disconnected. Folder definitions were kept.",
                    "WebDAV 账户已断开，文件夹定义已保留。",
                )
                .to_string(),
                Ok(SyncOutcome::Batch(results)) => format!(
                    "{}: {}/{}",
                    text(cx, "Folders completed", "文件夹已完成"),
                    results.iter().filter(|r| r.result.is_ok()).count(),
                    results.len()
                ),
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
        status.map(|status| sync_status("sync-operation-status", status))
    }

    fn render_selected_folder_operations(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let locked = self.busy || self.operation.busy || self.management_confirmation.is_some();
        let mut section = div().flex().flex_col().gap(px(theme::CONTROL_GAP));
        section = section.child(
            sync_actions().children(
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
                        sync_actions()
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
        SyncError::LegacyRemaining => text(
            cx,
            error.message(),
            "规范配置中的 WebDAV 凭据已清除，但旧版配置清理未完成。请重新读取配置后再次断开连接。",
        ),
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
