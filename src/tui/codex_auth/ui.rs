// 🎨 Codex Auth TUI UI rendering
// Draws the Codex multi-account management interface

use super::app::{CodexAuthApp, UsageState};
use crate::services::CodexUsageService;
use crate::tui::overlay::{Overlay, render_overlay};
use crate::tui::theme;
use crate::tui::toast::ToastKind;
use chrono::{Local, Utc};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

/// 🎨 Draw main interface
pub fn draw(f: &mut Frame, app: &CodexAuthApp) {
    // Unified background
    let background = Block::default().style(theme::background_style());
    f.render_widget(background, f.area());

    // Main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Min(8),     // Account list
            Constraint::Length(10), // Usage panel
            Constraint::Length(3),  // Status bar
            Constraint::Length(2),  // Help bar
        ])
        .split(f.area());

    draw_title(f, chunks[0], app);
    draw_account_list(f, chunks[1], app);
    draw_usage_panel(f, chunks[2], app);
    draw_status_bar(f, chunks[3], app);
    draw_help_bar(f, chunks[4], app);

    // Draw overlay (with dark backdrop) if active
    if let Some(overlay) = &app.overlay {
        render_overlay(f, overlay);
    }
}

/// Draw title bar
fn draw_title(f: &mut Frame, area: Rect, app: &CodexAuthApp) {
    let login_status = match &app.login_state {
        crate::models::LoginState::NotLoggedIn => "未登录".to_string(),
        crate::models::LoginState::LoggedInUnsaved => "已登录 (未保存)".to_string(),
        crate::models::LoginState::LoggedInSaved(name) => format!("已登录: {}", name),
    };

    let title = Paragraph::new(vec![Line::from(vec![
        Span::styled(
            " 🔐 Codex 账号管理 ",
            Style::default()
                .fg(theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(login_status, Style::default().fg(Color::Cyan)),
    ])])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::BORDER))
            .title(" CCR ")
            .title_style(Style::default().fg(theme::ACCENT)),
    )
    .alignment(Alignment::Center);

    f.render_widget(title, area);
}

/// Draw account list
fn draw_account_list(f: &mut Frame, area: Rect, app: &CodexAuthApp) {
    let accounts = app.current_page_accounts();

    let items: Vec<ListItem> = accounts
        .iter()
        .enumerate()
        .map(|(i, account)| {
            let is_selected = i == app.selected_index;

            let status = if account.is_current { "▶ " } else { "  " };

            let name = if account.is_virtual {
                format!("{} *", account.name)
            } else {
                account.name.clone()
            };

            let email = account.email.as_deref().unwrap_or("-");

            let saved_at = account
                .saved_at
                .map(|ts| ts.with_timezone(&Local).format("%Y-%m-%d").to_string())
                .unwrap_or_else(|| "-".to_string());

            let (expire_text, expire_style) = match account.expires_at {
                Some(ts) => {
                    let expired = ts <= Utc::now();
                    let local_ts = ts.with_timezone(&Local).format("%Y-%m-%d %H:%M");
                    let text = if expired {
                        format!("🔒 {}", local_ts)
                    } else {
                        local_ts.to_string()
                    };
                    let style =
                        Style::default().fg(if expired { Color::Red } else { Color::Green });
                    (text, style)
                }
                None => ("-".to_string(), Style::default().fg(Color::DarkGray)),
            };

            let desc = account.description.as_deref().unwrap_or("");

            let line = Line::from(vec![
                Span::styled(
                    status,
                    Style::default().fg(if account.is_current {
                        Color::Green
                    } else {
                        Color::DarkGray
                    }),
                ),
                Span::styled(
                    format!("{:<16}", name),
                    Style::default()
                        .fg(if account.is_virtual {
                            Color::Yellow
                        } else if account.is_current {
                            Color::Green
                        } else {
                            Color::White
                        })
                        .add_modifier(if is_selected {
                            Modifier::BOLD
                        } else {
                            Modifier::empty()
                        }),
                ),
                Span::raw(" "),
                Span::styled(format!("{:<24}", email), Style::default().fg(Color::Cyan)),
                Span::raw(" "),
                Span::styled(
                    format!("{:<12}", saved_at),
                    Style::default().fg(Color::White),
                ),
                Span::raw(" "),
                Span::styled(format!("{:<18}", expire_text), expire_style),
                Span::raw(" "),
                Span::styled(desc.to_string(), Style::default().fg(Color::DarkGray)),
            ]);

            let style = if is_selected {
                Style::default()
                    .bg(theme::CODEX_PRIMARY)
                    .fg(theme::BG_PRIMARY)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(line).style(style)
        })
        .collect();

    let page_info = format!(
        " 第 {}/{} 页 | 共 {} 个账号 ",
        app.current_page + 1,
        app.total_pages(),
        app.accounts.len()
    );

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::BORDER))
                .title(" 账号列表 ")
                .title_style(Style::default().fg(theme::ACCENT))
                .title_bottom(Line::from(page_info).alignment(Alignment::Right)),
        )
        .highlight_style(
            Style::default()
                .bg(theme::CODEX_PRIMARY)
                .fg(theme::BG_PRIMARY)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(list, area);
}

