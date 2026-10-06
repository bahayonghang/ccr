// 🎨 Codex Auth TUI UI rendering
// Draws the Codex multi-account management interface

use super::app::{
    CodexAuthApp, CodexAuthUsagePanelData, CodexUsageAttributionState, CodexUsageScope,
    PreviewMetricWindow, QuotaPreviewCellState, UsageState,
};
use crate::tui::footer::{ShortcutHint, shortcut_line};
use crate::tui::overlay::{Overlay, render_overlay};
use crate::tui::theme;
use crate::tui::toast::ToastKind;
use ccr_cli::services::CodexQuotaService;
use ccr_codex::services::codex_quota_service::relogin_required_detail;
use ccr_codex::services::codex_usage_estimation::{
    CodexCapacityEstimate, CodexCostStatus, CodexCostSummary, CodexEstimateRange,
    CodexEstimateStatus,
};
use chrono::Local;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap},
};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// 🎨 Draw main interface
pub fn draw(f: &mut Frame, app: &mut CodexAuthApp) {
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
        ccr_cli::models::LoginState::NotLoggedIn => {
            crate::tui_text!("Not logged in", "未登录").to_string()
        }
        ccr_cli::models::LoginState::LoggedInUnsaved => {
            crate::tui_text!("Logged in (unsaved)", "已登录（未保存）").to_string()
        }
        ccr_cli::models::LoginState::LoggedInSaved(name) => {
            crate::tui_format!("Logged in: {}", "已登录：{}", name)
        }
        ccr_cli::models::LoginState::ApiKeyActive => {
            crate::tui_text!("API Key mode", "API Key 模式").to_string()
        }
        ccr_cli::models::LoginState::ProviderKeyActive { env_key } => {
            crate::tui_format!("Provider Key: {}", "提供商密钥：{}", env_key)
        }
        ccr_cli::models::LoginState::Unknown { type_name, .. } => {
            crate::tui_format!("Unknown state: {}", "未知状态：{}", type_name)
        }
    };

    let title = Paragraph::new(vec![Line::from(vec![
        Span::styled(
            crate::tui_text!(" Codex Account Manager ", " Codex 账号管理 "),
            Style::default()
                .fg(theme::codex())
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(login_status, login_status_style(&app.login_state)),
    ])])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::border()))
            .title(" CCR ")
            .title_style(Style::default().fg(theme::codex())),
    )
    .alignment(Alignment::Center);

    f.render_widget(title, area);
}

fn draw_account_list(f: &mut Frame, area: Rect, app: &mut CodexAuthApp) {
    render_account_list_panel(
        f,
        area,
        app,
        crate::tui_text!(" Accounts ", " 账号列表 ").to_string(),
    );
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AccountColumn {
    Account,
    Email,
    Plan,
    QuotaSummary,
    HourlyQuota,
    WeeklyQuota,
    ExpiresAt,
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
        usize::from(self.resolved_width(column))
    }

    fn resolved_width(&self, column: AccountColumn) -> u16 {
        self.columns
            .iter()
            .position(|current| *current == column)
            .and_then(|index| self.resolved_widths.get(index))
            .copied()
            .unwrap_or(0)
    }

    fn account_name_width(&self, account: &ccr_cli::models::CodexAuthItem) -> usize {
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
const DETAIL_LABEL_WIDTH: usize = 14;

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
    let label = localized_detail_label(label);
    Span::styled(
        pad_text(label, DETAIL_LABEL_WIDTH.max(label.width() + 1)),
        Style::default()
            .fg(theme::subtext())
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

fn localized_detail_label(label: &str) -> &str {
    match label {
        "Account:" => crate::tui_text!("Account:", "账号："),
        "State:" => crate::tui_text!("State:", "状态："),
        "Email:" => crate::tui_text!("Email:", "邮箱："),
        "Plan:" => crate::tui_text!("Plan:", "属性："),
        "Saved at:" => crate::tui_text!("Saved at:", "保存时间："),
        "Last refresh:" => crate::tui_text!("Auth refresh:", "认证刷新："),
        "Quota scope:" => crate::tui_text!("Quota scope:", "配额范围："),
        "Usage scope:" => crate::tui_text!("Usage scope:", "用量范围："),
        "Attribution:" => crate::tui_text!("Attribution:", "归因："),
        _ => label,
    }
}

fn normalize_plan_display(plan: &str) -> Option<String> {
    let normalized = plan
        .trim()
        .replace(['_', '-'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    if normalized.is_empty() {
        None
    } else {
        Some(normalized.to_ascii_uppercase())
    }
}

fn quota_plan_for_account(
    app: &CodexAuthApp,
    account: &ccr_cli::models::CodexAuthItem,
) -> Option<String> {
    let selected_quota = app
        .selected_account()
        .filter(|selected| selected.name == account.name)
        .and_then(|_| app.selected_quota())
        .and_then(|quota| quota.quota.as_ref())
        .and_then(|quota| quota.plan_type.as_deref());

    let preview_quota = app
        .preview_quota_for_account(&account.name)
        .and_then(|quota| quota.quota.as_ref())
        .and_then(|quota| quota.plan_type.as_deref());

    selected_quota
        .or(preview_quota)
        .or(account.plan_type.as_deref())
        .and_then(normalize_plan_display)
}

fn account_property_display(
    app: &CodexAuthApp,
    account: &ccr_cli::models::CodexAuthItem,
) -> (String, Style) {
    if let Some(plan) = quota_plan_for_account(app, account) {
        return (plan, theme::info_style());
    }

    if account.is_virtual {
        return (
            "VIRTUAL".to_string(),
            theme::warning_style().add_modifier(Modifier::ITALIC),
        );
    }

    let registry_account = app.auth_registry.accounts.get(&account.name);

    match registry_account.and_then(|entry| entry.auth_method) {
        Some(ccr_cli::models::OpenAiAuthMethod::Chatgpt) => {
            ("CHATGPT".to_string(), theme::info_style())
        }
        Some(ccr_cli::models::OpenAiAuthMethod::Api) => ("API".to_string(), theme::muted_style()),
        None if registry_account
            .and_then(|entry| entry.api_provider_name.as_deref())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .is_some() =>
        {
            ("PROVIDER".to_string(), theme::muted_style())
        }
        None => ("-".to_string(), theme::muted_style()),
    }
}

fn detail_spans_line(label: &str, mut spans: Vec<Span<'static>>) -> Line<'static> {
    let mut all = vec![detail_label_span(label)];
    all.append(&mut spans);
    Line::from(all)
}

fn render_account_list_panel(f: &mut Frame, area: Rect, app: &mut CodexAuthApp, title: String) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::codex()))
        .title(title)
        .title_style(theme::codex_style());

    let inner = block.inner(area);
    if !app.accounts.is_empty() && inner.height >= 1 {
        let regions = account_list_regions(inner);
        app.sync_page_size(crate::tui::pagination::visible_page_size(
            regions.body.height,
        ));
    }

    let block = if inner.height >= 4 {
        block.title_bottom(account_list_footer_line(app))
    } else {
        block
    };
    f.render_widget(block, area);

    if app.accounts.is_empty() {
        app.list_area.set(None);
        let empty = Paragraph::new(crate::tui_text!(
            " No switchable Codex accounts detected",
            " 未检测到可切换的 Codex 账号"
        ))
        .style(theme::muted_style())
        .alignment(Alignment::Left);
        f.render_widget(empty, inner);
        return;
    }

    if inner.height == 0 {
        app.list_area.set(Some(inner));
        return;
    }

    let regions = account_list_regions(inner);
    let layout = account_table_layout(regions.header.width);

    app.list_area.set(Some(regions.body));
    render_account_list_header(f, regions.header, &layout);
    render_account_list_rows(f, regions.body, app, &layout);
}

fn account_list_footer_line(app: &CodexAuthApp) -> Line<'static> {
    Line::from(Span::styled(
        crate::tui_format!(
            " Page {}/{} · {} accounts ",
            " 第 {}/{} 页 · {} 个账号 ",
            app.current_page + 1,
            app.total_pages(),
            app.accounts.len()
        ),
        theme::muted_style(),
    ))
}

fn account_list_regions(inner: Rect) -> AccountListRegions {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(u16::from(inner.height >= 2)),
            Constraint::Min(0),
        ])
        .split(inner);

    AccountListRegions {
        header: chunks[0],
        body: chunks[1],
    }
}

fn account_table_layout(inner_width: u16) -> AccountTableLayout {
    if inner_width < 64 {
        return AccountTableLayout::new(
            vec![AccountColumn::Account, AccountColumn::QuotaSummary],
            vec![Constraint::Length(20), Constraint::Min(16)],
            inner_width,
        );
    }

    if inner_width < 96 {
        return AccountTableLayout::new(
            vec![
                AccountColumn::Account,
                AccountColumn::Email,
                AccountColumn::HourlyQuota,
                AccountColumn::WeeklyQuota,
            ],
            vec![
                Constraint::Length(20),
                Constraint::Min(22),
                Constraint::Length(12),
                Constraint::Length(12),
            ],
            inner_width,
        );
    }

    AccountTableLayout::new(
        vec![
            AccountColumn::Account,
            AccountColumn::Email,
            AccountColumn::Plan,
            AccountColumn::HourlyQuota,
            AccountColumn::WeeklyQuota,
            AccountColumn::ExpiresAt,
        ],
        vec![
            Constraint::Length(20),
            Constraint::Min(22),
            Constraint::Length(10),
            Constraint::Length(12),
            Constraint::Length(12),
            Constraint::Length(12),
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
                .fg(theme::subtext())
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
    let selected_style = theme::selected_row_style();

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

            Row::new(layout.columns.iter().map(|column| {
                account_cell(account, app, *column, layout, idx == app.selected_index)
            }))
            .style(row_style)
            .height(1)
        });

    let table = Table::new(rows, layout.widths.clone()).column_spacing(ACCOUNT_COLUMN_SPACING);
    f.render_widget(table, area);
}

fn account_header_cell(column: &AccountColumn) -> Cell<'static> {
    let label = match column {
        AccountColumn::Account => crate::tui_text!("Account", "账号"),
        AccountColumn::Email => crate::tui_text!("Email", "邮箱"),
        AccountColumn::Plan => crate::tui_text!("Plan", "属性"),
        AccountColumn::QuotaSummary => crate::tui_text!("Quota", "配额"),
        AccountColumn::HourlyQuota => "5h",
        AccountColumn::WeeklyQuota => "7d",
        AccountColumn::ExpiresAt => crate::tui_text!("Refresh", "刷新"),
    };

    Cell::from(label.to_string())
}

fn account_cell(
    account: &ccr_cli::models::CodexAuthItem,
    app: &CodexAuthApp,
    column: AccountColumn,
    layout: &AccountTableLayout,
    is_selected: bool,
) -> Cell<'static> {
    match column {
        AccountColumn::Account => {
            let name_style = if is_selected {
                theme::selected_row_style()
            } else if account.is_virtual {
                theme::warning_style().add_modifier(Modifier::ITALIC)
            } else if account.is_current {
                theme::success_style()
            } else {
                Style::default().fg(theme::text())
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
            Cell::from(Line::from(Span::styled(
                email,
                if is_selected {
                    theme::selected_row_style()
                } else {
                    theme::info_style()
                },
            )))
        }
        AccountColumn::Plan => {
            let (property, property_style) = account_property_display(app, account);
            let property = truncate_text(&property, layout.text_width(AccountColumn::Plan));
            Cell::from(Line::from(Span::styled(
                property,
                if is_selected {
                    theme::selected_row_style()
                } else {
                    property_style
                },
            )))
        }
        AccountColumn::QuotaSummary => {
            let five = app.preview_cell_for_account(&account.name, PreviewMetricWindow::FiveHour);
            let seven = app.preview_cell_for_account(&account.name, PreviewMetricWindow::SevenDay);
            let reset = app.preview_reset_cell_for_account(&account.name);
            let summary_style = preview_summary_style(&five, &seven, is_selected);
            let summary_text = format!("{}/{}·{}", five.text, seven.text, reset.text);
            Cell::from(Line::from(Span::styled(
                truncate_text(
                    &summary_text,
                    layout.text_width(AccountColumn::QuotaSummary),
                ),
                summary_style,
            )))
        }
        AccountColumn::HourlyQuota => {
            let cell = app.preview_cell_for_account(&account.name, PreviewMetricWindow::FiveHour);
            let reset = reset_duration_text(app, &account.name, QuotaWindow::Hourly);
            let text = compose_quota_cell_text(&cell.text, reset.as_deref());
            Cell::from(Line::from(Span::styled(
                truncate_text(&text, layout.text_width(AccountColumn::HourlyQuota)),
                preview_cell_style(&cell, is_selected),
            )))
        }
        AccountColumn::WeeklyQuota => {
            let cell = app.preview_cell_for_account(&account.name, PreviewMetricWindow::SevenDay);
            let reset = reset_duration_text(app, &account.name, QuotaWindow::Weekly);
            let text = compose_quota_cell_text(&cell.text, reset.as_deref());
            Cell::from(Line::from(Span::styled(
                truncate_text(&text, layout.text_width(AccountColumn::WeeklyQuota)),
                preview_cell_style(&cell, is_selected),
            )))
        }
        AccountColumn::ExpiresAt => {
            let (text, style) = format_expires_at(account);
            Cell::from(Line::from(Span::styled(
                text,
                if is_selected {
                    theme::selected_row_style()
                } else {
                    style
                },
            )))
        }
    }
}

#[derive(Clone, Copy)]
enum QuotaWindow {
    Hourly,
    Weekly,
}

/// 从 preview 缓存读取指定窗口的 reset 时间戳,并转成形如 `3h11m` / `2d3h` 的短字符串。
/// 仅用于行内 5h / 7d 单元格。quota 尚未缓存时返回 `None`。
fn reset_duration_text(
    app: &CodexAuthApp,
    account_name: &str,
    window: QuotaWindow,
) -> Option<String> {
    let quota = app
        .preview_quota_for_account(account_name)?
        .quota
        .as_ref()?;
    let ts = match window {
        QuotaWindow::Hourly if quota.hourly_window_present == Some(true) => {
            quota.hourly_reset_time?
        }
        QuotaWindow::Weekly if quota.weekly_window_present == Some(true) => {
            quota.weekly_reset_time?
        }
        _ => return None,
    };
    Some(CodexQuotaService::format_reset_duration(ts))
}

