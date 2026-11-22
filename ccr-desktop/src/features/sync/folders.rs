//! Folder edits retain their original identity and revision and patch only visible fields.
use super::*;
use gpui_kit::component::{ActiveTheme, FocusTrapElement};
use gpui_kit::{AnyElement, Focusable, Pixels, Point, ScrollHandle};

#[derive(Clone, PartialEq, Eq)]
pub(super) struct FolderTarget {
    revision: String,
    identity: String,
    generation: u64,
    pub(super) index: usize,
}

pub(super) struct FolderEditor {
    revision: String,
    target: Option<FolderTarget>,
    name: Entity<InputState>,
    local: Entity<InputState>,
    remote: Entity<InputState>,
}

enum FolderSaveFocus {
    Add,
    Row(String),
}

pub(super) struct FolderFocus {
    pub(super) scroll: ScrollHandle,
    /// The list offset, while the folder detail or add form is open.
    list_offset: Option<Point<Pixels>>,
    rows: Vec<FocusHandle>,
    add: FocusHandle,
    detail: FocusHandle,
    edit: FocusHandle,
    delete: FocusHandle,
    delete_confirmation: FocusHandle,
    form: FocusHandle,
    save_pending: Option<FolderSaveFocus>,
    restore_after_save: Option<FolderSaveFocus>,
}

impl FolderFocus {
    pub(super) fn new(cx: &mut Context<SyncView>) -> Self {
        Self {
            scroll: ScrollHandle::new(),
            list_offset: None,
            rows: Vec::new(),
            add: cx.focus_handle().tab_stop(false),
            detail: cx.focus_handle().tab_stop(false),
            edit: cx.focus_handle().tab_stop(false),
            delete: cx.focus_handle().tab_stop(false),
            delete_confirmation: cx.focus_handle().tab_stop(false),
            form: cx.focus_handle().tab_stop(false),
            save_pending: None,
            restore_after_save: None,
        }
    }

    pub(super) fn set_rows(&mut self, count: usize, cx: &mut Context<SyncView>) {
        self.rows
            .resize_with(count, || cx.focus_handle().tab_stop(false));
    }

    pub(super) fn suspend(&mut self) {
        self.list_offset = None;
        self.save_pending = None;
        self.restore_after_save = None;
    }
}

impl SyncView {
    pub(super) fn folder_decision_open(&self) -> bool {
        self.folder_editor.is_some()
            || self.folder_delete.is_some()
            || self.folder_focus.save_pending.is_some()
            || self.confirmation.is_some()
            || matches!(
                self.management_confirmation,
                Some(management::Confirmation::Batch(_))
            )
    }

    pub(super) fn selected_folder(&self) -> Option<&SyncFolderRow> {
        let SyncSnapshot::Present(local) = self.snapshot.as_ref()? else {
            return None;
        };
        local.folders.get(self.selected?)
    }

    pub(super) fn folder_target(&self) -> Option<FolderTarget> {
        Some(FolderTarget {
            revision: self.revision()?,
            identity: self.selected_folder()?.identity.clone()?,
            generation: self.generation,
            index: self.selected?,
        })
    }

    pub(super) fn target_current(&self, target: &FolderTarget) -> bool {
        self.folder_target().as_ref() == Some(target)
    }

    fn show_folder(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        if self.has_environment_draft() {
            return;
        }
        self.selected = Some(index);
        self.generation += 1;
        self.folder_focus.list_offset = None;
        self.open_folder_panel();
        self.restore_action_focus(self.folder_focus.detail.clone(), window, cx);
        cx.notify();
    }

    fn open_folder_panel(&mut self) {
        self.asset_detail = None;
        self.folder_focus
            .list_offset
            .get_or_insert_with(|| self.folder_focus.scroll.offset());
        self.folder_focus.scroll.scroll_to_top_of_item(2);
    }

    fn close_folder_panel(
        &mut self,
        anchor: FocusHandle,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.reset_folder_panel();
        self.restore_action_focus(anchor, window, cx);
        cx.notify();
    }

    pub(super) fn reset_folder_panel(&mut self) {
        if let Some(offset) = self.folder_focus.list_offset.take() {
            self.folder_focus.scroll.set_offset(offset);
        }
        self.folder_editor = None;
        self.folder_delete = None;
    }

