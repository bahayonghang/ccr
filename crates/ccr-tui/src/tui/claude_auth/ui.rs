// Claude Auth TUI UI rendering
// 绘制 Claude 官方订阅账号管理终端界面

use super::app::ClaudeAuthApp;
use crate::tui::footer::{ShortcutHint, shortcut_line};
use crate::tui::overlay::{Overlay, render_overlay};
use crate::tui::theme;
use crate::tui::toast::ToastKind;
use ccr_cli::models::{ClaudeAuthSourceObservation, ClaudeLoginState};
use ccr_cli::services::ClaudeAuthItem;
use chrono::{DateTime, Local, Utc};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Padding, Paragraph, Row, Table, Wrap},
};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

const ACCOUNT_COLUMN_SPACING: u16 = 1;
const DETAIL_LABEL_WIDTH: usize = 12;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AccountColumn {
    Status,
    Account,
    Email,
    Plan,
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

    fn resolved_width(&self, column: AccountColumn) -> u16 {
        self.columns
            .iter()
            .position(|candidate| *candidate == column)
            .and_then(|index| self.resolved_widths.get(index))
            .copied()
            .unwrap_or(0)
    }

    fn text_width(&self, column: AccountColumn) -> usize {
        usize::from(self.resolved_width(column))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AccountListRegions {
    header: Rect,
    body: Rect,
}

pub fn draw(f: &mut Frame, app: &mut ClaudeAuthApp) {
    let background = Block::default().style(theme::background_style());
    f.render_widget(background, f.area());

    let area = f.area();
    let mode = theme::viewport_mode(area.width, area.height);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(3),
            Constraint::Length(2),
        ])
        .split(area);

    draw_title(f, chunks[0], app);
    draw_embedded(f, app, chunks[1], chunks[2], mode);
    draw_help_bar(f, chunks[3], app);

    if let Some(overlay) = &app.overlay {
        render_overlay(f, overlay);
    }
}

pub fn draw_embedded(
    f: &mut Frame,
    app: &mut ClaudeAuthApp,
    content_area: Rect,
    footer_area: Rect,
    mode: crate::tui::theme::ViewportMode,
) {
    match mode {
        crate::tui::theme::ViewportMode::Compact => {
            let content_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(8), Constraint::Length(15)])
                .split(content_area);
            draw_account_list_with_status(f, content_chunks[0], app);
            draw_context_workspace(f, content_chunks[1], app, 7);
        }
        crate::tui::theme::ViewportMode::Standard => {
            let content_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(10), Constraint::Length(16)])
                .split(content_area);
            draw_account_list_with_status(f, content_chunks[0], app);
            draw_context_workspace(f, content_chunks[1], app, 7);
        }
        crate::tui::theme::ViewportMode::Wide => {
            let columns = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
                .split(content_area);
            draw_account_list_with_status(f, columns[0], app);
            draw_context_workspace(f, columns[1], app, 8);
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
        .map(|err| {
            crate::tui_format!(
                "Failed to initialize Claude Auth\n\n{err}",
                "Claude 认证初始化失败\n\n{err}"
            )
        })
        .unwrap_or_else(|| {
            crate::tui_text!("Initializing Claude Auth...", "正在初始化 Claude 认证...").to_string()
        });

    let panel = Paragraph::new(message)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::claude()))
                .title(crate::tui_text!(" Claude Auth ", " Claude 认证 "))
                .title_style(theme::claude_style()),
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
        let status = Paragraph::new(if error.is_some() {
            crate::tui_text!("Initialization failed", "初始化失败")
        } else {
            crate::tui_text!("Loading", "加载中")
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
                .title(crate::tui_text!(" Keys ", " 按键 "))
                .title_style(theme::claude_style()),
        );
        f.render_widget(status, footer_area);
    }
}

fn draw_title(f: &mut Frame, area: Rect, app: &ClaudeAuthApp) {
    let title = Paragraph::new(vec![Line::from(vec![
        Span::styled(
            crate::tui_text!(" Claude Official Account Manager ", " Claude 官方账号管理 "),
            theme::claude_style(),
        ),
        Span::raw(" | "),
        Span::styled(login_status_text(app), login_status_style(&app.login_state)),
    ])])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::border()))
            .title(" CCR ")
            .title_style(theme::claude_style()),
    )
    .alignment(Alignment::Center);

    f.render_widget(title, area);
}

fn draw_account_list_with_status(f: &mut Frame, area: Rect, app: &mut ClaudeAuthApp) {
    let title = crate::tui_format!(" Accounts · {} ", " 账号列表 · {} ", list_status_text(app));
    render_account_list_panel(f, area, app, title);
}

fn render_account_list_panel(f: &mut Frame, area: Rect, app: &mut ClaudeAuthApp, title: String) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::claude()))
        .title(title)
        .title_style(theme::claude_style())
        .padding(Padding::horizontal(1));

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
        let empty = Paragraph::new(vec![
            Line::from(crate::tui_text!(
                " No Claude official accounts saved",
                " 暂未保存 Claude 官方账号"
            )),
            Line::from(""),
            Line::from(crate::tui_text!(
                "Press s to save the current `claude login` snapshot",
                "按 s 保存当前 `claude login` 登录快照"
            )),
        ])
        .style(theme::muted_style())
        .wrap(Wrap { trim: true });
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

