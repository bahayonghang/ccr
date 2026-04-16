// Claude Auth TUI UI rendering
// 绘制 Claude 官方订阅账号管理终端界面

use super::app::{ClaudeAuthApp, PAGE_SIZE};
use crate::models::{ClaudeLoginState, TokenFreshness};
use crate::services::ClaudeAuthItem;
use crate::tui::overlay::{Overlay, render_overlay};
use crate::tui::theme;
use crate::tui::toast::ToastKind;
use chrono::{DateTime, Local, Utc};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Padding, Paragraph, Wrap},
};

pub fn draw(f: &mut Frame, app: &ClaudeAuthApp) {
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
    app: &ClaudeAuthApp,
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
            draw_account_list_panel(f, content_chunks[0], app);
            draw_context_panel(f, content_chunks[1], app);
        }
        crate::tui::theme::ViewportMode::Standard => {
            let content_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(10), Constraint::Length(16)])
                .split(content_area);
            draw_account_list_panel(f, content_chunks[0], app);
            draw_context_panel(f, content_chunks[1], app);
        }
        crate::tui::theme::ViewportMode::Wide => {
            let columns = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(44), Constraint::Percentage(56)])
                .split(content_area);
            draw_account_list_panel(f, columns[0], app);
            draw_context_panel(f, columns[1], app);
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
        .map(|err| format!("Claude Auth 初始化失败\n\n{err}"))
        .unwrap_or_else(|| "正在初始化 Claude Auth...".to_string());

    let panel = Paragraph::new(message)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::CLAUDE_PRIMARY))
                .title(" 🔐 Claude Auth ")
                .title_style(theme::claude_style()),
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
                .border_style(Style::default().fg(theme::BORDER))
                .title(" Keys ")
                .title_style(theme::claude_style()),
        );
        f.render_widget(status, footer_area);
    }
}

fn draw_title(f: &mut Frame, area: Rect, app: &ClaudeAuthApp) {
    let title = Paragraph::new(vec![Line::from(vec![
        Span::styled(" 🔐 Claude 官方账号管理 ", theme::claude_style()),
        Span::raw(" | "),
        Span::styled(login_status_text(app), login_status_style(&app.login_state)),
    ])])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::BORDER))
            .title(" CCR ")
            .title_style(theme::claude_style()),
    )
    .alignment(Alignment::Center);

    f.render_widget(title, area);
}

fn draw_account_list_panel(f: &mut Frame, area: Rect, app: &ClaudeAuthApp) {
    let total_accounts = app.accounts.len();
    let total_pages = app.total_pages();
    let visible_start = if total_accounts == 0 {
        0
    } else {
        app.current_page * PAGE_SIZE + 1
    };
    let visible_end = if total_accounts == 0 {
        0
    } else {
        app.current_page * PAGE_SIZE + app.current_page_accounts().len()
    };

    let title = if total_pages > 1 {
        format!(
            " Claude Auth ({})  {}-{} / {}  Page {}/{} ",
            total_accounts,
            visible_start,
            visible_end,
            total_accounts,
            app.current_page + 1,
            total_pages
        )
    } else {
        format!(" Claude Auth ({total_accounts}) ")
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::CLAUDE_PRIMARY))
        .title(title)
        .title_style(theme::claude_style())
        .title_bottom(
            Line::from(Span::styled(
                format!(" {} ", login_status_text(app)),
                login_status_style(&app.login_state),
            ))
            .alignment(Alignment::Right),
        )
        .padding(Padding::horizontal(1));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if app.accounts.is_empty() {
        app.list_area.set(None);
        let empty = Paragraph::new(vec![
            Line::from(" 暂未保存 Claude 官方账号"),
            Line::from(""),
            Line::from("按 s 保存当前 `claude login` 登录快照"),
        ])
        .style(theme::muted_style())
        .wrap(Wrap { trim: true });
        f.render_widget(empty, inner);
        return;
    }

    app.list_area.set(Some(inner));

    let items: Vec<ListItem> = app
        .current_page_accounts()
        .iter()
        .enumerate()
        .map(|(index, account)| ListItem::new(account_row(account, index == app.selected_index)))
        .collect();

    let list = List::new(items);
    f.render_widget(list, inner);
}

