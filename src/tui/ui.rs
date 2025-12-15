// TUI UI 渲染模块
// 负责渲染所有 UI 组件

use super::app::{App, TabState};
use super::tabs::{ConfigsTab, HistoryTab, SyncTab, SystemTab};
use super::widgets::StatusBar;
use crate::tui::theme;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Style,
    widgets::{Block, Borders, Tabs},
};

/// 渲染主 UI
pub fn draw(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Content
            Constraint::Length(5), // Footer (状态消息1行 + 快捷键3行 + 安全边距1行)
        ])
        .split(f.area());

    // 渲染标题栏
    render_header(f, app, chunks[0]);

    // 渲染内容区
    render_content(f, app, chunks[1]);

    // 渲染状态栏
    render_footer(f, app, chunks[2]);
}

/// 渲染标题栏
fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let titles = vec![
        TabState::Configs.title(),
        TabState::History.title(),
        TabState::Sync.title(),
        TabState::System.title(),
    ];

    let index = match app.current_tab {
        TabState::Configs => 0,
        TabState::History => 1,
        TabState::Sync => 2,
        TabState::System => 3,
    };

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" 🚀 CCR TUI - Claude Code Configuration Manager ")
                .title_alignment(Alignment::Center)
                .style(theme::title_style()),
        )
        .select(index)
        .style(Style::default().fg(theme::FG_PRIMARY))
        .highlight_style(theme::highlight_style());

    f.render_widget(tabs, area);
}

/// 渲染内容区
fn render_content(f: &mut Frame, app: &mut App, area: Rect) {
    match app.current_tab {
        TabState::Configs => {
            let configs_tab = ConfigsTab::new();
            configs_tab.render(f, app, area);
        }
        TabState::History => {
            let history_tab = HistoryTab::new();
            history_tab.render(f, app, area);
        }
        TabState::Sync => {
            let sync_tab = SyncTab::new();
            sync_tab.render(f, app, area);
        }
        TabState::System => {
            let system_tab = SystemTab::new();
            system_tab.render(f, app, area);
        }
    }
}

fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    // 使用 StatusBar Widget 渲染（简洁优雅！）
    let mut status_bar = StatusBar::new().with_auto_confirm(app.auto_confirm_mode);

    // 如果有状态消息，添加到 StatusBar
    if let Some((message, is_error)) = &app.status_message {
        status_bar = status_bar.with_status(message, *is_error);
    }

    status_bar.render(f, area);
}
