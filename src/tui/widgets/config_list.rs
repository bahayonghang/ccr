// 配置列表 Widget
// 可复用的配置列表渲染组件

use crate::services::config_service::ConfigInfo;
use crate::tui::theme;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    widgets::{Block, List, ListItem, ListState},
};

/// 📋 配置列表 Widget
///
/// 渲染配置列表，支持：
/// - ▶️ 当前配置标记
/// - ⭐ 默认配置标记
/// - 高亮选中项
/// - 自定义样式
pub struct ConfigList<'a> {
    /// 配置列表数据
    configs: &'a [ConfigInfo],
    /// 选中索引
    selected_index: usize,
    /// 外部 Block（可选）
    block: Option<Block<'a>>,
}

impl<'a> ConfigList<'a> {
    /// 🆕 创建新的配置列表 Widget
    pub fn new(configs: &'a [ConfigInfo], selected_index: usize) -> Self {
        Self {
            configs,
            selected_index,
            block: None,
        }
    }

    /// 🎨 设置外部 Block
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    /// 📋 构建列表项
    fn build_list_items(&self) -> Vec<ListItem<'a>> {
        self.configs
            .iter()
            .map(|config| {
                // 构建标记文本
                let mut markers = Vec::new();
                if config.is_current {
                    markers.push("▶️");
                }
                if config.is_default {
                    markers.push("⭐");
                }

                let marker_text = if !markers.is_empty() {
                    format!("{} ", markers.join(" "))
                } else {
                    "   ".to_string()
                };

                // 构建显示文本
                let display_text =
                    format!("{} {} - {}", marker_text, config.name, config.description);

                // 根据状态设置样式
                let style = if config.is_current {
                    Style::default()
                        .fg(theme::FG_SUCCESS)
                        .add_modifier(Modifier::BOLD)
                } else if config.is_default {
                    Style::default().fg(theme::FG_ACCENT)
                } else {
                    Style::default().fg(theme::FG_PRIMARY)
                };

                ListItem::new(display_text).style(style)
            })
            .collect()
    }

    /// 🎨 渲染配置列表
    pub fn render(&self, f: &mut Frame, area: Rect) {
        let items = self.build_list_items();

        // 创建列表组件
        let mut list = List::new(items)
            .highlight_style(theme::highlight_style())
            .highlight_symbol(">> ");

        // 应用外部 Block（如果有）
        if let Some(block) = self.block.clone() {
            list = list.block(block);
        }

        // 创建列表状态
        let mut list_state = ListState::default();
        list_state.select(Some(self.selected_index));

        // 渲染列表
        f.render_stateful_widget(list, area, &mut list_state);
    }
}