fn draw_context_panel(f: &mut Frame, area: Rect, app: &ClaudeAuthApp) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::BORDER))
        .title(" Context ")
        .title_style(
            Style::default()
                .fg(theme::FG_SECONDARY)
                .add_modifier(Modifier::BOLD),
        )
        .padding(Padding::horizontal(1));

    let paragraph = Paragraph::new(context_lines(app))
        .block(block)
        .wrap(Wrap { trim: false });
    f.render_widget(paragraph, area);
}

fn draw_footer_strip(f: &mut Frame, area: Rect, app: &ClaudeAuthApp) {
    let text = match &app.overlay {
        Some(Overlay::Confirm { .. }) => "y 确认删除 | n/Esc 取消",
        Some(Overlay::ImportCodexConfirm { .. }) => "y 确认 | n/Esc 取消",
        Some(Overlay::Input { .. }) => "输入账号名 | Enter 保存 | Esc 取消",
        None => "↑↓/jk 选择 | ←→/h l 翻页 | Enter 切换 | s 保存当前 | d 删除 | r 刷新 | q 退出",
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
            Line::from(Span::styled(text, theme::muted_style())),
        ]
    } else {
        vec![Line::from(Span::styled(text, theme::muted_style()))]
    };

    let footer = Paragraph::new(content)
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(theme::BORDER))
                .title(" Keys ")
                .title_style(theme::claude_style()),
        )
        .wrap(Wrap { trim: true });
    f.render_widget(footer, area);
}

fn draw_help_bar(f: &mut Frame, area: Rect, app: &ClaudeAuthApp) {
    let text = if let Some(toast) = app.toasts.active() {
        format!("{}  │  Tab 切换", toast.message)
    } else {
        "Tab 切换  │  ● 当前生效  ◐ 仅已登录  │  官方账号切换只写入 ~/.claude/.credentials.json"
            .to_string()
    };

    let bar = Paragraph::new(text)
        .alignment(Alignment::Center)
        .style(theme::muted_style());
    f.render_widget(bar, area);
}

fn account_row(account: &ClaudeAuthItem, selected: bool) -> Line<'static> {
    let selector = if selected { "▶" } else { " " };
    let current = if account.is_current {
        "●"
    } else if account.is_logged_in {
        "◐"
    } else {
        "○"
    };
    let email = account.email.as_deref().unwrap_or("-");
    let plan = account.subscription_type.as_deref().unwrap_or("-");
    let freshness = account.freshness.icon();
    let text = format!(
        "{selector} {current} {}  {email}  {plan}  {freshness}",
        account.name
    );

    let style = if selected {
        Style::default()
            .fg(theme::BG_PRIMARY)
            .bg(theme::CLAUDE_PRIMARY)
            .add_modifier(Modifier::BOLD)
    } else if account.is_current {
        theme::list_current_style()
    } else if account.is_logged_in {
        theme::info_style()
    } else {
        theme::list_normal_style()
    };

    Line::from(Span::styled(text, style))
}

