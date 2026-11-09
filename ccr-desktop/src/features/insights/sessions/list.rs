use crate::features::insights::{InsightsSnapshot, InsightsView};
use crate::{locales::text, shell::semantics, theme};
use gpui_kit::base::TestSupportExt;
use gpui_kit::component::{
    ActiveTheme,
    button::{Button, ButtonVariants},
    input::Input,
};
use gpui_kit::{
    Context, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, div, px, uniform_list,
};

pub(super) fn search(view: &InsightsView, cx: &mut Context<InsightsView>) -> impl IntoElement {
    div()
        .flex()
        .gap(px(theme::CONTROL_GAP))
        .items_end()
        .child(
            div()
                .w(px(280.))
                .flex()
                .flex_col()
                .gap(px(theme::SELECTION_RADIUS))
                .child(
                    div()
                        .text_size(px(theme::META_SIZE))
                        .text_color(cx.theme().muted_foreground)
                        .child(text(cx, "Title", "标题")),
                )
                .child(
                    Input::new(&view.fields[6])
                        .id("insights-filter-6")
                        .aria_label(text(cx, "Session title search", "会话标题搜索")),
                ),
        )
        .child(semantics::button(
            Button::new("session-search")
                .label(text(cx, "Search", "搜索"))
                .ghost()
                .on_click(cx.listener(|this, _, _, cx| this.refresh(cx))),
            view.loading,
            false,
            cx,
        ))
}

pub(super) fn session_list(
    view: &mut InsightsView,
    cx: &mut Context<InsightsView>,
) -> impl IntoElement {
    let Some(InsightsSnapshot::Sessions { page, indexed, .. }) = view.snapshot.as_deref() else {
        return div().into_any_element();
    };
    if !indexed {
        return div()
            .flex_1()
            .p(px(theme::SECTION_GAP))
            .bg(cx.theme().background)
            .border_1()
            .border_color(cx.theme().border)
            .rounded(px(theme::INPUT_RADIUS))
            .text_color(cx.theme().muted_foreground)
            .child(text(cx, "Sessions are not indexed.", "会话尚未索引。"))
            .into_any_element();
    }
    let count = page.items.len();
    if count == 0 {
        return div()
            .flex_1()
            .p(px(theme::SECTION_GAP))
            .bg(cx.theme().background)
            .border_1()
            .border_color(cx.theme().border)
            .rounded(px(theme::INPUT_RADIUS))
            .text_color(cx.theme().muted_foreground)
            .child(text(
                cx,
                "No indexed sessions match this page.",
                "这一页没有匹配的已索引会话。",
            ))
            .into_any_element();
    }
    uniform_list(
        "session-cards",
        count,
        cx.processor(|this, range: std::ops::Range<usize>, _, cx| {
            let Some(InsightsSnapshot::Sessions { page, .. }) = this.snapshot.as_deref() else {
                return Vec::new();
            };
            range
                .filter_map(|index| page.items.get(index).map(|row| (index, row.clone())))
                .map(|(index, row)| {
                    let title = row
                        .title
                        .clone()
                        .filter(|title| !title.is_empty())
                        .unwrap_or_else(|| row.session_id.clone());
                    let when = row
                        .updated_at
                        .with_timezone(&chrono::Local)
                        .format("%m-%d %H:%M")
                        .to_string();
                    div()
                        .id(SharedString::from(format!(
                            "insights-row:{}",
                            row.archive_id
                        )))
                        .test_support()
                        .h(px(84.))
                        .w_full()
                        .px(px(theme::SECTION_GAP))
                        .flex()
                        .flex_col()
                        .justify_center()
                        .gap(px(theme::SELECTION_RADIUS))
                        .bg(cx.theme().background)
                        .border_1()
                        .border_color(cx.theme().border)
                        .rounded(px(theme::INPUT_RADIUS))
                        .hover(|style| style.bg(cx.theme().secondary))
                        .on_click(cx.listener(move |this, _, _, cx| this.select(index, cx)))
                        .child(
                            div()
                                .flex()
                                .justify_between()
                                .child(
                                    div()
                                        .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                                        .child(title),
                                )
                                .child(
                                    div()
                                        .text_size(px(theme::META_SIZE))
                                        .text_color(cx.theme().muted_foreground)
                                        .child(when),
                                ),
                        )
                        .child(
                            div()
                                .text_size(px(theme::META_SIZE))
                                .text_color(cx.theme().muted_foreground)
                                .child(format!(
                                    "{} · {} {} · {}",
                                    row.platform,
                                    row.message_count,
                                    text(cx, "messages", "条消息"),
                                    fidelity_label(&row.source_fidelity, cx),
                                )),
                        )
                        .child(
                            div()
                                .text_size(px(theme::META_SIZE))
                                .text_color(cx.theme().muted_foreground)
                                .child(if row.cwd.is_empty() {
                                    text(cx, "No project path", "没有项目路径").to_string()
                                } else {
                                    row.cwd.clone()
                                }),
                        )
                        .into_any_element()
                })
                .collect()
        }),
    )
    .flex_1()
    .min_h(px(240.))
    .into_any_element()
}

fn fidelity_label(value: &str, cx: &gpui_kit::App) -> &'static str {
    match value {
        "full" => text(cx, "complete", "完整"),
        "partial" => text(cx, "partial", "部分"),
        "locked" => text(cx, "locked", "已锁定"),
        _ => text(cx, "unknown", "未知"),
    }
}
