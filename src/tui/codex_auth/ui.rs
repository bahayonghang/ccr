// 🎨 Codex Auth TUI 界面渲染
// 绘制 Codex 多账号管理界面

use super::app::{CodexAuthApp, Mode, UsageState};
use crate::models::TokenFreshness;
use crate::services::CodexUsageService;
use crate::tui::theme;
use chrono::{Local, Utc};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};

/// 🎨 绘制主界面
pub fn draw(f: &mut Frame, app: &CodexAuthApp) {
    // 统一背景色
    let background = Block::default().style(theme::background_style());
    f.render_widget(background, f.area());

    // 主布局
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),  // 标题
            Constraint::Min(8),     // 账号列表
            Constraint::Length(10), // 使用情况面板
            Constraint::Length(3),  // 状态栏
            Constraint::Length(2),  // 帮助栏
        ])
        .split(f.area());

    // 绘制标题
    draw_title(f, chunks[0], app);

    // 绘制账号列表
    draw_account_list(f, chunks[1], app);

    // 绘制使用情况面板
    draw_usage_panel(f, chunks[2], app);

    // 绘制状态栏
    draw_status_bar(f, chunks[3], app);

    // 绘制帮助栏
    draw_help_bar(f, chunks[4], app);

    // 绘制弹窗
    match app.mode {
        Mode::ConfirmDelete => draw_confirm_delete_popup(f, app),
        Mode::InputSaveName => draw_input_save_popup(f, app),
        Mode::Normal => {}
    }
}

/// 绘制标题
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

/// 绘制账号列表
fn draw_account_list(f: &mut Frame, area: Rect, app: &CodexAuthApp) {
    let accounts = app.current_page_accounts();

    let items: Vec<ListItem> = accounts
        .iter()
        .enumerate()
        .map(|(i, account)| {
            let is_selected = i == app.selected_index;

            // 状态标记
            let status = if account.is_current { "▶ " } else { "  " };

            // 名称
            let name = if account.is_virtual {
                format!("{} *", account.name)
            } else {
                account.name.clone()
            };

            // 邮箱
            let email = account.email.as_deref().unwrap_or("-");

            // 新鲜度
            let freshness = CodexAuthApp::freshness_text(account.freshness);

            // 到期
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

            // 描述
            let desc = account.description.as_deref().unwrap_or("");

            // 构建行
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
                    format!("{:<10}", freshness),
                    Style::default().fg(match account.freshness {
                        TokenFreshness::Fresh => Color::Green,
                        TokenFreshness::Stale => Color::Yellow,
                        TokenFreshness::Old => Color::Red,
                        TokenFreshness::Unknown => Color::DarkGray,
                    }),
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

    // 页码信息
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

/// 绘制状态栏
fn draw_status_bar(f: &mut Frame, area: Rect, app: &CodexAuthApp) {
    let (message, is_error) = app
        .status_message
        .as_ref()
        .map(|(m, e)| (m.as_str(), *e))
        .unwrap_or(("就绪", false));

    let style = if is_error {
        Style::default().fg(Color::Red)
    } else {
        Style::default().fg(Color::Green)
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

/// 绘制使用情况面板
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
                // 5小时窗口
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
                // 7天窗口
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
                // 提示信息
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

/// 绘制帮助栏
fn draw_help_bar(f: &mut Frame, area: Rect, app: &CodexAuthApp) {
    let help_text = match app.mode {
        Mode::Normal => "↑/k 上移 | ↓/j 下移 | Enter 切换 | s 保存当前 | d 删除 | r 刷新 | q 退出",
        Mode::ConfirmDelete => "y 确认删除 | n/Esc 取消",
        Mode::InputSaveName => "Enter 确认 | Esc 取消",
    };

    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);

    f.render_widget(help, area);
}

/// 绘制确认删除弹窗
fn draw_confirm_delete_popup(f: &mut Frame, app: &CodexAuthApp) {
    let area = centered_rect(50, 30, f.area());

    // 清除背景
    f.render_widget(Clear, area);

    let account_name = app
        .selected_account()
        .map(|a| a.name.as_str())
        .unwrap_or("未知");

    let text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "⚠️ 确认删除",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(format!("即将删除账号: {}", account_name)),
        Line::from(""),
        Line::from("此操作不可撤销！"),
        Line::from(""),
        Line::from(Span::styled(
            "按 y 确认 | 按 n 或 Esc 取消",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let popup = Paragraph::new(text)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow))
                .title(" 确认删除 ")
                .title_style(Style::default().fg(Color::Yellow)),
        );

    f.render_widget(popup, area);
}

/// 绘制输入保存名称弹窗
fn draw_input_save_popup(f: &mut Frame, app: &CodexAuthApp) {
    let area = centered_rect(50, 30, f.area());

    // 清除背景
    f.render_widget(Clear, area);

    let text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "💾 保存当前登录",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("请输入账号名称:"),
        Line::from(""),
        Line::from(Span::styled(
            format!("▶ {}_", app.input_buffer),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "(只能包含字母、数字、下划线和连字符)",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "按 Enter 确认 | 按 Esc 取消",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let popup = Paragraph::new(text)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(" 保存账号 ")
                .title_style(Style::default().fg(Color::Cyan)),
        );

    f.render_widget(popup, area);
}

/// 计算居中矩形
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
