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
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap},
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
            Constraint::Length(12), // Usage panel
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

fn draw_account_list(f: &mut Frame, area: Rect, app: &CodexAuthApp) {
    render_account_list_panel(f, area, app, " 账号列表 ".to_string());
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AccountColumn {
    Status,
    Account,
    Email,
    SavedAt,
    ExpiresAt,
    Description,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AccountTableLayout {
    columns: Vec<AccountColumn>,
    widths: Vec<Constraint>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AccountListRegions {
    header: Rect,
    body: Rect,
}

fn render_account_list_panel(f: &mut Frame, area: Rect, app: &CodexAuthApp, title: String) {
    let page_info = format!(
        " 第 {}/{} 页 | 共 {} 个账号 ",
        app.current_page + 1,
        app.total_pages(),
        app.accounts.len()
    );

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::BORDER))
        .title(title)
        .title_style(Style::default().fg(theme::ACCENT))
        .title_bottom(Line::from(page_info).alignment(Alignment::Right));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if app.accounts.is_empty() {
        app.list_area.set(None);
        let empty = Paragraph::new(" 未检测到可切换的 Codex 账号")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Left);
        f.render_widget(empty, inner);
        return;
    }

    if inner.height < 2 {
        app.list_area.set(Some(inner));
        return;
    }

    let regions = account_list_regions(inner);
    let layout = account_table_layout(regions.header.width);

    app.list_area.set(Some(regions.body));
    render_account_list_header(f, regions.header, &layout);
    render_account_list_rows(f, regions.body, app, &layout);
}

fn account_list_regions(inner: Rect) -> AccountListRegions {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(inner);

    AccountListRegions {
        header: chunks[0],
        body: chunks[1],
    }
}

fn account_table_layout(inner_width: u16) -> AccountTableLayout {
    if inner_width < 64 {
        return AccountTableLayout {
            columns: vec![
                AccountColumn::Status,
                AccountColumn::Account,
                AccountColumn::Email,
            ],
            widths: vec![
                Constraint::Length(3),
                Constraint::Length(18),
                Constraint::Min(8),
            ],
        };
    }

    if inner_width < 92 {
        return AccountTableLayout {
            columns: vec![
                AccountColumn::Status,
                AccountColumn::Account,
                AccountColumn::Email,
                AccountColumn::SavedAt,
                AccountColumn::ExpiresAt,
            ],
            widths: vec![
                Constraint::Length(3),
                Constraint::Length(18),
                Constraint::Min(12),
                Constraint::Length(10),
                Constraint::Length(10),
            ],
        };
    }

    AccountTableLayout {
        columns: vec![
            AccountColumn::Status,
            AccountColumn::Account,
            AccountColumn::Email,
            AccountColumn::SavedAt,
            AccountColumn::ExpiresAt,
            AccountColumn::Description,
        ],
        widths: vec![
            Constraint::Length(3),
            Constraint::Length(18),
            Constraint::Length(24),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Min(10),
        ],
    }
}

fn render_account_list_header(f: &mut Frame, area: Rect, layout: &AccountTableLayout) {
    let header_cells = layout.columns.iter().map(account_header_cell);
    let header = Table::new([Row::new(header_cells)], layout.widths.clone())
        .column_spacing(1)
        .style(
            Style::default()
                .fg(theme::FG_SECONDARY)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(header, area);
}

fn render_account_list_rows(
    f: &mut Frame,
    area: Rect,
    app: &CodexAuthApp,
    layout: &AccountTableLayout,
) {
    let selected_style = Style::default()
        .bg(theme::CODEX_PRIMARY)
        .fg(theme::BG_PRIMARY)
        .add_modifier(Modifier::BOLD);

    let rows = app
        .current_page_accounts()
        .iter()
        .enumerate()
        .map(|(idx, account)| {
            let row_style = if idx == app.selected_index {
                selected_style
            } else {
                Style::default()
            };

            Row::new(
                layout
                    .columns
                    .iter()
                    .map(|column| account_cell(account, *column)),
            )
            .style(row_style)
            .height(1)
        });

    let table = Table::new(rows, layout.widths.clone()).column_spacing(1);
    f.render_widget(table, area);
}

fn account_header_cell(column: &AccountColumn) -> Cell<'static> {
    let label = match column {
        AccountColumn::Status => "状态",
        AccountColumn::Account => "账号",
        AccountColumn::Email => "邮箱",
        AccountColumn::SavedAt => "保存",
        AccountColumn::ExpiresAt => "到期",
        AccountColumn::Description => "备注",
    };

    Cell::from(label.to_string())
}

