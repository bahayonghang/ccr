// 配置列表 Tab
// 负责渲染配置列表界面

use crate::tui::app::App;
use crate::tui::widgets::ConfigList;
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

/// 📋 配置列表 Tab
///
/// 显示所有可用的配置，支持:
/// - ▶️ 当前配置标记
/// - ⭐ 默认配置标记
/// - 高亮选中项
/// - 空状态和错误状态处理
pub struct ConfigsTab;

impl ConfigsTab {
    /// 🆕 创建新的配置列表 Tab
    pub fn new() -> Self {
        Self
    }

    /// 🎨 渲染配置列表 Tab
    pub fn render(&self, f: &mut Frame, app: &App, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" ⚙️  Configuration List ")
            .title_alignment(Alignment::Left);

        // 使用缓存的配置列表，避免渲染期间的磁盘 I/O
        let Some(config_list) = app.get_cached_config_list() else {
            self.render_error(f, block, area);
            return;
        };

        if config_list.configs.is_empty() {
            self.render_empty(f, block, area);
            return;
        }

        self.render_list(f, app, block, area);
    }

    /// ❌ 渲染错误状态
    fn render_error(&self, f: &mut Frame, block: Block, area: Rect) {
        let error_text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "❌ Configuration list not loaded",
                Style::default().fg(Color::Red),
            )),
            Line::from(""),
            Line::from("Press any key to refresh caches."),
        ];
        let paragraph = Paragraph::new(error_text)
            .block(block)
            .alignment(Alignment::Left);
        f.render_widget(paragraph, area);
    }

    /// 📭 渲染空状态
    fn render_empty(&self, f: &mut Frame, block: Block, area: Rect) {
        let empty_text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "No configurations found",
                Style::default().fg(Color::Yellow),
            )),
            Line::from(""),
            Line::from("Run 'ccr init' to create a configuration file."),
        ];
        let paragraph = Paragraph::new(empty_text)
            .block(block)
            .alignment(Alignment::Left);
        f.render_widget(paragraph, area);
    }

    /// 📋 渲染配置列表
    fn render_list(&self, f: &mut Frame, app: &App, block: Block, area: Rect) {
        let config_list = app.get_cached_config_list().expect("配置列表应该已缓存");

        // 使用 ConfigList Widget 渲染（简洁优雅！）
        let widget = ConfigList::new(&config_list.configs, app.config_list_index).block(block);
        widget.render(f, area);
    }
}

impl Default for ConfigsTab {
    fn default() -> Self {
        Self::new()
    }
}
