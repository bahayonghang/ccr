use super::*;
use crate::runtime::sync::{AssetDirection, AssetRequest, AssetRow};
use ccr_sync::sync::assets::{SYNC_ASSETS, SyncAssetDefinition, SyncAssetGroup};
use gpui_kit::component::button::ButtonRounded;
use gpui_kit::component::{ActiveTheme, Icon, IconName, Sizable};
use gpui_kit::{Focusable, FontWeight, relative};

pub(super) struct AssetEditor {
    revision: String,
    id: Option<String>,
    pub(super) passphrase: Entity<InputState>,
    force: bool,
    import_v1: bool,
    confirmation: Option<AssetDirection>,
}
impl SyncView {
    pub(super) fn edit_asset(
        &mut self,
        id: Option<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.asset_transfer_locked() || self.asset_editor.is_some() {
            return;
        }
        let Some(revision) = self.revision() else {
            return;
        };
        self.reset_folder_panel();
        if id.is_none() {
            self.asset_detail = None;
        }
        let passphrase = cx.new(|cx| InputState::new(window, cx).masked(true));
        window.focus(&passphrase.read(cx).focus_handle(cx), cx);
        self.asset_editor = Some(AssetEditor {
            revision,
            id,
            passphrase,
            force: false,
            import_v1: false,
            confirmation: None,
        });
        cx.notify();
    }
    pub(super) fn cancel_asset_edit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(editor) = self.asset_editor.take() else {
            return;
        };
        let anchor = if editor.id.is_some() {
            &self.asset_transfer_anchor
        } else {
            &self.all_assets_transfer_anchor
        };
        self.restore_action_focus(anchor.clone(), window, cx);
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
        self.close_asset_details(cx);
    }

    pub(super) fn asset_draft_blocked(&self) -> bool {
        self.editor.is_some()
            || self.folder_decision_open()
            || self.management_confirmation.is_some()
    }

    fn asset_actions_locked(&self) -> bool {
        self.busy
            || self.operation.busy
            || self.asset_draft_blocked()
            || self.environment != self.services.environment()
    }

    pub(super) fn asset_transfer_locked(&self) -> bool {
        self.asset_actions_locked() || !matches!(self.snapshot, Some(SyncSnapshot::Present(_)))
    }

    fn probe_revision(&self) -> Option<&str> {
        let Some(SyncSnapshot::Present(local)) = &self.snapshot else {
            return None;
        };
        (local.webdav_enabled
            && local.url != SyncUrl::Missing
            && local.username_present
            && local.password_present)
            .then_some(local.revision.as_str())
    }

    fn inventory_scope(&self) -> Option<bool> {
        if self.environment != self.services.environment() {
            return None;
        }
        self.operation
            .asset_inventory(&self.environment, self.probe_revision())
    }

    fn inventory_rows(&self) -> Option<&[AssetRow]> {
        self.inventory_scope()?;
        match self.operation.result.as_ref()? {
            Ok(SyncOutcome::AssetInventory(rows)) => Some(rows),
            _ => None,
        }
    }

    pub(super) fn asset_row(&self, id: &str) -> Option<&AssetRow> {
        self.inventory_rows()?.iter().find(|row| row.id == id)
    }

    pub(super) fn asset_statuses(&self, id: &str, cx: &App) -> (&'static str, &'static str) {
        let row = self.asset_row(id);
        let local = match row.map(|row| row.local_exists) {
            Some(Some(true)) => text(cx, "Environment: present", "当前环境：存在"),
            Some(Some(false)) => text(cx, "Environment: missing", "当前环境：缺失"),
            Some(None) => text(cx, "Environment: unknown", "当前环境：未知"),
            None => text(cx, "Environment: unchecked", "当前环境：未检查"),
        };
        let remote = if self.inventory_scope() == Some(true) && !self.operation.busy {
            match row.and_then(|row| row.remote_exists) {
                Some(true) => text(cx, "WebDAV: present", "WebDAV：存在"),
                Some(false) => text(cx, "WebDAV: missing", "WebDAV：缺失"),
                None => text(cx, "WebDAV: unknown", "WebDAV：未知"),
            }
        } else {
            text(cx, "WebDAV: unchecked", "WebDAV：未检查")
        };
        (local, remote)
    }

    pub(super) fn inspect_environment_button(
        &self,
        id: &'static str,
        cx: &mut Context<Self>,
    ) -> Button {
        semantics::ghost_button(
            Button::new(id)
                .label(text(cx, "Inspect environment assets", "查看当前环境资产"))
                .on_click(cx.listener(|this, _, _, cx| {
                    this.start(
                        Operation::AssetInventory {
                            revision: String::new(),
                            probe: false,
                        },
                        cx,
                    );
                })),
            self.asset_actions_locked() || self.asset_editor.is_some(),
            self.operation.busy && self.inventory_scope() == Some(false),
            cx,
        )
    }

    pub(super) fn render_local_inventory_status(&self, cx: &App) -> Option<layout::Panel> {
        if self.inventory_scope() != Some(false) {
            return None;
        }
        let status = if self.operation.busy {
            text(
                cx,
                "Inspecting environment assets…",
                "正在查看当前环境资产…",
            )
            .to_string()
        } else {
            let Some(Err(error)) = self.operation.result else {
                return None;
            };
            format!(
                "{} {}",
                text(
                    cx,
                    "Environment inspection failed.",
                    "查看当前环境资产失败。"
                ),
                sync_error(cx, error)
            )
        };
        Some(sync_status("sync-asset-local-status", status))
    }

    fn render_remote_inventory(&self, cx: &App) -> impl IntoElement {
        if self.inventory_scope() == Some(true)
            && let Some(rows) = self.inventory_rows()
        {
            return distribution(rows, cx);
        }
        let status = if self.inventory_scope() == Some(true) {
            if self.operation.busy {
                text(cx, "Checking remote assets…", "正在检查远程资产…").to_string()
            } else if let Some(Err(error)) = self.operation.result {
                format!(
                    "{} {}",
                    text(cx, "Remote asset check failed.", "远程资产检查失败。"),
                    sync_error(cx, error)
                )
            } else {
                text(
                    cx,
                    "Remote assets have not been checked.",
                    "尚未检查远程资产。",
                )
                .to_string()
            }
        } else if self.busy {
            text(cx, "Reading WebDAV configuration…", "正在读取 WebDAV 配置…").to_string()
        } else if self.snapshot.is_none() {
            text(
                cx,
                "Read the WebDAV configuration before checking remote assets.",
                "检查远程资产前，请先读取 WebDAV 配置。",
            )
            .to_string()
        } else if self.probe_revision().is_none() {
            text(
                cx,
                "Configure and enable WebDAV to check remote assets.",
                "配置并启用 WebDAV 后可检查远程资产。",
            )
            .to_string()
        } else {
            text(
                cx,
                "Remote assets have not been checked.",
                "尚未检查远程资产。",
            )
            .to_string()
        };
        div().child(sync_status("sync-asset-remote-status", status))
    }

    pub(super) fn render_assets(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let locked = self.asset_actions_locked();
        let mut section = div()
            .id("sync-assets")
            .test_support()
            .flex()
            .flex_col()
            .flex_shrink_0()
            .gap(px(theme::CONTROL_GAP))
            .child(section_heading(text(
                cx,
                "Configuration assets",
                "配置资产",
            )))
            .child(
                sync_actions()
                    .child(self.inspect_environment_button("sync-assets-local", cx))
                    .child(semantics::ghost_button(
                        Button::new("sync-assets-probe")
                            .label(text(cx, "Check remote assets", "检查远程资产"))
                            .on_click(cx.listener(|this, _, _, cx| {
                                if let Some(revision) = this.probe_revision().map(str::to_owned) {
                                    this.start(
                                        Operation::AssetInventory {
                                            revision,
                                            probe: true,
                                        },
                                        cx,
                                    );
                                }
                            })),
                        locked || self.asset_editor.is_some() || self.probe_revision().is_none(),
                        self.operation.busy && self.inventory_scope() == Some(true),
                        cx,
                    ))
                    .child(
                        div()
                            .id("sync-assets-all-anchor")
                            .track_focus(&self.all_assets_transfer_anchor)
                            .tab_stop(false)
                            .child(semantics::ghost_button(
                                Button::new("sync-assets-all")
                                    .label(text(cx, "All assets", "全部资产"))
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.edit_asset(None, window, cx);
                                    })),
                                self.asset_transfer_locked()
                                    || self.asset_editor.is_some()
                                    || self.is_ssh(),
                                false,
                                cx,
                            )),
                    ),
            )
            .child(
                div()
                    .text_size(theme::text_size(theme::META_SIZE))
                    .children(self.render_local_inventory_status(cx))
                    .child(self.render_remote_inventory(cx)),
            )
            .child(
                div().children(SYNC_ASSETS.iter().enumerate().map(|(index, asset)| {
                    let name = asset_name(asset);
                    let protection = protection(asset, cx);
                    let (local, remote) = self.asset_statuses(asset.id, cx);
                    let details = text(cx, "Details", "详情");
                    let id = SharedString::from(format!("sync-asset-details-{}", asset.id));
                    let selected = self.asset_detail == Some(index);
                    div()
                        .id(("sync-asset-row", index))
                        .track_focus(&self.asset_row_anchors[index])
                        .tab_stop(false)
                        .border_t_1()
                        .border_color(cx.theme().border)
                        .child(semantics::disclosure(
                            semantics::ghost_button(
                                Button::new(id.clone())
                                    .accessibility_id(id)
                                    .accessibility_label(format!(
                                        "{name} · {protection} · {local} · {remote} · {details}"
                                    ))
                                    .with_size(px(theme::ROW_HEIGHT))
                                    .w_full()
                                    .min_w_0()
                                    .min_h(px(theme::ROW_HEIGHT))
                                    .py(px(4.))
                                    .px(px(6.))
                                    .child(
                                        div()
                                            .flex()
                                            .flex_1()
                                            .min_w_0()
                                            .items_center()
                                            .gap(px(theme::CONTROL_GAP))
                                            .child(
                                                sync_actions()
                                                    .flex_1()
                                                    .min_w_0()
                                                    .items_center()
                                                    .justify_between()
                                                    .child(
                                                        div()
                                                            .font_weight(FontWeight::MEDIUM)
                                                            .child(name),
                                                    )
                                                    .child(
                                                        div()
                                                            .flex()
                                                            .flex_wrap()
                                                            .items_center()
                                                            .gap(px(theme::CONTROL_GAP))
                                                            .text_size(theme::text_size(
                                                                theme::META_SIZE,
                                                            ))
                                                            .text_color(cx.theme().muted_foreground)
                                                            .child(div().child(protection))
                                                            .child(div().child(local))
                                                            .child(div().child(remote)),
                                                    ),
                                            )
                                            .child(
                                                Icon::new(IconName::ChevronRight)
                                                    .size(px(theme::META_SIZE)),
                                            ),
                                    )
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.open_asset_details(index, cx);
                                    })),
                                self.asset_draft_blocked() || self.asset_editor.is_some(),
                                false,
                                cx,
                            )
                            .rounded(ButtonRounded::Size(px(theme::SELECTION_RADIUS))),
                            selected,
                        ))
                })),
            );
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

        if self.asset_detail.is_none() {
            section = section.children(self.render_asset_editor(cx));
        }
        section
    }

    pub(super) fn render_asset_editor(&self, cx: &mut Context<Self>) -> Option<layout::Panel> {
        let locked = self.asset_transfer_locked();
        let mut section = div()
            .id("sync-asset-transfer")
            .test_support()
            .flex()
            .flex_col()
            .gap(px(theme::CONTROL_GAP));
        let editor = self.asset_editor.as_ref()?;
        {
            section = section.child(div().id("sync-asset-editor").test_support().flex().flex_col().gap(px(theme::CONTROL_GAP))
                .child(editor.id.clone().unwrap_or_else(||text(cx,"All assets","全部资产").into()))
                .child(Input::new(&editor.passphrase).aria_label(text(cx,"Asset operation passphrase","资产操作口令")).accessibility_id("sync-asset-passphrase"))
                .child(semantics::button(Button::new("sync-asset-force").label(if editor.force{text(cx,"Replacement allowed","允许覆盖")}else{text(cx,"Replacement blocked","禁止覆盖")}).ghost().on_click(cx.listener(|this,_,_,cx|{if let Some(e)=&mut this.asset_editor{e.force = !e.force;e.confirmation=None;}cx.notify();})),locked,false,cx))
                .child(semantics::button(Button::new("sync-asset-v1").label(if editor.import_v1{text(cx,"Plaintext v1 import allowed","允许导入明文 v1")}else{text(cx,"Plaintext v1 import blocked","禁止导入明文 v1")}).ghost().on_click(cx.listener(|this,_,_,cx|{if let Some(e)=&mut this.asset_editor{e.import_v1 = !e.import_v1;e.confirmation=None;}cx.notify();})),locked,false,cx))
                .child(text(cx,"Replacement permits upload overwrite or local replacement with backup. Sync prefers local when both copies exist. Plaintext v1 import only reads old data; it does not re-encrypt the remote copy.","允许覆盖后，上传可替换远程数据，下载会备份再替换本地数据。双向同步在两端都存在时优先本地。明文 v1 导入只读取旧数据，不会重新加密远程副本。"))
                .child(sync_actions().children([(AssetDirection::Push,"sync-asset-push",text(cx,"Upload asset","上传资产")),(AssetDirection::Pull,"sync-asset-pull",text(cx,"Download asset","下载资产")),(AssetDirection::Sync,"sync-asset-sync",text(cx,"Sync asset","同步资产"))].into_iter().map(|(direction,id,label)|semantics::button(Button::new(id).label(label).on_click(cx.listener(move|this,_,_,cx|{if let Some(e)=&mut this.asset_editor{e.confirmation=Some(direction);}cx.notify();})),locked,false,cx))))
                .child(semantics::button(Button::new("sync-asset-discard").label(text(cx,"Cancel","取消")).ghost().on_click(cx.listener(|this,_,window,cx|this.cancel_asset_edit(window, cx))),locked,false,cx)));
            if let Some(direction) = editor.confirmation {
                section = section.child(div().id("sync-asset-confirmation").test_support().flex().flex_col().gap(px(theme::CONTROL_GAP))
                    .child(match direction{AssetDirection::Push=>text(cx,"Upload the selected configuration assets to the saved WebDAV account?","将所选配置资产上传到已保存的 WebDAV 账户？"),AssetDirection::Pull=>text(cx,"Download the selected configuration assets and apply them locally? Existing local data will be backed up when replacement is allowed.","下载所选配置资产并应用到本地？允许覆盖时，会备份已有本地数据。"),AssetDirection::Sync=>text(cx,"Synchronize selected assets using the displayed replacement and legacy import choices?","按上方覆盖和旧数据导入选项同步所选资产？")})
                    .child(semantics::button(Button::new("sync-asset-confirm").label(text(cx,"Confirm transfer","确认传输")).on_click(cx.listener(|this,_,_,cx|this.send_asset(cx))),locked,false,cx)));
            }
        }
        Some(section)
    }
}

