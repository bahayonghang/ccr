//! Toolbar and session list: search, fidelity and source-state capsules, then
//! 60px two-line rows in a solid reading panel with a paging bar.
use super::{column_size, providers::agent_name, reading_panel, state};
use crate::features::insights::{InsightsSnapshot, InsightsView};
use crate::{
    locales::{is_chinese, service_error, text},
    shell::{badge, page_header, semantics},
    theme::{self, Tone},
};
use ccr_db::database::repositories::usage_repo::{AgentSessionArchiveRow, UsageSourceState};
use ccr_store::sessions::AgentSessionAgentId;
use gpui_kit::base::TestSupportExt;
use gpui_kit::component::{
    ActiveTheme, IconName, Sizable,
    button::{Button, ButtonVariants},
    input::Input,
    tooltip::Tooltip,
};
use gpui_kit::{
    AnyElement, App, Context, Div, FontWeight, InteractiveElement, IntoElement, ParentElement,
    Role, SharedString, StatefulInteractiveElement, Styled, div, px, uniform_list,
};

fn placeholder(cx: &App) -> &'static str {
    text(cx, "Search session titles", "搜索会话标题")
}

/// 语言切换后同步一次搜索框 placeholder。
pub(super) fn sync_placeholder(
    view: &mut InsightsView,
    window: &mut gpui_kit::Window,
    cx: &mut Context<InsightsView>,
) {
    let chinese = is_chinese(cx);
    if view.search_placeholder_chinese == Some(chinese) {
        return;
    }
    view.search_placeholder_chinese = Some(chinese);
    let wanted = placeholder(cx);
    view.fields[6].update(cx, |state, cx| state.set_placeholder(wanted, window, cx));
}

/// Search, fidelity and source-state on the left; the page count on the right.
pub(super) fn toolbar(view: &InsightsView, cx: &mut Context<InsightsView>) -> AnyElement {
    let filters = div()
        .min_w_0()
        .flex()
        .flex_wrap()
        .items_center()
        .gap(px(theme::CONTROL_GAP))
        .child(badge::search_field(
            Input::new(&view.fields[6])
                .id("insights-filter-6")
                .aria_label(text(cx, "Session title search", "会话标题搜索")),
            cx,
        ))
        .child(choice_capsule(
            "session-fidelity",
            &state::FIDELITIES,
            view.fidelity,
            Choice::Fidelity,
            cx,
        ))
        .child(choice_capsule(
            "session-state",
            &state::SOURCE_STATES,
            view.source_state,
            Choice::SourceState,
            cx,
        ));
    div()
        .id("session-toolbar")
        .test_support()
        .flex_shrink_0()
        .flex()
        .flex_wrap()
        .items_center()
        .justify_between()
        .gap(px(theme::CONTROL_GAP))
        .child(filters)
        .child(page_count(view, cx))
        .into_any_element()
}

#[derive(Clone, Copy)]
enum Choice {
    Fidelity,
    SourceState,
}

