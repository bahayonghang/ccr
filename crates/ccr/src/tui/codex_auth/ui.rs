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
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

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
        Span::styled(login_status, login_status_style(&app.login_state)),
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
    resolved_widths: Vec<u16>,
}

impl AccountTableLayout {
    fn new(columns: Vec<AccountColumn>, widths: Vec<Constraint>, inner_width: u16) -> Self {
        let resolved_widths = resolve_table_widths(inner_width, &widths, ACCOUNT_COLUMN_SPACING);
        Self {
            columns,
            widths,
            resolved_widths,
        }
    }

    fn text_width(&self, column: AccountColumn) -> usize {
        let width = usize::from(self.resolved_width(column));

        match column {
            AccountColumn::Account | AccountColumn::Email | AccountColumn::Description => width,
            _ => width,
        }
    }

    fn resolved_width(&self, column: AccountColumn) -> u16 {
        self.columns
            .iter()
            .position(|current| *current == column)
            .and_then(|index| self.resolved_widths.get(index))
            .copied()
            .unwrap_or(0)
    }

    fn account_name_width(&self, account: &crate::models::CodexAuthItem) -> usize {
        let reserved = if account.is_virtual { 2 } else { 0 };
        self.text_width(AccountColumn::Account)
            .saturating_sub(reserved)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AccountListRegions {
    header: Rect,
    body: Rect,
}

const ACCOUNT_COLUMN_SPACING: u16 = 1;
const DETAIL_LABEL_WIDTH: usize = 12;

fn resolve_table_widths(
    inner_width: u16,
    constraints: &[Constraint],
    column_spacing: u16,
) -> Vec<u16> {
    let spacing = column_spacing.saturating_mul(constraints.len().saturating_sub(1) as u16);
    let mut remaining = inner_width.saturating_sub(spacing);
    let mut resolved = vec![0; constraints.len()];
    let mut flexible = Vec::new();

    for (index, constraint) in constraints.iter().enumerate() {
        match *constraint {
            Constraint::Length(width) => {
                let assigned = width.min(remaining);
                resolved[index] = assigned;
                remaining = remaining.saturating_sub(assigned);
            }
            Constraint::Min(width) => {
                let assigned = width.min(remaining);
                resolved[index] = assigned;
                remaining = remaining.saturating_sub(assigned);
                flexible.push(index);
            }
            _ => flexible.push(index),
        }
    }

    if !flexible.is_empty() && remaining > 0 {
        let share = remaining / flexible.len() as u16;
        let remainder = remaining % flexible.len() as u16;

        for (offset, index) in flexible.into_iter().enumerate() {
            resolved[index] = resolved[index]
                .saturating_add(share)
                .saturating_add(u16::from((offset as u16) < remainder));
        }
    }

    resolved
}

fn detail_label_span(label: &str) -> Span<'static> {
    Span::styled(
        format!("{label:<DETAIL_LABEL_WIDTH$}"),
        Style::default()
            .fg(theme::FG_SECONDARY)
            .add_modifier(Modifier::BOLD),
    )
}

fn detail_line(label: &str, value: impl Into<String>, style: Style) -> Line<'static> {
    detail_spans_line(label, vec![Span::styled(value.into(), style)])
}

fn detail_optional_line(label: &str, value: Option<&str>, style: Style) -> Line<'static> {
    match value.map(str::trim).filter(|value| !value.is_empty()) {
        Some(value) => detail_line(label, value.to_string(), style),
        None => detail_line(label, "-", theme::muted_style()),
    }
}

