// 🎨 OpenCode Auth TUI UI rendering
// Draws the OpenCode openai account management interface with local usage stats

use super::app::{
    OpenCodeAuthApp, OpenCodeAuthUsagePanelData, OpenCodeUsageAttributionState, OpenCodeUsageState,
    PreviewMetricWindow, QuotaPreviewCellState, QuotaState,
};
use crate::tui::overlay::{Overlay, render_overlay};
use crate::tui::theme;
use crate::tui::toast::ToastKind;
use ccr_cli::models::{OpenCodeAuthItem, OpenCodeLoginState};
use ccr_cli::services::{OpenCodeQuotaService, OpenCodeUsageService};
use chrono::{Local, Utc};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap},
};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

const ACCOUNT_COLUMN_SPACING: u16 = 1;
const DETAIL_LABEL_WIDTH: usize = 12;

pub fn draw(f: &mut Frame, app: &mut OpenCodeAuthApp) {
    let background = Block::default().style(theme::background_style());
    f.render_widget(background, f.area());

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(12),
            Constraint::Length(3),
            Constraint::Length(2),
        ])
        .split(f.area());

    draw_title(f, chunks[0], app);
    draw_account_list_with_status(f, chunks[1], app);
    draw_usage_panel(f, chunks[2], app);
    draw_status_bar(f, chunks[3], app);
    draw_help_bar(f, chunks[4], app);

    if let Some(overlay) = &app.overlay {
        render_overlay(f, overlay);
    }
}

pub fn draw_embedded(
    f: &mut Frame,
    app: &mut OpenCodeAuthApp,
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
        .map(|err| format!("OpenCode Auth 初始化失败\n\n{err}"))
        .unwrap_or_else(|| "正在初始化 OpenCode Auth...".to_string());

    let panel = Paragraph::new(message)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::border()))
                .title(" 🔐 OpenCode Auth ")
                .title_style(Style::default().fg(theme::opencode())),
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
        let status = Paragraph::new(if error.is_some() {
            "初始化失败"
        } else {
            "加载中"
        })
        .style(if error.is_some() {
            theme::error_style()
        } else {
            theme::info_style()
        })
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(theme::border()))
                .title(" Keys ")
                .title_style(Style::default().fg(theme::opencode())),
        );
        f.render_widget(status, footer_area);
    }
}

fn draw_title(f: &mut Frame, area: Rect, app: &OpenCodeAuthApp) {
    let title = Paragraph::new(vec![Line::from(vec![
        Span::styled(
            " 🔐 OpenCode 账号管理 ",
            Style::default()
                .fg(theme::opencode())
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(login_status_text(app), login_status_style(&app.login_state)),
    ])])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::border()))
            .title(" CCR ")
            .title_style(Style::default().fg(theme::opencode())),
    )
    .alignment(Alignment::Center);

    f.render_widget(title, area);
}

fn draw_account_list_with_status(f: &mut Frame, area: Rect, app: &mut OpenCodeAuthApp) {
    let title = format!(" 🔐 账号列表 · {} ", login_status_text(app));
    render_account_list_panel(f, area, app, title);
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
            .position(|candidate| *candidate == column)
            .and_then(|index| self.resolved_widths.get(index))
            .copied()
            .unwrap_or(0)
    }

    fn account_name_width(&self, account: &OpenCodeAuthItem) -> usize {
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

fn render_account_list_panel(f: &mut Frame, area: Rect, app: &mut OpenCodeAuthApp, title: String) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::opencode()))
        .title(title)
        .title_style(theme::opencode_style());
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
        let empty = Paragraph::new(" 暂未发现可切换的 OpenCode 账号")
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

