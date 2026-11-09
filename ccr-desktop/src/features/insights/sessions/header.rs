//! Heading row: icon and title on the left; index status, CCR trash and refresh on the right.
use crate::features::insights::InsightsView;
use crate::{
    locales::{is_chinese, service_error, text},
    runtime::insights::RefreshStatus,
    shell::{badge, semantics},
    theme::{self, Tone},
};
use gpui_kit::base::TestSupportExt;
use gpui_kit::component::{
    ActiveTheme, IconName,
    button::{Button, ButtonVariants},
};
use gpui_kit::{
    AnyElement, Context, Div, FontWeight, InteractiveElement, IntoElement, ParentElement, Role,
    StatefulInteractiveElement, Styled, div, px,
};

pub(super) fn header(view: &InsightsView, cx: &mut Context<InsightsView>) -> AnyElement {
    div()
        .id("session-header")
        .test_support()
        .flex_shrink_0()
        .flex()
        .flex_wrap()
        .items_start()
        .justify_between()
        .gap(px(theme::SECTION_GAP))
        .child(title(cx))
        .child(actions(view, cx))
        .into_any_element()
}

// h1 与其他页面共用内容左缘。
fn title(cx: &mut Context<InsightsView>) -> Div {
    div()
        .min_w_0()
        .flex()
        .flex_col()
        .gap(px(2.))
        .child(
            div()
                .id("session-title")
                .test_support()
                .role(Role::Heading)
                .aria_level(1)
                .aria_label(text(cx, "Agent sessions", "Agent 会话"))
                .text_size(px(theme::TITLE_SIZE))
                .font_weight(FontWeight::SEMIBOLD)
                .child(text(cx, "Agent sessions", "Agent 会话")),
        )
        .child(
            div()
                .text_size(px(theme::META_SIZE))
                .text_color(cx.theme().muted_foreground)
                .child(text(
                    cx,
                    "Indexed summaries from this computer.",
                    "本机索引出的会话摘要。",
                )),
        )
}

fn actions(view: &InsightsView, cx: &mut Context<InsightsView>) -> Div {
    let running = matches!(view.requests.refresh_status(), RefreshStatus::Running(_));
    let row = div()
        .flex()
        .flex_wrap()
        .items_center()
        .gap(px(theme::CONTROL_GAP));
    let row = match status_tag(view, cx) {
        Some(tag) => row.child(tag),
        None => row,
    };
    let show_trash = matches!(view.snapshot.as_deref(), Some(crate::features::insights::InsightsSnapshot::Sessions { statuses, .. })
        if statuses.iter().any(|status| status.agent == ccr_store::sessions::AgentSessionAgentId::Codex
            && status.availability != ccr_store::sessions::ProviderAvailability::NotInstalled));
    let row = if show_trash {
        row.child(semantics::ghost_button(
            Button::new("session-open-trash")
                .icon(IconName::Inbox)
                .label(text(cx, "CCR trash", "CCR 废纸篓"))
                .tooltip(text(
                    cx,
                    "Open or re-read recoverable Codex sessions",
                    "打开或重新读取可恢复的 Codex 会话",
                ))
                .ghost()
                .on_click(cx.listener(|this, _, _, cx| this.open_session_trash(cx))),
            view.session_ops_active(),
            false,
            cx,
        ))
    } else {
        row
    };
    row.child(semantics::button(
        Button::new("insights-index-refresh")
            .icon(IconName::RefreshCw)
            .label(if running {
                text(cx, "Indexing", "正在索引")
            } else {
                text(cx, "Refresh index", "刷新索引")
            })
            .primary()
            .on_click(cx.listener(|this, _, _, cx| this.start_archive_refresh(cx))),
        running,
        running,
        cx,
    ))
}

/// Index refresh state as a tag; Idle shows none. The tag keeps `role=Status`.
fn status_tag(view: &InsightsView, cx: &mut Context<InsightsView>) -> Option<AnyElement> {
    let (label, tone) = match view.requests.refresh_status() {
        RefreshStatus::Idle => return None,
        RefreshStatus::Running(_) => (text(cx, "Indexing…", "正在索引…").to_string(), Tone::Info),
        RefreshStatus::Finished(_, report) => (
            if is_chinese(cx) {
                format!(
                    "已索引 · 解析 {} / 写入 {}",
                    report.total.parsed, report.total.upserted
                )
            } else {
                format!(
                    "Indexed · parsed {} / upserted {}",
                    report.total.parsed, report.total.upserted
                )
            },
            Tone::Success,
        ),
        RefreshStatus::Failed(_, error) => {
            (service_error(cx, error.message()).to_string(), Tone::Danger)
        }
        RefreshStatus::Interrupted(_) => (
            text(cx, "Indexing stopped before a result", "索引中断，没有结果").to_string(),
            Tone::Danger,
        ),
    };
    Some(
        badge::tag("session-status", label, tone, cx)
            .role(Role::Status)
            .test_support()
            .into_any_element(),
    )
}