fn detail_spans_line(label: &str, mut spans: Vec<Span<'static>>) -> Line<'static> {
    let mut all = vec![detail_label_span(label)];
    all.append(&mut spans);
    Line::from(all)
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
            .style(theme::muted_style())
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
        return AccountTableLayout::new(
            vec![
                AccountColumn::Status,
                AccountColumn::Account,
                AccountColumn::Email,
            ],
            vec![
                Constraint::Length(4),
                Constraint::Length(16),
                Constraint::Min(12),
            ],
            inner_width,
        );
    }

    if inner_width < 92 {
        return AccountTableLayout::new(
            vec![
                AccountColumn::Status,
                AccountColumn::Account,
                AccountColumn::Email,
                AccountColumn::SavedAt,
                AccountColumn::ExpiresAt,
            ],
            vec![
                Constraint::Length(4),
                Constraint::Length(16),
                Constraint::Min(16),
                Constraint::Length(10),
                Constraint::Length(10),
            ],
            inner_width,
        );
    }

    AccountTableLayout::new(
        vec![
            AccountColumn::Status,
            AccountColumn::Account,
            AccountColumn::Email,
            AccountColumn::SavedAt,
            AccountColumn::ExpiresAt,
            AccountColumn::Description,
        ],
        vec![
            Constraint::Length(4),
            Constraint::Length(18),
            Constraint::Length(24),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Min(12),
        ],
        inner_width,
    )
}

fn render_account_list_header(f: &mut Frame, area: Rect, layout: &AccountTableLayout) {
    let header_cells = layout.columns.iter().map(account_header_cell);
    let header = Table::new([Row::new(header_cells)], layout.widths.clone())
        .column_spacing(ACCOUNT_COLUMN_SPACING)
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
                    .map(|column| account_cell(account, *column, layout)),
            )
            .style(row_style)
            .height(1)
        });

    let table = Table::new(rows, layout.widths.clone()).column_spacing(ACCOUNT_COLUMN_SPACING);
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

fn account_cell(
    account: &crate::models::CodexAuthItem,
    column: AccountColumn,
    layout: &AccountTableLayout,
) -> Cell<'static> {
    match column {
        AccountColumn::Status => Cell::from(Line::from(Span::styled(
            account.freshness.icon().to_string(),
            Style::default().fg(freshness_color(&account.freshness)),
        ))),
        AccountColumn::Account => {
            let name_style = if account.is_virtual {
                theme::warning_style().add_modifier(Modifier::ITALIC)
            } else if account.is_current {
                theme::success_style()
            } else {
                Style::default().fg(theme::FG_PRIMARY)
            };

            let account_name = truncate_text(&account.name, layout.account_name_width(account));
            let mut spans = vec![Span::styled(account_name, name_style)];

            if account.is_virtual {
                spans.push(Span::styled(
                    " *",
                    theme::warning_style().add_modifier(Modifier::ITALIC),
                ));
            }

            Cell::from(Line::from(spans))
        }
        AccountColumn::Email => {
            let email = truncate_text(
                account.email.as_deref().unwrap_or("-"),
                layout.text_width(AccountColumn::Email),
            );
            Cell::from(Line::from(Span::styled(email, theme::info_style())))
        }
        AccountColumn::SavedAt => Cell::from(Line::from(Span::styled(
            format_saved_at(account),
            Style::default().fg(theme::FG_PRIMARY),
        ))),
        AccountColumn::ExpiresAt => {
            let (text, style) = format_expires_at(account);
            Cell::from(Line::from(Span::styled(text, style)))
        }
        AccountColumn::Description => {
            let description = truncate_text(
                account.description.as_deref().unwrap_or("-"),
                layout.text_width(AccountColumn::Description),
            );
            Cell::from(Line::from(Span::styled(description, theme::muted_style())))
        }
    }
}

fn truncate_text(value: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }

    let width = value.width();
    if width <= max_width {
        return value.to_string();
    }

    if max_width == 1 {
        return "…".to_string();
    }

    let mut result = String::new();
    let mut current_width = 0;

    for ch in value.chars() {
        let ch_width = ch.width().unwrap_or(0);
        if current_width + ch_width > max_width - 1 {
            break;
        }
        result.push(ch);
        current_width += ch_width;
    }

    result.push('…');
    result
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

fn login_status_style(login_state: &crate::models::LoginState) -> Style {
    match login_state {
        crate::models::LoginState::NotLoggedIn | crate::models::LoginState::Unknown { .. } => {
            theme::error_style()
        }
        crate::models::LoginState::LoggedInUnsaved => theme::warning_style(),
        crate::models::LoginState::LoggedInSaved(_) => theme::success_style(),
        crate::models::LoginState::ApiKeyActive
        | crate::models::LoginState::ProviderKeyActive { .. } => theme::info_style(),
    }
}

