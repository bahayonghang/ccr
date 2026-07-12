// Shared overlay system for modal dialogs
// Provides centered overlays with dark backdrop for both main TUI and Codex Auth TUI

use crate::tui::theme;
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
    /// Confirmation dialog for Codex -> OpenCode import
    ImportCodexConfirm {
        /// Dialog title
        title: String,
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
    /// Rename input dialog (carries source account name context)
    RenameInput {
        /// Dialog title
        title: String,
        /// Source account name being renamed
        source: String,
        /// Current input buffer (new name)
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
            title: crate::tui_text!("Confirm deletion", "确认删除").to_string(),
            message: vec![
                crate::tui_format!("Account to delete: {}", "即将删除账号：{}", subject),
                String::new(),
                crate::tui_text!("This action cannot be undone!", "此操作不可撤销！").to_string(),
            ],
            subject,
        }
    }

    /// Create a confirmation overlay for importing saved Codex accounts
    pub fn confirm_import_codex(message: Vec<String>) -> Self {
        Self::ImportCodexConfirm {
            title: crate::tui_text!("Import Codex accounts", "导入 Codex 账号").to_string(),
            message,
        }
    }

    /// Create an input overlay for saving
    pub fn save_input() -> Self {
        Self::Input {
            title: crate::tui_text!("Save account", "保存账号").to_string(),
            prompt: crate::tui_text!("Enter an account name:", "请输入账号名称：").to_string(),
            buffer: String::new(),
            hint: crate::tui_text!(
                "(letters, numbers, underscores, and hyphens only)",
                "（只能包含字母、数字、下划线和连字符）"
            )
            .to_string(),
        }
    }

    /// Create a rename overlay pre-filled with the source account name
    pub fn rename_input(source: impl Into<String>) -> Self {
        let source = source.into();
        Self::RenameInput {
            title: crate::tui_text!("Rename account", "重命名账号").to_string(),
            buffer: source.clone(),
            source,
            hint: crate::tui_text!(
                "(Enter save · Esc cancel · letters/numbers/_/- only)",
                "（Enter 保存 · Esc 取消 · 只能含字母/数字/_/-）"
            )
            .to_string(),
        }
    }

    /// Push a character to input buffer (Input variant only)
    pub fn push_char(&mut self, c: char) {
        match self {
            Self::Input { buffer, .. } | Self::RenameInput { buffer, .. } if buffer.len() < 32 => {
                buffer.push(c);
            }
            _ => {}
        }
    }

    /// Pop a character from input buffer (Input variant only)
    pub fn pop_char(&mut self) {
        match self {
            Self::Input { buffer, .. } | Self::RenameInput { buffer, .. } => {
                buffer.pop();
            }
            _ => {}
        }
    }

    /// Take the input value, draining the buffer
    pub fn take_input(&mut self) -> String {
        match self {
            Self::Input { buffer, .. } | Self::RenameInput { buffer, .. } => std::mem::take(buffer),
            _ => String::new(),
        }
    }
}

// ═══════════════════════════════════════════════════════════
// Rendering
// ═══════════════════════════════════════════════════════════

// 在整屏铺一层 Mantle 暗化层,使对话框从背景中凸显。
fn render_backdrop(f: &mut Frame, area: Rect) {
    let backdrop = Block::default().style(Style::default().bg(theme::palette().bg_secondary));
    f.render_widget(backdrop, area);
}

// 统一对话框外观: surface 底 + 强调色边框/标题 + text 正文,明暗主题下都保证对比。
fn style_dialog<'a>(paragraph: Paragraph<'a>, title: &str, accent: Color) -> Paragraph<'a> {
    paragraph
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true })
        .style(
            Style::default()
                .bg(theme::palette().surface)
                .fg(theme::text()),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(accent))
                .title(format!(" {title} "))
                .title_style(Style::default().fg(accent).add_modifier(Modifier::BOLD))
                .style(Style::default().bg(theme::palette().surface)),
        )
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
                    crate::tui_text!("⚠ Confirm deletion", "⚠ 确认删除"),
                    Style::default()
                        .fg(theme::error())
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
            ];

            for msg in message {
                lines.push(Line::from(msg.as_str()));
            }

            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                crate::tui_text!(
                    "Press y to confirm | n or Esc to cancel",
                    "按 y 确认 | 按 n 或 Esc 取消"
                ),
                Style::default().fg(theme::muted()),
            )));

            f.render_widget(
                style_dialog(Paragraph::new(lines), title, theme::error()),
                area,
            );
        }
        Overlay::ImportCodexConfirm { title, message } => {
            let area = centered_rect(56, 36, full_area);
            f.render_widget(Clear, area);

            let mut lines = vec![
                Line::from(""),
                Line::from(Span::styled(
                    crate::tui_text!("⇄ Import Codex accounts", "⇄ 导入 Codex 账号"),
                    Style::default()
                        .fg(theme::info())
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
            ];

            for msg in message {
                lines.push(Line::from(msg.as_str()));
            }

            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                crate::tui_text!(
                    "Press y to import | n or Esc to cancel",
                    "按 y 确认导入 | 按 n 或 Esc 取消"
                ),
                Style::default().fg(theme::muted()),
            )));

            f.render_widget(
                style_dialog(Paragraph::new(lines), title, theme::info()),
                area,
            );
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
                    crate::tui_text!("Save current login", "保存当前登录"),
                    Style::default()
                        .fg(theme::info())
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(prompt.as_str()),
                Line::from(""),
                Line::from(Span::styled(
                    format!("▶ {buffer}_"),
                    Style::default()
                        .fg(theme::text())
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    hint.as_str(),
                    Style::default().fg(theme::muted()),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    crate::tui_text!(
                        "Press Enter to confirm | Esc to cancel",
                        "按 Enter 确认 | 按 Esc 取消"
                    ),
                    Style::default().fg(theme::muted()),
                )),
            ];

            f.render_widget(
                style_dialog(Paragraph::new(lines), title, theme::info()),
                area,
            );
        }
        Overlay::RenameInput {
            title,
            source,
            buffer,
            hint,
        } => {
            let area = centered_rect(54, 34, full_area);
            f.render_widget(Clear, area);

            let lines = vec![
                Line::from(""),
                Line::from(Span::styled(
                    crate::tui_text!("Rename saved account", "重命名已保存账号"),
                    Style::default()
                        .fg(theme::selection_bg())
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(crate::tui_format!(
                    "Current name: {source}",
                    "当前名称：{source}"
                )),
                Line::from(""),
                Line::from(crate::tui_text!("New name:", "新名称：")),
                Line::from(Span::styled(
                    format!("▶ {buffer}_"),
                    Style::default()
                        .fg(theme::text())
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    hint.as_str(),
                    Style::default().fg(theme::muted()),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    crate::tui_text!(
                        "Enter confirm · Ctrl+F overwrite · Esc cancel",
                        "Enter 确认 · Ctrl+F 强制覆盖 · Esc 取消"
                    ),
                    Style::default().fg(theme::muted()),
                )),
            ];

            f.render_widget(
                style_dialog(Paragraph::new(lines), title, theme::selection_bg()),
                area,
            );
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
