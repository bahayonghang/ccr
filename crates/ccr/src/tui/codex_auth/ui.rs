// 🎨 Codex Auth TUI UI rendering
// Draws the Codex multi-account management interface

use super::app::{CodexAuthApp, QuotaState, UsageState};
use crate::services::{CodexQuotaService, CodexUsageService};
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
            Constraint::Length(14), // Usage panel (enlarged for quota)
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
        crate::models::LoginState::ApiKeyActive => "API Key 模式".to_string(),
        crate::models::LoginState::ProviderKeyActive { env_key } => {
            format!("Provider Key: {}", env_key)
        }
        crate::models::LoginState::Unknown { type_name, .. } => format!("未知状态: {}", type_name),
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
    // 🖱️ 缓存列表区域供鼠标点击使用
    app.list_area.set(Some(area));
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

/// Draw usage panel (quota + local stats)
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

    let mut content: Vec<Line> = Vec::new();

    // ── 配额余额区域 ──
    match &app.quota_state {
        QuotaState::Idle => {
            content.push(Line::from(Span::styled(
                "  按 b 查询配额余额",
                Style::default().fg(Color::DarkGray),
            )));
        }
        QuotaState::Loading => {
            content.push(Line::from(Span::styled(
                "  ⏳ 正在查询配额...",
                Style::default().fg(Color::Yellow),
            )));
        }
        QuotaState::Error(err) => {
            content.push(Line::from(Span::styled(
                format!("  ⚠️ 配额查询失败: {}", err),
                Style::default().fg(Color::Red),
            )));
        }
        QuotaState::Loaded(quotas) => {
            // 查找当前选中账号的配额
            let selected_name = app
                .selected_account()
                .map(|a| a.name.as_str())
                .unwrap_or("");

            if let Some(aq) = quotas.iter().find(|q| q.account_name == selected_name) {
                if let Some(ref quota) = aq.quota {
                    let account_label = aq.email.as_deref().unwrap_or(&aq.account_name);
                    content.push(Line::from(vec![
                        Span::styled("  配额 ", Style::default().fg(Color::Cyan)),
                        Span::styled(
                            format!("({})", account_label),
                            Style::default().fg(Color::DarkGray),
                        ),
                    ]));

                    // 5h 限额
                    let h_color = percent_color(quota.hourly_percentage);
                    let h_bar = progress_bar(quota.hourly_percentage, 10);
                    let h_reset = quota
                        .hourly_reset_time
                        .map(|t| format!("  重置: {}", CodexQuotaService::format_reset_duration(t)))
                        .unwrap_or_default();
                    content.push(Line::from(vec![
                        Span::styled("  5h限额: ", Style::default().fg(Color::White)),
                        Span::styled(h_bar, Style::default().fg(h_color)),
                        Span::styled(
                            format!(" {}%", quota.hourly_percentage),
                            Style::default().fg(h_color),
                        ),
                        Span::styled(h_reset, Style::default().fg(Color::DarkGray)),
                    ]));

                    // 周限额
                    let w_color = percent_color(quota.weekly_percentage);
                    let w_bar = progress_bar(quota.weekly_percentage, 10);
                    let w_reset = quota
                        .weekly_reset_time
                        .map(|t| format!("  重置: {}", CodexQuotaService::format_reset_duration(t)))
                        .unwrap_or_default();
                    content.push(Line::from(vec![
                        Span::styled("  周限额: ", Style::default().fg(Color::White)),
                        Span::styled(w_bar, Style::default().fg(w_color)),
                        Span::styled(
                            format!(" {}%", quota.weekly_percentage),
                            Style::default().fg(w_color),
                        ),
                        Span::styled(w_reset, Style::default().fg(Color::DarkGray)),
                    ]));

                    // 订阅类型
                    if let Some(ref plan) = quota.plan_type {
                        content.push(Line::from(vec![
                            Span::styled("  订阅: ", Style::default().fg(Color::White)),
                            Span::styled(plan.clone(), Style::default().fg(Color::Magenta)),
                        ]));
                    }
                } else if let Some(ref err) = aq.error {
                    content.push(Line::from(Span::styled(
                        format!("  ⚠️ {}: {}", aq.account_name, err),
                        Style::default().fg(Color::Red),
                    )));
                }
            } else if !quotas.is_empty() {
                // 显示第一个有配额的账号
                content.push(Line::from(Span::styled(
                    "  选中账号无配额数据，按 b 刷新",
                    Style::default().fg(Color::DarkGray),
                )));
            }
        }
    }

    // ── 分隔线 ──
    content.push(Line::from(Span::styled(
        "  ────────────────────────────────",
        Style::default().fg(Color::DarkGray),
    )));

    // ── 本地统计区域 ──
    match &app.usage_state {
        UsageState::NoData => {
            content.push(Line::from(Span::styled(
                "  📭 暂无本地使用数据",
                Style::default().fg(Color::DarkGray),
            )));
        }
        UsageState::Error(err) => {
            content.push(Line::from(Span::styled(
                format!("  ⚠️ 统计加载失败: {}", err),
                Style::default().fg(Color::Red),
            )));
        }
        UsageState::Loaded(usage) => {
            let five_total =
                usage.five_hour.total_input_tokens + usage.five_hour.total_output_tokens;
            let seven_total =
                usage.seven_day.total_input_tokens + usage.seven_day.total_output_tokens;

            content.push(Line::from(vec![
                Span::styled("  5小时: ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    format!(
                        "{} tokens ({} 请求)",
                        CodexUsageService::format_tokens(five_total),
                        usage.five_hour.total_requests
                    ),
                    Style::default().fg(Color::White),
                ),
            ]));
            content.push(Line::from(vec![
                Span::styled("  7天:   ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    format!(
                        "{} tokens ({} 请求)",
                        CodexUsageService::format_tokens(seven_total),
                        usage.seven_day.total_requests
                    ),
                    Style::default().fg(Color::White),
                ),
            ]));
        }
        UsageState::Loading => {
            content.push(Line::from(Span::styled(
                "  ⏳ 加载中...",
                Style::default().fg(Color::DarkGray),
            )));
        }
    }

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

