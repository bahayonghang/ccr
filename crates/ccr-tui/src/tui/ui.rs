// TUI UI rendering module
// Renders dynamic multi-platform profile switcher interface

use super::app::App;
use super::claude_auth;
use super::codex_auth;
use super::footer::{ShortcutHint, shortcut_line};
use super::opencode_auth;
use super::theme;
use super::toast::ToastKind;
use super::usage::app::UsageLoadState;
use super::usage::ui::{format_cost, format_count};
use ccr_cli::models::{CodexRuntimeSummary, OpenAiAuthMethod, Platform, ProfileConfig};
use ccr_codex::CodexPlatform;
use ccr_usage::{ProviderBreakdownDto, SourceKind};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{
        Block, Borders, List, ListItem, Padding, Paragraph, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Tabs, Wrap,
    },
};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

// ═══════════════════════════════════════════════════════════
// Main render entry
// ═══════════════════════════════════════════════════════════

/// Render the main UI (responsive to terminal size)
pub fn draw(f: &mut Frame, app: &mut App) {
    let background = Block::default().style(theme::background_style());
    f.render_widget(background, f.area());

    let area = f.area();
    let mode = theme::viewport_mode(area.width, area.height);
    app.detail_area.set(None);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(root_constraints(mode))
        .split(area);

    render_header(f, app, chunks[0]);

    let content_area = if app.current_platform() == Platform::Codex && !app.is_opencode_auth_tab() {
        let runtime_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(if mode == theme::ViewportMode::Compact {
                    3
                } else {
                    4
                }),
                Constraint::Min(0),
            ])
            .split(chunks[1]);

        let runtime_summary = if app.is_codex_auth_tab() {
            app.codex_auth_app
                .as_ref()
                .and_then(|codex_app| codex_app.runtime_summary.as_ref())
        } else {
            app.current_codex_runtime_summary()
        };

        render_codex_runtime_banner(
            f,
            runtime_chunks[0],
            runtime_summary,
            mode == theme::ViewportMode::Compact,
        );
        runtime_chunks[1]
    } else {
        chunks[1]
    };

    if app.is_claude_auth_tab() {
        app.header_area.set(Some(chunks[0]));

        if let Some(ref mut claude_app) = app.claude_auth_app {
            claude_auth::ui::draw_embedded(f, claude_app, content_area, chunks[2], mode);
        } else {
            claude_auth::ui::draw_loading_placeholder(
                f,
                content_area,
                chunks[2],
                mode,
                app.claude_auth_error.as_deref(),
            );
        }
    } else if app.is_codex_auth_tab() {
        app.header_area.set(Some(chunks[0]));

        if let Some(ref mut codex_app) = app.codex_auth_app {
            codex_auth::ui::draw_embedded(f, codex_app, content_area, chunks[2], mode);
        } else {
            codex_auth::ui::draw_loading_placeholder(
                f,
                content_area,
                chunks[2],
                mode,
                app.codex_auth_error.as_deref(),
            );
        }
    } else if app.is_opencode_auth_tab() {
        app.header_area.set(Some(chunks[0]));

        if let Some(ref mut opencode_app) = app.opencode_auth_app {
            opencode_auth::ui::draw_embedded(f, opencode_app, content_area, chunks[2], mode);
        } else {
            opencode_auth::ui::draw_loading_placeholder(
                f,
                content_area,
                chunks[2],
                mode,
                app.opencode_auth_error.as_deref(),
            );
        }
    } else {
        app.header_area.set(Some(chunks[0]));

        render_profile_workspace(f, app, content_area, mode);
        render_footer(f, app, chunks[2]);
    }
}

fn render_codex_runtime_banner(
    f: &mut Frame,
    area: Rect,
    summary: Option<&CodexRuntimeSummary>,
    compact: bool,
) {
    let (mode_label, mode_style, profile_label, auth_label) = if let Some(summary) = summary {
        (
            codex_runtime_mode_label(summary.mode).to_string(),
            runtime_mode_style(summary.mode),
            localize_codex_runtime_text(summary.profile_label()),
            localize_codex_runtime_text(summary.auth_label()),
        )
    } else {
        (
            crate::tui_text!("Unresolved", "未解析").to_string(),
            theme::muted_style(),
            "-".to_string(),
            "-".to_string(),
        )
    };

    let lines = if compact {
        vec![Line::from(vec![
            Span::styled(
                crate::tui_text!(" Active driver: ", " 当前驱动："),
                theme::secondary_text_style(),
            ),
            Span::styled(mode_label, mode_style),
        ])]
    } else {
        vec![
            Line::from(vec![
                Span::styled(
                    crate::tui_text!(" Active driver: ", " 当前驱动："),
                    theme::secondary_text_style(),
                ),
                Span::styled(mode_label, mode_style),
            ]),
            Line::from(vec![
                Span::styled(
                    crate::tui_text!(" Profile: ", " 配置："),
                    theme::secondary_text_style(),
                ),
                Span::styled(profile_label, theme::primary_text_style()),
                Span::styled("  │  ", Style::default().fg(theme::border())),
                Span::styled(
                    crate::tui_text!("Auth: ", "认证："),
                    theme::secondary_text_style(),
                ),
                Span::styled(auth_label, Style::default().fg(theme::success())),
            ]),
        ]
    };

    let banner = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_set(symbols::border::ROUNDED)
                .border_style(Style::default().fg(theme::codex()))
                .title(crate::tui_text!(" Control plane ", " 当前控制面 "))
                .title_style(theme::codex_style()),
        )
        .wrap(Wrap { trim: true });

    f.render_widget(banner, area);
}

fn runtime_mode_style(mode: ccr_cli::models::CodexRuntimeMode) -> Style {
    match mode {
        ccr_cli::models::CodexRuntimeMode::ProfileOnly => theme::success_style(),
        ccr_cli::models::CodexRuntimeMode::ProfileWithAuth
        | ccr_cli::models::CodexRuntimeMode::ProfilePendingAuth => theme::warning_style(),
        ccr_cli::models::CodexRuntimeMode::RuntimeOnly => theme::secondary_text_emphasis_style(),
        ccr_cli::models::CodexRuntimeMode::Unresolved => theme::muted_style(),
    }
}

fn root_constraints(mode: theme::ViewportMode) -> Vec<Constraint> {
    vec![
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(theme::footer_height(mode)),
    ]
}

fn wide_profile_workspace_layout(area: Rect) -> (Rect, Rect) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(46), Constraint::Percentage(54)])
        .split(area);
    (columns[0], columns[1])
}

fn profile_list_rail_layout(area: Rect) -> (Rect, Rect) {
    // Selection 面板 3 行内容 + 上下边框 = 5; keys 行已并入全局 footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(8), Constraint::Length(5)])
        .split(area);
    (chunks[0], chunks[1])
}

fn profile_context_constraints(summary_height: u16, status_visible: bool) -> Vec<Constraint> {
    let mut constraints = vec![Constraint::Length(summary_height), Constraint::Min(10)];
    if status_visible {
        constraints.push(Constraint::Length(3));
    }
    constraints
}

fn compact_profile_workspace_layout(area: Rect) -> Option<(Rect, Rect)> {
    if area.height < 12 {
        return None;
    }

    let context_height = area.height.saturating_sub(6).clamp(6, 10);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(context_height)])
        .split(area);

    Some((chunks[0], chunks[1]))
}

/// Calculate column widths for profile list (responsive to terminal width)
/// Returns (name_width, desc_width) — desc_width is 0 when terminal is narrow
fn column_widths(area_width: u16) -> (usize, usize) {
    let inner = area_width.saturating_sub(4) as usize;
    let gap = 2usize;
    let available = inner.saturating_sub(gap);

    // Narrow terminal: name only, no description
    if area_width < 52 {
        return (available, 0);
    }

    let min_name = 18usize;
    let min_desc = 14usize;
    let mut name_width = available * 2 / 5;
    if name_width < min_name {
        name_width = min_name;
    }
    let max_name = available.saturating_sub(min_desc);
    if max_name == 0 {
        name_width = available;
    } else if name_width > max_name {
        name_width = max_name;
    }
    let desc_width = available.saturating_sub(name_width);
    (name_width, desc_width)
}

// 截断/填充一律按终端显示宽度计数 (CJK 为 2 列), 不能按字符数,
// 否则含中文的单元格会溢出列宽被 ratatui 硬裁剪、省略号丢失。
fn truncate_text(text: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    if text.width() <= width {
        return text.to_string();
    }
    if width == 1 {
        return "…".to_string();
    }

    // 预留 1 列给省略号; 剩余宽度为奇数时宁短 1 列也不溢出
    let mut out = String::new();
    let mut used = 0usize;
    for ch in text.chars() {
        let ch_width = ch.width().unwrap_or(0);
        if used + ch_width > width - 1 {
            break;
        }
        out.push(ch);
        used += ch_width;
    }
    out.push('…');
    out
}

fn pad_text(text: &str, width: usize) -> String {
    let text_width = text.width();
    if text_width >= width {
        return text.to_string();
    }
    let mut out = String::with_capacity(text.len() + width - text_width);
    out.push_str(text);
    out.extend(std::iter::repeat_n(' ', width - text_width));
    out
}

fn profile_list_row(
    profile: &crate::tui::app::ProfileItem,
    is_selected: bool,
    name_width: usize,
    desc_width: usize,
) -> Line<'static> {
    let selected_style = theme::selected_row_style();
    let selector = if is_selected { "▶ " } else { "  " };
    let current_marker = if profile.is_current { "●" } else { "○" };
    let desc = profile.description.as_deref().unwrap_or("").trim();
    let current_tag = if profile.is_current { " ✓" } else { "" };
    let name_raw = format!(
        "{}{} {}{}",
        selector, current_marker, profile.name, current_tag
    );
    let name_cell = pad_text(&truncate_text(&name_raw, name_width), name_width);

    let name_style = if is_selected {
        selected_style
    } else if profile.is_current {
        theme::list_current_style().add_modifier(Modifier::BOLD)
    } else {
        theme::list_normal_style()
    };

    let line_spans = if desc_width > 0 && !desc.is_empty() {
        let desc_cell = pad_text(&truncate_text(desc, desc_width), desc_width);
        let desc_style = if is_selected {
            selected_style
        } else if profile.is_current {
            theme::secondary_text_emphasis_style()
        } else {
            theme::muted_style()
        };
        vec![
            Span::styled(name_cell, name_style),
            Span::styled("  ", Style::default().fg(theme::border())),
            Span::styled(desc_cell, desc_style),
        ]
    } else {
        vec![Span::styled(name_cell, name_style)]
    };

    Line::from(line_spans)
}

// ═══════════════════════════════════════════════════════════
// Header and Tab rendering
// ═══════════════════════════════════════════════════════════

/// Render header with dynamic platform tabs
fn render_header(f: &mut Frame, app: &App, area: Rect) {
    // Build tab titles dynamically from loaded platforms
    let compact_tabs = area.width < 100;
    let tab_titles: Vec<Line> = app
        .tabs
        .iter()
        .enumerate()
        .map(|(i, tab)| tab_title_line(tab, i == app.active_tab, compact_tabs))
        .collect();

    let current_label = format!(
        " {} {} ",
        app.current_platform().icon(),
        app.current_tab().display_label()
    );

    let tabs = Tabs::new(tab_titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_set(symbols::border::ROUNDED)
                .border_style(Style::default().fg(theme::border()))
                .title(crate::tui_text!(
                    " CCR - Configuration Switcher ",
                    " CCR - 配置切换器 "
                ))
                .title_alignment(Alignment::Center)
                .title_style(theme::primary_text_emphasis_style())
                .title_bottom(
                    Line::from(Span::styled(
                        current_label,
                        theme::platform_style_for(app.current_platform()),
                    ))
                    .alignment(Alignment::Right),
                ),
        )
        .select(app.active_tab)
        .style(theme::tab_normal_style())
        .highlight_style(theme::tab_highlight_style_for(app.current_platform()))
        .divider(Span::styled("  │  ", Style::default().fg(theme::border())));

    f.render_widget(tabs, area);
}

fn tab_title_line(
    tab: &crate::tui::app::PlatformTab,
    is_active: bool,
    compact: bool,
) -> Line<'static> {
    let label = if compact {
        tab.compact_display_label()
    } else {
        tab.display_label()
    };
    let style = if is_active {
        theme::tab_active_style_for(tab.platform)
    } else {
        theme::tab_inactive_style()
    };
    let title = if is_active {
        format!(" {} {} ", tab.platform.icon(), label)
    } else {
        format!("  {} {} ", tab.platform.icon(), label)
    };

    Line::from(Span::styled(title, style))
}

