// 状态栏 Widget
// 显示状态消息和快捷键帮助

use crate::tui::theme;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

/// 📊 状态栏 Widget
///
/// 显示状态消息和快捷键帮助，支持：
/// - ✅ 成功消息（绿色）
/// - ❌ 错误消息（红色）
/// - ⚡ 快捷键帮助行
/// - 🔧 自动确认模式状态（AUTO/SAFE）
pub struct StatusBar<'a> {
    /// 状态消息（message, is_error）
    status_message: Option<(&'a str, bool)>,
    /// 自动确认模式
    auto_confirm_mode: bool,
}

impl<'a> StatusBar<'a> {
    /// 🆕 创建新的状态栏 Widget
    pub fn new() -> Self {
        Self {
            status_message: None,
            auto_confirm_mode: false,
        }
    }

    /// 💬 设置状态消息
    pub fn with_status(mut self, message: &'a str, is_error: bool) -> Self {
        self.status_message = Some((message, is_error));
        self
    }

    /// 🔧 设置自动确认模式
    pub fn with_auto_confirm(mut self, enabled: bool) -> Self {
        self.auto_confirm_mode = enabled;
        self
    }

    /// 📋 构建快捷键帮助文本
    fn build_help_text(&self) -> Line<'static> {
        let confirm_status = if self.auto_confirm_mode {
            let mut style = theme::highlight_style();
            style.bg = Some(Color::Black);
            Span::styled(" AUTO ", style)
        } else {
            Span::styled(
                " SAFE ",
                Style::default().fg(theme::FG_SUCCESS).bg(Color::Black),
            )
        };

        let help_text = vec![
            Span::raw(
                " [1-4] Tab | [↑↓/jk] Nav | [Enter] Switch | [d] Delete | [P/L/S] Sync | [Y] Auto | ",
            ),
            confirm_status,
            Span::raw(" | [Q] Quit "),
        ];

        Line::from(help_text)
    }

    /// 🎨 渲染快捷键帮助行
    fn render_help_line(&self, f: &mut Frame, area: Rect) {
        let help_text = self.build_help_text();
        let footer = Paragraph::new(help_text)
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center);

        f.render_widget(footer, area);
    }

    /// 🎨 渲染状态栏
    pub fn render(&self, f: &mut Frame, area: Rect) {
        if let Some((message, is_error)) = &self.status_message {
            // 分割为两行：状态消息1行 + 快捷键至少3行（包括边框）
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1), // 状态消息
                    Constraint::Min(3),    // 快捷键（至少3行，包括边框）
                ])
                .split(area);

            // 渲染状态消息
            let style = if *is_error {
                theme::status_error()
            } else {
                theme::status_success()
            };

            let status = Paragraph::new(Line::from(Span::styled(format!(" {} ", message), style)))
                .alignment(Alignment::Left);

            f.render_widget(status, chunks[0]);

            // 渲染快捷键
            self.render_help_line(f, chunks[1]);
        } else {
            // 没有状态消息,只渲染快捷键
            self.render_help_line(f, area);
        }
    }
}

impl<'a> Default for StatusBar<'a> {
    fn default() -> Self {
        Self::new()
    }
}