fn account_list_footer_line(app: &ClaudeAuthApp) -> Line<'static> {
    let selected_name = app
        .selected_account()
        .map(|account| account.name.clone())
        .unwrap_or_else(|| "-".to_string());
    let selected_style = app
        .selected_account()
        .map(|account| {
            if account.is_current {
                theme::success_style()
            } else if account.is_logged_in {
                theme::info_style()
            } else {
                Style::default()
                    .fg(theme::text())
                    .add_modifier(Modifier::BOLD)
            }
        })
        .unwrap_or_else(theme::muted_style);

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
            crate::tui_text!("● current", "● 当前"),
            theme::success_style(),
        ),
        Span::styled(" · ", theme::muted_style()),
        Span::styled(
            crate::tui_text!("◐ logged-in", "◐ 已登录"),
            theme::info_style(),
        ),
        Span::styled(" · ", theme::muted_style()),
        Span::styled(
            crate::tui_text!("○ saved", "○ 已保存"),
            theme::muted_style(),
        ),
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
    if inner_width < 72 {
        return AccountTableLayout::new(
            vec![
                AccountColumn::Status,
                AccountColumn::Account,
                AccountColumn::Plan,
            ],
            vec![
                Constraint::Length(3),
                Constraint::Min(18),
                Constraint::Length(12),
            ],
            inner_width,
        );
    }

    if inner_width < 108 {
        return AccountTableLayout::new(
            vec![
                AccountColumn::Status,
                AccountColumn::Account,
                AccountColumn::Email,
                AccountColumn::Plan,
            ],
            vec![
                Constraint::Length(3),
                Constraint::Length(18),
                Constraint::Min(20),
                Constraint::Length(12),
            ],
            inner_width,
        );
    }

    AccountTableLayout::new(
        vec![
            AccountColumn::Status,
            AccountColumn::Account,
            AccountColumn::Email,
            AccountColumn::Plan,
            AccountColumn::ExpiresAt,
        ],
        vec![
            Constraint::Length(3),
            Constraint::Length(18),
            Constraint::Min(22),
            Constraint::Length(12),
            Constraint::Length(14),
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
    app: &ClaudeAuthApp,
    layout: &AccountTableLayout,
) {
    let selected_style = theme::selected_row_style();

    let rows =
        app.current_page_accounts()
            .iter()
            .enumerate()
            .map(|(index, account)| {
                let row_style = if index == app.selected_index {
                    selected_style
                } else {
                    Style::default()
                };

                Row::new(layout.columns.iter().map(|column| {
                    account_cell(account, *column, layout, index == app.selected_index)
                }))
                .style(row_style)
                .height(1)
            });

    let table = Table::new(rows, layout.widths.clone()).column_spacing(ACCOUNT_COLUMN_SPACING);
    f.render_widget(table, area);
}

fn account_header_cell(column: &AccountColumn) -> Cell<'static> {
    let label = match column {
        AccountColumn::Status => crate::tui_text!("Status", "状态"),
        AccountColumn::Account => crate::tui_text!("Account", "账号"),
        AccountColumn::Email => crate::tui_text!("Email", "邮箱"),
        AccountColumn::Plan => crate::tui_text!("Plan", "订阅"),
        AccountColumn::ExpiresAt => crate::tui_text!("Expires", "到期"),
    };

    Cell::from(label.to_string())
}

fn account_cell(
    account: &ClaudeAuthItem,
    column: AccountColumn,
    layout: &AccountTableLayout,
    is_selected: bool,
) -> Cell<'static> {
    match column {
        AccountColumn::Status => Cell::from(Line::from(Span::styled(
            if account.is_current {
                "●".to_string()
            } else if account.is_logged_in {
                "◐".to_string()
            } else {
                "○".to_string()
            },
            if is_selected {
                theme::selected_row_style()
            } else if account.is_current {
                theme::success_style()
            } else if account.is_logged_in {
                theme::info_style()
            } else {
                theme::muted_style()
            },
        ))),
        AccountColumn::Account => Cell::from(Line::from(Span::styled(
            truncate_text(&account.name, layout.text_width(AccountColumn::Account)),
            account_name_style(account, is_selected),
        ))),
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
                account.subscription_type.as_deref().unwrap_or("-"),
                layout.text_width(AccountColumn::Plan),
            ),
            if is_selected {
                theme::selected_row_style()
            } else {
                theme::muted_style()
            },
        ))),
        AccountColumn::ExpiresAt => Cell::from(Line::from(Span::styled(
            format_compact_datetime(account.expires_at),
            if is_selected {
                theme::selected_row_style()
            } else {
                theme::info_style()
            },
        ))),
    }
}

fn account_name_style(account: &ClaudeAuthItem, is_selected: bool) -> Style {
    if is_selected {
        theme::selected_row_style()
    } else if account.is_current {
        theme::success_style()
    } else if account.is_logged_in {
        theme::info_style()
    } else {
        Style::default().fg(theme::text())
    }
}