// ═══════════════════════════════════════════════════════════
// Profile list rendering
// ═══════════════════════════════════════════════════════════

fn render_profile_workspace(f: &mut Frame, app: &mut App, area: Rect, mode: theme::ViewportMode) {
    match mode {
        theme::ViewportMode::Compact => {
            if let Some((list_area, context_area)) = compact_profile_workspace_layout(area) {
                render_profile_list_panel(f, app, list_area);
                render_profile_context_workspace(f, app, context_area, mode);
            } else {
                render_profile_list_panel(f, app, area);
            }
        }
        theme::ViewportMode::Standard => {
            let detail_height = area.height.saturating_sub(8).clamp(11, 14);
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(8), Constraint::Length(detail_height)])
                .split(area);

            render_profile_list_panel(f, app, chunks[0]);
            render_profile_context_workspace(f, app, chunks[1], mode);
        }
        theme::ViewportMode::Wide => {
            let (list_area, context_area) = wide_profile_workspace_layout(area);

            render_profile_list_rail(f, app, list_area);
            render_profile_context_workspace(f, app, context_area, mode);
        }
    }
}

fn render_profile_list_rail(f: &mut Frame, app: &mut App, area: Rect) {
    let (list_area, meta_area) = profile_list_rail_layout(area);

    render_profile_list_panel(f, app, list_area);
    render_profile_meta_panel(f, app, meta_area);
}

fn render_profile_list_panel(f: &mut Frame, app: &mut App, area: Rect) {
    app.list_area.set(Some(area));
    let content_height = Block::default()
        .borders(Borders::ALL)
        .padding(Padding::horizontal(1))
        .inner(area)
        .height;
    app.sync_profile_page_size(super::pagination::visible_page_size(content_height));

    let profiles = app.current_page_profiles();
    let all_profiles = app.current_profiles();
    let platform = app.current_platform();
    let platform_name = platform.display_name();
    let accent = theme::platform_selection_color_for(platform);

    let total_pages = app.total_pages();
    let total_profiles = all_profiles.len();
    let visible_start = if total_profiles == 0 {
        0
    } else {
        app.current_page * app.page_size + 1
    };
    let visible_end = if total_profiles == 0 {
        0
    } else {
        app.current_page * app.page_size + profiles.len()
    };
    let title = if all_profiles.is_empty() {
        crate::tui_format!(" {} Profiles ", " {} 配置 ", platform_name)
    } else if total_pages > 1 {
        crate::tui_format!(
            " {} Profiles ({})  {}-{} / {}  Page {}/{} ",
            " {} 配置 ({})  {}-{} / {}  第 {}/{} 页 ",
            platform_name,
            total_profiles,
            visible_start,
            visible_end,
            total_profiles,
            app.current_page + 1,
            total_pages
        )
    } else {
        crate::tui_format!(
            " {} Profiles ({}) ",
            " {} 配置 ({}) ",
            platform_name,
            total_profiles
        )
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(symbols::border::ROUNDED)
        .border_style(Style::default().fg(accent))
        .title(title)
        .title_alignment(Alignment::Left)
        .title_style(theme::platform_style_for(platform))
        .padding(Padding::horizontal(1));

    if profiles.is_empty() {
        render_empty_state(f, app, area, block);
        return;
    }

    let (name_width, desc_width) = column_widths(area.width);

    let items: Vec<ListItem> = profiles
        .iter()
        .enumerate()
        .map(|(i, profile)| {
            ListItem::new(profile_list_row(
                profile,
                i == app.selected_index,
                name_width,
                desc_width,
            ))
        })
        .collect();

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

fn render_profile_meta_panel(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(symbols::border::ROUNDED)
        .border_style(Style::default().fg(theme::border()))
        .title(crate::tui_text!(" Selection ", " 选择 "))
        .title_style(theme::secondary_text_emphasis_style())
        .padding(Padding::horizontal(1));

    let lines: Vec<Line> = profile_meta_strings(
        app.current_profiles().len(),
        app.current_page,
        app.total_pages(),
        app.selected_profile(),
    )
    .into_iter()
    .map(Line::from)
    .collect();

    let paragraph = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}

fn render_profile_context_workspace(
    f: &mut Frame,
    app: &mut App,
    area: Rect,
    mode: theme::ViewportMode,
) {
    let Some(profile) = app.selected_profile() else {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_set(symbols::border::ROUNDED)
            .border_style(Style::default().fg(theme::platform_color_for(app.current_platform())))
            .title(crate::tui_text!(" Focus ", " 当前焦点 "))
            .title_style(theme::platform_style_for(app.current_platform()))
            .padding(Padding::horizontal(1));

        let paragraph = Paragraph::new(profile_status_text(app))
            .block(block)
            .alignment(profile_status_alignment(app))
            .wrap(Wrap { trim: false });
        f.render_widget(paragraph, area);
        return;
    };

    let Some(config) = app.selected_profile_config() else {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_set(symbols::border::ROUNDED)
            .border_style(Style::default().fg(theme::platform_color_for(app.current_platform())))
            .title(crate::tui_text!(" Focus ", " 当前焦点 "))
            .title_style(theme::platform_style_for(app.current_platform()))
            .padding(Padding::horizontal(1));

        let paragraph = Paragraph::new(profile_status_text(app))
            .block(block)
            .alignment(profile_status_alignment(app))
            .wrap(Wrap { trim: false });
        f.render_widget(paragraph, area);
        return;
    };
    let profile_name = profile.name.clone();
    let is_current = profile.is_current;

    // Focus 高度随内容收缩 (2-3 行 + 边框), 让出的行给 Context 详情
    let summary = profile_summary_fields(
        app.current_platform(),
        profile_name.as_str(),
        config,
        is_current,
    );
    let summary_height = summary.len() as u16 + 2;

    if mode == theme::ViewportMode::Wide {
        let status_visible = profile_status_message(app, profile_name.as_str()).is_some();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(profile_context_constraints(summary_height, status_visible))
            .split(area);

        render_profile_summary_block(f, app, chunks[0], summary);
        render_profile_details(f, app, chunks[1], mode);
        if status_visible {
            render_profile_status_strip(f, app, chunks[2], profile_name.as_str());
        }
    } else {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(summary_height), Constraint::Min(0)])
            .split(area);

        render_profile_summary_block(f, app, chunks[0], summary);
        render_profile_details(f, app, chunks[1], mode);
    }
}

fn render_profile_details(f: &mut Frame, app: &mut App, area: Rect, mode: theme::ViewportMode) {
    app.detail_area.set(Some(area));
    let platform = app.current_platform();
    let accent = theme::platform_color_for(platform);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(symbols::border::ROUNDED)
        .border_style(Style::default().fg(accent))
        .title(crate::tui_text!(" Context ", " 上下文 "))
        .title_style(theme::platform_style_for(platform))
        .padding(Padding::horizontal(1));

    let Some(profile) = app.selected_profile() else {
        let paragraph = Paragraph::new(profile_status_text(app))
            .block(block)
            .alignment(profile_status_alignment(app))
            .wrap(Wrap { trim: false });
        f.render_widget(paragraph, area);
        return;
    };

    let Some(config) = app.selected_profile_config() else {
        let paragraph = Paragraph::new(profile_status_text(app))
            .block(block)
            .alignment(profile_status_alignment(app))
            .wrap(Wrap { trim: false });
        f.render_widget(paragraph, area);
        return;
    };

    // 用量引擎状态只读传入详情行构造(未初始化 = 加载中);渲染循环零 I/O
    let usage_state = app.usage_app.as_ref().map(|engine| &engine.state);
    let compact = mode == theme::ViewportMode::Compact;
    let lines = if platform == Platform::Codex {
        codex_profile_detail_lines(
            profile.name.as_str(),
            config,
            profile.is_current,
            usage_state,
            compact,
        )
    } else if platform == Platform::Claude {
        claude_profile_detail_lines(
            profile.name.as_str(),
            config,
            profile.is_current,
            usage_state,
            compact,
        )
    } else {
        generic_profile_detail_lines(
            profile.name.as_str(),
            config,
            profile.is_current,
            platform,
            compact,
        )
    };

    let inner_height = Block::default()
        .borders(Borders::ALL)
        .padding(Padding::horizontal(1))
        .inner(area)
        .height as usize;
    let max_scroll = lines.len().saturating_sub(inner_height);
    let scroll = (app.profile_detail_scroll as usize).min(max_scroll);
    app.profile_detail_scroll = scroll as u16;

    let paragraph = Paragraph::new(lines.clone())
        .block(block)
        .scroll((scroll as u16, 0))
        .wrap(Wrap { trim: false });
    f.render_widget(paragraph, area);

    if max_scroll > 0 {
        let mut scrollbar_state = ScrollbarState::new(lines.len()).position(scroll);
        f.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"))
                .thumb_symbol("█")
                .track_symbol(Some("│")),
            area,
            &mut scrollbar_state,
        );
    }
}

fn render_profile_summary_block(
    f: &mut Frame,
    app: &App,
    area: Rect,
    summary: Vec<ProfileSummaryField>,
) {
    let platform = app.current_platform();
    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(symbols::border::ROUNDED)
        .border_style(Style::default().fg(theme::platform_color_for(platform)))
        .title(crate::tui_text!(" Focus ", " 当前焦点 "))
        .title_style(theme::platform_style_for(platform))
        .padding(Padding::horizontal(1));

    let lines: Vec<Line> = summary.into_iter().map(profile_summary_line).collect();

    let paragraph = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}

fn render_profile_status_strip(f: &mut Frame, app: &App, area: Rect, profile_name: &str) {
    let Some(text) = profile_status_message(app, profile_name) else {
        return;
    };
    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(theme::border()))
        .title(crate::tui_text!(" Status ", " 状态 "))
        .title_style(theme::secondary_text_emphasis_style());

    // 快捷键只保留底部全局 Keys footer 一处; strip 只反馈 apply 结果/toast
    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}

