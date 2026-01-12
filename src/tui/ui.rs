// TUI UI 渲染模块
// 负责渲染双Tab配置选择器界面

use super::app::{App, TabState};
use super::theme;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Padding, Paragraph, Tabs},
};

// ═══════════════════════════════════════════════════════════
// 🎨 主渲染入口
// ═══════════════════════════════════════════════════════════

/// 渲染主 UI
pub fn draw(f: &mut Frame, app: &App) {
    // 统一背景色
    let background = Block::default().style(theme::background_style());
    f.render_widget(background, f.area());

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header with tabs
            Constraint::Min(0),    // Profile list
            Constraint::Length(5), // Footer: shortcuts + status
        ])
        .split(f.area());

    render_header(f, app, chunks[0]);
    render_profile_list(f, app, chunks[1]);
    render_footer(f, app, chunks[2]);
}

// ═══════════════════════════════════════════════════════════
// 🏷️ 标题栏和 Tab 渲染
// ═══════════════════════════════════════════════════════════

/// 渲染标题栏和Tab
fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let current_tab = app.current_tab;

    // 构建 Tab 标题，带平台图标和状态指示
    let claude_style = if current_tab == TabState::Claude {
        theme::claude_style()
    } else {
        theme::tab_normal_style()
    };

    let codex_style = if current_tab == TabState::Codex {
        theme::codex_style()
    } else {
        theme::tab_normal_style()
    };

    // Tab 选中指示器
    let claude_indicator = if current_tab == TabState::Claude {
        "▸ "
    } else {
        "  "
    };
    let codex_indicator = if current_tab == TabState::Codex {
        "▸ "
    } else {
        "  "
    };

    let tab_titles: Vec<Line> = vec![
        Line::from(vec![
            Span::styled(claude_indicator, claude_style),
            Span::raw("🤖 "),
            Span::styled("Claude Code", claude_style),
        ]),
        Line::from(vec![
            Span::styled(codex_indicator, codex_style),
            Span::raw("💻 "),
            Span::styled("Codex CLI", codex_style),
        ]),
    ];

    let index = match current_tab {
        TabState::Claude => 0,
        TabState::Codex => 1,
    };

    // 根据当前 Tab 设置边框颜色
    let border_color = match current_tab {
        TabState::Claude => theme::CLAUDE_PRIMARY,
        TabState::Codex => theme::CODEX_PRIMARY,
    };

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
        .select(index)
        .style(theme::tab_normal_style())
        .highlight_style(theme::tab_highlight_style())
        .divider(Span::styled("  │  ", Style::default().fg(theme::BORDER)));

    f.render_widget(tabs, area);
}

// ═══════════════════════════════════════════════════════════
// 📋 配置列表渲染
// ═══════════════════════════════════════════════════════════

/// 渲染配置列表
fn render_profile_list(f: &mut Frame, app: &App, area: Rect) {
    let profiles = app.current_page_profiles();
    let all_profiles = app.current_profiles();
    let platform_name = app.current_tab.title();
    let current_tab = app.current_tab;

    // 根据当前 Tab 设置边框颜色
    let border_color = match current_tab {
        TabState::Claude => theme::CLAUDE_PRIMARY,
        TabState::Codex => theme::CODEX_PRIMARY,
    };

    // 标题显示配置数量和分页信息
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
        .border_style(Style::default().fg(border_color))
        .title(title)
        .title_alignment(Alignment::Left)
        .title_style(theme::platform_style(platform_name))
        .padding(Padding::horizontal(1));

    if profiles.is_empty() {
        render_empty_state(f, app, area, block);
        return;
    }

    // 构建列表项
    let items: Vec<ListItem> = profiles
        .iter()
        .enumerate()
        .map(|(i, profile)| {
            let is_selected = i == app.selected_index;

            // 构建选中指示器 (箭头)
            let selector = if is_selected { "▶ " } else { "  " };

            // 当前激活标记 (实心/空心圆)
            let current_marker = if profile.is_current { "●" } else { "○" };

            // 配置名称
            let name = &profile.name;

            // 描述信息 (截断过长的描述)
            let desc = profile
                .description
                .as_ref()
                .map(|d| {
                    let truncated = if d.len() > 40 {
                        format!("{}...", &d[..37])
                    } else {
                        d.clone()
                    };
                    format!("  ─  {}", truncated)
                })
                .unwrap_or_default();

            // 当前标签
            let current_tag = if profile.is_current { " ✓" } else { "" };

            // 组合内容
            let content = format!(
                "{}{} {}{}{}",
                selector, current_marker, name, desc, current_tag
            );

            // 计算样式
            let style = if is_selected {
                theme::list_selected_style()
            } else if profile.is_current {
                theme::list_current_style()
            } else {
                theme::list_normal_style()
            };

            ListItem::new(Line::from(Span::styled(content, style)))
        })
        .collect();

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

/// 渲染空状态
fn render_empty_state(f: &mut Frame, app: &App, area: Rect, block: Block) {
    let platform_name = app.current_tab.title();
    let platform = app.current_tab.platform();
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
// 📝 底部状态栏渲染
// ═══════════════════════════════════════════════════════════

/// 渲染底部快捷键和状态
fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Length(2)])
        .split(area);

    // 快捷键提示
    render_shortcuts(f, app, chunks[0]);

    // 状态消息 (显示在最下面)
    render_status_message(f, app, chunks[1]);
}

/// 渲染快捷键提示
fn render_shortcuts(f: &mut Frame, app: &App, area: Rect) {
    let sep = Span::styled(" │ ", Style::default().fg(theme::BORDER));

    // 根据是否有多页显示不同的翻页提示
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

/// 渲染状态消息 (显示在最底部)
fn render_status_message(f: &mut Frame, app: &App, area: Rect) {
    if let Some((message, is_error)) = &app.status_message {
        let style = if *is_error {
            theme::error_style()
        } else {
            theme::success_style()
        };

        let status =
            Paragraph::new(Span::styled(message.as_str(), style)).alignment(Alignment::Center);

        f.render_widget(status, area);
    }
}
