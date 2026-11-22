mod focus;

use super::asset_details::AssetDetailFocus;
use super::assets::asset_name;
use super::*;
use crate::runtime::sync::{AssetDirection, AssetRequest};
use ccr_sync::sync::assets::{SYNC_ASSETS, SyncAssetDefinition, asset_by_id};
use focus::{Control, TransferFocus};
use gpui_kit::component::{ActiveTheme, Disableable, FocusTrapElement, checkbox::Checkbox};
use gpui_kit::{AnyElement, Pixels, Point};

pub(super) struct AssetEditor {
    revision: String,
    generation: u64,
    // None denotes all five fixed assets. A single index is also the originating detail.
    target: Option<usize>,
    return_offset: Point<Pixels>,
    focus: TransferFocus,
    pub(super) review: Option<AssetReview>,
}

pub(super) struct AssetReview {
    direction: AssetDirection,
    pub(super) passphrase: Option<Entity<InputState>>,
    force: bool,
    import_v1: bool,
}

impl AssetEditor {
    fn definitions(&self) -> &[SyncAssetDefinition] {
        match self.target {
            Some(index) => std::slice::from_ref(&SYNC_ASSETS[index]),
            None => SYNC_ASSETS,
        }
    }
}

impl AssetReview {
    fn clear_passphrase(&self, window: &mut Window, cx: &mut Context<SyncView>) {
        if let Some(input) = &self.passphrase {
            input.update(cx, |state, cx| state.set_value("", window, cx));
        }
    }
}

impl SyncView {
    pub(super) fn edit_asset(
        &mut self,
        target: Option<usize>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.asset_transfer_locked()
            || self.asset_editor.is_some()
            || target.is_some_and(|index| self.asset_detail != Some(index))
            || (self.is_ssh()
                && target.is_none_or(|index| SYNC_ASSETS[index].id == "ccr-platforms"))
        {
            return;
        }
        let Some(revision) = self.revision() else {
            return;
        };
        self.reset_folder_panel();
        self.asset_detail = target;
        self.asset_editor = Some(AssetEditor {
            revision,
            generation: self.generation,
            target,
            return_offset: self.folder_focus.scroll.offset(),
            focus: TransferFocus::new(window, cx),
            review: None,
        });
        self.reveal_asset_editor(cx);
    }

    fn reveal_asset_editor(&mut self, cx: &mut Context<Self>) {
        self.asset_detail_focus = Some((self.generation, AssetDetailFocus::TransferEditor));
        self.folder_focus.scroll.scroll_to_top_of_item(2);
        cx.notify();
    }

    fn return_from_asset_editor(&mut self, editor: &AssetEditor, cx: &mut Context<Self>) {
        self.folder_focus.scroll.set_offset(editor.return_offset);
        self.asset_detail_focus = Some((
            self.generation,
            if editor.target.is_some() {
                AssetDetailFocus::SingleTransfer
            } else {
                AssetDetailFocus::AllTransfer
            },
        ));
        cx.notify();
    }

