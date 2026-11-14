//! Side pane: CCR trash, the opened session, or a dashed empty state.
//!
//! 优先级：废纸篓 > 详情 > 空态。决策控件（确认、导出、更早的消息）放在可变高度的
//! 消息列表之前，保证分栏 pane 里主按钮与初始焦点在视口内。
use super::{column_size, list::session_title, providers::agent_name, reading_panel};
use crate::features::insights::InsightsView;
use crate::{
    locales::{service_error, text},
    shell::{badge, semantics},
    theme::{self, Tone},
};
use ccr_store::sessions::{AgentSessionFidelity, AgentSessionMessage};
use gpui_kit::base::TestSupportExt;
use gpui_kit::component::{
    ActiveTheme, Icon, IconName, Sizable,
    button::{Button, ButtonVariants},
    tooltip::Tooltip,
};
use gpui_kit::{
    AnyElement, App, Context, Div, FontWeight, InteractiveElement, IntoElement, ParentElement,
    Role, SharedString, StatefulInteractiveElement, Styled, div, px,
};

pub(super) fn pane_column(
    view: &mut InsightsView,
    beside: bool,
    window: &mut gpui_kit::Window,
    cx: &mut Context<InsightsView>,
) -> AnyElement {
    let panel = if view.session_ops_trash_open() {
        let trash = view.render_session_trash(window, cx);
        let panel = pane_panel(view, cx);
        let panel = if view.session_trash_empty() {
            panel.flex_none().flex_shrink(1.)
        } else {
            panel
        };
        panel.child(trash).into_any_element()
    } else if view.detail.archive_id.is_some() {
        detail(view, beside, window, cx)
    } else {
        empty_pane(view, cx)
    };
    column_size(panel, beside, 2., theme::DETAIL_MIN_WIDTH).into_any_element()
}

fn pane_panel(
    view: &InsightsView,
    cx: &App,
) -> gpui_kit::base::ObservedElement<gpui_kit::Stateful<Div>> {
    reading_panel("session-pane", cx)
        .flex_1()
        .track_focus(&view.session_pane_focus)
        .tab_stop(false)
        .overflow_y_scroll()
        .p(px(theme::SECTION_GAP))
        .gap(px(theme::SECTION_GAP))
}

// 宽窗未打开会话：说明下一步，只用本页读到的行数。
fn empty_pane(view: &InsightsView, cx: &mut Context<InsightsView>) -> AnyElement {
    let count = view.row_count();
    let body = if crate::locales::is_chinese(cx) {
        format!("本页列出 {count} 条已索引会话。")
    } else if count == 1 {
        "1 indexed session is listed on this page.".to_string()
    } else {
        format!("{count} indexed sessions are listed on this page.")
    };
    badge::empty_panel(
        "session-pane-empty",
        IconName::BookOpen,
        text(
            cx,
            "Select a session to read its messages.",
            "选择一个会话来阅读消息。",
        ),
        body,
        None,
        cx,
    )
    .test_support()
    .into_any_element()
}

fn detail(
    view: &mut InsightsView,
    beside: bool,
    window: &mut gpui_kit::Window,
    cx: &mut Context<InsightsView>,
) -> AnyElement {
    let mut panel = pane_panel(view, cx);
    // 窄窗 drill-in 需要明确的返回。
    if !beside {
        panel = panel.child(
            semantics::button(
                Button::new("insights-back")
                    .icon(IconName::ArrowLeft)
                    .label(text(cx, "Back to list", "返回列表"))
                    .ghost()
                    .on_click(
                        cx.listener(|this, _, window, cx| this.back_to_session_list(window, cx)),
                    ),
                false,
                false,
                cx,
            )
            .self_start(),
        );
    }
    let heading = detail_heading(view, cx);
    panel = panel.child(heading);
    if let Some(confirmation) = view.render_session_confirmation(window, cx) {
        panel = panel.child(confirmation);
    }
    if let Some(notice) = fidelity_notice(view, cx) {
        panel = panel.child(notice);
    }
    if let Some(export) = view.render_session_export(cx) {
        panel = panel.child(export);
    }
    panel = panel.child(read_state(view, cx));
    panel.child(transcript(view, cx)).into_any_element()
}

