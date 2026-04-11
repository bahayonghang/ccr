// TUI UI rendering module
// Renders dynamic multi-platform profile switcher interface

use super::app::App;
use super::codex_auth;
use super::theme;
use super::toast::ToastKind;
use crate::models::{CodexRuntimeSummary, OpenAiAuthMethod, Platform, ProfileConfig};
use ccr_codex::CodexPlatform;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Padding, Paragraph, Tabs, Wrap},
};

// ═══════════════════════════════════════════════════════════
// Main render entry
// ═══════════════════════════════════════════════════════════

/// Render the main UI (responsive to terminal size)
pub fn draw(f: &mut Frame, app: &App) {
    let background = Block::default().style(theme::background_style());
    f.render_widget(background, f.area());

    let area = f.area();
    let mode = theme::viewport_mode(area.width, area.height);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(root_constraints(mode))
        .split(area);

    render_header(f, app, chunks[0]);

    let content_area = if app.current_platform() == Platform::Codex {
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

    if app.is_codex_auth_tab() {
        app.header_area.set(Some(chunks[0]));

        if let Some(ref codex_app) = app.codex_auth_app {
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
            Style::default()
                .fg(runtime_mode_color(summary.mode))
                .add_modifier(Modifier::BOLD),
            summary.profile_label(),
            summary.auth_label(),
        )
    } else {
        (
            "未解析".to_string(),
            Style::default().fg(theme::FG_MUTED),
            "-".to_string(),
            "-".to_string(),
        )
    };

    let lines = if compact {
        vec![Line::from(vec![
            Span::styled(" 当前驱动: ", Style::default().fg(theme::FG_SECONDARY)),
            Span::styled(mode_label, mode_style),
        ])]
    } else {
        vec![
            Line::from(vec![
                Span::styled(" 当前驱动: ", Style::default().fg(theme::FG_SECONDARY)),
                Span::styled(mode_label, mode_style),
            ]),
            Line::from(vec![
                Span::styled(" Profile: ", Style::default().fg(theme::FG_SECONDARY)),
                Span::styled(profile_label, Style::default().fg(theme::FG_PRIMARY)),
                Span::styled("  │  ", Style::default().fg(theme::BORDER)),
                Span::styled("Auth: ", Style::default().fg(theme::FG_SECONDARY)),
                Span::styled(auth_label, Style::default().fg(theme::FG_SUCCESS)),
            ]),
        ]
    };

    let banner = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_set(symbols::border::ROUNDED)
                .border_style(Style::default().fg(theme::CODEX_PRIMARY))
                .title(" 当前控制面 ")
                .title_style(theme::codex_style()),
        )
        .wrap(Wrap { trim: true });

    f.render_widget(banner, area);
}

fn runtime_mode_color(mode: crate::models::CodexRuntimeMode) -> ratatui::style::Color {
    match mode {
        crate::models::CodexRuntimeMode::ProfileOnly => theme::FG_SUCCESS,
        crate::models::CodexRuntimeMode::ProfileWithAuth => theme::FG_WARNING,
        crate::models::CodexRuntimeMode::ProfilePendingAuth => theme::FG_WARNING,
        crate::models::CodexRuntimeMode::RuntimeOnly => theme::FG_SECONDARY,
        crate::models::CodexRuntimeMode::Unresolved => theme::FG_MUTED,
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
        .constraints([Constraint::Percentage(48), Constraint::Percentage(52)])
        .split(area);
    (columns[0], columns[1])
}

fn profile_list_rail_layout(area: Rect) -> (Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(8), Constraint::Length(7)])
        .split(area);
    (chunks[0], chunks[1])
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

fn truncate_text(text: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    let len = text.chars().count();
    if len <= width {
        return text.to_string();
    }
    if width == 1 {
        return "…".to_string();
    }
    let mut out: String = text.chars().take(width - 1).collect();
    out.push('…');
    out
}

fn pad_text(text: &str, width: usize) -> String {
    let len = text.chars().count();
    if len >= width {
        return text.to_string();
    }
    let mut out = String::with_capacity(width);
    out.push_str(text);
    out.extend(std::iter::repeat_n(' ', width - len));
    out
}

