// TUI UI rendering module
// Renders dynamic multi-platform profile switcher interface

use super::app::App;
use super::codex_auth;
use super::theme;
use super::toast::ToastKind;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Padding, Paragraph, Tabs},
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

    if app.is_codex_tab() {
        // Codex tab: delegate content + footer to codex_auth embedded renderer
        if let Some(ref codex_app) = app.codex_auth_app {
            codex_auth::ui::draw_embedded(f, codex_app, chunks[1], chunks[2], compact);
        }
    } else {
        // Claude tab: profile list + footer
        render_profile_list(f, app, chunks[1]);
        if compact {
            render_toast(f, app, chunks[2]);
        } else {
            render_footer(f, app, chunks[2]);
        }
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
                Span::styled(tab.platform.display_name(), style),
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

    // Title with profile count and pagination
    let total_pages = app.total_pages();
    let title = if all_profiles.is_empty() {
        format!(" {} Profiles ", platform_name)
    } else if total_pages > 1 {
        format!(
            " {} Profiles ({})  Page {}/{} ",
            platform_name,
            all_profiles.len(),
            app.current_page + 1,
            total_pages
        )
    } else {
        format!(" {} Profiles ({}) ", platform_name, all_profiles.len())
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

    // Platform-aware selection highlight
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
            let desc = profile.description.as_deref().unwrap_or("");
            let current_tag = if profile.is_current { " ✓" } else { "" };

            let name_raw = format!("{}{} {}{}", selector, current_marker, name, current_tag);
            let name_cell = pad_text(&truncate_text(&name_raw, name_width), name_width);

            let name_style = if is_selected {
                selected_style
            } else if profile.is_current {
                theme::list_current_style()
            } else {
                theme::list_normal_style()
            };

            // Responsive: hide description column when narrow
            let line_spans = if desc_width > 0 {
                let desc_cell = pad_text(&truncate_text(desc, desc_width), desc_width);
                let desc_style = if is_selected {
                    selected_style
                } else if profile.is_current {
                    theme::list_current_style()
                } else {
                    Style::default().fg(theme::FG_MUTED)
                };
                vec![
                    Span::styled(name_cell, name_style),
                    Span::raw("  "),
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

/// Render empty state for current platform
fn render_empty_state(f: &mut Frame, app: &App, area: Rect, block: Block) {
    let platform = app.current_platform();
    let platform_name = platform.display_name();
    let short_name = platform.short_name();

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
            Span::styled(" Page", theme::shortcut_desc_style()),
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
