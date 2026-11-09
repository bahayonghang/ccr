use crate::features::insights::{InsightsSnapshot, InsightsView};
use crate::{locales::text, shell::semantics, theme};
use gpui_kit::component::{
    ActiveTheme,
    button::{Button, ButtonVariants},
};
use gpui_kit::{
    Context, InteractiveElement, IntoElement, ParentElement, StatefulInteractiveElement, Styled,
    div, px,
};

pub(super) fn transcript(
    view: &mut InsightsView,
    window: &mut gpui_kit::Window,
    cx: &mut Context<InsightsView>,
) -> impl IntoElement {
    let operations = view.render_session_operations(window, cx);
    let Some(InsightsSnapshot::Detail(detail)) = view.snapshot.as_deref() else {
        return div();
    };
    let mut messages = div()
        .id("session-transcript")
        .flex()
        .flex_col()
        .gap(px(theme::CONTROL_GAP))
        .flex_1()
        .overflow_y_scroll();
    for message in &detail.page.messages {
        messages = messages.child(
            div()
                .p(px(theme::SECTION_GAP))
                .bg(cx.theme().background)
                .border_1()
                .border_color(cx.theme().border)
                .rounded(px(theme::INPUT_RADIUS))
                .flex()
                .flex_col()
                .gap(px(theme::SELECTION_RADIUS))
                .child(
                    div()
                        .text_size(px(theme::META_SIZE))
                        .text_color(if message.role == "assistant" {
                            gpui_kit::Hsla::from(gpui_kit::rgb(
                                theme::palette(cx.theme().mode).output,
                            ))
                        } else {
                            cx.theme().primary
                        })
                        .child(message.role.clone()),
                )
                .child(message.content.clone()),
        );
    }
    div()
        .flex()
        .flex_col()
        .gap(px(theme::CONTROL_GAP))
        .flex_1()
        .min_h_0()
        .child(operations)
        .child(
            div()
                .flex()
                .gap(px(theme::CONTROL_GAP))
                .child(semantics::button(
                    Button::new("insights-back")
                        .label(text(cx, "Back to list", "返回列表"))
                        .ghost()
                        .on_click(cx.listener(|this, _, _, cx| this.refresh(cx))),
                    view.loading,
                    false,
                    cx,
                ))
                .child(semantics::button(
                    Button::new("insights-next")
                        .label(text(cx, "Older messages", "更早的消息"))
                        .ghost()
                        .on_click(cx.listener(|this, _, _, cx| this.next(cx))),
                    view.loading || !detail.page.has_older,
                    false,
                    cx,
                )),
        )
        .child(messages)
}
