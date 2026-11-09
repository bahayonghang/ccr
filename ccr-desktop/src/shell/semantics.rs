//! Decorate the existing Button node; this builder is never rendered.
use gpui_kit::component::{
    ActiveTheme, Disableable, Selectable,
    button::{Button, ButtonRounded, ButtonVariants},
};
use gpui_kit::{App, InteractiveElement, Interactivity, StatefulInteractiveElement, Styled};

struct Builder<T>(T);
impl<T: InteractiveElement> InteractiveElement for Builder<T> {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.0.interactivity()
    }
}
impl<T: InteractiveElement> StatefulInteractiveElement for Builder<T> {}

pub fn button(button: Button, disabled: bool, busy: bool, cx: &App) -> Button {
    let mut button = button.rounded(ButtonRounded::Size(gpui_kit::px(crate::theme::PILL_RADIUS)));
    if disabled {
        button = button
            .text_color(cx.theme().muted_foreground)
            .bg(cx.theme().muted)
            .border_color(cx.theme().input);
    }
    states(button, disabled, busy)
}

/// A ghost header or toolbar action. It has the same disabled and busy semantics
/// as [`button`], but a disabled ghost keeps its transparent background and only
/// mutes its ink, so it never reads as more clickable than an enabled one.
///
/// 用于页头、工具栏和翻页的次要动作；全局 `button` 不变。
pub fn ghost_button(button: Button, disabled: bool, busy: bool, cx: &App) -> Button {
    let mut button = button
        .ghost()
        .rounded(ButtonRounded::Size(gpui_kit::px(crate::theme::PILL_RADIUS)));
    if disabled {
        button = button
            .text_color(cx.theme().muted_foreground)
            .bg(gpui_kit::transparent_black())
            .border_color(gpui_kit::transparent_black());
    }
    states(button, disabled, busy)
}

// 禁用与忙碌的交互和无障碍语义：两种按钮外观共用。
fn states(button: Button, disabled: bool, busy: bool) -> Button {
    Builder(button.disabled(disabled).tab_stop(!disabled))
        .a11y_synthetic_children(move |tree| {
            let node = tree.parent_node();
            if disabled {
                node.set_disabled();
            } else {
                node.clear_disabled();
            }
            if busy {
                node.set_busy();
            } else {
                node.clear_busy();
            }
        })
        .0
}

/// Segmented options retain Kit keyboard/focus behavior but own their persistent
/// selection colors rather than relying on the ghost variant's active palette.
/// 选中项是浮起的实色 pill（轻阴影 + 主色 semibold），未选中项透明无边框；
/// 禁用态保留不透明 control 边界与 muted 色，选中样式不得覆盖禁用色。
pub fn tab(button: Button, selected: bool, disabled: bool, cx: &App) -> Button {
    let colors = crate::theme::tab_colors(cx.theme().mode);
    let raised = selected && !disabled;
    let button = button
        .selected(selected)
        .toggled(selected)
        .border_1()
        .rounded(gpui_kit::px(crate::theme::PILL_RADIUS))
        .bg(if disabled {
            cx.theme().muted
        } else if selected {
            colors.selected
        } else {
            gpui_kit::transparent_black()
        })
        .text_color(if disabled {
            cx.theme().muted_foreground
        } else if selected {
            colors.selected_foreground
        } else {
            colors.foreground
        })
        .border_color(if disabled {
            cx.theme().input
        } else {
            gpui_kit::transparent_black()
        })
        .font_weight(if selected {
            gpui_kit::FontWeight::SEMIBOLD
        } else {
            gpui_kit::FontWeight::NORMAL
        });
    if raised {
        button.shadow(vec![colors.selected_shadow])
    } else {
        button
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use gpui_kit::test::TestWindowExt;
    use gpui_kit::{
        AppContext, Context, IntoElement, ParentElement, Render, Styled, TestAppContext, Window,
        WindowOptions, div,
    };
    use std::{cell::Cell, rc::Rc};

    struct Harness {
        clicks: Rc<Cell<usize>>,
    }

    impl Render for Harness {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let clicks = self.clicks.clone();
            div()
                .flex()
                .gap(gpui_kit::px(8.))
                .child(ghost_button(
                    Button::new("ghost-on").label("Reload"),
                    false,
                    false,
                    cx,
                ))
                .child(ghost_button(
                    Button::new("ghost-off")
                        .label("Reload")
                        .on_click(move |_, _, _| clicks.set(clicks.get() + 1)),
                    true,
                    true,
                    cx,
                ))
        }
    }

    #[gpui_kit::test]
    fn disabled_ghost_button_keeps_its_disabled_semantics(cx: &mut TestAppContext) {
        let clicks = Rc::new(Cell::new(0));
        let shared = clicks.clone();
        let (handle, _) = cx.update(|cx| {
            gpui_kit::init(cx);
            gpui_kit::open_window(WindowOptions::default(), cx, |_, cx| {
                cx.new(|_| Harness { clicks: shared })
            })
            .unwrap()
        });
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            // 透明底只改外观：禁用按钮没有焦点绑定、点击无效。
            assert_eq!(window.find("ghost-off").focused(), None);
            assert_ne!(window.find("ghost-on").disabled(), Some(true));
            assert_eq!(window.find("ghost-on").focused(), Some(false));
            window.click("ghost-off", cx);
            window.remove_window();
        })
        .unwrap();
        assert_eq!(clicks.get(), 0);
    }
}
