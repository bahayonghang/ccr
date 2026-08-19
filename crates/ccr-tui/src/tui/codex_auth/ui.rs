// 🎨 Codex Auth TUI UI rendering
// Draws the Codex multi-account management interface

use super::app::{
    CodexAuthApp, CodexAuthUsagePanelData, CodexUsageAttributionState, CodexUsageScope,
    PreviewMetricWindow, QuotaPreviewCellState, QuotaState, UsageState,
};
use crate::tui::footer::{ShortcutHint, shortcut_line};
use crate::tui::overlay::{Overlay, render_overlay};
use crate::tui::theme;
use crate::tui::toast::ToastKind;
use ccr_cli::services::{CodexQuotaService, CodexUsageService};
use chrono::Local;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
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
    let label = localized_detail_label(label);
    Span::styled(
        pad_text(label, DETAIL_LABEL_WIDTH),
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
        "Last refresh:" => crate::tui_text!("Last refresh:", "最近刷新："),
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

fn preview_reset_detail_line(
    label: &str,
    preview_value: String,
    preview_style: Style,
    reset_value: String,
) -> Line<'static> {
    detail_spans_line(
        label,
        vec![
            Span::styled(preview_value, preview_style),
            Span::styled(
                crate::tui_text!("  Reset ", "  重置 "),
                theme::muted_style(),
            ),
            Span::styled(reset_value, theme::muted_style()),
        ],
    )
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
    if !app.accounts.is_empty() && inner.height >= 2 {
        let regions = account_list_regions(inner);
        app.sync_page_size(crate::tui::pagination::visible_page_size(
            regions.body.height,
        ));
    }

    let block = block.title_bottom(account_list_footer_line(app));
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

fn account_list_footer_line(app: &CodexAuthApp) -> Line<'static> {
    let selected_name = app
        .selected_account()
        .map(|account| account.name.clone())
        .unwrap_or_else(|| "-".to_string());
    let selected_style = app
        .selected_account()
        .map(|account| {
            if account.is_virtual {
                theme::warning_style()
            } else if account.is_current {
                theme::success_style()
            } else {
                Style::default()
                    .fg(theme::text())
                    .add_modifier(Modifier::BOLD)
            }
        })
        .unwrap_or_else(theme::muted_style);

    let preview_hint = if app.is_activation_gate_pending() {
        crate::tui_text!("  ·  preview expands after 1s ", "  ·  速览将在 1s 后展开 ")
    } else if app.selected_preview_entry().is_some() {
        crate::tui_text!("  ·  all-account preview ready ", "  ·  全账号速览已就绪 ")
    } else {
        crate::tui_text!("  ·  preview idle ", "  ·  速览待命 ")
    };

    Line::from(vec![
        Span::styled(
            crate::tui_text!(" Selected: ", " 已选择："),
            theme::muted_style(),
        ),
        Span::styled(selected_name, selected_style),
        Span::styled(
            crate::tui_text!("  ·  Legend: ", "  ·  图例："),
            theme::muted_style(),
        ),
        Span::styled(
            crate::tui_text!("🟢 fresh", "🟢 新鲜"),
            theme::success_style(),
        ),
        Span::styled(" · ", theme::muted_style()),
        Span::styled(
            crate::tui_text!("🟡 stale", "🟡 陈旧"),
            theme::warning_style(),
        ),
        Span::styled(" · ", theme::muted_style()),
        Span::styled(crate::tui_text!("🔴 old", "🔴 过期"), theme::error_style()),
        Span::styled(
            crate::tui_format!(
                "  ·  Page {}/{}  ·  {} accounts ",
                "  ·  第 {}/{} 页  ·  {} 个账号 ",
                app.current_page + 1,
                app.total_pages(),
                app.accounts.len()
            ),
            theme::muted_style(),
        ),
        Span::styled(preview_hint, theme::muted_style()),
    ])
    .alignment(Alignment::Left)
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
        QuotaWindow::Hourly => quota.hourly_reset_time?,
        QuotaWindow::Weekly => quota.weekly_reset_time?,
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

fn login_status_text(app: &CodexAuthApp) -> String {
    match &app.login_state {
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
    }
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

/// Draw usage panel (quota + local stats)
fn draw_usage_panel(f: &mut Frame, area: Rect, app: &CodexAuthApp) {
    let title = Line::from(vec![
        Span::styled("📊 ", theme::card_block_style()),
        Span::styled(
            crate::tui_text!("Usage & Quota", "用量与配额"),
            Style::default()
                .fg(theme::text())
                .add_modifier(Modifier::BOLD),
        ),
    ]);

    let mut content: Vec<Line> = Vec::new();

    // ── 配额刷新确认提示 ──
    if app.pending_quota_confirm {
        content.push(Line::from(Span::styled(
            crate::tui_text!(
                "  Query quota? Press y to confirm or any other key to cancel",
                "  确认查询配额？按 y 确认 / 其他键取消"
            ),
            theme::warning_style(),
        )));
    }

    content.push(Line::from(Span::styled(
        crate::tui_text!(
            "  The list shows all-account previews; this panel focuses on the selected account's full quota and local usage.",
            "  列表已用于全账号速览；此处聚焦当前选中账号的完整配额与本地 usage。"
        ),
        theme::muted_style(),
    )));

    // ── 配额余额区域 ──
    content.push(scope_line(
        "Quota scope:",
        crate::tui_text!("selected account", "所选账号"),
        theme::success_style(),
    ));

    match &app.quota_state {
        QuotaState::Idle => {
            let idle_message = if app.is_activation_gate_pending() {
                crate::tui_text!(
                    "  All-account previews and selected account details load after 1s",
                    "  停留 1s 后自动展开全账号速览，并同步带出当前账号详情"
                )
            } else {
                crate::tui_text!(
                    "  All-account previews cached; press b to force-refresh the selected account or r to reload accounts and local statistics",
                    "  已缓存全账号速览；按 b 强刷当前账号，按 r 刷新账号与本地统计"
                )
            };
            content.push(Line::from(Span::styled(idle_message, theme::muted_style())));
        }
        QuotaState::Loading { .. } if app.selected_quota().is_none() => {
            content.push(Line::from(Span::styled(
                crate::tui_text!(
                    "  Querying the selected account quota...",
                    "  正在查询当前账号配额..."
                ),
                theme::warning_style(),
            )));
        }
        QuotaState::Error { .. } if app.selected_quota().is_none() => {
            if let Some(err) = app.selected_quota_error() {
                content.push(Line::from(Span::styled(
                    crate::tui_format!("  Quota query failed: {}", "  配额查询失败：{}", err),
                    theme::error_style(),
                )));
                if is_refresh_token_reused_error(err) {
                    content.push(Line::from(Span::styled(
                        crate::tui_text!(
                            "  Token rotated; press R to attempt repair. If it still fails, log in again and save the account",
                            "  Token 已轮换，可按 R 尝试修复；仍失败请重新登录后保存账号"
                        ),
                        theme::warning_style(),
                    )));
                }
            }
        }
        _ => {
            if let Some(aq) = app.selected_quota() {
                if let Some(ref quota) = aq.quota {
                    let account_label = aq.email.as_deref().unwrap_or(&aq.account_name);
                    content.push(Line::from(vec![
                        Span::styled(crate::tui_text!("  Quota ", "  配额 "), theme::info_style()),
                        Span::styled(format!("({})", account_label), theme::muted_style()),
                    ]));

                    if app.is_selected_quota_loading() {
                        content.push(Line::from(Span::styled(
                            crate::tui_text!(
                                "  Refreshing the selected account quota...",
                                "  正在刷新选中账号配额..."
                            ),
                            theme::warning_style(),
                        )));
                    }

                    let h_color = theme::quota_color(quota.hourly_percentage);
                    let h_bar = progress_bar(quota.hourly_percentage, 10);
                    let h_reset = quota
                        .hourly_reset_time
                        .map(|t| {
                            crate::tui_format!(
                                "  Reset: {}",
                                "  重置：{}",
                                CodexQuotaService::format_reset_duration(t)
                            )
                        })
                        .unwrap_or_default();
                    content.push(Line::from(vec![
                        Span::styled(
                            crate::tui_text!("  5h limit: ", "  5h限额："),
                            Style::default().fg(theme::text()),
                        ),
                        Span::styled(h_bar, Style::default().fg(h_color)),
                        Span::styled(
                            format!(" {}%", quota.hourly_percentage),
                            Style::default().fg(h_color),
                        ),
                        Span::styled(h_reset, theme::muted_style()),
                    ]));

                    let w_color = theme::quota_color(quota.weekly_percentage);
                    let w_bar = progress_bar(quota.weekly_percentage, 10);
                    let w_reset = quota
                        .weekly_reset_time
                        .map(|t| {
                            let relative = CodexQuotaService::format_reset_duration(t);
                            let dt = chrono::DateTime::from_timestamp(t, 0)
                                .map(|d| d.with_timezone(&chrono::Local));
                            if let Some(local) = dt {
                                crate::tui_format!(
                                    "  Reset: {} ({})",
                                    "  重置：{}（{}）",
                                    relative,
                                    local.format("%m/%d %H:%M")
                                )
                            } else {
                                crate::tui_format!("  Reset: {}", "  重置：{}", relative)
                            }
                        })
                        .unwrap_or_default();
                    content.push(Line::from(vec![
                        Span::styled(
                            crate::tui_text!("  7d limit: ", "  7d限额："),
                            Style::default().fg(theme::text()),
                        ),
                        Span::styled(w_bar, Style::default().fg(w_color)),
                        Span::styled(
                            format!(" {}%", quota.weekly_percentage),
                            Style::default().fg(w_color),
                        ),
                        Span::styled(w_reset, theme::muted_style()),
                    ]));

                    if let Some(plan) = quota.plan_type.as_deref().or_else(|| {
                        app.selected_account()
                            .and_then(|account| account.plan_type.as_deref())
                    }) {
                        content.push(Line::from(vec![
                            Span::styled(
                                crate::tui_text!("  Plan: ", "  订阅："),
                                Style::default().fg(theme::text()),
                            ),
                            Span::styled(plan.to_string(), theme::info_style()),
                        ]));
                    }
                } else if let Some(ref err) = aq.error {
                    content.push(Line::from(Span::styled(
                        format!("  ⚠️ {}: {}", aq.account_name, err),
                        theme::error_style(),
                    )));
                }
            } else {
                content.push(Line::from(Span::styled(
                    crate::tui_text!(
                        "  No cached quota for the selected account; querying on demand...",
                        "  选中账号暂无配额缓存，正在按需查询..."
                    ),
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
    if let Some(panel) = app.usage_panel_data() {
        let (scope_label, scope_style) = usage_scope_badge(&panel);
        content.push(scope_line("Usage scope:", scope_label, scope_style));
        content.push(scope_line(
            "Attribution:",
            usage_attribution_label(panel.attribution_state),
            usage_attribution_style(panel.attribution_state),
        ));
        if let Some(reason) = &panel.fallback_reason {
            content.push(Line::from(Span::styled(
                crate::tui_format!("  Note: {}", "  说明：{}", reason),
                theme::warning_style(),
            )));
        }
        content.extend(usage_digest_lines(&panel));
    } else {
        if app.is_activation_gate_pending() {
            content.push(Line::from(Span::styled(
                crate::tui_text!(
                    "  Local usage and list previews load after 1s",
                    "  停留 1s 后自动加载本地 usage，并与列表速览一起就位"
                ),
                theme::muted_style(),
            )));
        } else {
            match &app.usage_state {
                UsageState::NoData => {
                    content.push(Line::from(Span::styled(
                        crate::tui_text!("  No local usage data", "  暂无本地使用数据"),
                        theme::muted_style(),
                    )));
                }
                UsageState::Error(err) => {
                    content.push(Line::from(Span::styled(
                        crate::tui_format!(
                            "  Failed to load statistics: {}",
                            "  统计加载失败：{}",
                            err
                        ),
                        theme::error_style(),
                    )));
                }
                UsageState::Loaded(_) => {}
                UsageState::Loading => {
                    content.push(Line::from(Span::styled(
                        crate::tui_text!("  Loading...", "  加载中..."),
                        theme::muted_style(),
                    )));
                }
            }
        }
    }

    let panel = Paragraph::new(content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::border()))
                .title(title)
                .title_style(Style::default().fg(theme::codex())),
        )
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });

    f.render_widget(panel, area);
}

/// 百分比颜色：使用主题的5级渐变
#[allow(dead_code)]
fn percent_color(pct: i32) -> Color {
    theme::quota_color(pct)
}

/// 生成文本进度条
fn progress_bar(pct: i32, width: usize) -> String {
    let filled = ((pct as usize) * width / 100).min(width);
    let empty = width - filled;
    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
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
    let preview_five = app.preview_cell_for_account(&account.name, PreviewMetricWindow::FiveHour);
    let preview_seven = app.preview_cell_for_account(&account.name, PreviewMetricWindow::SevenDay);
    let preview_five_style = preview_cell_style(&preview_five, false);
    let preview_seven_style = preview_cell_style(&preview_seven, false);
    let (plan_text, plan_style) = quota_plan_for_account(app, account)
        .map(|plan| (plan, theme::info_style()))
        .unwrap_or_else(|| ("-".to_string(), theme::muted_style()));
    let (hourly_reset, weekly_reset) = app
        .selected_quota()
        .and_then(|quota| quota.quota.as_ref())
        .map(|quota| {
            (
                crate::tui::codex_auth::app::CodexAuthApp::quota_reset_detail_text(
                    quota.hourly_reset_time,
                ),
                crate::tui::codex_auth::app::CodexAuthApp::quota_reset_detail_text(
                    quota.weekly_reset_time,
                ),
            )
        })
        .unwrap_or_else(|| ("-".to_string(), "-".to_string()));

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
        preview_reset_detail_line("5h:", preview_five.text, preview_five_style, hourly_reset),
        preview_reset_detail_line("7d:", preview_seven.text, preview_seven_style, weekly_reset),
    ]
}

fn usage_digest_lines(panel: &CodexAuthUsagePanelData) -> Vec<Line<'static>> {
    let usage = &panel.rolling;
    let five_total = usage.five_hour.total_input_tokens + usage.five_hour.total_output_tokens;
    let seven_total = usage.seven_day.total_input_tokens + usage.seven_day.total_output_tokens;
    let all_time = usage.all_time.total_input_tokens + usage.all_time.total_output_tokens;

    let top_model = panel
        .top_model
        .as_ref()
        .map(|top| {
            crate::tui_format!(
                "  Top model: {} ({}, {} req)",
                "  主要模型：{}（{}，{} 次请求）",
                top.model,
                CodexUsageService::format_tokens(top.total_tokens),
                top.total_requests
            )
        })
        .unwrap_or_else(|| crate::tui_text!("  Top model: -", "  主要模型：-").to_string());

    vec![
        Line::from(crate::tui_format!(
            "  5 hours: {} tokens ({} requests)",
            "  5小时：{} tokens（{} 请求）",
            CodexUsageService::format_tokens(five_total),
            usage.five_hour.total_requests
        )),
        Line::from(crate::tui_format!(
            "  7 days:  {} tokens ({} requests)",
            "  7天：  {} tokens（{} 请求）",
            CodexUsageService::format_tokens(seven_total),
            usage.seven_day.total_requests
        )),
        Line::from(crate::tui_format!(
            "  All time: {} tokens ({} requests)",
            "  全时段：{} tokens（{} 请求）",
            CodexUsageService::format_tokens(all_time),
            usage.all_time.total_requests
        )),
        Line::from(top_model),
    ]
}

fn is_refresh_token_reused_error(message: &str) -> bool {
    let lower = message.to_ascii_lowercase();
    lower.contains("refresh_token_reused") || lower.contains("invalid_grant")
}

/// Draw help bar (overlay-aware)
fn draw_help_bar(f: &mut Frame, area: Rect, app: &CodexAuthApp) {
    let help_text = match &app.overlay {
        Some(Overlay::Confirm { .. }) => {
            crate::tui_text!("y confirm delete | n/Esc cancel", "y 确认删除 | n/Esc 取消")
        }
        Some(Overlay::ImportCodexConfirm { .. }) => {
            crate::tui_text!("y confirm | n/Esc cancel", "y 确认 | n/Esc 取消")
        }
        Some(Overlay::Input { .. }) => {
            crate::tui_text!("Enter confirm | Esc cancel", "Enter 确认 | Esc 取消")
        }
        Some(Overlay::RenameInput { .. }) => crate::tui_text!(
            "Enter save | Ctrl+F overwrite | Esc cancel",
            "Enter 保存 | Ctrl+F 强制覆盖 | Esc 取消"
        ),
        None => crate::tui_text!(
            "↑/k up | ↓/j down | Enter switch | s save current | n rename | d delete | r refresh | R repair | b quota | Ctrl+L language | q quit",
            "↑/k 上移 | ↓/j 下移 | Enter 切换 | s 保存当前 | n 重命名 | d 删除 | r 刷新 | R 修复 | b 配额 | Ctrl+L 语言 | q 退出"
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
                .constraints([Constraint::Min(10), Constraint::Length(14)])
                .split(content_area);

            draw_account_list_with_status(f, content_chunks[0], app);
            draw_usage_panel(f, content_chunks[1], app);
        }
        crate::tui::theme::ViewportMode::Wide => {
            let columns = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
                .split(content_area);

            draw_account_list_with_status(f, columns[0], app);

            let right = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(13), Constraint::Min(12)])
                .split(columns[1]);
            draw_account_snapshot_panel(f, right[0], app);
            draw_usage_panel(f, right[1], app);
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
    let title = crate::tui_format!(" Accounts · {} ", " 账号列表 · {} ", login_status_text(app));
    render_account_list_panel(f, area, app, title);
}

fn draw_account_snapshot_panel(f: &mut Frame, area: Rect, app: &CodexAuthApp) {
    let lines = app
        .selected_account()
        .map(|account| account_snapshot_lines(app, account))
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
                .border_style(Style::default().fg(theme::codex()))
                .title(crate::tui_text!(" Focus ", " 当前焦点 "))
                .title_style(theme::codex_style()),
        )
        .wrap(Wrap { trim: true });

    f.render_widget(panel, area);
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

    let mut line = shortcut_line(&hints, theme::codex());
    if let Some(toast) = app.toasts.active() {
        let style = match toast.kind {
            ToastKind::Success => theme::success_style(),
            ToastKind::Error => theme::error_style(),
            ToastKind::Warning => theme::warning_style(),
            ToastKind::Info => theme::info_style(),
        };
        line.spans.insert(
            0,
            Span::styled("  │  ", Style::default().fg(theme::muted())),
        );
        line.spans
            .insert(0, Span::styled(toast.message.clone(), style));
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

fn scope_line(label: &str, value: impl Into<String>, style: Style) -> Line<'static> {
    detail_line(label, value.into(), style)
}

fn usage_scope_badge(panel: &CodexAuthUsagePanelData) -> (String, Style) {
    match &panel.scope {
        CodexUsageScope::GlobalRuntime => (
            crate::tui_text!("global runtime", "全局运行时").to_string(),
            theme::info_style(),
        ),
        CodexUsageScope::AccountAttributed { account_name } => (
            crate::tui_format!("attributed to {}", "归因到 {}", account_name),
            theme::success_style(),
        ),
    }
}

fn usage_attribution_label(state: CodexUsageAttributionState) -> &'static str {
    match state {
        CodexUsageAttributionState::GlobalOnly => {
            crate::tui_text!("global aggregate", "全局汇总")
        }
        CodexUsageAttributionState::AccountAttributed => {
            crate::tui_text!("CCR ledger matched", "CCR 账本已匹配")
        }
        CodexUsageAttributionState::VirtualAccount => {
            crate::tui_text!("unsaved runtime login", "未保存的运行时登录")
        }
        CodexUsageAttributionState::UnattributedFallback => {
            crate::tui_text!("global fallback", "全局回退")
        }
    }
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
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use ratatui::{Terminal, backend::TestBackend};
    use std::path::PathBuf;

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
                },
            },
        );
        let lines = account_snapshot_lines(&app, &sample_account());

        assert!(plain_line_text(&lines[0]).contains("codexcn"));
        assert!(plain_line_text(&lines[3]).contains("PLUS"));
        assert!(plain_line_text(&lines[4]).contains("Saved at:"));
        assert!(plain_line_text(&lines[5]).contains("Last refresh:"));
        assert!(plain_line_text(&lines[6]).contains("5h:"));
        assert!(plain_line_text(&lines[6]).contains("95%"));
        assert!(plain_line_text(&lines[6]).contains("Reset"));
        assert!(plain_line_text(&lines[6]).contains("m"));
        assert!(plain_line_text(&lines[7]).contains("7d:"));
        assert!(plain_line_text(&lines[7]).contains("33%"));
        assert!(plain_line_text(&lines[7]).contains("Reset"));
        assert!(plain_line_text(&lines[7]).contains("m"));
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
                },
            },
        );

        let mut terminal = Terminal::new(TestBackend::new(100, 13)).unwrap();
        terminal
            .draw(|frame| draw_account_snapshot_panel(frame, frame.area(), &app))
            .unwrap();

        let rendered = buffer_text(terminal.backend());
        assert!(rendered.contains("7d:"), "{rendered}");
        assert!(rendered.contains("Reset"), "{rendered}");
        assert!(rendered.contains("5d6h13m"), "{rendered}");
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
        };

        let lines: Vec<String> = usage_digest_lines(&panel)
            .into_iter()
            .map(|line| plain_line_text(&line))
            .collect();

        assert!(
            lines
                .iter()
                .any(|line| line.contains("All time: 60.0K tokens"))
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
                },
            )]),
        };
        app.usage_state = UsageState::Loaded(crate::tui::codex_auth::app::CodexUsageDataset {
            global: ccr_cli::services::CodexUsageService::compute_rolling_usage_for_records(&[
                ccr_cli::services::CodexUsageRecord {
                    session_id: "global-only".to_string(),
                    timestamp: Utc::now(),
                    input_tokens: 1200,
                    output_tokens: 240,
                    model: Some("gpt-5.4".to_string()),
                },
            ]),
            records: vec![ccr_cli::services::CodexUsageRecord {
                session_id: "global-only".to_string(),
                timestamp: Utc::now(),
                input_tokens: 1200,
                output_tokens: 240,
                model: Some("gpt-5.4".to_string()),
            }],
        });

        let mut terminal = Terminal::new(TestBackend::new(90, 18)).unwrap();
        terminal
            .draw(|frame| draw_usage_panel(frame, frame.area(), &app))
            .unwrap();

        let rendered = buffer_text(terminal.backend());
        let compact = compact_text(&rendered);
        assert!(compact.contains("Usage&Quota"), "{rendered}");
        assert!(compact.contains("Quotascope:selectedaccount"), "{rendered}");
        assert!(compact.contains("Reset:"), "{rendered}");
        assert!(compact.contains("7dlimit:"), "{rendered}");
        assert!(compact.contains("Attribution:globalfallback"), "{rendered}");
        assert!(compact.contains("Note:CCR"), "{rendered}");
    }
}