// 互斥分段胶囊：All + 三个取值。选项构建放在独立函数中，避免增大 render 栈帧。
fn choice_capsule(
    prefix: &'static str,
    options: &[(&'static str, &'static str, &'static str); 3],
    current: Option<&'static str>,
    choice: Choice,
    cx: &mut Context<InsightsView>,
) -> Div {
    let mut capsule = page_header::capsule(cx).self_center().child(choice_tab(
        prefix,
        "all",
        "All",
        "全部",
        None,
        current.is_none(),
        choice,
        cx,
    ));
    for (value, en, zh) in options {
        capsule = capsule.child(choice_tab(
            prefix,
            value,
            en,
            zh,
            Some(*value),
            current == Some(*value),
            choice,
            cx,
        ));
    }
    capsule
}

#[allow(clippy::too_many_arguments)]
fn choice_tab(
    prefix: &'static str,
    key: &'static str,
    en: &'static str,
    zh: &'static str,
    value: Option<&'static str>,
    selected: bool,
    choice: Choice,
    cx: &mut Context<InsightsView>,
) -> Button {
    semantics::tab(
        semantics::button(
            Button::new(SharedString::from(format!("{prefix}-{key}")))
                .label(text(cx, en, zh))
                .ghost()
                .on_click(cx.listener(move |this, _, _, cx| match choice {
                    Choice::Fidelity => this.set_session_fidelity(value, cx),
                    Choice::SourceState => this.set_session_source_state(value, cx),
                })),
            false,
            false,
            cx,
        ),
        selected,
        false,
        cx,
    )
}

/// `100 on this page · older available`; reading state while a list request runs.
fn page_count(view: &InsightsView, cx: &mut Context<InsightsView>) -> AnyElement {
    let line = if view.loading {
        text(cx, "Reading the archive…", "正在读取归档…").to_string()
    } else {
        match view.snapshot.as_deref() {
            Some(InsightsSnapshot::Sessions {
                page,
                indexed: true,
                ..
            }) => {
                let count = page.items.len();
                match (is_chinese(cx), page.next_cursor.is_some()) {
                    (true, true) => format!("本页 {count} 条 · 还有更早的"),
                    (true, false) => format!("本页 {count} 条"),
                    (false, true) => format!("{count} on this page · older available"),
                    (false, false) => format!("{count} on this page"),
                }
            }
            _ => String::new(),
        }
    };
    div()
        .id("session-page-count")
        .test_support()
        .flex_shrink_0()
        .text_size(px(theme::META_SIZE))
        .text_color(cx.theme().muted_foreground)
        .font_features(theme::tabular_figures())
        .child(line)
        .into_any_element()
}

/// The list column: rows with a paging bar, or a stated reason for no rows.
pub(super) fn list_column(
    view: &mut InsightsView,
    beside: bool,
    cx: &mut Context<InsightsView>,
) -> AnyElement {
    let panel = match view.snapshot.as_deref() {
        Some(InsightsSnapshot::Sessions {
            indexed: true,
            page,
            ..
        }) if !page.items.is_empty() => rows(view, cx),
        _ => list_state(view, cx),
    };
    column_size(panel, beside, 3., theme::LIST_MIN_WIDTH).into_any_element()
}

// 没有行时的原因：读取中、读取失败、未索引、筛选无匹配。都不铺假 0。
fn list_state(view: &InsightsView, cx: &mut Context<InsightsView>) -> AnyElement {
    if let Some(error) = view.error {
        return badge::empty_panel(
            "session-list-empty",
            IconName::CircleAlert,
            text(cx, "The archive could not be read.", "无法读取归档。"),
            service_error(cx, error),
            None,
            cx,
        )
        .test_support()
        .flex_1()
        .into_any_element();
    }
    match view.snapshot.as_deref() {
        Some(InsightsSnapshot::Sessions { indexed: false, .. }) => {
            let refresh = semantics::button(
                Button::new("session-empty-refresh")
                    .icon(IconName::RefreshCw)
                    .label(text(cx, "Refresh index", "刷新索引"))
                    .on_click(cx.listener(|this, _, _, cx| this.start_archive_refresh(cx))),
                false,
                false,
                cx,
            );
            badge::empty_panel(
                "session-list-empty",
                IconName::Inbox,
                text(cx, "Sessions are not indexed.", "会话尚未索引。"),
                text(
                    cx,
                    "Refresh index reads local transcripts into summaries. Nothing is shown until then.",
                    "刷新索引会把本机记录读成摘要；在此之前这里不显示任何会话。",
                ),
                Some(refresh.into_any_element()),
                cx,
            )
            .test_support()
            .flex_1()
            .into_any_element()
        }
        Some(InsightsSnapshot::Sessions { .. }) => {
            let filtered = view.agents.len() > usize::from(view.locked_agent().is_some())
                || view.fidelity.is_some()
                || view.source_state.is_some()
                || !view.fields[6].read(cx).value().trim().is_empty();
            let clear = filtered.then(|| {
                badge::filter_chip(
                    Button::new("session-filter-clear")
                        .label(text(cx, "Clear search and filters", "清除搜索与筛选"))
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.fields[6].update(cx, |state, cx| state.set_value("", window, cx));
                            this.reset_session_filters();
                            this.apply_session_filters(cx);
                        })),
                    false,
                    false,
                    cx,
                )
                .into_any_element()
            });
            badge::empty_panel(
                "session-list-empty",
                IconName::Search,
                text(
                    cx,
                    "No indexed sessions match these filters.",
                    "没有已索引会话匹配这些筛选。",
                ),
                text(
                    cx,
                    "This is not proof that a provider has no sessions; refresh the index to read new transcripts.",
                    "这不代表提供方没有会话；刷新索引可读取新的记录。",
                ),
                clear,
                cx,
            )
            .test_support()
            .flex_1()
            .into_any_element()
        }
        _ => reading_panel("session-list", cx)
            .p(px(theme::SECTION_GAP))
            .text_size(px(theme::META_SIZE))
            .text_color(cx.theme().muted_foreground)
            .child(if view.loading {
                text(cx, "Reading the archive.", "正在读取归档。")
            } else {
                text(cx, "The archive has not been read.", "还没有读取归档。")
            })
            .into_any_element(),
    }
}