fn draw_context_workspace(f: &mut Frame, area: Rect, app: &ClaudeAuthApp, focus_height: u16) {
    if area.height < focus_height.saturating_add(2) {
        draw_context_panel(f, area, app);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(focus_height), Constraint::Min(0)])
        .split(area);

    draw_focus_panel(f, chunks[0], app);
    draw_context_panel(f, chunks[1], app);
}

fn draw_focus_panel(f: &mut Frame, area: Rect, app: &ClaudeAuthApp) {
    let lines = focus_lines(app);
    let panel = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::claude()))
                .title(crate::tui_text!(" Focus ", " 当前焦点 "))
                .title_style(theme::claude_style()),
        )
        .wrap(Wrap { trim: true });
    f.render_widget(panel, area);
}

fn draw_context_panel(f: &mut Frame, area: Rect, app: &ClaudeAuthApp) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::border()))
        .title(crate::tui_text!(" Context ", " 上下文 "))
        .title_style(
            Style::default()
                .fg(theme::subtext())
                .add_modifier(Modifier::BOLD),
        )
        .padding(Padding::horizontal(1));

    let paragraph = Paragraph::new(context_lines(app))
        .block(block)
        .wrap(Wrap { trim: false });
    f.render_widget(paragraph, area);
}

fn focus_lines(app: &ClaudeAuthApp) -> Vec<Line<'static>> {
    let Some(account) = app.selected_account() else {
        return vec![
            detail_line(
                crate::tui_text!("Account:", "账号："),
                "-",
                theme::muted_style(),
            ),
            detail_line(
                crate::tui_text!("State:", "状态："),
                "-",
                theme::muted_style(),
            ),
        ];
    };

    let state_label = if account.is_current {
        crate::tui_text!("Current", "当前")
    } else if account.is_logged_in {
        crate::tui_text!("Logged in", "已登录")
    } else {
        crate::tui_text!("Saved", "已保存")
    };
    let state_style = if account.is_current {
        theme::success_style()
    } else if account.is_logged_in {
        theme::info_style()
    } else {
        theme::muted_style()
    };

    vec![
        detail_line(
            crate::tui_text!("Account:", "账号："),
            account.name.clone(),
            account_name_style(account, false),
        ),
        detail_line(
            crate::tui_text!("State:", "状态："),
            state_label,
            state_style,
        ),
        detail_optional_line(
            crate::tui_text!("Email:", "邮箱："),
            account.email.as_deref(),
            theme::info_style(),
        ),
        detail_optional_line(
            crate::tui_text!("Plan:", "订阅："),
            account.subscription_type.as_deref(),
            Style::default().fg(theme::text()),
        ),
        detail_line(
            crate::tui_text!("Saved at:", "保存时间："),
            format_datetime(Some(account.saved_at)),
            theme::info_style(),
        ),
        detail_line(
            crate::tui_text!("Expires:", "到期时间："),
            format_datetime(account.expires_at),
            theme::info_style(),
        ),
    ]
}

fn draw_footer_strip(f: &mut Frame, area: Rect, app: &ClaudeAuthApp) {
    let hints = match &app.overlay {
        Some(Overlay::Confirm { .. }) => vec![
            ShortcutHint::new("y", crate::tui_text!("confirm delete", "确认删除")),
            ShortcutHint::new("n/Esc", crate::tui_text!("cancel", "取消")),
        ],
        Some(Overlay::ImportCodexConfirm { .. }) => vec![
            ShortcutHint::new("y", crate::tui_text!("confirm", "确认")),
            ShortcutHint::new("n/Esc", crate::tui_text!("cancel", "取消")),
        ],
        Some(Overlay::Input { .. }) => vec![
            ShortcutHint::new("", crate::tui_text!("enter account name", "输入账号名")),
            ShortcutHint::new("Enter", crate::tui_text!("save", "保存")),
            ShortcutHint::new("Esc", crate::tui_text!("cancel", "取消")),
        ],
        Some(Overlay::RenameInput { .. }) => vec![
            ShortcutHint::new("", crate::tui_text!("enter new name", "输入新名称")),
            ShortcutHint::new("Enter", crate::tui_text!("save", "保存")),
            ShortcutHint::new("Esc", crate::tui_text!("cancel", "取消")),
        ],
        None => vec![
            ShortcutHint::new("Tab/Shift+Tab", crate::tui_text!("switch", "切换")),
            ShortcutHint::new("←→/hl", crate::tui_text!("page", "翻页")),
            ShortcutHint::new("↑↓/jk", crate::tui_text!("select", "选择")),
            ShortcutHint::new("Enter", crate::tui_text!("switch", "切换")),
            ShortcutHint::new("s", crate::tui_text!("save", "保存")),
            ShortcutHint::new("d", crate::tui_text!("delete", "删除")),
            ShortcutHint::new("r", crate::tui_text!("refresh", "刷新")),
            ShortcutHint::new("Ctrl+L", crate::tui_text!("language", "语言")),
            ShortcutHint::new("q", crate::tui_text!("quit", "退出")),
        ],
    };

    let style = if let Some(toast) = app.toasts.active() {
        match toast.kind {
            ToastKind::Success => theme::success_style(),
            ToastKind::Error => theme::error_style(),
            ToastKind::Warning => theme::warning_style(),
            ToastKind::Info => theme::info_style(),
        }
    } else {
        theme::muted_style()
    };

    let content = if let Some(toast) = app.toasts.active() {
        vec![
            Line::from(Span::styled(toast.message.as_str(), style)),
            shortcut_line(&hints, theme::claude()),
        ]
    } else {
        vec![shortcut_line(&hints, theme::claude())]
    };

    let footer = Paragraph::new(content)
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(theme::border()))
                .title(crate::tui_text!(" Keys ", " 按键 "))
                .title_style(theme::claude_style()),
        )
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    f.render_widget(footer, area);
}

