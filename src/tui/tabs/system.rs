// 系统信息 Tab
// 负责渲染系统信息界面

use crate::tui::app::App;
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use std::path::PathBuf;

/// 💻 系统信息 Tab
///
/// 显示系统和 CCR 的详细信息，包括:
/// - 📊 系统信息（主机名、用户名、操作系统）
/// - ⚙️  CCR 信息（版本、当前配置、自动确认模式）
/// - 📁 文件路径（配置、设置、备份、历史、锁）
/// - 🔧 环境变量覆盖标识
pub struct SystemTab;

impl SystemTab {
    /// 🆕 创建新的系统信息 Tab
    pub fn new() -> Self {
        Self
    }

    /// 🎨 渲染系统信息 Tab
    pub fn render(&self, f: &mut Frame, app: &App, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" 💻 System Information ")
            .title_alignment(Alignment::Left);

        // 获取系统信息
        let hostname = whoami::devicename();
        let username = whoami::username();
        let os = whoami::platform().to_string();

        // 获取 CCR 版本
        let ccr_version = env!("CARGO_PKG_VERSION");

        // 获取主目录
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("~"));

        // 获取真实路径（考虑环境变量）
        let (config_path, config_is_custom) =
            Self::get_real_path("CCR_CONFIG_PATH", home.join(".ccr").join("config.toml"));
        let (settings_path, settings_is_custom) = Self::get_real_path(
            "CCR_SETTINGS_PATH",
            home.join(".claude").join("settings.json"),
        );
        let (backup_dir, backup_is_custom) =
            Self::get_real_path("CCR_BACKUP_DIR", home.join(".claude").join("backups"));
        let (history_path, history_is_custom) = Self::get_real_path(
            "CCR_HISTORY_PATH",
            home.join(".claude").join("ccr_history.json"),
        );
        let (lock_dir, lock_is_custom) =
            Self::get_real_path("CCR_LOCK_DIR", home.join(".claude").join(".locks"));

        // 获取当前配置名称
        let current_config = app
            .config_service
            .get_current()
            .ok()
            .map(|c| c.name)
            .unwrap_or_else(|| "N/A".to_string());

        // 自动确认模式状态
        let auto_confirm_status = if app.auto_confirm_mode {
            Span::styled(
                "ON (session-only)",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            Span::styled("OFF", Style::default().fg(Color::Green))
        };

        // 构建显示内容
        let mut text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "📊 System Information",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(vec![
                Span::raw("  Hostname    : "),
                Span::styled(hostname, Style::default().fg(Color::Yellow)),
            ]),
            Line::from(vec![
                Span::raw("  User        : "),
                Span::styled(username, Style::default().fg(Color::Yellow)),
            ]),
            Line::from(vec![
                Span::raw("  OS          : "),
                Span::styled(os, Style::default().fg(Color::Yellow)),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "⚙️  CCR Information",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(vec![
                Span::raw("  Version     : "),
                Span::styled(ccr_version, Style::default().fg(Color::Yellow)),
            ]),
            Line::from(vec![
                Span::raw("  Config      : "),
                Span::styled(current_config, Style::default().fg(Color::Green)),
            ]),
            Line::from(vec![Span::raw("  Auto-Confirm: "), auto_confirm_status]),
            Line::from(""),
            Line::from(Span::styled(
                "📁 File Paths",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
        ];

        // 添加路径信息
        text.push(Self::make_path_line(
            "Config:",
            config_path,
            config_is_custom,
        ));
        text.push(Self::make_path_line(
            "Settings:",
            settings_path,
            settings_is_custom,
        ));
        text.push(Self::make_path_line(
            "Backup:",
            backup_dir,
            backup_is_custom,
        ));
        text.push(Self::make_path_line(
            "History:",
            history_path,
            history_is_custom,
        ));
        text.push(Self::make_path_line("Lock:", lock_dir, lock_is_custom));

        // 如果有自定义路径，添加说明
        let has_custom = config_is_custom
            || settings_is_custom
            || backup_is_custom
            || history_is_custom
            || lock_is_custom;
        if has_custom {
            text.push(Line::from(""));
            text.push(Line::from(vec![
                Span::raw("  "),
                Span::styled("🔧", Style::default().fg(Color::Yellow)),
                Span::styled(
                    " = Using custom path from environment variable",
                    Style::default().fg(Color::DarkGray),
                ),
            ]));
        }

        let paragraph = Paragraph::new(text).block(block).alignment(Alignment::Left);

        f.render_widget(paragraph, area);
    }

    /// 📁 获取真实的文件路径（考虑环境变量覆盖）
    ///
    /// 返回 (路径, 是否为自定义路径)
    fn get_real_path(env_var: &str, default_path: PathBuf) -> (String, bool) {
        if let Ok(custom_path) = std::env::var(env_var) {
            (custom_path, true)
        } else {
            (default_path.display().to_string(), false)
        }
    }

    /// 🎨 创建路径显示行
    ///
    /// 格式：`  标签       [🔧] 路径`
    /// 如果是自定义路径，显示🔧标识
    fn make_path_line(label: &str, path: String, is_custom: bool) -> Line<'static> {
        let mut spans = vec![
            Span::raw("  "),
            Span::styled(format!("{:<12}", label), Style::default().fg(Color::Cyan)),
        ];

        if is_custom {
            spans.push(Span::styled("🔧 ", Style::default().fg(Color::Yellow)));
        } else {
            spans.push(Span::raw("   "));
        }

        spans.push(Span::styled(path, Style::default().fg(Color::White)));

        Line::from(spans)
    }
}

impl Default for SystemTab {
    fn default() -> Self {
        Self::new()
    }
}