// 头部：标题（16/600，省略）、mono session id、agent · variant 标签；右侧是 Codex 操作。
fn detail_heading(view: &mut InsightsView, cx: &mut Context<InsightsView>) -> AnyElement {
    let row = view.detail.row.clone();
    let (title, session_id) = match &row {
        Some(row) => (session_title(row), row.session_id.clone()),
        None => (
            text(cx, "Session", "会话").to_string(),
            view.detail.archive_id.clone().unwrap_or_default(),
        ),
    };
    let tip: SharedString = title.clone().into();
    let variant = view
        .session_detail()
        .map(|detail| format!("{} · {}", agent_name(detail.agent), detail.variant));
    let mut ident = div()
        .min_w_0()
        .flex()
        .items_center()
        .gap(px(theme::CONTROL_GAP))
        .child(
            div()
                .id("session-detail-id")
                .min_w_0()
                .truncate()
                .text_size(crate::theme::text_size(theme::META_SIZE))
                .text_color(cx.theme().muted_foreground)
                .font_family(cx.theme().mono_font_family.clone())
                .text_size(cx.theme().mono_font_size)
                .child(session_id),
        );
    if let Some(variant) = variant {
        ident = ident
            .child(badge::tag("session-detail-variant", variant, Tone::Neutral, cx).test_support());
    }
    let text_block = div()
        .flex_1()
        .min_w_0()
        .flex()
        .flex_col()
        .gap(px(theme::SELECTION_RADIUS))
        .child(
            div()
                .id("session-detail-title")
                .test_support()
                .role(Role::Heading)
                .aria_level(2)
                .min_w_0()
                .truncate()
                .text_size(crate::theme::text_size(theme::SECTION_TITLE_SIZE))
                .font_weight(FontWeight::SEMIBOLD)
                .tooltip(move |window, cx| Tooltip::new(tip.clone()).build(window, cx))
                .child(title),
        )
        .child(ident);
    let heading = div()
        .id("session-detail-heading")
        .test_support()
        .flex_shrink_0()
        .flex()
        .flex_wrap()
        .items_start()
        .justify_between()
        .gap(px(theme::CONTROL_GAP))
        .child(text_block);
    match view.render_session_actions(cx) {
        Some(actions) => heading.child(actions).into_any_element(),
        None => heading.into_any_element(),
    }
}

// partial / locked 时的 Warning 提示条；完整记录不提示。
fn fidelity_notice(view: &InsightsView, cx: &App) -> Option<AnyElement> {
    let fidelity = view.session_detail()?.page.fidelity;
    let line = match fidelity {
        AgentSessionFidelity::Full => return None,
        AgentSessionFidelity::Partial => text(
            cx,
            "Partial record: the provider exposes only part of this transcript.",
            "部分记录：提供方只给出了这段记录的一部分。",
        ),
        AgentSessionFidelity::Locked => text(
            cx,
            "Locked: the provider keeps this transcript unreadable; only the indexed summary is shown.",
            "锁定：提供方的这段记录不可读，这里只显示索引摘要。",
        ),
    };
    let colors = theme::tone_colors(Tone::Warning, cx.theme().mode);
    Some(
        div()
            .id("session-fidelity-notice")
            .test_support()
            .role(Role::Note)
            .aria_label(line)
            .flex_shrink_0()
            .flex()
            .items_start()
            .gap(px(theme::CONTROL_GAP))
            .px(px(theme::CONTROL_GAP + 4.))
            .py(px(theme::CONTROL_GAP))
            .rounded(px(theme::SELECTION_RADIUS))
            .border_1()
            .border_color(colors.edge)
            .bg(colors.fill)
            .text_color(colors.ink)
            .text_size(crate::theme::text_size(theme::META_SIZE))
            .child(
                Icon::new(IconName::TriangleAlert)
                    .size(px(theme::NAV_ICON_SIZE))
                    .text_color(colors.ink),
            )
            .child(div().min_w_0().child(line))
            .into_any_element(),
    )
}