fn profile_status_message(app: &App, profile_name: &str) -> Option<String> {
    last_apply_message(profile_name, app.last_applied.as_ref())
        .or_else(|| app.toasts.active().map(|toast| toast.message.clone()))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DetailKey {
    Description,
    BaseUrl,
    Model,
    ReasoningEffort,
    SmallFast,
    Account,
    SwitchCount,
    Tags,
    ProviderType,
    Provider,
    AuthMode,
    AuthSource,
    Token,
    OpenAiLogin,
    EnvKey,
    WireApi,
    RequiresOpenAi,
    Requests,
    Tokens,
    Input,
    Output,
    Cache,
    Total,
    ApproxCost,
    Note,
}

impl DetailKey {
    const ALL: [Self; 25] = [
        Self::Description,
        Self::BaseUrl,
        Self::Model,
        Self::ReasoningEffort,
        Self::SmallFast,
        Self::Account,
        Self::SwitchCount,
        Self::Tags,
        Self::ProviderType,
        Self::Provider,
        Self::AuthMode,
        Self::AuthSource,
        Self::Token,
        Self::OpenAiLogin,
        Self::EnvKey,
        Self::WireApi,
        Self::RequiresOpenAi,
        Self::Requests,
        Self::Tokens,
        Self::Input,
        Self::Output,
        Self::Cache,
        Self::Total,
        Self::ApproxCost,
        Self::Note,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::Description => crate::tui_text!("description", "描述"),
            Self::BaseUrl => crate::tui_text!("base_url", "基础地址"),
            Self::Model => crate::tui_text!("model", "模型"),
            Self::ReasoningEffort => crate::tui_text!("reasoning_effort", "推理强度"),
            Self::SmallFast => crate::tui_text!("small_fast", "快速模型"),
            Self::Account => crate::tui_text!("account", "账号"),
            Self::SwitchCount => crate::tui_text!("switch_count", "切换次数"),
            Self::Tags => crate::tui_text!("tags", "标签"),
            Self::ProviderType => crate::tui_text!("provider_type", "提供商类型"),
            Self::Provider => crate::tui_text!("provider", "提供商"),
            Self::AuthMode => crate::tui_text!("auth_mode", "认证模式"),
            Self::AuthSource => crate::tui_text!("auth_source", "认证来源"),
            Self::Token => crate::tui_text!("token", "令牌"),
            Self::OpenAiLogin => crate::tui_text!("openai_login", "OpenAI 登录"),
            Self::EnvKey => crate::tui_text!("env_key", "环境变量键"),
            Self::WireApi => crate::tui_text!("wire_api", "协议 API"),
            Self::RequiresOpenAi => crate::tui_text!("requires_openai", "需要 OpenAI"),
            Self::Requests => crate::tui_text!("requests", "请求数"),
            Self::Tokens => crate::tui_text!("tokens", "令牌数"),
            Self::Input => crate::tui_text!("input", "输入"),
            Self::Output => crate::tui_text!("output", "输出"),
            Self::Cache => crate::tui_text!("cache", "缓存"),
            Self::Total => crate::tui_text!("total", "总计"),
            Self::ApproxCost => crate::tui_text!("approx_cost", "估算费用"),
            Self::Note => crate::tui_text!("note", "说明"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DetailTone {
    Primary,
    Muted,
    Info,
    Success,
    Warning,
    StrongWarning,
    Cost,
    Accent { platform: Platform, strong: bool },
}

#[derive(Debug, Clone)]
struct DetailField {
    key: DetailKey,
    value: String,
    tone: DetailTone,
    emphasize_label: bool,
}

impl DetailField {
    fn new(key: DetailKey, value: String, tone: DetailTone) -> Self {
        Self {
            key,
            value,
            tone,
            emphasize_label: false,
        }
    }

    fn emphasized(mut self) -> Self {
        self.emphasize_label = true;
        self
    }
}

fn optional_tone(value: &str, present: DetailTone) -> DetailTone {
    if value == "-" {
        DetailTone::Muted
    } else {
        present
    }
}

fn generic_profile_detail_lines(
    _name: &str,
    config: &ProfileConfig,
    _is_current: bool,
    platform: Platform,
    compact: bool,
) -> Vec<Line<'static>> {
    let description = opt_text(config.description.as_deref());
    let base_url = opt_text(config.base_url.as_deref());
    let model = opt_text(config.model.as_deref());
    let account = opt_text(config.account.as_deref());
    vec![
        section_line(" Overview "),
        detail_line(
            DetailField::new(
                DetailKey::Description,
                description.clone(),
                optional_tone(&description, DetailTone::Primary),
            ),
            compact,
        ),
        Line::from(""),
        section_line(" Runtime "),
        detail_line(
            DetailField::new(
                DetailKey::BaseUrl,
                base_url.clone(),
                optional_tone(&base_url, DetailTone::Info),
            )
            .emphasized(),
            compact,
        ),
        detail_line(
            DetailField::new(
                DetailKey::Model,
                model.clone(),
                optional_tone(
                    &model,
                    DetailTone::Accent {
                        platform,
                        strong: true,
                    },
                ),
            )
            .emphasized(),
            compact,
        ),
        detail_line(
            DetailField::new(
                DetailKey::Account,
                account.clone(),
                optional_tone(&account, DetailTone::Info),
            ),
            compact,
        ),
        Line::from(""),
        section_line(" Activity "),
        detail_line(
            DetailField::new(
                DetailKey::SwitchCount,
                config.usage_count().to_string(),
                DetailTone::Primary,
            ),
            compact,
        ),
        detail_line(
            DetailField::new(DetailKey::Tags, tags_text(config), DetailTone::Primary),
            compact,
        ),
    ]
}

fn codex_profile_detail_lines(
    _name: &str,
    config: &ProfileConfig,
    _is_current: bool,
    usage: Option<&UsageLoadState>,
    compact: bool,
) -> Vec<Line<'static>> {
    let auth_mode = CodexPlatform::profile_auth_mode(config);
    let login_method =
        CodexPlatform::profile_openai_login_method(config).map(|method| match method {
            OpenAiAuthMethod::Chatgpt => "chatgpt".to_string(),
            OpenAiAuthMethod::Api => "api".to_string(),
        });
    let (token_state, token_tone) = match auth_mode.as_str() {
        "openai_api_key" | "provider_env_key" => match configured_token_text(config) {
            Some(token) => (token, DetailTone::Success),
            None => (
                crate::tui_text!("missing", "缺失").to_string(),
                DetailTone::Warning,
            ),
        },
        _ => ("-".to_string(), DetailTone::Muted),
    };
    let description = opt_text(config.description.as_deref());
    let provider_type = opt_text(config.provider_type.as_deref());
    let provider = opt_text(config.provider.as_deref());
    let auth_source = CodexPlatform::profile_auth_source(config);
    let env_key = opt_text(codex_platform_value(config, "env_key").as_deref());
    let wire_api = opt_text(codex_platform_value(config, "wire_api").as_deref());
    let requires_openai = codex_platform_value(config, "requires_openai_auth")
        .as_deref()
        .and_then(|value| match value {
            "true" => Some(true),
            "false" => Some(false),
            _ => None,
        });
    let requires_openai_text = bool_text(requires_openai).to_string();
    let base_url = opt_text(config.base_url.as_deref());
    let model = opt_text(config.model.as_deref());
    let small_fast = opt_text(config.small_fast_model.as_deref());
    let account = opt_text(config.account.as_deref());

    let mut lines = vec![
        section_line(" Overview "),
        detail_line(
            DetailField::new(
                DetailKey::Description,
                description.clone(),
                optional_tone(&description, DetailTone::Primary),
            ),
            compact,
        ),
        Line::from(""),
        section_line(" Engine "),
        detail_line(
            DetailField::new(
                DetailKey::BaseUrl,
                base_url.clone(),
                optional_tone(&base_url, DetailTone::Info),
            )
            .emphasized(),
            compact,
        ),
        detail_line(
            DetailField::new(
                DetailKey::Model,
                model.clone(),
                optional_tone(
                    &model,
                    DetailTone::Accent {
                        platform: Platform::Codex,
                        strong: true,
                    },
                ),
            )
            .emphasized(),
            compact,
        ),
        detail_line(codex_reasoning_effort_field(config), compact),
        detail_line(
            DetailField::new(
                DetailKey::SmallFast,
                small_fast.clone(),
                optional_tone(&small_fast, DetailTone::Info),
            ),
            compact,
        ),
        detail_line(
            DetailField::new(
                DetailKey::Account,
                account.clone(),
                optional_tone(&account, DetailTone::Info),
            ),
            compact,
        ),
        Line::from(""),
        section_line(" Routing/Auth "),
        detail_line(
            DetailField::new(
                DetailKey::ProviderType,
                provider_type.clone(),
                optional_tone(&provider_type, DetailTone::Info),
            ),
            compact,
        ),
        detail_line(
            DetailField::new(
                DetailKey::Provider,
                provider.clone(),
                optional_tone(
                    &provider,
                    DetailTone::Accent {
                        platform: Platform::Codex,
                        strong: false,
                    },
                ),
            )
            .emphasized(),
            compact,
        ),
        detail_line(
            DetailField::new(
                DetailKey::AuthMode,
                auth_mode.as_str().to_string(),
                DetailTone::Info,
            )
            .emphasized(),
            compact,
        ),
        detail_line(
            DetailField::new(
                DetailKey::AuthSource,
                auth_source.clone(),
                optional_tone(&auth_source, DetailTone::Info),
            ),
            compact,
        ),
        detail_line(
            DetailField::new(DetailKey::Token, token_state, token_tone).emphasized(),
            compact,
        ),
        detail_line(
            DetailField::new(
                DetailKey::OpenAiLogin,
                login_method.unwrap_or_else(|| "-".to_string()),
                DetailTone::Info,
            ),
            compact,
        ),
        detail_line(
            DetailField::new(
                DetailKey::EnvKey,
                env_key.clone(),
                optional_tone(&env_key, DetailTone::Info),
            ),
            compact,
        ),
        detail_line(
            DetailField::new(
                DetailKey::WireApi,
                wire_api.clone(),
                optional_tone(&wire_api, DetailTone::Info),
            ),
            compact,
        ),
        detail_line(
            DetailField::new(
                DetailKey::RequiresOpenAi,
                requires_openai_text,
                match requires_openai {
                    Some(true) => DetailTone::Success,
                    Some(false) => DetailTone::Warning,
                    None => DetailTone::Muted,
                },
            ),
            compact,
        ),
        Line::from(""),
        section_line(" Activity "),
        detail_line(
            DetailField::new(
                DetailKey::SwitchCount,
                config.usage_count().to_string(),
                DetailTone::Primary,
            ),
            compact,
        ),
        detail_line(
            DetailField::new(DetailKey::Tags, tags_text(config), DetailTone::Primary),
            compact,
        ),
    ];

    lines.extend(usage_section_lines(
        SourceKind::Codex,
        config.provider.as_deref(),
        usage,
        compact,
    ));

    lines
}

fn claude_profile_detail_lines(
    _name: &str,
    config: &ProfileConfig,
    _is_current: bool,
    usage: Option<&UsageLoadState>,
    compact: bool,
) -> Vec<Line<'static>> {
    let auth_mode = ccr_cli::platforms::ClaudePlatform::profile_auth_mode(config);
    let provider_type = opt_text(config.provider_type.as_deref());
    let provider = opt_text(config.provider.as_deref());
    let (token_state, token_tone) =
        if matches!(auth_mode, ccr_cli::models::ClaudeProfileAuthMode::ApiKey) {
            match configured_token_text(config) {
                Some(token) => (token, DetailTone::Success),
                None => (
                    crate::tui_text!("missing", "缺失").to_string(),
                    DetailTone::Warning,
                ),
            }
        } else {
            (
                crate::tui_text!("subscription", "订阅").to_string(),
                DetailTone::Info,
            )
        };
    let description = opt_text(config.description.as_deref());
    let base_url = opt_text(config.base_url.as_deref());
    let model = opt_text(config.model.as_deref());
    let small_fast = opt_text(config.small_fast_model.as_deref());
    let account = opt_text(config.account.as_deref());
    let auth_source = ccr_cli::platforms::ClaudePlatform::profile_auth_source(config);

    let mut lines = vec![
        section_line(" Overview "),
        detail_line(
            DetailField::new(
                DetailKey::Description,
                description.clone(),
                optional_tone(&description, DetailTone::Primary),
            ),
            compact,
        ),
        Line::from(""),
        section_line(" Engine "),
        detail_line(
            DetailField::new(
                DetailKey::BaseUrl,
                base_url.clone(),
                optional_tone(&base_url, DetailTone::Info),
            )
            .emphasized(),
            compact,
        ),
        detail_line(
            DetailField::new(
                DetailKey::Model,
                model.clone(),
                optional_tone(
                    &model,
                    DetailTone::Accent {
                        platform: Platform::Claude,
                        strong: true,
                    },
                ),
            )
            .emphasized(),
            compact,
        ),
        detail_line(
            DetailField::new(
                DetailKey::SmallFast,
                small_fast.clone(),
                optional_tone(&small_fast, DetailTone::Info),
            ),
            compact,
        ),
        detail_line(
            DetailField::new(
                DetailKey::Account,
                account.clone(),
                optional_tone(&account, DetailTone::Info),
            ),
            compact,
        ),
        Line::from(""),
        section_line(" Routing/Auth "),
        detail_line(
            DetailField::new(
                DetailKey::AuthMode,
                auth_mode.as_str().to_string(),
                DetailTone::Info,
            )
            .emphasized(),
            compact,
        ),
        detail_line(
            DetailField::new(
                DetailKey::AuthSource,
                auth_source.clone(),
                optional_tone(&auth_source, DetailTone::Info),
            ),
            compact,
        ),
        detail_line(
            DetailField::new(DetailKey::Token, token_state, token_tone).emphasized(),
            compact,
        ),
    ];

    if provider_type != "-" {
        lines.push(detail_line(
            DetailField::new(DetailKey::ProviderType, provider_type, DetailTone::Info),
            compact,
        ));
    }

    if provider != "-" {
        lines.push(detail_line(
            DetailField::new(
                DetailKey::Provider,
                provider,
                DetailTone::Accent {
                    platform: Platform::Claude,
                    strong: false,
                },
            )
            .emphasized(),
            compact,
        ));
    }

    lines.extend([
        Line::from(""),
        section_line(" Activity "),
        detail_line(
            DetailField::new(
                DetailKey::SwitchCount,
                config.usage_count().to_string(),
                DetailTone::Primary,
            ),
            compact,
        ),
        detail_line(
            DetailField::new(DetailKey::Tags, tags_text(config), DetailTone::Primary),
            compact,
        ),
    ]);

    lines.extend(usage_section_lines(
        SourceKind::Claude,
        config.provider.as_deref(),
        usage,
        compact,
    ));

    lines
}

// ═══════════════════════════════════════════════════════════
// Profile detail: Usage section (provider-level)
// ═══════════════════════════════════════════════════════════

// 详情面板的 Usage 分组: 数据来自 App 级用量引擎一次性加载的数据集,
// 这里只做纯内存查找与格式化;六种状态都渲染为组内单行,不打断整页详情。
// 归因粒度是 provider 级 —— 共享同一 provider 的多个 profile 数字相同。
fn usage_section_lines(
    platform: SourceKind,
    provider: Option<&str>,
    state: Option<&UsageLoadState>,
    compact: bool,
) -> Vec<Line<'static>> {
    let title = match provider {
        Some(name) => crate::tui_format!(" Usage (provider: {name}) ", " 用量（提供商：{name}） "),
        None => crate::tui_text!(" Usage ", " 用量 ").to_string(),
    };
    let mut lines = vec![Line::from(""), section_line(&title)];

    let Some(provider) = provider else {
        // profile 未填 provider 字段时激活事件无归因标签。不回退展示
        // unattributed 桶 —— 那里混着全部历史未归因用量,数字会误导。
        lines.push(usage_status_line(
            crate::tui_text!(
                "no provider label — usage unattributed",
                "缺少提供商标签，无法归因用量"
            ),
            theme::muted_style(),
        ));
        return lines;
    };

    match state {
        None | Some(UsageLoadState::Idle | UsageLoadState::Loading) => {
            lines.push(usage_status_line(
                crate::tui_text!("loading...", "加载中..."),
                theme::muted_style(),
            ));
        }
        Some(UsageLoadState::Unsupported(message)) => {
            lines.push(usage_status_line(message, theme::warning_style()));
        }
        Some(UsageLoadState::Error(message)) => {
            lines.push(usage_status_line(message, theme::error_style()));
        }
        Some(UsageLoadState::Empty) => {
            lines.push(usage_status_line(
                crate::tui_text!("no usage recorded", "暂无用量记录"),
                theme::muted_style(),
            ));
        }
        Some(UsageLoadState::Loaded(dataset)) => {
            match dataset
                .platform_rows(platform)
                .find(|row| row.breakdown.provider.as_deref() == Some(provider))
            {
                Some(row) => lines.extend(usage_metric_lines(&row.breakdown, compact)),
                None => {
                    lines.push(usage_status_line(
                        crate::tui_text!("no usage recorded", "暂无用量记录"),
                        theme::muted_style(),
                    ));
                }
            }
        }
    }

    lines
}

