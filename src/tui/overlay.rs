// Shared overlay system for modal dialogs
// Provides centered overlays with dark backdrop for both main TUI and Codex Auth TUI

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

/// Overlay variant for modal dialogs
#[derive(Debug, Clone)]
pub enum Overlay {
    /// Confirmation dialog (yes/no)
    Confirm {
        /// Dialog title
        title: String,
        /// Subject being confirmed (e.g., account name)
        subject: String,
        /// Description lines
        message: Vec<String>,
    },
    /// Text input dialog
    Input {
        /// Dialog title
        title: String,
        /// Prompt text
        prompt: String,
        /// Current input buffer
        buffer: String,
        /// Input constraint hint
        hint: String,
    },
}

impl Overlay {
    /// Create a confirmation overlay for deletion
    pub fn confirm_delete(subject: impl Into<String>) -> Self {
        let subject = subject.into();
        Self::Confirm {
            title: "确认删除".to_string(),
            message: vec![
                format!("即将删除账号: {}", subject),
                String::new(),
                "此操作不可撤销！".to_string(),
            ],
            subject,
        }
    }

    /// Create an input overlay for saving
    pub fn save_input() -> Self {
        Self::Input {
            title: "保存账号".to_string(),
            prompt: "请输入账号名称:".to_string(),
            buffer: String::new(),
            hint: "(只能包含字母、数字、下划线和连字符)".to_string(),
        }
    }

    /// Push a character to input buffer (Input variant only)
    pub fn push_char(&mut self, c: char) {
        if let Self::Input { buffer, .. } = self {
            if buffer.len() < 32 {
                buffer.push(c);
            }
        }
    }

    /// Pop a character from input buffer (Input variant only)
    pub fn pop_char(&mut self) {
        if let Self::Input { buffer, .. } = self {
            buffer.pop();
        }
    }

    /// Take the input value, draining the buffer
    pub fn take_input(&mut self) -> String {
        match self {
            Self::Input { buffer, .. } => std::mem::take(buffer),
            _ => String::new(),
        }
    }
}

// ═══════════════════════════════════════════════════════════
// Rendering
// ═══════════════════════════════════════════════════════════

/// Render a dark backdrop over the entire screen area
fn render_backdrop(f: &mut Frame, area: Rect) {
    let backdrop = Block::default().style(Style::default().bg(Color::Rgb(10, 10, 20)));
    f.render_widget(backdrop, area);
}

/// Render an overlay dialog centered on screen with dark backdrop
pub fn render_overlay(f: &mut Frame, overlay: &Overlay) {
    let full_area = f.area();
    render_backdrop(f, full_area);

    match overlay {
        Overlay::Confirm { title, message, .. } => {
            let area = centered_rect(50, 30, full_area);
            f.render_widget(Clear, area);

            let mut lines = vec![
                Line::from(""),
                Line::from(Span::styled(
                    "⚠️ 确认删除",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
            ];

            for msg in message {
                lines.push(Line::from(msg.as_str()));
            }

            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "按 y 确认 | 按 n 或 Esc 取消",
                Style::default().fg(Color::DarkGray),
            )));

            let popup = Paragraph::new(lines)
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true })
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Yellow))
                        .title(format!(" {} ", title))
                        .title_style(Style::default().fg(Color::Yellow)),
                );

            f.render_widget(popup, area);
        }
        Overlay::Input {
            title,
            prompt,
            buffer,
            hint,
        } => {
            let area = centered_rect(50, 30, full_area);
            f.render_widget(Clear, area);

            let lines = vec![
                Line::from(""),
                Line::from(Span::styled(
                    "💾 保存当前登录",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(prompt.as_str()),
                Line::from(""),
                Line::from(Span::styled(
                    format!("▶ {}_", buffer),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    hint.as_str(),
                    Style::default().fg(Color::DarkGray),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "按 Enter 确认 | 按 Esc 取消",
                    Style::default().fg(Color::DarkGray),
                )),
            ];

            let popup = Paragraph::new(lines)
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true })
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Cyan))
                        .title(format!(" {} ", title))
                        .title_style(Style::default().fg(Color::Cyan)),
                );

            f.render_widget(popup, area);
        }
    }
}

/// Calculate a centered rectangle within the given area
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