fn draw_help_bar(f: &mut Frame, area: Rect, app: &ClaudeAuthApp) {
    let text = if let Some(toast) = app.toasts.active() {
        crate::tui_format!("{}  │  Tab switch", "{}  │  Tab 切换", toast.message)
    } else {
        crate::tui_text!(
            "Tab switch  │  ● active  ◐ logged-in only  ○ saved  │  switch updates credentials and clears CCR-managed settings  │  Ctrl+L language",
            "Tab 切换  │  ● 当前生效  ◐ 仅已登录  ○ 已保存  │  切换会更新凭据并清理 CCR 托管设置  │  Ctrl+L 语言"
        )
        .to_string()
    };

    let bar = Paragraph::new(text)
        .alignment(Alignment::Center)
        .style(theme::muted_style());
    f.render_widget(bar, area);
}

fn context_lines(app: &ClaudeAuthApp) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    lines.push(section_title("Runtime"));
    if let Some(summary) = &app.runtime_summary {
        lines.push(kv_line(
            "mode",
            localize_claude_runtime_text(summary.mode.label().to_string()),
            theme::text(),
        ));
        lines.push(kv_line(
            "profile",
            localize_claude_runtime_text(summary.profile_label()),
            theme::text(),
        ));
        lines.push(kv_line(
            "auth_mode",
            summary
                .current_profile_auth_mode
                .map(|mode| mode.as_str().to_string())
                .unwrap_or_else(|| "-".to_string()),
            theme::warning(),
        ));
        lines.push(kv_line(
            "auth_source",
            summary
                .current_profile_auth_source
                .clone()
                .unwrap_or_else(|| "-".to_string()),
            theme::info(),
        ));
        lines.push(kv_line(
            "login",
            localize_claude_runtime_text(summary.official_login_label()),
            theme::info(),
        ));
        lines.push(kv_line(
            "effective_auth",
            localize_claude_runtime_text(summary.auth_label()),
            theme::success(),
        ));
        lines.push(kv_line(
            "current_auth",
            summary
                .current_auth_name
                .clone()
                .unwrap_or_else(|| "-".to_string()),
            theme::success(),
        ));

        lines.push(Line::from(""));
        lines.push(section_title("Auth Source Diagnosis"));
        let diagnosis = &summary.auth_diagnosis;
        lines.push(kv_line(
            "presumed_source",
            diagnosis
                .presumed_effective_source
                .as_ref()
                .map(format_diagnosed_auth_source)
                .unwrap_or_else(|| {
                    crate::tui_text!("unresolved or ambiguous", "未解析或存在歧义").to_string()
                }),
            if diagnosis.presumed_effective_source.is_some() {
                theme::warning()
            } else {
                theme::muted()
            },
        ));
        let suppressors = diagnosis.suppressors().collect::<Vec<_>>();
        lines.push(kv_line(
            "competing_sources",
            suppressors.len().to_string(),
            if suppressors.is_empty() {
                theme::success()
            } else {
                theme::warning()
            },
        ));
        for source in suppressors {
            lines.push(kv_line(
                "source",
                format_diagnosed_auth_source(source),
                theme::warning(),
            ));
        }
        lines.push(kv_line(
            "api_key_responses",
            if diagnosis.custom_api_key_responses_present {
                crate::tui_text!("present (context only)", "存在（仅解释信息）")
            } else {
                crate::tui_text!("not observed", "未观察到")
            }
            .to_string(),
            theme::muted(),
        ));
        lines.push(kv_line(
            "unobservable",
            diagnosis.unobservable.len().to_string(),
            theme::muted(),
        ));
    } else {
        lines.push(Line::from(crate::tui_text!(
            "  Could not read the Claude runtime summary",
            "  未能读取 Claude 运行时摘要"
        )));
    }

    lines.push(Line::from(""));
    lines.push(section_title("Current Official Login"));
    if let Some(info) = &app.current_info {
        lines.push(kv_line(
            "email",
            info.email.clone().unwrap_or_else(|| "-".to_string()),
            theme::text(),
        ));
        lines.push(kv_line(
            "billing",
            info.billing_type.clone().unwrap_or_else(|| "-".to_string()),
            theme::text(),
        ));
        lines.push(kv_line(
            "subscription",
            info.subscription_type
                .clone()
                .unwrap_or_else(|| "-".to_string()),
            theme::text(),
        ));
        lines.push(kv_line(
            "tier",
            info.rate_limit_tier
                .clone()
                .unwrap_or_else(|| "-".to_string()),
            theme::text(),
        ));
        lines.push(kv_line(
            "expires",
            format_datetime(info.expires_at),
            theme::info(),
        ));
    } else {
        lines.push(Line::from(crate::tui_text!(
            "  No official subscription login detected in ~/.claude/.credentials.json",
            "  未检测到 ~/.claude/.credentials.json 中的官方订阅登录"
        )));
    }

    lines.push(Line::from(""));
    lines.push(section_title("Selected Snapshot"));
    if let Some(account) = app.selected_account() {
        lines.push(kv_line(
            "current",
            if account.is_current {
                crate::tui_text!("yes", "是")
            } else {
                crate::tui_text!("no", "否")
            }
            .to_string(),
            if account.is_current {
                theme::success()
            } else {
                theme::muted()
            },
        ));
        lines.push(kv_line(
            "logged_in",
            if account.is_logged_in {
                crate::tui_text!("yes", "是")
            } else {
                crate::tui_text!("no", "否")
            }
            .to_string(),
            if account.is_logged_in {
                theme::info()
            } else {
                theme::muted()
            },
        ));
        lines.push(kv_line(
            "billing",
            account
                .billing_type
                .clone()
                .unwrap_or_else(|| "-".to_string()),
            theme::text(),
        ));
        lines.push(kv_line(
            "subscription",
            account
                .subscription_type
                .clone()
                .unwrap_or_else(|| "-".to_string()),
            theme::text(),
        ));
        lines.push(kv_line(
            "tier",
            account
                .rate_limit_tier
                .clone()
                .unwrap_or_else(|| "-".to_string()),
            theme::text(),
        ));
        lines.push(kv_line(
            "saved_at",
            format_datetime(Some(account.saved_at)),
            theme::info(),
        ));
        lines.push(kv_line(
            "last_used",
            format_datetime(account.last_used),
            theme::info(),
        ));
        lines.push(kv_line(
            "expires",
            format_datetime(account.expires_at),
            theme::info(),
        ));
    } else {
        lines.push(Line::from(crate::tui_text!(
            "  No saved account is selected",
            "  当前没有选中的已保存账号"
        )));
    }

    lines
}