// 列表面板：listbox 获得焦点后上下键移动选中，Enter 打开；底部是翻页条。
fn rows(view: &mut InsightsView, cx: &mut Context<InsightsView>) -> AnyElement {
    let count = view.row_count();
    let list = uniform_list(
        "session-cards",
        count,
        cx.processor(|this, range: std::ops::Range<usize>, _, cx| {
            let Some(InsightsSnapshot::Sessions { page, .. }) = this.snapshot.as_deref() else {
                return Vec::new();
            };
            let total = page.items.len();
            range
                .filter_map(|index| page.items.get(index).map(|row| (index, row.clone())))
                .map(|(index, row)| {
                    let selected = this.selected.as_deref() == Some(row.archive_id.as_str());
                    session_row(index, total, &row, selected, cx)
                })
                .collect()
        }),
    )
    .track_scroll(&view.scroll)
    .flex_1()
    .min_h_0();
    let listbox = div()
        .id("session-listbox")
        .test_support()
        .track_focus(&view.session_list_focus)
        .tab_stop(true)
        .role(Role::ListBox)
        .aria_label(text(cx, "Indexed sessions", "已索引会话"))
        .flex_1()
        .min_h_0()
        .flex()
        .flex_col()
        .p(px(theme::SELECTION_RADIUS))
        .on_key_down(
            cx.listener(|this, event: &gpui_kit::KeyDownEvent, window, cx| {
                match event.keystroke.key.as_str() {
                    "down" => this.select_offset(true, cx),
                    "up" => this.select_offset(false, cx),
                    "enter" => {
                        if let Some(index) = (0..this.row_count())
                            .find(|&i| this.row_key(i) == this.selected.as_deref())
                        {
                            this.open_session(index, window, cx);
                        }
                    }
                    _ => return,
                }
                cx.stop_propagation();
            }),
        )
        .child(list);
    let panel = reading_panel("session-list", cx).child(listbox);
    match paging(view, cx) {
        Some(bar) => panel.child(bar).into_any_element(),
        None => panel.into_any_element(),
    }
}

