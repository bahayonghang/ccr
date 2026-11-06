use super::*;

pub(super) enum Confirmation {
    Clear {
        revision: String,
        session: Arc<ccr_sync::sync::clear::ClearSession>,
        generation: u64,
    },
    Batch {
        revision: String,
        upload: bool,
        generation: u64,
    },
}
impl SyncView {
    fn confirm_management(&mut self, cx: &mut Context<Self>) {
        if self.busy || self.operation.busy {
            return;
        }
        let Some(pending) = self.management_confirmation.take() else {
            return;
        };
        let (revision, generation, operation) = match pending {
            Confirmation::Clear {
                revision,
                session,
                generation,
            } => (
                revision.clone(),
                generation,
                Operation::Clear { revision, session },
            ),
            Confirmation::Batch {
                revision,
                upload,
                generation,
            } => (
                revision.clone(),
                generation,
                Operation::Batch { revision, upload },
            ),
        };
        if self.generation != generation || self.revision().as_ref() != Some(&revision) {
            self.message = Some(SyncError::Conflict.message());
            cx.notify();
            return;
        }
        self.start(operation, cx);
    }
    pub(super) fn render_management(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> gpui_kit::AnyElement {
        let locked = self.busy || self.operation.busy;
        let mut root = div()
            .flex()
            .flex_col()
            .flex_shrink_0()
            .gap(px(theme::CONTROL_GAP));
        if let Some(pending) = &self.management_confirmation {
            if self.management_focus {
                self.management_focus = false;
                window.focus(&self.focus, cx);
            }
            let message = match pending {
                Confirmation::Clear { .. } => text(
                    cx,
                    "Disconnect WebDAV? Clear the saved endpoint and credentials and remove the legacy connection file. Folder definitions and remote files stay unchanged. Local configuration backups can retain old credentials.",
                    "断开 WebDAV？将清除已保存的地址和凭据，并删除旧版连接配置。文件夹定义及远程文件保持不变。本地配置备份可能保留旧凭据。",
                ),
                Confirmation::Batch { upload: true, .. } => text(
                    cx,
                    "Upload every enabled folder and overwrite matching remote content? Files transfer as stored over HTTPS and can contain credentials; this does not use the encrypted asset envelope.",
                    "上传全部已启用文件夹并覆盖对应远程内容？文件通过 HTTPS 按原始内容传输，可能包含凭据，不使用加密资产封装。",
                ),
                Confirmation::Batch { upload: false, .. } => text(
                    cx,
                    "Replace every enabled local folder with remote content? Each download preserves a backup. Completed folders are not rolled back if another folder fails.",
                    "用远程内容替换全部已启用的本地文件夹？每项下载会保留备份。其他文件夹失败时，不会回滚已完成的文件夹。",
                ),
            };
            return root
                .id("sync-management-confirmation")
                .test_support()
                .child(message)
                .child(semantics::button(
                    Button::new("sync-management-cancel")
                        .label(text(cx, "Cancel", "取消"))
                        .ghost()
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.management_confirmation = None;
                            cx.notify();
                        })),
                    locked,
                    false,
                    cx,
                ))
                .child(semantics::button(
                    Button::new("sync-management-confirm")
                        .label(text(cx, "Confirm", "确认"))
                        .on_click(cx.listener(|this, _, _, cx| this.confirm_management(cx))),
                    locked,
                    false,
                    cx,
                ))
                .into_any_element();
        }
        let blocked = locked
            || self.editor.is_some()
            || self.folder_editor.is_some()
            || self.asset_editor.is_some()
            || self.confirmation.is_some()
            || self.folder_delete.is_some()
            || self.snapshot.is_none();
        root = root.child(semantics::button(
            Button::new("sync-disconnect")
                .label(text(cx, "Disconnect WebDAV", "断开 WebDAV"))
                .ghost()
                .on_click(cx.listener(|this, _, _, cx| {
                    if let Some(revision) = this.revision() {
                        this.start(Operation::PrepareClear { revision }, cx);
                    }
                })),
            blocked,
            false,
            cx,
        ));
        let enabled = matches!(&self.snapshot, Some(SyncSnapshot::Present(local)) if local.folders.iter().any(|f|f.enabled && f.identity.is_some()));
        for (upload, id, label) in [
            (
                true,
                "sync-push-all",
                text(cx, "Upload enabled folders", "上传已启用文件夹"),
            ),
            (
                false,
                "sync-pull-all",
                text(cx, "Download enabled folders", "下载已启用文件夹"),
            ),
        ] {
            root = root.child(semantics::button(
                Button::new(id).label(label).ghost().on_click(cx.listener(
                    move |this, _, _, cx| {
                        if this.busy || this.operation.busy {
                            return;
                        }
                        if let Some(revision) = this.revision() {
                            this.management_confirmation = Some(Confirmation::Batch {
                                revision,
                                upload,
                                generation: this.generation,
                            });
                            this.management_focus = true;
                            cx.notify();
                        }
                    },
                )),
                blocked || !enabled,
                false,
                cx,
            ));
        }
        if let Some(Ok(SyncOutcome::Batch(results))) = &self.operation.result {
            for item in results {
                let result = match &item.result {
                    Ok(backup) => backup
                        .as_ref()
                        .map(|path| format!("{}: {path}", text(cx, "Backup", "备份")))
                        .unwrap_or_else(|| text(cx, "Completed", "已完成").into()),
                    Err(error) => sync_error(cx, *error).into(),
                };
                root = root.child(format!("{} · {result}", item.name));
            }
        }
        root.into_any_element()
    }
}