fn context_lines(app: &ClaudeAuthApp) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    lines.push(section_title("Runtime"));
    if let Some(summary) = &app.runtime_summary {
        lines.push(kv_line(
            "mode",
            summary.mode.label().to_string(),
            theme::FG_PRIMARY,
        ));
        lines.push(kv_line(
            "profile",
            summary.profile_label(),
            theme::FG_PRIMARY,
        ));
        lines.push(kv_line(
            "auth_mode",
            summary
                .current_profile_auth_mode
                .map(|mode| mode.as_str().to_string())
                .unwrap_or_else(|| "-".to_string()),
            theme::FG_WARNING,
        ));
        lines.push(kv_line(
            "auth_source",
            summary
                .current_profile_auth_source
                .clone()
                .unwrap_or_else(|| "-".to_string()),
            theme::FG_INFO,
        ));
        lines.push(kv_line(
            "login",
            summary.official_login_label(),
            theme::FG_INFO,
        ));
        lines.push(kv_line(
            "effective_auth",
            summary.auth_label(),
            theme::FG_SUCCESS,
        ));
        lines.push(kv_line(
            "current_auth",
            summary
                .current_auth_name
                .clone()
                .unwrap_or_else(|| "-".to_string()),
            theme::FG_SUCCESS,
        ));
    } else {
        lines.push(Line::from("  未能读取 Claude 运行时摘要"));
    }

    lines.push(Line::from(""));
    lines.push(section_title("Current Official Login"));
    if let Some(info) = &app.current_info {
        lines.push(kv_line(
            "email",
            info.email.clone().unwrap_or_else(|| "-".to_string()),
            theme::FG_PRIMARY,
        ));
        lines.push(kv_line(
            "billing",
            info.billing_type.clone().unwrap_or_else(|| "-".to_string()),
            theme::FG_PRIMARY,
        ));
        lines.push(kv_line(
            "subscription",
            info.subscription_type
                .clone()
                .unwrap_or_else(|| "-".to_string()),
            theme::FG_PRIMARY,
        ));
        lines.push(kv_line(
            "tier",
            info.rate_limit_tier
                .clone()
                .unwrap_or_else(|| "-".to_string()),
            theme::FG_PRIMARY,
        ));
        lines.push(kv_line(
            "expires",
            format_datetime(info.expires_at),
            freshness_color(&info.freshness),
        ));
        lines.push(kv_line(
            "freshness",
            info.freshness.description().to_string(),
            freshness_color(&info.freshness),
        ));
    } else {
        lines.push(Line::from(
            "  未检测到 ~/.claude/.credentials.json 中的官方订阅登录",
        ));
    }

    lines.push(Line::from(""));
    lines.push(section_title("Selected Snapshot"));
    if let Some(account) = app.selected_account() {
        lines.push(kv_line("name", account.name.clone(), theme::FG_PRIMARY));
        lines.push(kv_line(
            "current",
            if account.is_current { "yes" } else { "no" }.to_string(),
            if account.is_current {
                theme::FG_SUCCESS
            } else {
                theme::FG_MUTED
            },
        ));
        lines.push(kv_line(
            "logged_in",
            if account.is_logged_in { "yes" } else { "no" }.to_string(),
            if account.is_logged_in {
                theme::FG_INFO
            } else {
                theme::FG_MUTED
            },
        ));
        lines.push(kv_line(
            "email",
            account.email.clone().unwrap_or_else(|| "-".to_string()),
            theme::FG_PRIMARY,
        ));
        lines.push(kv_line(
            "billing",
            account
                .billing_type
                .clone()
                .unwrap_or_else(|| "-".to_string()),
            theme::FG_PRIMARY,
        ));
        lines.push(kv_line(
            "subscription",
            account
                .subscription_type
                .clone()
                .unwrap_or_else(|| "-".to_string()),
            theme::FG_PRIMARY,
        ));
        lines.push(kv_line(
            "saved_at",
            format_datetime(Some(account.saved_at)),
            theme::FG_INFO,
        ));
        lines.push(kv_line(
            "last_used",
            format_datetime(account.last_used),
            theme::FG_INFO,
        ));
        lines.push(kv_line(
            "expires",
            format_datetime(account.expires_at),
            freshness_color(&account.freshness),
        ));
        lines.push(kv_line(
            "freshness",
            account.freshness.description().to_string(),
            freshness_color(&account.freshness),
        ));
    } else {
        lines.push(Line::from("  当前没有选中的已保存账号"));
    }

    lines
}

