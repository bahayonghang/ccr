//! Entry list: 56px two-line rows inside a solid reading panel.
use super::{McpView, Node, platform_name, reading_panel};
use crate::{
    locales::{is_chinese, text},
    runtime::mcp::{McpEntry, McpScope, McpTarget},
    shell::badge,
    theme::{self, Tone},
};
use gpui_kit::base::TestSupportExt;
use gpui_kit::component::{ActiveTheme, Icon, IconName, button::Button, tooltip::Tooltip};
use gpui_kit::{
    AnyElement, App, Context, Div, FontWeight, InteractiveElement, IntoElement, ParentElement,
    Role, SharedString, StatefulInteractiveElement, Styled, div, px,
};

/// File-level disabled state: `disabled = true` or `enabled = false`.
pub(super) fn entry_disabled(entry: &McpEntry) -> bool {
    entry.disabled == Some(true) || entry.enabled == Some(false)
}

/// 只读来源（旧版 Claude settings 等）在这里不能写入。
pub(super) fn read_only(entry: &McpEntry) -> bool {
    McpTarget::for_entry(entry).is_err()
}

/// Last path segment of a project key; the full key stays in the tooltip.
pub(super) fn project_leaf(key: &str) -> &str {
    key.trim_end_matches(['/', '\\'])
        .rsplit(['/', '\\'])
        .next()
        .filter(|leaf| !leaf.is_empty())
        .unwrap_or(key)
}

/// `User` or `Project · leaf`, as shown on the second row line.
pub(super) fn scope_short(cx: &App, scope: &McpScope) -> String {
    match scope {
        McpScope::User => text(cx, "User", "用户").into(),
        McpScope::Project(key) => {
            format!("{} · {}", text(cx, "Project", "项目"), project_leaf(key))
        }
    }
}

/// `平台 · 名称 [· disabled in the file]`, then scope, transport and read-only state.
fn row_label(cx: &App, entry: &McpEntry) -> String {
    let (platform_en, platform_zh) = platform_name(entry.source);
    let mut label = format!("{} · {}", text(cx, platform_en, platform_zh), entry.name);
    if entry_disabled(entry) {
        label.push_str(text(cx, " · disabled in the file", " · 文件中已禁用"));
    }
    label.push_str(" · ");
    match &entry.scope {
        McpScope::User => label.push_str(text(cx, "User scope", "用户作用域")),
        McpScope::Project(key) => {
            if is_chinese(cx) {
                label.push_str(&format!("项目作用域 {key}"));
            } else {
                label.push_str(&format!("Project scope {key}"));
            }
        }
    }
    label.push_str(" · ");
    label.push_str(&transport_line(cx, entry));
    if read_only(entry) {
        label.push_str(text(cx, " · read-only here", " · 此处只读"));
    }
    label
}

/// 命令原文；只有 URL 时写 `URL set (hidden)`，永不显示 URL 值。
fn transport_line(cx: &App, entry: &McpEntry) -> SharedString {
    match entry.command.as_deref() {
        Some(command) => command.to_string().into(),
        None if entry.has_url => text(cx, "URL set (hidden)", "已设置 URL（隐藏）").into(),
        None => text(cx, "No command or URL", "未设置命令或 URL").into(),
    }
}

/// Transport glyph: a terminal for stdio, a globe when only a URL is set.
pub(super) fn transport_icon(entry: &McpEntry) -> IconName {
    if entry.command.is_none() && entry.has_url {
        IconName::Globe
    } else {
        IconName::SquareTerminal
    }
}

impl McpView {
    /// The left column. Empty and filtered-out states replace the panel with a
    /// dashed one instead of nesting it.
    pub(super) fn list_panel(
        &self,
        visible: &[McpEntry],
        beside: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let column = if self.catalog.is_none() {
            self.status_panel(cx)
        } else if self.scoped().next().is_none() {
            badge::empty_panel(
                "mcp-empty",
                IconName::Inbox,
                text(
                    cx,
                    "No MCP entries were found in the files this page reads.",
                    "此页读取的文件里没有 MCP 条目。",
                ),
                text(
                    cx,
                    "Use Add server to write one into a chosen source, or start from Presets.",
                    "使用“添加服务器”写入所选来源，或从“预设”开始。",
                ),
                None,
                cx,
            )
            .test_support()
        } else if visible.is_empty() {
            self.filter_empty(cx)
        } else {
            self.rows(visible, cx)
        };
        column_size(column, beside, 3., theme::LIST_MIN_WIDTH).into_any_element()
    }