fn account_cell(account: &crate::models::CodexAuthItem, column: AccountColumn) -> Cell<'static> {
    match column {
        AccountColumn::Status => {
            let marker = if account.is_current { "▶" } else { " " };
            let style = if account.is_current {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            Cell::from(Line::from(Span::styled(marker, style)))
        }
        AccountColumn::Account => {
            let freshness = Span::styled(
                account.freshness.icon().to_string(),
                Style::default().fg(freshness_color(&account.freshness)),
            );
            let name_style = if account.is_virtual {
                Style::default().fg(Color::Yellow)
            } else if account.is_current {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let mut spans = vec![
                freshness,
                Span::raw(" "),
                Span::styled(account.name.clone(), name_style),
            ];

            if account.is_virtual {
                spans.push(Span::styled(
                    " *",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::ITALIC),
                ));
            }

            Cell::from(Line::from(spans))
        }
        AccountColumn::Email => {
            let email = account.email.as_deref().unwrap_or("-");
            Cell::from(Line::from(Span::styled(
                email.to_string(),
                Style::default().fg(Color::Cyan),
            )))
        }
        AccountColumn::SavedAt => Cell::from(Line::from(Span::styled(
            format_saved_at(account),
            Style::default().fg(Color::White),
        ))),
        AccountColumn::ExpiresAt => {
            let (text, style) = format_expires_at(account);
            Cell::from(Line::from(Span::styled(text, style)))
        }
        AccountColumn::Description => {
            let description = account.description.as_deref().unwrap_or("-");
            Cell::from(Line::from(Span::styled(
                description.to_string(),
                Style::default().fg(Color::DarkGray),
            )))
        }
    }
}

fn login_status_text(app: &CodexAuthApp) -> String {
    match &app.login_state {
        crate::models::LoginState::NotLoggedIn => "未登录".to_string(),
        crate::models::LoginState::LoggedInUnsaved => "已登录 (未保存)".to_string(),
        crate::models::LoginState::LoggedInSaved(name) => format!("已登录: {}", name),
        crate::models::LoginState::ApiKeyActive => "API Key 模式".to_string(),
        crate::models::LoginState::ProviderKeyActive { env_key } => {
            format!("Provider Key: {}", env_key)
        }
        crate::models::LoginState::Unknown { type_name, .. } => format!("未知状态: {}", type_name),
    }
}

fn format_saved_at(account: &crate::models::CodexAuthItem) -> String {
    account
        .saved_at
        .map(|ts| ts.with_timezone(&Local).format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "-".to_string())
}

fn format_expires_at(account: &crate::models::CodexAuthItem) -> (String, Style) {
    match account.expires_at {
        Some(ts) => {
            let expired = ts <= Utc::now();
            let text = ts.with_timezone(&Local).format("%Y-%m-%d").to_string();
            let style = Style::default().fg(if expired { Color::Red } else { Color::Green });
            (text, style)
        }
        None => ("-".to_string(), Style::default().fg(Color::DarkGray)),
    }
}