fn format_diagnosed_auth_source(source: &ClaudeAuthSourceObservation) -> String {
    format!(
        "{} @ {} ({}; {}; {})",
        source.kind.as_str(),
        source.location.as_str(),
        source.confidence.as_str(),
        source.evidence.as_str(),
        source.ownership.as_str()
    )
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

fn detail_spans_line(label: &str, mut spans: Vec<Span<'static>>) -> Line<'static> {
    let mut all = vec![detail_label_span(label)];
    all.append(&mut spans);
    Line::from(all)
}

fn section_title(title: &str) -> Line<'static> {
    let title = match title {
        "Runtime" => crate::tui_text!("Runtime", "运行时"),
        "Current Official Login" => {
            crate::tui_text!("Current Official Login", "当前官方登录")
        }
        "Selected Snapshot" => crate::tui_text!("Selected Snapshot", "所选快照"),
        "Auth Source Diagnosis" => {
            crate::tui_text!("Auth Source Diagnosis", "认证来源诊断")
        }
        _ => title,
    };
    Line::from(Span::styled(
        format!("  {title}"),
        Style::default()
            .fg(theme::subtext())
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
    ))
}

fn kv_line(key: &str, value: String, color: ratatui::style::Color) -> Line<'static> {
    let key = localized_kv_key(key);
    Line::from(vec![
        Span::styled(
            format!("  {}", pad_text(key, 12)),
            Style::default().fg(theme::muted()),
        ),
        Span::styled(value, Style::default().fg(color)),
    ])
}

fn localized_detail_label(label: &str) -> &str {
    match label {
        "Account:" => crate::tui_text!("Account:", "账号："),
        "State:" => crate::tui_text!("State:", "状态："),
        "Email:" => crate::tui_text!("Email:", "邮箱："),
        "Plan:" => crate::tui_text!("Plan:", "订阅："),
        "Saved at:" => crate::tui_text!("Saved at:", "保存时间："),
        "Expires:" => crate::tui_text!("Expires:", "到期时间："),
        _ => label,
    }
}

