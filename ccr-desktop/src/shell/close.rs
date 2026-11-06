//! Main-window close guard. Cleanup remains owned by the app's quit/final join.
use crate::locales::text;
use gpui_kit::component::{
    ActiveTheme, WindowExt,
    button::{Button, ButtonVariants},
    dialog::{Cancel, DialogAction, DialogClose, DialogFooter},
};
use gpui_kit::{
    App, AppContext, InteractiveElement, KeyBinding, ParentElement, Styled, Window, div,
};
use std::{cell::Cell, rc::Rc};

gpui_kit::actions!(ccr, [Quit]);

#[derive(Clone)]
pub(super) struct CloseGuard {
    pending: Rc<Cell<bool>>,
    pub confirm: Rc<Cell<bool>>,
}

pub(super) fn install(window: &Window, cx: &mut App) -> CloseGuard {
    // Kit's Dialog Enter binding otherwise confirms even while Cancel is focused.
    cx.bind_keys([KeyBinding::new("enter", Cancel, Some("CcrQuitCancel"))]);
    cx.bind_keys([KeyBinding::new("secondary-q", Quit, None)]);
    cx.set_menus(vec![gpui_kit::Menu::new("CCR").items([
        gpui_kit::MenuItem::action(text(cx, "Quit CCR", "退出 CCR"), Quit),
    ])]);
    let guard = CloseGuard {
        pending: Rc::new(Cell::new(false)),
        confirm: Rc::new(Cell::new(true)),
    };
    let window_guard = guard.clone();
    window.on_window_should_close(cx, move |window, cx| window_guard.should_close(window, cx));
    let action_guard = guard.clone();
    let handle = window.window_handle();
    cx.on_action(move |_: &Quit, cx| {
        let guard = action_guard.clone();
        cx.defer(move |cx| {
            let _ = cx.update_window(handle, |_, window, cx| guard.request(window, cx));
        });
    });
    guard
}

impl CloseGuard {
    pub fn request(&self, window: &mut Window, cx: &mut App) {
        if self.should_close(window, cx) {
            window.remove_window();
        }
    }

    fn should_close(&self, window: &mut Window, cx: &mut App) -> bool {
        if !self.confirm.get() {
            return true;
        }
        if self.pending.replace(true) {
            return false;
        }
        let pending = self.pending.clone();
        let opening = Rc::new(Cell::new(true));
        let initial_focus = Cell::new(true);
        let quit_focus = cx.focus_handle();
        window.open_alert_dialog(cx, move |dialog, window, cx| {
            if initial_focus.replace(false) {
                let active = opening.clone();
                let focus = window.focused(cx);
                // Enter Cancel only after this opening's buttons have mounted.
                // An old callback must not steal focus after dismissal or input.
                window.on_next_frame(move |window, cx| {
                    if active.get() && focus.is_some_and(|focus| focus.is_focused(window)) {
                        window.focus_next(cx);
                    }
                });
            }
            let confirmed = pending.clone();
            let dismissed = pending.clone();
            let opening = opening.clone();
            let permitted = quit_focus.clone();
            let clicked = quit_focus.clone();
            dialog
                .title(text(cx, "Quit CCR?", "退出 CCR？"))
                .description(text(
                    cx,
                    "Unsaved changes will be lost. The app will request cancellation of running tasks and wait for them to finish.",
                    "未保存的更改将丢失。退出时会请求取消运行中的任务，并等待它们结束。",
                ))
                .footer(
                    DialogFooter::new()
                        .child(div().key_context("CcrQuitCancel").child(
                            DialogClose::new().child(
                                Button::new("cancel")
                                    .label(text(cx, "Keep working", "继续使用")),
                            ),
                        ))
                        .child(div().track_focus(&quit_focus).tab_stop(false).child(DialogAction::new().child(
                            Button::new("ok").label(text(cx, "Quit", "退出")).primary()
                                .on_click(move |_, window, cx| window.focus(&clicked, cx)),
                        ))),
                )
                .bg(cx.theme().popover)
                .on_ok(move |_, window, cx| {
                    // A generic Enter can arrive while the modal root still has
                    // focus, before Cancel's first frame. That is a safe cancel.
                    if confirmed.replace(false) && permitted.contains_focused(window, cx) {
                        // Removing the last window follows the existing app quit
                        // observer and final RuntimeOwner join; no early exit.
                        window.remove_window();
                    }
                    true
                })
                .on_close(move |_, _, _| {
                    opening.set(false);
                    dismissed.set(false);
                })
        });
        false
    }
}
