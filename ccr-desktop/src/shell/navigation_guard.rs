//! Scoped navigation decisions preserve drafts until the user explicitly discards them.
use super::*;
use gpui_kit::component::FocusTrapElement;

pub(super) struct Decision {
    pub destination: Destination,
    pub lens: Lens,
    pub generation: u64,
    pub enter: bool,
    pub return_focus: Option<FocusHandle>,
}
impl Shell {
    fn cancel_navigation(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(decision) = self.navigation_decision.take()
            && let Some(focus) = decision.return_focus
        {
            window.focus(&focus, cx);
        }
        self.navigation_generation += 1;
        cx.notify();
    }
    pub(super) fn confirm_navigation(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        if let Some(decision) = &mut self.navigation_decision
            && std::mem::take(&mut decision.enter)
        {
            let generation = decision.generation;
            let weak = cx.weak_entity();
            window.on_next_frame(move |window, cx| {
                let _ = weak.update(cx, |this, cx| {
                    if this
                        .navigation_decision
                        .as_ref()
                        .is_some_and(|decision| decision.generation == generation)
                    {
                        window.focus(&this.leave_focus, cx);
                        window.focus_next(cx);
                    }
                });
            });
        }
        div()
            .id("profile-navigation-confirmation")
            .test_support()
            .role(gpui_kit::Role::AlertDialog)
            .aria_label(text(
                cx,
                "Discard unsaved profile draft?",
                "丢弃未保存的方案草稿？",
            ))
            .p(px(24.))
            .flex()
            .flex_col()
            .gap(px(16.))
            .track_focus(&self.leave_focus)
            .tab_stop(false)
            .on_key_down(
                cx.listener(|this, event: &gpui_kit::KeyDownEvent, window, cx| {
                    if event.keystroke.key == "escape" {
                        this.cancel_navigation(window, cx);
                        cx.stop_propagation();
                    }
                }),
            )
            .child(text(
                cx,
                "The draft has unsaved changes. Stay to keep editing or discard it and leave.",
                "草稿有未保存的修改。可继续编辑，或丢弃后离开。",
            ))
            .child(
                div()
                    .flex()
                    .gap(px(8.))
                    .child(semantics::ghost_button(
                        Button::new("profile-navigation-cancel")
                            .label(text(cx, "Keep editing", "继续编辑"))
                            .ghost()
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.cancel_navigation(window, cx)
                            })),
                        false,
                        false,
                        cx,
                    ))
                    .child(semantics::button(
                        Button::new("profile-navigation-discard")
                            .label(text(cx, "Discard and leave", "丢弃并离开"))
                            .on_click(cx.listener(|this, _, window, cx| {
                                if let Some(decision) = this.navigation_decision.take() {
                                    this.configurations
                                        .update(cx, |view, cx| view.leave(window, cx));
                                    this.go(decision.destination, decision.lens, window, cx);
                                }
                            })),
                        false,
                        false,
                        cx,
                    ))
                    .focus_trap("profile-navigation-trap", &self.leave_focus),
            )
            .into_any_element()
    }
}