fn localized_kv_key(key: &str) -> &str {
    match key {
        "mode" => crate::tui_text!("mode", "模式"),
        "profile" => crate::tui_text!("profile", "配置"),
        "auth_mode" => crate::tui_text!("auth_mode", "认证模式"),
        "auth_source" => crate::tui_text!("auth_source", "认证来源"),
        "login" => crate::tui_text!("login", "登录"),
        "effective_auth" => crate::tui_text!("effective_auth", "生效认证"),
        "current_auth" => crate::tui_text!("current_auth", "当前认证"),
        "presumed_source" => crate::tui_text!("presumed", "推定来源"),
        "visible_suppressors" => crate::tui_text!("suppressors", "压制来源"),
        "source" => crate::tui_text!("source", "来源"),
        "api_key_responses" => crate::tui_text!("api_key_state", "API Key 状态"),
        "unobservable" => crate::tui_text!("unobservable", "不可观测层"),
        "email" => crate::tui_text!("email", "邮箱"),
        "billing" => crate::tui_text!("billing", "计费"),
        "subscription" => crate::tui_text!("subscription", "订阅"),
        "tier" => crate::tui_text!("tier", "等级"),
        "expires" => crate::tui_text!("expires", "到期"),
        "current" => crate::tui_text!("current", "当前"),
        "logged_in" => crate::tui_text!("logged_in", "已登录"),
        "saved_at" => crate::tui_text!("saved_at", "保存时间"),
        "last_used" => crate::tui_text!("last_used", "最近使用"),
        _ => key,
    }
}

fn localize_claude_runtime_text(text: String) -> String {
    if crate::tui::i18n::active_language() == ccr_cli::managers::TuiLanguage::English {
        text.replace("Profile 驱动", "Profile driven")
            .replace("Profile + 官方订阅", "Profile + official subscription")
            .replace(
                "Profile 等待官方订阅",
                "Profile waiting for official subscription",
            )
            .replace("仅官方订阅运行时", "official subscription runtime only")
            .replace("未解析", "unresolved")
            .replace("未绑定", "not bound")
            .replace("未登录", "not logged in")
            .replace("已登录", "logged in")
            .replace("未保存", "unsaved")
            .replace("未就绪", "not ready")
    } else {
        text
    }
}

fn login_status_text(app: &ClaudeAuthApp) -> String {
    if let Some(summary) = &app.runtime_summary {
        return match &summary.login_state {
            ClaudeLoginState::NotLoggedIn => {
                crate::tui_text!("Not logged in", "未登录").to_string()
            }
            ClaudeLoginState::LoggedInUnsaved => {
                crate::tui_text!("Logged in (unsaved)", "已登录（未保存）").to_string()
            }
            ClaudeLoginState::LoggedInSaved { account_name } => {
                crate::tui_format!(
                    "Logged in and saved: {account_name}",
                    "已登录并保存：{account_name}"
                )
            }
            ClaudeLoginState::ApiKeyActive => summary.current_login_name.as_ref().map_or_else(
                || {
                    crate::tui_text!(
                        "Current Profile uses an API Key",
                        "当前 Profile 使用 API Key"
                    )
                    .to_string()
                },
                |account_name| {
                    crate::tui_format!(
                        "Current Profile uses an API Key · official login {account_name}",
                        "当前 Profile 使用 API Key · 官方已登录 {account_name}"
                    )
                },
            ),
        };
    }

    match &app.login_state {
        ClaudeLoginState::NotLoggedIn => crate::tui_text!("Not logged in", "未登录").to_string(),
        ClaudeLoginState::LoggedInUnsaved => {
            crate::tui_text!("Logged in (unsaved)", "已登录（未保存）").to_string()
        }
        ClaudeLoginState::LoggedInSaved { account_name } => crate::tui_format!(
            "Logged in and saved: {account_name}",
            "已登录并保存：{account_name}"
        ),
        ClaudeLoginState::ApiKeyActive => crate::tui_text!(
            "Current Profile uses an API Key",
            "当前 Profile 使用 API Key"
        )
        .to_string(),
    }
}

fn login_status_style(state: &ClaudeLoginState) -> Style {
    match state {
        ClaudeLoginState::NotLoggedIn => theme::warning_style(),
        ClaudeLoginState::LoggedInUnsaved => theme::info_style(),
        ClaudeLoginState::LoggedInSaved { .. } => theme::success_style(),
        ClaudeLoginState::ApiKeyActive => Style::default()
            .fg(theme::warning())
            .add_modifier(Modifier::BOLD),
    }
}

fn list_status_text(app: &ClaudeAuthApp) -> String {
    match &app.login_state {
        ClaudeLoginState::NotLoggedIn => crate::tui_text!("Not logged in", "未登录").to_string(),
        ClaudeLoginState::LoggedInUnsaved => crate::tui_text!("Logged in", "已登录").to_string(),
        ClaudeLoginState::LoggedInSaved { account_name } => {
            crate::tui_format!("Logged in: {account_name}", "已登录：{account_name}")
        }
        ClaudeLoginState::ApiKeyActive => {
            crate::tui_text!("Profile uses an API Key", "Profile 使用 API Key").to_string()
        }
    }
}

fn format_datetime(value: Option<DateTime<Utc>>) -> String {
    value
        .map(|dt| {
            dt.with_timezone(&Local)
                .format("%Y-%m-%d %H:%M")
                .to_string()
        })
        .unwrap_or_else(|| "-".to_string())
}

