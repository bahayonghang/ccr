// Grok Auth TUI rendering — session status and auth off only

use super::app::GrokAuthApp;
use crate::tui::footer::{ShortcutHint, shortcut_line};
use crate::tui::theme;
use crate::tui::toast::ToastKind;
use ccr_cli::models::Platform;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Paragraph, Wrap},
};

pub fn draw(f: &mut Frame, app: &mut GrokAuthApp) {
    let background = Block::default().style(theme::background_style());
    f.render_widget(background, f.area());
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Min(8), Constraint::Length(3)])
        .split(f.area());
    draw_status(f, chunks[0], app);
    draw_footer_strip(f, chunks[1], app);
}

pub fn draw_embedded(
    f: &mut Frame,
    app: &mut GrokAuthApp,
    content_area: Rect,
    footer_area: Rect,
    _mode: crate::tui::theme::ViewportMode,
) {
    draw_status(f, content_area, app);
    draw_footer_strip(f, footer_area, app);
}

pub fn draw_loading_placeholder(
    f: &mut Frame,
    content_area: Rect,
    footer_area: Rect,
    _mode: crate::tui::theme::ViewportMode,
    error: Option<&str>,
) {
    let message = match error {
        Some(error) => crate::tui_format!(
            "Failed to load Grok Auth: {}",
            "无法加载 Grok 认证：{}",
            error
        ),
        None => crate::tui_text!("Loading Grok Auth…", "正在加载 Grok 认证…").to_string(),
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(crate::tui_text!(" Grok Auth ", " Grok 认证 "))
        .style(theme::background_style());
    f.render_widget(
        Paragraph::new(message)
            .block(block)
            .style(theme::muted_style())
            .wrap(Wrap { trim: true }),
        content_area,
    );
    let placeholder = GrokAuthApp {
        logged_in: false,
        toasts: crate::tui::toast::ToastManager::new(),
        should_quit: false,
        last_off: None,
    };
    draw_footer_strip(f, footer_area, &placeholder);
}

fn draw_status(f: &mut Frame, area: Rect, app: &GrokAuthApp) {
    let status = if app.logged_in {
        crate::tui_text!("Official session: signed in", "官方会话：已登录")
    } else {
        crate::tui_text!(
            "Official session: signed out (may fall back to XAI_API_KEY)",
            "官方会话：未登录（可回退 XAI_API_KEY）"
        )
    };
    let lines = vec![
        Line::from(Span::styled(
            crate::tui_text!("Grok official session", "Grok 官方会话"),
            theme::title_style().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(status, theme::info_style())),
        Line::from(""),
        Line::from(Span::styled(
            crate::tui_text!(
                "Press o to log out the official runtime session. Saved CCR profiles are not deleted.",
                "按 o 登出官方运行时登录。不会删除已保存的 CCR profile。"
            ),
            theme::muted_style(),
        )),
    ];
    let block = Block::default()
        .borders(Borders::ALL)
        .title(crate::tui_text!(" Session ", " 会话 "))
        .padding(Padding::new(1, 1, 1, 1))
        .border_style(Style::default().fg(theme::border()));
    f.render_widget(
        Paragraph::new(lines)
            .block(block)
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn draw_footer_strip(f: &mut Frame, area: Rect, app: &GrokAuthApp) {
    let hints = vec![
        ShortcutHint::new("Tab/Shift+Tab", crate::tui_text!("switch", "切换")),
        ShortcutHint::new("o", crate::tui_text!("auth off", "登出")),
        ShortcutHint::new("r", crate::tui_text!("refresh", "刷新")),
        ShortcutHint::new("Ctrl+L", crate::tui_text!("language", "语言")),
        ShortcutHint::new("q", crate::tui_text!("quit", "退出")),
    ];
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
            shortcut_line(&hints, theme::accent_for(Platform::Grok)),
        ]
    } else {
        vec![shortcut_line(&hints, theme::accent_for(Platform::Grok))]
    };

    let footer = Paragraph::new(content)
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(theme::border()))
                .title(crate::tui_text!(" Keys ", " 按键 ")),
        )
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    f.render_widget(footer, area);
}
