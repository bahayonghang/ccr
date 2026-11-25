//! Capture the invoking control by stable identity, including sources outside the account list.
use super::*;
use std::cell::Cell;

struct Origin {
    control: String,
    offset: Point<Pixels>,
}

pub(super) struct Navigation {
    detail: Option<Origin>,
    editor: Option<Origin>,
    target: String,
    pending: bool,
    mounted: Cell<bool>,
    anchor: FocusHandle,
}

impl Navigation {
    pub(super) fn new(cx: &App) -> Self {
        Self {
            detail: None,
            editor: None,
            target: String::new(),
            pending: false,
            mounted: Cell::new(false),
            anchor: cx.focus_handle().tab_stop(false),
        }
    }

    pub(super) fn clear(&mut self) {
        self.detail = None;
        self.editor = None;
        self.target.clear();
        self.pending = false;
    }
}

impl CheckinView {
    pub(super) fn origin_control(&self, key: &str, button: Button) -> impl IntoElement + use<> {
        let mut anchor = div()
            .id(format!("checkin-focus-{key}"))
            .flex_shrink_0()
            .max_w_full();
        if self.navigation.target == key {
            self.navigation.mounted.set(true);
            anchor = anchor.track_focus(&self.navigation.anchor).tab_stop(false);
        }
        anchor.child(button)
    }

    fn origin(&self, control: &str) -> Origin {
        Origin {
            control: control.to_owned(),
            offset: self.scroll.offset(),
        }
    }

    pub(super) fn focus_control(&mut self, key: &str) {
        self.navigation.target = key.to_owned();
        self.navigation.pending = true;
    }

    pub(super) fn restore_control(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.navigation.pending && !self.busy && !self.operation.busy {
            self.navigation.pending = false;
            let anchor = if self.navigation.mounted.get() {
                self.navigation.anchor.clone()
            } else {
                self.focus.clone()
            };
            window.focus(&anchor, cx);
            window.on_next_frame(move |window, cx| {
                if anchor.is_focused(window) {
                    window.focus_next(cx);
                }
            });
        }
        self.navigation.mounted.set(false);
    }

    pub(super) fn open_account_detail(&mut self, id: &str, source: &str, cx: &mut Context<Self>) {
        if self.rows().iter().any(|row| row.id == id) {
            self.editor = None;
            self.confirmation = None;
            self.cdk = None;
            self.navigation.editor = None;
            self.navigation.detail = Some(self.origin(source));
            self.selected = Some(id.to_owned());
            self.scroll.set_offset(Default::default());
            self.focus_control("back");
        } else {
            self.focus_control(source);
        }
        cx.notify();
    }

    pub(super) fn close_detail(&mut self) {
        self.editor = None;
        self.confirmation = None;
        self.selected = None;
        self.cdk = None;
        if let Some(origin) = self.navigation.detail.take() {
            self.scroll.set_offset(origin.offset);
            self.focus_control(&origin.control);
        }
    }

    pub(super) fn open_account_credentials(
        &mut self,
        id: &str,
        source: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.rows().iter().any(|row| row.id == id) {
            self.remember_editor_source(source);
            self.edit(
                operations::EditorKind::Account(Some(id.to_owned())),
                window,
                cx,
            );
        } else {
            self.focus_control(source);
            cx.notify();
        }
    }

    pub(super) fn remember_editor_source(&mut self, source: &str) {
        self.navigation.editor = Some(self.origin(source));
    }

    pub(super) fn show_created_account(&mut self, id: &str) {
        let origin = self.navigation.editor.take();
        self.close_editor();
        self.navigation.detail = origin;
        self.selected = Some(id.to_owned());
        self.scroll.set_offset(Default::default());
        self.focus_control("back");
    }

    pub(super) fn close_editor(&mut self) {
        self.editor = None;
        self.catalog = false;
        if let Some(origin) = self.navigation.editor.take() {
            self.scroll.set_offset(origin.offset);
            self.focus_control(&origin.control);
        }
    }
}