fn account_list_footer_line(app: &OpenCodeAuthApp) -> Line<'static> {
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
        "  ·  速览将在 1s 后展开 "
    } else if app.selected_preview_entry().is_some() {
        "  ·  全账号速览已就绪 "
    } else {
        "  ·  速览待命 "
    };

    Line::from(vec![
        Span::styled(" Selected: ", theme::muted_style()),
        Span::styled(selected_name, selected_style),
        Span::styled("  ·  Legend: ", theme::muted_style()),
        Span::styled("🟢 fresh", theme::success_style()),
        Span::styled(" · ", theme::muted_style()),
        Span::styled("🟡 stale", theme::warning_style()),
        Span::styled(" · ", theme::muted_style()),
        Span::styled("🔴 old", theme::error_style()),
        Span::styled(
            format!(
                "  ·  Page {}/{}  ·  {} accounts ",
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
            Constraint::Length(10),
        ],
        inner_width,
    )
}

fn render_account_list_header(f: &mut Frame, area: Rect, layout: &AccountTableLayout) {
    let header = Table::new(
        [Row::new(layout.columns.iter().map(account_header_cell))],
        layout.widths.clone(),
    )
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
    app: &OpenCodeAuthApp,
    layout: &AccountTableLayout,
) {
    let selected_style = theme::selected_row_style();

    let rows = app
        .current_page_accounts()
        .iter()
        .enumerate()
        .map(|(index, account)| {
            let row_style = if index == app.selected_index {
                selected_style
            } else {
                Style::default()
            };

            Row::new(layout.columns.iter().map(|column| {
                account_cell(account, app, *column, layout, index == app.selected_index)
            }))
            .style(row_style)
            .height(1)
        });

    let table = Table::new(rows, layout.widths.clone()).column_spacing(ACCOUNT_COLUMN_SPACING);
    f.render_widget(table, area);
}

fn account_header_cell(column: &AccountColumn) -> Cell<'static> {
    let label = match column {
        AccountColumn::Account => "账号",
        AccountColumn::Email => "邮箱",
        AccountColumn::Plan => "类型",
        AccountColumn::QuotaSummary => "配额",
        AccountColumn::HourlyQuota => "5h",
        AccountColumn::WeeklyQuota => "7d",
        AccountColumn::ExpiresAt => "到期",
    };

    Cell::from(label.to_string())
}

fn account_cell(
    account: &OpenCodeAuthItem,
    app: &OpenCodeAuthApp,
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

            let mut spans = vec![Span::styled(
                truncate_text(&account.name, layout.account_name_width(account)),
                name_style,
            )];

            if account.is_virtual {
                spans.push(Span::styled(
                    " *",
                    theme::warning_style().add_modifier(Modifier::ITALIC),
                ));
            }

            Cell::from(Line::from(spans))
        }
        AccountColumn::Email => Cell::from(Line::from(Span::styled(
            truncate_text(
                account.email.as_deref().unwrap_or("-"),
                layout.text_width(AccountColumn::Email),
            ),
            if is_selected {
                theme::selected_row_style()
            } else {
                theme::info_style()
            },
        ))),
        AccountColumn::Plan => Cell::from(Line::from(Span::styled(
            truncate_text(
                account.plan_type.as_deref().unwrap_or("-"),
                layout.text_width(AccountColumn::Plan),
            ),
            if is_selected {
                theme::selected_row_style()
            } else {
                theme::muted_style()
            },
        ))),
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
    app: &OpenCodeAuthApp,
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
    Some(OpenCodeQuotaService::format_reset_duration(ts))
}

/// 把百分比文案 (`"52%"` / `"ERR"` / `"…"`) 与重置时间组合为 `"52% (3h11m)"`。
/// 仅在百分比是就绪值且 reset 存在时拼括号;加载/错误态直接返回原文本。
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

fn draw_account_snapshot_panel(f: &mut Frame, area: Rect, app: &OpenCodeAuthApp) {
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
                .border_style(Style::default().fg(theme::opencode()))
                .title(" Focus ")
                .title_style(theme::opencode_style()),
        )
        .wrap(Wrap { trim: true });

    f.render_widget(panel, area);
}

fn account_snapshot_lines(app: &OpenCodeAuthApp, account: &OpenCodeAuthItem) -> Vec<Line<'static>> {
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
    let (expires_text, expires_style) = format_expires_at(account);
    let preview_five = app.preview_cell_for_account(&account.name, PreviewMetricWindow::FiveHour);
    let preview_seven = app.preview_cell_for_account(&account.name, PreviewMetricWindow::SevenDay);
    let preview_five_style = preview_cell_style(&preview_five, false);
    let preview_seven_style = preview_cell_style(&preview_seven, false);

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
        detail_optional_line(
            "OpenAI ID:",
            account.account_id.as_deref(),
            theme::info_style(),
        ),
        detail_optional_line("Email:", account.email.as_deref(), theme::info_style()),
        detail_optional_line("Plan:", account.plan_type.as_deref(), theme::muted_style()),
        detail_line(
            "Saved at:",
            format_saved_at(account),
            Style::default().fg(theme::text()),
        ),
        detail_line("Expires:", expires_text, expires_style),
        preview_reset_detail_line(
            "5h:",
            preview_five.text,
            preview_five_style,
            app.selected_quota()
                .and_then(|quota| quota.quota.as_ref())
                .map(|quota| {
                    crate::tui::opencode_auth::app::OpenCodeAuthApp::quota_reset_detail_text(
                        quota.hourly_reset_time,
                    )
                })
                .unwrap_or_else(|| "-".to_string()),
        ),
        preview_reset_detail_line(
            "7d:",
            preview_seven.text,
            preview_seven_style,
            app.selected_quota()
                .and_then(|quota| quota.quota.as_ref())
                .map(|quota| {
                    crate::tui::opencode_auth::app::OpenCodeAuthApp::quota_reset_detail_text(
                        quota.weekly_reset_time,
                    )
                })
                .unwrap_or_else(|| "-".to_string()),
        ),
    ]
}