/// 把百分比文案 (`"52%"` / `"ERR"` / `"…"`) 与重置时间组合为 `"52% (3h11m)"`。
/// 仅在百分比文案是正常就绪值且 reset 存在时拼括号;其它状态(加载/错误)直接返回原文本。
fn compose_quota_cell_text(percent_text: &str, reset: Option<&str>) -> String {
    match reset {
        Some(reset) if !reset.is_empty() && !matches!(percent_text, "-" | "…" | "1s…" | "ERR") =>
        {
            format!("{percent_text} ({reset})")
        }
        _ => percent_text.to_string(),
    }
}

fn preview_summary_style(
    left: &super::app::QuotaPreviewCell,
    right: &super::app::QuotaPreviewCell,
    is_selected: bool,
) -> Style {
    if is_selected {
        return theme::selected_row_style();
    }

    match (left.state, right.state) {
        (QuotaPreviewCellState::Error, _) | (_, QuotaPreviewCellState::Error) => {
            theme::error_style()
        }
        (QuotaPreviewCellState::Waiting, _) | (_, QuotaPreviewCellState::Waiting) => {
            theme::warning_style()
        }
        (QuotaPreviewCellState::Loading, _) | (_, QuotaPreviewCellState::Loading) => {
            theme::muted_style()
        }
        (QuotaPreviewCellState::Ready, QuotaPreviewCellState::Ready) => {
            if left.text == "ERR" || right.text == "ERR" {
                theme::error_style()
            } else {
                Style::default().fg(theme::text())
            }
        }
        _ => theme::muted_style(),
    }
}

fn preview_cell_style(cell: &super::app::QuotaPreviewCell, is_selected: bool) -> Style {
    if is_selected {
        return theme::selected_row_style();
    }

    match cell.state {
        QuotaPreviewCellState::Ready => {
            let percentage = cell
                .text
                .trim_end_matches('%')
                .parse::<i32>()
                .ok()
                .map(theme::quota_color)
                .unwrap_or(theme::text());
            Style::default().fg(percentage)
        }
        QuotaPreviewCellState::Waiting => theme::warning_style(),
        QuotaPreviewCellState::Loading | QuotaPreviewCellState::Empty => theme::muted_style(),
        QuotaPreviewCellState::Error => theme::error_style(),
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

fn pad_text(value: &str, width: usize) -> String {
    let value_width = value.width();
    if value_width >= width {
        return value.to_string();
    }
    let mut result = String::with_capacity(value.len() + width - value_width);
    result.push_str(value);
    result.extend(std::iter::repeat_n(' ', width - value_width));
    result
}

fn login_status_style(login_state: &ccr_cli::models::LoginState) -> Style {
    match login_state {
        ccr_cli::models::LoginState::NotLoggedIn | ccr_cli::models::LoginState::Unknown { .. } => {
            theme::error_style()
        }
        ccr_cli::models::LoginState::LoggedInUnsaved => theme::warning_style(),
        ccr_cli::models::LoginState::LoggedInSaved(_) => theme::success_style(),
        ccr_cli::models::LoginState::ApiKeyActive
        | ccr_cli::models::LoginState::ProviderKeyActive { .. } => theme::info_style(),
    }
}

fn format_saved_at(account: &ccr_cli::models::CodexAuthItem) -> String {
    account
        .saved_at
        .map(|ts| ts.with_timezone(&Local).format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "-".to_string())
}

fn format_expires_at(account: &ccr_cli::models::CodexAuthItem) -> (String, Style) {
    match account.last_refresh {
        Some(ts) => {
            let text = ts.with_timezone(&Local).format("%Y-%m-%d").to_string();
            (text, theme::info_style())
        }
        None => ("-".to_string(), theme::muted_style()),
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
        (crate::tui_text!("Ready", "就绪"), theme::success_style())
    };

    let status = Paragraph::new(message).style(style).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::border()))
            .title(crate::tui_text!(" Status ", " 状态 "))
            .title_style(Style::default().fg(theme::codex())),
    );

    f.render_widget(status, area);
}

/// The legacy renderer and embedded compact layout share the same content.
fn draw_usage_panel(f: &mut Frame, area: Rect, app: &CodexAuthApp) {
    draw_combined_panel(f, area, app);
}

fn clipped_line(line: Line<'static>, width: usize) -> Line<'static> {
    if line.width() <= width {
        return line;
    }
    let mut remaining = width.saturating_sub(1);
    let mut spans = Vec::new();
    let mut ellipsis_style = line.style;
    for span in line.spans {
        ellipsis_style = span.style;
        let mut content = String::new();
        for ch in span.content.chars() {
            let columns = ch.width().unwrap_or(0);
            if columns > remaining {
                break;
            }
            content.push(ch);
            remaining -= columns;
        }
        let complete = content == span.content;
        spans.push(Span::styled(content, span.style));
        if !complete || remaining == 0 {
            break;
        }
    }
    if width > 0 {
        spans.push(Span::styled("…", ellipsis_style));
    }
    Line::from(spans).style(line.style)
}

fn render_detail_card(f: &mut Frame, area: Rect, title: &'static str, lines: Vec<Line<'static>>) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::border()))
        .title(title)
        .title_style(theme::codex_style());
    let inner = block.inner(area);
    f.render_widget(block, area);
    f.render_widget(
        Paragraph::new(
            lines
                .into_iter()
                .map(|line| clipped_line(line, usize::from(inner.width)))
                .collect::<Vec<_>>(),
        ),
        inner,
    );
}

/// Clamp remaining quota once at the presentation boundary, before filling cells.
fn quota_window_line(
    label: &str,
    percentage: i32,
    present: Option<bool>,
    reset: Option<i64>,
    width: usize,
) -> Line<'static> {
    let mut spans = vec![Span::styled(format!("{label} "), theme::info_style())];
    if present != Some(true) {
        spans.push(Span::styled(
            if present == Some(false) {
                crate::tui_text!("Not provided", "未提供")
            } else {
                crate::tui_text!("Unknown window", "窗口未知")
            },
            theme::muted_style(),
        ));
        return Line::from(spans);
    }
    let percentage = percentage.clamp(0, 100);
    let color = Style::default().fg(theme::quota_color(percentage));
    let relative = if width >= 64 {
        CodexAuthApp::quota_reset_detail_text(reset)
    } else {
        reset
            .map(CodexQuotaService::format_reset_duration)
            .unwrap_or_else(|| "-".to_string())
    };
    let reset_text = crate::tui_format!(" Reset {}", " 重置 {}", relative);
    let fixed_width = label.width() + 1 + 5 + reset_text.width();
    let bar_width = width.saturating_sub(fixed_width + 1).min(20);
    if bar_width >= 3 {
        let filled = percentage as usize * bar_width / 100;
        spans.push(Span::styled("█".repeat(filled), color));
        spans.push(Span::styled(
            "░".repeat(bar_width - filled),
            theme::muted_style(),
        ));
        spans.push(Span::raw(" "));
    }
    spans.push(Span::styled(format!("{percentage:>3}%"), color));
    spans.push(Span::styled(reset_text, theme::muted_style()));
    clipped_line(Line::from(spans), width)
}

fn quota_lines(app: &CodexAuthApp, width: usize) -> Vec<Line<'static>> {
    if let Some(quota) = app.selected_quota().and_then(|entry| entry.quota.as_ref()) {
        return vec![
            quota_window_line(
                "5h",
                quota.hourly_percentage,
                quota.hourly_window_present,
                quota.hourly_reset_time,
                width,
            ),
            quota_window_line(
                "7d",
                quota.weekly_percentage,
                quota.weekly_window_present,
                quota.weekly_reset_time,
                width,
            ),
        ];
    }
    let preview_loading = app.selected_account().is_some_and(|account| {
        app.preview_cell_for_account(&account.name, PreviewMetricWindow::FiveHour)
            .state
            == QuotaPreviewCellState::Loading
    });
    let (text, style) = if app.selected_quota_error().is_some() {
        (
            crate::tui_text!("Unavailable", "不可用"),
            theme::error_style(),
        )
    } else if app.is_selected_quota_loading() || preview_loading {
        (
            crate::tui_text!("Loading…", "加载中…"),
            theme::muted_style(),
        )
    } else if app.is_activation_gate_pending() {
        (
            crate::tui_text!("Waiting…", "等待中…"),
            theme::muted_style(),
        )
    } else {
        (
            crate::tui_text!("No cached quota", "暂无配额缓存"),
            theme::muted_style(),
        )
    };
    ["5h", "7d"]
        .into_iter()
        .map(|label| {
            Line::from(vec![
                Span::styled(format!("{label} "), theme::info_style()),
                Span::styled(text, style),
            ])
        })
        .collect()
}

fn quota_status_line(app: &CodexAuthApp) -> Line<'static> {
    let cached = app
        .selected_quota()
        .filter(|entry| entry.quota.is_some())
        .map(|entry| {
            crate::tui_format!(
                "Cached {}",
                "缓存 {}",
                entry.fetched_at.with_timezone(&Local).format("%m/%d %H:%M")
            )
        });
    let mut spans = Vec::new();
    if app.pending_quota_confirm {
        spans.push(Span::styled(
            crate::tui_text!(
                "Query quota? y confirm / any key cancel",
                "查询配额？y 确认 / 其他键取消"
            ),
            theme::warning_style(),
        ));
    } else if let Some(error) = app.selected_quota_error() {
        let text = match relogin_required_detail(error) {
            Some(detail) => crate::tui_format!(
                "Quota error: re-login required (refresh token revoked or used); press o before codex login · {}",
                "配额错误：需重新登录（refresh token 已吊销或已使用）；codex login 前先按 o 本地登出 · {}",
                detail
            ),
            None => crate::tui_format!("Quota error: {}", "配额错误：{}", error),
        };
        spans.push(Span::styled(text, theme::error_style()));
    } else if app.is_selected_quota_loading() {
        spans.push(Span::styled(
            crate::tui_text!("Refreshing", "刷新中"),
            theme::info_style(),
        ));
    } else if app.is_quota_preview_loading() {
        spans.push(Span::styled(
            crate::tui_text!("Refreshing previews", "速览刷新中"),
            theme::info_style(),
        ));
    }
    if let Some(cached) = cached {
        // Keep the timestamp visible even when a raw error needs truncation.
        if !spans.is_empty() {
            spans.insert(
                0,
                Span::styled(format!("{cached} · "), theme::muted_style()),
            );
        } else {
            spans.push(Span::styled(cached, theme::muted_style()));
        }
    }
    Line::from(spans)
}

fn format_compact_count(value: u64) -> String {
    const UNITS: [&str; 5] = ["", "K", "M", "B", "T"];
    let mut scaled = value as f64;
    let mut unit = 0;
    while scaled >= 1000.0 && unit < UNITS.len() - 1 {
        scaled /= 1000.0;
        unit += 1;
    }
    if unit == 0 {
        return value.to_string();
    }
    scaled = (scaled * 10.0).round() / 10.0;
    if scaled >= 1000.0 && unit < UNITS.len() - 1 {
        scaled /= 1000.0;
        unit += 1;
    }
    if scaled.fract() == 0.0 {
        format!("{scaled:.0}{}", UNITS[unit])
    } else {
        format!("{scaled:.1}{}", UNITS[unit])
    }
}

fn format_api_usd(value: Option<f64>) -> String {
    match value {
        Some(value) if value.is_finite() && value > 0.0 && value < 0.01 => "<$0.01".into(),
        Some(value) if value.is_finite() && value >= 0.0 => format!("${value:.2}"),
        _ => "N/A".into(),
    }
}

/// 只渲染有值的子类；括号内为包含关系（输入含缓存读写，输出含推理）
fn token_total_with_parts(total: u64, parts: &[(&str, Option<u64>)]) -> String {
    let parts: Vec<String> = parts
        .iter()
        .filter_map(|(label, value)| {
            value.map(|value| format!("{label} {}", format_compact_count(value)))
        })
        .collect();
    if parts.is_empty() {
        format_compact_count(total)
    } else {
        crate::tui_format!(
            "{} ({})",
            "{}（{}）",
            format_compact_count(total),
            parts.join(" · ")
        )
    }
}

fn token_classification_lines(panel: &CodexAuthUsagePanelData) -> Vec<Line<'static>> {
    let rows = [
        ("5h", &panel.rolling.five_hour),
        ("7d", &panel.rolling.seven_day),
        (crate::tui_text!("All", "累计"), &panel.rolling.all_time),
    ];
    // 全部子类缺失时整区省略，不渲染 N/A 占位行
    if rows.iter().all(|(_, usage)| {
        usage.details.cache_read_tokens.is_none()
            && usage.details.cache_write_tokens.is_none()
            && usage.details.reasoning_tokens.is_none()
    }) {
        return Vec::new();
    }
    rows.into_iter()
        .map(|(label, usage)| {
            let input = token_total_with_parts(
                usage.total_input_tokens,
                &[
                    (
                        crate::tui_text!("Read", "读"),
                        usage.details.cache_read_tokens,
                    ),
                    (
                        crate::tui_text!("Write", "写"),
                        usage.details.cache_write_tokens,
                    ),
                ],
            );
            let output = token_total_with_parts(
                usage.total_output_tokens,
                &[(
                    crate::tui_text!("Reasoning", "推理"),
                    usage.details.reasoning_tokens,
                )],
            );
            Line::from(Span::styled(
                crate::tui_format!(
                    "{} In {} · Out {}",
                    "{} 输入{} · 输出{}",
                    label,
                    input,
                    output
                ),
                theme::info_style(),
            ))
        })
        .collect()
}