fn usage_status_line(text: &str, style: Style) -> Line<'static> {
    Line::from(Span::styled(text.to_string(), style))
}

fn usage_metric_lines(breakdown: &ProviderBreakdownDto, compact: bool) -> Vec<Line<'static>> {
    if compact {
        // Compact 视口合并为 3 行,控制详情面板高度
        return vec![
            detail_line(
                DetailField::new(
                    DetailKey::Requests,
                    format_count(breakdown.request_count),
                    DetailTone::Primary,
                ),
                true,
            ),
            detail_line(
                DetailField::new(
                    DetailKey::Tokens,
                    crate::tui_format!(
                        "in {} · out {} · cache {}",
                        "输入 {} · 输出 {} · 缓存 {}",
                        format_count(breakdown.input_tokens),
                        format_count(breakdown.output_tokens_total()),
                        format_count(breakdown.cache_tokens_total()),
                    ),
                    DetailTone::Primary,
                ),
                true,
            ),
            detail_line(
                DetailField::new(
                    DetailKey::ApproxCost,
                    format_cost(breakdown.cost_with_cache_usd),
                    DetailTone::Cost,
                )
                .emphasized(),
                true,
            ),
        ];
    }

    vec![
        detail_line(
            DetailField::new(
                DetailKey::Requests,
                format_count(breakdown.request_count),
                DetailTone::Primary,
            ),
            false,
        ),
        detail_line(
            DetailField::new(
                DetailKey::Input,
                format_count(breakdown.input_tokens),
                DetailTone::Primary,
            ),
            false,
        ),
        detail_line(
            DetailField::new(
                DetailKey::Output,
                format_count(breakdown.output_tokens_total()),
                DetailTone::Primary,
            ),
            false,
        ),
        detail_line(
            DetailField::new(
                DetailKey::Cache,
                format_count(breakdown.cache_tokens_total()),
                DetailTone::Primary,
            ),
            false,
        ),
        detail_line(
            DetailField::new(
                DetailKey::Total,
                format_count(breakdown.total_tokens),
                DetailTone::Primary,
            ),
            false,
        ),
        detail_line(
            DetailField::new(
                DetailKey::ApproxCost,
                format_cost(breakdown.cost_with_cache_usd),
                DetailTone::Cost,
            )
            .emphasized(),
            false,
        ),
        detail_line(
            DetailField::new(
                DetailKey::Note,
                crate::tui_text!(
                    "approx official-equivalent · provider-level (all-time)",
                    "约等于官方口径 · 提供商级别（全时段）"
                )
                .to_string(),
                DetailTone::Muted,
            ),
            false,
        ),
    ]
}

// token 行展示统一走这里: 掩码策略唯一归属 ccr_core::mask_sensitive,
// 这里只负责拼接 `configured (<masked>)` 展示形态, 不得输出明文。
fn configured_token_text(config: &ProfileConfig) -> Option<String> {
    config
        .auth_token
        .as_ref()
        .map(|token| token.expose().trim())
        .filter(|token| !token.is_empty())
        .map(|token| {
            crate::tui_format!(
                "configured ({})",
                "已配置（{}）",
                ccr_core::mask_sensitive(token)
            )
        })
}

fn codex_runtime_mode_label(mode: ccr_cli::models::CodexRuntimeMode) -> &'static str {
    match mode {
        ccr_cli::models::CodexRuntimeMode::ProfileOnly => {
            crate::tui_text!("Profile driven", "Profile 驱动")
        }
        ccr_cli::models::CodexRuntimeMode::ProfileWithAuth => {
            crate::tui_text!(
                "Profile routing + Auth identity",
                "Profile 路由 + Auth 身份"
            )
        }
        ccr_cli::models::CodexRuntimeMode::ProfilePendingAuth => {
            crate::tui_text!(
                "Profile routing, waiting for Auth",
                "Profile 路由，等待 Auth"
            )
        }
        ccr_cli::models::CodexRuntimeMode::RuntimeOnly => {
            crate::tui_text!("Runtime/Auth only", "仅 Runtime/Auth 生效")
        }
        ccr_cli::models::CodexRuntimeMode::Unresolved => {
            crate::tui_text!("Unresolved", "未解析")
        }
    }
}

fn localize_codex_runtime_text(text: String) -> String {
    if crate::tui::i18n::active_language() == ccr_cli::managers::TuiLanguage::English {
        text.replace("未绑定", "not bound")
            .replace("未保存账号", "unsaved account")
            .replace("未登录", "not logged in")
            .replace("未知状态", "unknown state")
    } else {
        text
    }
}

fn section_line(title: &str) -> Line<'static> {
    let title = match title {
        " Overview " => crate::tui_text!(" Overview ", " 概览 "),
        " Runtime " => crate::tui_text!(" Runtime ", " 运行时 "),
        " Activity " => crate::tui_text!(" Activity ", " 活动 "),
        " Routing/Auth " => crate::tui_text!(" Routing/Auth ", " 路由/认证 "),
        " Engine " => crate::tui_text!(" Engine ", " 引擎 "),
        _ => title,
    };
    Line::from(vec![
        Span::styled("▌ ", theme::info_style()),
        Span::styled(
            title.to_string(),
            theme::secondary_text_emphasis_style().add_modifier(Modifier::UNDERLINED),
        ),
    ])
}

fn detail_label_width(compact: bool) -> usize {
    let natural_width = DetailKey::ALL
        .iter()
        .map(|key| UnicodeWidthStr::width(key.label()))
        .max()
        .unwrap_or(8)
        .saturating_add(2);
    if compact {
        natural_width.clamp(8, 12)
    } else {
        natural_width.clamp(10, 18)
    }
}

fn detail_tone_style(tone: DetailTone) -> Style {
    match tone {
        DetailTone::Primary => theme::primary_text_style(),
        DetailTone::Muted => theme::muted_style(),
        DetailTone::Info => theme::info_style(),
        DetailTone::Success => theme::success_style(),
        DetailTone::Warning => theme::warning_style(),
        DetailTone::StrongWarning => theme::warning_style().add_modifier(Modifier::BOLD),
        DetailTone::Cost => theme::warning_style().add_modifier(Modifier::BOLD),
        DetailTone::Accent { platform, strong } => {
            let style = Style::default().fg(theme::accent_for(platform));
            if strong {
                style.add_modifier(Modifier::BOLD)
            } else {
                style
            }
        }
    }
}

fn detail_line(field: DetailField, compact: bool) -> Line<'static> {
    let label_style = if field.emphasize_label {
        let style = if field.tone == DetailTone::Muted {
            theme::info_style()
        } else {
            detail_tone_style(field.tone)
        };
        style.add_modifier(Modifier::BOLD)
    } else {
        theme::secondary_text_emphasis_style()
    };
    Line::from(vec![
        Span::styled(
            pad_text(field.key.label(), detail_label_width(compact)),
            label_style,
        ),
        Span::styled(field.value, detail_tone_style(field.tone)),
    ])
}

#[derive(Debug, Clone)]
struct ProfileSummaryField {
    label: String,
    value: String,
    tone: DetailTone,
}

fn profile_summary_line(field: ProfileSummaryField) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("{}: ", field.label),
            theme::secondary_text_emphasis_style(),
        ),
        Span::styled(field.value, detail_tone_style(field.tone)),
    ])
}

fn opt_text(value: Option<&str>) -> String {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("-")
        .to_string()
}

fn bool_text(value: Option<bool>) -> &'static str {
    match value {
        Some(true) => crate::tui_text!("yes", "是"),
        Some(false) => crate::tui_text!("no", "否"),
        None => "-",
    }
}

fn tags_text(config: &ProfileConfig) -> String {
    config
        .tags
        .as_ref()
        .filter(|tags| !tags.is_empty())
        .map(|tags| tags.join(", "))
        .unwrap_or_else(|| "-".to_string())
}

fn codex_platform_value(config: &ProfileConfig, key: &str) -> Option<String> {
    config.platform_data.get(key).and_then(|value| match value {
        serde_json::Value::String(text) if !text.trim().is_empty() => Some(text.trim().to_string()),
        serde_json::Value::Bool(flag) => Some(flag.to_string()),
        serde_json::Value::Number(num) => Some(num.to_string()),
        _ => None,
    })
}

fn codex_reasoning_effort_field(config: &ProfileConfig) -> DetailField {
    let (value, tone) = match config.platform_data.get("model_reasoning_effort") {
        None => ("-".to_string(), DetailTone::Muted),
        Some(serde_json::Value::String(raw)) if raw.trim().is_empty() => {
            ("-".to_string(), DetailTone::Muted)
        }
        Some(serde_json::Value::String(raw)) => {
            let normalized = raw.trim().to_ascii_lowercase();
            let tone = match normalized.as_str() {
                "minimal" => DetailTone::Muted,
                "low" => DetailTone::Info,
                "medium" => DetailTone::Accent {
                    platform: Platform::Codex,
                    strong: false,
                },
                "high" => DetailTone::Accent {
                    platform: Platform::Codex,
                    strong: true,
                },
                "xhigh" => DetailTone::StrongWarning,
                _ => DetailTone::Warning,
            };
            (normalized, tone)
        }
        Some(_) => (
            crate::tui_text!("invalid", "无效").to_string(),
            DetailTone::Warning,
        ),
    };

    DetailField::new(DetailKey::ReasoningEffort, value, tone).emphasized()
}

fn profile_meta_strings(
    total_profiles: usize,
    current_page: usize,
    total_pages: usize,
    selected: Option<&crate::tui::app::ProfileItem>,
) -> Vec<String> {
    let selection = selected
        .map(|profile| {
            if profile.is_current {
                crate::tui_format!("Selected: {} (current)", "已选择：{}（当前）", profile.name)
            } else {
                crate::tui_format!("Selected: {}", "已选择：{}", profile.name)
            }
        })
        .unwrap_or_else(|| crate::tui_text!("Selected: -", "已选择：-").to_string());

    vec![
        selection,
        crate::tui_format!(
            "Profiles: {total_profiles} · Page: {}/{}",
            "配置数：{total_profiles} · 页码：{}/{}",
            current_page + 1,
            total_pages.max(1)
        ),
        crate::tui_text!("Legend: ● current · ▶ selected", "图例：● 当前 · ▶ 已选择").to_string(),
    ]
}