fn detail_label_span(label: &str) -> Span<'static> {
    Span::styled(
        format!("{label:<DETAIL_LABEL_WIDTH$}"),
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
            Span::styled("  Reset ", theme::muted_style()),
            Span::styled(reset_value, theme::muted_style()),
        ],
    )
}

fn detail_spans_line(label: &str, mut spans: Vec<Span<'static>>) -> Line<'static> {
    let mut all = vec![detail_label_span(label)];
    all.append(&mut spans);
    Line::from(all)
}

fn draw_status_bar(f: &mut Frame, area: Rect, app: &OpenCodeAuthApp) {
    let (message, style) = if let Some(toast) = app.toasts.active() {
        let style = match toast.kind {
            ToastKind::Success => theme::success_style(),
            ToastKind::Error => theme::error_style(),
            ToastKind::Warning => theme::warning_style(),
            ToastKind::Info => theme::info_style(),
        };
        (toast.message.as_str(), style)
    } else {
        ("就绪", theme::success_style())
    };

    let status = Paragraph::new(message).style(style).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::border()))
            .title(" 状态 ")
            .title_style(Style::default().fg(theme::opencode())),
    );

    f.render_widget(status, area);
}

fn draw_usage_panel(f: &mut Frame, area: Rect, app: &OpenCodeAuthApp) {
    let title = Line::from(vec![
        Span::styled("📊 ", theme::card_block_style()),
        Span::styled(
            "Usage & Quota",
            Style::default()
                .fg(theme::text())
                .add_modifier(Modifier::BOLD),
        ),
    ]);

    let mut content: Vec<Line> = Vec::new();

    content.push(Line::from(Span::styled(
        "  列表已用于全账号速览；此处聚焦当前选中账号的完整配额，并明确标注本地 usage 的 provider 限制。",
        theme::muted_style(),
    )));

    content.push(scope_line(
        "Quota scope:",
        "selected account",
        theme::success_style(),
    ));

    match &app.quota_state {
        QuotaState::Idle => {
            let idle_message = if app.is_activation_gate_pending() {
                "  ⏳ 停留 1s 后自动展开全账号速览，并同步带出当前账号详情"
            } else {
                "  ⏳ 已缓存全账号速览；按 r 强刷当前账号与本地统计"
            };
            content.push(Line::from(Span::styled(idle_message, theme::muted_style())));
        }
        QuotaState::Loading { .. } if app.selected_quota().is_none() => {
            content.push(Line::from(Span::styled(
                "  ⏳ 正在查询当前账号配额...",
                theme::warning_style(),
            )));
        }
        QuotaState::Error { .. } if app.selected_quota().is_none() => {
            if let Some(err) = app.selected_quota_error() {
                content.push(Line::from(Span::styled(
                    format!("  ⚠️ 配额查询失败: {}", err),
                    theme::error_style(),
                )));
            }
        }
        _ => {
            if let Some(aq) = app.selected_quota() {
                if let Some(ref quota) = aq.quota {
                    let account_label = aq.email.as_deref().unwrap_or(&aq.account_name);
                    content.push(Line::from(vec![
                        Span::styled("  配额 ", theme::info_style()),
                        Span::styled(format!("({})", account_label), theme::muted_style()),
                    ]));

                    if app.is_selected_quota_loading() {
                        content.push(Line::from(Span::styled(
                            "  ⏳ 正在刷新选中账号配额...",
                            theme::warning_style(),
                        )));
                    }

                    let h_color = theme::quota_color(quota.hourly_percentage);
                    let h_bar = progress_bar(quota.hourly_percentage, 10);
                    let h_reset = quota
                        .hourly_reset_time
                        .map(|value| {
                            format!(
                                "  重置: {}",
                                OpenCodeQuotaService::format_reset_duration(value)
                            )
                        })
                        .unwrap_or_default();
                    content.push(Line::from(vec![
                        Span::styled("  5h限额: ", Style::default().fg(theme::text())),
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
                        .map(|value| {
                            let relative = OpenCodeQuotaService::format_reset_duration(value);
                            let dt = chrono::DateTime::from_timestamp(value, 0)
                                .map(|dt| dt.with_timezone(&chrono::Local));
                            if let Some(local) = dt {
                                format!("  重置: {} ({})", relative, local.format("%m/%d %H:%M"))
                            } else {
                                format!("  重置: {}", relative)
                            }
                        })
                        .unwrap_or_default();
                    content.push(Line::from(vec![
                        Span::styled("  周限额: ", Style::default().fg(theme::text())),
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
                            Span::styled("  订阅: ", Style::default().fg(theme::text())),
                            Span::styled(plan.to_string(), theme::info_style()),
                        ]));
                    }
                } else if let Some(ref err) = aq.error {
                    content.push(Line::from(Span::styled(
                        format!("  ⚠️ {}: {}", aq.account_name, err),
                        theme::error_style(),
                    )));
                }
            }
        }
    }

    content.push(Line::from(Span::styled(
        "  ────────────────────────────────",
        theme::muted_style(),
    )));

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
                format!("  Note: {}", reason),
                theme::warning_style(),
            )));
        }
        content.extend(usage_digest_lines(&panel));
    } else {
        if app.is_activation_gate_pending() {
            content.push(Line::from(Span::styled(
                "  ⏳ 停留 1s 后自动加载本地 openai usage，并与列表速览一起就位",
                theme::muted_style(),
            )));
        } else {
            match &app.usage_state {
                OpenCodeUsageState::NoData => {
                    content.push(Line::from(Span::styled(
                        "  📭 暂无本地 openai usage 数据",
                        theme::muted_style(),
                    )));
                    content.push(Line::from(Span::styled(
                        "  说明: 仅统计 OpenCode message 表里 providerID=openai 的 assistant 消息",
                        theme::muted_style(),
                    )));
                }
                OpenCodeUsageState::Error(err) => {
                    content.push(Line::from(Span::styled(
                        format!("  ⚠️ 统计加载失败: {}", err),
                        theme::error_style(),
                    )));
                }
                OpenCodeUsageState::Loading => {
                    content.push(Line::from(Span::styled(
                        "  ⏳ 正在加载本地 openai usage...",
                        theme::muted_style(),
                    )));
                }
                OpenCodeUsageState::Loaded(_) => {}
            }
        }
    }

    let panel = Paragraph::new(content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::border()))
                .title(title)
                .title_style(Style::default().fg(theme::opencode())),
        )
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });

    f.render_widget(panel, area);
}