    fn status_panel(&self, cx: &mut Context<Self>) -> Node {
        let (line, ink) = if self.busy {
            (
                text(cx, "Reading local MCP files.", "正在读取本地 MCP 文件。"),
                cx.theme().muted_foreground,
            )
        } else if let Some(message) = self.message {
            (
                crate::locales::service_error(cx, message),
                cx.theme().warning,
            )
        } else {
            (
                text(
                    cx,
                    "Local MCP files have not been read yet.",
                    "尚未读取本地 MCP 文件。",
                ),
                cx.theme().muted_foreground,
            )
        };
        reading_panel("mcp-list", cx)
            .p(px(theme::SECTION_GAP))
            .child(
                div()
                    .id("mcp-status")
                    .test_support()
                    .role(Role::Status)
                    .aria_label(line)
                    .text_color(ink)
                    .child(line),
            )
    }

    fn filter_empty(&self, cx: &mut Context<Self>) -> Node {
        let clear = badge::filter_chip(
            Button::new("mcp-filter-clear")
                .label(text(cx, "Clear search and platform", "清除搜索与平台"))
                .on_click(cx.listener(|this, _, window, cx| {
                    this.family = None;
                    this.filter
                        .update(cx, |state, cx| state.set_value("", window, cx));
                    cx.notify();
                })),
            false,
            false,
            cx,
        );
        badge::empty_panel(
            "mcp-filter-empty",
            IconName::Search,
            text(
                cx,
                "No MCP entries match this filter.",
                "没有 MCP 条目匹配这个筛选。",
            ),
            text(
                cx,
                "Search matches server names and commands.",
                "搜索匹配服务器名称和命令。",
            ),
            Some(clear.into_any_element()),
            cx,
        )
        .test_support()
    }

    fn rows(&self, visible: &[McpEntry], cx: &mut Context<Self>) -> Node {
        let mut list = reading_panel("mcp-list", cx)
            .track_focus(&self.list_focus)
            .tab_stop(false)
            .role(Role::List)
            .aria_label(text(cx, "MCP entries", "MCP 条目"))
            .p(px(theme::SELECTION_RADIUS));
        let hairline = theme::hairline(cx.theme().mode);
        for (index, entry) in visible.iter().enumerate() {
            if index > 0 {
                list = list.child(
                    div()
                        .flex_shrink_0()
                        .mx(px(theme::CONTROL_GAP))
                        .h(px(1.))
                        .bg(hairline),
                );
            }
            let selected = self.selected.as_deref() == Some(entry.id().as_str());
            list = list.child(entry_row(index, entry, selected, cx));
        }
        list
    }
}

