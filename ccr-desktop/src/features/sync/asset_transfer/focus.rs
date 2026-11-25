use super::SyncView;
use crate::theme;
use gpui_kit::{
    Bounds, Context, Div, FocusHandle, InteractiveElement, IntoElement, ParentElement, Pixels,
    Stateful, Subscription, Window, div, px,
};
use std::{cell::Cell, rc::Rc};

#[derive(Clone, Copy)]
pub(super) enum Control {
    Upload,
    Download,
    Sync,
    Passphrase,
    Overwrite,
    PlaintextImport,
    ChangeDirection,
    Cancel,
    Confirm,
}

pub(super) struct TransferFocus([ControlFocus; Control::Confirm as usize + 1]);

struct ControlFocus {
    handle: FocusHandle,
    bounds: Rc<Cell<Option<Bounds<Pixels>>>>,
    _reveal: Subscription,
}

impl TransferFocus {
    pub(super) fn new(window: &mut Window, cx: &mut Context<SyncView>) -> Self {
        Self(std::array::from_fn(|_| {
            let handle = cx.focus_handle().tab_stop(false);
            let bounds = Rc::new(Cell::new(None::<Bounds<Pixels>>));
            let measured = bounds.clone();
            let reveal = cx.on_focus_in(&handle, window, move |this, _, cx| {
                let Some(control) = measured.get() else {
                    return;
                };
                let scroll = &this.folder_focus.scroll;
                let viewport = scroll.bounds();
                // Leave the focus ring clear of the page's clipping edge.
                let margin = px(theme::CONTROL_GAP);
                let top = viewport.top() + margin;
                let bottom = viewport.bottom() - margin;
                let delta = if control.top() < top {
                    top - control.top()
                } else if control.bottom() > bottom {
                    bottom - control.bottom()
                } else {
                    return;
                };
                let mut offset = scroll.offset();
                let next = (offset.y + delta).clamp(-scroll.max_offset().y, px(0.));
                if offset.y != next {
                    offset.y = next;
                    scroll.set_offset(offset);
                    cx.notify();
                }
            });
            ControlFocus {
                handle,
                bounds,
                _reveal: reveal,
            }
        }))
    }

    pub(super) fn track(&self, control: Control, element: impl IntoElement) -> Stateful<Div> {
        let focus = &self.0[control as usize];
        let bounds = focus.bounds.clone();
        div()
            .on_children_prepainted(move |children, _, _| bounds.set(children.first().copied()))
            .id(&focus.handle)
            .track_focus(&focus.handle)
            .tab_stop(false)
            .child(element)
    }
}