fn usage_scope_line(panel: &CodexAuthUsagePanelData) -> Line<'static> {
    let (scope, style) = match &panel.scope {
        CodexUsageScope::AccountAttributed { account_name } => (
            crate::tui_format!(
                "Local: account {} · inferred",
                "本地：账号 {} · 推断归属",
                truncate_text(account_name, 20)
            ),
            theme::info_style(),
        ),
        CodexUsageScope::GlobalRuntime => (
            if panel.attribution_state == CodexUsageAttributionState::GlobalOnly {
                crate::tui_text!("Local: global", "本地：全局").to_string()
            } else {
                crate::tui_format!(
                    "Local: global (not selected) · {}",
                    "本地：全局（非所选账号）· {}",
                    panel.fallback_reason.as_deref().unwrap_or("-")
                )
            },
            usage_attribution_style(panel.attribution_state),
        ),
    };
    Line::from(Span::styled(scope, style))
}

fn cost_status_label(cost: &CodexCostSummary) -> &'static str {
    match cost.status {
        CodexCostStatus::Unpriced => crate::tui_text!("UNPRICED", "未定价"),
        CodexCostStatus::Partial => crate::tui_text!("partial", "部分"),
        CodexCostStatus::AssumedStandard => crate::tui_text!("assumed", "假设"),
        CodexCostStatus::CompletePriced => crate::tui_text!("priced", "已估值"),
    }
}

fn price_mode_label(cost: &CodexCostSummary) -> &'static str {
    match cost.tier_basis.as_str() {
        "fast" => "Fast",
        "standard" => "Standard",
        "mixed" => crate::tui_text!("mixed", "混合"),
        _ => crate::tui_text!("Std assumed", "Std假设"),
    }
}

fn cost_severity(status: CodexCostStatus) -> u8 {
    match status {
        CodexCostStatus::Unpriced => 3,
        CodexCostStatus::Partial => 2,
        CodexCostStatus::AssumedStandard => 1,
        CodexCostStatus::CompletePriced => 0,
    }
}

fn cost_is_warning(cost: &CodexCostSummary) -> bool {
    matches!(
        cost.status,
        CodexCostStatus::Unpriced | CodexCostStatus::Partial
    )
}

/// 计价状态徽标：取各窗口最严重状态（unpriced > partial > Std assumed > priced），面板内只出现一次
fn cost_badge(panel: &CodexAuthUsagePanelData) -> Span<'static> {
    let cost = [
        &panel.rolling.seven_day.details.cost,
        &panel.rolling.five_hour.details.cost,
    ]
    .into_iter()
    .fold(&panel.rolling.all_time.details.cost, |worst, cost| {
        if cost_severity(cost.status) > cost_severity(worst.status) {
            cost
        } else {
            worst
        }
    });
    let text = match cost.status {
        CodexCostStatus::Unpriced => cost_status_label(cost).to_string(),
        CodexCostStatus::Partial => crate::tui_format!(
            "partly priced {}/{} · {}",
            "部分估值 {}/{} · {}",
            cost.priced_records,
            cost.total_records,
            price_mode_label(cost)
        ),
        _ => price_mode_label(cost).to_string(),
    };
    Span::styled(
        text,
        if cost_is_warning(cost) {
            theme::warning_style()
        } else {
            theme::info_style()
        },
    )
}

fn row_cost(cost: &CodexCostSummary) -> String {
    if cost.status == CodexCostStatus::Unpriced {
        "N/A".into()
    } else {
        format_api_usd(cost.subtotal_usd)
    }
}

fn estimate_status_label(status: CodexEstimateStatus) -> &'static str {
    match status {
        CodexEstimateStatus::LocalEstimate => crate::tui_text!("local estimate", "本地估算"),
        CodexEstimateStatus::InsufficientSamples => crate::tui_text!("samples<3", "样本不足"),
        CodexEstimateStatus::PartialUsage => crate::tui_text!("partial usage", "用量不完整"),
        CodexEstimateStatus::InvalidScope => crate::tui_text!("scope", "范围不匹配"),
        CodexEstimateStatus::UnsupportedWindow => crate::tui_text!("window", "未知窗口"),
        CodexEstimateStatus::ResetChanged => crate::tui_text!("reset changed", "重置变更"),
        CodexEstimateStatus::Stale => crate::tui_text!("stale", "已过期"),
        CodexEstimateStatus::Unpriced => crate::tui_text!("unpriced", "未定价"),
        CodexEstimateStatus::Unstable => crate::tui_text!("unstable", "不稳定"),
        CodexEstimateStatus::HistoryError => crate::tui_text!("history error", "历史错误"),
        CodexEstimateStatus::UnexplainedQuotaChange => {
            crate::tui_text!("unexplained change", "变化原因未查明")
        }
    }
}

fn token_range(value: Option<CodexEstimateRange>) -> String {
    value
        .map(|value| {
            format!(
                "{} [{}–{}]",
                format_compact_count(value.median.max(0.0) as u64),
                format_compact_count(value.min.max(0.0) as u64),
                format_compact_count(value.max.max(0.0) as u64)
            )
        })
        .unwrap_or_else(|| "N/A".into())
}

fn usd_range(value: Option<CodexEstimateRange>) -> String {
    value
        .map(|value| {
            format!(
                "{} [{}–{}]",
                format_api_usd(Some(value.median)),
                format_api_usd(Some(value.min)),
                format_api_usd(Some(value.max))
            )
        })
        .unwrap_or_else(|| "N/A".into())
}

fn compact_capacity(estimate: &CodexCapacityEstimate) -> String {
    if estimate.status != CodexEstimateStatus::LocalEstimate
        && estimate.status == estimate.usd_status
    {
        return format!("N/A:{}", estimate_status_label(estimate.status));
    }
    let token = if estimate.status == CodexEstimateStatus::LocalEstimate {
        crate::tui_format!(
            "est {}",
            "估算{}",
            estimate
                .token_remaining
                .map(|value| format_compact_count(value.median.max(0.0) as u64))
                .unwrap_or_else(|| "N/A".into()),
        )
    } else {
        format!("N/A:{}", estimate_status_label(estimate.status))
    };
    let usd = if estimate.usd_status == CodexEstimateStatus::LocalEstimate {
        let amount = format_api_usd(estimate.usd_remaining.map(|value| value.median));
        if estimate.status == CodexEstimateStatus::LocalEstimate {
            amount
        } else {
            crate::tui_format!("est {}", "估算{}", amount)
        }
    } else {
        format!("N/A:{}", estimate_status_label(estimate.usd_status))
    };
    format!("{token} / {usd}")
}

fn compact_usage_lines(
    panel: &CodexAuthUsagePanelData,
    inline_capacity: bool,
) -> Vec<Line<'static>> {
    [
        (
            "5h",
            &panel.rolling.five_hour,
            Some(&panel.estimate.five_hour),
        ),
        (
            "7d",
            &panel.rolling.seven_day,
            Some(&panel.estimate.seven_day),
        ),
        (
            crate::tui_text!("All", "累计"),
            &panel.rolling.all_time,
            None,
        ),
    ]
    .into_iter()
    .map(|(label, usage, estimate)| {
        let mut text = crate::tui_format!(
            "{} {} {}r {}",
            "{} {} {}条 {}",
            label,
            format_compact_count(usage.total_tokens()),
            format_compact_count(usage.total_requests),
            row_cost(&usage.details.cost),
        );
        if inline_capacity && let Some(estimate) = estimate {
            text.push_str(&crate::tui_format!(
                " rem {}",
                " 剩余{}",
                compact_capacity(estimate)
            ));
        }
        if estimate.is_none() {
            text.push_str(crate::tui_text!(" · folded", " · 折叠"));
        }
        Line::from(Span::styled(
            text,
            if cost_is_warning(&usage.details.cost) {
                theme::warning_style()
            } else {
                theme::info_style()
            },
        ))
    })
    .collect()
}

fn usage_table_lines(panel: &CodexAuthUsagePanelData, header: bool) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let label_width = 8;
    if header {
        lines.push(Line::from(Span::styled(
            format!(
                "{} {:>8}  {}{}  API USD",
                pad_text(crate::tui_text!("Window", "时段"), label_width),
                "Tokens",
                " ".repeat(8usize.saturating_sub(crate::tui_text!("Records", "记录数").width())),
                crate::tui_text!("Records", "记录数")
            ),
            theme::muted_style(),
        )));
    }
    for (label, usage) in [
        ("5h", &panel.rolling.five_hour),
        ("7d", &panel.rolling.seven_day),
        (
            crate::tui_text!("All time", "累计"),
            &panel.rolling.all_time,
        ),
    ] {
        lines.push(Line::from(vec![
            Span::styled(pad_text(label, label_width), theme::muted_style()),
            Span::styled(
                format!(
                    " {:>8}  {:>8}  ",
                    format_compact_count(
                        usage
                            .total_input_tokens
                            .saturating_add(usage.total_output_tokens)
                    ),
                    format_compact_count(usage.total_requests)
                ),
                Style::default().fg(theme::text()),
            ),
            Span::styled(
                format!("{:>8}", row_cost(&usage.details.cost)),
                if cost_is_warning(&usage.details.cost) {
                    theme::warning_style()
                } else {
                    theme::info_style()
                },
            ),
        ]));
    }
    lines
}

/// compact 路径无表头时的单行计价徽标
fn cost_badge_line(panel: &CodexAuthUsagePanelData) -> Line<'static> {
    Line::from(vec![
        Span::styled("API ", theme::muted_style()),
        cost_badge(panel),
    ])
}

fn has_capacity_estimate(estimate: &CodexCapacityEstimate) -> bool {
    estimate.status == CodexEstimateStatus::LocalEstimate
        || estimate.usd_status == CodexEstimateStatus::LocalEstimate
}

fn joint_capacity_line(panel: &CodexAuthUsagePanelData) -> Line<'static> {
    let joint = &panel.estimate.joint;
    let text = if joint.token_remaining.is_none() && joint.usd_remaining.is_none() {
        crate::tui_format!(
            "Joint remaining: N/A ({})",
            "联合剩余：N/A（{}）",
            estimate_status_label(joint.status),
        )
    } else {
        crate::tui_format!(
            "Joint remaining ≈ {} / {} ({})",
            "联合剩余约{} / {}（{}）",
            token_range(joint.token_remaining),
            usd_range(joint.usd_remaining),
            estimate_status_label(joint.status),
        )
    };
    Line::from(Span::styled(text, theme::info_style()))
}

fn capacity_lines(panel: &CodexAuthUsagePanelData) -> Vec<Line<'static>> {
    let windows = [
        ("5h", &panel.estimate.five_hour),
        ("7d", &panel.estimate.seven_day),
    ];
    if !windows
        .iter()
        .any(|(_, estimate)| has_capacity_estimate(estimate))
    {
        // 无估算时折叠：同一状态合并为一行，联合剩余同状态时并入
        let status = panel.estimate.five_hour.status;
        let uniform = windows
            .iter()
            .all(|(_, estimate)| estimate.status == status && estimate.usd_status == status);
        if !uniform {
            let mut lines: Vec<Line<'static>> = windows
                .into_iter()
                .map(|(label, estimate)| {
                    Line::from(Span::styled(
                        crate::tui_format!(
                            "{} rem {}",
                            "{} 剩余{}",
                            label,
                            compact_capacity(estimate)
                        ),
                        theme::muted_style(),
                    ))
                })
                .collect();
            lines.push(joint_capacity_line(panel));
            return lines;
        }
        let joint = &panel.estimate.joint;
        if joint.status == status
            && joint.token_remaining.is_none()
            && joint.usd_remaining.is_none()
        {
            return vec![Line::from(Span::styled(
                crate::tui_format!(
                    "Capacity 5h/7d/joint rem: N/A ({})",
                    "容量 5h/7d/联合 剩余：N/A（{}）",
                    estimate_status_label(status),
                ),
                theme::muted_style(),
            ))];
        }
        return vec![
            Line::from(Span::styled(
                crate::tui_format!(
                    "Capacity 5h/7d rem: N/A ({})",
                    "容量 5h/7d 剩余：N/A（{}）",
                    estimate_status_label(status),
                ),
                theme::muted_style(),
            )),
            joint_capacity_line(panel),
        ];
    }
    let mut lines = vec![Line::from(Span::styled(
        crate::tui_text!(
            "Capacity · recent local workload estimate",
            "容量 · 按近期本地负载估算"
        ),
        theme::info_style(),
    ))];
    for (label, estimate) in windows {
        let token = if estimate.status == CodexEstimateStatus::LocalEstimate {
            token_range(estimate.token_remaining)
        } else {
            format!("N/A:{}", estimate_status_label(estimate.status))
        };
        let usd = if estimate.usd_status == CodexEstimateStatus::LocalEstimate {
            usd_range(estimate.usd_remaining)
        } else {
            format!("N/A:{}", estimate_status_label(estimate.usd_status))
        };
        lines.push(Line::from(Span::styled(
            crate::tui_format!("{} rem {} / {}", "{} 剩余{} / {}", label, token, usd),
            theme::info_style(),
        )));
        // 样本为 0 时不渲染样本/跨度明细
        if estimate.sample_count == 0 && estimate.usd_sample_count == 0 {
            continue;
        }
        let mut detail = crate::tui_format!(
            "   total {} / {} · samples {}/{}",
            "   总量{} / {} · 样本{}/{}",
            estimate
                .token_total
                .map(|value| format_compact_count(value.median.max(0.0) as u64))
                .unwrap_or_else(|| "N/A".into()),
            format_api_usd(estimate.usd_total.map(|value| value.median)),
            estimate.sample_count,
            estimate.usd_sample_count,
        );
        if let (Some(start), Some(end)) = (estimate.span_start, estimate.span_end) {
            detail.push_str(&format!(
                " · {}–{}",
                start.with_timezone(&Local).format("%m/%d %H:%M"),
                end.with_timezone(&Local).format("%m/%d %H:%M")
            ));
        }
        lines.push(Line::from(Span::styled(detail, theme::muted_style())));
    }
    lines.push(joint_capacity_line(panel));
    lines.push(Line::from(Span::styled(
        crate::tui_text!(
            "Other devices/cloud usage unavailable; no guaranteed capacity",
            "其他设备及云端用量不可见；可用量无保证"
        ),
        theme::muted_style(),
    )));
    lines
}

