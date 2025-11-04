// 🎨 TUI 主题与样式集中管理
// 统一颜色方案、强调样式与间距策略，便于后续一致性优化与切换主题

use ratatui::style::{Color, Modifier, Style};

/// 基础前景色
pub const FG_PRIMARY: Color = Color::White;
pub const FG_SECONDARY: Color = Color::DarkGray;
pub const FG_ACCENT: Color = Color::Cyan;
pub const FG_SUCCESS: Color = Color::Green;
pub const FG_WARNING: Color = Color::Yellow;
pub const FG_ERROR: Color = Color::Red;

/// 标题样式
pub fn title_style() -> Style {
    Style::default().fg(FG_PRIMARY).add_modifier(Modifier::BOLD)
}

/// 高亮样式（用于选中项/Tab 高亮）
pub fn highlight_style() -> Style {
    Style::default()
        .fg(FG_WARNING)
        .add_modifier(Modifier::BOLD | Modifier::REVERSED)
}

/// 状态成功样式
pub fn status_success() -> Style {
    Style::default().fg(FG_SUCCESS).add_modifier(Modifier::BOLD)
}

/// 状态错误样式
pub fn status_error() -> Style {
    Style::default().fg(FG_ERROR).add_modifier(Modifier::BOLD)
}

/// 次要文本样式（提示/说明）
pub fn secondary_text() -> Style {
    Style::default().fg(FG_SECONDARY)
}
