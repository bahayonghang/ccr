// TUI theme & style — centralized color scheme and style functions
// Unified color palette, accent styles, spacing strategy for consistent theming

use crate::models::platform::Platform;
use ratatui::style::{Color, Modifier, Style};

// ═══════════════════════════════════════════════════════════
// Color definitions - modern gradient palette
// ═══════════════════════════════════════════════════════════

/// Brand color - Claude amber
pub const CLAUDE_PRIMARY: Color = Color::Rgb(245, 158, 11); // #f59e0b

/// Brand color - Codex indigo
pub const CODEX_PRIMARY: Color = Color::Rgb(99, 102, 241); // #6366f1

/// Brand color - Gemini blue
pub const GEMINI_PRIMARY: Color = Color::Rgb(66, 133, 244); // #4285f4

/// Brand color - Droid emerald
pub const DROID_PRIMARY: Color = Color::Rgb(16, 185, 129); // #10b981

/// Foreground colors
pub const FG_PRIMARY: Color = Color::Rgb(248, 250, 252); // #f8fafc - pure white
pub const FG_SECONDARY: Color = Color::Rgb(148, 163, 184); // #94a3b8 - blue gray
pub const FG_MUTED: Color = Color::Rgb(100, 116, 139); // #64748b - dark gray

/// Semantic colors
pub const FG_SUCCESS: Color = Color::Rgb(34, 197, 94); // #22c55e - green
pub const FG_WARNING: Color = Color::Rgb(234, 179, 8); // #eab308 - yellow
pub const FG_ERROR: Color = Color::Rgb(239, 68, 68); // #ef4444 - red

/// Background color (for selected items)
pub const BG_PRIMARY: Color = Color::Rgb(15, 23, 42); // #0f172a - deep blue black

/// Border color
pub const BORDER_DEFAULT: Color = Color::Rgb(71, 85, 105); // #475569

// ═══════════════════════════════════════════════════════════
// Aliases (kept for semantic clarity)
// ═══════════════════════════════════════════════════════════

pub const ACCENT: Color = CODEX_PRIMARY;
pub const BORDER: Color = BORDER_DEFAULT;

// ═══════════════════════════════════════════════════════════
// Platform-aware style functions
// ═══════════════════════════════════════════════════════════

/// Get the accent color for a platform variant
pub fn platform_color_for(platform: Platform) -> Color {
    match platform {
        Platform::Claude => CLAUDE_PRIMARY,
        Platform::Codex => CODEX_PRIMARY,
        Platform::Gemini => GEMINI_PRIMARY,
        Platform::Droid => DROID_PRIMARY,
        _ => CODEX_PRIMARY,
    }
}

/// Get the bold style for a platform variant
pub fn platform_style_for(platform: Platform) -> Style {
    Style::default()
        .fg(platform_color_for(platform))
        .add_modifier(Modifier::BOLD)
}

/// Get accent color by platform display name (legacy string-based API)
#[allow(dead_code)]
pub fn platform_color(platform: &str) -> Color {
    match platform.to_lowercase().as_str() {
        "claude" | "claude code" => CLAUDE_PRIMARY,
        "codex" => CODEX_PRIMARY,
        "gemini" | "gemini cli" => GEMINI_PRIMARY,
        "droid" | "factory droid" => DROID_PRIMARY,
        _ => CODEX_PRIMARY,
    }
}

/// Get style by platform display name (legacy string-based API)
#[allow(dead_code)]
pub fn platform_style(platform: &str) -> Style {
    Style::default()
        .fg(platform_color(platform))
        .add_modifier(Modifier::BOLD)
}

// ═══════════════════════════════════════════════════════════
// General style functions
// ═══════════════════════════════════════════════════════════

/// Title style
pub fn title_style() -> Style {
    Style::default().fg(FG_PRIMARY).add_modifier(Modifier::BOLD)
}

/// Tab highlight style (for selected tab)
pub fn tab_highlight_style() -> Style {
    Style::default()
        .fg(CLAUDE_PRIMARY)
        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
}

/// Tab normal style
pub fn tab_normal_style() -> Style {
    Style::default().fg(FG_SECONDARY)
}

/// List item selected style (default fallback with Codex accent)
#[allow(dead_code)]
pub fn list_selected_style() -> Style {
    Style::default()
        .fg(BG_PRIMARY)
        .bg(CODEX_PRIMARY)
        .add_modifier(Modifier::BOLD)
}

/// List item current-active style
pub fn list_current_style() -> Style {
    Style::default().fg(FG_SUCCESS).add_modifier(Modifier::BOLD)
}

/// List item normal style
pub fn list_normal_style() -> Style {
    Style::default().fg(FG_PRIMARY)
}

/// List item description style
#[allow(dead_code)]
pub fn list_description_style(is_selected: bool, is_current: bool) -> Style {
    if is_selected {
        list_selected_style()
    } else if is_current {
        list_current_style()
    } else {
        Style::default().fg(FG_MUTED)
    }
}

/// Shortcut key style
pub fn shortcut_key_style() -> Style {
    Style::default()
        .fg(CLAUDE_PRIMARY)
        .add_modifier(Modifier::BOLD)
}

/// Shortcut description style
pub fn shortcut_desc_style() -> Style {
    Style::default().fg(FG_SECONDARY)
}

/// Success message style
pub fn success_style() -> Style {
    Style::default().fg(FG_SUCCESS).add_modifier(Modifier::BOLD)
}

/// Error message style
pub fn error_style() -> Style {
    Style::default().fg(FG_ERROR).add_modifier(Modifier::BOLD)
}

/// Empty state hint style
pub fn empty_hint_style() -> Style {
    Style::default()
        .fg(FG_WARNING)
        .add_modifier(Modifier::ITALIC)
}

/// Global background style
pub fn background_style() -> Style {
    Style::default()
}

/// Claude platform style
#[allow(dead_code)]
pub fn claude_style() -> Style {
    Style::default()
        .fg(CLAUDE_PRIMARY)
        .add_modifier(Modifier::BOLD)
}

/// Codex platform style
#[allow(dead_code)]
pub fn codex_style() -> Style {
    Style::default()
        .fg(CODEX_PRIMARY)
        .add_modifier(Modifier::BOLD)
}
