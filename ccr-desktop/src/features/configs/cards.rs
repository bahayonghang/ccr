use super::*;
use gpui_kit::base::Button as CardButton;
use gpui_kit::{AnyElement, App, FontWeight, KeyDownEvent, transparent_black};

impl ConfigurationApp {
    fn card_handles(&mut self, name: &str, cx: &App) -> (FocusHandle, FocusHandle) {
        self.card_focus
            .entry(name.to_owned())
            .or_insert_with(|| (cx.focus_handle(), cx.focus_handle()))
            .clone()
    }

    fn preview(&mut self, name: &str, window: &mut Window, cx: &mut Context<Self>) {
        if self.tools.read(cx).expanded() || self.tools.read(cx).busy() {
            return;
        }
        self.selected = Some(name.to_owned());
        self.detail_open = true;
        self.filter_panel = None;
        self.sync_target(cx);
        window.focus(&self.detail_focus, cx);
        cx.notify();
    }

    fn move_focus(
        &mut self,
        index: usize,
        apply: bool,
        key: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let target = match key {
            "left" => index.saturating_sub(1),
            "right" => index + 1,
            "up" => index.saturating_sub(self.columns),
            "down" => index + self.columns,
            _ => return,
        }
        .min(self.rows.len().saturating_sub(1));
        let Some(name) = self.rows.get(target).map(|r| r.name.clone()) else {
            return;
        };
        let (body, action) = self.card_handles(&name, cx);
        self.scroll.scroll_to_reveal_item(target / self.columns);
        window.focus(if apply { &action } else { &body }, cx);
        cx.stop_propagation();
        cx.notify();
    }

    fn resize_grid(
        &mut self,
        width: gpui_kit::Pixels,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let minimum = (f32::from(cx.theme().font_size) / theme::BODY_SIZE) * 264.;
        let columns = (((f32::from(width) + 12.) / (minimum + 12.)) as usize).clamp(1, 3);
        if columns == self.columns {
            return;
        }
        let top = self.scroll.logical_scroll_top();
        let anchor = top.item_ix * self.columns;
        self.columns = columns;
        self.scroll.reset(self.rows.len().div_ceil(columns));
        self.scroll.scroll_to(ListOffset {
            item_ix: anchor / columns,
            ..top
        });
        if let Some(index) = self.rows.iter().position(|r| {
            self.card_focus
                .get(&r.name)
                .is_some_and(|(body, apply)| body.is_focused(window) || apply.is_focused(window))
        }) {
            self.scroll.scroll_to_reveal_item(index / columns);
        }
        cx.notify();
    }

    pub(super) fn card_list(&mut self, _: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let virtual_grid = list(
            self.scroll.clone(),
            cx.processor(|this, band: usize, _, cx| {
                let start = band * this.columns;
                let end = (start + this.columns).min(this.rows.len());
                let mut line = div()
                    .grid()
                    .grid_cols(this.columns as u16)
                    .gap_x(px(12.))
                    .pb(px(12.))
                    .w_full();
                for index in start..end {
                    let row = this.rows[index].clone();
                    line = line.child(this.card(index, &row, cx));
                }
                line.into_any_element()
            }),
        )
        .size_full();
        let view = cx.weak_entity();
        div()
            .on_children_prepainted(move |bounds, window, cx| {
                if let Some(bounds) = bounds.first() {
                    let _ = view.update(cx, |this, cx| {
                        this.resize_grid(bounds.size.width, window, cx)
                    });
                }
            })
            .id("list-focus")
            .test_support()
            .role(gpui_kit::Role::List)
            .aria_label(text(cx, "Configuration profiles", "配置方案"))
            .track_focus(&self.list_focus)
            .tab_stop(false)
            .flex_grow(3.)
            .flex_basis(px(0.))
            .min_w_0()
            .min_h_0()
            .child(virtual_grid)
            .into_any_element()
    }