fn profile_list_row(
    profile: &crate::tui::app::ProfileItem,
    is_selected: bool,
    accent: ratatui::style::Color,
    name_width: usize,
    desc_width: usize,
) -> Line<'static> {
    let selected_style = Style::default()
        .fg(theme::BG_PRIMARY)
        .bg(accent)
        .add_modifier(Modifier::BOLD);
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
            Style::default()
                .fg(theme::FG_SECONDARY)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme::FG_MUTED)
        };
        vec![
            Span::styled(name_cell, name_style),
            Span::styled("  ", Style::default().fg(theme::BORDER)),
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
    let tab_titles: Vec<Line> = app
        .tabs
        .iter()
        .enumerate()
        .map(|(i, tab)| {
            let is_active = i == app.active_tab;
            let indicator = if is_active { "▸ " } else { "  " };
            let style = if is_active {
                theme::platform_style_for(tab.platform)
            } else {
                theme::tab_normal_style()
            };
            Line::from(vec![
                Span::styled(indicator, style),
                Span::raw(format!("{} ", tab.platform.icon())),
                Span::styled(&tab.label, style),
            ])
        })
        .collect();

    let border_color = theme::platform_color_for(app.current_platform());
    let current_label = format!(
        " {} {} ",
        app.current_platform().icon(),
        app.current_platform().display_name()
    );

    let tabs = Tabs::new(tab_titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_set(symbols::border::ROUNDED)
                .border_style(Style::default().fg(border_color))
                .title(" 🚀 CCR - Configuration Switcher ")
                .title_alignment(Alignment::Center)
                .title_style(theme::platform_style_for(app.current_platform()))
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
        .highlight_style(theme::tab_highlight_style())
        .divider(Span::styled("  │  ", Style::default().fg(theme::BORDER)));

    f.render_widget(tabs, area);
}

// ═══════════════════════════════════════════════════════════
// Profile list rendering
// ═══════════════════════════════════════════════════════════

fn render_profile_workspace(f: &mut Frame, app: &App, area: Rect, mode: theme::ViewportMode) {
    match mode {
        theme::ViewportMode::Compact => {
            render_profile_list_panel(f, app, area);
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

fn render_profile_list_rail(f: &mut Frame, app: &App, area: Rect) {
    let (list_area, meta_area) = profile_list_rail_layout(area);

    render_profile_list_panel(f, app, list_area);
    render_profile_meta_panel(f, app, meta_area);
}

fn render_profile_list_panel(f: &mut Frame, app: &App, area: Rect) {
    app.list_area.set(Some(area));
    let profiles = app.current_page_profiles();
    let all_profiles = app.current_profiles();
    let platform = app.current_platform();
    let platform_name = platform.display_name();
    let accent = theme::platform_color_for(platform);

    let total_pages = app.total_pages();
    let total_profiles = all_profiles.len();
    let visible_start = if total_profiles == 0 {
        0
    } else {
        app.current_page * super::app::PAGE_SIZE + 1
    };
    let visible_end = if total_profiles == 0 {
        0
    } else {
        app.current_page * super::app::PAGE_SIZE + profiles.len()
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
                accent,
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
        .border_style(Style::default().fg(theme::BORDER))
        .title(" Selection ")
        .title_style(
            Style::default()
                .fg(theme::FG_SECONDARY)
                .add_modifier(Modifier::BOLD),
        )
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
    app: &App,
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

    if mode == theme::ViewportMode::Wide {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(7),
                Constraint::Min(10),
                Constraint::Length(3),
            ])
            .split(area);

        render_profile_summary_block(
            f,
            app,
            chunks[0],
            profile.name.as_str(),
            config,
            profile.is_current,
        );
        render_profile_details(f, app, chunks[1]);
        render_profile_status_strip(f, app, chunks[2], profile.name.as_str());
    } else {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(7), Constraint::Min(0)])
            .split(area);

        render_profile_summary_block(
            f,
            app,
            chunks[0],
            profile.name.as_str(),
            config,
            profile.is_current,
        );
        render_profile_details(f, app, chunks[1]);
    }
}

fn render_profile_details(f: &mut Frame, app: &App, area: Rect) {
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
    } else {
        generic_profile_detail_lines(profile.name.as_str(), config, profile.is_current)
    };

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });
    f.render_widget(paragraph, area);
}