fn format_compact_datetime(value: Option<DateTime<Utc>>) -> String {
    value
        .map(|dt| dt.with_timezone(&Local).format("%m/%d %H:%M").to_string())
        .unwrap_or_else(|| "-".to_string())
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

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use ccr_cli::models::{
        ClaudeAuthConfidence, ClaudeAuthDiagnosis, ClaudeAuthEvidence, ClaudeAuthOwnership,
        ClaudeAuthSourceKind, ClaudeAuthSourceLocation, ClaudeAuthSourceObservation,
        ClaudeCurrentAuthInfo, ClaudeProfileAuthMode, ClaudeRuntimeMode, ClaudeRuntimeSummary,
    };
    use ccr_cli::services::ClaudeAuthService;
    use chrono::TimeZone;
    use ratatui::{Terminal, backend::TestBackend};
    fn sample_account() -> ClaudeAuthItem {
        ClaudeAuthItem {
            name: "main_pro".to_string(),
            description: Some("Main Claude Pro".to_string()),
            email: Some("bah***@gmail.com".to_string()),
            billing_type: Some("apple_subscription".to_string()),
            subscription_type: Some("pro".to_string()),
            rate_limit_tier: Some("default_claude_ai".to_string()),
            is_current: true,
            is_logged_in: true,
            saved_at: Utc.with_ymd_and_hms(2026, 4, 15, 5, 8, 0).unwrap(),
            last_used: Some(Utc.with_ymd_and_hms(2026, 4, 18, 3, 43, 0).unwrap()),
            expires_at: Some(Utc.with_ymd_and_hms(2026, 4, 18, 11, 45, 0).unwrap()),
        }
    }

    fn sample_app() -> ClaudeAuthApp {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let mut app = ClaudeAuthApp::from_service(ClaudeAuthService::from_parts(
            home.join(".ccr").join("platforms").join("claude"),
            home.join(".claude"),
            home.join(".claude.json"),
        ))
        .unwrap();
        app.accounts = vec![sample_account()];
        app.selected_index = 0;
        app.current_info = Some(ClaudeCurrentAuthInfo {
            account_uuid: Some("acc-main-pro".to_string()),
            email: Some("bahyonghang@gmail.com".to_string()),
            billing_type: Some("apple_subscription".to_string()),
            subscription_type: Some("pro".to_string()),
            rate_limit_tier: Some("default_claude_ai".to_string()),
            expires_at: Some(Utc.with_ymd_and_hms(2026, 4, 18, 11, 45, 0).unwrap()),
        });
        app.runtime_summary = Some(ClaudeRuntimeSummary {
            mode: ClaudeRuntimeMode::ProfileOnly,
            current_profile_name: Some("anyrouter2".to_string()),
            current_profile_provider: Some("anyrouter".to_string()),
            current_profile_auth_mode: Some(ClaudeProfileAuthMode::ApiKey),
            current_profile_auth_source: Some("provider:anyrouter".to_string()),
            current_login_name: Some("main_pro".to_string()),
            official_login_state: ClaudeLoginState::LoggedInSaved {
                account_name: "main_pro".to_string(),
            },
            current_auth_name: Some("effective:main_pro".to_string()),
            login_state: ClaudeLoginState::ApiKeyActive,
            auth_diagnosis: ClaudeAuthDiagnosis {
                observations: vec![ClaudeAuthSourceObservation {
                    kind: ClaudeAuthSourceKind::AnthropicApiKey,
                    location: ClaudeAuthSourceLocation::SettingsEnv,
                    confidence: ClaudeAuthConfidence::Potential,
                    evidence: ClaudeAuthEvidence::OfficialContract,
                    ownership: ClaudeAuthOwnership::UserOwned,
                    suppresses_subscription: true,
                }],
                presumed_effective_source: Some(ClaudeAuthSourceObservation {
                    kind: ClaudeAuthSourceKind::AnthropicApiKey,
                    location: ClaudeAuthSourceLocation::SettingsEnv,
                    confidence: ClaudeAuthConfidence::Potential,
                    evidence: ClaudeAuthEvidence::OfficialContract,
                    ownership: ClaudeAuthOwnership::UserOwned,
                    suppresses_subscription: true,
                }),
                custom_api_key_responses_present: true,
                unobservable: vec!["other_shell_environment".to_string()],
            },
        });
        app
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

    #[test]
    fn draw_loading_placeholder_renders_claude_auth_title() {
        let backend = TestBackend::new(90, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                draw_loading_placeholder(
                    frame,
                    Rect::new(0, 0, 90, 20),
                    Rect::new(0, 20, 90, 4),
                    theme::ViewportMode::Standard,
                    None,
                );
            })
            .unwrap();

        let rendered = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect::<Vec<_>>()
            .join("");
        assert!(rendered.contains("Claude Auth"), "{rendered}");
    }

    #[test]
    fn footer_shortcuts_are_centered_and_use_claude_accent() {
        let app = sample_app();
        let mut terminal = Terminal::new(TestBackend::new(220, 3)).unwrap();

        terminal
            .draw(|frame| draw_footer_strip(frame, frame.area(), &app))
            .unwrap();

        let buffer = terminal.backend().buffer();
        let occupied = (0_u16..220)
            .filter(|&x| buffer[(x, 1)].symbol() != " ")
            .collect::<Vec<_>>();
        let left_margin = *occupied.first().expect("footer should render shortcuts");
        let right_margin = 219 - occupied.last().expect("footer should render shortcuts");
        assert!(left_margin.abs_diff(right_margin) <= 1);
        assert_eq!(buffer[(left_margin, 1)].fg, theme::claude());
        assert!(buffer[(left_margin, 1)].modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn login_status_text_handles_api_key_state() {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let mut app = ClaudeAuthApp::from_service(ClaudeAuthService::from_parts(
            home.join(".ccr").join("platforms").join("claude"),
            home.join(".claude"),
            home.join(".claude.json"),
        ))
        .unwrap();
        app.runtime_summary = None;
        app.login_state = ClaudeLoginState::ApiKeyActive;
        assert_eq!(login_status_text(&app), "Current Profile uses an API Key");

        crate::tui::i18n::set_language(ccr_cli::managers::TuiLanguage::SimplifiedChinese);
        assert_eq!(login_status_text(&app), "当前 Profile 使用 API Key");
        crate::tui::i18n::set_language(ccr_cli::managers::TuiLanguage::English);
    }

    #[test]
    fn login_status_text_shows_logged_in_account_when_api_key_profile_is_active() {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let mut app = ClaudeAuthApp::from_service(ClaudeAuthService::from_parts(
            home.join(".ccr").join("platforms").join("claude"),
            home.join(".claude"),
            home.join(".claude.json"),
        ))
        .unwrap();
        app.runtime_summary = Some(ccr_cli::models::ClaudeRuntimeSummary {
            mode: ccr_cli::models::ClaudeRuntimeMode::ProfileOnly,
            current_profile_name: Some("main_pro".to_string()),
            current_profile_provider: Some("pro".to_string()),
            current_profile_auth_mode: Some(ccr_cli::models::ClaudeProfileAuthMode::ApiKey),
            current_profile_auth_source: Some("provider:pro".to_string()),
            current_login_name: Some("work".to_string()),
            official_login_state: ClaudeLoginState::LoggedInSaved {
                account_name: "work".to_string(),
            },
            current_auth_name: None,
            login_state: ClaudeLoginState::ApiKeyActive,
            auth_diagnosis: Default::default(),
        });

        assert_eq!(
            login_status_text(&app),
            "Current Profile uses an API Key · official login work"
        );
    }

    #[test]
    fn account_table_layout_hides_secondary_columns_on_narrow_widths() {
        let layout = account_table_layout(60);

        assert_eq!(
            layout.columns,
            vec![
                AccountColumn::Status,
                AccountColumn::Account,
                AccountColumn::Plan
            ]
        );
    }

    #[test]
    fn account_table_layout_shows_email_and_expiry_on_wide_widths() {
        let layout = account_table_layout(120);

        assert_eq!(
            layout.columns,
            vec![
                AccountColumn::Status,
                AccountColumn::Account,
                AccountColumn::Email,
                AccountColumn::Plan,
                AccountColumn::ExpiresAt
            ]
        );
    }

    #[test]
    fn focus_lines_show_identity_and_saved_metadata() {
        let app = sample_app();
        let lines = focus_lines(&app);

        assert!(plain_line_text(&lines[0]).contains("main_pro"));
        assert!(plain_line_text(&lines[1]).contains("Current"));
        assert!(plain_line_text(&lines[2]).contains("bah***@gmail.com"));
        assert!(plain_line_text(&lines[3]).contains("pro"));
        assert!(plain_line_text(&lines[4]).contains("Saved at:"));
        assert!(plain_line_text(&lines[5]).contains("Expires:"));
    }

    #[test]
    fn context_lines_keep_runtime_and_snapshot_sections() {
        let app = sample_app();
        let lines: Vec<String> = context_lines(&app)
            .into_iter()
            .map(|line| plain_line_text(&line))
            .collect();

        assert!(lines.iter().any(|line| line.contains("Runtime")));
        assert!(
            lines
                .iter()
                .any(|line| line.contains("Current Official Login"))
        );
        assert!(lines.iter().any(|line| line.contains("Selected Snapshot")));
        assert!(lines.iter().any(|line| line.contains("provider:anyrouter")));
        assert!(lines.iter().any(|line| line.contains("apple_subscription")));
        assert!(
            lines
                .iter()
                .any(|line| line.contains("Auth Source Diagnosis"))
        );
        assert!(lines.iter().any(|line| {
            line.contains("anthropic_api_key @ settings_env") && line.contains("potential")
        }));
        assert!(
            lines
                .iter()
                .any(|line| line.contains("present (context only)"))
        );
    }

    #[test]
    fn draw_focus_panel_renders_selected_account_summary() {
        let app = sample_app();
        let mut terminal = Terminal::new(TestBackend::new(50, 8)).unwrap();

        terminal
            .draw(|frame| draw_focus_panel(frame, frame.area(), &app))
            .unwrap();

        let rendered = buffer_text(terminal.backend());
        assert!(rendered.contains("Focus"), "{rendered}");
        assert!(rendered.contains("main_pro"), "{rendered}");
        assert!(rendered.contains("Current"), "{rendered}");
        assert!(rendered.contains("bah***@gmail.com"), "{rendered}");
    }
}