/// 价格版本、来源与上下文假设：一行放得下时合并，否则来源另起一行；URL 仅在完整可见时渲染
fn price_source_lines(cost: &CodexCostSummary, width: usize) -> Vec<Line<'static>> {
    let mut price = crate::tui_format!("Price {}", "价格{}", cost.price_version);
    if cost.context_assumption {
        price.push_str(crate::tui_text!(" · context assumed", " · 上下文假设"));
    }
    let mut provenance = Vec::new();
    if cost.pricing_sources.contains("official_verified") {
        provenance.push(crate::tui_format!(
            "gpt-6.1-sol verified {}",
            "gpt-6.1-sol核实{}",
            cost.verified_date.as_deref().unwrap_or("N/A"),
        ));
    }
    if cost.pricing_sources.contains("catalog_estimate") {
        provenance.push(crate::tui_text!("catalog estimate; date N/A", "目录估价；日期N/A").into());
    }
    if provenance.is_empty() {
        provenance.push(crate::tui_text!("provenance N/A", "来源N/A").into());
    }
    let provenance = provenance.join(" · ");
    let combined = format!("{price} · {provenance}");
    let mut lines = if combined.width() <= width {
        vec![combined]
    } else {
        vec![price, provenance]
    };
    if let Some(source) = &cost.source_url {
        let source = crate::tui_format!("Source: {}", "来源：{}", source);
        if source.width() <= width {
            lines.push(source);
        }
    }
    lines
        .into_iter()
        .map(|text| Line::from(Span::styled(text, theme::muted_style())))
        .collect()
}

fn usage_note_line(panel: &CodexAuthUsagePanelData) -> Option<Line<'static>> {
    panel.fallback_reason.as_ref().map(|reason| {
        Line::from(Span::styled(
            reason.clone(),
            if panel.attribution_state == CodexUsageAttributionState::AccountAttributed {
                theme::muted_style()
            } else {
                usage_attribution_style(panel.attribution_state)
            },
        ))
    })
}

fn usage_state_line(app: &CodexAuthApp) -> Line<'static> {
    if let Some(error) = &app.usage_error {
        return Line::from(Span::styled(
            crate::tui_format!("Local usage error: {}", "本地用量错误：{}", error),
            theme::error_style(),
        ));
    }
    let (text, style) = match &app.usage_state {
        UsageState::Error(error) => (
            crate::tui_format!("Local usage error: {}", "本地用量错误：{}", error),
            theme::error_style(),
        ),
        UsageState::NoData => (
            crate::tui_text!("No local usage records", "暂无本地用量记录").to_string(),
            theme::muted_style(),
        ),
        _ => (
            crate::tui_text!("Loading local usage…", "本地用量加载中…").to_string(),
            theme::muted_style(),
        ),
    };
    Line::from(Span::styled(text, style))
}

fn top_model_line(panel: &CodexAuthUsagePanelData) -> Line<'static> {
    Line::from(Span::styled(
        crate::tui_format!(
            "Top model: {}",
            "主要模型：{}",
            panel
                .top_model
                .as_ref()
                .map(|model| model.model.as_str())
                .unwrap_or("-")
        ),
        theme::info_style(),
    ))
}

fn local_usage_lines(app: &CodexAuthApp, budget: usize, width: usize) -> Vec<Line<'static>> {
    let Some(panel) = app.usage_panel_data() else {
        return vec![usage_state_line(app)];
    };
    let mut scope = usage_scope_line(&panel);
    let diagnostics = &panel.estimate.diagnostics;
    if !diagnostics.is_complete() || panel.rolling.all_time.details.partial {
        scope.spans.insert(
            0,
            Span::styled(
                crate::tui_text!("partial · ", "不完整 · "),
                theme::warning_style(),
            ),
        );
    }
    let mut lines = vec![scope];
    if budget < 4 {
        lines.push(Line::from(Span::styled(
            crate::tui_text!(
                "More space needed; statistics omitted",
                "空间不足，统计已省略"
            ),
            theme::muted_style(),
        )));
        return lines;
    }
    let full = budget >= 22;
    let inline_capacity = budget < 6;
    // Each required value occupies its own short row when the panel is narrow.
    // A four-row panel keeps the remaining value beside its matching window.
    let mut badge_below_table = false;
    if full {
        let mut table = usage_table_lines(&panel, true);
        // 徽标完整放得下时并入表头，否则在表格下方单独一行，避免截断状态
        let badge = cost_badge(&panel);
        if table[0].width() + 3 + badge.width() <= width {
            table[0]
                .spans
                .push(Span::styled(" · ", theme::muted_style()));
            table[0].spans.push(badge);
        } else {
            badge_below_table = true;
        }
        lines.extend(table);
    } else {
        lines.extend(compact_usage_lines(&panel, inline_capacity));
    }
    // Errors take precedence over secondary metadata at every supported size.
    if app.usage_error.is_some() {
        lines[0].spans.insert(
            0,
            Span::styled(
                crate::tui_text!("Stale; usage error · ", "旧快照；用量错误 · "),
                theme::error_style(),
            ),
        );
    } else if panel.estimate.history_warning.is_some() {
        lines[0].spans.insert(
            0,
            Span::styled(
                crate::tui_text!("History error · ", "历史错误 · "),
                theme::warning_style(),
            ),
        );
    } else if app.is_usage_refreshing() {
        lines[0].spans.push(Span::styled(
            crate::tui_text!(" · stale/refreshing", " · 旧快照/刷新中"),
            theme::muted_style(),
        ));
    }
    if full {
        if badge_below_table {
            lines.push(cost_badge_line(&panel));
        }
        lines.extend(capacity_lines(&panel));
        lines.extend(token_classification_lines(&panel));
        lines.extend(price_source_lines(
            &panel.rolling.all_time.details.cost,
            width,
        ));
    } else {
        if !inline_capacity {
            for (label, estimate) in [
                ("5h", &panel.estimate.five_hour),
                ("7d", &panel.estimate.seven_day),
            ] {
                lines.push(Line::from(Span::styled(
                    crate::tui_format!("{} rem {}", "{} 剩余{}", label, compact_capacity(estimate)),
                    theme::muted_style(),
                )));
            }
        }
        // 无表头时徽标单独一行；放不下则并入首行，保证计价状态只出现一次且可见
        if lines.len() < budget {
            lines.push(cost_badge_line(&panel));
        } else {
            lines[0]
                .spans
                .push(Span::styled(" · API ", theme::muted_style()));
            lines[0].spans.push(cost_badge(&panel));
        }
    }
    if lines.len() < budget {
        lines.push(top_model_line(&panel));
    }
    if lines.len() < budget
        && let Some(note) = usage_note_line(&panel)
    {
        lines.push(note);
    }
    if !full || lines.len() > budget {
        let omitted = Line::from(Span::styled(
            crate::tui_text!("Details omitted; enlarge terminal", "详情已省略；增大终端"),
            theme::muted_style(),
        ));
        if lines.len() < budget {
            lines.push(omitted);
        } else if full {
            lines.truncate(budget.saturating_sub(1));
            lines.push(omitted);
        }
    }
    // Truncation is explicit and preserves the semantic span styles.
    lines
        .into_iter()
        .map(|line| clipped_line(line, width))
        .collect()
}

fn draw_quota_panel(f: &mut Frame, area: Rect, app: &CodexAuthApp) {
    let mut lines = quota_lines(app, usize::from(area.width.saturating_sub(2)));
    let status = quota_status_line(app);
    if status.width() > 0 {
        lines.push(status);
    }
    render_detail_card(
        f,
        area,
        crate::tui_text!(" Quota remaining ", " 剩余配额 "),
        lines,
    );
}

fn draw_local_usage_panel(f: &mut Frame, area: Rect, app: &CodexAuthApp) {
    render_detail_card(
        f,
        area,
        crate::tui_text!(
            " Local usage · API equivalent USD · Tokens / Records ",
            " 本地用量 · API等值USD · Tokens / 记录数 "
        ),
        local_usage_lines(
            app,
            usize::from(area.height.saturating_sub(2)),
            usize::from(area.width.saturating_sub(2)),
        ),
    );
}

fn draw_combined_panel(f: &mut Frame, area: Rect, app: &CodexAuthApp) {
    let budget = usize::from(area.height.saturating_sub(2));
    let width = usize::from(area.width.saturating_sub(2));
    let mut lines = quota_lines(app, width);
    let status = quota_status_line(app);
    let status_rows = usize::from(status.width() > 0);
    if budget < 6 + status_rows {
        let mut minimal = Vec::new();
        if status_rows > 0 {
            minimal.push(status);
        }
        minimal.append(&mut lines);
        minimal.push(
            app.usage_panel_data()
                .map(|panel| usage_scope_line(&panel))
                .unwrap_or_else(|| usage_state_line(app)),
        );
        minimal.truncate(budget.saturating_sub(1));
        minimal.push(Line::from(Span::styled(
            crate::tui_text!(
                "More space needed; statistics omitted",
                "空间不足，统计已省略"
            ),
            theme::muted_style(),
        )));
        render_detail_card(
            f,
            area,
            crate::tui_text!(" Quota remaining · Local usage ", " 剩余配额 · 本地用量 "),
            minimal,
        );
        return;
    }
    // Six rows are the indivisible quota + scope + three-statistic core.
    // Below it, explicitly omit numbers instead of losing their scope.
    let local_budget = budget.saturating_sub(2 + status_rows);
    lines.extend(local_usage_lines(app, local_budget, width));
    if status_rows > 0 {
        lines.push(status);
    }
    if lines.len() > budget {
        lines.truncate(budget.saturating_sub(1));
        lines.push(Line::from(Span::styled(
            crate::tui_text!("More space needed; details omitted", "空间不足，详情已省略"),
            theme::muted_style(),
        )));
    }
    render_detail_card(
        f,
        area,
        crate::tui_text!(
            " Quota / Local usage · API equivalent USD ",
            " 剩余配额 / 本地用量 · API等值USD "
        ),
        lines,
    );
}

fn account_snapshot_lines(
    app: &CodexAuthApp,
    account: &ccr_cli::models::CodexAuthItem,
) -> Vec<Line<'static>> {
    let account_style = if account.is_current {
        theme::success_style()
    } else if account.is_virtual {
        theme::warning_style().add_modifier(Modifier::ITALIC)
    } else {
        Style::default()
            .fg(theme::text())
            .add_modifier(Modifier::BOLD)
    };
    let state_style = if account.is_current {
        theme::success_style()
    } else if account.is_virtual {
        theme::warning_style()
    } else {
        Style::default().fg(theme::text())
    };
    let (refresh_text, refresh_style) = format_expires_at(account);
    let (plan_text, plan_style) = quota_plan_for_account(app, account)
        .map(|plan| (plan, theme::info_style()))
        .unwrap_or_else(|| ("-".to_string(), theme::muted_style()));

    vec![
        detail_line("Account:", account.name.clone(), account_style),
        detail_line(
            "State:",
            format!(
                "{}{}",
                if account.is_current {
                    crate::tui_text!("Current", "当前")
                } else {
                    crate::tui_text!("Saved", "已保存")
                },
                if account.is_virtual {
                    crate::tui_text!(" · Virtual", " · 临时")
                } else {
                    ""
                }
            ),
            state_style,
        ),
        detail_optional_line("Email:", account.email.as_deref(), theme::info_style()),
        detail_line("Plan:", plan_text, plan_style),
        detail_line(
            "Saved at:",
            format_saved_at(account),
            Style::default().fg(theme::text()),
        ),
        detail_line("Last refresh:", refresh_text, refresh_style),
    ]
}

/// Draw help bar (overlay-aware)
fn draw_help_bar(f: &mut Frame, area: Rect, app: &CodexAuthApp) {
    let help_text = match &app.overlay {
        Some(Overlay::Confirm { .. }) => {
            crate::tui_text!("y confirm delete | n/Esc cancel", "y 确认删除 | n/Esc 取消")
        }
        Some(Overlay::Input { .. }) => {
            crate::tui_text!("Enter confirm | Esc cancel", "Enter 确认 | Esc 取消")
        }
        Some(Overlay::RenameInput { .. }) => crate::tui_text!(
            "Enter save | Ctrl+F overwrite | Esc cancel",
            "Enter 保存 | Ctrl+F 强制覆盖 | Esc 取消"
        ),
        None => crate::tui_text!(
            "↑/k up | ↓/j down | Enter switch | s save current | n rename | d delete | o auth off | r refresh | R repair | b quota | Ctrl+L language | q quit",
            "↑/k 上移 | ↓/j 下移 | Enter 切换 | s 保存当前 | n 重命名 | d 删除 | o 登出 | r 刷新 | R 修复 | b 配额 | Ctrl+L 语言 | q 退出"
        ),
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
    app: &mut CodexAuthApp,
    content_area: Rect,
    footer_area: Rect,
    mode: crate::tui::theme::ViewportMode,
) {
    match mode {
        crate::tui::theme::ViewportMode::Compact | crate::tui::theme::ViewportMode::Standard => {
            let list_height = if content_area.height <= 12 {
                3
            } else if content_area.height <= 16 {
                4
            } else {
                6
            };
            let content_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(list_height), Constraint::Min(0)])
                .split(content_area);

            draw_account_list_with_status(f, content_chunks[0], app);
            draw_usage_panel(f, content_chunks[1], app);
        }
        crate::tui::theme::ViewportMode::Wide => {
            let columns = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(content_area);

            draw_account_list_with_status(f, columns[0], app);

            if content_area.height < 23 {
                draw_combined_panel(f, columns[1], app);
            } else {
                let right = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(if content_area.height >= 35 { 12 } else { 5 }),
                        Constraint::Min(0),
                    ])
                    .split(columns[1]);
                if content_area.height >= 35 {
                    draw_account_snapshot_panel(f, right[0], app);
                } else {
                    draw_quota_panel(f, right[0], app);
                }
                draw_local_usage_panel(f, right[1], app);
            }
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
        .map(|err| {
            crate::tui_format!(
                "Failed to initialize Codex Auth\n\n{}",
                "Codex 认证初始化失败\n\n{}",
                err
            )
        })
        .unwrap_or_else(|| {
            crate::tui_text!("Initializing Codex Auth...", "正在初始化 Codex 认证...").to_string()
        });

    let panel = Paragraph::new(message)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::border()))
                .title(crate::tui_text!(" Codex Auth ", " Codex 认证 "))
                .title_style(Style::default().fg(theme::codex())),
        )
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    f.render_widget(panel, content_area);

    if mode == crate::tui::theme::ViewportMode::Compact {
        let help = Paragraph::new(crate::tui_text!("Tab switch", "Tab 切换"))
            .style(theme::muted_style())
            .alignment(Alignment::Center);
        f.render_widget(help, footer_area);
    } else {
        let status_text = if error.is_some() {
            crate::tui_text!("Initialization failed", "初始化失败")
        } else {
            crate::tui_text!("Loading", "加载中")
        };
        let status_style = if error.is_some() {
            theme::error_style()
        } else {
            theme::info_style()
        };

        let status = Paragraph::new(status_text).style(status_style).block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(theme::border()))
                .title(crate::tui_text!(" Keys ", " 按键 "))
                .title_style(Style::default().fg(theme::codex())),
        );
        f.render_widget(status, footer_area);
    }
}