    pub(super) fn dismiss_folder_panel(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self.operation.busy && self.folder_focus.save_pending.is_some() {
            return false;
        }
        if let Some(editor) = self.folder_editor.take() {
            if editor.target.is_some() {
                self.restore_action_focus(self.folder_focus.edit.clone(), window, cx);
                cx.notify();
            } else {
                self.close_folder_panel(self.folder_focus.add.clone(), window, cx);
            }
        } else if self.folder_delete.take().is_some() {
            self.restore_action_focus(self.folder_focus.delete.clone(), window, cx);
            cx.notify();
        } else if self.folder_focus.list_offset.is_some() {
            self.close_selected_folder(window, cx);
        } else {
            return false;
        }
        true
    }

    fn close_selected_folder(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let anchor = self
            .selected
            .and_then(|index| self.folder_focus.rows.get(index))
            .unwrap_or(&self.folder_focus.add)
            .clone();
        self.close_folder_panel(anchor, window, cx);
    }

    fn edit_folder(&mut self, existing: bool, window: &mut Window, cx: &mut Context<Self>) {
        if self.management_blocked() || !matches!(self.snapshot, Some(SyncSnapshot::Present(_))) {
            return;
        }
        let Some(revision) = self.revision() else {
            return;
        };
        let target = if existing {
            let Some(target) = self.folder_target() else {
                return;
            };
            Some(target)
        } else {
            None
        };
        let name = cx.new(|cx| InputState::new(window, cx));
        window.focus(&name.read(cx).focus_handle(cx), cx);
        self.open_folder_panel();
        self.folder_editor = Some(FolderEditor {
            revision,
            target,
            name,
            local: cx.new(|cx| InputState::new(window, cx)),
            remote: cx.new(|cx| InputState::new(window, cx)),
        });
        cx.notify();
    }

    fn save_folder(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(editor) = &self.folder_editor else {
            return;
        };
        if self.revision().as_ref() != Some(&editor.revision)
            || editor
                .target
                .as_ref()
                .is_some_and(|target| !self.target_current(target))
        {
            return;
        }
        let value = |field: &Entity<InputState>| {
            let value = field.read(cx).value().to_string();
            (!value.trim().is_empty()).then(|| value.trim().to_string())
        };
        let patch = FolderPatch {
            name: value(&editor.name),
            local_path: value(&editor.local),
            remote_path: value(&editor.remote),
        };
        let change = if let Some(target) = &editor.target {
            FolderChange::Edit {
                identity: target.identity.clone(),
                patch,
            }
        } else {
            FolderChange::Add(patch)
        };
        self.start_folder_change(editor.revision.clone(), change, window, cx);
    }

    fn delete_folder(
        &mut self,
        target: &FolderTarget,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.folder_delete.as_ref() == Some(target) && self.target_current(target) {
            self.start_folder_change(
                target.revision.clone(),
                FolderChange::Delete {
                    identity: target.identity.clone(),
                },
                window,
                cx,
            );
        }
    }

    fn start_folder_change(
        &mut self,
        revision: String,
        change: FolderChange,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let restore = match &change {
            FolderChange::Edit { identity, patch } => FolderSaveFocus::Row(
                patch
                    .name
                    .as_ref()
                    .map(|name| ccr_core::core::content_version_token(name.as_bytes()))
                    .unwrap_or_else(|| identity.clone()),
            ),
            FolderChange::Enabled { identity, .. } => FolderSaveFocus::Row(identity.clone()),
            FolderChange::Add(_) | FolderChange::Delete { .. } => FolderSaveFocus::Add,
        };
        let generation = self.generation;
        self.start(Operation::Folder { revision, change }, cx);
        if let Some(target) = self
            .folder_editor
            .as_mut()
            .and_then(|editor| editor.target.as_mut())
            && generation == target.generation
        {
            // Saving advances the request generation. Keep the same revision and
            // identity so a failed validation can be corrected in this form.
            target.generation = self.generation;
        }
        if self.operation.busy && self.operation.kind == Some(OperationKind::Folder) {
            self.folder_focus.save_pending = Some(restore);
            window.focus(&self.folder_focus.form, cx);
        }
    }

    pub(super) fn record_folder_completion(&mut self) {
        if !self.operation.busy
            && let Some(restore) = self.folder_focus.save_pending.take()
            && matches!(self.operation.result, Some(Ok(SyncOutcome::FolderSaved)))
        {
            self.folder_focus.restore_after_save = Some(restore);
        }
    }

