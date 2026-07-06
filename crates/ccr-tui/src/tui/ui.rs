// TUI UI rendering module
// Renders dynamic multi-platform profile switcher interface

use super::app::App;
use super::claude_auth;
use super::codex_auth;
use super::opencode_auth;
use super::theme;
use super::toast::ToastKind;
use super::usage;
use ccr_cli::models::{CodexRuntimeSummary, OpenAiAuthMethod, Platform, ProfileConfig};
use ccr_codex::CodexPlatform;
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

    let content_area = if app.current_platform() == Platform::Codex
        && !app.is_opencode_auth_tab()
        && !app.is_usage_tab()
    {
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
    } else if app.is_usage_tab() {
        app.header_area.set(Some(chunks[0]));
        app.ensure_usage_app();

        if let Some(ref mut usage_app) = app.usage_app {
            usage::ui::draw_embedded(f, usage_app, content_area, chunks[2], mode);
        } else {
            usage::ui::draw_loading_placeholder(
                f,
                content_area,
                chunks[2],
                mode,
                app.usage_error.as_deref(),
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
            summary.mode.label().to_string(),
            runtime_mode_style(summary.mode),
            summary.profile_label(),
            summary.auth_label(),
        )
    } else {
        (
            "未解析".to_string(),
            theme::muted_style(),
            "-".to_string(),
            "-".to_string(),
        )
    };

    let lines = if compact {
        vec![Line::from(vec![
            Span::styled(" 当前驱动: ", theme::secondary_text_style()),
            Span::styled(mode_label, mode_style),
        ])]
    } else {
        vec![
            Line::from(vec![
                Span::styled(" 当前驱动: ", theme::secondary_text_style()),
                Span::styled(mode_label, mode_style),
            ]),
            Line::from(vec![
                Span::styled(" Profile: ", theme::secondary_text_style()),
                Span::styled(profile_label, theme::primary_text_style()),
                Span::styled("  │  ", Style::default().fg(theme::border())),
                Span::styled("Auth: ", theme::secondary_text_style()),
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
                .title(" 当前控制面 ")
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
        .constraints([Constraint::Percentage(54), Constraint::Percentage(46)])
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
        app.current_tab().label
    );

    let tabs = Tabs::new(tab_titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_set(symbols::border::ROUNDED)
                .border_style(Style::default().fg(theme::border()))
                .title(" 🚀 CCR - Configuration Switcher ")
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
        compact_tab_label(&tab.label)
    } else {
        tab.label.as_str()
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

fn compact_tab_label(label: &str) -> &str {
    match label {
        "Claude Auth" => "Auth",
        "Claude Code" => "Claude",
        "Codex Auth" => "CxAuth",
        "OpenCode Auth" => "Open",
        "Codex Profile" => "Codex",
        "Usage" => "Usage",
        _ => label,
    }
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
        format!(" {} Profiles ", platform_name)
    } else if total_pages > 1 {
        format!(
            " {} Profiles ({})  {}-{} / {}  Page {}/{} ",
            platform_name,
            total_profiles,
            visible_start,
            visible_end,
            total_profiles,
            app.current_page + 1,
            total_pages
        )
    } else {
        format!(" {} Profiles ({}) ", platform_name, total_profiles)
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
        .title(" Selection ")
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
            .title(" Focus ")
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
            .title(" Focus ")
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
    let summary = profile_summary_strings(
        profile_name.as_str(),
        config,
        is_current,
        app.last_applied.as_ref(),
    );
    let summary_height = summary.len() as u16 + 2;

    if mode == theme::ViewportMode::Wide {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(summary_height),
                Constraint::Min(10),
                Constraint::Length(3),
            ])
            .split(area);

        render_profile_summary_block(f, app, chunks[0], summary);
        render_profile_details(f, app, chunks[1]);
        render_profile_status_strip(f, app, chunks[2], profile_name.as_str());
    } else {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(summary_height), Constraint::Min(0)])
            .split(area);

        render_profile_summary_block(f, app, chunks[0], summary);
        render_profile_details(f, app, chunks[1]);
    }
}

fn render_profile_details(f: &mut Frame, app: &mut App, area: Rect) {
    app.detail_area.set(Some(area));
    let platform = app.current_platform();
    let accent = theme::platform_color_for(platform);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(symbols::border::ROUNDED)
        .border_style(Style::default().fg(accent))
        .title(" Context ")
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

    let lines = if platform == Platform::Codex {
        codex_profile_detail_lines(profile.name.as_str(), config, profile.is_current)
    } else if platform == Platform::Claude {
        claude_profile_detail_lines(profile.name.as_str(), config, profile.is_current)
    } else {
        generic_profile_detail_lines(profile.name.as_str(), config, profile.is_current)
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

fn render_profile_summary_block(f: &mut Frame, app: &App, area: Rect, summary: Vec<String>) {
    let platform = app.current_platform();
    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(symbols::border::ROUNDED)
        .border_style(Style::default().fg(theme::platform_color_for(platform)))
        .title(" Focus ")
        .title_style(theme::platform_style_for(platform))
        .padding(Padding::horizontal(1));

    let lines: Vec<Line> = summary.into_iter().map(profile_summary_line).collect();

    let paragraph = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}

fn render_profile_status_strip(f: &mut Frame, app: &App, area: Rect, profile_name: &str) {
    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(theme::border()))
        .title(" Status ")
        .title_style(theme::secondary_text_emphasis_style());

    // 快捷键只保留底部全局 Keys footer 一处; strip 只反馈 apply 结果/toast
    let text = last_apply_message(profile_name, app.last_applied.as_ref())
        .or_else(|| app.toasts.active().map(|toast| toast.message.clone()))
        .unwrap_or_default();

    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}

fn generic_profile_detail_lines(
    name: &str,
    config: &ProfileConfig,
    is_current: bool,
) -> Vec<Line<'static>> {
    vec![
        section_line(" Overview "),
        detail_line("name", name.to_string()),
        detail_line("current", yes_no(is_current)),
        detail_line("enabled", yes_no(config.is_enabled())),
        detail_line("description", opt_text(config.description.as_deref())),
        Line::from(""),
        section_line(" Runtime "),
        detail_line("base_url", opt_text(config.base_url.as_deref())),
        detail_line("model", opt_text(config.model.as_deref())),
        detail_line("account", opt_text(config.account.as_deref())),
        Line::from(""),
        section_line(" Activity "),
        detail_line("switch_count", config.usage_count().to_string()),
        detail_line("tags", tags_text(config)),
    ]
}

fn codex_profile_detail_lines(
    name: &str,
    config: &ProfileConfig,
    is_current: bool,
) -> Vec<Line<'static>> {
    let auth_mode = CodexPlatform::profile_auth_mode(config);
    let login_method =
        CodexPlatform::profile_openai_login_method(config).map(|method| match method {
            OpenAiAuthMethod::Chatgpt => "chatgpt".to_string(),
            OpenAiAuthMethod::Api => "api".to_string(),
        });
    let token_state = match auth_mode.as_str() {
        "openai_api_key" | "provider_env_key" => {
            configured_token_text(config).unwrap_or_else(|| "missing".to_string())
        }
        _ => "-".to_string(),
    };

    vec![
        section_line(" Overview "),
        detail_line("name", name.to_string()),
        detail_line("current", yes_no(is_current)),
        detail_line("enabled", yes_no(config.is_enabled())),
        detail_line("description", opt_text(config.description.as_deref())),
        Line::from(""),
        section_line(" Routing/Auth "),
        detail_line("provider_type", opt_text(config.provider_type.as_deref())),
        detail_line("provider", opt_text(config.provider.as_deref())),
        detail_line("auth_mode", auth_mode.as_str().to_string()),
        detail_line("auth_source", CodexPlatform::profile_auth_source(config)),
        detail_line("token", token_state),
        detail_line(
            "openai_login",
            login_method.unwrap_or_else(|| "-".to_string()),
        ),
        detail_line(
            "env_key",
            opt_text(codex_platform_value(config, "env_key").as_deref()),
        ),
        detail_line(
            "wire_api",
            opt_text(codex_platform_value(config, "wire_api").as_deref()),
        ),
        detail_line(
            "requires_openai",
            bool_text(
                codex_platform_value(config, "requires_openai_auth")
                    .as_deref()
                    .and_then(|value| match value {
                        "true" => Some(true),
                        "false" => Some(false),
                        _ => None,
                    }),
            )
            .to_string(),
        ),
        Line::from(""),
        section_line(" Engine "),
        detail_line("base_url", opt_text(config.base_url.as_deref())),
        detail_line("model", opt_text(config.model.as_deref())),
        detail_line("small_fast", opt_text(config.small_fast_model.as_deref())),
        detail_line("account", opt_text(config.account.as_deref())),
        Line::from(""),
        section_line(" Activity "),
        detail_line("switch_count", config.usage_count().to_string()),
        detail_line("tags", tags_text(config)),
    ]
}

