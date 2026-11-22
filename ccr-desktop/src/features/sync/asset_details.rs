use super::assets::asset_name;
use super::*;
use ccr_sync::sync::assets::SYNC_ASSETS;
use gpui_kit::Role;
use gpui_kit::component::ActiveTheme;

impl SyncView {
    pub(super) fn open_asset_details(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.environment != self.services.environment() || self.asset_draft_blocked() {
            return;
        }
        self.asset_detail = Some(index);
        self.asset_editor = None;
        self.restore_action_focus(self.asset_back_anchor.clone(), window, cx);
        cx.notify();
    }

    pub(super) fn close_asset_details(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(index) = self.asset_detail.take() else {
            return;
        };
        self.asset_editor = None;
        self.restore_action_focus(self.asset_row_anchors[index].clone(), window, cx);
        cx.notify();
    }

    pub(super) fn render_asset_details(&mut self, cx: &mut Context<Self>) -> Option<layout::Panel> {
        let index = self.asset_detail?;
        let asset = &SYNC_ASSETS[index];
        let name = asset_name(asset);
        let (local, remote) = self.asset_statuses(asset.id, cx);
        let mut pane = div()
            .id("sync-asset-details")
            .test_support()
            .flex()
            .flex_col()
            .min_w_0()
            .gap(px(theme::CONTROL_GAP))
            .child(
                div()
                    .id("sync-asset-back-anchor")
                    .track_focus(&self.asset_back_anchor)
                    .tab_stop(false)
                    .child(semantics::ghost_button(
                        Button::new("sync-asset-back")
                            .label(text(cx, "Back to assets", "返回资产列表"))
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.close_asset_details(window, cx);
                            })),
                        false,
                        false,
                        cx,
                    )),
            )
            .child(
                div()
                    .id("sync-asset-title")
                    .role(Role::Heading)
                    .aria_level(2)
                    .aria_label(name.clone())
                    .text_size(theme::text_size(theme::SECTION_TITLE_SIZE))
                    .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                    .child(name),
            )
            .child(sync_status("sync-asset-detail-status", format!("{local} · {remote}")))
            .child(if asset.sensitive {
                text(
                    cx,
                    "Transfers use the v2 encrypted envelope. The operation passphrase is never saved.",
                    "传输使用 v2 加密封装，本次操作口令不会保存。",
                )
            } else {
                text(
                    cx,
                    "This instruction file transfers as stored over HTTPS.",
                    "此指令文件通过 HTTPS 原样传输。",
                )
            })
            .child(
                div()
                    .font_weight(gpui_kit::FontWeight::MEDIUM)
                    .child(text(cx, "Path in this environment", "当前执行环境路径")),
            );
        if let Some(path) = self
            .asset_row(asset.id)
            .and_then(|row| row.local_path.as_ref())
        {
            pane = pane.child(
                div()
                    .id("sync-asset-environment-path")
                    .test_support()
                    .aria_label(path.clone())
                    .w_full()
                    .min_w_0()
                    .text_color(cx.theme().muted_foreground)
                    .child(path.clone()),
            );
        } else {
            pane = pane.child(text(
                cx,
                "Inspect the current environment to read the full path.",
                "查看当前环境资产以读取完整路径。",
            ));
        }
        pane = pane
            .children(self.render_local_inventory_status(cx))
            .child(div().child(self.inspect_environment_button("sync-asset-inspect", cx)));
        if self.is_ssh() && asset.id == "ccr-platforms" {
            pane = pane.child(text(
                cx,
                "This asset can be inspected on SSH. Profile database transfer needs a remote transaction owner.",
                "可检查 SSH 上的此资产。方案数据库传输需要远端事务所有者。",
            ));
        } else if self.asset_editor.is_none() {
            let id = asset.id;
            pane = pane.child(
                div()
                    .id("sync-asset-transfer-anchor")
                    .track_focus(&self.asset_transfer_anchor)
                    .tab_stop(false)
                    .child(semantics::ghost_button(
                        Button::new(("sync-asset", index))
                            .label(text(cx, "Transfer asset…", "传输资产…"))
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.edit_asset(Some(id.to_owned()), window, cx);
                            })),
                        self.asset_transfer_locked(),
                        false,
                        cx,
                    )),
            );
        }
        Some(pane.children(self.render_asset_editor(cx)))
    }
}