    pub(super) fn finish_folder_save(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.busy
            && let Some(restore) = self.folder_focus.restore_after_save.take()
            && self.folder_focus.form.is_focused(window)
        {
            let anchor = match &self.snapshot {
                Some(SyncSnapshot::Present(local)) => match restore {
                    FolderSaveFocus::Add => &self.folder_focus.add,
                    FolderSaveFocus::Row(identity) => local
                        .folders
                        .iter()
                        .position(|folder| folder.identity.as_ref() == Some(&identity))
                        .and_then(|index| self.folder_focus.rows.get(index))
                        .unwrap_or(&self.folder_focus.add),
                },
                _ => &self.focus,
            };
            self.restore_action_focus(anchor.clone(), window, cx);
        }
    }

    pub(super) fn render_folders(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let mut panel = reading_panel("sync-folders", cx);
        let heading = section_heading(text(cx, "Sync folders", "同步文件夹"));
        let Some(SyncSnapshot::Present(local)) = &self.snapshot else {
            panel = panel.child(heading);
            if matches!(&self.snapshot, Some(SyncSnapshot::Absent)) {
                panel = panel.child(div().text_color(cx.theme().muted_foreground).child(text(
                    cx,
                    "Save WebDAV settings before adding sync folders.",
                    "请先保存 WebDAV 设置，再添加同步文件夹。",
                )));
            }
            return panel;
        };
        let blocked = self.management_blocked();
        let legacy = local.folders.iter().any(|folder| folder.identity.is_none());
        panel = panel.child(
            sync_actions()
                .items_center()
                .justify_between()
                .child(heading)
                .child(
                    folder_action_anchor("sync-folder-add-action", &self.folder_focus.add).child(
                        semantics::ghost_button(
                            Button::new("sync-folder-add")
                                .label(text(cx, "Add folder", "添加文件夹"))
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.edit_folder(false, window, cx)
                                })),
                            blocked || legacy,
                            false,
                            cx,
                        ),
                    ),
                ),
        );
        if local.folders.is_empty() {
            return panel.child(div().text_color(cx.theme().muted_foreground).child(text(
                cx,
                "No sync folders configured.",
                "尚未配置同步文件夹。",
            )));
        }
        if legacy {
            panel = panel.child(sync_error(cx, SyncError::LegacyFolders));
        }
        let selection_blocked = self.has_environment_draft();
        panel
            .children(
                local
                    .folders
                    .iter()
                    .zip(&self.folder_focus.rows)
                    .enumerate()
                    .map(|(index, (folder, focus))| {
                        let state = folder_state(cx, folder.enabled);
                        let label = format!("{} · {state}", folder.name);
                        let expanded =
                            self.folder_focus.list_offset.is_some() && self.selected == Some(index);
                        let button = semantics::disclosure(
                            Button::new(("sync-select", index))
                                .label(text(cx, "Details", "详情"))
                                .accessibility_label(format!(
                                    "{} · {}",
                                    text(cx, "Folder details", "文件夹详情"),
                                    folder.name
                                ))
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    this.show_folder(index, window, cx)
                                })),
                            expanded,
                        );
                        div()
                            .id(SharedString::from(format!("sync-folder-{index}")))
                            .test_support()
                            .flex()
                            .items_center()
                            .gap(px(theme::CONTROL_GAP))
                            .min_w_0()
                            .border_t_1()
                            .border_color(cx.theme().border)
                            .pt(px(theme::CONTROL_GAP))
                            .aria_label(label.clone())
                            .child(
                                div()
                                    .min_w_0()
                                    .flex_1()
                                    .flex()
                                    .flex_col()
                                    .gap(px(2.))
                                    .child(div().truncate().child(label))
                                    .child(
                                        div()
                                            .truncate()
                                            .text_color(cx.theme().muted_foreground)
                                            .text_size(theme::text_size(theme::META_SIZE))
                                            .child(folder.local_path.clone()),
                                    ),
                            )
                            .child(
                                div()
                                    .id(("sync-folder-row-action", index))
                                    .track_focus(focus)
                                    .tab_stop(false)
                                    .child(semantics::ghost_button(
                                        button,
                                        selection_blocked,
                                        false,
                                        cx,
                                    )),
                            )
                    }),
            )
            .child(self.render_folder_transfers(None, cx))
    }

    pub(super) fn render_folder_details(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        self.folder_focus.list_offset?;
        let mut panel = div()
            .id("sync-folder-detail")
            .test_support()
            .track_focus(&self.folder_focus.form)
            .tab_stop(false)
            .flex()
            .flex_col()
            .min_w_0()
            .gap(px(theme::CONTROL_GAP));
        if self.operation.kind == Some(OperationKind::Folder) {
            panel = panel.children(self.render_operation_status(cx));
        }
        if let Some(editor) = &self.folder_editor {
            return Some(
                panel
                    .child(self.render_folder_editor(editor, cx))
                    .focus_trap("sync-folder-form-trap", &self.folder_focus.form)
                    .into_any_element(),
            );
        }
        let folder = self.selected_folder()?;
        if let Some(target) = &self.folder_delete {
            return Some(
                panel
                    .child(self.render_folder_delete(folder, target, cx))
                    .into_any_element(),
            );
        }
        panel = panel
            .child(
                folder_action_anchor("sync-folder-detail-action", &self.folder_focus.detail).child(
                    semantics::ghost_button(
                        Button::new("sync-folder-back")
                            .label(text(cx, "Back to sync list", "返回同步列表"))
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.close_selected_folder(window, cx)
                            })),
                        self.folder_focus.save_pending.is_some(),
                        false,
                        cx,
                    ),
                ),
            )
            .child(
                div()
                    .min_w_0()
                    .text_size(theme::text_size(theme::SECTION_TITLE_SIZE))
                    .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                    .child(folder.name.clone()),
            )
            .child(sync_status(
                "sync-folder-state",
                folder_state(cx, folder.enabled),
            ))
            .child(folder_path(
                "sync-folder-local-path",
                text(cx, "Current environment path", "当前执行环境路径"),
                &folder.local_path,
                cx,
            ))
            .child(folder_path(
                "sync-folder-remote-path",
                text(cx, "WebDAV path", "WebDAV 路径"),
                &folder.remote_path,
                cx,
            ));
        if let Some(target) = self.folder_target() {
            panel = panel
                .child(self.render_selected_folder_actions(folder, &target, cx))
                .child(self.render_folder_transfers(Some(&target), cx));
        }
        Some(panel.into_any_element())
    }

    fn render_selected_folder_actions(
        &self,
        folder: &SyncFolderRow,
        target: &FolderTarget,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let blocked = self.management_blocked();
        let enabled = folder.enabled;
        let toggle = target.clone();
        let delete = target.clone();
        sync_actions()
            .child(
                folder_action_anchor("sync-folder-edit-action", &self.folder_focus.edit).child(
                    semantics::ghost_button(
                        Button::new("sync-folder-edit")
                            .label(text(cx, "Edit", "编辑"))
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.edit_folder(true, window, cx)
                            })),
                        blocked,
                        false,
                        cx,
                    ),
                ),
            )
            .child(semantics::ghost_button(
                Button::new("sync-folder-enabled")
                    .label(if enabled {
                        text(cx, "Disable", "停用")
                    } else {
                        text(cx, "Enable", "启用")
                    })
                    .on_click(cx.listener(move |this, _, window, cx| {
                        if !this.management_blocked() && this.target_current(&toggle) {
                            this.start_folder_change(
                                toggle.revision.clone(),
                                FolderChange::Enabled {
                                    identity: toggle.identity.clone(),
                                    enabled: !enabled,
                                },
                                window,
                                cx,
                            );
                        }
                    })),
                blocked,
                false,
                cx,
            ))
            .child(
                folder_action_anchor("sync-folder-delete-action", &self.folder_focus.delete).child(
                    semantics::ghost_button(
                        Button::new("sync-folder-delete")
                            .label(text(cx, "Remove", "移除"))
                            .on_click(cx.listener(move |this, _, window, cx| {
                                if !this.management_blocked() && this.target_current(&delete) {
                                    this.folder_delete = Some(delete.clone());
                                    this.restore_action_focus(
                                        this.folder_focus.delete_confirmation.clone(),
                                        window,
                                        cx,
                                    );
                                    cx.notify();
                                }
                            })),
                        blocked,
                        false,
                        cx,
                    ),
                ),
            )
    }

    fn render_folder_editor(
        &self,
        editor: &FolderEditor,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let locked = self.busy || self.operation.busy;
        let existing = editor.target.is_some();
        let current = self.selected_folder().filter(|_| existing);
        div()
            .id("sync-folder-editor")
            .test_support()
            .flex()
            .flex_col()
            .min_w_0()
            .gap(px(theme::CONTROL_GAP))
            .child(section_heading(if existing {
                text(cx, "Edit folder", "编辑文件夹")
            } else {
                text(cx, "Add folder", "添加文件夹")
            }))
            .children(existing.then(|| {
                text(
                    cx,
                    "Fill in the fields you want to change.",
                    "填写需要修改的字段。",
                )
            }))
            .child(folder_field(
                "sync-folder-name",
                text(cx, "Folder name", "文件夹名称"),
                &editor.name,
                current.map(|folder| folder.name.as_str()),
            ))
            .child(folder_field(
                "sync-folder-local",
                text(cx, "Current environment path", "当前执行环境路径"),
                &editor.local,
                current.map(|folder| folder.local_path.as_str()),
            ))
            .child(folder_field(
                "sync-folder-remote",
                text(cx, "WebDAV path", "WebDAV 路径"),
                &editor.remote,
                current.map(|folder| folder.remote_path.as_str()),
            ))
            .child(
                sync_actions()
                    .child(semantics::ghost_button(
                        Button::new("sync-folder-cancel")
                            .label(text(cx, "Cancel", "取消"))
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.dismiss_folder_panel(window, cx);
                            })),
                        locked,
                        false,
                        cx,
                    ))
                    .child(semantics::button(
                        Button::new("sync-folder-save")
                            .label(text(cx, "Save folder", "保存文件夹"))
                            .on_click(
                                cx.listener(|this, _, window, cx| this.save_folder(window, cx)),
                            ),
                        locked,
                        false,
                        cx,
                    )),
            )
    }

    fn render_folder_delete(
        &self,
        folder: &SyncFolderRow,
        target: &FolderTarget,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let target = target.clone();
        let locked = self.busy || self.operation.busy;
        div()
            .id("sync-folder-delete-confirmation")
            .test_support()
            .flex()
            .flex_col()
            .gap(px(theme::CONTROL_GAP))
            .child(section_heading(text(cx, "Remove sync folder", "移除同步文件夹")))
            .child(folder.name.clone())
            .child(text(cx, "Remove this definition from sync settings? Files in the current environment and on WebDAV will be kept.", "从同步设置中移除此定义？当前执行环境和 WebDAV 上的文件将保留。"))
            .child(sync_actions()
                .child(semantics::ghost_button(
                    Button::new("sync-folder-delete-cancel")
                        .label(text(cx, "Cancel", "取消"))
                        .on_click(cx.listener(|this, _, window, cx| { this.dismiss_folder_panel(window, cx); })),
                    locked, false, cx,
                ))
                .child(semantics::button(
                    Button::new("sync-folder-delete-confirm")
                        .label(text(cx, "Confirm removal", "确认移除"))
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.delete_folder(&target, window, cx);
                        })),
                    locked, false, cx,
                ))
                .focus_trap("sync-folder-delete-trap", &self.folder_focus.delete_confirmation))
    }
}