fn freshness_color(freshness: &crate::models::TokenFreshness) -> Color {
    match freshness {
        crate::models::TokenFreshness::Fresh => Color::Green,
        crate::models::TokenFreshness::Stale => Color::Yellow,
        crate::models::TokenFreshness::Old => Color::Red,
        crate::models::TokenFreshness::Unknown(_) => Color::DarkGray,
    }
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

    // ── 配额刷新确认提示 ──
    if app.pending_quota_confirm {
        content.push(Line::from(Span::styled(
            "  确认查询配额？按 y 确认 / 其他键取消",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(ratatui::style::Modifier::BOLD),
        )));
    }

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
                        .map(|t| {
                            let relative = CodexQuotaService::format_reset_duration(t);
                            let dt = chrono::DateTime::from_timestamp(t, 0)
                                .map(|d| d.with_timezone(&chrono::Local));
                            if let Some(local) = dt {
                                format!("  重置: {} ({})", relative, local.format("%m/%d %H:%M"))
                            } else {
                                format!("  重置: {}", relative)
                            }
                        })
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
                    if is_refresh_token_reused_error(err) {
                        content.push(Line::from(Span::styled(
                            "  提示: Token 已轮换，可按 R 尝试修复；仍失败请重新登录后保存账号",
                            Style::default().fg(Color::Yellow),
                        )));
                    }
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

fn is_refresh_token_reused_error(message: &str) -> bool {
    let lower = message.to_ascii_lowercase();
    lower.contains("refresh_token_reused") || lower.contains("invalid_grant")
}

/// Draw help bar (overlay-aware)
fn draw_help_bar(f: &mut Frame, area: Rect, app: &CodexAuthApp) {
    let help_text = match &app.overlay {
        Some(Overlay::Confirm { .. }) => "y 确认删除 | n/Esc 取消",
        Some(Overlay::Input { .. }) => "Enter 确认 | Esc 取消",
        None => {
            "↑/k 上移 | ↓/j 下移 | Enter 切换 | s 保存当前 | d 删除 | r 刷新 | R 修复 | b 配额 | q 退出"
        }
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
            Constraint::Length(12), // Usage panel
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

pub fn draw_loading_placeholder(
    f: &mut Frame,
    content_area: Rect,
    footer_area: Rect,
    compact: bool,
    error: Option<&str>,
) {
    let message = error
        .map(|err| format!("Codex Auth 初始化失败\n\n{}", err))
        .unwrap_or_else(|| "正在初始化 Codex Auth...".to_string());

    let panel = Paragraph::new(message)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::BORDER))
                .title(" 🔐 Codex Auth ")
                .title_style(Style::default().fg(theme::ACCENT)),
        )
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    f.render_widget(panel, content_area);

    if compact {
        let help = Paragraph::new("Tab 切换")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        f.render_widget(help, footer_area);
    } else {
        let footer_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Length(2)])
            .split(footer_area);

        let status_text = if error.is_some() {
            "初始化失败"
        } else {
            "加载中"
        };
        let status_style = if error.is_some() {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::Cyan)
        };

        let status = Paragraph::new(status_text).style(status_style).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::BORDER))
                .title(" 状态 ")
                .title_style(Style::default().fg(theme::ACCENT)),
        );
        f.render_widget(status, footer_chunks[0]);

        let help = Paragraph::new("Tab 切换")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        f.render_widget(help, footer_chunks[1]);
    }
}

fn draw_account_list_with_status(f: &mut Frame, area: Rect, app: &CodexAuthApp) {
    let title = format!(" 🔐 账号列表 | {} ", login_status_text(app));
    render_account_list_panel(f, area, app, title);
}

/// Draw help bar with Tab switch hint (embedded mode)
fn draw_help_bar_embedded(f: &mut Frame, area: Rect, app: &CodexAuthApp) {
    let help_text = match &app.overlay {
        Some(Overlay::Confirm { .. }) => "y 确认删除 | n/Esc 取消",
        Some(Overlay::Input { .. }) => "Enter 确认 | Esc 取消",
        None => {
            "Tab 切换 | ↑/k 上移 | ↓/j 下移 | Enter 切换 | s 保存当前 | d 删除 | r 刷新 | R 修复 | b 配额 | q 退出"
        }
    };

    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);

    f.render_widget(help, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn account_table_layout_hides_secondary_columns_on_narrow_widths() {
        let layout = account_table_layout(60);
        assert_eq!(
            layout.columns,
            vec![
                AccountColumn::Status,
                AccountColumn::Account,
                AccountColumn::Email
            ]
        );
        assert_eq!(layout.widths.len(), 3);
    }

    #[test]
    fn account_table_layout_shows_description_on_wide_widths() {
        let layout = account_table_layout(108);
        assert_eq!(
            layout.columns,
            vec![
                AccountColumn::Status,
                AccountColumn::Account,
                AccountColumn::Email,
                AccountColumn::SavedAt,
                AccountColumn::ExpiresAt,
                AccountColumn::Description,
            ]
        );
        assert_eq!(layout.widths.len(), 6);
    }

    #[test]
    fn account_list_regions_reserve_one_row_for_header() {
        let inner = Rect::new(2, 3, 80, 9);
        let regions = account_list_regions(inner);

        assert_eq!(regions.header, Rect::new(2, 3, 80, 1));
        assert_eq!(regions.body, Rect::new(2, 4, 80, 8));
    }
}
