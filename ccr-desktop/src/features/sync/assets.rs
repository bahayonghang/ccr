use super::*;
use crate::runtime::sync::{AssetDirection, AssetRequest};
use ccr_sync::sync::assets::SYNC_ASSETS;

pub(super) struct AssetEditor {
    revision: String,
    id: Option<String>,
    pub(super) passphrase: Entity<InputState>,
    force: bool,
    import_v1: bool,
    confirmation: Option<AssetDirection>,
}
impl SyncView {
    fn edit_asset(&mut self, id: Option<String>, window: &mut Window, cx: &mut Context<Self>) {
        if self.busy || self.operation.busy {
            return;
        }
        let Some(revision) = self.revision() else {
            return;
        };
        self.editor = None;
        self.folder_editor = None;
        self.folder_delete = None;
        self.confirmation = None;
        self.asset_editor = Some(AssetEditor {
            revision,
            id,
            passphrase: cx.new(|cx| InputState::new(window, cx).masked(true)),
            force: false,
            import_v1: false,
            confirmation: None,
        });
        cx.notify();
    }
    fn send_asset(&mut self, cx: &mut Context<Self>) {
        if self.busy || self.operation.busy {
            return;
        }
        let Some(editor) = self.asset_editor.take() else {
            return;
        };
        let Some(direction) = editor.confirmation else {
            self.asset_editor = Some(editor);
            return;
        };
        let value = editor.passphrase.read(cx).value().to_string();
        self.start(
            Operation::Assets(AssetRequest {
                revision: editor.revision,
                id: editor.id,
                direction,
                force: editor.force,
                passphrase: (!value.is_empty()).then(|| value.into()),
                import_plaintext_v1: editor.import_v1,
            }),
            cx,
        );
    }
    pub(super) fn render_assets(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let locked = self.busy
            || self.operation.busy
            || !matches!(self.snapshot, Some(SyncSnapshot::Present(_)));
        let mut section=div().id("sync-assets").test_support().flex().flex_col().flex_shrink_0().gap(px(theme::CONTROL_GAP))
            .child(text(cx,"Configuration assets","配置资产"))
            .child(text(cx,"Sensitive assets use the v2 encrypted envelope. The operation passphrase is never saved. Instruction files transfer without encryption. Automatic sync is not enabled.","敏感资产使用 v2 加密封装，操作口令不会保存。指令文件不加密传输。不会启用自动同步。"));
        for (index, asset) in SYNC_ASSETS.iter().enumerate() {
            let id = asset.id.to_string();
            section = section.child(
                div()
                    .flex()
                    .gap(px(theme::CONTROL_GAP))
                    .child(format!("{} · {}", asset.name, asset.local_path))
                    .child(semantics::button(
                        Button::new(("sync-asset", index))
                            .label(text(cx, "Choose asset", "选择资产"))
                            .ghost()
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.edit_asset(Some(id.clone()), window, cx)
                            })),
                        locked,
                        false,
                        cx,
                    )),
            );
        }
        section = section.child(
            div()
                .flex()
                .gap(px(theme::CONTROL_GAP))
                .child(semantics::button(
                    Button::new("sync-assets-all")
                        .label(text(cx, "All assets", "全部资产"))
                        .on_click(
                            cx.listener(|this, _, window, cx| this.edit_asset(None, window, cx)),
                        ),
                    locked,
                    false,
                    cx,
                ))
                .children(
                    [
                        (
                            false,
                            "sync-assets-local",
                            text(cx, "Inspect local assets", "查看本地资产"),
                        ),
                        (
                            true,
                            "sync-assets-probe",
                            text(cx, "Check remote assets", "检查远程资产"),
                        ),
                    ]
                    .into_iter()
                    .map(|(probe, id, label)| {
                        semantics::button(
                            Button::new(id).label(label).ghost().on_click(cx.listener(
                                move |this, _, _, cx| {
                                    if let Some(revision) = this.revision() {
                                        this.start(
                                            Operation::AssetInventory { revision, probe },
                                            cx,
                                        );
                                    }
                                },
                            )),
                            locked || self.asset_editor.is_some(),
                            false,
                            cx,
                        )
                    }),
                ),
        );
        if let Some(Ok(SyncOutcome::AssetInventory(rows))) = &self.operation.result {
            for row in rows {
                section = section.child(format!(
                    "{} · {} · {} · {}",
                    row.id,
                    row.local_path,
                    if row.local_exists {
                        text(cx, "Local present", "本地存在")
                    } else {
                        text(cx, "Local missing", "本地缺失")
                    },
                    match row.remote_exists {
                        Some(true) => text(cx, "Remote present", "远程存在"),
                        Some(false) => text(cx, "Remote missing", "远程缺失"),
                        None => text(cx, "Remote unknown", "远程未确认"),
                    }
                ));
            }
        }
        if let Some(Ok(SyncOutcome::Assets(results))) = &self.operation.result {
            for item in results {
                section = section.child(match &item.result {
                    Ok(backup) => format!(
                        "{} · {}{}",
                        item.id,
                        text(cx, "Completed", "已完成"),
                        backup
                            .as_ref()
                            .map(|p| format!(" · {}: {p}", text(cx, "Backup", "备份")))
                            .unwrap_or_default()
                    ),
                    Err(error) => format!("{} · {}", item.id, sync_error(cx, *error)),
                });
            }
        }
        if let Some(editor) = &self.asset_editor {
            section=section.child(div().id("sync-asset-editor").test_support().flex().flex_col().gap(px(theme::CONTROL_GAP))
                .child(editor.id.clone().unwrap_or_else(||text(cx,"All assets","全部资产").into()))
                .child(Input::new(&editor.passphrase).aria_label(text(cx,"Asset operation passphrase","资产操作口令")).accessibility_id("sync-asset-passphrase"))
                .child(semantics::button(Button::new("sync-asset-force").label(if editor.force{text(cx,"Replacement allowed","允许覆盖")}else{text(cx,"Replacement blocked","禁止覆盖")}).ghost().on_click(cx.listener(|this,_,_,cx|{if let Some(e)=&mut this.asset_editor{e.force = !e.force;e.confirmation=None;}cx.notify();})),locked,false,cx))
                .child(semantics::button(Button::new("sync-asset-v1").label(if editor.import_v1{text(cx,"Plaintext v1 import allowed","允许导入明文 v1")}else{text(cx,"Plaintext v1 import blocked","禁止导入明文 v1")}).ghost().on_click(cx.listener(|this,_,_,cx|{if let Some(e)=&mut this.asset_editor{e.import_v1 = !e.import_v1;e.confirmation=None;}cx.notify();})),locked,false,cx))
                .child(text(cx,"Replacement permits upload overwrite or local replacement with backup. Sync prefers local when both copies exist. Plaintext v1 import only reads old data; it does not re-encrypt the remote copy.","允许覆盖后，上传可替换远程数据，下载会备份再替换本地数据。双向同步在两端都存在时优先本地。明文 v1 导入只读取旧数据，不会重新加密远程副本。"))
                .child(div().flex().gap(px(theme::CONTROL_GAP)).children([(AssetDirection::Push,"sync-asset-push",text(cx,"Upload asset","上传资产")),(AssetDirection::Pull,"sync-asset-pull",text(cx,"Download asset","下载资产")),(AssetDirection::Sync,"sync-asset-sync",text(cx,"Sync asset","同步资产"))].into_iter().map(|(direction,id,label)|semantics::button(Button::new(id).label(label).on_click(cx.listener(move|this,_,_,cx|{if let Some(e)=&mut this.asset_editor{e.confirmation=Some(direction);}cx.notify();})),locked,false,cx))))
                .child(semantics::button(Button::new("sync-asset-discard").label(text(cx,"Cancel","取消")).ghost().on_click(cx.listener(|this,_,_,cx|{this.asset_editor=None;cx.notify();})),locked,false,cx)));
            if let Some(direction) = editor.confirmation {
                section=section.child(div().id("sync-asset-confirmation").test_support().flex().flex_col().gap(px(theme::CONTROL_GAP))
                    .child(match direction{AssetDirection::Push=>text(cx,"Upload the selected configuration assets to the saved WebDAV account?","将所选配置资产上传到已保存的 WebDAV 账户？"),AssetDirection::Pull=>text(cx,"Download the selected configuration assets and apply them locally? Existing local data will be backed up when replacement is allowed.","下载所选配置资产并应用到本地？允许覆盖时，会备份已有本地数据。"),AssetDirection::Sync=>text(cx,"Synchronize selected assets using the displayed replacement and legacy import choices?","按上方覆盖和旧数据导入选项同步所选资产？")})
                    .child(semantics::button(Button::new("sync-asset-confirm").label(text(cx,"Confirm transfer","确认传输")).on_click(cx.listener(|this,_,_,cx|this.send_asset(cx))),locked,false,cx)));
            }
        }
        section
    }
}
