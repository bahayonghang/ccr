use super::*;
use crate::runtime::sync::AssetRow;
use ccr_sync::sync::assets::{SYNC_ASSETS, SyncAssetDefinition, SyncAssetGroup};
use gpui_kit::component::button::ButtonRounded;
use gpui_kit::component::{ActiveTheme, Icon, IconName, Sizable};
use gpui_kit::{FontWeight, relative};

impl SyncView {
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
        div()
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
                                    .label(text(cx, "Transfer all assets…", "传输全部资产…"))
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.edit_asset(None, cx);
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
            )
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