fn codex_runtime_mode_color(mode: crate::models::CodexRuntimeMode) -> Color {
    match mode {
        crate::models::CodexRuntimeMode::ProfileOnly => theme::FG_SUCCESS,
        crate::models::CodexRuntimeMode::ProfileWithAuth
        | crate::models::CodexRuntimeMode::ProfilePendingAuth => theme::FG_WARNING,
        crate::models::CodexRuntimeMode::RuntimeOnly => theme::FG_INFO,
        crate::models::CodexRuntimeMode::Unresolved => theme::FG_MUTED,
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
            let style = if expired {
                theme::error_style()
            } else {
                theme::success_style()
            };
            (text, style)
        }
        None => ("-".to_string(), theme::muted_style()),
    }
}

fn freshness_color(freshness: &crate::models::TokenFreshness) -> Color {
    match freshness {
        crate::models::TokenFreshness::Fresh => theme::FG_SUCCESS,
        crate::models::TokenFreshness::Stale => theme::FG_WARNING,
        crate::models::TokenFreshness::Old => theme::FG_ERROR,
        crate::models::TokenFreshness::Unknown(_) => theme::FG_MUTED,
    }
}

/// Render status bar with toast notification
fn draw_status_bar(f: &mut Frame, area: Rect, app: &CodexAuthApp) {
    let (message, style) = if let Some(toast) = app.toasts.active() {
        let s = match toast.kind {
            ToastKind::Success => theme::success_style(),
            ToastKind::Error => theme::error_style(),
            ToastKind::Warning => theme::warning_style(),
            ToastKind::Info => theme::info_style(),
        };
        (toast.message.as_str(), s)
    } else {
        ("就绪", theme::success_style())
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
        Span::styled(" 📊 ", theme::info_style()),
        Span::styled(
            "Codex 使用情况",
            Style::default()
                .fg(theme::FG_PRIMARY)
                .add_modifier(Modifier::BOLD),
        ),
    ]);

    let mut content: Vec<Line> = Vec::new();

    // ── 配额刷新确认提示 ──
    if app.pending_quota_confirm {
        content.push(Line::from(Span::styled(
            "  确认查询配额？按 y 确认 / 其他键取消",
            theme::warning_style(),
        )));
    }

    // ── 配额余额区域 ──
    match &app.quota_state {
        QuotaState::Idle => {
            content.push(Line::from(Span::styled(
                "  按 b 查询配额余额",
                theme::muted_style(),
            )));
        }
        QuotaState::Loading => {
            content.push(Line::from(Span::styled(
                "  ⏳ 正在查询配额...",
                theme::warning_style(),
            )));
        }
        QuotaState::Error(err) => {
            content.push(Line::from(Span::styled(
                format!("  ⚠️ 配额查询失败: {}", err),
                theme::error_style(),
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
                        Span::styled("  配额 ", theme::info_style()),
                        Span::styled(format!("({})", account_label), theme::muted_style()),
                    ]));

                    // 5h 限额
                    let h_color = percent_color(quota.hourly_percentage);
                    let h_bar = progress_bar(quota.hourly_percentage, 10);
                    let h_reset = quota
                        .hourly_reset_time
                        .map(|t| format!("  重置: {}", CodexQuotaService::format_reset_duration(t)))
                        .unwrap_or_default();
                    content.push(Line::from(vec![
                        Span::styled("  5h限额: ", Style::default().fg(theme::FG_PRIMARY)),
                        Span::styled(h_bar, Style::default().fg(h_color)),
                        Span::styled(
                            format!(" {}%", quota.hourly_percentage),
                            Style::default().fg(h_color),
                        ),
                        Span::styled(h_reset, theme::muted_style()),
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
                        Span::styled("  周限额: ", Style::default().fg(theme::FG_PRIMARY)),
                        Span::styled(w_bar, Style::default().fg(w_color)),
                        Span::styled(
                            format!(" {}%", quota.weekly_percentage),
                            Style::default().fg(w_color),
                        ),
                        Span::styled(w_reset, theme::muted_style()),
                    ]));

                    // 订阅类型
                    if let Some(ref plan) = quota.plan_type {
                        content.push(Line::from(vec![
                            Span::styled("  订阅: ", Style::default().fg(theme::FG_PRIMARY)),
                            Span::styled(plan.clone(), theme::info_style()),
                        ]));
                    }
                } else if let Some(ref err) = aq.error {
                    content.push(Line::from(Span::styled(
                        format!("  ⚠️ {}: {}", aq.account_name, err),
                        theme::error_style(),
                    )));
                    if is_refresh_token_reused_error(err) {
                        content.push(Line::from(Span::styled(
                            "  提示: Token 已轮换，可按 R 尝试修复；仍失败请重新登录后保存账号",
                            theme::warning_style(),
                        )));
                    }
                }
            } else if !quotas.is_empty() {
                // 显示第一个有配额的账号
                content.push(Line::from(Span::styled(
                    "  选中账号无配额数据，按 b 刷新",
                    theme::muted_style(),
                )));
            }
        }
    }

    // ── 分隔线 ──
    content.push(Line::from(Span::styled(
        "  ────────────────────────────────",
        theme::muted_style(),
    )));

    // ── 本地统计区域 ──
    match &app.usage_state {
        UsageState::NoData => {
            content.push(Line::from(Span::styled(
                "  📭 暂无本地使用数据",
                theme::muted_style(),
            )));
        }
        UsageState::Error(err) => {
            content.push(Line::from(Span::styled(
                format!("  ⚠️ 统计加载失败: {}", err),
                theme::error_style(),
            )));
        }
        UsageState::Loaded(usage) => {
            content.extend(usage_digest_strings(usage).into_iter().map(|line| {
                Line::from(Span::styled(line, Style::default().fg(theme::FG_PRIMARY)))
            }));
        }
        UsageState::Loading => {
            content.push(Line::from(Span::styled(
                "  ⏳ 加载中...",
                theme::muted_style(),
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
        theme::FG_SUCCESS
    } else if pct >= 30 {
        theme::FG_WARNING
    } else {
        theme::FG_ERROR
    }
}

/// 生成文本进度条
fn progress_bar(pct: i32, width: usize) -> String {
    let filled = ((pct as usize) * width / 100).min(width);
    let empty = width - filled;
    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}

fn account_snapshot_lines(account: &crate::models::CodexAuthItem) -> Vec<Line<'static>> {
    let account_style = if account.is_current {
        theme::success_style()
    } else if account.is_virtual {
        theme::warning_style().add_modifier(Modifier::ITALIC)
    } else {
        Style::default()
            .fg(theme::FG_PRIMARY)
            .add_modifier(Modifier::BOLD)
    };
    let state_style = if account.is_current {
        theme::success_style()
    } else if account.is_virtual {
        theme::warning_style()
    } else {
        Style::default().fg(theme::FG_PRIMARY)
    };
    let (expires_text, expires_style) = format_expires_at(account);

    vec![
        detail_line("Account:", account.name.clone(), account_style),
        detail_line(
            "State:",
            format!(
                "{}{}",
                if account.is_current {
                    "Current"
                } else {
                    "Saved"
                },
                if account.is_virtual {
                    " · Virtual"
                } else {
                    ""
                }
            ),
            state_style,
        ),
        detail_optional_line("Email:", account.email.as_deref(), theme::info_style()),
        detail_line(
            "Freshness:",
            account.freshness.description(),
            Style::default().fg(freshness_color(&account.freshness)),
        ),
        detail_line(
            "Saved at:",
            format_saved_at(account),
            Style::default().fg(theme::FG_PRIMARY),
        ),
        detail_line("Expires:", expires_text, expires_style),
        detail_optional_line(
            "Description:",
            account.description.as_deref(),
            theme::muted_style(),
        ),
    ]
}

fn usage_digest_strings(usage: &crate::services::CodexRollingUsage) -> Vec<String> {
    let five_total = usage.five_hour.total_input_tokens + usage.five_hour.total_output_tokens;
    let seven_total = usage.seven_day.total_input_tokens + usage.seven_day.total_output_tokens;
    let all_time = usage.all_time.total_input_tokens + usage.all_time.total_output_tokens;

    let top_model = usage
        .by_model
        .iter()
        .max_by_key(|(_, stats)| stats.total_input_tokens + stats.total_output_tokens)
        .map(|(model, stats)| {
            let total = stats.total_input_tokens + stats.total_output_tokens;
            format!(
                "  Top model: {} ({}, {} req)",
                model,
                CodexUsageService::format_tokens(total),
                stats.total_requests
            )
        })
        .unwrap_or_else(|| "  Top model: -".to_string());

    vec![
        format!(
            "  5小时: {} tokens ({} 请求)",
            CodexUsageService::format_tokens(five_total),
            usage.five_hour.total_requests
        ),
        format!(
            "  7天:   {} tokens ({} 请求)",
            CodexUsageService::format_tokens(seven_total),
            usage.seven_day.total_requests
        ),
        format!(
            "  All time: {} tokens ({} 请求)",
            CodexUsageService::format_tokens(all_time),
            usage.all_time.total_requests
        ),
        top_model,
    ]
}

fn runtime_digest_lines(app: &CodexAuthApp) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    if let Some(summary) = &app.runtime_summary {
        lines.push(detail_line(
            "Mode:",
            summary.mode.label(),
            Style::default().fg(codex_runtime_mode_color(summary.mode)),
        ));
        lines.push(detail_line(
            "Profile:",
            summary.profile_label(),
            Style::default().fg(theme::FG_PRIMARY),
        ));
        lines.push(detail_line(
            "Auth:",
            summary.auth_label(),
            theme::info_style(),
        ));
    } else {
        lines.push(detail_line("Mode:", "未解析", theme::muted_style()));
    }

    lines.push(detail_line(
        "Login:",
        login_status_text(app),
        login_status_style(&app.login_state),
    ));

    let recent = if let Some((action, name, success, error)) = &app.last_action {
        if *success {
            detail_line(
                "Recent:",
                format!("{action} {name}"),
                theme::success_style(),
            )
        } else {
            detail_line(
                "Recent:",
                format!(
                    "{} {} failed{}",
                    action,
                    name,
                    error
                        .as_ref()
                        .map(|err| format!(" ({err})"))
                        .unwrap_or_default()
                ),
                theme::error_style(),
            )
        }
    } else {
        detail_line(
            "Recent:",
            "Enter switch · s save · b quota",
            theme::muted_style(),
        )
    };

    lines.push(recent);
    lines
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
        .style(theme::muted_style())
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
    mode: crate::tui::theme::ViewportMode,
) {
    match mode {
        crate::tui::theme::ViewportMode::Compact => {
            let content_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(8), Constraint::Length(12)])
                .split(content_area);

            draw_account_list_with_status(f, content_chunks[0], app);
            draw_usage_panel(f, content_chunks[1], app);
        }
        crate::tui::theme::ViewportMode::Standard => {
            let content_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(8),
                    Constraint::Length(12),
                    Constraint::Length(8),
                ])
                .split(content_area);

            draw_account_list_with_status(f, content_chunks[0], app);
            draw_usage_panel(f, content_chunks[1], app);
            draw_runtime_panel(f, content_chunks[2], app);
        }
        crate::tui::theme::ViewportMode::Wide => {
            let columns = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
                .split(content_area);

            let left = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(8), Constraint::Length(4)])
                .split(columns[0]);
            draw_account_list_with_status(f, left[0], app);
            draw_account_list_meta_panel(f, left[1], app);

            let right = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(8),
                    Constraint::Length(12),
                    Constraint::Min(6),
                ])
                .split(columns[1]);
            draw_account_snapshot_panel(f, right[0], app);
            draw_usage_panel(f, right[1], app);
            draw_runtime_panel(f, right[2], app);
        }
    }

    draw_footer_strip(f, footer_area, app);

    // Draw overlay (with dark backdrop) if active
    if let Some(overlay) = &app.overlay {
        render_overlay(f, overlay);
    }
}

