use super::*;
use gpui_kit::component::FocusTrapElement;
use gpui_kit::{Pixels, Point};

pub(super) enum Confirmation {
    Clear {
        revision: String,
        session: Arc<ccr_sync::sync::clear::ClearSession>,
        generation: u64,
    },
    Batch(FolderTransfer),
}

pub(super) struct FolderTransfer {
    revision: String,
    generation: u64,
    upload: bool,
    selected: Option<(FolderTarget, SyncFolderRow)>,
    count: usize,
    return_offset: Point<Pixels>,
}

impl SyncView {
    fn confirm_management(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.busy || self.operation.busy {
            return;
        }
        if let Some(transfer) = self.confirmation.take().or_else(|| {
            match self
                .management_confirmation
                .take_if(|pending| matches!(pending, Confirmation::Batch(_)))
            {
                Some(Confirmation::Batch(transfer)) => Some(transfer),
                _ => None,
            }
        }) {
            let current = transfer
                .selected
                .as_ref()
                .is_none_or(|(target, _)| self.target_current(target));
            if current
                && self.generation == transfer.generation
                && self.revision().as_ref() == Some(&transfer.revision)
            {
                self.reset_folder_panel();
                self.asset_detail = None;
                self.folder_focus.scroll.scroll_to_top_of_item(2);
                window.focus(&self.focus, cx);
                self.start(
                    Operation::TransferFolders {
                        revision: transfer.revision,
                        selected: transfer.selected.map(|(target, _)| target.index),
                        upload: transfer.upload,
                    },
                    cx,
                );
            } else {
                self.message = Some(SyncError::Conflict.message());
                cx.notify();
            }
            return;
        }
        let Some(Confirmation::Clear {
            revision,
            session,
            generation,
        }) = self.management_confirmation.take()
        else {
            return;
        };
        if self.generation != generation || self.revision().as_ref() != Some(&revision) {
            self.message = Some(SyncError::Conflict.message());
            cx.notify();
            return;
        }
        self.start(Operation::Clear { revision, session }, cx);
    }

    pub(super) fn cancel_management(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let anchor = if let Some(transfer) = self.confirmation.take() {
            self.folder_focus.scroll.set_offset(transfer.return_offset);
            &self.selected_transfer_anchors[usize::from(transfer.upload)]
        } else {
            match self.management_confirmation.take() {
                Some(Confirmation::Clear { .. }) => &self.disconnect_anchor,
                Some(Confirmation::Batch(transfer)) => {
                    self.folder_focus.scroll.set_offset(transfer.return_offset);
                    &self.batch_anchors[usize::from(transfer.upload)]
                }
                None => return,
            }
        };
        self.restore_action_focus(anchor.clone(), window, cx);
        cx.notify();
    }

