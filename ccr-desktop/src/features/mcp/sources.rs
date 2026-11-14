//! Source health strip: one non-interactive chip per source file.
//!
//! chip 只写短名与读数；完整原句（`note_line`）同时作为无障碍标签和 tooltip，
//! 测试按原句断言。窄宽在条内横向滚动，不换行撑高。
use super::{McpView, Node, note_line};
use crate::{
    locales::text,
    runtime::mcp::{McpSource, McpSourceState},
    shell::badge,
    theme::{self, Tone},
};
use gpui_kit::base::TestSupportExt;
use gpui_kit::component::{ActiveTheme, Icon, IconName, tooltip::Tooltip};
use gpui_kit::{
    AnyElement, App, Context, Hsla, InteractiveElement, IntoElement, ParentElement, Role,
    SharedString, StatefulInteractiveElement, Styled, div, px, rgb,
};

impl McpView {
    pub(super) fn sources(&self, cx: &mut Context<Self>) -> Option<Node> {
        let catalog = self.catalog.as_ref()?;
        let mut strip = div()
            .id("mcp-sources")
            .test_support()
            .flex_shrink_0()
            .flex()
            .items_center()
            .gap(px(theme::CONTROL_GAP))
            .overflow_x_scroll();
        for (index, note) in catalog.notes.iter().enumerate() {
            if self.source_in_scope(note.source) {
                strip = strip.child(source_chip(index, note.source, note.state, cx));
            }
        }
        Some(strip)
    }
}

fn source_short(source: McpSource) -> (&'static str, &'static str) {
    match source {
        McpSource::ClaudeState => ("Claude state", "Claude 状态"),
        McpSource::ClaudeSettings => ("Claude settings", "Claude 设置"),
        McpSource::CodexConfig => ("Codex", "Codex"),
        McpSource::AntigravityConfig => ("Antigravity", "Antigravity"),
        McpSource::ProjectFile => ("Project", "项目"),
        McpSource::OpenCode => ("OpenCode", "OpenCode"),
        McpSource::Grok => ("Grok", "Grok"),
    }
}

/// 读数只陈述 catalog 给出的状态，不推断服务器是否在运行。
fn reading(cx: &App, source: McpSource, state: McpSourceState) -> String {
    match state {
        McpSourceState::Entries(count) if count > 0 => count.to_string(),
        McpSourceState::Empty | McpSourceState::Entries(_) => text(cx, "none", "无").into(),
        McpSourceState::Missing => text(cx, "not found", "未找到").into(),
        McpSourceState::Invalid => text(cx, "invalid", "无效").into(),
        McpSourceState::Unreadable => text(cx, "unreadable", "无法读取").into(),
        McpSourceState::NotRead if source == McpSource::Grok => {
            text(cx, "not listed", "不列出").into()
        }
        McpSourceState::NotRead => text(cx, "not read", "未读取").into(),
    }
}

fn indicator(state: McpSourceState, cx: &App) -> AnyElement {
    let mode = cx.theme().mode;
    let muted: Hsla = rgb(theme::palette(mode).muted).into();
    match state {
        McpSourceState::Entries(count) if count > 0 => {
            badge::dot(theme::tone_colors(Tone::Success, mode).ink).into_any_element()
        }
        McpSourceState::Empty | McpSourceState::Entries(_) => badge::dot(muted).into_any_element(),
        McpSourceState::Missing | McpSourceState::NotRead => {
            badge::hollow_dot(muted).into_any_element()
        }
        McpSourceState::Invalid => Icon::new(IconName::TriangleAlert)
            .size(px(theme::META_SIZE))
            .text_color(theme::tone_colors(Tone::Warning, mode).ink)
            .into_any_element(),
        McpSourceState::Unreadable => Icon::new(IconName::CircleAlert)
            .size(px(theme::META_SIZE))
            .text_color(theme::tone_colors(Tone::Danger, mode).ink)
            .into_any_element(),
    }
}

fn source_chip(index: usize, source: McpSource, state: McpSourceState, cx: &App) -> Node {
    let line: SharedString = note_line(cx, source, state).into();
    let tip = line.clone();
    let mode = cx.theme().mode;
    let (short_en, short_zh) = source_short(source);
    let chip = div()
        .id(SharedString::from(format!("mcp-source-{index}")))
        .test_support()
        .role(Role::Label)
        .aria_label(line)
        .tooltip(move |window, cx| Tooltip::new(tip.clone()).build(window, cx))
        .flex_shrink_0()
        .h(crate::theme::text_size(theme::TAG_HEIGHT))
        .flex()
        .items_center()
        .gap(px(theme::SELECTION_RADIUS + 2.))
        .px(px(theme::CONTROL_GAP))
        .rounded(px(theme::SELECTION_RADIUS))
        .border_1()
        .text_size(crate::theme::text_size(theme::META_SIZE))
        .font_features(theme::tabular_figures())
        .whitespace_nowrap();
    // 只有 Invalid / Unreadable 用语调色；未读取用虚线边，其余是中性 hairline。
    let chip = match state {
        McpSourceState::Invalid | McpSourceState::Unreadable => {
            let tone = if state == McpSourceState::Invalid {
                Tone::Warning
            } else {
                Tone::Danger
            };
            let colors = theme::tone_colors(tone, mode);
            chip.bg(colors.fill)
                .border_color(colors.edge)
                .text_color(colors.ink)
        }
        McpSourceState::NotRead => chip
            .border_dashed()
            .border_color(rgb(theme::palette(mode).subtle))
            .text_color(rgb(theme::palette(mode).secondary_text)),
        _ => chip
            .border_color(theme::hairline(mode))
            .text_color(rgb(theme::palette(mode).secondary_text)),
    };
    chip.child(indicator(state, cx)).child(format!(
        "{} · {}",
        text(cx, short_en, short_zh),
        reading(cx, source, state)
    ))
}