fn draw_account_list_with_status(f: &mut Frame, area: Rect, app: &mut CodexAuthApp) {
    let title = crate::tui_text!(" Accounts ", " 账号列表 ").to_string();
    render_account_list_panel(f, area, app, title);
}

fn draw_account_snapshot_panel(f: &mut Frame, area: Rect, app: &CodexAuthApp) {
    let mut lines = app
        .selected_account()
        .map(|account| account_snapshot_lines(app, account))
        .unwrap_or_else(|| {
            vec![
                detail_line("Account:", "-", theme::muted_style()),
                detail_line("State:", "-", theme::muted_style()),
            ]
        });

    lines.push(Line::from(Span::styled(
        crate::tui_text!("Quota remaining", "剩余配额"),
        theme::info_style(),
    )));
    lines.extend(quota_lines(app, usize::from(area.width.saturating_sub(2))));
    lines.push(quota_status_line(app));
    render_detail_card(
        f,
        area,
        crate::tui_text!(" Account & quota ", " 账号与配额 "),
        lines,
    );
}

fn draw_footer_strip(f: &mut Frame, area: Rect, app: &CodexAuthApp) {
    let mut hints = vec![
        ShortcutHint::new("Tab/Shift+Tab", crate::tui_text!("switch", "切换")),
        ShortcutHint::new("↑↓/jk", crate::tui_text!("select", "选择")),
        ShortcutHint::new("Enter", crate::tui_text!("switch", "切换")),
        ShortcutHint::new("s", crate::tui_text!("save", "保存")),
    ];
    if app.toasts.active().is_none() {
        hints.push(ShortcutHint::new("d", crate::tui_text!("delete", "删除")));
    }
    hints.extend([
        ShortcutHint::new("b", crate::tui_text!("quota", "配额")),
        ShortcutHint::new("r", crate::tui_text!("refresh", "刷新")),
        ShortcutHint::new("Ctrl+L", crate::tui_text!("language", "语言")),
        ShortcutHint::new("q", crate::tui_text!("quit", "退出")),
    ]);

    let compact = area.height <= 2 || area.width < 120;
    if compact {
        hints = vec![
            ShortcutHint::new("q", crate::tui_text!("quit", "退出")),
            ShortcutHint::new("Ctrl+L", crate::tui_text!("language", "语言")),
            ShortcutHint::new("Tab", ""),
            ShortcutHint::new("↑↓/jk", ""),
            ShortcutHint::new("Enter", ""),
            ShortcutHint::new("s", ""),
            ShortcutHint::new("d", ""),
            ShortcutHint::new("b", ""),
            ShortcutHint::new("r", ""),
        ];
    }
    let mut line = shortcut_line(&hints, theme::codex());
    if let Some(toast) = app.toasts.active() {
        let style = match toast.kind {
            ToastKind::Success => theme::success_style(),
            ToastKind::Error => theme::error_style(),
            ToastKind::Warning => theme::warning_style(),
            ToastKind::Info => theme::info_style(),
        };
        if compact {
            line.spans.push(Span::styled("  │  ", theme::muted_style()));
            line.spans.push(Span::styled(toast.message.clone(), style));
        } else {
            line.spans.insert(
                0,
                Span::styled("  │  ", Style::default().fg(theme::muted())),
            );
            line.spans
                .insert(0, Span::styled(toast.message.clone(), style));
        }
    }

    let help = Paragraph::new(line)
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(theme::border()))
                .title(crate::tui_text!(" Keys ", " 按键 "))
                .title_style(Style::default().fg(theme::muted())),
        )
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

    f.render_widget(help, area);
}

