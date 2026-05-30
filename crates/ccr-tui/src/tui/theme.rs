// TUI theme & style — centralized color scheme and style functions
// Unified color palette, accent styles, spacing strategy for consistent theming

use crate::models::Platform;
use ratatui::style::{Color, Modifier, Style};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewportMode {
    Compact,
    Standard,
    Wide,
}

pub fn viewport_mode(width: u16, height: u16) -> ViewportMode {
    if width < 90 || height < 22 {
        ViewportMode::Compact
    } else if width >= 120 && height >= 22 {
        ViewportMode::Wide
    } else {
        ViewportMode::Standard
    }
}

pub fn footer_height(mode: ViewportMode) -> u16 {
    match mode {
        ViewportMode::Compact => 2,
        ViewportMode::Standard | ViewportMode::Wide => 3,
    }
}

// ═══════════════════════════════════════════════════════════
// Color definitions - modern gradient palette
// ═══════════════════════════════════════════════════════════

/// Brand color - Claude amber
pub const CLAUDE_PRIMARY: Color = Color::Rgb(180, 83, 9); // #b45309

/// Brand color - Codex indigo
pub const CODEX_PRIMARY: Color = Color::Rgb(99, 102, 241); // #6366f1

/// Brand color - Gemini blue
pub const GEMINI_PRIMARY: Color = Color::Rgb(37, 99, 235); // #2563eb

/// Brand color - Droid emerald
pub const DROID_PRIMARY: Color = Color::Rgb(5, 150, 105); // #059669

/// Foreground colors
pub const FG_PRIMARY: Color = Color::Rgb(248, 250, 252); // #f8fafc - pure white
pub const FG_SECONDARY: Color = Color::Rgb(148, 163, 184); // #94a3b8 - blue gray
pub const FG_MUTED: Color = Color::Rgb(100, 116, 139); // #64748b - dark gray

/// Semantic colors
pub const FG_SUCCESS: Color = Color::Rgb(21, 128, 61); // #15803d - green
pub const FG_WARNING: Color = Color::Rgb(161, 98, 7); // #a16207 - amber
pub const FG_ERROR: Color = Color::Rgb(220, 38, 38); // #dc2626 - red
pub const FG_INFO: Color = Color::Rgb(3, 105, 161); // #0369a1 - blue

/// Background color (for selected items)
pub const BG_PRIMARY: Color = Color::Rgb(15, 23, 42); // #0f172a - deep blue black

/// Border color
pub const BORDER_DEFAULT: Color = Color::Rgb(71, 85, 105); // #475569

// ═══════════════════════════════════════════════════════════
// Extended color palette for enhanced UI
// ═══════════════════════════════════════════════════════════

/// Accent colors - for emphasis and grouping
pub const ACCENT_CYAN: Color = Color::Rgb(34, 211, 238); // #22d3ee - cyan
pub const ACCENT_PURPLE: Color = Color::Rgb(168, 85, 247); // #a855f7 - purple
#[allow(dead_code)]
pub const ACCENT_GOLD: Color = Color::Rgb(251, 191, 36); // #fbbf24 - gold

/// Background colors - for cards and grouping
#[allow(dead_code)]
pub const BG_CARD: Color = Color::Rgb(30, 41, 59); // #1e293b - card background
#[allow(dead_code)]
pub const BG_HOVER: Color = Color::Rgb(51, 65, 85); // #334155 - hover background
#[allow(dead_code)]
pub const BG_SELECTED: Color = Color::Rgb(71, 85, 105); // #475569 - selected background

/// Border colors - for separation and emphasis
pub const BORDER_LIGHT: Color = Color::Rgb(100, 116, 139); // #64748b - light border
#[allow(dead_code)]
pub const BORDER_ACCENT: Color = Color::Rgb(99, 102, 241); // #6366f1 - accent border

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