    pub(super) fn render_management_confirmation(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<gpui_kit::AnyElement> {
        let single = self.confirmation.is_some();
        let transfer = self
            .confirmation
            .as_ref()
            .or(match &self.management_confirmation {
                Some(Confirmation::Batch(transfer)) => Some(transfer),
                _ => None,
            });
        let mut panel = div()
            .id(if single {
                "sync-confirmation"
            } else {
                "sync-management-confirmation"
            })
            .test_support()
            .flex()
            .flex_col()
            .min_w_0()
            .gap(px(theme::CONTROL_GAP));
        if let Some(transfer) = transfer {
            panel = panel.child(self.render_folder_transfer_review(transfer, cx));
        } else if matches!(
            self.management_confirmation,
            Some(Confirmation::Clear { .. })
        ) {
            panel = panel
                .child(section_heading(text(cx, "Disconnect WebDAV", "断开 WebDAV")))
                .child(text(
                    cx,
                    "Disconnect WebDAV? Clear the saved endpoint and credentials and remove the legacy connection file. Folder definitions and remote files stay unchanged. Local configuration backups can retain old credentials.",
                    "断开 WebDAV？将清除已保存的地址和凭据，并删除旧版连接配置。文件夹定义及远程文件保持不变。本地配置备份可能保留旧凭据。",
                ));
        } else {
            return None;
        }
        if std::mem::take(&mut self.management_focus) {
            self.restore_action_focus(self.management_anchor.clone(), window, cx);
        }
        let locked = self.busy || self.operation.busy;
        Some(
            panel
                .child(
                    sync_actions()
                        .child(semantics::ghost_button(
                            Button::new(if single {
                                "sync-transfer-cancel"
                            } else {
                                "sync-management-cancel"
                            })
                            .label(text(cx, "Cancel", "取消"))
                            .on_click(
                                cx.listener(|this, _, window, cx| {
                                    this.cancel_management(window, cx)
                                }),
                            ),
                            locked,
                            false,
                            cx,
                        ))
                        .child(semantics::button(
                            Button::new(if single {
                                "sync-transfer-confirm"
                            } else {
                                "sync-management-confirm"
                            })
                            .label(if transfer.is_some() {
                                text(cx, "Confirm transfer", "确认传输")
                            } else {
                                text(cx, "Confirm", "确认")
                            })
                            .on_click(cx.listener(
                                |this, _, window, cx| this.confirm_management(window, cx),
                            )),
                            locked,
                            false,
                            cx,
                        ))
                        .focus_trap("sync-management-trap", &self.management_anchor),
                )
                .into_any_element(),
        )
    }

    fn render_folder_transfer_review(
        &self,
        transfer: &FolderTransfer,
        cx: &App,
    ) -> impl IntoElement {
        let direction = match (transfer.upload, transfer.selected.is_some()) {
            (true, true) => text(cx, "Upload selected folder", "上传所选文件夹"),
            (false, true) => text(cx, "Download selected folder", "下载所选文件夹"),
            (true, false) => text(cx, "Upload all enabled folders", "上传全部已启用文件夹"),
            (false, false) => text(cx, "Download all enabled folders", "下载全部已启用文件夹"),
        };
        let warning = match (transfer.upload, self.is_ssh()) {
            (true, _) => text(
                cx,
                "Upload sends files as stored over HTTPS, including any credentials in the files. Matching WebDAV content is overwritten.",
                "通过 HTTPS 上传原始内容，包括文件中可能含有的凭据，并覆盖 WebDAV 上对应的内容。",
            ),
            (false, false) => text(
                cx,
                "Download transfers the original contents over HTTPS and replaces the destination. An existing destination is backed up before replacement.",
                "通过 HTTPS 下载原始内容并替换目标。已有目标会在替换前备份。",
            ),
            (false, true) => text(
                cx,
                "Download transfers the original contents over HTTPS and writes files on the SSH host. Replaced files are backed up on that host. Publication may stop after writing some files.",
                "通过 HTTPS 下载原始内容并写入 SSH 主机。被替换的文件会在该主机备份，写入部分文件后可能中止。",
            ),
        };
        let mut panel = div()
            .flex()
            .flex_col()
            .gap(px(theme::CONTROL_GAP))
            .child(section_heading(direction))
            .child(self.environment_label(cx))
            .child(sync_status("sync-folder-transfer-warning", warning));
        if let Some((_, folder)) = &transfer.selected {
            panel = panel
                .child(sync_status(
                    "sync-folder-transfer-targets",
                    folder.name.clone(),
                ))
                .child(folders::folder_path(
                    "sync-confirm-current-path",
                    text(cx, "Current environment path", "当前执行环境路径"),
                    &folder.local_path,
                    cx,
                ))
                .child(folders::folder_path(
                    "sync-confirm-webdav-path",
                    text(cx, "WebDAV path", "WebDAV 路径"),
                    &folder.remote_path,
                    cx,
                ));
        } else {
            let scope = if is_chinese(cx) {
                format!("全部 {} 个已启用文件夹；跳过已停用文件夹。", transfer.count)
            } else {
                format!(
                    "All enabled folders ({}). Disabled folders are skipped.",
                    transfer.count
                )
            };
            panel = panel
                .child(sync_status("sync-folder-transfer-targets", scope))
                .child(text(
                    cx,
                    "Completed folders are kept if another folder fails.",
                    "其他文件夹失败时，已完成的文件夹会保留。",
                ));
        }
        panel
    }

    pub(super) fn management_blocked(&self) -> bool {
        self.busy || self.operation.busy || self.has_environment_draft() || self.snapshot.is_none()
    }

    pub(super) fn render_disconnect(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("sync-disconnect-action")
            .track_focus(&self.disconnect_anchor)
            .tab_stop(false)
            .child(semantics::ghost_button(
                Button::new("sync-disconnect")
                    .label(text(cx, "Disconnect WebDAV", "断开 WebDAV"))
                    .on_click(cx.listener(|this, _, _, cx| {
                        if let Some(revision) = this.revision() {
                            this.start(Operation::PrepareClear { revision }, cx);
                        }
                    })),
                self.management_blocked() || self.is_ssh(),
                self.operation.busy && self.operation.kind == Some(OperationKind::PrepareClear),
                cx,
            ))
    }

    fn enabled_folder_count(&self) -> usize {
        match &self.snapshot {
            Some(SyncSnapshot::Present(local)) => local
                .folders
                .iter()
                .filter(|folder| folder.enabled && folder.identity.is_some())
                .count(),
            _ => 0,
        }
    }

    fn prepare_folder_transfer(
        &mut self,
        upload: bool,
        selected: Option<&FolderTarget>,
        cx: &mut Context<Self>,
    ) {
        if self.management_blocked() {
            return;
        }
        let Some(revision) = self.revision() else {
            return;
        };
        let count = self.enabled_folder_count();
        let selected = if let Some(target) = selected {
            let Some(folder) = self
                .selected_folder()
                .filter(|folder| folder.enabled && self.target_current(target))
            else {
                return;
            };
            Some((target.clone(), folder.clone()))
        } else if count > 0 {
            None
        } else {
            return;
        };
        // The revision pins both the selected index and the entire enabled set.
        let transfer = FolderTransfer {
            revision,
            generation: self.generation,
            upload,
            selected,
            count,
            return_offset: self.folder_focus.scroll.offset(),
        };
        if transfer.selected.is_some() {
            self.confirmation = Some(transfer);
        } else {
            self.management_confirmation = Some(Confirmation::Batch(transfer));
        }
        self.management_focus = true;
        self.folder_focus.scroll.scroll_to_top_of_item(2);
        cx.notify();
    }

    pub(super) fn render_folder_transfers(
        &self,
        selected: Option<&FolderTarget>,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let blocked = self.management_blocked()
            || if selected.is_some() {
                !self.selected_folder().is_some_and(|folder| folder.enabled)
            } else {
                self.enabled_folder_count() == 0
            };
        let anchors = if selected.is_some() {
            &self.selected_transfer_anchors
        } else {
            &self.batch_anchors
        };
        let mut actions = sync_actions();
        for upload in [true, false] {
            let (id, label) = match (upload, selected.is_some()) {
                (true, true) => (
                    "sync-push",
                    text(cx, "Upload selected folder", "上传所选文件夹"),
                ),
                (false, true) => (
                    "sync-pull",
                    text(cx, "Download selected folder", "下载所选文件夹"),
                ),
                (true, false) => (
                    "sync-push-all",
                    text(cx, "Upload enabled folders", "上传已启用文件夹"),
                ),
                (false, false) => (
                    "sync-pull-all",
                    text(cx, "Download enabled folders", "下载已启用文件夹"),
                ),
            };
            let target = selected.cloned();
            actions = actions.child(
                div()
                    .id((
                        "sync-folder-transfer-action",
                        usize::from(upload) + 2 * usize::from(selected.is_some()),
                    ))
                    .track_focus(&anchors[usize::from(upload)])
                    .tab_stop(false)
                    .child(semantics::ghost_button(
                        Button::new(id).label(label).on_click(cx.listener(
                            move |this, _, _, cx| {
                                this.prepare_folder_transfer(upload, target.as_ref(), cx);
                            },
                        )),
                        blocked,
                        false,
                        cx,
                    )),
            );
        }
        actions
    }
}