fn claude_profile_detail_lines(
    name: &str,
    config: &ProfileConfig,
    is_current: bool,
) -> Vec<Line<'static>> {
    let auth_mode = ccr_cli::platforms::ClaudePlatform::profile_auth_mode(config);
    let provider_type = opt_text(config.provider_type.as_deref());
    let provider = opt_text(config.provider.as_deref());
    let token_state = if matches!(auth_mode, ccr_cli::models::ClaudeProfileAuthMode::ApiKey) {
        configured_token_text(config).unwrap_or_else(|| "missing".to_string())
    } else {
        "subscription".to_string()
    };

    let mut lines = vec![
        section_line(" Overview "),
        detail_line("name", name.to_string()),
        detail_line("current", yes_no(is_current)),
        detail_line("enabled", yes_no(config.is_enabled())),
        detail_line("description", opt_text(config.description.as_deref())),
        Line::from(""),
        section_line(" Engine "),
        detail_line("base_url", opt_text(config.base_url.as_deref())),
        detail_line("model", opt_text(config.model.as_deref())),
        detail_line("small_fast", opt_text(config.small_fast_model.as_deref())),
        detail_line("account", opt_text(config.account.as_deref())),
        Line::from(""),
        section_line(" Routing/Auth "),
        detail_line("auth_mode", auth_mode.as_str().to_string()),
        detail_line(
            "auth_source",
            ccr_cli::platforms::ClaudePlatform::profile_auth_source(config),
        ),
        detail_line("token", token_state),
    ];

    if provider_type != "-" {
        lines.push(detail_line("provider_type", provider_type));
    }

    if provider != "-" {
        lines.push(detail_line("provider", provider));
    }

    lines.extend([
        Line::from(""),
        section_line(" Activity "),
        detail_line("switch_count", config.usage_count().to_string()),
        detail_line("tags", tags_text(config)),
    ]);

    lines
}

