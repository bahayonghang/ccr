// 🎨 TUI 主题与样式集中管理
// 统一颜色方案、强调样式与间距策略，便于后续一致性优化与切换主题

use ratatui::style::{Color, Modifier, Style};

// ═══════════════════════════════════════════════════════════
// 🎨 颜色定义 - 现代渐变配色方案
// ═══════════════════════════════════════════════════════════

/// 品牌色 - Claude 橙 (琥珀)
pub const CLAUDE_PRIMARY: Color = Color::Rgb(245, 158, 11); // #f59e0b

/// 品牌色 - Codex 紫 (靛蓝)
pub const CODEX_PRIMARY: Color = Color::Rgb(99, 102, 241); // #6366f1

/// 基础前景色
pub const FG_PRIMARY: Color = Color::Rgb(248, 250, 252); // #f8fafc - 纯白
pub const FG_SECONDARY: Color = Color::Rgb(148, 163, 184); // #94a3b8 - 灰蓝
pub const FG_MUTED: Color = Color::Rgb(100, 116, 139); // #64748b - 暗灰

/// 语义色
pub const FG_SUCCESS: Color = Color::Rgb(34, 197, 94); // #22c55e - 绿色
pub const FG_WARNING: Color = Color::Rgb(234, 179, 8); // #eab308 - 黄色
pub const FG_ERROR: Color = Color::Rgb(239, 68, 68); // #ef4444 - 红色

/// 背景色 (用于选中项)
pub const BG_PRIMARY: Color = Color::Rgb(15, 23, 42); // #0f172a - 深蓝黑

/// 边框色
pub const BORDER_DEFAULT: Color = Color::Rgb(71, 85, 105); // #475569

// ═══════════════════════════════════════════════════════════
// 🔧 便捷别名 (保留用于语义清晰)
// ═══════════════════════════════════════════════════════════

#[allow(dead_code)]
pub const ACCENT: Color = CODEX_PRIMARY;
pub const BORDER: Color = BORDER_DEFAULT;

// ═══════════════════════════════════════════════════════════
// 🖌️ 样式函数
// ═══════════════════════════════════════════════════════════

/// 标题样式
pub fn title_style() -> Style {
    Style::default().fg(FG_PRIMARY).add_modifier(Modifier::BOLD)
}

/// Tab 高亮样式（用于选中的 Tab）
pub fn tab_highlight_style() -> Style {
    Style::default()
        .fg(CLAUDE_PRIMARY)
        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
}

/// Tab 未选中样式
pub fn tab_normal_style() -> Style {
    Style::default().fg(FG_SECONDARY)
}

/// 列表项选中样式
pub fn list_selected_style() -> Style {
    Style::default()
        .fg(BG_PRIMARY)
        .bg(CODEX_PRIMARY)
        .add_modifier(Modifier::BOLD)
}

/// 列表项当前激活样式
pub fn list_current_style() -> Style {
    Style::default().fg(FG_SUCCESS).add_modifier(Modifier::BOLD)
}

/// 列表项普通样式
pub fn list_normal_style() -> Style {
    Style::default().fg(FG_PRIMARY)
}

/// 快捷键样式
pub fn shortcut_key_style() -> Style {
    Style::default()
        .fg(CLAUDE_PRIMARY)
        .add_modifier(Modifier::BOLD)
}

/// 快捷键说明样式
pub fn shortcut_desc_style() -> Style {
    Style::default().fg(FG_SECONDARY)
}

/// 成功消息样式
pub fn success_style() -> Style {
    Style::default().fg(FG_SUCCESS).add_modifier(Modifier::BOLD)
}

/// 错误消息样式
pub fn error_style() -> Style {
    Style::default().fg(FG_ERROR).add_modifier(Modifier::BOLD)
}

/// 空状态提示样式
pub fn empty_hint_style() -> Style {
    Style::default()
        .fg(FG_WARNING)
        .add_modifier(Modifier::ITALIC)
}

/// 全局背景样式
pub fn background_style() -> Style {
    Style::default()
}

/// Claude 平台专属样式
pub fn claude_style() -> Style {
    Style::default()
        .fg(CLAUDE_PRIMARY)
        .add_modifier(Modifier::BOLD)
}

/// Codex 平台专属样式
pub fn codex_style() -> Style {
    Style::default()
        .fg(CODEX_PRIMARY)
        .add_modifier(Modifier::BOLD)
}

// ═══════════════════════════════════════════════════════════
// 🎯 平台颜色获取
// ═══════════════════════════════════════════════════════════

/// 根据平台名获取主色
#[allow(dead_code)]
pub fn platform_color(platform: &str) -> Color {
    match platform.to_lowercase().as_str() {
        "claude" => CLAUDE_PRIMARY,
        "codex" => CODEX_PRIMARY,
        _ => CODEX_PRIMARY,
    }
}

/// 根据平台名获取样式
pub fn platform_style(platform: &str) -> Style {
    let color = match platform.to_lowercase().as_str() {
        "claude" => CLAUDE_PRIMARY,
        "codex" => CODEX_PRIMARY,
        _ => CODEX_PRIMARY,
    };
    Style::default().fg(color).add_modifier(Modifier::BOLD)
}
