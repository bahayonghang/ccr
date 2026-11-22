//! Canonical sync settings and retained WebDAV operations.
mod asset_details;
mod asset_transfer;
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
            SettingsPatch, SyncError, SyncFolderRow, SyncOutcome, SyncSnapshot, SyncUrl,
        },
    },
    shell::semantics,
    theme,
};
use ccr_app::runtime::{Completion, EnvironmentId, Platform, RequestKey, ResponseGate, TaskEvent};
use folders::{FolderEditor, FolderFocus, FolderTarget};
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
    focus: FocusHandle,
    saved: bool,
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
    batch_anchors: [FocusHandle; 2],
    selected_transfer_anchors: [FocusHandle; 2],
    management_anchor: FocusHandle,
    editor: Option<Editor>,
    asset_editor: Option<asset_transfer::AssetEditor>,
    asset_detail: Option<usize>,
    asset_detail_focus: Option<(u64, asset_details::AssetDetailFocus)>,
    asset_row_anchors: [FocusHandle; ccr_sync::sync::assets::SYNC_ASSETS.len()],
    asset_back_anchor: FocusHandle,
    asset_transfer_anchor: FocusHandle,
    all_assets_transfer_anchor: FocusHandle,
    asset_editor_anchor: FocusHandle,
    asset_result_anchor: FocusHandle,
    folder_focus: FolderFocus,
    folder_editor: Option<FolderEditor>,
    folder_delete: Option<FolderTarget>,
    folder_delete_focus: bool,
    selected: Option<usize>,
    confirmation: Option<management::FolderTransfer>,
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
            || self.folder_decision_open()
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
            batch_anchors: std::array::from_fn(|_| cx.focus_handle().tab_stop(false)),
            selected_transfer_anchors: std::array::from_fn(|_| cx.focus_handle().tab_stop(false)),
            management_anchor: cx.focus_handle().tab_stop(false),
            editor: None,
            asset_editor: None,
            asset_detail: None,
            asset_detail_focus: None,
            asset_row_anchors: std::array::from_fn(|_| cx.focus_handle().tab_stop(false)),
            asset_back_anchor: cx.focus_handle().tab_stop(false),
            asset_transfer_anchor: cx.focus_handle().tab_stop(false),
            all_assets_transfer_anchor: cx.focus_handle().tab_stop(false),
            asset_editor_anchor: cx.focus_handle().tab_stop(false),
            asset_result_anchor: cx.focus_handle().tab_stop(false),
            folder_focus: FolderFocus::new(cx),
            folder_editor: None,
            folder_delete: None,
            folder_delete_focus: false,
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

    pub fn suspend(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        drop(self.discard_asset_editor(window, cx));
        self.reset(cx);
    }

    fn reset(&mut self, cx: &mut Context<Self>) {
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
        self.asset_detail = None;
        self.asset_detail_focus = None;
        self.confirmation = None;
        self.management_confirmation = None;
        self.management_focus = false;
        self.preparing_disconnect = false;
        self.reset_folder_panel();
        self.folder_focus.suspend();
        cx.notify();
    }

    pub fn open(&mut self, environment_name: String, cx: &mut Context<Self>) {
        self.refresh(cx);
        self.environment_name = Some(environment_name);
    }

    pub fn refresh(&mut self, cx: &mut Context<Self>) {
        let environment = self.services.environment();
        if self.environment != environment {
            // Shell parks the current view with its window before changing environments.
            self.reset(cx);
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
        self.asset_detail = None;
        self.asset_detail_focus = None;
        self.reset_folder_panel();
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
                                        let folder_count = match snapshot {
                                            SyncSnapshot::Present(local) => local.folders.len(),
                                            SyncSnapshot::Absent => 0,
                                        };
                                        this.folder_focus.set_rows(folder_count, cx);
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
                        this.record_folder_completion();
                        if saved {
                            if let Some(editor) = &mut this.editor
                                && matches!(this.operation.result, Some(Ok(SyncOutcome::Saved)))
                            {
                                // Keep the form's focus scope through the reload.
                                editor.saved = true;
                            } else {
                                this.editor = None;
                            }
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
        self.confirmation = None;
        self.management_confirmation = None;
        self.folder_delete = None;
        self.folder_delete_focus = false;
        self.observe(cx);
        self.message = error
            .filter(|error| {
                !matches!(self.operation.result.as_ref(), Some(Err(stored)) if stored == error)
            })
            .map(SyncError::message);
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
}

impl SyncView {
    fn render_operation_status(&self, cx: &App) -> Option<layout::Panel> {
        if matches!(
            self.operation.kind,
            Some(OperationKind::Test | OperationKind::AssetInventory)
        ) || matches!(
            self.operation.result,
            Some(Ok(SyncOutcome::ClearPrepared(_)))
        ) {
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
                Ok(SyncOutcome::Batch(results)) => Self::folder_result_summary(results, cx),
                Ok(SyncOutcome::Assets(results)) => {
                    let completed = results.iter().filter(|asset| asset.result.is_ok()).count();
                    let label = if completed == results.len() {
                        text(cx, "Asset transfer completed", "资产传输完成")
                    } else if completed > 0
                        || results
                            .iter()
                            .any(|asset| asset.result == Err(SyncError::RemotePartial))
                    {
                        text(cx, "Asset transfer partially completed", "资产传输部分完成")
                    } else {
                        text(cx, "Asset transfer failed", "资产传输失败")
                    };
                    format!("{label} · {completed}/{}", results.len())
                }
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
        status.map(|status| {
            let panel = sync_status("sync-operation-status", status)
                .track_focus(&self.asset_result_anchor)
                .tab_stop(false)
                .flex()
                .flex_col()
                .gap(px(theme::CONTROL_GAP));
            match &self.operation.result {
                Some(Ok(SyncOutcome::Batch(results))) => {
                    panel.children(self.render_folder_results(results, cx))
                }
                Some(Ok(SyncOutcome::Assets(results))) => {
                    panel.children(results.iter().map(|asset| {
                        asset_transfer::render_asset_result(&asset.id, &asset.result, cx)
                    }))
                }
                _ => panel,
            }
        })
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
        SyncError::AssetPassphrase => text(cx, error.message(), "敏感资产需要本次操作口令。"),
        SyncError::AssetLegacy => text(cx, error.message(), "导入明文 v1 需要本次明确授权。"),
        SyncError::AssetDecrypt => text(
            cx,
            error.message(),
            "资产解密失败，请检查口令和加密封装。当前环境中的数据已保留。",
        ),
        SyncError::AssetConflict => {
            text(cx, error.message(), "两端都有副本。请核对覆盖后果再重试。")
        }
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
