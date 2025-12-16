// 历史记录 Tab
// 负责渲染操作历史界面

use crate::managers::history::OperationResult;
use crate::tui::app::App;
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

/// 📜 历史记录 Tab
///
/// 显示所有操作历史，包括:
/// - ✅ 成功操作（绿色）
/// - ❌ 失败操作（红色）
/// - ⚠️  警告操作（黄色）
/// - 时间戳、操作类型、目标配置
pub struct HistoryTab;

impl HistoryTab {
    /// 🆕 创建新的历史记录 Tab
    pub fn new() -> Self {
        Self
    }

    /// 🎨 渲染历史记录 Tab
    pub fn render(&self, f: &mut Frame, app: &App, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" 📜 Operation History ")
            .title_alignment(Alignment::Left);

        // 使用缓存的历史列表
        let Some(history_list) = app.get_cached_history() else {
            self.render_error(f, block, area);
            return;
        };

        if history_list.is_empty() {
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
                "❌ History not loaded",
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
                "No operation history found",
                Style::default().fg(Color::Yellow),
            )),
            Line::from(""),
            Line::from("History will appear here after you perform operations."),
        ];
        let paragraph = Paragraph::new(empty_text)
            .block(block)
            .alignment(Alignment::Left);
        f.render_widget(paragraph, area);
    }

    /// 📋 渲染历史记录列表
    fn render_list(&self, f: &mut Frame, app: &App, block: Block, area: Rect) {
        let history_list = app.get_cached_history().expect("历史记录应该已缓存");

        // 构建历史记录列表项
        let items: Vec<ListItem> = history_list
            .iter()
            .map(|entry| {
                // 格式化时间戳
                let time = entry.timestamp.format("%m-%d %H:%M:%S").to_string();

                // 操作类型（格式化为固定宽度）
                let op_type = format!("{:<8}", entry.operation.as_str());

                // 详情(目标配置)
                let target = entry.details.to_config.as_deref().unwrap_or("N/A");

                // 结果图标和颜色
                let (result_icon, result_color) = match &entry.result {
                    OperationResult::Success => ("✅", Color::Green),
                    OperationResult::Failure(_) => ("❌", Color::Red),
                    OperationResult::Warning(_) => ("⚠️ ", Color::Yellow),
                };

                // 构建显示文本
                let display_text = format!("[{}] {} {} → {}", time, result_icon, op_type, target);

                ListItem::new(display_text).style(Style::default().fg(result_color))
            })
            .collect();

        // 创建列表组件
        let list = List::new(items)
            .block(block)
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD | Modifier::REVERSED),
            )
            .highlight_symbol(">> ");

        // 创建列表状态
        let mut list_state = ListState::default();
        list_state.select(Some(app.history_list_index));

        // 渲染列表
        f.render_stateful_widget(list, area, &mut list_state);
    }
}

impl Default for HistoryTab {
    fn default() -> Self {
        Self::new()
    }
}