fn render_profile_summary_block(
    f: &mut Frame,
    app: &App,
    area: Rect,
    profile_name: &str,
    config: &ProfileConfig,
    is_current: bool,
) {
    let platform = app.current_platform();
    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(symbols::border::ROUNDED)
        .border_style(Style::default().fg(theme::platform_color_for(platform)))
        .title(" Focus ")
        .title_style(theme::platform_style_for(platform))
        .padding(Padding::horizontal(1));

    let lines: Vec<Line> = profile_summary_strings(
        platform,
        profile_name,
        config,
        is_current,
        app.last_applied.as_ref(),
    )
    .into_iter()
    .map(profile_summary_line)
    .collect();

    let paragraph = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}

fn render_profile_status_strip(f: &mut Frame, app: &App, area: Rect, profile_name: &str) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(symbols::border::ROUNDED)
        .border_style(Style::default().fg(theme::BORDER))
        .title(" Status ")
        .title_style(
            Style::default()
                .fg(theme::FG_SECONDARY)
                .add_modifier(Modifier::BOLD),
        )
        .padding(Padding::horizontal(1));

    let mut text = footer_text(app);
    if let Some(action) = last_apply_message(profile_name, app.last_applied.as_ref()) {
        text = format!("{action}  │  {text}");
    }

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
        detail_line("usage_count", config.usage_count().to_string()),
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
            if config
                .auth_token
                .as_ref()
                .is_some_and(|token| !token.trim().is_empty())
            {
                "configured".to_string()
            } else {
                "missing".to_string()
            }
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
        detail_line("usage_count", config.usage_count().to_string()),
        detail_line("tags", tags_text(config)),
        detail_line("token", token_state),
    ]
}

fn section_line(title: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled("▌ ", Style::default().fg(theme::FG_INFO)),
        Span::styled(
            title.to_string(),
            Style::default()
                .fg(theme::FG_SECONDARY)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        ),
    ])
}

fn detail_line(label: &str, value: String) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("{label:<16}"),
            Style::default()
                .fg(theme::FG_SECONDARY)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(value.clone(), detail_value_style(label, &value)),
    ])
}

fn detail_value_style(label: &str, value: &str) -> Style {
    let normalized_label = label.trim().to_ascii_lowercase();
    let normalized_value = value.trim().to_ascii_lowercase();

    if normalized_value == "yes"
        || normalized_value == "enabled"
        || normalized_value == "configured"
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

    Style::default().fg(theme::FG_PRIMARY)
}

fn profile_summary_line(text: String) -> Line<'static> {
    if let Some((label, value)) = text.split_once(':') {
        return Line::from(vec![
            Span::styled(
                format!("{label}: "),
                Style::default()
                    .fg(theme::FG_SECONDARY)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                value.trim().to_string(),
                detail_value_style(label, value.trim()),
            ),
        ]);
    }

    Line::from(Span::styled(text, Style::default().fg(theme::FG_PRIMARY)))
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
        format!("Profiles: {total_profiles}"),
        format!("Page: {}/{}", current_page + 1, total_pages.max(1)),
        "Legend: ● current · ▶ selected".to_string(),
        "Enter apply · r reload · Tab switch".to_string(),
    ]
}