#[allow(dead_code)]
fn percent_color(pct: i32) -> Color {
    theme::quota_color(pct)
}

fn progress_bar(pct: i32, width: usize) -> String {
    let filled = ((pct as usize) * width / 100).min(width);
    let empty = width - filled;
    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}

fn usage_scope_badge(panel: &OpenCodeAuthUsagePanelData) -> (String, Style) {
    (
        format!("{} provider", panel.provider_label),
        theme::info_style(),
    )
}

fn usage_attribution_label(state: OpenCodeUsageAttributionState) -> &'static str {
    match state {
        OpenCodeUsageAttributionState::ProviderGlobal => "provider aggregate",
        OpenCodeUsageAttributionState::CurrentSavedSelection => {
            "current selection · provider aggregate"
        }
        OpenCodeUsageAttributionState::SavedSelectionFallback => {
            "saved selection · provider fallback"
        }
        OpenCodeUsageAttributionState::VirtualCurrentLogin => {
            "unsaved current login · provider aggregate"
        }
    }
}

fn usage_attribution_style(state: OpenCodeUsageAttributionState) -> Style {
    match state {
        OpenCodeUsageAttributionState::ProviderGlobal => theme::info_style(),
        OpenCodeUsageAttributionState::CurrentSavedSelection
        | OpenCodeUsageAttributionState::SavedSelectionFallback
        | OpenCodeUsageAttributionState::VirtualCurrentLogin => theme::warning_style(),
    }
}