/// Get a brighter platform accent for selected-row backgrounds.
pub fn platform_selection_color_for(platform: Platform) -> Color {
    match platform {
        Platform::Claude => Color::Rgb(251, 191, 36), // #fbbf24
        Platform::Codex => Color::Rgb(129, 140, 248), // #818cf8
        Platform::Gemini => Color::Rgb(96, 165, 250), // #60a5fa
        Platform::Droid => Color::Rgb(52, 211, 153),  // #34d399
        _ => Color::Rgb(129, 140, 248),
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

/// Primary body text style.
///
/// Keep the foreground unset so transparent TUI backgrounds inherit the
/// terminal theme's readable default foreground instead of forcing near-white.
pub fn primary_text_style() -> Style {
    Style::default()
}

/// Primary emphasized body text style.
pub fn primary_text_emphasis_style() -> Style {
    primary_text_style().add_modifier(Modifier::BOLD)
}

/// Secondary body text style.
///
/// Uses the terminal default foreground with a lightweight dim modifier so it
/// remains legible on both light and dark transparent backgrounds.
pub fn secondary_text_style() -> Style {
    Style::default().add_modifier(Modifier::DIM)
}

/// Secondary emphasized body text style.
pub fn secondary_text_emphasis_style() -> Style {
    secondary_text_style().add_modifier(Modifier::BOLD)
}

/// Muted/hint body text style.
pub fn muted_text_style() -> Style {
    Style::default().add_modifier(Modifier::DIM | Modifier::ITALIC)
}

/// Title style
#[allow(dead_code)]
pub fn title_style() -> Style {
    primary_text_emphasis_style()
}

/// Filled chip style for the active tab.
pub fn tab_active_style_for(platform: Platform) -> Style {
    Style::default()
        .fg(BG_PRIMARY)
        .bg(platform_selection_color_for(platform))
        .add_modifier(Modifier::BOLD)
}

/// Tab highlight style (for selected tab)
pub fn tab_highlight_style_for(platform: Platform) -> Style {
    tab_active_style_for(platform)
}

/// Tab normal style
pub fn tab_normal_style() -> Style {
    secondary_text_style()
}

/// Unfilled style for inactive tabs.
pub fn tab_inactive_style() -> Style {
    tab_normal_style()
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
    primary_text_style()
}

/// List item description style
#[allow(dead_code)]
pub fn list_description_style(is_selected: bool, is_current: bool) -> Style {
    if is_selected {
        list_selected_style()
    } else if is_current {
        list_current_style()
    } else {
        muted_style()
    }
}

/// Success message style
pub fn success_style() -> Style {
    Style::default().fg(FG_SUCCESS).add_modifier(Modifier::BOLD)
}

/// Warning message style
pub fn warning_style() -> Style {
    Style::default().fg(FG_WARNING).add_modifier(Modifier::BOLD)
}

/// Error message style
pub fn error_style() -> Style {
    Style::default().fg(FG_ERROR).add_modifier(Modifier::BOLD)
}

/// Informational message style
pub fn info_style() -> Style {
    Style::default().fg(FG_INFO)
}

/// Muted secondary text style
pub fn muted_style() -> Style {
    muted_text_style()
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

// ═══════════════════════════════════════════════════════════
// Enhanced style functions for optimized UI
// ═══════════════════════════════════════════════════════════

/// Get quota color based on percentage (5-level gradient)
pub fn quota_color(percentage: i32) -> Color {
    match percentage {
        90..=100 => Color::Rgb(34, 197, 94), // #22c55e - green (充足)
        70..=89 => Color::Rgb(132, 204, 22), // #84cc16 - lime (良好)
        50..=69 => Color::Rgb(234, 179, 8),  // #eab308 - yellow (注意)
        30..=49 => Color::Rgb(249, 115, 22), // #f97316 - orange (警告)
        _ => Color::Rgb(239, 68, 68),        // #ef4444 - red (危险)
    }
}

/// Enhanced selected row style with purple accent
pub fn selected_row_style() -> Style {
    Style::default()
        .bg(ACCENT_PURPLE)
        .fg(BG_PRIMARY)
        .add_modifier(Modifier::BOLD)
}

/// Card block style with cyan accent
pub fn card_block_style() -> Style {
    Style::default().fg(ACCENT_CYAN)
}

/// Separator style
#[allow(dead_code)]
pub fn separator_style() -> Style {
    Style::default().fg(BORDER_LIGHT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transparent_text_styles_inherit_terminal_foreground() {
        assert_eq!(primary_text_style().fg, None);
        assert_eq!(secondary_text_style().fg, None);
        assert_eq!(muted_style().fg, None);
        assert_eq!(background_style().bg, None);
    }

    #[test]
    fn secondary_and_muted_styles_use_modifiers_for_hierarchy() {
        assert!(secondary_text_style().add_modifier.contains(Modifier::DIM));
        assert!(muted_style().add_modifier.contains(Modifier::DIM));
        assert!(muted_style().add_modifier.contains(Modifier::ITALIC));
    }

    #[test]
    fn tab_highlight_style_is_platform_aware_filled_chip() {
        let codex = tab_highlight_style_for(Platform::Codex);

        assert_eq!(codex.fg, Some(BG_PRIMARY));
        assert_eq!(
            codex.bg,
            Some(platform_selection_color_for(Platform::Codex))
        );
        assert_ne!(
            codex.bg,
            Some(platform_selection_color_for(Platform::Claude))
        );
        assert!(codex.add_modifier.contains(Modifier::BOLD));
        assert!(!codex.add_modifier.contains(Modifier::UNDERLINED));
    }
}