/// `Older sessions` when the page has a cursor; `Newest sessions` off the first page.
fn paging(view: &InsightsView, cx: &mut Context<InsightsView>) -> Option<AnyElement> {
    let older = matches!(
        view.snapshot.as_deref(),
        Some(InsightsSnapshot::Sessions { page, .. }) if page.next_cursor.is_some()
    );
    let first = view.cursor.is_some();
    if !older && !first {
        return None;
    }
    let mut bar = div()
        .id("session-paging")
        .test_support()
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_end()
        .gap(px(theme::CONTROL_GAP))
        .px(px(theme::CONTROL_GAP))
        .py(px(theme::SELECTION_RADIUS + 2.))
        .border_t_1()
        .border_color(theme::hairline(cx.theme().mode));
    if first {
        bar = bar.child(semantics::button(
            Button::new("session-first-page")
                .label(text(cx, "Newest sessions", "最新的会话"))
                .ghost()
                .small()
                .on_click(cx.listener(|this, _, _, cx| this.first_session_page(cx))),
            view.loading,
            false,
            cx,
        ));
    }
    if older {
        bar = bar.child(semantics::button(
            Button::new("insights-next")
                .label(text(cx, "Older sessions", "更早的会话"))
                .ghost()
                .small()
                .on_click(cx.listener(|this, _, _, cx| this.next(cx))),
            view.loading,
            view.loading,
            cx,
        ));
    }
    Some(bar.into_any_element())
}

pub(super) fn fidelity_label(value: &str, cx: &App) -> &'static str {
    match value {
        "full" => text(cx, "Full record", "完整记录"),
        "partial" => text(cx, "Partial", "部分"),
        "locked" => text(cx, "Locked", "锁定"),
        _ => text(cx, "Unknown", "未知"),
    }
}

pub(super) fn session_title(row: &AgentSessionArchiveRow) -> String {
    row.title
        .clone()
        .filter(|title| !title.trim().is_empty())
        .unwrap_or_else(|| row.session_id.clone())
}

/// `platform · variant · N messages · N tools`.
pub(super) fn summary_line(row: &AgentSessionArchiveRow, cx: &App) -> String {
    let platform = AgentSessionAgentId::parse(&row.platform)
        .map(agent_name)
        .unwrap_or(row.platform.as_str());
    format!(
        "{platform} · {} · {}",
        row.source_variant,
        counts_line(row, cx)
    )
}

/// `N messages · N tools`; the tool count is omitted when no tool was used.
pub(super) fn counts_line(row: &AgentSessionArchiveRow, cx: &App) -> String {
    let mut line = format!("{} {}", row.message_count, text(cx, "messages", "条消息"));
    if row.tool_use_count > 0 {
        line.push_str(&format!(
            " · {} {}",
            row.tool_use_count,
            text(cx, "tools", "次工具")
        ));
    }
    line
}

// 一行 60px：身份圆点 | 标题（单行省略 + tooltip）与时间 / 摘要、路径与异常标签。
fn session_row(
    index: usize,
    total: usize,
    row: &AgentSessionArchiveRow,
    selected: bool,
    cx: &mut Context<InsightsView>,
) -> AnyElement {
    let colors = cx.theme();
    let mode = colors.mode;
    let title = session_title(row);
    let when = row
        .updated_at
        .with_timezone(&chrono::Local)
        .format("%m-%d %H:%M")
        .to_string();
    let summary = summary_line(row, cx);
    let label = format!("{title} · {summary} · {when}");
    let weak = cx.weak_entity();
    let content = div()
        .flex_1()
        .min_h_0()
        .w_full()
        .px(px(theme::CONTROL_GAP))
        .flex()
        .items_center()
        .gap(px(theme::CONTROL_GAP + 4.))
        .rounded(px(theme::SELECTION_RADIUS))
        .bg(if selected {
            colors.secondary
        } else {
            gpui_kit::transparent_black()
        })
        .child(badge::dot(theme::platform_color(&row.platform, mode)))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .gap(px(2.))
                .child(title_line(row, title, when, cx))
                .child(meta_line(row, summary, cx)),
        );
    div()
        .id(SharedString::from(format!(
            "insights-row:{}",
            row.archive_id
        )))
        .test_support()
        .role(Role::ListBoxOption)
        .aria_label(label)
        .aria_selected(selected)
        .aria_position_in_set(index + 1)
        .aria_size_of_set(total)
        .on_a11y_action(gpui_kit::AccessibleAction::Click, move |_, window, cx| {
            let _ = weak.update(cx, |this, cx| this.open_session(index, window, cx));
        })
        .h(px(theme::TWO_LINE_ROW_HEIGHT))
        .w_full()
        .flex()
        .flex_col()
        .cursor_pointer()
        .hover(|style| style.bg(colors.muted))
        .on_click(cx.listener(move |this, _, window, cx| this.open_session(index, window, cx)))
        .child(content)
        .child(
            div()
                .flex_shrink_0()
                .mx(px(theme::CONTROL_GAP))
                .h(px(1.))
                .bg(theme::hairline(mode)),
        )
        .into_any_element()
}

