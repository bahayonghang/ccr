// 云端同步 Tab
// 负责渲染 WebDAV 同步配置界面

use crate::managers::sync_config::SyncConfigManager;
use crate::tui::app::App;
use crate::tui::theme;
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

/// ☁️  云端同步 Tab
///
/// 显示 WebDAV 同步配置信息，包括:
/// - 启用状态
/// - WebDAV 服务器地址
/// - 用户名和远程路径
/// - 自动同步状态
/// - 可用操作（Push/Pull/Status）
/// - 配置指南
pub struct SyncTab;

impl SyncTab {
    /// 🆕 创建新的云端同步 Tab
    pub fn new() -> Self {
        Self
    }

    /// 🎨 渲染云端同步 Tab
    pub fn render(&self, f: &mut Frame, app: &App, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" ☁️  Cloud Sync ")
            .title_alignment(Alignment::Left);

        // 检查配置加载（仅用于错误检查）
        if let Err(e) = app.config_service.load_config() {
            self.render_error(f, block, area, e.to_string());
            return;
        }

        // 从独立文件加载sync配置
        let sync_config_result = SyncConfigManager::with_default().and_then(|m| m.load());

        match sync_config_result {
            Ok(sync_config) if sync_config.enabled => {
                self.render_enabled(f, block, area, sync_config);
            }
            _ => {
                self.render_unconfigured(f, block, area);
            }
        }
    }

    /// ❌ 渲染错误状态
    fn render_error(&self, f: &mut Frame, block: Block, area: Rect, error_msg: String) {
        let error_text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "❌ Failed to load configuration",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(format!("Error: {}", error_msg)),
        ];
        let paragraph = Paragraph::new(error_text)
            .block(block)
            .alignment(Alignment::Center);
        f.render_widget(paragraph, area);
    }

    /// ✅ 渲染已配置状态
    fn render_enabled(
        &self,
        f: &mut Frame,
        block: Block,
        area: Rect,
        sync_config: crate::managers::sync_config::SyncConfig,
    ) {
        // clone数据以解决生命周期问题
        let webdav_url = sync_config.webdav_url.clone();
        let username = sync_config.username.clone();
        let remote_path = sync_config.remote_path.clone();
        let auto_sync = sync_config.auto_sync;

        let mut lines = vec![Line::from("")];

        // 同步已配置
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled("状态         : ", Style::default().fg(Color::Cyan)),
            Span::styled(
                "✅ 已启用",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(""));

        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled("WebDAV 服务器: ", Style::default().fg(Color::Cyan)),
            Span::styled(webdav_url, Style::default().fg(Color::White)),
        ]));

        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled("用户名       : ", Style::default().fg(Color::Cyan)),
            Span::styled(username, Style::default().fg(Color::White)),
        ]));

        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled("远程路径     : ", Style::default().fg(Color::Cyan)),
            Span::styled(remote_path, Style::default().fg(Color::White)),
        ]));

        let auto_sync_status = if auto_sync {
            Span::styled("✅ 开启", Style::default().fg(Color::Green))
        } else {
            Span::styled("⭕ 关闭", Style::default().fg(Color::DarkGray))
        };
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled("自动同步     : ", Style::default().fg(Color::Cyan)),
            auto_sync_status,
        ]));

        lines.push(Line::from(""));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  ⚡ 可用操作:",
            theme::highlight_style(),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::raw("     "),
            Span::styled("[P]", theme::highlight_style()),
            Span::raw(" Push   - 上传配置到云端"),
        ]));
        lines.push(Line::from(vec![
            Span::raw("     "),
            Span::styled("[L]", theme::highlight_style()),
            Span::raw(" Pull   - 从云端下载配置"),
        ]));
        lines.push(Line::from(vec![
            Span::raw("     "),
            Span::styled("[S]", theme::highlight_style()),
            Span::raw(" Status - 查看同步状态"),
        ]));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  💡 提示: 这些操作会在退出 TUI 后在命令行执行",
            theme::secondary_text(),
        )));

        let paragraph = Paragraph::new(lines)
            .block(block)
            .alignment(Alignment::Left);
        f.render_widget(paragraph, area);
    }

    /// ⚙️  渲染未配置状态
    fn render_unconfigured(&self, f: &mut Frame, block: Block, area: Rect) {
        let mut lines = vec![Line::from("")];

        // 同步未配置
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled("状态: ", Style::default().fg(theme::FG_ACCENT)),
            Span::styled("⚠️  未配置", theme::highlight_style()),
        ]));
        lines.push(Line::from(""));
        lines.push(Line::from(""));

        lines.push(Line::from(Span::styled(
            "  📝 配置 WebDAV 同步",
            theme::highlight_style(),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                "1.",
                Style::default()
                    .fg(theme::FG_ACCENT)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" 退出 TUI (按 "),
            Span::styled("Q", theme::highlight_style()),
            Span::raw(")"),
        ]));
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                "2.",
                Style::default()
                    .fg(theme::FG_ACCENT)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" 运行命令: "),
            Span::styled("ccr sync config", Style::default().fg(Color::Green)),
        ]));
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                "3.",
                Style::default()
                    .fg(theme::FG_ACCENT)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" 输入 WebDAV 服务器信息"),
        ]));
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                "4.",
                Style::default()
                    .fg(theme::FG_ACCENT)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" 测试连接成功后即可使用"),
        ]));
        lines.push(Line::from(""));
        lines.push(Line::from(""));

        lines.push(Line::from(Span::styled(
            "  💡 支持的服务:",
            Style::default()
                .fg(theme::FG_ACCENT)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::raw("     "),
            Span::styled("•", Style::default().fg(Color::Green)),
            Span::raw(" 坚果云 (推荐，免费)"),
        ]));
        lines.push(Line::from(vec![
            Span::raw("     "),
            Span::styled("•", Style::default().fg(Color::Green)),
            Span::raw(" Nextcloud / ownCloud"),
        ]));
        lines.push(Line::from(vec![
            Span::raw("     "),
            Span::styled("•", Style::default().fg(Color::Green)),
            Span::raw(" 其他标准 WebDAV 服务"),
        ]));

        let paragraph = Paragraph::new(lines)
            .block(block)
            .alignment(Alignment::Left);
        f.render_widget(paragraph, area);
    }
}

impl Default for SyncTab {
    fn default() -> Self {
        Self::new()
    }
}
