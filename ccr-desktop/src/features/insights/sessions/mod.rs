//! Session page shown from the sidebar. The workbench renderer stays for tests.
mod header;
mod list;
mod pane;
mod providers;

use super::{InsightsSnapshot, InsightsView};
use crate::{runtime::insights::RefreshStatus, theme};
use gpui_kit::base::TestSupportExt;
use gpui_kit::{
    Context, InteractiveElement, IntoElement, ParentElement, StatefulInteractiveElement, Styled,
    div, px,
};
use header::{header, status_line};
use list::{search, session_list};
use pane::transcript;
use providers::providers;

pub(super) fn sessions(
    view: &mut InsightsView,
    window: &mut gpui_kit::Window,
    cx: &mut Context<InsightsView>,
) -> impl IntoElement {
    let running = matches!(view.requests.refresh_status(), RefreshStatus::Running(_));
    let root = div()
        .id("session-page")
        .test_support()
        .size_full()
        .flex()
        .flex_col()
        .min_h_0()
        .overflow_y_scroll()
        .p(px(theme::CONTENT_PADDING))
        .gap(px(theme::SECTION_GAP))
        .child(header(view, running, cx))
        .child(status_line(view, cx))
        .child(providers(view, cx));
    if view.session_confirmation_pending() {
        return root.child(view.render_session_operations(window, cx));
    }
    if matches!(view.snapshot.as_deref(), Some(InsightsSnapshot::Detail(_))) {
        return root.child(transcript(view, window, cx));
    }
    root.child(view.render_session_operations(window, cx))
        .child(search(view, cx))
        .child(session_list(view, cx))
}
