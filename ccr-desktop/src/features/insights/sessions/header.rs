use crate::features::insights::{InsightsSnapshot, InsightsView};
use crate::{
    locales::{is_chinese, service_error, text},
    runtime::insights::RefreshStatus,
    shell::semantics,
    theme,
};
use gpui_kit::base::TestSupportExt;
use gpui_kit::component::{ActiveTheme, button::Button};
use gpui_kit::{
    Context, InteractiveElement, IntoElement, ParentElement, StatefulInteractiveElement, Styled,
    div, px,
};

pub(super) fn header(
    _view: &InsightsView,
    running: bool,
    cx: &mut Context<InsightsView>,
) -> impl IntoElement {
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

pub(super) fn status_line(view: &InsightsView, cx: &mut Context<InsightsView>) -> impl IntoElement {
    let message = if view.loading {
        text(cx, "Reading the archive.", "正在读取归档。").to_string()
    } else if let Some(error) = view.error {
        service_error(cx, error).to_string()
    } else {
        match view.requests.refresh_status() {
            RefreshStatus::Running(_) => text(
                cx,
                "Indexing local transcripts. Summaries already stored stay listed.",
                "正在索引本机记录。已经写入的摘要仍会列出。",
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
                let not_indexed = matches!(
                    view.snapshot.as_deref(),
                    Some(InsightsSnapshot::Sessions { indexed: false, .. }) | None
                );
                if not_indexed {
                    text(
                        cx,
                        "Indexing finished. Sessions are not indexed.",
                        "索引结束了。会话尚未索引。",
                    )
                    .to_string()
                } else if is_chinese(cx) {
                    format!(
                        "索引已结束：发现 {}，写入 {}",
                        report.total.discovered, report.total.upserted
                    )
                } else {
                    format!(
                        "Index finished: discovered {}, written {}",
                        report.total.discovered, report.total.upserted
                    )
                }
            }
            RefreshStatus::Idle => match view.snapshot.as_deref() {
                Some(InsightsSnapshot::Sessions { indexed: false, .. }) => {
                    text(cx, "Sessions are not indexed.", "会话尚未索引。").to_string()
                }
                Some(InsightsSnapshot::Sessions { page, .. }) if page.items.is_empty() => text(
                    cx,
                    "No indexed sessions match this page.",
                    "这一页没有匹配的已索引会话。",
                )
                .to_string(),
                Some(InsightsSnapshot::Sessions { page, .. }) => {
                    let count = page.items.len();
                    if page.next_cursor.is_some() {
                        if is_chinese(cx) {
                            format!("本页 {count} 条已索引会话，还有更早的。")
                        } else {
                            format!("{count} indexed sessions on this page. Older ones follow.")
                        }
                    } else if is_chinese(cx) {
                        format!("{count} 条已索引会话。")
                    } else {
                        format!("{count} indexed sessions.")
                    }
                }
                _ => text(cx, "The archive has not been read.", "还没有读取归档。").to_string(),
            },
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