fn section_title(title: &str) -> Line<'static> {
    Line::from(Span::styled(
        format!("  {title}"),
        Style::default()
            .fg(theme::FG_SECONDARY)
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
    ))
}

fn kv_line(key: &str, value: String, color: ratatui::style::Color) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("  {:<12}", key),
            Style::default().fg(theme::FG_MUTED),
        ),
        Span::styled(value, Style::default().fg(color)),
    ])
}

fn login_status_text(app: &ClaudeAuthApp) -> String {
    if let Some(summary) = &app.runtime_summary {
        return match &summary.login_state {
            ClaudeLoginState::NotLoggedIn => "未登录".to_string(),
            ClaudeLoginState::LoggedInUnsaved => "已登录 (未保存)".to_string(),
            ClaudeLoginState::LoggedInSaved { account_name } => {
                format!("已登录并保存: {account_name}")
            }
            ClaudeLoginState::ApiKeyActive => summary.current_login_name.as_ref().map_or_else(
                || "当前 Profile 使用 API Key".to_string(),
                |account_name| format!("当前 Profile 使用 API Key · 官方已登录 {account_name}"),
            ),
        };
    }

    match &app.login_state {
        ClaudeLoginState::NotLoggedIn => "未登录".to_string(),
        ClaudeLoginState::LoggedInUnsaved => "已登录 (未保存)".to_string(),
        ClaudeLoginState::LoggedInSaved { account_name } => format!("已登录并保存: {account_name}"),
        ClaudeLoginState::ApiKeyActive => "当前 Profile 使用 API Key".to_string(),
    }
}

fn login_status_style(state: &ClaudeLoginState) -> Style {
    match state {
        ClaudeLoginState::NotLoggedIn => theme::warning_style(),
        ClaudeLoginState::LoggedInUnsaved => theme::info_style(),
        ClaudeLoginState::LoggedInSaved { .. } => theme::success_style(),
        ClaudeLoginState::ApiKeyActive => Style::default()
            .fg(theme::FG_WARNING)
            .add_modifier(Modifier::BOLD),
    }
}

fn freshness_color(freshness: &TokenFreshness) -> ratatui::style::Color {
    match freshness {
        TokenFreshness::Fresh => theme::FG_SUCCESS,
        TokenFreshness::Stale => theme::FG_WARNING,
        TokenFreshness::Old => theme::FG_ERROR,
        TokenFreshness::Unknown(_) => theme::FG_MUTED,
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

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use crate::services::ClaudeAuthService;
    use ratatui::{Terminal, backend::TestBackend};

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
    fn login_status_text_handles_api_key_state() {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let mut app = ClaudeAuthApp::from_service(ClaudeAuthService::from_parts(
            home.join(".ccr").join("platforms").join("claude"),
            home.join(".claude"),
            home.join(".claude.json"),
        ))
        .unwrap();
        app.login_state = ClaudeLoginState::ApiKeyActive;
        assert_eq!(login_status_text(&app), "当前 Profile 使用 API Key");
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
        app.runtime_summary = Some(crate::models::ClaudeRuntimeSummary {
            mode: crate::models::ClaudeRuntimeMode::ProfileOnly,
            current_profile_name: Some("main_pro".to_string()),
            current_profile_provider: Some("pro".to_string()),
            current_profile_auth_mode: Some(crate::models::ClaudeProfileAuthMode::ApiKey),
            current_profile_auth_source: Some("provider:pro".to_string()),
            current_login_name: Some("work".to_string()),
            official_login_state: ClaudeLoginState::LoggedInSaved {
                account_name: "work".to_string(),
            },
            current_auth_name: None,
            login_state: ClaudeLoginState::ApiKeyActive,
        });

        assert_eq!(
            login_status_text(&app),
            "当前 Profile 使用 API Key · 官方已登录 work"
        );
    }
}