fn usage_digest_lines(panel: &OpenCodeAuthUsagePanelData) -> Vec<Line<'static>> {
    let usage = &panel.rolling;
    let five_total = usage.five_hour.total_input_tokens + usage.five_hour.total_output_tokens;
    let seven_total = usage.seven_day.total_input_tokens + usage.seven_day.total_output_tokens;
    let all_time = usage.all_time.total_input_tokens + usage.all_time.total_output_tokens;

    let top_model = panel
        .top_model
        .as_ref()
        .map(|top| {
            format!(
                "  Top model: {} ({}, {} req)",
                top.model,
                OpenCodeUsageService::format_tokens(top.total_tokens),
                top.total_requests
            )
        })
        .unwrap_or_else(|| "  Top model: -".to_string());

    vec![
        Line::from(format!(
            "  5小时: {} tokens ({} 请求)",
            OpenCodeUsageService::format_tokens(five_total),
            usage.five_hour.total_requests
        )),
        Line::from(format!(
            "  7天:   {} tokens ({} 请求)",
            OpenCodeUsageService::format_tokens(seven_total),
            usage.seven_day.total_requests
        )),
        Line::from(format!(
            "  All time: {} tokens ({} 请求)",
            OpenCodeUsageService::format_tokens(all_time),
            usage.all_time.total_requests
        )),
        Line::from(format!(
            "  Records: {} local assistant messages",
            panel.record_count
        )),
        Line::from(top_model),
    ]
}

fn draw_help_bar(f: &mut Frame, area: Rect, app: &OpenCodeAuthApp) {
    let help_text = match &app.overlay {
        Some(Overlay::Confirm { .. }) => "y 确认删除 | n/Esc 取消",
        Some(Overlay::ImportCodexConfirm { .. }) => "y 确认导入 | n/Esc 取消",
        Some(Overlay::Input { .. }) => "Enter 确认 | Esc 取消",
        Some(Overlay::RenameInput { .. }) => "Enter 保存 | Esc 取消",
        None => {
            "↑/k 上移 | ↓/j 下移 | Enter 切换 | s 保存当前 | i 导入 Codex | d 删除 | r 刷新账号/统计 | q 退出"
        }
    };

    let help = Paragraph::new(help_text)
        .style(theme::muted_style())
        .alignment(Alignment::Center);

    f.render_widget(help, area);
}

fn draw_footer_strip(f: &mut Frame, area: Rect, app: &OpenCodeAuthApp) {
    let message = if let Some(toast) = app.toasts.active() {
        format!(
            "{}  │  Tab/Shift+Tab switch  │  ↑↓/jk select  │  Enter switch  │  s save  │  i import  │  d delete  │  r refresh  │  q quit",
            toast.message
        )
    } else {
        "Tab/Shift+Tab switch  │  ↑↓/jk select  │  Enter switch  │  s save  │  i import  │  d delete  │  r refresh  │  q quit"
            .to_string()
    };

    let help = Paragraph::new(message)
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(theme::border()))
                .title(" Keys ")
                .title_style(Style::default().fg(theme::muted())),
        )
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

    f.render_widget(help, area);
}

fn scope_line(label: &str, value: impl Into<String>, style: Style) -> Line<'static> {
    detail_line(label, value.into(), style)
}

fn login_status_text(app: &OpenCodeAuthApp) -> String {
    match &app.login_state {
        OpenCodeLoginState::NotLoggedIn => "未登录".to_string(),
        OpenCodeLoginState::LoggedInUnsaved => "已登录 (未保存)".to_string(),
        OpenCodeLoginState::LoggedInSaved(name) => format!("已登录: {}", name),
    }
}

fn login_status_style(state: &OpenCodeLoginState) -> Style {
    match state {
        OpenCodeLoginState::NotLoggedIn => theme::error_style(),
        OpenCodeLoginState::LoggedInUnsaved => theme::warning_style(),
        OpenCodeLoginState::LoggedInSaved(_) => theme::success_style(),
    }
}

fn format_saved_at(account: &OpenCodeAuthItem) -> String {
    account
        .saved_at
        .map(|saved_at| {
            saved_at
                .with_timezone(&Local)
                .format("%Y-%m-%d")
                .to_string()
        })
        .unwrap_or_else(|| "-".to_string())
}

fn format_expires_at(account: &OpenCodeAuthItem) -> (String, Style) {
    match account.expires_at {
        Some(expires_at) => {
            let expired = expires_at <= Utc::now();
            let text = expires_at
                .with_timezone(&Local)
                .format("%Y-%m-%d")
                .to_string();
            if expired {
                (text, theme::error_style())
            } else {
                (text, theme::success_style())
            }
        }
        None => ("-".to_string(), theme::muted_style()),
    }
}

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