    pub(super) fn cancel_asset_edit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(editor) = self.discard_asset_editor(window, cx) {
            self.return_from_asset_editor(&editor, cx);
        }
    }

    pub(super) fn discard_asset_editor(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<AssetEditor> {
        let editor = self.asset_editor.take()?;
        if let Some(review) = &editor.review {
            review.clear_passphrase(window, cx);
        }
        Some(editor)
    }

    fn review_asset_transfer(
        &mut self,
        direction: AssetDirection,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.asset_transfer_locked() {
            return;
        }
        if let Some(editor) = &mut self.asset_editor
            && editor.review.is_none()
        {
            let sensitive = editor.definitions().iter().any(|asset| asset.sensitive);
            editor.review = Some(AssetReview {
                direction,
                passphrase: sensitive
                    .then(|| cx.new(|cx| InputState::new(window, cx).masked(true))),
                force: false,
                import_v1: false,
            });
            self.reveal_asset_editor(cx);
        }
    }

    fn choose_asset_direction(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(review) = self
            .asset_editor
            .as_mut()
            .and_then(|editor| editor.review.take())
        {
            review.clear_passphrase(window, cx);
            self.reveal_asset_editor(cx);
        }
    }

    fn send_asset(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.asset_transfer_locked() {
            return;
        }
        let Some(mut editor) = self.asset_editor.take_if(|editor| editor.review.is_some()) else {
            return;
        };
        let Some(review) = editor.review.take() else {
            return;
        };
        let current = self.generation == editor.generation
            && self.asset_detail == editor.target
            && self.revision().as_ref() == Some(&editor.revision);
        if !current {
            review.clear_passphrase(window, cx);
            self.message = Some(SyncError::Conflict.message());
            self.return_from_asset_editor(&editor, cx);
            return;
        }
        let passphrase = review.passphrase.as_ref().and_then(|input| {
            let value = input.read(cx).value().to_string();
            (!value.is_empty()).then(|| value.into())
        });
        review.clear_passphrase(window, cx);
        self.start(
            Operation::Assets(AssetRequest {
                revision: editor.revision,
                id: editor.target.map(|index| SYNC_ASSETS[index].id.to_owned()),
                direction: review.direction,
                force: review.force,
                passphrase,
                import_plaintext_v1: review.import_v1,
            }),
            cx,
        );
        self.asset_detail = None;
        self.asset_detail_focus = Some((self.generation, AssetDetailFocus::TransferResult));
        self.folder_focus.scroll.scroll_to_top_of_item(2);
    }

    pub(super) fn render_asset_editor(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        let editor = self.asset_editor.as_ref()?;
        if self.asset_detail_focus == Some((self.generation, AssetDetailFocus::TransferEditor)) {
            self.asset_detail_focus = None;
            self.restore_action_focus(self.asset_editor_anchor.clone(), window, cx);
        }
        let locked = self.asset_transfer_locked();
        let mut pane = div()
            .flex()
            .flex_col()
            .min_w_0()
            .gap(px(theme::CONTROL_GAP))
            .child(section_heading(
                match editor.review.as_ref().map(|review| review.direction) {
                    None => text(cx, "Transfer configuration assets", "传输配置资产"),
                    Some(AssetDirection::Push) => {
                        text(cx, "Upload configuration assets", "上传配置资产")
                    }
                    Some(AssetDirection::Pull) => {
                        text(cx, "Download configuration assets", "下载配置资产")
                    }
                    Some(AssetDirection::Sync) => {
                        text(cx, "Sync configuration assets", "同步配置资产")
                    }
                },
            ))
            .child(sync_status(
                "sync-asset-transfer-environment",
                self.environment_label(cx),
            ));
        if let Some(SyncSnapshot::Present(local)) = &self.snapshot {
            pane = pane
                .child(sync_status(
                    "sync-asset-transfer-webdav",
                    super::webdav::url_line(cx, &local.url),
                ))
                .children(local.asset_remote_base.as_ref().map(|base| {
                    sync_status(
                        "sync-asset-transfer-base",
                        format!(
                            "{} · {base}",
                            text(cx, "WebDAV asset base path", "WebDAV 资产根路径")
                        ),
                    )
                }));
        }
        if editor.target.is_none() {
            pane = pane.child(text(
                cx,
                "All five configuration assets",
                "全部五项配置资产",
            ));
        }
        pane = pane.child(
            div()
                .flex()
                .flex_wrap()
                .gap(px(theme::CONTROL_GAP))
                .children(editor.definitions().iter().map(|asset| {
                    let label = asset_name(asset);
                    div()
                        .id(SharedString::from(format!(
                            "sync-asset-transfer-target-{}",
                            asset.id
                        )))
                        .test_support()
                        .aria_label(label.clone())
                        .text_color(cx.theme().muted_foreground)
                        .child(label)
                })),
        );

        let cancel = editor.focus.track(
            Control::Cancel,
            semantics::ghost_button(
                Button::new("sync-asset-discard")
                    .label(text(cx, "Cancel", "取消"))
                    .on_click(
                        cx.listener(|this, _, window, cx| this.cancel_asset_edit(window, cx)),
                    ),
                false,
                false,
                cx,
            ),
        );
        let content = if let Some(review) = &editor.review {
            self.render_asset_review(editor, review, locked, cancel, cx)
                .into_any_element()
        } else {
            sync_actions()
                .children(
                    [
                        (
                            Control::Upload,
                            AssetDirection::Push,
                            "sync-asset-push",
                            text(cx, "Upload", "上传"),
                        ),
                        (
                            Control::Download,
                            AssetDirection::Pull,
                            "sync-asset-pull",
                            text(cx, "Download", "下载"),
                        ),
                        (
                            Control::Sync,
                            AssetDirection::Sync,
                            "sync-asset-sync",
                            text(cx, "Sync", "同步"),
                        ),
                    ]
                    .map(|(control, direction, id, label)| {
                        let button = semantics::button(
                            Button::new(id).label(label).on_click(cx.listener(
                                move |this, _, window, cx| {
                                    this.review_asset_transfer(direction, window, cx)
                                },
                            )),
                            locked,
                            false,
                            cx,
                        );
                        editor.focus.track(control, button)
                    }),
                )
                .child(cancel)
                .into_any_element()
        };
        Some(
            div()
                .id("sync-asset-editor")
                .test_support()
                .min_w_0()
                .child(
                    pane.child(content)
                        .focus_trap("sync-asset-transfer-trap", &self.asset_editor_anchor),
                )
                .into_any_element(),
        )
    }

    fn render_asset_review(
        &self,
        editor: &AssetEditor,
        review: &AssetReview,
        locked: bool,
        cancel: impl IntoElement,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let (direction, consequence) = transfer_explanation(review, cx);
        let definitions = editor.definitions();
        let includes_instructions = definitions.iter().any(|asset| !asset.sensitive);
        let replaces_on_download = review.direction == AssetDirection::Pull && review.force;
        let includes_platforms = definitions.iter().any(|asset| asset.id == "ccr-platforms");
        let includes_file = definitions.iter().any(|asset| asset.id != "ccr-platforms");
        let passphrase_label = text(cx, "Operation passphrase", "本次操作口令");
        let legacy = review.direction != AssetDirection::Push && review.passphrase.is_some();
        let focus = &editor.focus;
        div()
            .id("sync-asset-confirmation")
            .test_support()
            .flex()
            .flex_col()
            .min_w_0()
            .gap(px(theme::CONTROL_GAP))
            .child(sync_status("sync-asset-transfer-direction", direction))
            .child(sync_status("sync-asset-transfer-consequence", consequence))
            .children(includes_instructions.then(|| {
                text(
                    cx,
                    "Instruction files transfer their original content over HTTPS.",
                    "指令文件通过 HTTPS 传输原始内容。",
                )
            }))
            .children(review.passphrase.as_ref().map(|input| {
                div()
                    .flex()
                    .flex_col()
                    .min_w_0()
                    .max_w(px(360.))
                    .gap(px(4.))
                    .child(passphrase_label)
                    .child(focus.track(
                        Control::Passphrase,
                        Input::new(input)
                            .aria_label(passphrase_label)
                            .accessibility_id("sync-asset-passphrase")
                            .disabled(locked),
                    ))
                    .child(div().text_color(cx.theme().muted_foreground).child(text(cx,
                        "Sensitive assets require this operation’s passphrase and upload with v2 encryption.",
                        "敏感资产使用本次操作的口令，上传时采用 v2 加密。",
                    )))
            }))
            .child(sync_actions().child(focus.track(
                Control::Overwrite,
                Checkbox::new("sync-asset-force")
                    .label(text(cx, "Allow overwrite", "允许覆盖"))
                    .checked(review.force)
                    .disabled(locked)
                    .on_change(cx.listener(|this, checked: &bool, _, cx| {
                        if let Some(review) = this.asset_editor.as_mut().and_then(|editor| editor.review.as_mut()) {
                            review.force = *checked;
                            cx.notify();
                        }
                    })),
            )))
            .children(legacy.then(|| {
                div()
                    .flex()
                    .flex_col()
                    .min_w_0()
                    .gap(px(4.))
                    .child(sync_actions().child(focus.track(
                        Control::PlaintextImport,
                        Checkbox::new("sync-asset-v1")
                            .label(text(cx, "Import plaintext v1 once", "仅本次导入明文 v1"))
                            .checked(review.import_v1)
                            .disabled(locked)
                            .on_change(cx.listener(|this, checked: &bool, _, cx| {
                                if let Some(review) = this.asset_editor.as_mut().and_then(|editor| editor.review.as_mut()) {
                                    review.import_v1 = *checked;
                                    cx.notify();
                                }
                            })),
                    )))
                    .child(div().text_color(cx.theme().muted_foreground).child(text(cx,
                        "Allows a one-time import of plaintext v1 only if this operation downloads the copy.",
                        "仅在此操作下载副本时，允许读取并导入旧版明文 v1。",
                    )))
            }))
            .children((replaces_on_download && includes_file).then(|| {
                sync_status(
                    "sync-asset-file-backup",
                    if self.is_ssh() {
                        text(cx, "When a file is replaced, its previous contents are backed up on the SSH host.", "替换文件时，原内容会在 SSH 主机上备份。")
                    } else {
                        text(cx, "When a local file is replaced, its previous contents are backed up.", "替换本地文件时会先备份原内容。")
                    },
                )
            }))
            .children((replaces_on_download && includes_platforms).then(|| {
                sync_status(
                    "sync-asset-platform-backup",
                    text(cx,
                        "CCR Platforms downloads containing profiles.db use database transactions with rollback. A separate directory backup is available only when the download replaces the directory as a whole.",
                        "下载 CCR Platforms 时，profiles.db 在数据库事务内更新，事务失败时回滚。只有整体替换目录时才会保留独立的目录备份。",
                    ),
                )
            }))
            .child(
                sync_actions()
                    .child(focus.track(
                        Control::ChangeDirection,
                        semantics::ghost_button(
                            Button::new("sync-asset-change-direction")
                                .label(text(cx, "Change direction", "更改方向"))
                                .on_click(cx.listener(|this, _, window, cx| this.choose_asset_direction(window, cx))),
                            locked,
                            false,
                            cx,
                        ),
                    ))
                    .child(cancel)
                    .child(focus.track(
                        Control::Confirm,
                        semantics::button(
                            Button::new("sync-asset-confirm")
                                .label(text(cx, "Confirm transfer", "确认传输"))
                                .primary()
                                .on_click(cx.listener(|this, _, window, cx| this.send_asset(window, cx))),
                            locked,
                            false,
                            cx,
                        ),
                    )),
            )
    }
}