// token 行展示统一走这里: 掩码策略唯一归属 ccr_core::mask_sensitive,
// 这里只负责拼接 `configured (<masked>)` 展示形态, 不得输出明文。
fn configured_token_text(config: &ProfileConfig) -> Option<String> {
    config
        .auth_token
        .as_ref()
        .map(|token| token.expose().trim())
        .filter(|token| !token.is_empty())
        .map(|token| format!("configured ({})", ccr_core::mask_sensitive(token)))
}

fn section_line(title: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled("▌ ", theme::info_style()),
        Span::styled(
            title.to_string(),
            theme::secondary_text_emphasis_style().add_modifier(Modifier::UNDERLINED),
        ),
    ])
}

fn detail_line(label: &str, value: String) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("{label:<16}"),
            theme::secondary_text_emphasis_style(),
        ),
        Span::styled(value.clone(), detail_value_style(label, &value)),
    ])
}

fn detail_value_style(label: &str, value: &str) -> Style {
    let normalized_label = label.trim().to_ascii_lowercase();
    let normalized_value = value.trim().to_ascii_lowercase();

    if normalized_value == "yes"
        || normalized_value == "enabled"
        || normalized_value.starts_with("configured")
        || normalized_value == "current"
    {
        return theme::success_style();
    }

    if normalized_value == "no" || normalized_value == "disabled" || normalized_value == "missing" {
        return theme::warning_style();
    }

    if normalized_value == "-" || normalized_value == "none" || normalized_value == "unresolved" {
        return theme::muted_style();
    }

    if normalized_label.contains("auth")
        || normalized_label.contains("provider")
        || normalized_label.contains("login")
        || normalized_label.contains("base_url")
        || normalized_label.contains("model")
        || normalized_label.contains("account")
    {
        return theme::info_style();
    }

    theme::primary_text_style()
}

