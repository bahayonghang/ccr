// TUI UI rendering module
// Renders dynamic multi-platform profile switcher interface

use super::app::App;
use super::codex_auth;
use super::theme;
use super::toast::ToastKind;
use crate::models::{CodexRuntimeSummary, OpenAiAuthMethod, Platform, ProfileConfig};
use crate::platforms::codex::CodexPlatform;
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
    let (constraints, compact) = responsive_constraints(area.height);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    render_header(f, app, chunks[0]);

    let content_area = if app.current_platform() == Platform::Codex {
        let runtime_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(if compact { 3 } else { 4 }),
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

        render_codex_runtime_banner(f, runtime_chunks[0], runtime_summary, compact);
        runtime_chunks[1]
    } else {
        chunks[1]
    };

    if app.is_codex_auth_tab() {
        app.header_area.set(Some(chunks[0]));
        app.list_area.set(Some(content_area));

        if let Some(ref codex_app) = app.codex_auth_app {
            codex_auth::ui::draw_embedded(f, codex_app, content_area, chunks[2], compact);
        } else {
            codex_auth::ui::draw_loading_placeholder(
                f,
                content_area,
                chunks[2],
                compact,
                app.codex_auth_error.as_deref(),
            );
        }
    } else {
        let content_chunks = if compact {
            vec![content_area]
        } else {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(52), Constraint::Percentage(48)])
                .split(content_area)
                .to_vec()
        };

        app.header_area.set(Some(chunks[0]));
        app.list_area.set(Some(content_chunks[0]));

        render_profile_list(f, app, content_chunks[0]);
        if !compact {
            render_profile_details(f, app, content_chunks[1]);
        }

        if compact {
            render_toast(f, app, chunks[2]);
        } else {
            render_footer(f, app, chunks[2]);
        }
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

/// Calculate responsive layout constraints based on terminal height
fn responsive_constraints(height: u16) -> (Vec<Constraint>, bool) {
    let compact = height < 20;
    let constraints = if compact {
        vec![
            Constraint::Length(3), // Header with tabs
            Constraint::Min(0),    // Profile list / Codex content
            Constraint::Length(2), // Toast only (compact)
        ]
    } else {
        vec![
            Constraint::Length(3), // Header with tabs
            Constraint::Min(0),    // Profile list / Codex content
            Constraint::Length(5), // Footer: shortcuts + toast
        ]
    };
    (constraints, compact)
}