// 读取中 / 失败 / 更早的消息。按钮在消息之前，长记录不会把它挤出视口。
fn read_state(view: &InsightsView, cx: &mut Context<InsightsView>) -> AnyElement {
    let row = div()
        .flex_shrink_0()
        .flex()
        .flex_wrap()
        .items_center()
        .gap(px(theme::CONTROL_GAP))
        .text_size(crate::theme::text_size(theme::META_SIZE))
        .text_color(cx.theme().muted_foreground);
    if view.detail.loading {
        return row
            .child(text(cx, "Reading the session.", "正在读取会话。"))
            .into_any_element();
    }
    if let Some(error) = view.detail.error {
        return row
            .child(
                div()
                    .text_color(theme::tone_colors(Tone::Danger, cx.theme().mode).ink)
                    .child(service_error(cx, error)),
            )
            .into_any_element();
    }
    let Some(detail) = view.session_detail() else {
        return row.into_any_element();
    };
    let count = detail.page.messages.len();
    let total = view
        .detail
        .row
        .as_ref()
        .map_or(count as u64, |row| row.message_count as u64);
    let tools = view.detail.row.as_ref().map_or(0, |row| row.tool_use_count);
    let counts = if crate::locales::is_chinese(cx) {
        format!("显示 {count} / {total} 条消息 · {tools} 次工具")
    } else {
        format!("{count} of {total} messages shown · {tools} tools")
    };
    let row = row.child(
        div()
            .id("session-detail-counts")
            .test_support()
            .aria_label(counts.clone())
            .font_features(theme::tabular_figures())
            .child(counts),
    );
    if !detail.page.has_older {
        return row.into_any_element();
    }
    row.child(semantics::button(
        Button::new("session-older-messages")
            .label(text(cx, "Earlier messages", "更早的消息"))
            .ghost()
            .small()
            .on_click(cx.listener(|this, _, _, cx| this.older_session_messages(cx))),
        view.detail.loading,
        false,
        cx,
    ))
    .into_any_element()
}

fn transcript(view: &InsightsView, cx: &mut Context<InsightsView>) -> AnyElement {
    let Some(detail) = view.session_detail() else {
        return div().into_any_element();
    };
    let mut messages = div()
        .id("session-transcript")
        .test_support()
        .flex()
        .flex_col()
        .gap(px(theme::CONTROL_GAP));
    for (index, message) in detail.page.messages.iter().enumerate() {
        if index > 0 {
            messages = messages.child(
                div()
                    .flex_shrink_0()
                    .h(px(1.))
                    .bg(theme::hairline(cx.theme().mode)),
            );
        }
        messages = messages.child(message_card(message, cx));
    }
    messages.into_any_element()
}

// 每张消息卡在独立函数里构建，避免增大 render 栈帧（Windows UI 线程只有 1 MiB）。
fn message_card(message: &AgentSessionMessage, cx: &App) -> AnyElement {
    let colors = cx.theme();
    let icon = match message.role.as_str() {
        "user" => IconName::User,
        "assistant" => IconName::Bot,
        _ => IconName::SquareTerminal,
    };
    let ink = colors.muted_foreground;
    let role = match message.role.as_str() {
        "user" => text(cx, "User", "用户"),
        "assistant" => text(cx, "Assistant", "助手"),
        "tool" => text(cx, "Tool", "工具"),
        "system" => text(cx, "System", "系统"),
        other => other,
    };
    let when = message
        .timestamp
        .map(|time| {
            time.with_timezone(&chrono::Local)
                .format("%m-%d %H:%M")
                .to_string()
        })
        .unwrap_or_else(|| format!("#{}", message.ordinal));
    let mut head = div()
        .flex()
        .items_center()
        .gap(px(theme::SELECTION_RADIUS + 2.))
        .text_size(crate::theme::text_size(theme::META_SIZE))
        .child(
            Icon::new(icon)
                .size(px(theme::META_SIZE + 2.))
                .text_color(ink),
        )
        .child(div().text_color(ink).child(role.to_string()));
    if let Some(tool) = &message.tool_name {
        head = head.child(
            div()
                .min_w_0()
                .truncate()
                .text_color(colors.muted_foreground)
                .font_family(colors.mono_font_family.clone())
                .text_size(colors.mono_font_size)
                .child(tool.clone()),
        );
    }
    head = head.child(div().flex_1());
    if message.clipped {
        head = head.child(
            div()
                .flex_shrink_0()
                .text_color(theme::tone_colors(Tone::Warning, colors.mode).ink)
                .child(text(cx, "Clipped in archive", "归档中已截断")),
        );
    }
    head = head.child(
        div()
            .flex_shrink_0()
            .text_color(colors.muted_foreground)
            .font_features(theme::tabular_figures())
            .child(when),
    );
    div()
        .flex_shrink_0()
        .py(px(theme::CONTROL_GAP + 4.))
        .flex()
        .flex_col()
        .gap(px(theme::SELECTION_RADIUS + 2.))
        .child(head)
        .child(
            div()
                .text_size(crate::theme::text_size(theme::BODY_SIZE))
                .child(message.content.clone()),
        )
        .into_any_element()
}