pub(super) fn asset_name(asset: &SyncAssetDefinition) -> String {
    match asset.group {
        SyncAssetGroup::Ccr => asset.name.to_owned(),
        SyncAssetGroup::Claude => format!("Claude Code · {}", asset.name),
        SyncAssetGroup::Codex => format!("Codex · {}", asset.name),
    }
}

fn protection(asset: &SyncAssetDefinition, cx: &App) -> &'static str {
    if asset.sensitive {
        text(cx, "Encrypted v2", "v2 加密")
    } else {
        text(cx, "As stored", "原样传输")
    }
}

fn distribution(rows: &[AssetRow], cx: &App) -> gpui_kit::Div {
    let present = rows
        .iter()
        .filter(|row| row.remote_exists == Some(true))
        .count();
    let missing = rows
        .iter()
        .filter(|row| row.remote_exists == Some(false))
        .count();
    let unknown = rows.len() - present - missing;
    let counts = [
        (present, text(cx, "Present", "存在"), cx.theme().success),
        (
            missing,
            text(cx, "Missing", "缺失"),
            cx.theme().muted_foreground,
        ),
        (unknown, text(cx, "Unknown", "未知"), cx.theme().warning),
    ];
    let summary = if unknown == 0 {
        text(cx, "Remote assets checked", "远程资产检查完成")
    } else {
        text(cx, "Remote check incomplete", "远程检查结果不完整")
    };
    let [present_label, missing_label, unknown_label] =
        counts.map(|(count, label, _)| format!("{label} {count}"));
    div()
        .flex()
        .flex_col()
        .gap(px(4.))
        .child(sync_status(
            "sync-asset-distribution",
            format!("{summary} · {present_label} · {missing_label} · {unknown_label}"),
        ))
        .child(
            div()
                .flex()
                .w(px(180.))
                .max_w_full()
                .h(px(5.))
                .rounded_full()
                .overflow_hidden()
                .children(counts.into_iter().filter(|(count, _, _)| *count > 0).map(
                    |(count, _, color)| {
                        div()
                            .h_full()
                            .w(relative(count as f32 / rows.len() as f32))
                            .bg(color)
                    },
                )),
        )
}