fn profile_summary_line(text: String) -> Line<'static> {
    if let Some((label, value)) = text.split_once(':') {
        return Line::from(vec![
            Span::styled(format!("{label}: "), theme::secondary_text_emphasis_style()),
            Span::styled(
                value.trim().to_string(),
                detail_value_style(label, value.trim()),
            ),
        ]);
    }

    Line::from(Span::styled(text, theme::primary_text_style()))
}

fn opt_text(value: Option<&str>) -> String {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("-")
        .to_string()
}

fn yes_no(value: bool) -> String {
    if value {
        "yes".to_string()
    } else {
        "no".to_string()
    }
}

fn bool_text(value: Option<bool>) -> &'static str {
    match value {
        Some(true) => "yes",
        Some(false) => "no",
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

fn profile_meta_strings(
    total_profiles: usize,
    current_page: usize,
    total_pages: usize,
    selected: Option<&crate::tui::app::ProfileItem>,
) -> Vec<String> {
    let selection = selected
        .map(|profile| {
            if profile.is_current {
                format!("Selected: {} (current)", profile.name)
            } else {
                format!("Selected: {}", profile.name)
            }
        })
        .unwrap_or_else(|| "Selected: -".to_string());

    vec![
        selection,
        format!(
            "Profiles: {total_profiles} · Page: {}/{}",
            current_page + 1,
            total_pages.max(1)
        ),
        "Legend: ● current · ▶ selected".to_string(),
    ]
}

// Focus 块只保留 Context 分组里没有的信息: 选中态/当前态 + 最近 apply 结果。
// Description/Model/Base URL 等在 Context 的 Overview/Engine 分组已完整展示。
fn profile_summary_strings(
    name: &str,
    config: &ProfileConfig,
    is_current: bool,
    last_applied: Option<&(String, String, bool, Option<String>)>,
) -> Vec<String> {
    let mut lines = vec![
        format!("Name: {name}"),
        format!(
            "Status: {} · {}",
            if is_current { "Current" } else { "Available" },
            if config.is_enabled() {
                "Enabled"
            } else {
                "Disabled"
            }
        ),
    ];

    if let Some(message) = last_apply_message(name, last_applied) {
        lines.push(format!("Last apply: {message}"));
    }

    lines
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
        Some("Applied successfully".to_string())
    } else {
        Some(format!(
            "Apply failed{}",
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
                format!("⚠ Failed to load {} profiles", platform_name),
                theme::empty_hint_style(),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "CCR could not read the profile source below:".to_string(),
                theme::secondary_text_style(),
            )),
            Line::from(""),
            Line::from(Span::styled(error.to_string(), theme::primary_text_style())),
            Line::from(""),
            Line::from(Span::styled(
                "Fix the file content or path, then press 'r' to reload.".to_string(),
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
            format!("📭 No {} configurations found", platform_name),
            theme::empty_hint_style(),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("Run 'ccr platform init {}' to initialize", short_name),
            theme::secondary_text_style(),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Or 'ccr add' to create a new configuration".to_string(),
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
                "⚠ Profile list unavailable".to_string(),
                theme::empty_hint_style(),
            )),
            Line::from(""),
            Line::from(Span::styled(error.to_string(), theme::primary_text_style())),
        ];
    }

    if let Some(error) = app.current_profile_status_error() {
        return vec![
            Line::from(Span::styled(
                "⚠ Current profile state unavailable".to_string(),
                theme::empty_hint_style(),
            )),
            Line::from(""),
            Line::from(Span::styled(error.to_string(), theme::primary_text_style())),
        ];
    }

    vec![Line::from("No profile selected")]
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
    let paragraph = Paragraph::new(footer_text(app))
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_set(symbols::border::ROUNDED)
                .border_style(Style::default().fg(theme::border()))
                .title(" Keys ")
                .title_alignment(Alignment::Center)
                .title_style(theme::muted_style()),
        )
        .style(theme::secondary_text_style())
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