pub fn draw_loading_placeholder(
    f: &mut Frame,
    content_area: Rect,
    footer_area: Rect,
    mode: crate::tui::theme::ViewportMode,
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

    if mode == crate::tui::theme::ViewportMode::Compact {
        let help = Paragraph::new("Tab 切换")
            .style(theme::muted_style())
            .alignment(Alignment::Center);
        f.render_widget(help, footer_area);
    } else {
        let status_text = if error.is_some() {
            "初始化失败"
        } else {
            "加载中"
        };
        let status_style = if error.is_some() {
            theme::error_style()
        } else {
            theme::info_style()
        };

        let status = Paragraph::new(status_text).style(status_style).block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(theme::BORDER))
                .title(" Keys ")
                .title_style(Style::default().fg(theme::ACCENT)),
        );
        f.render_widget(status, footer_area);
    }
}

fn draw_account_list_with_status(f: &mut Frame, area: Rect, app: &CodexAuthApp) {
    let title = format!(" 🔐 账号列表 | {} ", login_status_text(app));
    render_account_list_panel(f, area, app, title);
}

fn draw_account_list_meta_panel(f: &mut Frame, area: Rect, app: &CodexAuthApp) {
    let selected_line = if let Some(account) = app.selected_account() {
        let style = if account.is_current {
            theme::success_style()
        } else if account.is_virtual {
            theme::warning_style()
        } else {
            Style::default().fg(theme::FG_PRIMARY)
        };
        detail_line("Selected:", account.name.clone(), style)
    } else {
        detail_line("Selected:", "-", theme::muted_style())
    };
    let lines = vec![
        selected_line,
        detail_line(
            "Page:",
            format!("{}/{}", app.current_page + 1, app.total_pages()),
            Style::default().fg(theme::FG_PRIMARY),
        ),
        detail_spans_line(
            "Legend:",
            vec![
                Span::styled("🟢 fresh", theme::success_style()),
                Span::styled(" · ", theme::muted_style()),
                Span::styled("🟡 stale", theme::warning_style()),
                Span::styled(" · ", theme::muted_style()),
                Span::styled("🔴 old", theme::error_style()),
            ],
        ),
        detail_line(
            "Actions:",
            "Enter switch · s save · d delete",
            theme::muted_style(),
        ),
    ];

    let panel = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::BORDER))
                .title(" Legend ")
                .title_style(Style::default().fg(theme::FG_SECONDARY)),
        )
        .wrap(Wrap { trim: true });

    f.render_widget(panel, area);
}

