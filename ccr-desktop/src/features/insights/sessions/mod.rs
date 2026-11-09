//! Agent sessions page shown from the sidebar. The workbench renderer stays for tests.
//!
//! 版式：页头 → 提供方 chip → 工具栏 → 操作反馈 → 主体。主体在窗口宽度 ≥ `SPLIT_WIDTH`
//! 时为列表 | pane（3:2），否则单列：废纸篓打开时显示在列表上方，打开详情时 drill-in。
mod header;
mod list;
mod pane;
mod providers;
mod state;
#[cfg(test)]
mod tests;

pub(super) use state::SessionDetailSlot;

use super::{InsightsSnapshot, InsightsView};
use crate::theme;
use gpui_kit::base::TestSupportExt;
use gpui_kit::component::ActiveTheme;
use gpui_kit::{
    AnyElement, App, Context, Div, InteractiveElement, IntoElement, ParentElement, Stateful,
    StatefulInteractiveElement, Styled, div, px,
};

/// Shortest body before the page scrolls instead of squeezing the list.
const BODY_MIN_HEIGHT: f32 = 320.;

pub(super) fn sessions(
    view: &mut InsightsView,
    window: &mut gpui_kit::Window,
    cx: &mut Context<InsightsView>,
) -> impl IntoElement {
    list::sync_placeholder(view, window, cx);
    let width = window.viewport_size().width;
    let split = width >= px(theme::SPLIT_WIDTH);
    let root = div()
        .id("session-page")
        .test_support()
        .size_full()
        .flex()
        .flex_col()
        .min_h_0()
        .overflow_y_scroll()
        .p(px(theme::content_padding(width)))
        .gap(px(theme::SECTION_GAP))
        .child(header::header(view, cx));
    // 窄窗 drill-in 到会话详情时，列表被隐藏，提供方 chip 与工具栏只筛选看不见的
    // 列表，不再渲染，把高度让给消息。废纸篓打开时列表仍可见，二者保留。
    let drilled = !split && view.detail.archive_id.is_some() && !view.session_ops_trash_open();
    let root = if drilled {
        root
    } else {
        root.child(providers::providers(view, !split, cx))
            .child(list::toolbar(view, cx))
    };
    let root = match view.render_session_feedback(cx) {
        Some(feedback) => root.child(feedback),
        None => root,
    };
    root.child(body(view, split, window, cx))
}

fn body(
    view: &mut InsightsView,
    split: bool,
    window: &mut gpui_kit::Window,
    cx: &mut Context<InsightsView>,
) -> AnyElement {
    let body = div()
        .id("session-body")
        .test_support()
        .flex_1()
        .min_h(px(BODY_MIN_HEIGHT))
        .flex()
        .gap(px(theme::SECTION_GAP));
    let trash = view.session_ops_trash_open();
    let detail = view.detail.archive_id.is_some();
    // 没有可选的会话、也没有打开的内容时，宽窗只放列表列，不并排一块空 pane。
    let beside = split && (trash || detail || has_rows(view));
    if beside {
        let list = list::list_column(view, true, cx);
        let pane = pane::pane_column(view, true, window, cx);
        return body.flex_row().child(list).child(pane).into_any_element();
    }
    if trash {
        // 窄窗：废纸篓按内容高度排在列表上方，列表保留固定高度；整页纵向滚动。
        // 两者平分主体会让废纸篓行落到面板视口之外。
        let pane = pane::pane_column(view, false, window, cx);
        let list = list::list_column(view, false, cx);
        return body
            .flex_none()
            .min_h_0()
            .flex_col()
            .child(div().flex_shrink_0().flex().flex_col().child(pane))
            .child(
                div()
                    .flex_shrink_0()
                    .h(px(BODY_MIN_HEIGHT))
                    .flex()
                    .flex_col()
                    .child(list),
            )
            .into_any_element();
    }
    if detail {
        let pane = pane::pane_column(view, false, window, cx);
        return body.flex_col().child(pane).into_any_element();
    }
    let list = list::list_column(view, false, cx);
    body.flex_col().child(list).into_any_element()
}

fn has_rows(view: &InsightsView) -> bool {
    view.error.is_none()
        && matches!(
            view.snapshot.as_deref(),
            Some(InsightsSnapshot::Sessions { page, indexed: true, .. }) if !page.items.is_empty()
        )
}

/// 列宽：并排时按份额分配剩余宽度并保留最小宽；单列时占满整列。
///
/// 列本身不带内边距与描边：flex 基准为 0 时，taffy 会把基准抬到条目自身的
/// padding + border，比例就不再是 3:2。面板按内容或 pane 类型决定高度。
fn column_size(panel: AnyElement, beside: bool, share: f32, min: f32) -> Div {
    let column = div().min_h_0().flex().flex_col();
    let column = if beside {
        column
            .flex_grow(share)
            .flex_shrink(1.)
            .flex_basis(px(0.))
            .min_w(px(min))
    } else {
        column.flex_1().w_full()
    };
    column.child(panel)
}

/// 实色阅读面板：列表与 pane 共用同一描边、圆角。
fn reading_panel(id: &'static str, cx: &App) -> gpui_kit::base::ObservedElement<Stateful<Div>> {
    div()
        .id(id)
        .test_support()
        .min_h_0()
        .flex()
        .flex_col()
        .bg(cx.theme().background)
        .border_1()
        .border_color(cx.theme().border)
        .rounded(px(theme::PANEL_RADIUS))
}