fn footer_text(app: &App) -> String {
    let page_hint = if app.total_pages() > 1 {
        "←→ page  │  "
    } else {
        ""
    };

    let shortcuts = format!(
        "Tab/Shift+Tab switch  │  {page_hint}↑↓/jk select  │  PgUp/PgDn details  │  Enter apply  │  r reload  │  q quit"
    );

    if let Some(toast) = app.toasts.active() {
        format!("{}  │  {}", toast.message, shortcuts)
    } else {
        shortcuts
    }
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
    use crate::tui::theme::ViewportMode;
    use crate::tui::toast::ToastManager;
    use indexmap::IndexMap;
    use ratatui::{Terminal, backend::TestBackend};
    use std::cell::Cell;

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
        let mut profile_configs = IndexMap::new();
        profile_configs.insert(profile.name.clone(), config);

        App {
            tabs: vec![PlatformTab {
                platform: Platform::Claude,
                variant: TabVariant::Profile,
                label: "Claude Code".to_string(),
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
            usage_error: None,
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
    fn wide_profile_workspace_layout_favors_list_rail_more_than_before() {
        let (list_area, context_area) = wide_profile_workspace_layout(Rect::new(0, 0, 120, 20));

        assert!(list_area.width > context_area.width);
        assert_eq!(list_area.width + context_area.width, 120);
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
    fn summary_and_detail_plain_values_use_explicit_palette_foreground() {
        let summary = profile_summary_line("Name: fovts".to_string());
        assert_eq!(summary.spans[0].style.fg, Some(theme::subtext()));
        assert_eq!(summary.spans[1].style.fg, Some(theme::text()));

        let detail = detail_line("switch_count", "42".to_string());
        assert_eq!(detail.spans[0].style.fg, Some(theme::subtext()));
        assert_eq!(detail.spans[1].style.fg, Some(theme::text()));
    }

    #[test]
    fn footer_uses_terminal_foreground_with_modifier_hierarchy() {
        let profile = ProfileItem {
            name: "default".to_string(),
            description: Some("Default profile".to_string()),
            is_current: false,
        };
        let app = sample_profile_app(profile, ProfileConfig::new());
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

        assert_eq!(footer_cell.fg, theme::subtext());
        assert_ne!(footer_cell.fg, theme::text());
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
    fn profile_summary_strings_focus_on_primary_profile_facts() {
        let mut config = ProfileConfig::new();
        config.description = Some("fovts 公益".to_string());
        config.model = Some("gpt-5.4".to_string());
        config.base_url = Some("https://example.com/v1".to_string());

        let lines = profile_summary_strings(
            "fovts",
            &config,
            true,
            Some(&("Codex Profile".to_string(), "fovts".to_string(), true, None)),
        );

        assert!(lines.iter().any(|line| line.contains("Name: fovts")));
        assert!(
            lines
                .iter()
                .any(|line| line.contains("Status: Current · Enabled"))
        );
        assert!(
            lines
                .iter()
                .any(|line| line.contains("Last apply: Applied successfully"))
        );
        // Context 的 Overview/Engine 分组已展示这些信息, Focus 不再重复
        assert!(!lines.iter().any(|line| line.starts_with("Description:")));
        assert!(!lines.iter().any(|line| line.starts_with("Model:")));
        assert!(!lines.iter().any(|line| line.starts_with("Base URL:")));
    }

    #[test]
    fn profile_summary_strings_skip_apply_line_for_other_profiles() {
        let config = ProfileConfig::new();

        let lines = profile_summary_strings(
            "fovts",
            &config,
            false,
            Some(&("Codex Profile".to_string(), "other".to_string(), true, None)),
        );

        assert_eq!(lines.len(), 2);
        assert!(!lines.iter().any(|line| line.contains("Last apply:")));
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
        assert!(rendered.contains("Status"), "{rendered}");
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
            .draw(|frame| render_profile_details(frame, &mut app, frame.area()))
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
            .draw(|frame| render_profile_details(frame, &mut app, frame.area()))
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

        let texts = detail_texts(&claude_profile_detail_lines("api", &config, false));
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

        let texts = detail_texts(&claude_profile_detail_lines("api", &config, false));
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
        let texts = detail_texts(&claude_profile_detail_lines("m", &claude_missing, false));
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
        ));
        assert!(
            token_line(&texts).trim_end().ends_with("subscription"),
            "{texts:?}"
        );

        let mut codex_missing = ProfileConfig::new();
        codex_missing
            .platform_data
            .insert("auth_mode".to_string(), serde_json::json!("openai_api_key"));
        let texts = detail_texts(&codex_profile_detail_lines("m", &codex_missing, false));
        assert!(
            token_line(&texts).trim_end().ends_with("missing"),
            "{texts:?}"
        );

        let mut codex_no_auth = ProfileConfig::new();
        codex_no_auth
            .platform_data
            .insert("auth_mode".to_string(), serde_json::json!("no_auth"));
        let texts = detail_texts(&codex_profile_detail_lines("n", &codex_no_auth, false));
        assert!(token_line(&texts).trim_end().ends_with('-'), "{texts:?}");
    }

    #[test]
    fn codex_detail_token_line_is_masked_and_lives_in_routing_auth_group() {
        let mut config = ProfileConfig::new();
        config
            .platform_data
            .insert("auth_mode".to_string(), serde_json::json!("openai_api_key"));
        config.auth_token = Some(ccr_core::Secret::new("sk-test-abcdef1234567890"));

        let texts = detail_texts(&codex_profile_detail_lines("codex-key", &config, false));

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
            routing_idx < token_idx && token_idx < engine_idx,
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
        ))
        .join("\n");
        let codex_text = detail_texts(&codex_profile_detail_lines(
            "codex-key",
            &codex_config,
            true,
        ))
        .join("\n");

        assert!(!claude_text.contains(plaintext), "{claude_text}");
        assert!(!codex_text.contains(plaintext), "{codex_text}");
    }

    #[test]
    fn detail_value_style_keeps_configured_masked_value_green() {
        assert_eq!(
            detail_value_style("token", "configured (sk-a...7890)"),
            theme::success_style()
        );
        assert_eq!(
            detail_value_style("token", "missing"),
            theme::warning_style()
        );
    }

    #[test]
    fn detail_activity_group_labels_switch_count_not_usage_count() {
        let config = ProfileConfig::new();

        for lines in [
            generic_profile_detail_lines("g", &config, false),
            claude_profile_detail_lines("c", &config, false),
            codex_profile_detail_lines("x", &config, false),
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
}