fn profile_summary_strings(
    platform: Platform,
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
        format!("Description: {}", opt_text(config.description.as_deref())),
        format!("Model: {}", opt_text(config.model.as_deref())),
    ];

    if platform == Platform::Codex {
        let auth_mode = CodexPlatform::profile_auth_mode(config);
        lines.push(format!(
            "Routing: {} · {}",
            opt_text(config.provider.as_deref()),
            auth_mode.as_str()
        ));
        lines.push(format!(
            "Base URL: {}",
            opt_text(config.base_url.as_deref())
        ));
    } else {
        lines.push(format!(
            "Base URL: {}",
            opt_text(config.base_url.as_deref())
        ));
    }

    lines.push(format!(
        "Last action: {}",
        last_apply_message(name, last_applied).unwrap_or_else(|| "None this session".to_string())
    ));

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
                Style::default().fg(theme::FG_SECONDARY),
            )),
            Line::from(""),
            Line::from(Span::styled(
                error.to_string(),
                Style::default().fg(theme::FG_PRIMARY),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Fix the file content or path, then press 'r' to reload.".to_string(),
                Style::default().fg(theme::FG_MUTED),
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
            Style::default().fg(theme::FG_SECONDARY),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Or 'ccr add' to create a new configuration".to_string(),
            Style::default().fg(theme::FG_MUTED),
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
            Line::from(Span::styled(
                error.to_string(),
                Style::default().fg(theme::FG_PRIMARY),
            )),
        ];
    }

    if let Some(error) = app.current_profile_status_error() {
        return vec![
            Line::from(Span::styled(
                "⚠ Current profile state unavailable".to_string(),
                theme::empty_hint_style(),
            )),
            Line::from(""),
            Line::from(Span::styled(
                error.to_string(),
                Style::default().fg(theme::FG_PRIMARY),
            )),
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
                .border_style(Style::default().fg(theme::BORDER))
                .title(" Keys ")
                .title_alignment(Alignment::Center)
                .title_style(
                    Style::default()
                        .fg(theme::FG_MUTED)
                        .add_modifier(Modifier::ITALIC),
                ),
        )
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

    let shortcuts =
        format!("Tab switch  │  {page_hint}↑↓/jk select  │  Enter apply  │  r reload  │  q quit");

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
            ToastKind::Warning => Style::default()
                .fg(theme::FG_WARNING)
                .add_modifier(Modifier::BOLD),
            ToastKind::Info => Style::default().fg(theme::FG_SECONDARY),
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
                codex_runtime_summary: None,
                instance: None,
            }],
            active_tab: 0,
            selected_index: 0,
            current_page: 0,
            selected_profile_name: Some(profile.name),
            toasts: ToastManager::new(),
            last_applied: None,
            codex_auth_app: None,
            codex_auth_error: None,
            last_codex_action: None,
            header_area: Cell::new(None),
            list_area: Cell::new(None),
        }
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

        assert_eq!(list_area.width, 58);
        assert_eq!(context_area.width, 62);
    }

    #[test]
    fn profile_list_rail_layout_keeps_full_selection_panel_visible() {
        let (list_area, meta_area) = profile_list_rail_layout(Rect::new(0, 0, 58, 20));

        assert_eq!(list_area.height, 13);
        assert_eq!(meta_area.height, 7);
    }

    #[test]
    fn column_widths_prioritize_name_without_hiding_description_too_early() {
        assert_eq!(column_widths(51), (45, 0));
        assert_eq!(column_widths(52), (18, 28));
        assert_eq!(column_widths(58), (20, 32));
    }

    #[test]
    fn profile_list_row_shows_more_of_long_profile_names_on_wider_list_rail() {
        let profile = ProfileItem {
            name: "anyrouter_temp_backup".to_string(),
            description: Some("AnyRouter temp profile".to_string()),
            is_current: false,
        };

        let rendered = plain_line_text(&profile_list_row(
            &profile,
            true,
            theme::CLAUDE_PRIMARY,
            20,
            32,
        ));

        assert!(rendered.contains("anyrouter_temp"), "{rendered}");
        assert!(rendered.contains("AnyRouter temp profile"), "{rendered}");
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
        assert!(lines.iter().any(|line| line.contains("Page: 2/2")));
        assert!(lines.iter().any(|line| line.contains("Legend:")));
    }

    #[test]
    fn profile_summary_strings_include_last_apply_feedback() {
        let mut config = ProfileConfig::new();
        config.description = Some("fovts 公益".to_string());
        config.model = Some("gpt-5.4".to_string());

        let lines = profile_summary_strings(
            Platform::Codex,
            "fovts",
            &config,
            true,
            Some(&("Codex Profile".to_string(), "fovts".to_string(), true, None)),
        );

        assert!(
            lines
                .iter()
                .any(|line| line.contains("Status: Current · Enabled"))
        );
        assert!(lines.iter().any(|line| line.contains("Model: gpt-5.4")));
        assert!(
            lines
                .iter()
                .any(|line| line.contains("Last action: Applied successfully"))
        );
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

        let app = sample_profile_app(profile, config);
        let mut terminal = Terminal::new(TestBackend::new(32, 14)).unwrap();

        terminal
            .draw(|frame| render_profile_details(frame, &app, frame.area()))
            .unwrap();

        let rendered = buffer_text(terminal.backend());
        assert!(rendered.contains("https://2capi.com"), "{rendered}");
        assert!(rendered.contains("compatible"), "{rendered}");
        assert!(rendered.contains("chat/complet"), "{rendered}");
    }
}