/// 列宽：并排时按 3:2 分配剩余宽度并保留最小宽；单列时占满整列。
///
/// 列本身不带内边距与描边：flex 基准为 0 时，taffy 会把基准抬到条目自身的
/// padding + border，比例只作用在剩余宽度上；面板按内容或 pane 类型决定高度。
pub(super) fn column_size(panel: Node, beside: bool, share: f32, min: f32) -> Div {
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

fn entry_row(
    index: usize,
    entry: &McpEntry,
    selected: bool,
    cx: &mut Context<McpView>,
) -> gpui_kit::base::Button {
    let colors = cx.theme();
    let id = entry.id();
    gpui_kit::base::Button::new(SharedString::from(format!("mcp-entry-{index}")))
        .accessibility_label(row_label(cx, entry))
        .aria_selected(selected)
        .selected(selected)
        .flex_shrink_0()
        .w_full()
        .h(crate::theme::data_row_height(theme::MCP_ROW_HEIGHT, cx))
        .px(px(theme::CONTROL_GAP))
        .gap(px(theme::CONTROL_GAP + 4.))
        .justify_start()
        .rounded(px(theme::SELECTION_RADIUS))
        // 2px 透明边框只为键盘焦点环预留位置；行不缩放，避免文字抖动。
        .border_2()
        .border_color(gpui_kit::transparent_black())
        .bg(if selected {
            colors.secondary
        } else {
            gpui_kit::transparent_black()
        })
        .hover(|style| {
            style.bg(if selected {
                colors.secondary
            } else {
                colors.muted
            })
        })
        .focus_visible(|style| style.border_color(colors.ring))
        .child(
            Icon::new(transport_icon(entry))
                .size(px(theme::NAV_ICON_SIZE))
                .text_color(colors.muted_foreground),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .gap(px(2.))
                .child(name_line(index, entry, cx))
                .child(meta_line(index, entry, cx)),
        )
        .on_click(cx.listener(move |this, _, window, cx| this.select(id.clone(), window, cx)))
}

fn name_line(index: usize, entry: &McpEntry, cx: &App) -> Div {
    let family = super::Family::of(entry.source);
    let (platform_en, platform_zh) = platform_name(entry.source);
    let name: SharedString = entry.name.clone().into();
    let tip = name.clone();
    div()
        .w_full()
        .flex()
        .items_center()
        .gap(px(theme::CONTROL_GAP))
        .child(
            div()
                .id(SharedString::from(format!("mcp-entry-{index}-name")))
                .flex_1()
                .min_w_0()
                .truncate()
                .text_size(crate::theme::text_size(theme::BODY_SIZE))
                .line_height(px(20.))
                .font_weight(FontWeight::MEDIUM)
                .tooltip(move |window, cx| Tooltip::new(tip.clone()).build(window, cx))
                .child(name),
        )
        .child(
            badge::identity(
                family.map_or("", super::Family::id),
                text(cx, platform_en, platform_zh),
                cx,
            )
            .text_size(crate::theme::text_size(theme::META_SIZE))
            .text_color(cx.theme().muted_foreground),
        )
}

/// 第二行：命令（mono）或 `URL set (hidden)`，加作用域；URL 与参数值从不显示。
fn meta_line(index: usize, entry: &McpEntry, cx: &App) -> Div {
    let transport = transport_line(cx, entry);
    let tip = transport.clone();
    let mono = entry.command.is_some();
    let scope = scope_short(cx, &entry.scope);
    let scope_tip: SharedString = match &entry.scope {
        McpScope::User => scope.clone().into(),
        McpScope::Project(key) => key.clone().into(),
    };
    let mut command = div()
        .id(SharedString::from(format!("mcp-entry-{index}-command")))
        .min_w_0()
        .flex_shrink(1.)
        .truncate()
        .tooltip(move |window, cx| Tooltip::new(tip.clone()).build(window, cx))
        .child(transport);
    if mono {
        command = command
            .font_family(cx.theme().mono_font_family.clone())
            .text_size(cx.theme().mono_font_size);
    }
    let line = div()
        .w_full()
        .min_w_0()
        .flex()
        .items_center()
        .gap(px(theme::SELECTION_RADIUS))
        .text_size(crate::theme::text_size(theme::META_SIZE))
        .line_height(px(theme::TAG_HEIGHT))
        .text_color(cx.theme().muted_foreground)
        .child(command)
        .child(div().flex_shrink_0().child("·"))
        .child(
            div()
                .id(SharedString::from(format!("mcp-entry-{index}-scope")))
                .flex_shrink_0()
                .max_w(px(160.))
                .truncate()
                .tooltip(move |window, cx| Tooltip::new(scope_tip.clone()).build(window, cx))
                .child(scope),
        )
        .child(div().flex_1());
    tags(index, entry, line, cx)
}

// 只在异常时出现标签：文件里禁用、或此处只读。
fn tags(index: usize, entry: &McpEntry, line: Div, cx: &App) -> Div {
    let line = if entry_disabled(entry) {
        line.child(
            badge::icon_tag(
                SharedString::from(format!("mcp-entry-{index}-disabled")),
                IconName::Ban,
                text(cx, "Disabled", "已禁用"),
                Tone::Warning,
                cx,
            )
            .test_support(),
        )
    } else {
        line
    };
    if read_only(entry) {
        line.child(
            badge::icon_tag(
                SharedString::from(format!("mcp-entry-{index}-read-only")),
                IconName::Eye,
                text(cx, "Read-only", "只读"),
                Tone::Neutral,
                cx,
            )
            .test_support(),
        )
    } else {
        line
    }
}
