//! Session page shown from the sidebar. The workbench renderer stays for tests.
use super::{InsightsSnapshot, InsightsView};
use crate::{
    locales::{service_error, text},
    runtime::insights::RefreshStatus,
    shell::semantics,
    theme,
};
use ccr_store::sessions::ProviderAvailability;
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

pub(super) fn sessions(
    view: &mut InsightsView,
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
    if matches!(view.snapshot.as_deref(), Some(InsightsSnapshot::Detail(_))) {
        return root.child(transcript(view, cx));
    }
    root.child(search(view, cx)).child(session_list(view, cx))
}

fn header(_view: &InsightsView, running: bool, cx: &mut Context<InsightsView>) -> impl IntoElement {
    div()
        .flex()
        .justify_between()
        .items_start()
        .gap(px(theme::SECTION_GAP))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(theme::CONTROL_GAP))
                .child(
                    div()
                        .id("session-title")
                        .test_support()
                        .aria_label(text(cx, "Agent sessions", "Agent 会话"))
                        .text_size(px(theme::TITLE_SIZE))
                        .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                        .child(text(cx, "Agent sessions", "Agent 会话")),
                )
                .child(
                    div()
                        .text_size(px(theme::META_SIZE))
                        .text_color(cx.theme().muted_foreground)
                        .child(text(
                            cx,
                            "Indexed summaries from this computer. Refresh reads local transcripts and does not invent sessions.",
                            "这里是本机索引出的会话摘要。刷新会读取本地记录，不会编造会话。",
                        )),
                ),
        )
        .child(semantics::button(
            Button::new("insights-index-refresh")
                .label(if running {
                    text(cx, "Indexing", "正在索引")
                } else {
                    text(cx, "Refresh index", "刷新索引")
                })
                .on_click(cx.listener(|this, _, _, cx| this.start_archive_refresh(cx))),
            running,
            running,
            cx,
        ))
}

fn status_line(view: &InsightsView, cx: &mut Context<InsightsView>) -> impl IntoElement {
    let message = if view.loading {
        text(cx, "Reading the archive.", "正在读取归档。").to_string()
    } else if let Some(error) = view.error {
        service_error(cx, error).to_string()
    } else {
        match view.requests.refresh_status() {
            RefreshStatus::Running(_) => text(
                cx,
                "Indexing local transcripts. The list stays empty until that finishes.",
                "正在索引本机记录。完成之前列表是空的。",
            )
            .to_string(),
            RefreshStatus::Failed(_, error) => service_error(cx, error.message()).to_string(),
            RefreshStatus::Interrupted(_) => text(
                cx,
                "Indexing stopped before a result. Refresh to try again.",
                "索引中断了，还没有结果。可以再刷新一次。",
            )
            .to_string(),
            RefreshStatus::Finished(_, report) => {
                let indexed = view
                    .snapshot
                    .as_ref()
                    .and_then(|snapshot| match snapshot.as_ref() {
                        InsightsSnapshot::Sessions { page, .. } => Some(page.items.len()),
                        _ => None,
                    })
                    .unwrap_or(0);
                if cx.global::<crate::locales::Language>() == &crate::locales::Language::Chinese {
                    format!(
                        "索引已结束：发现 {}，写入 {}，本页 {} 条",
                        report.total.discovered, report.total.upserted, indexed
                    )
                } else {
                    format!(
                        "Index finished: discovered {}, written {}, {} on this page",
                        report.total.discovered, report.total.upserted, indexed
                    )
                }
            }
            RefreshStatus::Idle => {
                if matches!(view.snapshot.as_deref(), Some(InsightsSnapshot::Sessions { page, .. }) if page.items.is_empty())
                {
                    text(
                        cx,
                        "The archive has no session summaries yet.",
                        "归档里还没有会话摘要。",
                    )
                    .to_string()
                } else {
                    text(cx, "Archive ready.", "归档已就绪。").to_string()
                }
            }
        }
    };
    div()
        .id("session-status")
        .test_support()
        .role(gpui_kit::Role::Status)
        .aria_label(message.clone())
        .text_size(px(theme::META_SIZE))
        .text_color(cx.theme().muted_foreground)
        .child(message)
}

fn providers(view: &InsightsView, cx: &mut Context<InsightsView>) -> impl IntoElement {
    let Some(InsightsSnapshot::Sessions { statuses, .. }) = view.snapshot.as_deref() else {
        return div();
    };
    let mut row = div().flex().flex_wrap().gap(px(theme::CONTROL_GAP));
    for status in statuses {
        let (label, tone_ok) = match status.availability {
            ProviderAvailability::Available => (
                format!("{} {}", status.agent.as_str(), text(cx, "readable", "可读")),
                true,
            ),
            ProviderAvailability::NoData => (
                format!(
                    "{} {}",
                    status.agent.as_str(),
                    text(cx, "no files", "无文件")
                ),
                false,
            ),
            ProviderAvailability::NotInstalled => (
                format!(
                    "{} {}",
                    status.agent.as_str(),
                    text(cx, "not installed", "未安装")
                ),
                false,
            ),
            ProviderAvailability::Error => (
                format!(
                    "{} {}",
                    status.agent.as_str(),
                    text(cx, "unreadable", "不可读")
                ),
                false,
            ),
        };
        row = row.child(
            div()
                .px(px(theme::CONTROL_GAP))
                .h(px(28.))
                .flex()
                .items_center()
                .rounded(px(theme::PILL_RADIUS))
                .bg(cx.theme().secondary)
                .text_size(px(theme::META_SIZE))
                .text_color(if tone_ok {
                    cx.theme().primary
                } else {
                    cx.theme().muted_foreground
                })
                .child(label),
        );
    }
    row
}

fn search(view: &InsightsView, cx: &mut Context<InsightsView>) -> impl IntoElement {
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

fn session_list(view: &mut InsightsView, cx: &mut Context<InsightsView>) -> impl IntoElement {
    let count = match view.snapshot.as_deref() {
        Some(InsightsSnapshot::Sessions { page, .. }) => page.items.len(),
        _ => 0,
    };
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
                "No session summaries on this page.",
                "这一页还没有会话摘要。",
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

fn transcript(view: &InsightsView, cx: &mut Context<InsightsView>) -> impl IntoElement {
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

fn fidelity_label(value: &str, cx: &gpui_kit::App) -> &'static str {
    match value {
        "full" => text(cx, "complete", "完整"),
        "partial" => text(cx, "partial", "部分"),
        "locked" => text(cx, "locked", "已锁定"),
        _ => text(cx, "unknown", "未知"),
    }
}