fn draw_account_snapshot_panel(f: &mut Frame, area: Rect, app: &CodexAuthApp) {
    let lines = app
        .selected_account()
        .map(account_snapshot_lines)
        .unwrap_or_else(|| {
            vec![
                detail_line("Account:", "-", theme::muted_style()),
                detail_line("State:", "-", theme::muted_style()),
            ]
        });

    let panel = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::BORDER))
                .title(" Focus ")
                .title_style(Style::default().fg(theme::ACCENT)),
        )
        .wrap(Wrap { trim: true });

    f.render_widget(panel, area);
}

fn draw_runtime_panel(f: &mut Frame, area: Rect, app: &CodexAuthApp) {
    let panel = Paragraph::new(runtime_digest_lines(app))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::BORDER))
                .title(" Runtime ")
                .title_style(Style::default().fg(theme::ACCENT)),
        )
        .wrap(Wrap { trim: true });

    f.render_widget(panel, area);
}

fn draw_footer_strip(f: &mut Frame, area: Rect, app: &CodexAuthApp) {
    let message = if let Some(toast) = app.toasts.active() {
        format!(
            "{}  │  Tab switch  │  ↑↓/jk select  │  Enter switch  │  s save  │  b quota  │  q quit",
            toast.message
        )
    } else {
        "Tab switch  │  ↑↓/jk select  │  Enter switch  │  s save  │  d delete  │  b quota  │  q quit"
            .to_string()
    };

    let help = Paragraph::new(message)
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(theme::BORDER))
                .title(" Keys ")
                .title_style(Style::default().fg(theme::FG_MUTED)),
        )
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

    f.render_widget(help, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ccr_codex::services::codex_usage_service::CodexUsageStats;
    use chrono::{TimeZone, Utc};
    use ratatui::{Terminal, backend::TestBackend};
    use std::collections::HashMap;

    fn sample_account() -> crate::models::CodexAuthItem {
        crate::models::CodexAuthItem {
            name: "codexcn".to_string(),
            description: Some("Primary account".to_string()),
            email: Some("bah***@gmail.com".to_string()),
            is_current: true,
            is_virtual: false,
            saved_at: Some(Utc.with_ymd_and_hms(2026, 4, 5, 12, 0, 0).unwrap()),
            last_used: None,
            last_refresh: None,
            freshness: crate::models::TokenFreshness::Fresh,
            expires_at: None,
        }
    }

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
    fn account_table_layout_resolves_flexible_widths_from_available_space() {
        let narrow = account_table_layout(60);
        let wide = account_table_layout(108);

        assert_eq!(narrow.resolved_width(AccountColumn::Status), 4);
        assert_eq!(narrow.resolved_width(AccountColumn::Email), 38);
        assert_eq!(wide.resolved_width(AccountColumn::Description), 37);
    }

    #[test]
    fn account_list_regions_reserve_one_row_for_header() {
        let inner = Rect::new(2, 3, 80, 9);
        let regions = account_list_regions(inner);

        assert_eq!(regions.header, Rect::new(2, 3, 80, 1));
        assert_eq!(regions.body, Rect::new(2, 4, 80, 8));
    }

    #[test]
    fn account_snapshot_lines_include_freshness_description_and_semantic_styles() {
        let lines = account_snapshot_lines(&sample_account());

        assert!(plain_line_text(&lines[0]).contains("codexcn"));
        assert!(plain_line_text(&lines[3]).contains("Token 状态良好"));
        assert!(plain_line_text(&lines[6]).contains("Primary account"));
        assert_eq!(lines[0].spans[0].style.fg, Some(theme::FG_SECONDARY));
        assert_eq!(lines[0].spans[1].style.fg, Some(theme::FG_SUCCESS));
        assert_eq!(lines[2].spans[1].style.fg, Some(theme::FG_INFO));
    }

    #[test]
    fn account_table_render_keeps_email_visible_when_space_is_available() {
        let layout = account_table_layout(60);
        let mut terminal = Terminal::new(TestBackend::new(60, 1)).unwrap();
        let account = sample_account();

        terminal
            .draw(|frame| {
                let row = Row::new(
                    layout
                        .columns
                        .iter()
                        .map(|column| account_cell(&account, *column, &layout)),
                );
                let table =
                    Table::new([row], layout.widths.clone()).column_spacing(ACCOUNT_COLUMN_SPACING);
                frame.render_widget(table, frame.area());
            })
            .unwrap();

        let rendered = buffer_line_text(terminal.backend(), 0);
        assert!(rendered.contains("🟢"), "{rendered}");
        assert!(rendered.contains("codexcn"), "{rendered}");
        assert!(rendered.contains("bah***@gmail.com"), "{rendered}");
        assert!(!rendered.contains("bah***@gmai…"), "{rendered}");
    }

    #[test]
    fn usage_digest_strings_include_all_time_and_top_model() {
        let mut usage = crate::services::CodexRollingUsage::default();
        usage.five_hour.total_input_tokens = 1_000;
        usage.five_hour.total_output_tokens = 2_000;
        usage.five_hour.total_requests = 3;
        usage.seven_day.total_input_tokens = 10_000;
        usage.seven_day.total_output_tokens = 20_000;
        usage.seven_day.total_requests = 30;
        usage.all_time.total_input_tokens = 50_000;
        usage.all_time.total_output_tokens = 10_000;
        usage.all_time.total_requests = 42;
        usage.by_model = HashMap::from([(
            "gpt-5.4".to_string(),
            CodexUsageStats {
                total_input_tokens: 30_000,
                total_output_tokens: 5_000,
                total_requests: 21,
                ..Default::default()
            },
        )]);

        let lines = usage_digest_strings(&usage);

        assert!(
            lines
                .iter()
                .any(|line| line.contains("All time: 60.0K tokens"))
        );
        assert!(lines.iter().any(|line| line.contains("Top model: gpt-5.4")));
    }
}