fn usage_attribution_style(state: CodexUsageAttributionState) -> Style {
    match state {
        CodexUsageAttributionState::AccountAttributed => theme::success_style(),
        CodexUsageAttributionState::GlobalOnly => theme::info_style(),
        CodexUsageAttributionState::VirtualAccount
        | CodexUsageAttributionState::UnattributedFallback => theme::warning_style(),
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::super::app::QuotaState;
    use super::*;
    use chrono::{TimeZone, Utc};
    use ratatui::{Terminal, backend::TestBackend};
    use std::path::PathBuf;

    pub(crate) fn presentation_fixture() -> (tempfile::TempDir, CodexAuthApp) {
        let dir = tempfile::tempdir().expect("isolated Codex Auth fixture directory");
        let service = ccr_cli::services::CodexAuthService::from_dirs(
            dir.path().join("ccr"),
            dir.path().join("codex"),
        );
        let mut app = CodexAuthApp::from_service(service).expect("injected Codex Auth service");
        let account = sample_account();
        app.accounts = vec![account.clone()];
        app.selected_index = 0;
        app.auth_registry
            .accounts
            .insert(account.name.clone(), sample_registry_account(None));
        let now = Utc::now();
        app.auth_registry.record_usage_activation(
            &account.name,
            "acc-codexcn",
            now - chrono::Duration::hours(2),
        );
        app.auth_registry.record_usage_activation(
            "other",
            "acc-other",
            now - chrono::Duration::minutes(45),
        );
        let records = [
            ccr_cli::services::CodexUsageRecord {
                session_id: "selected".into(),
                timestamp: now - chrono::Duration::hours(1),
                input_tokens: 12_000,
                output_tokens: 345,
                model: Some("gpt-example".into()),
                ..Default::default()
            },
            ccr_cli::services::CodexUsageRecord {
                session_id: "other".into(),
                timestamp: now - chrono::Duration::minutes(30),
                input_tokens: 900_000,
                output_tokens: 0,
                model: Some("other-model".into()),
                ..Default::default()
            },
        ];
        let selected_records = vec![records[0].clone()];
        app.usage_state = UsageState::Loaded(Box::new(super::super::app::CodexUsageDataset {
            snapshot: ccr_codex::services::codex_usage_estimation::CodexAuthUsageSnapshot {
                account_id: Some("acc-codexcn".into()),
                account_name: "codexcn".into(),
                as_of: now,
                rolling: ccr_cli::services::CodexUsageService::compute_rolling_usage_at(
                    &selected_records,
                    now,
                ),
                scope: ccr_codex::services::codex_usage_estimation::CodexUsageScope::ActivationIntervalInferred,
                excluded_records: 1,
                ..Default::default()
            },
        }));
        app.preview_cache.insert(
            account.name.clone(),
            super::super::app::QuotaPreviewEntry {
                quota: ccr_cli::models::CodexAccountQuota {
                    account_name: account.name,
                    email: account.email,
                    quota: Some(ccr_cli::models::CodexQuota {
                        hourly_percentage: 80,
                        weekly_percentage: 10,
                        hourly_reset_time: Some((now + chrono::Duration::hours(2)).timestamp()),
                        weekly_reset_time: Some((now + chrono::Duration::days(3)).timestamp()),
                        hourly_window_present: Some(true),
                        weekly_window_present: Some(true),
                        hourly_window_minutes: Some(300),
                        weekly_window_minutes: Some(10080),
                        plan_type: Some("pro".into()),
                        raw_data: None,
                    }),
                    error: None,
                    fetched_at: now - chrono::Duration::minutes(5),
                    ..Default::default()
                },
            },
        );
        (dir, app)
    }

    pub(crate) fn set_fixture_quota_error(app: &mut CodexAuthApp) {
        app.quota_state = QuotaState::Error {
            account_name: "codexcn".into(),
            message: "fixture unavailable".into(),
            cache: Default::default(),
        };
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub(crate) enum UsagePresentationCase {
        Estimate,
        Standard,
        CatalogPrice,
        MixedPriceSources,
        Fast,
        Partial,
        Unpriced,
        TinyCost,
        InsufficientSamples,
        Stale,
        GlobalFallback,
        HistoryWarning,
        UsageError,
        QuotaMissing,
        QuotaError,
        UsdUnpriced,
        UsdOnly,
        Unstable,
    }

    pub(crate) fn set_usage_presentation_case(app: &mut CodexAuthApp, case: UsagePresentationCase) {
        use ccr_codex::services::codex_usage_estimation::{
            AUTH_PRICE_SOURCE, AUTH_PRICE_VERSION, CodexJointEstimate, CodexUsageScope,
        };
        let now = Utc::now();
        let cost = CodexCostSummary {
            subtotal_usd: Some(5.15),
            status: CodexCostStatus::AssumedStandard,
            priced_records: 100,
            total_records: 100,
            priced_tokens: 10_200_000,
            total_tokens: 10_200_000,
            assumed_records: 100,
            price_version: AUTH_PRICE_VERSION.into(),
            source_url: Some(AUTH_PRICE_SOURCE.into()),
            verified_date: Some("2026-10-06".into()),
            tier_basis: "standard_assumed".into(),
            model_matches: ["gpt-6.1-sol".into()].into(),
            pricing_sources: ["official_verified".into()].into(),
            ..Default::default()
        };
        let estimate = CodexCapacityEstimate {
            status: CodexEstimateStatus::LocalEstimate,
            usd_status: CodexEstimateStatus::LocalEstimate,
            token_total: Some(CodexEstimateRange {
                median: 10_000_000.0,
                min: 8_000_000.0,
                max: 12_000_000.0,
            }),
            token_remaining: Some(CodexEstimateRange {
                median: 5_000_000.0,
                min: 4_000_000.0,
                max: 6_000_000.0,
            }),
            usd_total: Some(CodexEstimateRange {
                median: 5.0,
                min: 4.0,
                max: 6.0,
            }),
            usd_remaining: Some(CodexEstimateRange {
                median: 2.5,
                min: 2.0,
                max: 3.0,
            }),
            sample_count: 3,
            usd_sample_count: 3,
            span_start: Some(now - chrono::Duration::hours(4)),
            span_end: Some(now),
            bucket: Some("wham_main:wham_main".into()),
            workload_basis: Some("models=gpt-6.1-sol;tiers=standard_assumed".into()),
            pricing_basis: Some(AUTH_PRICE_VERSION.into()),
            expires_at: Some(now + chrono::Duration::minutes(5)),
            network_acquired_at: Some(now),
        };
        let UsageState::Loaded(dataset) = &mut app.usage_state else {
            panic!("loaded presentation fixture required");
        };
        let snapshot = &mut dataset.snapshot;
        for stats in [
            &mut snapshot.rolling.five_hour,
            &mut snapshot.rolling.seven_day,
            &mut snapshot.rolling.all_time,
        ] {
            stats.total_input_tokens = 10_000_000;
            stats.total_output_tokens = 200_000;
            stats.total_requests = 100;
            stats.details.cache_read_tokens = Some(9_000_000);
            stats.details.cache_write_tokens = Some(500_000);
            stats.details.reasoning_tokens = Some(100_000);
            stats.details.cost = cost.clone();
        }
        snapshot.five_hour = estimate.clone();
        snapshot.seven_day = estimate;
        snapshot.seven_day.token_remaining = Some(CodexEstimateRange {
            median: 2_000_000.0,
            min: 1_600_000.0,
            max: 2_400_000.0,
        });
        snapshot.seven_day.usd_remaining = Some(CodexEstimateRange {
            median: 1.0,
            min: 0.8,
            max: 1.2,
        });
        snapshot.joint = CodexJointEstimate::default();
        for stats in [
            &mut snapshot.rolling.five_hour,
            &mut snapshot.rolling.seven_day,
            &mut snapshot.rolling.all_time,
        ] {
            match case {
                UsagePresentationCase::Standard => {
                    stats.details.cost.status = CodexCostStatus::CompletePriced;
                    stats.details.cost.assumed_records = 0;
                    stats.details.cost.tier_basis = "standard".into();
                }
                UsagePresentationCase::CatalogPrice => {
                    stats.details.cost.source_url = None;
                    stats.details.cost.verified_date = None;
                    stats.details.cost.model_matches = ["gpt-4.1".into()].into();
                    stats.details.cost.pricing_sources = ["catalog_estimate".into()].into();
                }
                UsagePresentationCase::MixedPriceSources => {
                    stats.details.cost.model_matches.insert("gpt-4.1".into());
                    stats
                        .details
                        .cost
                        .pricing_sources
                        .insert("catalog_estimate".into());
                }
                UsagePresentationCase::Fast => {
                    stats.details.cost.subtotal_usd = Some(10.30);
                    stats.details.cost.status = CodexCostStatus::CompletePriced;
                    stats.details.cost.tier_basis = "fast".into();
                }
                UsagePresentationCase::Partial => {
                    stats.details.partial = true;
                    stats.details.cost.status = CodexCostStatus::Partial;
                    stats.details.cost.priced_records = 50;
                    stats.details.cost.priced_tokens = 5_100_000;
                }
                UsagePresentationCase::Unpriced => {
                    stats.details.cost = CodexCostSummary {
                        total_records: 100,
                        total_tokens: 10_200_000,
                        ..Default::default()
                    };
                }
                UsagePresentationCase::TinyCost => stats.details.cost.subtotal_usd = Some(0.001),
                _ => {}
            }
        }
        match case {
            UsagePresentationCase::Stale => {
                for estimate in [&mut snapshot.five_hour, &mut snapshot.seven_day] {
                    estimate.expires_at = Some(now - chrono::Duration::seconds(1));
                }
            }
            UsagePresentationCase::GlobalFallback => {
                snapshot.scope = CodexUsageScope::GlobalFallback;
                for estimate in [&mut snapshot.five_hour, &mut snapshot.seven_day] {
                    *estimate = CodexCapacityEstimate {
                        status: CodexEstimateStatus::InvalidScope,
                        usd_status: CodexEstimateStatus::InvalidScope,
                        network_acquired_at: Some(now),
                        ..Default::default()
                    };
                }
            }
            UsagePresentationCase::Partial
            | UsagePresentationCase::Unpriced
            | UsagePresentationCase::InsufficientSamples
            | UsagePresentationCase::Unstable => {
                let status = match case {
                    UsagePresentationCase::Partial => CodexEstimateStatus::PartialUsage,
                    UsagePresentationCase::Unstable => CodexEstimateStatus::Unstable,
                    _ => CodexEstimateStatus::InsufficientSamples,
                };
                for estimate in [&mut snapshot.five_hour, &mut snapshot.seven_day] {
                    *estimate = CodexCapacityEstimate {
                        status,
                        usd_status: status,
                        network_acquired_at: Some(now),
                        ..Default::default()
                    };
                }
            }
            UsagePresentationCase::UsdUnpriced => {
                for estimate in [&mut snapshot.five_hour, &mut snapshot.seven_day] {
                    estimate.usd_total = None;
                    estimate.usd_remaining = None;
                    estimate.usd_status = CodexEstimateStatus::Unpriced;
                }
            }
            UsagePresentationCase::UsdOnly => {
                for estimate in [&mut snapshot.five_hour, &mut snapshot.seven_day] {
                    estimate.token_total = None;
                    estimate.token_remaining = None;
                    estimate.status = CodexEstimateStatus::Unstable;
                }
            }
            UsagePresentationCase::UsageError => {
                app.usage_error = Some("fixture scan failure".into())
            }
            _ => {}
        }
        let entry = app.preview_cache.get_mut("codexcn").expect("fixture quota");
        entry.quota.fetched_at = now;
        entry.quota.observation = Some(ccr_codex::CodexQuotaProvenance {
            account_id: Some("acc-codexcn".into()),
            request_started_at: now,
            network_acquired_at: now,
            returned_at: now,
            cache_hit: false,
            history_warning: (case == UsagePresentationCase::HistoryWarning)
                .then(|| "quota_history_write_failed".into()),
        });
        if case == UsagePresentationCase::QuotaMissing {
            let quota = entry.quota.quota.as_mut().expect("fixture quota windows");
            quota.hourly_window_present = Some(false);
            quota.weekly_window_present = Some(false);
        }
        if case == UsagePresentationCase::QuotaError {
            set_fixture_quota_error(app);
        }
    }

    #[test]
    fn remaining_quota_bar_fill_colors_and_presence_share_the_value() {
        for percentage in [0, 10, 50, 80, 100, -1, 101] {
            let line = quota_window_line("5h", percentage, Some(true), None, 80);
            let text = plain_line_text(&line);
            let value = percentage.clamp(0, 100);
            assert_eq!(text.matches('█').count(), value as usize / 5, "{text}");
            assert_eq!(text.matches('░').count(), 20 - value as usize / 5, "{text}");
            assert!(text.contains(&format!("{value:>3}%")));
            assert_eq!(line.spans[1].style.fg, Some(theme::quota_color(value)));
            assert_eq!(line.spans[2].style.fg, Some(theme::muted()));
            assert_eq!(line.spans[4].style.fg, Some(theme::quota_color(value)));
        }
        let (_dir, mut app) = presentation_fixture();
        for present in [Some(false), None, Some(true)] {
            let entry = app
                .preview_cache
                .get_mut("codexcn")
                .expect("fixture preview");
            let quota = entry.quota.quota.as_mut().expect("fixture quota");
            quota.hourly_window_present = present;
            quota.weekly_window_present = present;
            for (index, window) in [PreviewMetricWindow::FiveHour, PreviewMetricWindow::SevenDay]
                .into_iter()
                .enumerate()
            {
                let preview = app.preview_cell_for_account("codexcn", window);
                let detail = plain_line_text(&quota_lines(&app, 80)[index]);
                assert_eq!(preview.text.contains('%'), present == Some(true));
                assert_eq!(detail.contains('░'), present == Some(true));
                if present == Some(false) {
                    assert!(detail.contains("Not provided"));
                }
                if present.is_none() {
                    assert!(detail.contains("Unknown window"));
                }
            }
        }
    }

    #[test]
    fn cached_idle_refresh_and_failure_keep_bars_and_fetch_time() {
        let (_dir, mut app) = presentation_fixture();
        for state in [
            QuotaState::Idle,
            QuotaState::Loading {
                account_name: "codexcn".into(),
                cache: Default::default(),
            },
            QuotaState::Error {
                account_name: "codexcn".into(),
                message: "network unavailable".into(),
                cache: Default::default(),
            },
        ] {
            app.quota_state = state;
            let mut terminal = Terminal::new(TestBackend::new(70, 12)).unwrap();
            terminal
                .draw(|frame| draw_combined_panel(frame, frame.area(), &app))
                .unwrap();
            let text = buffer_text(terminal.backend());
            assert!(text.contains('█'), "{text}");
            assert!(text.contains("Cached"), "{text}");
            assert!(text.contains("12.3K"), "{text}");
            if app.is_selected_quota_loading() {
                assert!(text.contains("Refreshing"), "{text}");
            }
            if app.selected_quota_error().is_some() {
                assert!(text.contains("Quota error: network unavailable"), "{text}");
                let cell = terminal
                    .backend()
                    .buffer()
                    .content
                    .iter()
                    .find(|cell| cell.symbol() == "Q" && cell.fg == theme::error());
                assert!(cell.is_some(), "{text}");
            }
        }
        app.preview_cache.clear();
        assert!(
            quota_lines(&app, 60)
                .iter()
                .all(|line| !plain_line_text(line).contains('█'))
        );
        assert!(plain_line_text(&quota_status_line(&app)).contains("network unavailable"));
        app.quota_state = QuotaState::Loading {
            account_name: "codexcn".into(),
            cache: Default::default(),
        };
        assert!(plain_line_text(&quota_lines(&app, 60)[0]).contains("Loading"));
        app.quota_state = QuotaState::Idle;
        assert!(plain_line_text(&quota_lines(&app, 60)[0]).contains("No cached quota"));
    }

    #[test]
    fn selected_account_load_error_does_not_render_the_previous_account_snapshot() {
        let (_dir, mut app) = presentation_fixture();
        app.accounts[0].name = "other-selection".into();
        app.usage_error = Some("fixture scan failure".into());
        assert!(app.usage_panel_data().is_none());
        let mut terminal = Terminal::new(TestBackend::new(80, 12)).unwrap();
        terminal
            .draw(|frame| draw_combined_panel(frame, frame.area(), &app))
            .unwrap();
        let rendered = buffer_text(terminal.backend());
        assert!(
            rendered.contains("Local usage error: fixture scan failure"),
            "{rendered}"
        );
        assert!(!rendered.contains("12.3K"), "{rendered}");
    }

    #[test]
    fn attribution_notes_are_neutral_and_fallback_scope_stays_with_numbers() {
        let (_dir, mut app) = presentation_fixture();
        let panel = app.usage_panel_data().expect("fixture usage panel");
        let note = usage_note_line(&panel).expect("fixture coverage note");
        assert_eq!(note.spans[0].style.fg, Some(theme::muted()));
        assert!(plain_line_text(&note).contains("other accounts"));
        assert_eq!(panel.rolling.all_time.total_input_tokens, 12_000);
        for fallback in 0..3 {
            let (_dir, mut fallback_app) = presentation_fixture();
            match fallback {
                0 => fallback_app.auth_registry.accounts.clear(),
                1 => fallback_app.auth_registry.usage_ledger.clear(),
                _ => fallback_app.accounts[0].is_virtual = true,
            }
            if let UsageState::Loaded(dataset) = &mut fallback_app.usage_state {
                dataset.snapshot.scope =
                    ccr_codex::services::codex_usage_estimation::CodexUsageScope::GlobalFallback;
                dataset.snapshot.account_id = None;
                for stats in [
                    &mut dataset.snapshot.rolling.five_hour,
                    &mut dataset.snapshot.rolling.seven_day,
                    &mut dataset.snapshot.rolling.all_time,
                ] {
                    stats.total_input_tokens = 912_000;
                    stats.total_requests = 2;
                }
            }
            let panel = fallback_app
                .usage_panel_data()
                .expect("fixture global fallback panel");
            let lines = local_usage_lines(&fallback_app, 4, 200);
            assert!(plain_line_text(&lines[0]).contains("Local: global (not selected)"));
            assert_eq!(lines[0].spans[0].style.fg, Some(theme::warning()));
            assert!(
                plain_line_text(&lines[0]).contains(
                    panel
                        .fallback_reason
                        .as_deref()
                        .expect("fixture fallback reason")
                )
            );
            assert_eq!(panel.rolling.all_time.total_input_tokens, 912_000);
            assert_eq!(lines.len(), 4);
        }
        app.usage_state = UsageState::Error("fixture read failure".into());
        assert_eq!(
            usage_state_line(&app).spans[0].style.fg,
            Some(theme::error())
        );
        app.usage_state = UsageState::NoData;
        assert_eq!(
            usage_state_line(&app).spans[0].style.fg,
            Some(theme::muted())
        );
    }

    #[test]
    fn long_cjk_identity_and_error_are_clipped_with_statistics_and_folded_details() {
        let (_dir, mut app) = presentation_fixture();
        let name = "测试账号".repeat(20);
        app.accounts[0].name = name.clone();
        let account = app
            .auth_registry
            .accounts
            .shift_remove("codexcn")
            .expect("fixture registered account");
        app.auth_registry.accounts.insert(name.clone(), account);
        let mut quota = app
            .preview_cache
            .shift_remove("codexcn")
            .expect("fixture cached quota");
        quota.quota.account_name = name.clone();
        app.preview_cache.insert(name.clone(), quota);
        if let UsageState::Loaded(dataset) = &mut app.usage_state {
            dataset.snapshot.account_name = name.clone();
            dataset.snapshot.rolling.by_model.clear();
            let model_usage = ccr_codex::services::codex_usage_service::CodexUsageStats {
                total_input_tokens: 12_000,
                total_output_tokens: 345,
                total_requests: 1,
                ..Default::default()
            };
            dataset
                .snapshot
                .rolling
                .by_model
                .insert("非常长的模型名称".repeat(20), model_usage);
        }
        app.quota_state = QuotaState::Error {
            account_name: name,
            message: "long fixture failure ".repeat(20),
            cache: Default::default(),
        };
        let mut terminal = Terminal::new(TestBackend::new(70, 12)).unwrap();
        terminal
            .draw(|frame| draw_combined_panel(frame, frame.area(), &app))
            .unwrap();
        let text = buffer_text(terminal.backend());
        assert!(text.matches("12.3K").count() >= 3, "{text}");
        assert!(text.contains("Local: account"), "{text}");
        assert!(text.contains("folded"), "{text}");
        assert!(text.contains("5h rem N/A:"), "{text}");
        assert!(text.contains("7d rem N/A:"), "{text}");
        assert!(text.contains("Quota error:"), "{text}");
        assert!(text.matches('…').count() >= 2, "{text}");
        for y in 1..11 {
            assert_eq!(
                terminal
                    .backend()
                    .buffer()
                    .cell((69, y))
                    .expect("right border cell within fixture")
                    .symbol(),
                "│"
            );
        }
        app.pending_quota_confirm = true;
        assert!(plain_line_text(&quota_status_line(&app)).contains("y confirm"));
    }

    #[test]
    fn compact_counts_promote_rounded_units_and_columns_align_in_both_languages() {
        for (value, expected) in [
            (0, "0"),
            (999, "999"),
            (1_000, "1K"),
            (999_950, "1M"),
            (1_234_567, "1.2M"),
            (6_371_200_000, "6.4B"),
            (1_000_000_000_000, "1T"),
        ] {
            assert_eq!(format_compact_count(value), expected);
        }
        let (_dir, app) = presentation_fixture();
        for language in [
            ccr_cli::managers::TuiLanguage::English,
            ccr_cli::managers::TuiLanguage::SimplifiedChinese,
        ] {
            crate::tui::i18n::set_language(language);
            let rows =
                usage_table_lines(&app.usage_panel_data().expect("fixture usage panel"), true);
            let widths: Vec<_> = rows.iter().map(Line::width).collect();
            assert!(widths.iter().all(|width| *width <= 80), "{widths:?}");
            for line in &rows[1..] {
                assert_eq!(line.spans[0].width(), 8);
                assert!(plain_line_text(line).contains("1"));
            }
            let line = clipped_line(
                Line::from(vec![
                    Span::raw("账号："),
                    Span::styled("很长的模型名字".repeat(10), theme::error_style()),
                ]),
                23,
            );
            assert!(plain_line_text(&line).ends_with('…'));
            assert!(line.width() <= 23);
            assert_eq!(
                line.spans.last().expect("truncated line ellipsis").style.fg,
                Some(theme::error())
            );
        }
        crate::tui::i18n::set_language(ccr_cli::managers::TuiLanguage::English);
    }

    fn sample_account() -> ccr_cli::models::CodexAuthItem {
        ccr_cli::models::CodexAuthItem {
            name: "codexcn".to_string(),
            description: Some("Primary account".to_string()),
            email: Some("bah***@gmail.com".to_string()),
            plan_type: Some("plus".to_string()),
            is_current: true,
            is_virtual: false,
            saved_at: Some(Utc.with_ymd_and_hms(2026, 4, 5, 12, 0, 0).unwrap()),
            last_used: None,
            last_refresh: None,
        }
    }

    fn sample_account_without_plan() -> ccr_cli::models::CodexAuthItem {
        let mut account = sample_account();
        account.plan_type = None;
        account
    }

    fn sample_registry_account(
        auth_method: Option<ccr_cli::models::OpenAiAuthMethod>,
    ) -> ccr_cli::models::CodexAuthAccount {
        ccr_cli::models::CodexAuthAccount {
            description: Some("Primary account".to_string()),
            account_id: "acc-codexcn".to_string(),
            auth_method,
            api_base_url: None,
            api_provider_name: None,
            email: Some("bah***@gmail.com".to_string()),
            plan_type: None,
            saved_at: Utc.with_ymd_and_hms(2026, 4, 5, 12, 0, 0).unwrap(),
            last_used: None,
            last_refresh: None,
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

    fn buffer_text(backend: &TestBackend) -> String {
        let height = backend.buffer().area.height;
        let width = backend.buffer().area.width;
        (0..height)
            .map(|y| {
                (0..width)
                    .filter_map(|x| backend.buffer().cell((x, y)))
                    .map(|cell| cell.symbol())
                    .collect::<Vec<_>>()
                    .join("")
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn compact_text(value: &str) -> String {
        value.chars().filter(|ch| !ch.is_whitespace()).collect()
    }

    #[test]
    fn account_table_layout_hides_secondary_columns_on_narrow_widths() {
        let layout = account_table_layout(60);
        assert_eq!(
            layout.columns,
            vec![AccountColumn::Account, AccountColumn::QuotaSummary]
        );
        assert_eq!(layout.widths.len(), 2);
    }

    #[test]
    fn account_table_layout_shows_plan_on_wide_widths() {
        let layout = account_table_layout(108);
        assert_eq!(
            layout.columns,
            vec![
                AccountColumn::Account,
                AccountColumn::Email,
                AccountColumn::Plan,
                AccountColumn::HourlyQuota,
                AccountColumn::WeeklyQuota,
                AccountColumn::ExpiresAt,
            ]
        );
        assert_eq!(layout.widths.len(), 6);
    }

    #[test]
    fn account_table_layout_resolves_flexible_widths_from_available_space() {
        let narrow = account_table_layout(60);
        let wide = account_table_layout(108);

        assert_eq!(narrow.resolved_width(AccountColumn::Account), 20);
        assert!(narrow.resolved_width(AccountColumn::QuotaSummary) >= 16);
        assert_eq!(wide.resolved_width(AccountColumn::HourlyQuota), 12);
        assert_eq!(wide.resolved_width(AccountColumn::WeeklyQuota), 12);
        assert_eq!(wide.resolved_width(AccountColumn::Plan), 10);
    }

    #[test]
    fn account_list_regions_reserve_one_row_for_header() {
        let inner = Rect::new(2, 3, 80, 9);
        let regions = account_list_regions(inner);

        assert_eq!(regions.header, Rect::new(2, 3, 80, 1));
        assert_eq!(regions.body, Rect::new(2, 4, 80, 8));
    }

    #[test]
    fn account_snapshot_lines_show_identity_and_refresh_metadata() {
        let service =
            ccr_cli::services::CodexAuthService::from_dirs(PathBuf::from("."), PathBuf::from("."));
        let mut app = crate::tui::codex_auth::app::CodexAuthApp::from_service(service)
            .expect("test codex auth app should initialize from injected service");
        app.accounts = vec![sample_account()];
        app.selected_index = 0;
        app.preview_cache.insert(
            "codexcn".to_string(),
            crate::tui::codex_auth::app::QuotaPreviewEntry {
                quota: ccr_cli::models::CodexAccountQuota {
                    account_name: "codexcn".to_string(),
                    email: Some("bah***@gmail.com".to_string()),
                    quota: Some(ccr_cli::models::CodexQuota {
                        hourly_percentage: 95,
                        hourly_reset_time: Some(
                            (Utc::now()
                                + chrono::Duration::hours(3)
                                + chrono::Duration::minutes(11))
                            .timestamp(),
                        ),
                        hourly_window_minutes: Some(300),
                        hourly_window_present: Some(true),
                        weekly_percentage: 33,
                        weekly_reset_time: Some(
                            (Utc::now()
                                + chrono::Duration::days(2)
                                + chrono::Duration::hours(3)
                                + chrono::Duration::minutes(17))
                            .timestamp(),
                        ),
                        weekly_window_minutes: Some(10080),
                        weekly_window_present: Some(true),
                        plan_type: Some("plus".to_string()),
                        raw_data: None,
                    }),
                    error: None,
                    fetched_at: Utc::now(),
                    ..Default::default()
                },
            },
        );
        let lines = account_snapshot_lines(&app, &sample_account());

        assert!(plain_line_text(&lines[0]).contains("codexcn"));
        assert!(plain_line_text(&lines[3]).contains("PLUS"));
        assert!(plain_line_text(&lines[4]).contains("Saved at:"));
        assert!(plain_line_text(&lines[5]).contains("Auth refresh:"));
        let quota = quota_lines(&app, 80);
        assert!(plain_line_text(&quota[0]).contains("5h "));
        assert!(plain_line_text(&quota[0]).contains("95%"));
        assert!(plain_line_text(&quota[0]).contains("Reset"));
        assert!(plain_line_text(&quota[1]).contains("7d "));
        assert!(plain_line_text(&quota[1]).contains("33%"));
        assert!(plain_line_text(&quota[1]).contains("Reset"));
        assert_eq!(lines[0].spans[0].style.fg, Some(theme::subtext()));
        assert_eq!(lines[0].spans[1].style.fg, Some(theme::success()));
        assert_eq!(lines[2].spans[1].style.fg, Some(theme::info()));
        assert_eq!(lines[3].spans[1].style.fg, Some(theme::info()));
    }

    #[test]
    fn account_snapshot_lines_prefer_quota_plan_when_account_plan_missing() {
        let service =
            ccr_cli::services::CodexAuthService::from_dirs(PathBuf::from("."), PathBuf::from("."));
        let mut app = crate::tui::codex_auth::app::CodexAuthApp::from_service(service)
            .expect("test codex auth app should initialize from injected service");
        let account = sample_account_without_plan();
        app.accounts = vec![account.clone()];
        app.selected_index = 0;
        app.preview_cache.insert(
            "codexcn".to_string(),
            crate::tui::codex_auth::app::QuotaPreviewEntry {
                quota: ccr_cli::models::CodexAccountQuota {
                    account_name: "codexcn".to_string(),
                    email: Some("bah***@gmail.com".to_string()),
                    quota: Some(ccr_cli::models::CodexQuota {
                        hourly_percentage: 88,
                        hourly_reset_time: Some(
                            (Utc::now() + chrono::Duration::hours(4)).timestamp(),
                        ),
                        hourly_window_minutes: Some(300),
                        hourly_window_present: Some(true),
                        weekly_percentage: 76,
                        weekly_reset_time: Some(
                            (Utc::now() + chrono::Duration::days(3)).timestamp(),
                        ),
                        weekly_window_minutes: Some(10080),
                        weekly_window_present: Some(true),
                        plan_type: Some("team".to_string()),
                        raw_data: None,
                    }),
                    error: None,
                    fetched_at: Utc::now(),
                    ..Default::default()
                },
            },
        );

        let lines = account_snapshot_lines(&app, &account);

        assert!(plain_line_text(&lines[3]).contains("TEAM"));
    }

    #[test]
    fn plan_column_falls_back_to_auth_property_when_plan_missing() {
        let service =
            ccr_cli::services::CodexAuthService::from_dirs(PathBuf::from("."), PathBuf::from("."));
        let mut app = crate::tui::codex_auth::app::CodexAuthApp::from_service(service)
            .expect("test codex auth app should initialize from injected service");
        let account = sample_account_without_plan();
        app.accounts = vec![account.clone()];
        app.selected_index = 0;
        app.auth_registry.accounts.insert(
            account.name.clone(),
            sample_registry_account(Some(ccr_cli::models::OpenAiAuthMethod::Chatgpt)),
        );

        let layout = account_table_layout(108);
        let cell = account_cell(&account, &app, AccountColumn::Plan, &layout, false);
        let mut terminal = Terminal::new(TestBackend::new(10, 1)).unwrap();
        terminal
            .draw(|frame| {
                let table = Table::new([Row::new(vec![cell])], vec![Constraint::Length(10)]);
                frame.render_widget(table, frame.area());
            })
            .unwrap();

        let rendered = buffer_line_text(terminal.backend(), 0);
        assert!(rendered.contains("CHATGPT"), "{rendered}");
    }

    #[test]
    fn plan_column_prefers_quota_plan_over_auth_property() {
        let service =
            ccr_cli::services::CodexAuthService::from_dirs(PathBuf::from("."), PathBuf::from("."));
        let mut app = crate::tui::codex_auth::app::CodexAuthApp::from_service(service)
            .expect("test codex auth app should initialize from injected service");
        let account = sample_account_without_plan();
        app.accounts = vec![account.clone()];
        app.selected_index = 0;
        app.auth_registry.accounts.insert(
            account.name.clone(),
            sample_registry_account(Some(ccr_cli::models::OpenAiAuthMethod::Chatgpt)),
        );
        app.preview_cache.insert(
            account.name.clone(),
            crate::tui::codex_auth::app::QuotaPreviewEntry {
                quota: ccr_cli::models::CodexAccountQuota {
                    account_name: account.name.clone(),
                    email: account.email.clone(),
                    quota: Some(ccr_cli::models::CodexQuota {
                        hourly_percentage: 92,
                        hourly_reset_time: Some(
                            (Utc::now() + chrono::Duration::hours(2)).timestamp(),
                        ),
                        hourly_window_minutes: Some(300),
                        hourly_window_present: Some(true),
                        weekly_percentage: 73,
                        weekly_reset_time: Some(
                            (Utc::now() + chrono::Duration::days(4)).timestamp(),
                        ),
                        weekly_window_minutes: Some(10080),
                        weekly_window_present: Some(true),
                        plan_type: Some("team".to_string()),
                        raw_data: None,
                    }),
                    error: None,
                    fetched_at: Utc::now(),
                    ..Default::default()
                },
            },
        );

        let (property, _) = account_property_display(&app, &account);

        assert_eq!(property, "TEAM");
    }

    #[test]
    fn account_table_render_keeps_account_and_quota_summary_visible_in_compact_layout() {
        let layout = account_table_layout(60);
        let mut terminal = Terminal::new(TestBackend::new(60, 1)).unwrap();
        let account = sample_account();
        let service =
            ccr_cli::services::CodexAuthService::from_dirs(PathBuf::from("."), PathBuf::from("."));
        let mut app = crate::tui::codex_auth::app::CodexAuthApp::from_service(service)
            .expect("test codex auth app should initialize from injected service");
        app.accounts = vec![account.clone()];
        app.selected_index = 0;
        app.preview_cache.insert(
            "codexcn".to_string(),
            crate::tui::codex_auth::app::QuotaPreviewEntry {
                quota: ccr_cli::models::CodexAccountQuota {
                    account_name: "codexcn".to_string(),
                    email: None,
                    quota: Some(ccr_cli::models::CodexQuota {
                        hourly_percentage: 95,
                        hourly_reset_time: Some(
                            (Utc::now()
                                + chrono::Duration::hours(3)
                                + chrono::Duration::minutes(11))
                            .timestamp(),
                        ),
                        hourly_window_minutes: Some(300),
                        hourly_window_present: Some(true),
                        weekly_percentage: 33,
                        weekly_reset_time: Some(
                            (Utc::now()
                                + chrono::Duration::days(2)
                                + chrono::Duration::hours(3)
                                + chrono::Duration::minutes(17))
                            .timestamp(),
                        ),
                        weekly_window_minutes: Some(10080),
                        weekly_window_present: Some(true),
                        plan_type: None,
                        raw_data: None,
                    }),
                    error: None,
                    fetched_at: Utc::now(),
                    ..Default::default()
                },
            },
        );

        terminal
            .draw(|frame| {
                let row = Row::new(
                    layout
                        .columns
                        .iter()
                        .map(|column| account_cell(&account, &app, *column, &layout, false)),
                );
                let table =
                    Table::new([row], layout.widths.clone()).column_spacing(ACCOUNT_COLUMN_SPACING);
                frame.render_widget(table, frame.area());
            })
            .unwrap();

        let rendered = buffer_line_text(terminal.backend(), 0);
        assert!(
            !rendered.contains("●"),
            "status dot should be gone: {rendered}"
        );
        assert!(rendered.contains("codexcn"), "{rendered}");
        assert!(rendered.contains("95%/33%"), "{rendered}");
    }

    #[test]
    fn hourly_cell_appends_reset_in_parentheses() {
        let account = sample_account();
        let service =
            ccr_cli::services::CodexAuthService::from_dirs(PathBuf::from("."), PathBuf::from("."));
        let mut app = crate::tui::codex_auth::app::CodexAuthApp::from_service(service)
            .expect("test codex auth app should initialize from injected service");
        app.accounts = vec![account.clone()];
        app.selected_index = 0;
        app.preview_cache.insert(
            "codexcn".to_string(),
            crate::tui::codex_auth::app::QuotaPreviewEntry {
                quota: ccr_cli::models::CodexAccountQuota {
                    account_name: "codexcn".to_string(),
                    email: None,
                    quota: Some(ccr_cli::models::CodexQuota {
                        hourly_percentage: 52,
                        hourly_reset_time: Some(
                            (Utc::now()
                                + chrono::Duration::hours(3)
                                + chrono::Duration::minutes(11))
                            .timestamp(),
                        ),
                        hourly_window_minutes: Some(300),
                        hourly_window_present: Some(true),
                        weekly_percentage: 41,
                        weekly_reset_time: Some(
                            (Utc::now() + chrono::Duration::days(2) + chrono::Duration::hours(3))
                                .timestamp(),
                        ),
                        weekly_window_minutes: Some(10080),
                        weekly_window_present: Some(true),
                        plan_type: Some("plus".to_string()),
                        raw_data: None,
                    }),
                    error: None,
                    fetched_at: Utc::now(),
                    ..Default::default()
                },
            },
        );

        let layout = account_table_layout(100);
        let cell = account_cell(&account, &app, AccountColumn::HourlyQuota, &layout, false);
        let mut terminal = Terminal::new(TestBackend::new(20, 1)).unwrap();
        terminal
            .draw(|frame| {
                let table = Table::new([Row::new(vec![cell])], vec![Constraint::Length(20)]);
                frame.render_widget(table, frame.area());
            })
            .unwrap();
        let rendered = buffer_line_text(terminal.backend(), 0);
        assert!(rendered.contains("52%"), "{rendered}");
        assert!(rendered.contains("(3h"), "{rendered}");
        assert!(rendered.contains("m)"), "{rendered}");
    }

    #[test]
    fn weekly_cell_appends_reset_in_parentheses() {
        let account = sample_account();
        let service =
            ccr_cli::services::CodexAuthService::from_dirs(PathBuf::from("."), PathBuf::from("."));
        let mut app = crate::tui::codex_auth::app::CodexAuthApp::from_service(service)
            .expect("test codex auth app should initialize from injected service");
        app.accounts = vec![account.clone()];
        app.selected_index = 0;
        app.preview_cache.insert(
            "codexcn".to_string(),
            crate::tui::codex_auth::app::QuotaPreviewEntry {
                quota: ccr_cli::models::CodexAccountQuota {
                    account_name: "codexcn".to_string(),
                    email: None,
                    quota: Some(ccr_cli::models::CodexQuota {
                        hourly_percentage: 10,
                        hourly_reset_time: Some(
                            (Utc::now() + chrono::Duration::hours(1)).timestamp(),
                        ),
                        hourly_window_minutes: Some(300),
                        hourly_window_present: Some(true),
                        weekly_percentage: 41,
                        weekly_reset_time: Some(
                            (Utc::now() + chrono::Duration::days(2) + chrono::Duration::hours(3))
                                .timestamp(),
                        ),
                        weekly_window_minutes: Some(10080),
                        weekly_window_present: Some(true),
                        plan_type: Some("plus".to_string()),
                        raw_data: None,
                    }),
                    error: None,
                    fetched_at: Utc::now(),
                    ..Default::default()
                },
            },
        );

        let layout = account_table_layout(100);
        let cell = account_cell(&account, &app, AccountColumn::WeeklyQuota, &layout, false);
        let mut terminal = Terminal::new(TestBackend::new(20, 1)).unwrap();
        terminal
            .draw(|frame| {
                let table = Table::new([Row::new(vec![cell])], vec![Constraint::Length(20)]);
                frame.render_widget(table, frame.area());
            })
            .unwrap();
        let rendered = buffer_line_text(terminal.backend(), 0);
        assert!(rendered.contains("41%"), "{rendered}");
        assert!(rendered.contains("(2d"), "{rendered}");
        assert!(rendered.contains("h)"), "{rendered}");
    }

    #[test]
    fn draw_account_snapshot_panel_keeps_weekly_reset_visible() {
        let service =
            ccr_cli::services::CodexAuthService::from_dirs(PathBuf::from("."), PathBuf::from("."));
        let mut app = crate::tui::codex_auth::app::CodexAuthApp::from_service(service)
            .expect("test codex auth app should initialize from injected service");
        app.accounts = vec![sample_account()];
        app.selected_index = 0;
        app.preview_cache.insert(
            "codexcn".to_string(),
            crate::tui::codex_auth::app::QuotaPreviewEntry {
                quota: ccr_cli::models::CodexAccountQuota {
                    account_name: "codexcn".to_string(),
                    email: Some("bah***@gmail.com".to_string()),
                    quota: Some(ccr_cli::models::CodexQuota {
                        hourly_percentage: 95,
                        hourly_reset_time: Some(
                            (Utc::now()
                                + chrono::Duration::hours(4)
                                + chrono::Duration::minutes(59))
                            .timestamp(),
                        ),
                        hourly_window_minutes: Some(300),
                        hourly_window_present: Some(true),
                        weekly_percentage: 33,
                        weekly_reset_time: Some(
                            (Utc::now()
                                + chrono::Duration::days(5)
                                + chrono::Duration::hours(6)
                                + chrono::Duration::minutes(13))
                            .timestamp(),
                        ),
                        weekly_window_minutes: Some(10080),
                        weekly_window_present: Some(true),
                        plan_type: Some("plus".to_string()),
                        raw_data: None,
                    }),
                    error: None,
                    fetched_at: Utc::now(),
                    ..Default::default()
                },
            },
        );

        let mut terminal = Terminal::new(TestBackend::new(100, 13)).unwrap();
        terminal
            .draw(|frame| draw_account_snapshot_panel(frame, frame.area(), &app))
            .unwrap();

        let rendered = buffer_text(terminal.backend());
        assert!(rendered.contains("7d "), "{rendered}");
        assert!(rendered.contains("Reset"), "{rendered}");
        assert!(rendered.contains("5d6h13m"), "{rendered}");
        assert!(
            rendered.contains('█'),
            "cached Idle quota must show a bar: {rendered}"
        );
    }

    #[test]
    fn usage_digest_lines_include_all_time_and_top_model() {
        let mut usage = ccr_cli::services::CodexRollingUsage::default();
        usage.five_hour.total_input_tokens = 1_000;
        usage.five_hour.total_output_tokens = 2_000;
        usage.five_hour.total_requests = 3;
        usage.seven_day.total_input_tokens = 10_000;
        usage.seven_day.total_output_tokens = 20_000;
        usage.seven_day.total_requests = 30;
        usage.all_time.total_input_tokens = 50_000;
        usage.all_time.total_output_tokens = 10_000;
        usage.all_time.total_requests = 42;
        let panel = CodexAuthUsagePanelData {
            scope: CodexUsageScope::AccountAttributed {
                account_name: "codexcn".to_string(),
            },
            attribution_state: CodexUsageAttributionState::AccountAttributed,
            rolling: usage,
            top_model: Some(crate::tui::codex_auth::app::CodexUsageTopModel {
                model: "gpt-5.4".to_string(),
                total_tokens: 35_000,
                total_requests: 21,
            }),
            fallback_reason: None,
            estimate: Default::default(),
        };

        let lines: Vec<String> = usage_table_lines(&panel, true)
            .into_iter()
            .chain([top_model_line(&panel)])
            .map(|line| plain_line_text(&line))
            .collect();

        assert!(
            lines.iter().any(|line| line.contains("All time")
                && line.contains("60K")
                && line.contains("42"))
        );
        assert!(lines.iter().any(|line| line.contains("Top model: gpt-5.4")));
    }

    #[test]
    fn draw_usage_panel_keeps_quota_and_local_attribution_note() {
        let service =
            ccr_cli::services::CodexAuthService::from_dirs(PathBuf::from("."), PathBuf::from("."));
        let mut app = crate::tui::codex_auth::app::CodexAuthApp::from_service(service)
            .expect("test codex auth app should initialize from injected service");
        app.accounts = vec![sample_account()];
        app.selected_index = 0;
        app.quota_state = QuotaState::Loaded {
            cache: indexmap::IndexMap::from([(
                "codexcn".to_string(),
                ccr_cli::models::CodexAccountQuota {
                    account_name: "codexcn".to_string(),
                    email: Some("bah***@gmail.com".to_string()),
                    quota: Some(ccr_cli::models::CodexQuota {
                        hourly_percentage: 52,
                        hourly_reset_time: Some(
                            (Utc::now() + chrono::Duration::hours(3)).timestamp(),
                        ),
                        hourly_window_minutes: Some(300),
                        hourly_window_present: Some(true),
                        weekly_percentage: 41,
                        weekly_reset_time: Some(
                            (Utc::now() + chrono::Duration::days(2)).timestamp(),
                        ),
                        weekly_window_minutes: Some(10080),
                        weekly_window_present: Some(true),
                        plan_type: Some("plus".to_string()),
                        raw_data: None,
                    }),
                    error: None,
                    fetched_at: Utc::now(),
                    ..Default::default()
                },
            )]),
        };
        let global_now = Utc::now();
        let global_records = [ccr_cli::services::CodexUsageRecord {
            session_id: "global-only".to_string(),
            timestamp: global_now,
            input_tokens: 1200,
            output_tokens: 240,
            model: Some("gpt-5.4".to_string()),
            ..Default::default()
        }];
        app.usage_state = UsageState::Loaded(Box::new(
            crate::tui::codex_auth::app::CodexUsageDataset {
                snapshot: ccr_codex::services::codex_usage_estimation::CodexAuthUsageSnapshot {
                    account_name: "codexcn".into(),
                    as_of: global_now,
                    rolling: ccr_cli::services::CodexUsageService::compute_rolling_usage_at(
                        &global_records,
                        global_now,
                    ),
                    scope:
                        ccr_codex::services::codex_usage_estimation::CodexUsageScope::GlobalFallback,
                    ..Default::default()
                },
            },
        ));

        let mut terminal = Terminal::new(TestBackend::new(90, 18)).unwrap();
        terminal
            .draw(|frame| draw_usage_panel(frame, frame.area(), &app))
            .unwrap();

        let rendered = buffer_text(terminal.backend());
        let compact = compact_text(&rendered);
        assert!(
            compact.contains("Quota/Localusage·APIequivalentUSD"),
            "{rendered}"
        );
        assert!(compact.contains("Localusage"), "{rendered}");
        assert!(compact.contains("Reset"), "{rendered}");
        assert!(compact.contains("7d"), "{rendered}");
        assert!(compact.contains("Local:global(notselected)"), "{rendered}");
        assert!(
            compact.contains("NomatchingCCRattributionrecords"),
            "{rendered}"
        );
    }

    #[test]
    fn relogin_quota_error_shows_local_logout_hint_in_both_languages() {
        let (_dir, mut app) = presentation_fixture();
        app.quota_state = QuotaState::Error {
            account_name: "codexcn".into(),
            message: format!(
                "{}Token 刷新失败 (401) [refresh_token_invalidated]",
                ccr_codex::services::codex_quota_service::RELOGIN_REQUIRED_PREFIX
            ),
            cache: Default::default(),
        };
        for (language, expected) in [
            (
                ccr_cli::managers::TuiLanguage::English,
                "re-login required (refresh token revoked or used); press o before codex login",
            ),
            (
                ccr_cli::managers::TuiLanguage::SimplifiedChinese,
                "需重新登录（refresh token 已吊销或已使用）；codex login 前先按 o 本地登出",
            ),
        ] {
            crate::tui::i18n::set_language(language);
            let text = plain_line_text(&quota_status_line(&app));
            assert!(text.contains(expected), "{text}");
            assert!(text.contains("[refresh_token_invalidated]"), "{text}");
            // 前缀只出现一次（标记被剥离后再以本地化文案呈现）
            assert_eq!(
                text.matches("需重新登录").count(),
                usize::from(expected.contains("需重新登录"))
            );
        }
        crate::tui::i18n::set_language(ccr_cli::managers::TuiLanguage::English);
    }

    #[test]
    fn full_usage_panel_states_cost_mode_once_and_never_truncates_the_source_url() {
        crate::tui::i18n::set_language(ccr_cli::managers::TuiLanguage::English);
        let (_dir, mut app) = presentation_fixture();
        set_usage_presentation_case(&mut app, UsagePresentationCase::Estimate);
        let source = ccr_codex::services::codex_usage_estimation::AUTH_PRICE_SOURCE;
        for width in [60, 68, 88] {
            let lines: Vec<String> = local_usage_lines(&app, 30, width)
                .iter()
                .map(plain_line_text)
                .collect();
            let text = lines.join("\n");
            assert_eq!(text.matches("Std assumed").count(), 1, "{text}");
            let url_lines = lines
                .iter()
                .filter(|line| line.contains("https://"))
                .count();
            assert_eq!(url_lines, usize::from(width >= 68), "{text}");
            if url_lines > 0 {
                assert!(text.contains(source), "{text}");
            }
            // 表格行内不再重复计价状态
            for line in lines
                .iter()
                .filter(|line| line.starts_with("5h ") && line.contains('$'))
            {
                assert!(!line.contains("assumed"), "{line}");
            }
        }
    }

    #[test]
    fn insufficient_samples_capacity_folds_and_missing_token_classes_are_omitted() {
        for language in [
            ccr_cli::managers::TuiLanguage::English,
            ccr_cli::managers::TuiLanguage::SimplifiedChinese,
        ] {
            crate::tui::i18n::set_language(language);
            let (_dir, mut app) = presentation_fixture();
            set_usage_presentation_case(&mut app, UsagePresentationCase::InsufficientSamples);
            let mut panel = app.usage_panel_data().expect("fixture usage panel");
            panel.estimate.joint = Default::default();
            let capacity: Vec<String> =
                capacity_lines(&panel).iter().map(plain_line_text).collect();
            assert_eq!(capacity.len(), 1, "{capacity:?}");
            assert!(
                capacity[0].contains(crate::tui_text!("joint", "联合"))
                    && capacity[0].contains(crate::tui_text!("samples<3", "样本不足")),
                "{capacity:?}"
            );
            panel.estimate.joint.status = CodexEstimateStatus::Stale;
            let capacity: Vec<String> =
                capacity_lines(&panel).iter().map(plain_line_text).collect();
            assert_eq!(capacity.len(), 2, "{capacity:?}");
            assert!(
                capacity[1].contains(crate::tui_text!("Joint remaining", "联合剩余")),
                "{capacity:?}"
            );

            for usage in [
                &mut panel.rolling.five_hour,
                &mut panel.rolling.seven_day,
                &mut panel.rolling.all_time,
            ] {
                usage.details.cache_read_tokens = None;
                usage.details.cache_write_tokens = None;
                usage.details.reasoning_tokens = None;
            }
            assert!(token_classification_lines(&panel).is_empty());
            panel.rolling.all_time.details.reasoning_tokens = Some(100_000);
            let rows: Vec<String> = token_classification_lines(&panel)
                .iter()
                .map(plain_line_text)
                .collect();
            assert_eq!(rows.len(), 3);
            assert!(rows.iter().all(|row| !row.contains("N/A")), "{rows:?}");
            assert!(rows[2].contains("100K"), "{rows:?}");
        }
        crate::tui::i18n::set_language(ccr_cli::managers::TuiLanguage::English);
    }
}