fn truncate_text(value: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }

    if value.width() <= max_width {
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

#[cfg(test)]
mod tests {
    use super::*;
    use ccr_cli::models::OpenCodeAuthItem;
    use chrono::{Duration, TimeZone, Utc};
    use indexmap::IndexMap;
    use ratatui::{Terminal, backend::TestBackend};
    use std::path::PathBuf;

    fn sample_account() -> OpenCodeAuthItem {
        OpenCodeAuthItem {
            name: "primary".to_string(),
            account_id: Some("acc-primary".to_string()),
            email: Some("use***@example.com".to_string()),
            plan_type: Some("PLUS".to_string()),
            is_current: true,
            is_virtual: false,
            saved_at: Some(Utc.with_ymd_and_hms(2026, 4, 14, 12, 0, 0).unwrap()),
            last_used: Some(Utc.with_ymd_and_hms(2026, 4, 14, 18, 30, 0).unwrap()),
            expires_at: Some(Utc::now() + Duration::days(7)),
        }
    }

    fn sample_usage_panel() -> OpenCodeAuthUsagePanelData {
        let mut usage = ccr_cli::services::OpenCodeRollingUsage::default();
        usage.five_hour.total_input_tokens = 1_000;
        usage.five_hour.total_output_tokens = 200;
        usage.five_hour.total_requests = 2;
        usage.seven_day.total_input_tokens = 10_000;
        usage.seven_day.total_output_tokens = 1_500;
        usage.seven_day.total_requests = 8;
        usage.all_time.total_input_tokens = 45_000;
        usage.all_time.total_output_tokens = 5_500;
        usage.all_time.total_requests = 21;

        OpenCodeAuthUsagePanelData {
            provider_label: "openai".to_string(),
            attribution_state: OpenCodeUsageAttributionState::SavedSelectionFallback,
            rolling: usage,
            record_count: 21,
            top_model: Some(super::super::app::OpenCodeUsageTopModel {
                model: "gpt-5.4".to_string(),
                total_tokens: 20_000,
                total_requests: 9,
            }),
            fallback_reason: Some("provider aggregate".to_string()),
        }
    }

    fn plain_line_text(line: &Line<'_>) -> String {
        line.spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect::<Vec<_>>()
            .join("")
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
        let layout = account_table_layout(58);
        assert_eq!(
            layout.columns,
            vec![AccountColumn::Account, AccountColumn::QuotaSummary]
        );
    }

    #[test]
    fn account_table_layout_shows_saved_column_on_wide_widths() {
        let layout = account_table_layout(108);
        assert!(layout.columns.contains(&AccountColumn::HourlyQuota));
        assert!(layout.columns.contains(&AccountColumn::WeeklyQuota));
        assert!(layout.columns.contains(&AccountColumn::Plan));
        assert!(layout.columns.contains(&AccountColumn::ExpiresAt));
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
    fn account_snapshot_lines_show_identity_and_usage_metadata() {
        let service = ccr_cli::services::OpenCodeAuthService::from_dirs(
            PathBuf::from("."),
            PathBuf::from("."),
        );
        let mut app = crate::tui::opencode_auth::app::OpenCodeAuthApp::from_service(service)
            .expect("test opencode auth app should initialize from injected service");
        app.accounts = vec![sample_account()];
        app.selected_index = 0;
        app.preview_cache.insert(
            "primary".to_string(),
            crate::tui::opencode_auth::app::QuotaPreviewEntry {
                quota: ccr_cli::models::CodexAccountQuota {
                    account_name: "primary".to_string(),
                    email: Some("use***@example.com".to_string()),
                    quota: Some(ccr_cli::models::CodexQuota {
                        hourly_percentage: 48,
                        hourly_reset_time: Some(
                            (Utc::now()
                                + chrono::Duration::hours(3)
                                + chrono::Duration::minutes(11))
                            .timestamp(),
                        ),
                        hourly_window_minutes: Some(300),
                        hourly_window_present: Some(true),
                        weekly_percentage: 72,
                        weekly_reset_time: Some(
                            (Utc::now()
                                + chrono::Duration::days(2)
                                + chrono::Duration::hours(3)
                                + chrono::Duration::minutes(17))
                            .timestamp(),
                        ),
                        weekly_window_minutes: Some(10080),
                        weekly_window_present: Some(true),
                        plan_type: Some("PLUS".to_string()),
                        raw_data: None,
                    }),
                    error: None,
                    fetched_at: Utc::now(),
                },
            },
        );
        let lines = account_snapshot_lines(&app, &sample_account());
        assert!(plain_line_text(&lines[0]).contains("primary"));
        assert!(plain_line_text(&lines[2]).contains("acc-primary"));
        assert!(plain_line_text(&lines[4]).contains("PLUS"));
        assert!(plain_line_text(&lines[5]).contains("Saved at:"));
        assert!(plain_line_text(&lines[6]).contains("Expires:"));
        assert!(plain_line_text(&lines[7]).contains("5h:"));
        assert!(plain_line_text(&lines[7]).contains("48%"));
        assert!(plain_line_text(&lines[7]).contains("Reset"));
        assert!(plain_line_text(&lines[7]).contains("m"));
        assert!(plain_line_text(&lines[8]).contains("7d:"));
        assert!(plain_line_text(&lines[8]).contains("72%"));
        assert!(plain_line_text(&lines[8]).contains("Reset"));
        assert!(plain_line_text(&lines[8]).contains("m"));
    }

    #[test]
    fn usage_digest_lines_include_all_time_and_top_model() {
        let lines: Vec<String> = usage_digest_lines(&sample_usage_panel())
            .into_iter()
            .map(|line| plain_line_text(&line))
            .collect();

        assert!(
            lines
                .iter()
                .any(|line| line.contains("All time: 50.5K tokens"))
        );
        assert!(lines.iter().any(|line| line.contains("Top model: gpt-5.4")));
    }

    #[test]
    fn hourly_cell_appends_reset_in_parentheses() {
        let account = sample_account();
        let service = ccr_cli::services::OpenCodeAuthService::from_dirs(
            PathBuf::from("."),
            PathBuf::from("."),
        );
        let mut app = crate::tui::opencode_auth::app::OpenCodeAuthApp::from_service(service)
            .expect("test opencode auth app should initialize from injected service");
        app.accounts = vec![account.clone()];
        app.selected_index = 0;
        app.preview_cache.insert(
            "primary".to_string(),
            crate::tui::opencode_auth::app::QuotaPreviewEntry {
                quota: ccr_cli::models::CodexAccountQuota {
                    account_name: "primary".to_string(),
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
                        plan_type: Some("PLUS".to_string()),
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
        let rendered = (0..terminal.backend().buffer().area.width)
            .filter_map(|x| terminal.backend().buffer().cell((x, 0)))
            .map(|cell| cell.symbol())
            .collect::<String>();
        assert!(rendered.contains("52%"), "{rendered}");
        assert!(rendered.contains("(3h"), "{rendered}");
        assert!(rendered.contains("m)"), "{rendered}");
    }

    #[test]
    fn weekly_cell_appends_reset_in_parentheses() {
        let account = sample_account();
        let service = ccr_cli::services::OpenCodeAuthService::from_dirs(
            PathBuf::from("."),
            PathBuf::from("."),
        );
        let mut app = crate::tui::opencode_auth::app::OpenCodeAuthApp::from_service(service)
            .expect("test opencode auth app should initialize from injected service");
        app.accounts = vec![account.clone()];
        app.selected_index = 0;
        app.preview_cache.insert(
            "primary".to_string(),
            crate::tui::opencode_auth::app::QuotaPreviewEntry {
                quota: ccr_cli::models::CodexAccountQuota {
                    account_name: "primary".to_string(),
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
                        plan_type: Some("PLUS".to_string()),
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
        let rendered = (0..terminal.backend().buffer().area.width)
            .filter_map(|x| terminal.backend().buffer().cell((x, 0)))
            .map(|cell| cell.symbol())
            .collect::<String>();
        assert!(rendered.contains("41%"), "{rendered}");
        assert!(rendered.contains("(2d"), "{rendered}");
        assert!(rendered.contains("h)"), "{rendered}");
    }

    #[test]
    fn draw_usage_panel_shows_quota_and_provider_usage_together() {
        let service = ccr_cli::services::OpenCodeAuthService::from_dirs(
            PathBuf::from("."),
            PathBuf::from("."),
        );
        let mut app = crate::tui::opencode_auth::app::OpenCodeAuthApp::from_service(service)
            .expect("test opencode auth app should initialize from injected service");
        app.accounts = vec![sample_account()];
        app.selected_index = 0;
        app.quota_state = QuotaState::Loaded {
            cache: IndexMap::from([(
                "primary".to_string(),
                ccr_cli::models::CodexAccountQuota {
                    account_name: "primary".to_string(),
                    email: Some("use***@example.com".to_string()),
                    quota: Some(ccr_cli::models::CodexQuota {
                        hourly_percentage: 48,
                        hourly_reset_time: Some(
                            (Utc::now() + chrono::Duration::hours(3)).timestamp(),
                        ),
                        hourly_window_minutes: Some(300),
                        hourly_window_present: Some(true),
                        weekly_percentage: 72,
                        weekly_reset_time: Some(
                            (Utc::now() + chrono::Duration::days(2)).timestamp(),
                        ),
                        weekly_window_minutes: Some(10080),
                        weekly_window_present: Some(true),
                        plan_type: Some("PLUS".to_string()),
                        raw_data: None,
                    }),
                    error: None,
                    fetched_at: Utc::now(),
                },
            )]),
        };
        app.usage_state = OpenCodeUsageState::Loaded(Box::new(
            crate::tui::opencode_auth::app::OpenCodeUsageDataset {
                provider_id: "openai".to_string(),
                rolling: sample_usage_panel().rolling,
                records: vec![ccr_cli::services::OpenCodeUsageRecord {
                    session_id: "ses-1".to_string(),
                    timestamp: Utc::now(),
                    input_tokens: 1200,
                    output_tokens: 240,
                    provider: Some("openai".to_string()),
                    model: Some("gpt-5.4".to_string()),
                }],
            },
        ));

        let mut terminal = Terminal::new(TestBackend::new(90, 22)).unwrap();
        terminal
            .draw(|frame| draw_usage_panel(frame, frame.area(), &app))
            .unwrap();

        let rendered = buffer_text(terminal.backend());
        let compact = compact_text(&rendered);
        assert!(compact.contains("Usage&Quota"), "{rendered}");
        assert!(compact.contains("Quotascope:selectedaccount"), "{rendered}");
        assert!(compact.contains("重置:"), "{rendered}");
        assert!(compact.contains("Usagescope:openaiprovider"), "{rendered}");
        assert!(
            compact.contains("Records:1localassistantmessages"),
            "{rendered}"
        );
    }

    #[test]
    fn draw_account_snapshot_panel_keeps_weekly_reset_visible() {
        let service = ccr_cli::services::OpenCodeAuthService::from_dirs(
            PathBuf::from("."),
            PathBuf::from("."),
        );
        let mut app = crate::tui::opencode_auth::app::OpenCodeAuthApp::from_service(service)
            .expect("test opencode auth app should initialize from injected service");
        app.accounts = vec![sample_account()];
        app.selected_index = 0;
        app.preview_cache.insert(
            "primary".to_string(),
            crate::tui::opencode_auth::app::QuotaPreviewEntry {
                quota: ccr_cli::models::CodexAccountQuota {
                    account_name: "primary".to_string(),
                    email: Some("use***@example.com".to_string()),
                    quota: Some(ccr_cli::models::CodexQuota {
                        hourly_percentage: 48,
                        hourly_reset_time: Some(
                            (Utc::now()
                                + chrono::Duration::hours(4)
                                + chrono::Duration::minutes(59))
                            .timestamp(),
                        ),
                        hourly_window_minutes: Some(300),
                        hourly_window_present: Some(true),
                        weekly_percentage: 72,
                        weekly_reset_time: Some(
                            (Utc::now()
                                + chrono::Duration::days(5)
                                + chrono::Duration::hours(6)
                                + chrono::Duration::minutes(13))
                            .timestamp(),
                        ),
                        weekly_window_minutes: Some(10080),
                        weekly_window_present: Some(true),
                        plan_type: Some("PLUS".to_string()),
                        raw_data: None,
                    }),
                    error: None,
                    fetched_at: Utc::now(),
                },
            },
        );

        let mut terminal = Terminal::new(TestBackend::new(100, 14)).unwrap();
        terminal
            .draw(|frame| draw_account_snapshot_panel(frame, frame.area(), &app))
            .unwrap();

        let rendered = buffer_text(terminal.backend());
        assert!(rendered.contains("7d:"), "{rendered}");
        assert!(rendered.contains("Reset"), "{rendered}");
        assert!(rendered.contains("5d6h13m"), "{rendered}");
    }

    #[test]
    fn draw_loading_placeholder_renders_error_message() {
        let mut terminal = Terminal::new(TestBackend::new(60, 12)).unwrap();
        terminal
            .draw(|frame| {
                draw_loading_placeholder(
                    frame,
                    Rect::new(0, 0, 60, 10),
                    Rect::new(0, 10, 60, 2),
                    crate::tui::theme::ViewportMode::Compact,
                    Some("boom"),
                )
            })
            .unwrap();

        let rendered = buffer_text(terminal.backend());
        assert!(rendered.contains("OpenCode Auth"), "{rendered}");
        assert!(rendered.contains("boom"), "{rendered}");
    }
}