// Focus 块只保留 profile 身份与状态。详情参数在 Context 展示，apply/toast 反馈在
// 按需出现的 Status strip 展示，避免同一信息占据两处首屏空间。
fn profile_summary_fields(
    platform: Platform,
    name: &str,
    config: &ProfileConfig,
    is_current: bool,
) -> Vec<ProfileSummaryField> {
    vec![
        ProfileSummaryField {
            label: crate::tui_text!("Name", "名称").to_string(),
            value: name.to_string(),
            tone: DetailTone::Accent {
                platform,
                strong: true,
            },
        },
        ProfileSummaryField {
            label: crate::tui_text!("Status", "状态").to_string(),
            value: crate::tui_format!(
                "{} · {}",
                "{} · {}",
                if is_current {
                    crate::tui_text!("Current", "当前")
                } else {
                    crate::tui_text!("Available", "可用")
                },
                if config.is_enabled() {
                    crate::tui_text!("Enabled", "启用")
                } else {
                    crate::tui_text!("Disabled", "禁用")
                }
            ),
            tone: if !config.is_enabled() {
                DetailTone::Warning
            } else if is_current {
                DetailTone::Success
            } else {
                DetailTone::Info
            },
        },
    ]
}

fn last_apply_message(
    profile_name: &str,
    last_applied: Option<&(String, String, bool, Option<String>)>,
) -> Option<String> {
    let (_, name, success, error) = last_applied?;
    if name != profile_name {
        return None;
    }

    if *success {
        Some(crate::tui_text!("Applied successfully", "应用成功").to_string())
    } else {
        Some(crate::tui_format!(
            "Apply failed{}",
            "应用失败{}",
            error
                .as_ref()
                .map(|err| format!(" ({err})"))
                .unwrap_or_default()
        ))
    }
}

/// Render empty state for current platform
fn render_empty_state(f: &mut Frame, app: &App, area: Rect, block: Block) {
    let platform = app.current_platform();
    let platform_name = platform.display_name();
    let short_name = platform.short_name();

    if let Some(error) = app.current_profile_load_error() {
        let error_text = vec![
            Line::from(""),
            Line::from(Span::styled(
                crate::tui_format!(
                    "Failed to load {} profiles",
                    "无法加载 {} 配置",
                    platform_name
                ),
                theme::empty_hint_style(),
            )),
            Line::from(""),
            Line::from(Span::styled(
                crate::tui_text!(
                    "CCR could not read the profile source below:",
                    "CCR 无法读取以下配置来源："
                )
                .to_string(),
                theme::secondary_text_style(),
            )),
            Line::from(""),
            Line::from(Span::styled(error.to_string(), theme::primary_text_style())),
            Line::from(""),
            Line::from(Span::styled(
                crate::tui_text!(
                    "Fix the file content or path, then press 'r' to reload.",
                    "修复文件内容或路径后，按 r 重新加载。"
                )
                .to_string(),
                theme::muted_style(),
            )),
        ];

        let paragraph = Paragraph::new(error_text)
            .block(block)
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: false });

        f.render_widget(paragraph, area);
        return;
    }

    let empty_text = vec![
        Line::from(""),
        Line::from(Span::styled(
            crate::tui_format!(
                "No {} configurations found",
                "未找到 {} 配置",
                platform_name
            ),
            theme::empty_hint_style(),
        )),
        Line::from(""),
        Line::from(Span::styled(
            crate::tui_format!(
                "Run 'ccr platform init {}' to initialize",
                "运行 'ccr platform init {}' 进行初始化",
                short_name
            ),
            theme::secondary_text_style(),
        )),
        Line::from(""),
        Line::from(Span::styled(
            crate::tui_text!(
                "Or 'ccr add' to create a new configuration",
                "或运行 'ccr add' 创建新配置"
            )
            .to_string(),
            theme::muted_style(),
        )),
    ];

    let paragraph = Paragraph::new(empty_text)
        .block(block)
        .alignment(Alignment::Center);

    f.render_widget(paragraph, area);
}

fn profile_status_text(app: &App) -> Vec<Line<'static>> {
    if let Some(error) = app.current_profile_load_error() {
        return vec![
            Line::from(Span::styled(
                crate::tui_text!("Profile list unavailable", "配置列表不可用").to_string(),
                theme::empty_hint_style(),
            )),
            Line::from(""),
            Line::from(Span::styled(error.to_string(), theme::primary_text_style())),
        ];
    }

    if let Some(error) = app.current_profile_status_error() {
        return vec![
            Line::from(Span::styled(
                crate::tui_text!("Current profile state unavailable", "当前配置状态不可用")
                    .to_string(),
                theme::empty_hint_style(),
            )),
            Line::from(""),
            Line::from(Span::styled(error.to_string(), theme::primary_text_style())),
        ];
    }

    vec![Line::from(crate::tui_text!(
        "No profile selected",
        "未选择配置"
    ))]
}

fn profile_status_alignment(app: &App) -> Alignment {
    if app.current_profile_load_error().is_some() || app.current_profile_status_error().is_some() {
        Alignment::Left
    } else {
        Alignment::Center
    }
}

// ═══════════════════════════════════════════════════════════
// Footer rendering
// ═══════════════════════════════════════════════════════════

/// Render footer with keyboard shortcuts and toast notification
fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let hints = footer_hints_for_width(app, area.width);
    let mut line = shortcut_line(&hints, theme::accent_for(app.current_platform()));
    if let Some(toast) = app.toasts.active() {
        let style = match toast.kind {
            ToastKind::Success => theme::success_style(),
            ToastKind::Error => theme::error_style(),
            ToastKind::Warning => theme::warning_style(),
            ToastKind::Info => theme::info_style(),
        };
        line.spans.insert(
            0,
            Span::styled("  │  ", Style::default().fg(theme::muted())),
        );
        line.spans
            .insert(0, Span::styled(toast.message.clone(), style));
    }

    let paragraph = Paragraph::new(line)
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_set(symbols::border::ROUNDED)
                .border_style(Style::default().fg(theme::border()))
                .title(crate::tui_text!(" Keys ", " 按键 "))
                .title_alignment(Alignment::Center)
                .title_style(theme::muted_style()),
        )
        .style(theme::secondary_text_style())
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

#[cfg(test)]
fn footer_text(app: &App) -> String {
    footer_text_for_width(app, u16::MAX)
}

#[cfg(test)]
fn footer_text_for_width(app: &App, width: u16) -> String {
    let shortcuts = footer_hints_for_width(app, width)
        .iter()
        .map(|hint| {
            if hint.label.is_empty() {
                hint.key.to_string()
            } else {
                format!("{} {}", hint.key, hint.label)
            }
        })
        .collect::<Vec<_>>()
        .join("  │  ");

    if let Some(toast) = app.toasts.active() {
        format!("{}  │  {}", toast.message, shortcuts)
    } else {
        shortcuts
    }
}

fn footer_hints_for_width(app: &App, width: u16) -> Vec<ShortcutHint<'static>> {
    if width < 90 {
        return vec![
            ShortcutHint::new("Tab", ""),
            ShortcutHint::new("↑↓", crate::tui_text!("select", "选择")),
            ShortcutHint::new("PgUp/PgDn", crate::tui_text!("details", "详情")),
            ShortcutHint::new("Enter", crate::tui_text!("apply", "应用")),
            ShortcutHint::new("Ctrl+L", crate::tui_text!("lang", "语言")),
            ShortcutHint::new("q", crate::tui_text!("quit", "退出")),
        ];
    }

    let mut hints = vec![ShortcutHint::new(
        "Tab/Shift+Tab",
        crate::tui_text!("switch", "切换"),
    )];
    if app.total_pages() > 1 {
        hints.push(ShortcutHint::new("←→", crate::tui_text!("page", "翻页")));
    }
    hints.extend([
        ShortcutHint::new("↑↓/jk", crate::tui_text!("select", "选择")),
        ShortcutHint::new("PgUp/PgDn", crate::tui_text!("details", "详情")),
        ShortcutHint::new("Enter", crate::tui_text!("apply", "应用")),
        ShortcutHint::new("r", crate::tui_text!("reload", "刷新")),
        ShortcutHint::new("Ctrl+L", crate::tui_text!("language", "语言")),
        ShortcutHint::new("q", crate::tui_text!("quit", "退出")),
    ]);
    hints
}