/// 百分比颜色：>=60% Green, 30-59% Yellow, <30% Red
fn percent_color(pct: i32) -> Color {
    if pct >= 60 {
        Color::Green
    } else if pct >= 30 {
        Color::Yellow
    } else {
        Color::Red
    }
}

/// 生成文本进度条
fn progress_bar(pct: i32, width: usize) -> String {
    let filled = ((pct as usize) * width / 100).min(width);
    let empty = width - filled;
    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}

/// Draw help bar (overlay-aware)
fn draw_help_bar(f: &mut Frame, area: Rect, app: &CodexAuthApp) {
    let help_text = match &app.overlay {
        Some(Overlay::Confirm { .. }) => "y 确认删除 | n/Esc 取消",
        Some(Overlay::Input { .. }) => "Enter 确认 | Esc 取消",
        None => "↑/k 上移 | ↓/j 下移 | Enter 切换 | s 保存当前 | d 删除 | r 刷新 | b 配额 | q 退出",
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
            Constraint::Length(14), // Usage panel (enlarged for quota)
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
    // 🖱️ 缓存列表区域供鼠标点击使用
    app.list_area.set(Some(area));
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
        crate::models::LoginState::ApiKeyActive => "API Key 模式".to_string(),
        crate::models::LoginState::ProviderKeyActive { env_key } => {
            format!("Provider Key: {}", env_key)
        }
        crate::models::LoginState::Unknown { type_name, .. } => format!("未知状态: {}", type_name),
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
            "Tab 切换 | ↑/k 上移 | ↓/j 下移 | Enter 切换 | s 保存当前 | d 删除 | r 刷新 | b 配额 | q 退出"
        }
    };

    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);

    f.render_widget(help, area);
}