fn title_line(row: &AgentSessionArchiveRow, title: String, when: String, cx: &App) -> Div {
    let tip: SharedString = title.clone().into();
    div()
        .w_full()
        .min_w_0()
        .flex()
        .items_center()
        .gap(px(theme::CONTROL_GAP))
        .child(
            div()
                .id(SharedString::from(format!(
                    "session-row-title:{}",
                    row.archive_id
                )))
                .test_support()
                .flex_1()
                .min_w_0()
                .truncate()
                .text_size(px(theme::BODY_SIZE))
                .line_height(px(20.))
                .font_weight(FontWeight::MEDIUM)
                .tooltip(move |window, cx| Tooltip::new(tip.clone()).build(window, cx))
                .child(title),
        )
        .child(
            div()
                .flex_shrink_0()
                .text_size(px(theme::META_SIZE))
                .text_color(cx.theme().muted_foreground)
                .font_features(theme::tabular_figures())
                .child(when),
        )
}

// 第二行：摘要（平台 · 变体 · 计数）从不收缩、不省略；只有项目路径可伸缩并省略，
// 分隔符 `·` 属于路径元素。连摘要都放不下时，路径宽度收到 0，整段隐藏。
fn meta_line(row: &AgentSessionArchiveRow, summary: String, cx: &App) -> Div {
    let line = div()
        .w_full()
        .min_w_0()
        .overflow_hidden()
        .flex()
        .items_center()
        .gap(px(theme::SELECTION_RADIUS))
        .text_size(px(theme::META_SIZE))
        .line_height(px(theme::TAG_HEIGHT))
        .text_color(cx.theme().muted_foreground)
        .child(
            div()
                .id(SharedString::from(format!(
                    "session-row-meta:{}",
                    row.archive_id
                )))
                .test_support()
                .flex_shrink_0()
                .whitespace_nowrap()
                .font_features(theme::tabular_figures())
                .child(summary),
        );
    let line = if row.cwd.is_empty() {
        line.child(div().flex_1())
    } else {
        let tip: SharedString = row.cwd.clone().into();
        line.child(
            div()
                .id(SharedString::from(format!(
                    "session-row-cwd:{}",
                    row.archive_id
                )))
                .test_support()
                .flex_1()
                .min_w_0()
                .truncate()
                .font_family(cx.theme().mono_font_family.clone())
                .tooltip(move |window, cx| Tooltip::new(tip.clone()).build(window, cx))
                .child(format!("· {}", row.cwd)),
        )
    };
    row_tags(row, line, cx)
}

fn row_tags(row: &AgentSessionArchiveRow, line: Div, cx: &App) -> Div {
    let line = match row.source_fidelity.as_str() {
        "full" => line,
        value => line.child(badge::tag(
            SharedString::from(format!("session-row-fidelity:{}", row.archive_id)),
            fidelity_label(value, cx),
            if value == "partial" || value == "locked" {
                Tone::Warning
            } else {
                Tone::Neutral
            },
            cx,
        )),
    };
    match row.source_state {
        UsageSourceState::Live => line,
        UsageSourceState::Missing => line.child(badge::tag(
            SharedString::from(format!("session-row-state:{}", row.archive_id)),
            text(cx, "Missing", "缺失"),
            Tone::Warning,
            cx,
        )),
        UsageSourceState::DeletedByUser => line.child(badge::tag(
            SharedString::from(format!("session-row-state:{}", row.archive_id)),
            text(cx, "Deleted", "已删除"),
            Tone::Neutral,
            cx,
        )),
    }
}
