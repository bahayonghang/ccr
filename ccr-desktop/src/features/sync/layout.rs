use super::*;
use gpui_kit::component::ActiveTheme;
use gpui_kit::{Div, Role, Stateful, relative};

pub(super) type Panel = gpui_kit::base::ObservedElement<Stateful<Div>>;

impl Render for SyncView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.finish_editor_save(window, cx);
        let split = window.viewport_size().width >= theme::data_row_height(theme::SPLIT_WIDTH, cx);
        let details = self
            .render_management_confirmation(window, cx)
            .or_else(|| self.render_editor(cx))
            .or_else(|| {
                self.render_asset_details(cx)
                    .map(IntoElement::into_any_element)
            });
        let editing = details.is_some();
        let mut result = if editing {
            None
        } else {
            self.render_operation_status(cx)
        };
        let mut body = div()
            .id("sync-workspace")
            .test_support()
            .flex()
            .gap(px(theme::SECTION_GAP));
        if split {
            body = body.items_start();
        } else {
            body = body.flex_col();
        }
        if !editing || split {
            let mut list = div()
                .flex()
                .flex_col()
                .flex_1()
                .min_w_0()
                .gap(px(theme::SECTION_GAP));
            if !split {
                list = list.children(
                    result
                        .take()
                        .map(|result| reading_panel("sync-result", cx).child(result)),
                );
            }
            list = list
                .child(reading_panel("sync-asset-list", cx).child(self.render_assets(cx)))
                .child(self.render_folders(cx));
            body = body.child(list);
        }
        if let Some(details) = details {
            body = body.child(detail_panel(split, cx).child(details));
        } else if split && let Some(result) = result {
            body = body.child(
                detail_panel(true, cx)
                    .child(section_heading(text(cx, "Latest operation", "最近操作")))
                    .child(result),
            );
        }
        div()
            .id("sync-page")
            .test_support()
            .track_focus(&self.focus)
            .tab_stop(false)
            .on_key_down(
                cx.listener(|this, event: &gpui_kit::KeyDownEvent, window, cx| {
                    if event.keystroke.key != "escape" {
                        return;
                    }
                    if this.management_confirmation.is_some() {
                        this.cancel_management(window, cx);
                        cx.stop_propagation();
                    } else if this.editor.is_some() && !this.busy && !this.operation.busy {
                        this.close_editor(window, cx);
                        cx.stop_propagation();
                    } else if this.asset_editor.is_some() {
                        this.cancel_asset_edit(window, cx);
                        cx.stop_propagation();
                    } else if this.editor.is_none() && this.asset_detail.is_some() {
                        this.close_asset_details(window, cx);
                        cx.stop_propagation();
                    }
                }),
            )
            .flex()
            .flex_col()
            .size_full()
            .min_w_0()
            .overflow_y_scroll()
            .p(px(
                if window.viewport_size().width < px(theme::NARROW_WIDTH) {
                    theme::NARROW_CONTENT_PADDING
                } else {
                    theme::CONTENT_PADDING
                },
            ))
            .gap(px(theme::SECTION_GAP))
            .child(
                sync_actions()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .id("sync-heading")
                            .test_support()
                            .role(Role::Heading)
                            .aria_level(1)
                            .aria_label(text(cx, "Sync", "同步"))
                            .text_size(theme::text_size(theme::TITLE_SIZE))
                            .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                            .child(text(cx, "Sync", "同步")),
                    )
                    .child(self.render_environment(cx)),
            )
            .child(self.render_webdav(cx))
            .child(body)
    }
}

impl SyncView {
    fn render_environment(&self, cx: &App) -> impl IntoElement {
        let environment = if self.is_ssh() {
            self.environment_name
                .as_ref()
                .map_or_else(|| "SSH".into(), |name| format!("SSH · {name}"))
        } else {
            text(cx, "Local", "本地").into()
        };
        let label = format!(
            "{} · {environment}",
            text(cx, "Execution environment", "执行环境")
        );
        div()
            .id("sync-environment")
            .test_support()
            .aria_label(label.clone())
            .text_color(cx.theme().muted_foreground)
            .child(label)
    }

    fn render_folders(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let mut panel = reading_panel("sync-folders", cx).child(section_heading(text(
            cx,
            "Sync folders",
            "同步文件夹",
        )));
        if let Some(SyncSnapshot::Present(local)) = &self.snapshot {
            if local.folders.is_empty() {
                panel = panel.child(div().text_color(cx.theme().muted_foreground).child(text(
                    cx,
                    "No sync folders configured.",
                    "尚未配置同步文件夹。",
                )));
            }
            let blocked = self.busy
                || self.operation.busy
                || self.editor.is_some()
                || self.management_confirmation.is_some();
            panel = panel.children(local.folders.iter().enumerate().map(|(index, folder)| {
                let label = folder_label(cx, folder);
                div()
                    .id(SharedString::from(format!("sync-folder-{index}")))
                    .test_support()
                    .flex()
                    .flex_wrap()
                    .min_w_0()
                    .items_center()
                    .gap(px(theme::CONTROL_GAP))
                    .border_t_1()
                    .border_color(cx.theme().border)
                    .aria_label(label.clone())
                    .child(div().flex_1().min_w_0().child(label))
                    .child(semantics::ghost_button(
                        Button::new(("sync-select", index))
                            .label(if self.selected == Some(index) {
                                text(cx, "Selected", "已选择")
                            } else {
                                text(cx, "Select", "选择")
                            })
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.generation += 1;
                                this.selected = Some(index);
                                this.confirmation = None;
                                this.folder_editor = None;
                                this.folder_delete = None;
                                cx.notify();
                            })),
                        blocked,
                        false,
                        cx,
                    ))
            }));
        }
        panel
            .child(self.render_folder_controls(cx))
            .child(self.render_selected_folder_operations(cx))
            .child(self.render_batch_controls(cx))
    }
}

pub(super) fn sync_actions() -> Div {
    div().flex().flex_wrap().gap(px(theme::CONTROL_GAP))
}

pub(super) fn reading_panel(id: &'static str, cx: &App) -> Panel {
    div()
        .id(id)
        .test_support()
        .flex()
        .flex_col()
        .flex_shrink_0()
        .min_w_0()
        .rounded(px(theme::PANEL_RADIUS))
        .bg(cx.theme().background)
        .border_1()
        .border_color(cx.theme().border)
        .p(px(theme::SECTION_GAP))
        .gap(px(theme::CONTROL_GAP))
}

fn detail_panel(split: bool, cx: &App) -> Panel {
    let panel = reading_panel("sync-detail", cx);
    if split {
        panel.w(relative(0.42))
    } else {
        panel.w_full()
    }
}

pub(super) fn section_heading(label: &'static str) -> impl IntoElement {
    div()
        .id(label)
        .test_support()
        .role(Role::Heading)
        .aria_level(2)
        .aria_label(label)
        .text_size(theme::text_size(theme::SECTION_TITLE_SIZE))
        .font_weight(gpui_kit::FontWeight::SEMIBOLD)
        .child(label)
}

pub(super) fn sync_status(id: &'static str, label: impl Into<SharedString>) -> Panel {
    let label = label.into();
    div()
        .id(id)
        .test_support()
        .role(Role::Status)
        .aria_label(label.clone())
        .child(label)
}