/// Render status bar with toast notification
fn draw_status_bar(f: &mut Frame, area: Rect, app: &CodexAuthApp) {
    let (message, style) = if let Some(toast) = app.toasts.active() {
        let s = match toast.kind {
            ToastKind::Success => Style::default().fg(Color::Green),
            ToastKind::Error => Style::default().fg(Color::Red),
            ToastKind::Warning => Style::default().fg(Color::Yellow),
            ToastKind::Info => Style::default().fg(Color::Cyan),
        };
        (toast.message.as_str(), s)
    } else {
        ("就绪", Style::default().fg(Color::Green))
    };

    let status = Paragraph::new(message).style(style).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::BORDER))
            .title(" 状态 ")
            .title_style(Style::default().fg(theme::ACCENT)),
    );

    f.render_widget(status, area);
}

/// Draw usage panel
fn draw_usage_panel(f: &mut Frame, area: Rect, app: &CodexAuthApp) {
    let title = Line::from(vec![
        Span::styled(" 📊 ", Style::default().fg(Color::Cyan)),
        Span::styled(
            "Codex 使用情况",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
    ]);

    let content = match &app.usage_state {
        UsageState::NoData => {
            vec![
                Line::from(""),
                Line::from(Span::styled(
                    "📭 暂无使用数据",
                    Style::default().fg(Color::DarkGray),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "使用 Codex 后将会显示使用统计",
                    Style::default().fg(Color::DarkGray),
                )),
            ]
        }
        UsageState::Error(err) => {
            vec![
                Line::from(""),
                Line::from(Span::styled("⚠️ 加载失败", Style::default().fg(Color::Red))),
                Line::from(""),
                Line::from(Span::styled(
                    err.as_str(),
                    Style::default().fg(Color::DarkGray),
                )),
            ]
        }
        UsageState::Loaded(usage) => {
            let five_total =
                usage.five_hour.total_input_tokens + usage.five_hour.total_output_tokens;
            let seven_total =
                usage.seven_day.total_input_tokens + usage.seven_day.total_output_tokens;

            vec![
                Line::from(""),
                // 5-hour window
                Line::from(vec![
                    Span::styled("5小时: ", Style::default().fg(Color::Cyan)),
                    Span::styled(
                        format!(
                            "{} tokens ({} 请求)",
                            CodexUsageService::format_tokens(five_total),
                            usage.five_hour.total_requests
                        ),
                        Style::default().fg(Color::White),
                    ),
                ]),
                Line::from(vec![
                    Span::raw("  "),
                    Span::styled(
                        format!(
                            "{} in / {} out",
                            CodexUsageService::format_tokens(usage.five_hour.total_input_tokens),
                            CodexUsageService::format_tokens(usage.five_hour.total_output_tokens)
                        ),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]),
                Line::from(""),
                // 7-day window
                Line::from(vec![
                    Span::styled("7天:   ", Style::default().fg(Color::Cyan)),
                    Span::styled(
                        format!(
                            "{} tokens ({} 请求)",
                            CodexUsageService::format_tokens(seven_total),
                            usage.seven_day.total_requests
                        ),
                        Style::default().fg(Color::White),
                    ),
                ]),
                Line::from(vec![
                    Span::raw("  "),
                    Span::styled(
                        format!(
                            "{} in / {} out",
                            CodexUsageService::format_tokens(usage.seven_day.total_input_tokens),
                            CodexUsageService::format_tokens(usage.seven_day.total_output_tokens)
                        ),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]),
                Line::from(""),
                Line::from(Span::styled(
                    "💡 在 Codex CLI 中运行 /status 查看官方限制",
                    Style::default().fg(Color::Yellow),
                )),
            ]
        }
        UsageState::Loading => {
            vec![
                Line::from(""),
                Line::from(Span::styled(
                    "⏳ 加载中...",
                    Style::default().fg(Color::DarkGray),
                )),
            ]
        }
    };

    let panel = Paragraph::new(content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::BORDER))
                .title(title)
                .title_style(Style::default().fg(theme::ACCENT)),
        )
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });

    f.render_widget(panel, area);
}

/// Draw help bar (overlay-aware)
fn draw_help_bar(f: &mut Frame, area: Rect, app: &CodexAuthApp) {
    let help_text = match &app.overlay {
        Some(Overlay::Confirm { .. }) => "y 确认删除 | n/Esc 取消",
        Some(Overlay::Input { .. }) => "Enter 确认 | Esc 取消",
        None => "↑/k 上移 | ↓/j 下移 | Enter 切换 | s 保存当前 | d 删除 | r 刷新 | q 退出",
    };

    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);

    f.render_widget(help, area);
}

// ═══════════════════════════════════════════════════════════
// Embedded rendering (used when Codex tab is active in main TUI)
// ═══════════════════════════════════════════════════════════