impl SyncView {
    pub(super) fn folder_result_summary(
        results: &[crate::runtime::sync::FolderResult],
        cx: &App,
    ) -> String {
        let completed = results
            .iter()
            .filter(|folder| folder.result.is_ok())
            .count();
        let label = if completed == results.len() {
            text(cx, "Folders completed", "文件夹已完成")
        } else if completed > 0
            || results
                .iter()
                .any(|folder| folder.result == Err(SyncError::RemotePartial))
        {
            text(
                cx,
                "Folder transfer partially completed",
                "文件夹传输部分完成",
            )
        } else {
            text(cx, "Folder transfer failed", "文件夹传输失败")
        };
        format!("{label} · {completed}/{}", results.len())
    }

    pub(super) fn render_folder_results(
        &self,
        results: &[crate::runtime::sync::FolderResult],
        cx: &App,
    ) -> impl IntoIterator<Item = impl IntoElement> {
        let ssh_download = self.is_ssh()
            && matches!(
                self.operation.kind,
                Some(OperationKind::Pull | OperationKind::BatchPull)
            );
        results
            .iter()
            .enumerate()
            .map(|(index, folder)| {
                let detail = match &folder.result {
                    Ok(Some(backup)) => format!(
                        "{} · {}: {backup}",
                        text(cx, "Completed", "已完成"),
                        text(cx, "Backup", "备份")
                    ),
                    Ok(None) if ssh_download => text(
                        cx,
                        "Completed. Backups of replaced files are on the SSH host.",
                        "已完成。被替换文件的备份保存在 SSH 主机。",
                    )
                    .into(),
                    Ok(None) => text(cx, "Completed", "已完成").into(),
                    Err(SyncError::RemoteDatabase) => text(
                        cx,
                        "Refused: this folder contains a database or database sidecar.",
                        "已拒绝：此文件夹含数据库或其附属文件。",
                    )
                    .into(),
                    Err(error) => sync_error(cx, *error).into(),
                };
                let label = format!("{} · {detail}", folder.name);
                div()
                    .id(("sync-folder-result", index))
                    .test_support()
                    .min_w_0()
                    .whitespace_normal()
                    .aria_label(label.clone())
                    .text_color(if folder.result.is_ok() {
                        cx.theme().foreground
                    } else {
                        cx.theme().danger
                    })
                    .child(label)
            })
            .collect::<Vec<_>>()
    }
}
fn folder_state(cx: &App, enabled: bool) -> &'static str {
    if enabled {
        text(cx, "enabled", "已启用")
    } else {
        text(cx, "disabled", "已停用")
    }
}

