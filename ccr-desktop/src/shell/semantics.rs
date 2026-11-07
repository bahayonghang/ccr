//! Decorate the existing Button node; this builder is never rendered.
use gpui_kit::component::{
    ActiveTheme, Disableable, Selectable,
    button::{Button, ButtonRounded},
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
pub fn tab(button: Button, selected: bool, disabled: bool, cx: &App) -> Button {
    let colors = crate::theme::tab_colors(cx.theme().mode);
    button
        .selected(selected)
        .toggled(selected)
        .border_1()
        .rounded(gpui_kit::px(crate::theme::TAB_RADIUS))
        .bg(if disabled {
            cx.theme().muted
        } else if selected {
            colors.selected
        } else {
            colors.background
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
        } else if selected {
            colors.border
        } else {
            gpui_kit::transparent_black()
        })
        .font_weight(if selected {
            gpui_kit::FontWeight::SEMIBOLD
        } else {
            gpui_kit::FontWeight::NORMAL
        })
}