/// Calculate column widths for profile list (responsive to terminal width)
/// Returns (name_width, desc_width) — desc_width is 0 when terminal is narrow
fn column_widths(area_width: u16) -> (usize, usize) {
    let inner = area_width.saturating_sub(4) as usize;
    let gap = 2usize;
    let available = inner.saturating_sub(gap);

    // Narrow terminal: name only, no description
    if area_width < 60 {
        return (available, 0);
    }

    let min_name = 12usize;
    let min_desc = 10usize;
    let mut name_width = available * 3 / 10;
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

    let tabs = Tabs::new(tab_titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_set(symbols::border::ROUNDED)
                .border_style(Style::default().fg(border_color))
                .title(" 🚀 CCR - Configuration Switcher ")
                .title_alignment(Alignment::Center)
                .title_style(theme::title_style()),
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

/// Render profile list with platform-aware accent color
fn render_profile_list(f: &mut Frame, app: &App, area: Rect) {
    render_profile_list_panel(f, app, area);
}

fn render_profile_list_panel(f: &mut Frame, app: &App, area: Rect) {
    let profiles = app.current_page_profiles();
    let all_profiles = app.current_profiles();
    let platform = app.current_platform();
    let platform_name = platform.display_name();
    let accent = theme::platform_color_for(platform);

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
    let selected_style = Style::default()
        .fg(theme::BG_PRIMARY)
        .bg(accent)
        .add_modifier(Modifier::BOLD);

    let items: Vec<ListItem> = profiles
        .iter()
        .enumerate()
        .map(|(i, profile)| {
            let is_selected = i == app.selected_index;
            let selector = if is_selected { "▶ " } else { "  " };
            let current_marker = if profile.is_current { "●" } else { "○" };
            let name = &profile.name;
            let desc = profile.description.as_deref().unwrap_or("").trim();
            let current_tag = if profile.is_current { " ✓" } else { "" };
            let name_raw = format!("{}{} {}{}", selector, current_marker, name, current_tag);
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

            ListItem::new(Line::from(line_spans))
        })
        .collect();

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

fn render_profile_details(f: &mut Frame, app: &App, area: Rect) {
    let platform = app.current_platform();
    let accent = theme::platform_color_for(platform);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(symbols::border::ROUNDED)
        .border_style(Style::default().fg(accent))
        .title(" Details ")
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

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}

fn generic_profile_detail_lines(
    name: &str,
    config: &ProfileConfig,
    is_current: bool,
) -> Vec<Line<'static>> {
    vec![
        detail_line("name", name.to_string()),
        detail_line("current", yes_no(is_current)),
        detail_line("description", opt_text(config.description.as_deref())),
        detail_line("base_url", opt_text(config.base_url.as_deref())),
        detail_line("model", opt_text(config.model.as_deref())),
        detail_line("account", opt_text(config.account.as_deref())),
        detail_line("enabled", yes_no(config.is_enabled())),
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
        detail_line("name", name.to_string()),
        detail_line("current", yes_no(is_current)),
        detail_line("description", opt_text(config.description.as_deref())),
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
        detail_line("base_url", opt_text(config.base_url.as_deref())),
        detail_line("model", opt_text(config.model.as_deref())),
        detail_line("small_fast", opt_text(config.small_fast_model.as_deref())),
        detail_line("account", opt_text(config.account.as_deref())),
        detail_line("enabled", yes_no(config.is_enabled())),
        detail_line("usage_count", config.usage_count().to_string()),
        detail_line("tags", tags_text(config)),
        detail_line("token", token_state),
    ]
}

fn detail_line(label: &str, value: String) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("{label:<16}"),
            Style::default()
                .fg(theme::FG_SECONDARY)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(value, Style::default().fg(theme::FG_PRIMARY)),
    ])
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
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Length(2)])
        .split(area);

    render_shortcuts(f, app, chunks[0]);
    render_toast(f, app, chunks[1]);
}

/// Render keyboard shortcuts bar (Claude tab only)
fn render_shortcuts(f: &mut Frame, app: &App, area: Rect) {
    let sep = Span::styled(" │ ", Style::default().fg(theme::BORDER));

    let page_hint = if app.total_pages() > 1 {
        vec![
            Span::styled("←→", theme::shortcut_key_style()),
            Span::styled(" Prev/Next page", theme::shortcut_desc_style()),
            sep.clone(),
        ]
    } else {
        vec![]
    };

    let mut shortcuts_spans = vec![
        Span::styled("Tab", theme::shortcut_key_style()),
        Span::styled(" Switch", theme::shortcut_desc_style()),
        sep.clone(),
    ];

    shortcuts_spans.extend(page_hint);

    shortcuts_spans.extend(vec![
        Span::styled("↑↓", theme::shortcut_key_style()),
        Span::styled("/", Style::default().fg(theme::FG_MUTED)),
        Span::styled("jk", theme::shortcut_key_style()),
        Span::styled(" Select", theme::shortcut_desc_style()),
        sep.clone(),
        Span::styled("Enter", theme::shortcut_key_style()),
        Span::styled(" Apply", theme::shortcut_desc_style()),
        sep.clone(),
        Span::styled("r", theme::shortcut_key_style()),
        Span::styled(" Reload", theme::shortcut_desc_style()),
        sep.clone(),
        Span::styled("🖱️", theme::shortcut_key_style()),
        Span::styled(" Mouse", theme::shortcut_desc_style()),
        sep,
        Span::styled("q", theme::shortcut_key_style()),
        Span::styled(" Quit", theme::shortcut_desc_style()),
    ]);

    let shortcuts = Line::from(shortcuts_spans);

    let block = Block::default()
        .borders(Borders::TOP)
        .border_set(symbols::border::ROUNDED)
        .border_style(Style::default().fg(theme::BORDER))
        .title(" Keys ")
        .title_alignment(Alignment::Center)
        .title_style(
            Style::default()
                .fg(theme::FG_MUTED)
                .add_modifier(Modifier::ITALIC),
        );

    let shortcuts_paragraph = Paragraph::new(shortcuts)
        .block(block)
        .alignment(Alignment::Center);

    f.render_widget(shortcuts_paragraph, area);
}

/// Render toast notification (replaces old status_message)
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