    fn card(&mut self, index: usize, row: &ConfigRow, cx: &mut Context<Self>) -> AnyElement {
        let (preview_focus, apply_focus) = self.card_handles(&row.name, cx);
        let selected = self.selected.as_deref() == Some(&row.name);
        let disabled = self.loading || self.tools.read(cx).expanded() || self.tools.read(cx).busy();
        let state = self
            .observation
            .as_ref()
            .filter(|o| o.name.as_ref() == Some(&row.name))
            .map(|o| o.state);
        let mut labels = vec![row.name.clone()];
        let mut heading = div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap(px(4.))
            .w_full()
            .child(
                div()
                    .min_w_0()
                    .max_w_full()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_size(theme::text_size(14.))
                    .child(row.name.clone()),
            );
        if let Some(state) = state {
            let label = state_label(state, cx);
            labels.push(label.into());
            let tone = if matches!(
                state,
                RuntimeState::Matched | RuntimeState::OverridesCleared
            ) {
                theme::Tone::Success
            } else {
                theme::Tone::Warning
            };
            heading = heading.child(badge::tag(
                format!("configuration-observed:{}", row.name),
                label,
                tone,
                cx,
            ));
        }
        if !row.enabled {
            let label = text(cx, "Disabled", "已禁用");
            labels.push(label.into());
            heading = heading.child(badge::tag(
                format!("configuration-disabled:{}", row.name),
                label,
                theme::Tone::Neutral,
                cx,
            ));
        }
        let unknown = text(cx, "Unspecified", "未指定");
        let host = crate::runtime::profiles::safe_host(&row.base_url);
        let mut fields = div().flex().flex_col().gap(px(3.)).w_full();
        for (label, value) in [
            (
                text(cx, "Provider", "供应商"),
                row.provider.as_deref().unwrap_or(unknown),
            ),
            (text(cx, "Host", "主机"), host.as_deref().unwrap_or(unknown)),
            (
                text(cx, "Model", "模型"),
                row.model.as_deref().unwrap_or(unknown),
            ),
            (text(cx, "Auth", "认证"), auth_label(row, cx)),
        ] {
            fields = fields.child(
                div()
                    .flex()
                    .items_start()
                    .w_full()
                    .gap(px(8.))
                    .text_size(theme::text_size(13.))
                    .child(
                        div()
                            .flex_shrink_0()
                            .w(theme::text_size(54.))
                            .text_size(theme::text_size(12.))
                            .text_color(cx.theme().muted_foreground)
                            .child(label),
                    )
                    .child(div().flex_1().min_w_0().child(value.to_string())),
            );
        }
        let name = row.name.clone();
        let body = CardButton::new(format!("configuration:{}", row.name))
            .accessibility_label(labels.join(", "))
            .aria_selected(selected)
            .disabled(disabled)
            .track_focus(&preview_focus)
            .tab_stop(!disabled)
            .border_2()
            .border_color(transparent_black())
            .rounded(px(10.))
            .cursor_pointer()
            .focus_visible(|s| s.border_color(cx.theme().ring))
            .hover(|s| s.bg(cx.theme().muted))
            .w_full()
            .flex_1()
            .flex_col()
            .items_start()
            .justify_start()
            .line_height(gpui_kit::relative(1.25))
            .p(px(10.))
            .gap(px(8.))
            .on_key_down(cx.listener(move |this, event: &KeyDownEvent, window, cx| {
                this.move_focus(index, false, &event.keystroke.key, window, cx);
            }))
            .on_click(cx.listener(move |this, _, window, cx| this.preview(&name, window, cx)))
            .child(heading)
            .child(fields);
        let tags = row.tags.as_deref().unwrap_or_default();
        let label = if row.enabled {
            text(cx, "Apply", "应用")
        } else {
            text(cx, "Enable and apply", "启用并应用")
        };
        let name = row.name.clone();
        let enable = !row.enabled;
        let keyboard_target = name.clone();
        let action = CardButton::new(format!("configuration-apply:{}", row.name))
            .accessibility_label(format!("{label} {}", row.name))
            .track_focus(&apply_focus)
            .disabled(disabled)
            .tab_stop(!disabled)
            .a11y_synthetic_children(move |t| {
                if disabled {
                    t.parent_node().set_disabled()
                }
            })
            .rounded(px(8.))
            .border_2()
            .border_color(transparent_black())
            .focus_visible(|s| s.border_color(cx.theme().ring))
            .hover(|s| s.bg(cx.theme().muted))
            .cursor_pointer()
            .px(px(7.))
            .py(px(5.))
            .text_size(theme::text_size(13.))
            .text_color(if disabled {
                cx.theme().muted_foreground
            } else {
                cx.theme().link
            })
            .on_key_down(cx.listener(move |this, event: &KeyDownEvent, window, cx| {
                let key = event.keystroke.key.as_str();
                if matches!(key, "enter" | "space") && !event.keystroke.modifiers.modified() {
                    // A virtual band can be redrawn between native key-down and key-up.
                    // Commit once on key-down and suppress the synthesized key-up click.
                    window.prevent_default();
                    cx.stop_propagation();
                    if !event.is_held && !disabled {
                        this.tools.update(cx, |tools, cx| {
                            tools.apply_profile(keyboard_target.clone(), enable, cx)
                        });
                    }
                } else {
                    this.move_focus(index, true, key, window, cx);
                }
            }))
            .on_click(cx.listener(move |this, _, _, cx| {
                this.tools
                    .update(cx, |t, cx| t.apply_profile(name.clone(), enable, cx));
            }))
            .child(label);
        let name = row.name.clone();
        let mut tag_labels = tags.iter().take(2).cloned().collect::<Vec<_>>();
        if tags.len() > 2 {
            tag_labels.push(format!("+{}", tags.len() - 2));
        }
        let footer = div()
            .flex()
            .items_center()
            .flex_wrap()
            .justify_between()
            .gap(px(4.))
            .px(px(10.))
            .pb(px(8.))
            .child(
                div()
                    .id(format!("configuration-tags:{}", row.name))
                    .min_w_0()
                    .flex_1()
                    .text_size(theme::text_size(12.))
                    .text_color(cx.theme().muted_foreground)
                    .cursor_pointer()
                    .on_click(
                        cx.listener(move |this, _, window, cx| this.preview(&name, window, cx)),
                    )
                    .child(tag_labels.join(" · ")),
            )
            .child(action);
        div()
            .min_w_0()
            .flex()
            .flex_col()
            .rounded(px(12.))
            .border_1()
            .border_color(if selected {
                cx.theme().muted_foreground
            } else {
                cx.theme().border
            })
            .bg(if selected {
                cx.theme().secondary
            } else {
                cx.theme().background
            })
            .child(body)
            .child(footer)
            .into_any_element()
    }
}