/// Draw Codex Auth UI embedded within the main TUI layout.
///
/// `content_area` is the middle section (profile list area in Claude tab).
/// `footer_area` is the bottom section (shortcuts + toast in Claude tab).
pub fn draw_embedded(
    f: &mut Frame,
    app: &CodexAuthApp,
    content_area: Rect,
    footer_area: Rect,
    compact: bool,
) {
    // Content area: account list + usage panel
    let content_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(8),     // Account list (with login status in title)
            Constraint::Length(10), // Usage panel
        ])
        .split(content_area);

    draw_account_list_with_status(f, content_chunks[0], app);
    draw_usage_panel(f, content_chunks[1], app);

    // Footer area: status bar + help bar (or just help in compact)
    if compact {
        draw_help_bar_embedded(f, footer_area, app);
    } else {
        let footer_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Status bar
                Constraint::Length(2), // Help bar
            ])
            .split(footer_area);

        draw_status_bar(f, footer_chunks[0], app);
        draw_help_bar_embedded(f, footer_chunks[1], app);
    }

    // Draw overlay (with dark backdrop) if active
    if let Some(overlay) = &app.overlay {
        render_overlay(f, overlay);
    }
}

/// Draw account list with login status merged into the title
fn draw_account_list_with_status(f: &mut Frame, area: Rect, app: &CodexAuthApp) {
    let accounts = app.current_page_accounts();

    let items: Vec<ListItem> = accounts
        .iter()
        .enumerate()
        .map(|(i, account)| {
            let is_selected = i == app.selected_index;

            let status = if account.is_current { "▶ " } else { "  " };

            let name = if account.is_virtual {
                format!("{} *", account.name)
            } else {
                account.name.clone()
            };

            let email = account.email.as_deref().unwrap_or("-");

            let saved_at = account
                .saved_at
                .map(|ts| ts.with_timezone(&Local).format("%Y-%m-%d").to_string())
                .unwrap_or_else(|| "-".to_string());

            let (expire_text, expire_style) = match account.expires_at {
                Some(ts) => {
                    let expired = ts <= Utc::now();
                    let local_ts = ts.with_timezone(&Local).format("%Y-%m-%d %H:%M");
                    let text = if expired {
                        format!("🔒 {}", local_ts)
                    } else {
                        local_ts.to_string()
                    };
                    let style =
                        Style::default().fg(if expired { Color::Red } else { Color::Green });
                    (text, style)
                }
                None => ("-".to_string(), Style::default().fg(Color::DarkGray)),
            };

            let desc = account.description.as_deref().unwrap_or("");

            let line = Line::from(vec![
                Span::styled(
                    status,
                    Style::default().fg(if account.is_current {
                        Color::Green
                    } else {
                        Color::DarkGray
                    }),
                ),
                Span::styled(
                    format!("{:<16}", name),
                    Style::default()
                        .fg(if account.is_virtual {
                            Color::Yellow
                        } else if account.is_current {
                            Color::Green
                        } else {
                            Color::White
                        })
                        .add_modifier(if is_selected {
                            Modifier::BOLD
                        } else {
                            Modifier::empty()
                        }),
                ),
                Span::raw(" "),
                Span::styled(format!("{:<24}", email), Style::default().fg(Color::Cyan)),
                Span::raw(" "),
                Span::styled(
                    format!("{:<12}", saved_at),
                    Style::default().fg(Color::White),
                ),
                Span::raw(" "),
                Span::styled(format!("{:<18}", expire_text), expire_style),
                Span::raw(" "),
                Span::styled(desc.to_string(), Style::default().fg(Color::DarkGray)),
            ]);

            let style = if is_selected {
                Style::default()
                    .bg(theme::CODEX_PRIMARY)
                    .fg(theme::BG_PRIMARY)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(line).style(style)
        })
        .collect();

    // Merge login status into the list title
    let login_status = match &app.login_state {
        crate::models::LoginState::NotLoggedIn => "未登录".to_string(),
        crate::models::LoginState::LoggedInUnsaved => "已登录 (未保存)".to_string(),
        crate::models::LoginState::LoggedInSaved(name) => format!("已登录: {}", name),
    };

    let title = format!(" 🔐 账号列表 | {} ", login_status);

    let page_info = format!(
        " 第 {}/{} 页 | 共 {} 个账号 ",
        app.current_page + 1,
        app.total_pages(),
        app.accounts.len()
    );

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::BORDER))
                .title(title)
                .title_style(Style::default().fg(theme::ACCENT))
                .title_bottom(Line::from(page_info).alignment(Alignment::Right)),
        )
        .highlight_style(
            Style::default()
                .bg(theme::CODEX_PRIMARY)
                .fg(theme::BG_PRIMARY)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(list, area);
}

/// Draw help bar with Tab switch hint (embedded mode)
fn draw_help_bar_embedded(f: &mut Frame, area: Rect, app: &CodexAuthApp) {
    let help_text = match &app.overlay {
        Some(Overlay::Confirm { .. }) => "y 确认删除 | n/Esc 取消",
        Some(Overlay::Input { .. }) => "Enter 确认 | Esc 取消",
        None => {
            "Tab 切换 | ↑/k 上移 | ↓/j 下移 | Enter 切换 | s 保存当前 | d 删除 | r 刷新 | q 退出"
        }
    };

    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);

    f.render_widget(help, area);
}
