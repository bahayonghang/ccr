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

/// Which guarded action a confirmation gate will run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmKind {
    /// Apply the frozen profile.
    Apply,
    /// Switch the frozen official account.
    Switch,
    /// Turn profile mode off for the frozen platform.
    ProfileOff,
    /// Log out the official session.
    AuthOff,
    /// Save a profile edit. The frozen request lives beside the dialog.
    Save,
}

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
    /// Default-cancel gate for profile apply, account switch, deactivate, and logout.
    /// `y` reads `kind`; this variant must not share the delete handler.
    ConfirmGate {
        /// Dialog title
        title: String,
        /// Target name, when the action has one
        subject: String,
        /// Description lines
        message: Vec<String>,
        /// Action committed by `y`
        kind: ConfirmKind,
    },
    /// Restart-daemon confirmation dialog
    ConfirmRestartDaemon {
        /// Dialog title
        title: String,
        /// Description lines
        message: Vec<String>,
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

    /// Profile apply. Re-applying the current profile still asks, because it rewrites runtime files.
    pub fn confirm_profile_apply(
        platform: &str,
        name: impl Into<String>,
        is_current: bool,
    ) -> Self {
        let name = name.into();
        if is_current {
            Self::ConfirmGate {
                title: crate::tui_text!("Confirm re-apply", "确认重新应用").to_string(),
                message: vec![crate::tui_format!(
                    "{name} is already current. Re-applying rewrites the runtime files.",
                    "{name} 已是当前配置。重新应用会再次写入运行时文件。"
                )],
                subject: name,
                kind: ConfirmKind::Apply,
            }
        } else {
            Self::ConfirmGate {
                title: crate::tui_text!("Confirm profile switch", "确认切换配置").to_string(),
                message: vec![
                    crate::tui_format!("Target: {name}", "目标：{name}"),
                    crate::tui_format!(
                        "Apply this {platform} profile and stay in the TUI.",
                        "将应用此 {platform} 配置，并留在 TUI。"
                    ),
                ],
                subject: name,
                kind: ConfirmKind::Apply,
            }
        }
    }

    /// Profile edit. Default is cancel. Secrets are named, never printed.
    pub fn confirm_profile_save(name: &str, changes: &[String], rewrites_runtime: bool) -> Self {
        let mut message = vec![crate::tui_format!(
            "Save changes to {name}:",
            "保存对 {name} 的改动："
        )];
        if changes.is_empty() {
            message.push(crate::tui_text!("No changed fields", "没有改动字段").to_string());
        } else {
            message.extend(changes.iter().cloned());
        }
        if rewrites_runtime {
            message.push(
                crate::tui_text!(
                    "Saving rewrites the runtime file immediately.",
                    "保存后会立即重写运行时文件。"
                )
                .to_string(),
            );
        }
        Self::ConfirmGate {
            title: crate::tui_text!("Confirm save", "确认保存").to_string(),
            subject: name.to_string(),
            message,
            kind: ConfirmKind::Save,
        }
    }

    /// Official account switch. Profile mode is turned off only after `y`.
    pub fn confirm_account_switch(
        platform: &str,
        name: impl Into<String>,
        running_processes: usize,
    ) -> Self {
        let name = name.into();
        let mut message = vec![
            crate::tui_format!("Account: {name}", "账号：{name}"),
            crate::tui_format!(
                "Switch the official {platform} session to this account. Profile mode turns off first.",
                "将官方 {platform} 会话切换到此账号。会先关闭配置模式。"
            ),
        ];
        if running_processes > 0 {
            message.push(crate::tui_format!(
                "{running_processes} {platform} processes are running.",
                "检测到 {running_processes} 个 {platform} 进程正在运行。"
            ));
        }
        Self::ConfirmGate {
            title: crate::tui_text!("Confirm account switch", "确认切换账号").to_string(),
            subject: name,
            message,
            kind: ConfirmKind::Switch,
        }
    }

    /// Profile deactivate. Already-off still asks; the worker reports no change.
    pub fn confirm_profile_off(platform: &str) -> Self {
        Self::ConfirmGate {
            title: crate::tui_text!("Confirm deactivate", "确认解绑").to_string(),
            subject: platform.to_string(),
            message: vec![
                crate::tui_format!(
                    "Turn off {platform} profile mode and clear CCR login leftovers.",
                    "关闭 {platform} 配置模式，并清理 CCR 留下的登录现场。"
                ),
                crate::tui_text!("Press o again to cancel.", "再按 o 也会取消。").to_string(),
            ],
            kind: ConfirmKind::ProfileOff,
        }
    }

    /// Official session logout. Does not read the real credential path before `y`.
    pub fn confirm_auth_off(platform: &str) -> Self {
        Self::ConfirmGate {
            title: crate::tui_text!("Confirm logout", "确认退出登录").to_string(),
            subject: platform.to_string(),
            message: vec![
                crate::tui_format!(
                    "Remove the current official {platform} session.",
                    "移除当前官方 {platform} 会话。"
                ),
                crate::tui_text!("Press o again to cancel.", "再按 o 也会取消。").to_string(),
            ],
            kind: ConfirmKind::AuthOff,
        }
    }

    /// Create a confirmation overlay for restarting the app-server daemon
    pub fn confirm_restart_daemon(pid: u32) -> Self {
        Self::ConfirmRestartDaemon {
            title: crate::tui_text!("Restart app-server daemon", "重启 app-server 守护进程")
                .to_string(),
            message: vec![
                crate::tui_format!(
                    "Managed app-server daemon (PID {}) still uses the previous account",
                    "托管 app-server 守护进程 (PID {}) 仍在使用切换前的账号",
                    pid
                ),
                String::new(),
                crate::tui_text!(
                    "Restart applies the switch to new Codex sessions; running sessions are interrupted",
                    "重启后 Codex 新会话使用新账号；正在运行的会话会被中断"
                )
                .to_string(),
            ],
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
        Overlay::ConfirmRestartDaemon { title, message } => {
            let area = centered_rect(50, 30, full_area);
            f.render_widget(Clear, area);

            let mut lines = vec![
                Line::from(""),
                Line::from(Span::styled(
                    crate::tui_text!("⚠ Restart app-server daemon", "⚠ 重启 app-server 守护进程"),
                    Style::default()
                        .fg(theme::warning())
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
                style_dialog(Paragraph::new(lines), title, theme::warning()),
                area,
            );
        }
        Overlay::ConfirmGate {
            title,
            message,
            kind,
            ..
        } => {
            let area = centered_rect(56, 34, full_area);
            f.render_widget(Clear, area);

            let mut lines = vec![
                Line::from(""),
                Line::from(Span::styled(
                    title.as_str(),
                    Style::default()
                        .fg(theme::warning())
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
            ];
            for msg in message {
                lines.push(Line::from(msg.as_str()));
            }
            lines.push(Line::from(""));
            let hint = match kind {
                ConfirmKind::ProfileOff | ConfirmKind::AuthOff => crate::tui_text!(
                    "Press y to confirm | o, n, or Esc to cancel",
                    "按 y 确认 | 按 o、n 或 Esc 取消"
                ),
                ConfirmKind::Apply | ConfirmKind::Switch | ConfirmKind::Save => crate::tui_text!(
                    "Press y to confirm | Enter, n, or Esc to cancel",
                    "按 y 确认 | 按 Enter、n 或 Esc 取消"
                ),
            };
            lines.push(Line::from(Span::styled(
                hint,
                Style::default().fg(theme::muted()),
            )));

            f.render_widget(
                style_dialog(Paragraph::new(lines), title, theme::warning()),
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

#[cfg(test)]
mod tests {
    use super::*;
    use ccr_cli::managers::TuiLanguage;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    fn rendered(overlay: &Overlay) -> String {
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).expect("test terminal");
        terminal
            .draw(|frame| render_overlay(frame, overlay))
            .expect("draw overlay");
        let buffer = terminal.backend().buffer();
        (0..buffer.area.height)
            .map(|y| {
                let mut line = String::new();
                let mut x = 0;
                while x < buffer.area.width {
                    let Some(cell) = buffer.cell((x, y)) else {
                        break;
                    };
                    let symbol = cell.symbol();
                    line.push_str(symbol);
                    let step = unicode_width::UnicodeWidthStr::width(symbol).max(1) as u16;
                    x += step;
                }
                line.trim_end().to_string()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn has_glyph_color(overlay: &Overlay, color: ratatui::style::Color) -> bool {
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).expect("test terminal");
        terminal
            .draw(|frame| render_overlay(frame, overlay))
            .expect("draw overlay");
        terminal
            .backend()
            .buffer()
            .content
            .iter()
            .any(|cell| cell.fg == color && !cell.symbol().trim().is_empty())
    }

    #[test]
    fn delete_confirm_keeps_deletion_copy() {
        let previous = crate::tui::i18n::active_language();
        crate::tui::i18n::set_language(TuiLanguage::SimplifiedChinese);
        let overlay = Overlay::confirm_delete("work");
        let text = rendered(&overlay);
        assert!(text.contains("确认删除"), "{text}");
        assert!(text.contains("work"), "{text}");
        assert!(has_glyph_color(&overlay, theme::error()));
        assert!(!has_glyph_color(&overlay, theme::warning()));
        crate::tui::i18n::set_language(previous);
    }

    #[test]
    fn profile_apply_gate_names_the_target_and_is_not_a_delete_dialog() {
        let previous = crate::tui::i18n::active_language();
        crate::tui::i18n::set_language(TuiLanguage::English);
        let text = rendered(&Overlay::confirm_profile_apply(
            "Claude",
            "anyrouter3",
            false,
        ));
        assert!(text.contains("anyrouter3"), "{text}");
        assert!(text.contains("Confirm profile switch"), "{text}");
        assert!(!text.contains("deletion"), "{text}");
        assert!(!text.contains("删除"), "{text}");
        assert!(has_glyph_color(
            &Overlay::confirm_profile_apply("Claude", "anyrouter3", false),
            theme::warning()
        ));
        crate::tui::i18n::set_language(TuiLanguage::SimplifiedChinese);
        let chinese = rendered(&Overlay::confirm_profile_apply(
            "Claude",
            "anyrouter3",
            true,
        ));
        assert!(chinese.contains("确认重新应用"), "{chinese}");
        assert!(chinese.contains("anyrouter3"), "{chinese}");
        assert!(!chinese.contains("删除"), "{chinese}");
        crate::tui::i18n::set_language(previous);
    }
}