fn folder_action_anchor(
    id: &'static str,
    focus: &FocusHandle,
) -> gpui_kit::Stateful<gpui_kit::Div> {
    div().id(id).track_focus(focus).tab_stop(false)
}

pub(super) fn folder_path(
    id: &'static str,
    label: &'static str,
    value: &str,
    cx: &App,
) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .min_w_0()
        .gap(px(2.))
        .child(div().text_color(cx.theme().muted_foreground).child(label))
        .child(
            div()
                .id(id)
                .test_support()
                .min_w_0()
                .whitespace_normal()
                .aria_label(value.to_owned())
                .child(value.to_owned()),
        )
}

fn folder_field(
    id: &'static str,
    label: &'static str,
    field: &Entity<InputState>,
    saved: Option<&str>,
) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .min_w_0()
        .gap(px(2.))
        .child(label)
        .child(Input::new(field).accessibility_id(id).aria_label(label))
        .children(saved.map(|value| div().min_w_0().whitespace_normal().child(value.to_owned())))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use ccr_app::runtime::RuntimeOwner;
    use gpui_kit::test::{TestAppContextExt, TestWindowExt};
    use gpui_kit::{TestAppContext, WindowOptions};
    use std::time::Duration;

    #[gpui_kit::test]
    async fn folder_editor_and_delete_confirmation_keep_the_original_target(
        cx: &mut TestAppContext,
    ) {
        cx.executor().allow_parking();
        let root = tempfile::tempdir().unwrap();
        let owner = RuntimeOwner::new().unwrap();
        let mut config = ccr_sync::SyncFoldersConfig::default();
        config.webdav.url = "https://dav.example.test/".into();
        config.webdav.username = "fixture".into();
        for name in ["first", "second"] {
            config.folders.push(
                ccr_sync::SyncFolder::builder()
                    .name(name)
                    .local_path(root.path().join(name).to_string_lossy().into_owned())
                    .remote_path(format!("/{name}"))
                    .build()
                    .unwrap(),
            );
        }
        let path = root.path().join("sync_folders.toml");
        std::fs::write(&path, toml::to_string(&config).unwrap()).unwrap();
        let services = Services::new(
            owner.handle(),
            crate::runtime::ConfigSource::Fixture {
                profiles: root.path().join("profiles.toml"),
                registry: root.path().join("registry.toml"),
                delay: Duration::ZERO,
            },
        );
        let retained = services.clone();
        let (handle, view) = cx.update(|cx| {
            gpui_kit::init(cx);
            cx.set_global(crate::locales::Language::English);
            gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
                cx.new(|cx| SyncView::new(services, window, cx))
            })
            .unwrap()
        });
        view.update(cx, |this, cx| this.refresh(cx));
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
            view.read(cx).snapshot.is_some()
        })
        .await;
        let stale = cx
            .update_window(handle, |_, window, cx| {
                window.render_frame(cx);
                view.update(cx, |this, cx| {
                    this.selected = Some(0);
                    this.edit_folder(true, window, cx);
                    let target = this.folder_target().unwrap();
                    this.folder_delete = Some(target.clone());
                    this.selected = Some(1);
                    this.delete_folder(&target, window, cx);
                    this.save_folder(window, cx);
                    assert!(!retained.sync_operation_state().busy);
                    target
                })
            })
            .unwrap();
        let before = std::fs::read(&path).unwrap();
        cx.update_window(handle, |_, window, cx| {
            view.update(cx, |this, cx| {
                this.refresh(cx);
                this.delete_folder(&stale, window, cx);
                assert!(this.folder_delete.is_none());
                assert!(this.folder_editor.is_none());
            });
        })
        .unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), before);
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| !view.read(cx).busy)
            .await;
        cx.update_window(handle, |_, window, cx| {
            view.update(cx, |this, cx| {
                this.selected = Some(0);
                this.edit_folder(true, window, cx);
                let name = this.folder_editor.as_ref().unwrap().name.clone();
                name.update(cx, |state, cx| state.set_value("renamed", window, cx));
                this.save_folder(window, cx);
            });
        })
        .unwrap();
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
            !retained.sync_operation_state().busy
                && view.read(cx).folder_editor.is_none()
                && !view.read(cx).busy
        })
        .await;
        let saved: ccr_sync::SyncFoldersConfig =
            toml::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        assert_eq!(saved.folders[0].name, "renamed");
        assert_eq!(saved.folders[1].name, "second");
        owner.shutdown();
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod controls {
    use super::*;
    use ccr_app::runtime::RuntimeOwner;
    use gpui_kit::test::{TestAppContextExt, TestWindowExt};
    use gpui_kit::{Bounds, TestAppContext, WindowBounds, WindowOptions, size};
    use std::time::Duration;

    fn open_workbench(
        cx: &mut TestAppContext,
        owner: &RuntimeOwner,
        root: &std::path::Path,
    ) -> (gpui_kit::AnyWindowHandle, Entity<SyncView>) {
        let services = Services::new(
            owner.handle(),
            crate::runtime::ConfigSource::Fixture {
                profiles: root.join("profiles.toml"),
                registry: root.join("registry.toml"),
                delay: Duration::ZERO,
            },
        );
        let (handle, view) = cx.update(|cx| {
            gpui_kit::init(cx);
            cx.set_global(crate::locales::Language::English);
            gpui_kit::open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                        None,
                        size(px(960.), px(1200.)),
                        cx,
                    ))),
                    ..Default::default()
                },
                cx,
                |window, cx| cx.new(|cx| SyncView::new(services, window, cx)),
            )
            .unwrap()
        });
        view.update(cx, |view, cx| view.refresh(cx));
        (handle, view)
    }

    #[gpui_kit::test]
    async fn folder_workbench_selected_confirmation_shows_its_target_and_returns_focus(
        cx: &mut TestAppContext,
    ) {
        cx.executor().allow_parking();
        let root = tempfile::tempdir().unwrap();
        let owner = RuntimeOwner::new().unwrap();
        let local_path = format!("~/fixture/{}", "long-path-segment/".repeat(15));
        let mut config = ccr_sync::SyncFoldersConfig::default();
        config.webdav.url = "https://fixture.invalid/".into();
        config.webdav.username = "fixture".into();
        config.folders.push(
            ccr_sync::SyncFolder::builder()
                .name("notes-fixture")
                .local_path(local_path.clone())
                .remote_path("/fixture/notes/")
                .build()
                .unwrap(),
        );
        std::fs::write(
            root.path().join("sync_folders.toml"),
            toml::to_string(&config).unwrap(),
        )
        .unwrap();
        let (handle, _) = open_workbench(cx, &owner, root.path());
        cx.wait_for(handle, Duration::from_secs(10), |window, _| {
            window.try_find("sync-folder-0").is_some()
        })
        .await;
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            assert!(window.try_find("sync-folder-detail").is_none());
            window.click(("sync-select", 0usize), cx);
            window.render_frame(cx);
            assert!(window.find("sync-folder-detail").visible());
            assert_eq!(
                window.find("sync-folder-local-path").label(),
                Some(local_path.as_str())
            );
            assert!(window.try_find("sync-asset-list").is_none());
            window.click("sync-push", cx);
            window.render_frame(cx);
            window.simulate_next_frame(cx);
            window.render_frame(cx);
            assert_eq!(window.find("sync-transfer-cancel").focused(), Some(true));
            assert!(
                window
                    .find("sync-folder-transfer-targets")
                    .label()
                    .unwrap()
                    .contains("notes-fixture")
            );
            assert!(
                window
                    .find("sync-folder-transfer-warning")
                    .label()
                    .unwrap()
                    .contains("HTTPS")
            );
            window.press("tab", cx);
            window.render_frame(cx);
            assert_eq!(window.find("sync-transfer-confirm").focused(), Some(true));
            window.press("tab", cx);
            window.render_frame(cx);
            assert_eq!(window.find("sync-transfer-cancel").focused(), Some(true));
            window.press("escape", cx);
            window.simulate_next_frame(cx);
            window.render_frame(cx);
            assert_eq!(window.find("sync-push").focused(), Some(true));
            window.press("escape", cx);
            window.simulate_next_frame(cx);
            window.render_frame(cx);
            assert_eq!(window.find(("sync-select", 0usize)).focused(), Some(true));
            window.click(("sync-select", 0usize), cx);
            window.render_frame(cx);
            window.click("sync-pull", cx);
            window.render_frame(cx);
            let config_file = root.path().join("sync_folders.toml");
            let newer = std::fs::read_to_string(&config_file).unwrap() + "\n# changed elsewhere\n";
            std::fs::write(&config_file, newer).unwrap();
            window.click("sync-transfer-confirm", cx);
        })
        .unwrap();
        cx.wait_for(handle, Duration::from_secs(10), |window, _| {
            window
                .try_find("sync-operation-status")
                .and_then(|node| {
                    node.label()
                        .map(|label| label.contains("changed elsewhere"))
                })
                .unwrap_or(false)
        })
        .await;
        cx.update_window(handle, |_, window, _| {
            window.remove_window();
        })
        .unwrap();
        owner.shutdown();
    }

    #[gpui_kit::test]
    async fn folder_workbench_crud_uses_rendered_controls(cx: &mut TestAppContext) {
        cx.executor().allow_parking();
        let root = tempfile::tempdir().unwrap();
        let owner = RuntimeOwner::new().unwrap();
        let config_file = root.path().join("sync_folders.toml");
        let mut config = ccr_sync::SyncFoldersConfig::default();
        config.webdav.url = "https://fixture.invalid/".into();
        config.webdav.username = "fixture".into();
        config.webdav.password = "synthetic-password".into();
        std::fs::write(&config_file, toml::to_string(&config).unwrap()).unwrap();
        let (handle, _) = open_workbench(cx, &owner, root.path());
        cx.wait_for(handle, Duration::from_secs(10), |window, _| {
            window.try_find("sync-folder-add").is_some()
        })
        .await;
        let local = root.path().join("notes.txt");
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            window.click("sync-folder-add", cx);
            window.render_frame(cx);
            window.input("notes-fixture", cx);
            window.press("tab", cx);
            window.input(local.to_str().unwrap(), cx);
            window.press("tab", cx);
            window.input("/fixture/notes.txt", cx);
            window.click("sync-folder-save", cx);
        })
        .unwrap();
        for (expected, action) in [
            ("notes-fixture · enabled", "sync-folder-edit"),
            ("renamed · enabled", "sync-folder-enabled"),
            ("renamed · disabled", "sync-folder-delete"),
        ] {
            cx.wait_for(handle, Duration::from_secs(10), |window, _| {
                window
                    .try_find("sync-folder-0")
                    .is_some_and(|node| node.label() == Some(expected))
            })
            .await;
            cx.update_window(handle, |_, window, cx| {
                window.render_frame(cx);
                window.click(("sync-select", 0usize), cx);
                window.render_frame(cx);
                assert_eq!(
                    window.find("sync-folder-local-path").label(),
                    local.to_str()
                );
                window.click(action, cx);
                window.render_frame(cx);
                match action {
                    "sync-folder-edit" => {
                        window.input("renamed", cx);
                        window.click("sync-folder-save", cx);
                    }
                    "sync-folder-delete" => {
                        window.simulate_next_frame(cx);
                        window.render_frame(cx);
                        assert_eq!(
                            window.find("sync-folder-delete-cancel").focused(),
                            Some(true)
                        );
                        window.press("escape", cx);
                        window.simulate_next_frame(cx);
                        window.render_frame(cx);
                        assert_eq!(window.find("sync-folder-delete").focused(), Some(true));
                        window.click(action, cx);
                        window.render_frame(cx);
                        window.click("sync-folder-delete-confirm", cx);
                    }
                    _ => {}
                }
            })
            .unwrap();
        }
        cx.wait_for(handle, Duration::from_secs(10), |window, _| {
            window.try_find("sync-folder-0").is_none()
                && window.try_find("sync-folder-add").is_some()
        })
        .await;
        let stored: ccr_sync::SyncFoldersConfig =
            toml::from_str(&std::fs::read_to_string(config_file).unwrap()).unwrap();
        assert!(stored.folders.is_empty());
        assert_eq!(stored.webdav.password, "synthetic-password");
        cx.update_window(handle, |_, window, _| window.remove_window())
            .unwrap();
        owner.shutdown();
    }
}
