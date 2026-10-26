use gpui_kit::component::{
    ActiveTheme,
    input::{Input, InputEvent, InputState},
};
use gpui_kit::{
    AppContext, Context, Entity, IntoElement, ParentElement, Render, SharedString, Styled,
    Subscription, Window, div, px,
};

use crate::theme;

/// Fixture-only native feasibility probe. It never locates or reads user data.
pub struct Probe {
    input: Entity<InputState>,
    preview: SharedString,
    _input_subscription: Subscription,
}

impl Probe {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input =
            cx.new(|cx| InputState::new(window, cx).placeholder("Type here / 在此输入中文"));
        let subscription = cx.subscribe_in(&input, window, |this, state, event, _, cx| {
            if matches!(event, InputEvent::Change) {
                this.preview = state.read(cx).value();
                cx.notify();
            }
        });
        Self {
            input,
            preview: "".into(),
            _input_subscription: subscription,
        }
    }
}

impl Render for Probe {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .size_full()
            .p(px(theme::CONTENT_PADDING))
            .gap(px(theme::SECTION_GAP))
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .text_size(px(theme::BODY_SIZE))
            .child(
                div()
                    .text_size(px(theme::TITLE_SIZE))
                    .child("CCR Desktop (GPUI)"),
            )
            .child("Native input probe / 原生输入探针")
            .child("Fixture only · No user configuration is loaded / 不读取用户配置")
            .child(Input::new(&self.input).id("probe-input"))
            .child(div().child("Input preview / 输入预览"))
            .child(self.preview.clone())
            .child("Use the input to verify CJK composition, selection and focus. Close the window to exit.")
    }
}