/// Render toast notification (replaces old status_message)
#[allow(dead_code)]
fn render_toast(f: &mut Frame, app: &App, area: Rect) {
    if let Some(toast) = app.toasts.active() {
        let style = match toast.kind {
            ToastKind::Success => theme::success_style(),
            ToastKind::Error => theme::error_style(),
            ToastKind::Warning => theme::warning_style(),
            ToastKind::Info => theme::secondary_text_style(),
        };

        let status = Paragraph::new(Span::styled(toast.message.as_str(), style))
            .alignment(Alignment::Center);

        f.render_widget(status, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::app::{PlatformTab, ProfileItem, TabVariant};
    use crate::tui::runtime::AsyncTaskExecutor;
    use crate::tui::theme::ViewportMode;
    use crate::tui::toast::ToastManager;
    use crate::tui::usage::app::{UsageApp, UsageDataset};
    use ccr_usage::TaggedProviderBreakdown;
    use indexmap::IndexMap;
    use ratatui::{Terminal, backend::TestBackend};
    use std::cell::Cell;
    use std::sync::Arc;

    fn plain_line_text(line: &Line<'_>) -> String {
        line.spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect::<Vec<_>>()
            .join("")
    }

    fn buffer_line_text(backend: &TestBackend, y: u16) -> String {
        let width = backend.buffer().area.width;
        (0..width)
            .filter_map(|x| backend.buffer().cell((x, y)))
            .map(|cell| cell.symbol())
            .collect::<Vec<_>>()
            .join("")
            .trim_end()
            .to_string()
    }

    fn buffer_text(backend: &TestBackend) -> String {
        let height = backend.buffer().area.height;
        (0..height)
            .map(|y| buffer_line_text(backend, y))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn sample_profile_app(profile: ProfileItem, config: ProfileConfig) -> App {
        sample_profile_app_for(Platform::Claude, profile, config)
    }

    fn sample_profile_app_for(
        platform: Platform,
        profile: ProfileItem,
        config: ProfileConfig,
    ) -> App {
        let mut profile_configs = IndexMap::new();
        profile_configs.insert(profile.name.clone(), config);

        App {
            tabs: vec![PlatformTab {
                platform,
                variant: TabVariant::Profile,
                label: platform.display_name().to_string(),
                profiles: vec![profile.clone()],
                profile_configs,
                profile_load_error: None,
                current_profile_error: None,
                claude_runtime_summary: None,
                codex_runtime_summary: None,
                instance: None,
                saved_selection: None,
            }],
            active_tab: 0,
            selected_index: 0,
            current_page: 0,
            page_size: crate::tui::pagination::DEFAULT_PAGE_SIZE,
            selected_profile_name: Some(profile.name),
            toasts: ToastManager::new(),
            last_applied: None,
            claude_auth_app: None,
            claude_auth_error: None,
            last_claude_action: None,
            codex_auth_app: None,
            codex_auth_error: None,
            last_codex_action: None,
            opencode_auth_app: None,
            opencode_auth_error: None,
            last_opencode_action: None,
            usage_app: None,
            header_area: Cell::new(None),
            list_area: Cell::new(None),
            detail_area: Cell::new(None),
            profile_detail_scroll: 0,
            task_executor: crate::tui::runtime::AsyncTaskExecutor::from_current_or_test(),
        }
    }

    fn empty_platform_tab(platform: Platform, variant: TabVariant, label: &str) -> PlatformTab {
        PlatformTab {
            platform,
            variant,
            label: label.to_string(),
            profiles: Vec::new(),
            profile_configs: IndexMap::new(),
            profile_load_error: None,
            current_profile_error: None,
            claude_runtime_summary: None,
            codex_runtime_summary: None,
            instance: None,
            saved_selection: None,
        }
    }

    #[test]
    fn active_tab_title_uses_platform_filled_chip() {
        let tab = empty_platform_tab(Platform::Codex, TabVariant::Profile, "Codex Profile");
        let line = tab_title_line(&tab, true, false);

        assert_eq!(line.spans.len(), 1);
        assert_eq!(line.spans[0].style.fg, Some(theme::selection_fg()));
        assert_eq!(
            line.spans[0].style.bg,
            Some(theme::platform_selection_color_for(Platform::Codex))
        );
        assert_ne!(
            line.spans[0].style.bg,
            Some(theme::platform_selection_color_for(Platform::Claude))
        );
        assert!(line.spans[0].style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn inactive_tab_title_uses_subtext_without_fill() {
        let tab = empty_platform_tab(Platform::Claude, TabVariant::Profile, "Claude Code");
        let line = tab_title_line(&tab, false, false);

        assert_eq!(line.spans.len(), 1);
        assert_eq!(line.spans[0].style.fg, Some(theme::subtext()));
        assert_eq!(line.spans[0].style.bg, None);
    }

    #[test]
    fn footer_hint_mentions_reverse_tab_switching() {
        let profile = ProfileItem {
            name: "default".to_string(),
            description: Some("Default profile".to_string()),
            is_current: false,
        };
        let app = sample_profile_app(profile, ProfileConfig::new());

        assert!(footer_text(&app).contains("Tab/Shift+Tab switch"));
        // 快捷键只保留 footer 一处, Selection 面板不再重复
        assert!(
            !profile_meta_strings(1, 0, 1, app.selected_profile())
                .iter()
                .any(|line| line.contains("Tab/Shift+Tab switch"))
        );
    }

    #[test]
    fn viewport_mode_prefers_wide_only_when_both_dimensions_allow() {
        assert_eq!(theme::viewport_mode(140, 30), ViewportMode::Wide);
        assert_eq!(theme::viewport_mode(100, 30), ViewportMode::Standard);
        assert_eq!(theme::viewport_mode(140, 20), ViewportMode::Compact);
        assert_eq!(theme::viewport_mode(80, 30), ViewportMode::Compact);
    }

    #[test]
    fn simplified_chinese_renders_across_all_viewport_modes() {
        crate::tui::i18n::set_language(ccr_cli::managers::TuiLanguage::SimplifiedChinese);

        for (width, height) in [(80, 20), (100, 30), (140, 30)] {
            let profile = ProfileItem {
                name: "main".to_string(),
                description: Some("主要配置".to_string()),
                is_current: true,
            };
            let mut app = sample_profile_app(profile, ProfileConfig::new());
            let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
            terminal.draw(|frame| draw(frame, &mut app)).unwrap();

            let rendered = buffer_text(terminal.backend());
            let compact = rendered.replace(' ', "");
            assert!(
                compact.contains("配置切换器"),
                "{width}x{height}: {rendered}"
            );
            assert!(
                compact.contains("Claude配置"),
                "{width}x{height}: {rendered}"
            );
            assert!(
                compact.contains("Ctrl+L语言"),
                "{width}x{height}: {rendered}"
            );
        }

        crate::tui::i18n::set_language(ccr_cli::managers::TuiLanguage::English);
    }

    #[test]
    fn wide_profile_workspace_layout_favors_detail_readability() {
        let (list_area, context_area) = wide_profile_workspace_layout(Rect::new(0, 0, 120, 20));

        assert!(context_area.width > list_area.width);
        assert_eq!(list_area.width + context_area.width, 120);
    }

    #[test]
    fn profile_context_only_reserves_status_rows_for_feedback() {
        assert_eq!(profile_context_constraints(4, false).len(), 2);
        assert_eq!(profile_context_constraints(4, true).len(), 3);
        assert_eq!(
            profile_context_constraints(4, true)[2],
            Constraint::Length(3)
        );
    }

    #[test]
    fn detail_label_width_tracks_language_and_viewport() {
        crate::tui::i18n::set_language(ccr_cli::managers::TuiLanguage::English);
        let english = detail_label_width(false);
        assert_eq!(english, 18);
        assert!(detail_label_width(true) <= 12);

        crate::tui::i18n::set_language(ccr_cli::managers::TuiLanguage::SimplifiedChinese);
        let chinese = detail_label_width(false);
        assert!((10..=18).contains(&chinese));
        assert!(detail_label_width(true) <= 12);
        crate::tui::i18n::set_language(ccr_cli::managers::TuiLanguage::English);
    }

    #[test]
    fn profile_list_rail_layout_keeps_full_selection_panel_visible() {
        let (list_area, meta_area) = profile_list_rail_layout(Rect::new(0, 0, 58, 20));

        assert_eq!(list_area.height, 15);
        assert_eq!(meta_area.height, 5);
    }

    #[test]
    fn compact_profile_workspace_preserves_a_context_drawer() {
        let Some((list_area, context_area)) =
            compact_profile_workspace_layout(Rect::new(0, 0, 80, 12))
        else {
            panic!("80x12 compact profile workspace should include a context drawer");
        };

        assert_eq!(list_area.height, 6);
        assert_eq!(context_area.height, 6);
    }

    #[test]
    fn column_widths_prioritize_name_without_hiding_description_too_early() {
        assert_eq!(column_widths(51), (45, 0));
        assert_eq!(column_widths(52), (18, 28));
        assert_eq!(column_widths(58), (20, 32));
    }

    #[test]
    fn truncate_text_limits_display_width_for_cjk_and_emoji() {
        for (text, width) in [
            ("中文描述很长", 8),
            ("ab中文cd", 6),
            ("📭📭📭", 4),
            ("公益 AnyRouter 中转", 10),
            ("中文", 1),
        ] {
            let out = truncate_text(text, width);
            assert!(out.width() <= width, "{text} @ {width} -> {out}");
            assert!(out.ends_with('…'), "{text} @ {width} -> {out}");
        }
    }

    #[test]
    fn truncate_and_pad_keep_ascii_behavior_unchanged() {
        assert_eq!(truncate_text("abcdef", 6), "abcdef");
        assert_eq!(truncate_text("abcdefg", 6), "abcde…");
        assert_eq!(truncate_text("abc", 0), "");
        assert_eq!(pad_text("abc", 6), "abc   ");
        assert_eq!(pad_text("abcdef", 4), "abcdef");
    }

    #[test]
    fn pad_text_fills_to_display_width_for_cjk() {
        let out = pad_text("中文", 6);
        assert_eq!(out, "中文  ");
        assert_eq!(out.width(), 6);
    }

    #[test]
    fn profile_list_row_with_cjk_description_stays_within_column_budget() {
        let profile = ProfileItem {
            name: "anyrouter4".to_string(),
            description: Some("AnyRouter 公益中转,含超长中文描述用于截断验证".to_string()),
            is_current: true,
        };
        let (name_width, desc_width) = (20usize, 24usize);

        let rendered = plain_line_text(&profile_list_row(&profile, false, name_width, desc_width));

        assert!(
            rendered.width() <= name_width + 2 + desc_width,
            "{rendered}"
        );
        assert!(rendered.contains('…'), "{rendered}");
    }

    #[test]
    fn profile_list_row_shows_more_of_long_profile_names_on_wider_list_rail() {
        let profile = ProfileItem {
            name: "anyrouter_temp_backup".to_string(),
            description: Some("AnyRouter temp profile".to_string()),
            is_current: false,
        };

        let rendered = plain_line_text(&profile_list_row(&profile, true, 20, 32));

        assert!(rendered.contains("anyrouter_temp"), "{rendered}");
        assert!(rendered.contains("AnyRouter temp profile"), "{rendered}");
    }

    #[test]
    fn unselected_profile_row_uses_explicit_palette_foreground() {
        let profile = ProfileItem {
            name: "default".to_string(),
            description: Some("Default profile".to_string()),
            is_current: false,
        };

        let line = profile_list_row(&profile, false, 18, 24);

        assert_eq!(line.spans[0].style.fg, Some(theme::text()));
        assert_eq!(line.spans[0].style.bg, None);
        assert_eq!(line.spans[2].style.fg, Some(theme::muted()));
    }

    #[test]
    fn summary_and_detail_values_use_explicit_tones() {
        let summary = profile_summary_line(ProfileSummaryField {
            label: "Name".to_string(),
            value: "fovts".to_string(),
            tone: DetailTone::Primary,
        });
        assert_eq!(summary.spans[0].style.fg, Some(theme::subtext()));
        assert_eq!(summary.spans[1].style.fg, Some(theme::text()));

        let detail = detail_line(
            DetailField::new(
                DetailKey::SwitchCount,
                "42".to_string(),
                DetailTone::Primary,
            ),
            false,
        );
        assert_eq!(detail.spans[0].style.fg, Some(theme::subtext()));
        assert_eq!(detail.spans[1].style.fg, Some(theme::text()));
    }

    #[test]
    fn footer_uses_platform_accent_for_keys() {
        let profile = ProfileItem {
            name: "default".to_string(),
            description: Some("Default profile".to_string()),
            is_current: false,
        };
        let app = sample_profile_app(profile, ProfileConfig::new());
        let expected_accent = theme::accent_for(app.current_platform());
        let mut terminal = Terminal::new(TestBackend::new(72, 3)).unwrap();

        terminal
            .draw(|frame| render_footer(frame, &app, frame.area()))
            .unwrap();

        let buffer = terminal.backend().buffer();
        let footer_cell = buffer
            .content()
            .iter()
            .find(|cell| cell.symbol() == "T")
            .expect("footer should render Tab/Shift+Tab switch shortcut");

        assert_eq!(footer_cell.fg, expected_accent);
        assert!(footer_cell.modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn profile_meta_strings_show_selection_and_paging() {
        let profile = crate::tui::app::ProfileItem {
            name: "fovts".to_string(),
            description: Some("公益".to_string()),
            is_current: true,
        };

        let lines = profile_meta_strings(15, 1, 2, Some(&profile));

        assert!(lines.iter().any(|line| line.contains("Selected: fovts")));
        assert!(
            lines
                .iter()
                .any(|line| line.contains("Profiles: 15 · Page: 2/2"))
        );
        assert!(lines.iter().any(|line| line.contains("Legend:")));
        assert!(!lines.iter().any(|line| line.contains("Enter apply")));
    }

    #[test]
    fn profile_summary_fields_focus_on_identity_and_status() {
        let mut config = ProfileConfig::new();
        config.description = Some("fovts 公益".to_string());
        config.model = Some("gpt-5.4".to_string());
        config.base_url = Some("https://example.com/v1".to_string());

        let fields = profile_summary_fields(Platform::Codex, "fovts", &config, true);
        let lines: Vec<String> = fields
            .clone()
            .into_iter()
            .map(profile_summary_line)
            .map(|line| plain_line_text(&line))
            .collect();

        assert!(lines.iter().any(|line| line.contains("Name: fovts")));
        assert!(
            lines
                .iter()
                .any(|line| line.contains("Status: Current · Enabled"))
        );
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[1].tone, DetailTone::Success);
        // Context 展示这些参数, Focus 不再重复。
        assert!(!lines.iter().any(|line| line.starts_with("Description:")));
        assert!(!lines.iter().any(|line| line.starts_with("Model:")));
        assert!(!lines.iter().any(|line| line.starts_with("Base URL:")));
    }

    #[test]
    fn profile_summary_fields_warn_when_profile_is_disabled() {
        let mut config = ProfileConfig::new();
        config.enabled = Some(false);

        let fields = profile_summary_fields(Platform::Codex, "fovts", &config, false);

        assert_eq!(fields.len(), 2);
        assert_eq!(fields[1].tone, DetailTone::Warning);
    }

    #[test]
    fn profile_status_strip_surfaces_last_apply_feedback() {
        let profile = ProfileItem {
            name: "fovts".to_string(),
            description: Some("fovts 公益".to_string()),
            is_current: true,
        };
        let mut app = sample_profile_app(profile, ProfileConfig::new());
        app.last_applied = Some(("Codex Profile".to_string(), "fovts".to_string(), true, None));

        let mut terminal = Terminal::new(TestBackend::new(72, 3)).unwrap();
        terminal
            .draw(|frame| render_profile_status_strip(frame, &app, frame.area(), "fovts"))
            .unwrap();

        let rendered = buffer_text(terminal.backend());
        assert!(rendered.contains("Applied successfully"), "{rendered}");
        // strip 不再重复快捷键列表
        assert!(!rendered.contains("Enter apply"), "{rendered}");
        assert!(!rendered.contains("q quit"), "{rendered}");
    }

    #[test]
    fn profile_status_strip_stays_quiet_without_apply_feedback() {
        let profile = ProfileItem {
            name: "fovts".to_string(),
            description: Some("fovts 公益".to_string()),
            is_current: true,
        };
        let app = sample_profile_app(profile, ProfileConfig::new());

        let mut terminal = Terminal::new(TestBackend::new(72, 3)).unwrap();
        terminal
            .draw(|frame| render_profile_status_strip(frame, &app, frame.area(), "fovts"))
            .unwrap();

        let rendered = buffer_text(terminal.backend());
        assert!(!rendered.contains("Status"), "{rendered}");
        assert!(!rendered.contains("Enter apply"), "{rendered}");
        assert!(!rendered.contains("Tab/Shift+Tab"), "{rendered}");
    }

    #[test]
    fn profile_meta_panel_render_shows_legend_when_rail_has_extra_height() {
        let profile = ProfileItem {
            name: "2CAPI".to_string(),
            description: Some("2CAPI 公益".to_string()),
            is_current: false,
        };
        let app = sample_profile_app(profile, ProfileConfig::new());
        let mut terminal = Terminal::new(TestBackend::new(58, 7)).unwrap();

        terminal
            .draw(|frame| render_profile_meta_panel(frame, &app, frame.area()))
            .unwrap();

        let rendered = buffer_text(terminal.backend());
        assert!(rendered.contains("Legend: ● current"), "{rendered}");
    }

    #[test]
    fn profile_details_wrap_long_runtime_values_instead_of_clipping_them() {
        let profile = ProfileItem {
            name: "2CAPI".to_string(),
            description: Some("2CAPI 公益".to_string()),
            is_current: false,
        };
        let mut config = ProfileConfig::new();
        config.description = Some("2CAPI 公益".to_string());
        config.base_url =
            Some("https://2capi.com/compatible-mode/openai/v1/chat/completions".to_string());
        config.model = Some("gpt-5.4-mini".to_string());

        let mut app = sample_profile_app(profile, config);
        let mut terminal = Terminal::new(TestBackend::new(32, 14)).unwrap();

        terminal
            .draw(|frame| {
                render_profile_details(frame, &mut app, frame.area(), ViewportMode::Standard)
            })
            .unwrap();

        let rendered = buffer_text(terminal.backend());
        assert!(rendered.contains("https://2capi.com"), "{rendered}");
        assert!(rendered.contains("compatible"), "{rendered}");
        assert!(rendered.contains("chat/complet"), "{rendered}");
    }

    #[test]
    fn compact_codex_profile_draw_keeps_focus_context_and_scroll_hint_visible() {
        let profile = ProfileItem {
            name: "oneapi".to_string(),
            description: Some("oneapi商业".to_string()),
            is_current: false,
        };
        let mut config = ProfileConfig::new();
        config.description = Some("oneapi商业".to_string());
        config.provider = Some("oneapi".to_string());
        config.provider_type = Some("openai".to_string());
        config.base_url = Some("https://oneapi.example.com/v1".to_string());
        config.model = Some("gpt-5.5".to_string());

        let mut app = sample_profile_app(profile, config);
        app.tabs[0].platform = Platform::Codex;
        app.tabs[0].label = "Codex Profile".to_string();
        app.selected_profile_name = Some("oneapi".to_string());

        let mut terminal = Terminal::new(TestBackend::new(80, 20)).unwrap();
        terminal.draw(|frame| draw(frame, &mut app)).unwrap();

        let rendered = buffer_text(terminal.backend());
        assert!(rendered.contains("Codex Profiles"), "{rendered}");
        assert!(rendered.contains("Focus"), "{rendered}");
        assert!(rendered.contains("Context"), "{rendered}");
        assert!(rendered.contains("PgUp/PgDn details"), "{rendered}");
    }

    #[test]
    fn profile_details_clamp_oversized_scroll_offsets() {
        let profile = ProfileItem {
            name: "long-codex".to_string(),
            description: Some("long detail profile".to_string()),
            is_current: false,
        };
        let mut config = ProfileConfig::new();
        config.description = Some("long detail profile".to_string());
        config.provider = Some("provider".to_string());
        config.provider_type = Some("openai".to_string());
        config.base_url = Some("https://example.com/very/long/path/for/rendering".to_string());
        config.model = Some("gpt-5.5".to_string());
        config.small_fast_model = Some("gpt-5.4-mini".to_string());
        config.account = Some("account".to_string());

        let mut app = sample_profile_app(profile, config);
        app.tabs[0].platform = Platform::Codex;
        app.profile_detail_scroll = 100;

        let mut terminal = Terminal::new(TestBackend::new(48, 8)).unwrap();
        terminal
            .draw(|frame| {
                render_profile_details(frame, &mut app, frame.area(), ViewportMode::Standard)
            })
            .unwrap();

        assert!(app.profile_detail_scroll < 100);
        assert!(app.profile_detail_scroll > 0);
    }

    fn detail_texts(lines: &[Line<'_>]) -> Vec<String> {
        lines.iter().map(plain_line_text).collect()
    }

    fn token_line(texts: &[String]) -> String {
        texts
            .iter()
            .find(|text| text.starts_with("token"))
            .cloned()
            .expect("token line present")
    }

    #[test]
    fn claude_detail_token_line_shows_masked_key_when_configured() {
        let mut config = ProfileConfig::new();
        config
            .platform_data
            .insert("auth_mode".to_string(), serde_json::json!("api_key"));
        config.auth_token = Some(ccr_core::Secret::new("sk-ant-test1234567890"));

        let texts = detail_texts(&claude_profile_detail_lines(
            "api", &config, false, None, false,
        ));
        assert!(
            token_line(&texts).contains("configured (sk-a...7890)"),
            "{texts:?}"
        );
    }

    #[test]
    fn claude_detail_token_line_masks_short_key_fully() {
        let mut config = ProfileConfig::new();
        config
            .platform_data
            .insert("auth_mode".to_string(), serde_json::json!("api_key"));
        config.auth_token = Some(ccr_core::Secret::new("shortkey12"));

        let texts = detail_texts(&claude_profile_detail_lines(
            "api", &config, false, None, false,
        ));
        assert!(
            token_line(&texts).contains("configured (**********)"),
            "{texts:?}"
        );
    }

    #[test]
    fn detail_token_states_keep_missing_subscription_and_dash() {
        let mut claude_missing = ProfileConfig::new();
        claude_missing
            .platform_data
            .insert("auth_mode".to_string(), serde_json::json!("api_key"));
        let texts = detail_texts(&claude_profile_detail_lines(
            "m",
            &claude_missing,
            false,
            None,
            false,
        ));
        assert!(
            token_line(&texts).trim_end().ends_with("missing"),
            "{texts:?}"
        );

        let mut claude_subscription = ProfileConfig::new();
        claude_subscription
            .platform_data
            .insert("auth_mode".to_string(), serde_json::json!("subscription"));
        let texts = detail_texts(&claude_profile_detail_lines(
            "s",
            &claude_subscription,
            false,
            None,
            false,
        ));
        assert!(
            token_line(&texts).trim_end().ends_with("subscription"),
            "{texts:?}"
        );

        let mut codex_missing = ProfileConfig::new();
        codex_missing
            .platform_data
            .insert("auth_mode".to_string(), serde_json::json!("openai_api_key"));
        let texts = detail_texts(&codex_profile_detail_lines(
            "m",
            &codex_missing,
            false,
            None,
            false,
        ));
        assert!(
            token_line(&texts).trim_end().ends_with("missing"),
            "{texts:?}"
        );

        let mut codex_no_auth = ProfileConfig::new();
        codex_no_auth
            .platform_data
            .insert("auth_mode".to_string(), serde_json::json!("no_auth"));
        let texts = detail_texts(&codex_profile_detail_lines(
            "n",
            &codex_no_auth,
            false,
            None,
            false,
        ));
        assert!(token_line(&texts).trim_end().ends_with('-'), "{texts:?}");
    }

    #[test]
    fn codex_detail_token_line_is_masked_and_lives_in_routing_auth_group() {
        let mut config = ProfileConfig::new();
        config
            .platform_data
            .insert("auth_mode".to_string(), serde_json::json!("openai_api_key"));
        config.auth_token = Some(ccr_core::Secret::new("sk-test-abcdef1234567890"));

        let texts = detail_texts(&codex_profile_detail_lines(
            "codex-key",
            &config,
            false,
            None,
            false,
        ));

        let routing_idx = texts
            .iter()
            .position(|text| text.contains("Routing/Auth"))
            .expect("Routing/Auth section present");
        let engine_idx = texts
            .iter()
            .position(|text| text.contains("Engine"))
            .expect("Engine section present");
        let activity_idx = texts
            .iter()
            .position(|text| text.contains("Activity"))
            .expect("Activity section present");
        let token_idx = texts
            .iter()
            .position(|text| text.starts_with("token"))
            .expect("token line present");

        assert!(
            engine_idx < routing_idx && routing_idx < token_idx,
            "{texts:?}"
        );
        assert!(
            texts[activity_idx..]
                .iter()
                .all(|text| !text.starts_with("token")),
            "{texts:?}"
        );
        assert!(
            texts[token_idx].contains("configured (sk-t...7890)"),
            "{texts:?}"
        );
    }

    #[test]
    fn profile_detail_lines_never_render_plaintext_token() {
        let plaintext = "sk-ant-test1234567890";

        let mut claude_config = ProfileConfig::new();
        claude_config
            .platform_data
            .insert("auth_mode".to_string(), serde_json::json!("api_key"));
        claude_config.auth_token = Some(ccr_core::Secret::new(plaintext));

        let mut codex_config = ProfileConfig::new();
        codex_config
            .platform_data
            .insert("auth_mode".to_string(), serde_json::json!("openai_api_key"));
        codex_config.auth_token = Some(ccr_core::Secret::new(plaintext));

        let claude_text = detail_texts(&claude_profile_detail_lines(
            "claude-key",
            &claude_config,
            true,
            None,
            false,
        ))
        .join("\n");
        let codex_text = detail_texts(&codex_profile_detail_lines(
            "codex-key",
            &codex_config,
            true,
            None,
            false,
        ))
        .join("\n");

        assert!(!claude_text.contains(plaintext), "{claude_text}");
        assert!(!codex_text.contains(plaintext), "{codex_text}");
    }

    #[test]
    fn codex_reasoning_effort_maps_known_missing_and_invalid_values() {
        let missing = codex_reasoning_effort_field(&ProfileConfig::new());
        assert_eq!(missing.value, "-");
        assert_eq!(missing.tone, DetailTone::Muted);

        let cases = [
            ("MINIMAL", "minimal", DetailTone::Muted),
            ("low", "low", DetailTone::Info),
            (
                "medium",
                "medium",
                DetailTone::Accent {
                    platform: Platform::Codex,
                    strong: false,
                },
            ),
            (
                "HIGH",
                "high",
                DetailTone::Accent {
                    platform: Platform::Codex,
                    strong: true,
                },
            ),
            ("xhigh", "xhigh", DetailTone::StrongWarning),
            ("ultra", "ultra", DetailTone::Warning),
        ];
        for (raw, expected, tone) in cases {
            let mut config = ProfileConfig::new();
            config
                .platform_data
                .insert("model_reasoning_effort".to_string(), serde_json::json!(raw));
            let field = codex_reasoning_effort_field(&config);
            assert_eq!(field.value, expected);
            assert_eq!(field.tone, tone);
        }

        let mut invalid = ProfileConfig::new();
        invalid.platform_data.insert(
            "model_reasoning_effort".to_string(),
            serde_json::json!(true),
        );
        let field = codex_reasoning_effort_field(&invalid);
        assert_eq!(field.value, "invalid");
        assert_eq!(field.tone, DetailTone::Warning);
    }

    #[test]
    fn codex_engine_places_reasoning_after_model_without_identity_duplication() {
        let mut config = ProfileConfig::new();
        config.model = Some("gpt-5.6-sol".to_string());
        config.platform_data.insert(
            "model_reasoning_effort".to_string(),
            serde_json::json!("high"),
        );

        let lines = codex_profile_detail_lines("work", &config, true, None, false);
        let texts = detail_texts(&lines);
        let model = texts
            .iter()
            .position(|text| text.starts_with("model"))
            .expect("model line present");
        let reasoning = texts
            .iter()
            .position(|text| text.starts_with("reasoning_effort"))
            .expect("reasoning effort line present");
        let small_fast = texts
            .iter()
            .position(|text| text.starts_with("small_fast"))
            .expect("small fast line present");

        assert!(model < reasoning && reasoning < small_fast, "{texts:?}");
        assert!(!texts.iter().any(|text| text.starts_with("name")));
        assert!(!texts.iter().any(|text| text.starts_with("current")));
        assert!(!texts.iter().any(|text| text.starts_with("enabled")));
    }

    #[test]
    fn codex_reasoning_label_is_localized_and_rendered_across_viewports() {
        let profile = ProfileItem {
            name: "work".to_string(),
            description: Some("主力配置".to_string()),
            is_current: true,
        };
        let mut config = ProfileConfig::new();
        config.model = Some("gpt-5.6-sol".to_string());
        config.platform_data.insert(
            "model_reasoning_effort".to_string(),
            serde_json::json!("high"),
        );

        for language in [
            ccr_cli::managers::TuiLanguage::English,
            ccr_cli::managers::TuiLanguage::SimplifiedChinese,
        ] {
            crate::tui::i18n::set_language(language);
            for (width, height) in [(80, 20), (100, 30), (140, 30)] {
                let mut app =
                    sample_profile_app_for(Platform::Codex, profile.clone(), config.clone());
                let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
                terminal.draw(|frame| draw(frame, &mut app)).unwrap();

                if width == 140 {
                    let rendered = buffer_text(terminal.backend()).replace(' ', "");
                    let label = match language {
                        ccr_cli::managers::TuiLanguage::English => "reasoning_effort",
                        ccr_cli::managers::TuiLanguage::SimplifiedChinese => "推理强度",
                    };
                    assert!(rendered.contains(label), "{width}x{height}: {rendered}");
                    assert!(rendered.contains("high"), "{width}x{height}: {rendered}");
                }
            }
        }
        crate::tui::i18n::set_language(ccr_cli::managers::TuiLanguage::English);
    }

    #[test]
    fn detail_token_tones_are_explicit() {
        let configured = detail_line(
            DetailField::new(
                DetailKey::Token,
                "configured (sk-a...7890)".to_string(),
                DetailTone::Success,
            ),
            false,
        );
        let missing = detail_line(
            DetailField::new(DetailKey::Token, "missing".to_string(), DetailTone::Warning),
            false,
        );

        assert_eq!(configured.spans[1].style, theme::success_style());
        assert_eq!(missing.spans[1].style, theme::warning_style());
    }

    #[test]
    fn detail_activity_group_labels_switch_count_not_usage_count() {
        let config = ProfileConfig::new();

        for lines in [
            generic_profile_detail_lines("g", &config, false, Platform::Gemini, false),
            claude_profile_detail_lines("c", &config, false, None, false),
            codex_profile_detail_lines("x", &config, false, None, false),
        ] {
            let text = detail_texts(&lines).join("\n");
            assert!(text.contains("switch_count"), "{text}");
            assert!(!text.contains("usage_count"), "{text}");
        }
    }

    #[test]
    fn wide_profile_draw_shows_shortcuts_only_in_global_footer() {
        let profile = ProfileItem {
            name: "fovts".to_string(),
            description: Some("fovts 公益".to_string()),
            is_current: true,
        };
        let mut app = sample_profile_app(profile, ProfileConfig::new());
        app.last_applied = Some(("Claude Code".to_string(), "fovts".to_string(), true, None));

        let mut terminal = Terminal::new(TestBackend::new(140, 32)).unwrap();
        terminal.draw(|frame| draw(frame, &mut app)).unwrap();

        let rendered = buffer_text(terminal.backend());
        assert_eq!(
            rendered.matches("Enter apply").count(),
            1,
            "shortcuts must only appear in the Keys footer: {rendered}"
        );
        assert!(rendered.contains("Applied successfully"), "{rendered}");
    }

    // ── Profile 详情 Usage 分组 ──────────────────────────────

    fn usage_row(
        source: SourceKind,
        provider: Option<&str>,
        requests: i64,
    ) -> TaggedProviderBreakdown {
        TaggedProviderBreakdown {
            source,
            breakdown: ProviderBreakdownDto {
                provider: provider.map(str::to_string),
                request_count: requests,
                input_tokens: 1_500,
                cache_read_tokens: 200,
                cache_creation_tokens: 100,
                output_tokens: 40,
                reasoning_output_tokens: 5,
                total_tokens: 1_845,
                cost_with_cache_usd: 1.5,
                cost_without_cache_usd: 2.0,
            },
        }
    }

    fn loaded_usage(rows: Vec<TaggedProviderBreakdown>) -> UsageLoadState {
        UsageLoadState::Loaded(UsageDataset { rows })
    }

    #[test]
    fn usage_section_shows_loading_before_dataset_arrives() {
        let idle = UsageLoadState::Idle;
        let loading = UsageLoadState::Loading;
        for state in [None, Some(&idle), Some(&loading)] {
            let texts = detail_texts(&usage_section_lines(
                SourceKind::Codex,
                Some("anyrouter"),
                state,
                false,
            ));
            assert!(
                texts
                    .iter()
                    .any(|text| text.contains("Usage (provider: anyrouter)")),
                "{texts:?}"
            );
            assert!(
                texts.iter().any(|text| text.contains("loading...")),
                "{texts:?}"
            );
        }
    }

    #[test]
    fn usage_section_reports_unattributed_without_provider_label() {
        // 数据集里的 provider=null 桶混着全部历史未归因用量;
        // 未填 provider 的 profile 不得回退展示该桶数字
        let state = loaded_usage(vec![usage_row(SourceKind::Codex, None, 999_999)]);
        let lines = usage_section_lines(SourceKind::Codex, None, Some(&state), false);
        let texts = detail_texts(&lines);

        assert!(
            texts
                .iter()
                .any(|text| text.contains("no provider label — usage unattributed")),
            "{texts:?}"
        );
        assert!(
            !texts.iter().any(|text| text.starts_with("requests")),
            "{texts:?}"
        );
        assert_eq!(
            lines.last().expect("usage status line present").spans[0].style,
            theme::muted_style()
        );
    }

    #[test]
    fn usage_section_reports_no_usage_for_unmatched_provider_or_platform() {
        let cases = [
            // provider 无对应行
            loaded_usage(vec![usage_row(SourceKind::Codex, Some("other"), 10)]),
            // provider 同名但平台不同(Claude 行不服务 Codex 详情)
            loaded_usage(vec![usage_row(SourceKind::Claude, Some("anyrouter"), 10)]),
            // 数据集整体为空
            UsageLoadState::Empty,
        ];
        for state in &cases {
            let texts = detail_texts(&usage_section_lines(
                SourceKind::Codex,
                Some("anyrouter"),
                Some(state),
                false,
            ));
            assert!(
                texts.iter().any(|text| text.contains("no usage recorded")),
                "{texts:?}"
            );
        }
    }

    #[test]
    fn usage_section_unsupported_and_error_render_semantic_status_lines() {
        let unsupported =
            UsageLoadState::Unsupported("provider_breakdown requires schema >= 14".to_string());
        let lines = usage_section_lines(
            SourceKind::Claude,
            Some("anthropic"),
            Some(&unsupported),
            false,
        );
        let status = lines.last().expect("unsupported status line present");
        assert!(plain_line_text(status).contains("schema >= 14"));
        assert_eq!(status.spans[0].style, theme::warning_style());

        let error = UsageLoadState::Error("query failed: boom".to_string());
        let lines = usage_section_lines(SourceKind::Claude, Some("anthropic"), Some(&error), false);
        let status = lines.last().expect("error status line present");
        assert!(plain_line_text(status).contains("boom"));
        assert_eq!(status.spans[0].style, theme::error_style());
    }

    #[test]
    fn usage_section_hit_renders_full_metric_fields() {
        let state = loaded_usage(vec![usage_row(
            SourceKind::Codex,
            Some("anyrouter"),
            56_200,
        )]);
        let lines = usage_section_lines(SourceKind::Codex, Some("anyrouter"), Some(&state), false);
        let texts = detail_texts(&lines);

        let has = |label: &str, value: &str| {
            texts
                .iter()
                .any(|text| text.starts_with(label) && text.contains(value))
        };
        assert!(has("requests", "56.2K"), "{texts:?}");
        assert!(has("input", "1.5K"), "{texts:?}");
        // output 含 reasoning: 40 + 5
        assert!(has("output", "45"), "{texts:?}");
        assert!(has("cache", "300"), "{texts:?}");
        assert!(has("total", "1.8K"), "{texts:?}");
        assert!(has("approx_cost", "$1.50"), "{texts:?}");
        let cost_line = lines
            .iter()
            .find(|line| plain_line_text(line).starts_with("approx_cost"))
            .expect("cost line present");
        assert_eq!(
            cost_line.spans[1].style,
            detail_tone_style(DetailTone::Cost)
        );
        assert!(
            cost_line.spans[1]
                .style
                .add_modifier
                .contains(Modifier::BOLD)
        );
        // 成本口径提示保留为 muted 单行
        assert!(
            texts.iter().any(|text| text.starts_with("note")
                && text.contains("approx official-equivalent")
                && text.contains("provider-level (all-time)")),
            "{texts:?}"
        );
        let note_line = lines
            .iter()
            .find(|line| plain_line_text(line).starts_with("note"))
            .expect("note line present");
        assert_eq!(note_line.spans[1].style, theme::muted_style());
    }

    #[test]
    fn usage_section_compact_merges_metrics_into_three_lines() {
        let state = loaded_usage(vec![usage_row(
            SourceKind::Codex,
            Some("anyrouter"),
            56_200,
        )]);
        let lines = usage_section_lines(SourceKind::Codex, Some("anyrouter"), Some(&state), true);
        let texts = detail_texts(&lines);

        // 空行 + 分组头 + requests/tokens/approx_cost 共 3 行指标
        assert_eq!(lines.len(), 5, "{texts:?}");
        assert!(
            texts.iter().any(|text| text.starts_with("tokens")
                && text.contains("in 1.5K")
                && text.contains("out 45")
                && text.contains("cache 300")),
            "{texts:?}"
        );
        assert!(
            !texts.iter().any(|text| text.starts_with("input")),
            "{texts:?}"
        );
        assert!(
            !texts.iter().any(|text| text.starts_with("note")),
            "{texts:?}"
        );
    }

    #[test]
    fn detail_lines_append_usage_section_after_activity_per_platform() {
        let state = loaded_usage(vec![
            usage_row(SourceKind::Claude, Some("anyrouter"), 111),
            usage_row(SourceKind::Codex, Some("anyrouter"), 222),
        ]);
        let mut config = ProfileConfig::new();
        config.provider = Some("anyrouter".to_string());

        let cases = [
            (
                claude_profile_detail_lines("c", &config, false, Some(&state), false),
                "111",
            ),
            (
                codex_profile_detail_lines("x", &config, false, Some(&state), false),
                "222",
            ),
        ];
        for (lines, requests) in cases {
            let texts = detail_texts(&lines);
            let activity_idx = texts
                .iter()
                .position(|text| text.contains("Activity"))
                .expect("Activity section present");
            let usage_idx = texts
                .iter()
                .position(|text| text.contains("Usage (provider: anyrouter)"))
                .expect("Usage section present");
            assert!(activity_idx < usage_idx, "{texts:?}");
            // 平台过滤正确: Claude/Codex 详情各取本平台行
            assert!(
                texts
                    .iter()
                    .any(|text| text.starts_with("requests") && text.contains(requests)),
                "{texts:?}"
            );
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn injected_loader_dataset_drives_detail_numbers_per_provider() {
        let rows = vec![
            usage_row(SourceKind::Codex, Some("anyrouter"), 1_500),
            usage_row(SourceKind::Codex, Some("fovts"), 42),
        ];
        let mut engine = UsageApp::with_loader(
            AsyncTaskExecutor::from_current_or_test(),
            Arc::new(move || Ok(UsageDataset { rows: rows.clone() })),
        );
        engine.refresh();
        for _ in 0..200 {
            if engine.tick() {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
        assert!(matches!(engine.state, UsageLoadState::Loaded(_)));

        let mut anyrouter = ProfileConfig::new();
        anyrouter.provider = Some("anyrouter".to_string());
        let mut fovts = ProfileConfig::new();
        fovts.provider = Some("fovts".to_string());

        // 选中不同 profile(不同 provider)时数字各自跟随
        let texts = detail_texts(&codex_profile_detail_lines(
            "a",
            &anyrouter,
            false,
            Some(&engine.state),
            false,
        ));
        assert!(
            texts
                .iter()
                .any(|text| text.starts_with("requests") && text.contains("1.5K")),
            "{texts:?}"
        );
        let texts = detail_texts(&codex_profile_detail_lines(
            "b",
            &fovts,
            false,
            Some(&engine.state),
            false,
        ));
        assert!(
            texts
                .iter()
                .any(|text| text.starts_with("requests") && text.contains("42")),
            "{texts:?}"
        );
    }

    #[test]
    fn usage_section_renders_across_viewports_for_claude_and_codex() {
        for (width, height) in [(140u16, 32u16), (100, 30), (80, 28)] {
            for platform in [Platform::Claude, Platform::Codex] {
                let profile = ProfileItem {
                    name: "p1".to_string(),
                    description: None,
                    is_current: false,
                };
                let mut config = ProfileConfig::new();
                config.provider = Some("anyrouter".to_string());

                let mut app = sample_profile_app(profile, config);
                app.tabs[0].platform = platform;
                app.tabs[0].label = format!("{} Profile", platform.display_name());

                let mut engine = UsageApp::with_loader(
                    AsyncTaskExecutor::from_current_or_test(),
                    Arc::new(|| Ok(UsageDataset { rows: Vec::new() })),
                );
                engine.state = loaded_usage(vec![
                    usage_row(SourceKind::Claude, Some("anyrouter"), 77),
                    usage_row(SourceKind::Codex, Some("anyrouter"), 77),
                ]);
                app.usage_app = Some(engine);
                // 详情行较多,滚到底部确认 Usage 分组真实渲染进缓冲区
                app.profile_detail_scroll = 500;

                let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
                terminal.draw(|frame| draw(frame, &mut app)).unwrap();

                let rendered = buffer_text(terminal.backend());
                assert!(
                    rendered.contains("approx_cost"),
                    "{platform:?} {width}x{height}: {rendered}"
                );
            }
        }
    }
}