fn transfer_explanation(review: &AssetReview, cx: &App) -> (&'static str, &'static str) {
    match review.direction {
        AssetDirection::Push => (
            text(cx, "Current environment → WebDAV", "当前环境 → WebDAV"),
            if review.force {
                text(
                    cx,
                    "Upload replaces the WebDAV copy if it exists.",
                    "已有 WebDAV 副本会被上传内容覆盖。",
                )
            } else {
                text(
                    cx,
                    "An existing WebDAV copy causes a conflict.",
                    "已有 WebDAV 副本时会报告冲突。",
                )
            },
        ),
        AssetDirection::Pull => (
            text(cx, "WebDAV → Current environment", "WebDAV → 当前环境"),
            if review.force {
                text(
                    cx,
                    "Download can replace an existing copy in this environment.",
                    "下载可替换当前环境中已有的副本。",
                )
            } else {
                text(
                    cx,
                    "An existing copy in this environment causes a conflict.",
                    "当前环境已有副本时会报告冲突。",
                )
            },
        ),
        AssetDirection::Sync => (
            text(
                cx,
                "Current environment ↔ WebDAV. If only one copy exists, sync fills the missing side.",
                "当前环境 ↔ WebDAV。仅一侧有副本时，同步会补齐缺失的一侧。",
            ),
            if review.force {
                text(
                    cx,
                    "When both copies exist, the current environment overwrites WebDAV.",
                    "两端都有副本时，用当前环境的数据覆盖 WebDAV。",
                )
            } else {
                text(
                    cx,
                    "When both copies exist, sync reports a conflict.",
                    "两端都有副本时，同步会报告冲突。",
                )
            },
        ),
    }
}

pub(super) fn render_asset_result(
    id: &str,
    result: &Result<Option<String>, SyncError>,
    cx: &App,
) -> impl IntoElement {
    let name = asset_by_id(id).map_or_else(|_| id.to_owned(), asset_name);
    let outcome = match result {
        Ok(Some(backup)) => format!(
            "{} · {}: {backup}",
            text(cx, "Completed", "已完成"),
            text(cx, "Backup", "备份"),
        ),
        Ok(None) => text(cx, "Completed", "已完成").into(),
        Err(error) => format!(
            "{} · {}",
            if *error == SyncError::RemotePartial {
                text(cx, "Partially completed", "部分完成")
            } else {
                text(cx, "Failed", "失败")
            },
            sync_error(cx, *error),
        ),
    };
    let label = format!("{name} · {outcome}");
    div()
        .id(SharedString::from(format!("sync-asset-result-{id}")))
        .test_support()
        .min_w_0()
        .whitespace_normal()
        .aria_label(label.clone())
        .text_color(if result.is_ok() {
            cx.theme().foreground
        } else {
            cx.theme().danger
        })
        .child(label)
}
