// TUI theme & style — Catppuccin 双主题（Mocha / Latte）集中配色
//
// 设计目标:
// - 以 Catppuccin 为基底,暗色终端用 Mocha、亮色终端用 Latte,启动时按终端背景自动选定。
// - 两套调色板各自满足明暗对比,保证「明暗终端下所有文字都清晰」。
// - 三个页面(Claude Code / Codex Auth / OpenCode Auth)共用同一套「外壳」语言,
//   仅以平台强调色区分身份: Claude=Peach, Codex=Blue, OpenCode=Teal。

use ccr_cli::models::Platform;
use ratatui::style::{Color, Modifier, Style};
use std::sync::atomic::{AtomicU8, Ordering};

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
// Palette — semantic color tokens for one theme variant
// ═══════════════════════════════════════════════════════════

/// Semantic color tokens for a single theme variant.
///
/// Both [`MOCHA`] and [`LATTE`] populate every field so call sites never
/// branch on the active variant — they read tokens through [`palette`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Palette {
    /// Main background (Base).
    pub bg: Color,
    /// Secondary surface (Mantle) — footers and chrome.
    pub bg_secondary: Color,
    /// Card / inactive-selection surface (Surface0).
    pub surface: Color,
    /// Neutral panel border (Surface2).
    pub border: Color,
    /// Emphasized border (Overlay0).
    pub border_strong: Color,
    /// Primary foreground (Text).
    pub text: Color,
    /// Secondary foreground (Subtext0).
    pub subtext: Color,
    /// Muted foreground / hints (Overlay1).
    pub muted: Color,
    /// Success / positive (Green).
    pub success: Color,
    /// Warning / attention (Yellow).
    pub warning: Color,
    /// Error / danger (Red).
    pub error: Color,
    /// Informational (Blue).
    pub info: Color,
    /// Selected-row background (Mauve).
    pub selection_bg: Color,
    /// Foreground used on top of `selection_bg`.
    pub selection_fg: Color,
    /// Claude identity accent (Peach).
    pub claude: Color,
    /// Codex identity accent (Blue).
    pub codex: Color,
    /// OpenCode identity accent (Teal).
    pub opencode: Color,
    /// Gemini identity accent (Sapphire).
    pub gemini: Color,
    /// Droid identity accent (Green).
    pub droid: Color,
    /// Quota gradient, high→low occupancy (90+, 70+, 50+, 30+, <30).
    pub quota: [Color; 5],
}

/// Catppuccin Mocha — dark theme.
static MOCHA: Palette = Palette {
    bg: Color::Rgb(30, 30, 46),               // Base #1e1e2e
    bg_secondary: Color::Rgb(24, 24, 37),     // Mantle #181825
    surface: Color::Rgb(49, 50, 68),          // Surface0 #313244
    border: Color::Rgb(88, 91, 112),          // Surface2 #585b70
    border_strong: Color::Rgb(108, 112, 134), // Overlay0 #6c7086
    text: Color::Rgb(205, 214, 244),          // Text #cdd6f4
    subtext: Color::Rgb(166, 173, 200),       // Subtext0 #a6adc8
    muted: Color::Rgb(127, 132, 156),         // Overlay1 #7f849c
    success: Color::Rgb(166, 227, 161),       // Green #a6e3a1
    warning: Color::Rgb(249, 226, 175),       // Yellow #f9e2af
    error: Color::Rgb(243, 139, 168),         // Red #f38ba8
    info: Color::Rgb(137, 180, 250),          // Blue #89b4fa
    selection_bg: Color::Rgb(203, 166, 247),  // Mauve #cba6f7
    selection_fg: Color::Rgb(17, 17, 27),     // Crust #11111b
    claude: Color::Rgb(250, 179, 135),        // Peach #fab387
    codex: Color::Rgb(137, 180, 250),         // Blue #89b4fa
    opencode: Color::Rgb(148, 226, 213),      // Teal #94e2d5
    gemini: Color::Rgb(116, 199, 236),        // Sapphire #74c7ec
    droid: Color::Rgb(166, 227, 161),         // Green #a6e3a1
    quota: [
        Color::Rgb(166, 227, 161), // Green
        Color::Rgb(249, 226, 175), // Yellow
        Color::Rgb(250, 179, 135), // Peach
        Color::Rgb(235, 160, 172), // Maroon
        Color::Rgb(243, 139, 168), // Red
    ],
};

/// Catppuccin Latte — light theme.
static LATTE: Palette = Palette {
    bg: Color::Rgb(239, 241, 245),            // Base #eff1f5
    bg_secondary: Color::Rgb(230, 233, 239),  // Mantle #e6e9ef
    surface: Color::Rgb(204, 208, 218),       // Surface0 #ccd0da
    border: Color::Rgb(172, 176, 190),        // Surface2 #acb0be
    border_strong: Color::Rgb(156, 160, 176), // Overlay0 #9ca0b0
    text: Color::Rgb(76, 79, 105),            // Text #4c4f69
    subtext: Color::Rgb(108, 111, 133),       // Subtext0 #6c6f85
    muted: Color::Rgb(140, 143, 161),         // Overlay1 #8c8fa1
    success: Color::Rgb(64, 160, 43),         // Green #40a02b
    warning: Color::Rgb(223, 142, 29),        // Yellow #df8e1d
    error: Color::Rgb(210, 15, 57),           // Red #d20f39
    info: Color::Rgb(30, 102, 245),           // Blue #1e66f5
    selection_bg: Color::Rgb(136, 57, 239),   // Mauve #8839ef
    selection_fg: Color::Rgb(239, 241, 245),  // Base #eff1f5
    claude: Color::Rgb(254, 100, 11),         // Peach #fe640b
    codex: Color::Rgb(30, 102, 245),          // Blue #1e66f5
    opencode: Color::Rgb(23, 146, 153),       // Teal #179299
    gemini: Color::Rgb(32, 159, 181),         // Sapphire #209fb5
    droid: Color::Rgb(64, 160, 43),           // Green #40a02b
    quota: [
        Color::Rgb(64, 160, 43),  // Green
        Color::Rgb(223, 142, 29), // Yellow
        Color::Rgb(254, 100, 11), // Peach
        Color::Rgb(230, 69, 83),  // Maroon
        Color::Rgb(210, 15, 57),  // Red
    ],
};

// ═══════════════════════════════════════════════════════════
// Active theme — runtime-selected, light/dark aware
// ═══════════════════════════════════════════════════════════

/// Theme variant currently driving [`palette`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeVariant {
    Mocha,
    Latte,
}

// 0 = Mocha, 1 = Latte。默认 Mocha,保证未调用 init_theme 的场景(含测试)有确定基线。
static ACTIVE: AtomicU8 = AtomicU8::new(0);

/// The palette for the active theme variant.
pub fn palette() -> &'static Palette {
    match ACTIVE.load(Ordering::Relaxed) {
        1 => &LATTE,
        _ => &MOCHA,
    }
}

/// The active theme variant.
pub fn active_variant() -> ThemeVariant {
    match ACTIVE.load(Ordering::Relaxed) {
        1 => ThemeVariant::Latte,
        _ => ThemeVariant::Mocha,
    }
}

/// Force the active theme variant.
pub fn set_theme(variant: ThemeVariant) {
    let value = match variant {
        ThemeVariant::Mocha => 0,
        ThemeVariant::Latte => 1,
    };
    ACTIVE.store(value, Ordering::Relaxed);
}

/// Toggle between Mocha and Latte (used by the in-app theme switch key).
pub fn toggle_theme() {
    let next = match active_variant() {
        ThemeVariant::Mocha => ThemeVariant::Latte,
        ThemeVariant::Latte => ThemeVariant::Mocha,
    };
    set_theme(next);
}

/// Resolve and apply the startup theme: explicit `CCR_TUI_THEME` override wins,
/// otherwise auto-detect the terminal background, falling back to Mocha.
///
/// Must run before the terminal enters raw mode / the alternate screen so the
/// background query can round-trip cleanly.
pub fn init_theme() {
    let env_value = std::env::var("CCR_TUI_THEME").ok();
    let variant = forced_variant_from_env(env_value.as_deref())
        .or_else(detect_terminal_variant)
        .unwrap_or(ThemeVariant::Mocha);
    set_theme(variant);
}

// 解析 CCR_TUI_THEME: 显式 mocha/latte 直接返回; auto/未知/未设置返回 None(走自动探测)。
fn forced_variant_from_env(value: Option<&str>) -> Option<ThemeVariant> {
    match value.map(|raw| raw.trim().to_ascii_lowercase()).as_deref() {
        Some("mocha") => Some(ThemeVariant::Mocha),
        Some("latte") => Some(ThemeVariant::Latte),
        _ => None,
    }
}

// 通过 termbg 探测终端背景明暗。非交互/不支持/超时一律返回 None。
fn detect_terminal_variant() -> Option<ThemeVariant> {
    use std::io::IsTerminal;
    if !std::io::stdout().is_terminal() {
        return None;
    }
    match termbg::theme(std::time::Duration::from_millis(100)) {
        Ok(termbg::Theme::Light) => Some(ThemeVariant::Latte),
        Ok(termbg::Theme::Dark) => Some(ThemeVariant::Mocha),
        Err(_) => None,
    }
}

// ═══════════════════════════════════════════════════════════
// Token accessors — read the active palette
// ═══════════════════════════════════════════════════════════

/// Main background color.
pub fn bg() -> Color {
    palette().bg
}

/// Secondary surface color (footers / chrome).
#[allow(dead_code)]
pub fn bg_secondary() -> Color {
    palette().bg_secondary
}

/// Card / inactive-selection surface color.
#[allow(dead_code)]
pub fn surface() -> Color {
    palette().surface
}

/// Neutral panel border color.
pub fn border() -> Color {
    palette().border
}

/// Emphasized border color.
#[allow(dead_code)]
pub fn border_strong() -> Color {
    palette().border_strong
}

/// Primary foreground color.
pub fn text() -> Color {
    palette().text
}

/// Secondary foreground color.
pub fn subtext() -> Color {
    palette().subtext
}

/// Muted foreground color.
pub fn muted() -> Color {
    palette().muted
}

/// Success color.
pub fn success() -> Color {
    palette().success
}

/// Warning color.
pub fn warning() -> Color {
    palette().warning
}

/// Error color.
pub fn error() -> Color {
    palette().error
}

/// Informational color.
pub fn info() -> Color {
    palette().info
}

/// Selected-row background color (Mauve).
pub fn selection_bg() -> Color {
    palette().selection_bg
}

/// Foreground used on top of [`selection_bg`].
pub fn selection_fg() -> Color {
    palette().selection_fg
}

/// Claude identity accent.
pub fn claude() -> Color {
    palette().claude
}

/// Codex identity accent.
pub fn codex() -> Color {
    palette().codex
}

/// OpenCode identity accent.
pub fn opencode() -> Color {
    palette().opencode
}

// ═══════════════════════════════════════════════════════════
// Platform-aware accent helpers
// ═══════════════════════════════════════════════════════════

/// Identity accent color for a platform variant.
pub fn accent_for(platform: Platform) -> Color {
    let p = palette();
    match platform {
        Platform::Claude => p.claude,
        Platform::Codex => p.codex,
        Platform::Gemini => p.gemini,
        Platform::Droid => p.droid,
        _ => p.codex,
    }
}

/// Brighter platform accent for selected tab / list highlight backgrounds.
///
/// Kept as a distinct entry point so call sites read intent clearly; the dual
/// palette already tunes each accent for its background.
pub fn platform_selection_color_for(platform: Platform) -> Color {
    accent_for(platform)
}

/// Accent color for a platform variant (alias kept for call-site clarity).
pub fn platform_color_for(platform: Platform) -> Color {
    accent_for(platform)
}

/// Bold accent style for a platform variant.
pub fn platform_style_for(platform: Platform) -> Style {
    Style::default()
        .fg(accent_for(platform))
        .add_modifier(Modifier::BOLD)
}

/// Panel border style following the unified contract:
/// focused/primary panels use the platform accent, others a neutral border.
pub fn panel_border(platform: Platform, focused: bool) -> Style {
    if focused {
        Style::default().fg(accent_for(platform))
    } else {
        Style::default().fg(border())
    }
}

/// Panel title style — platform accent, bold.
#[allow(dead_code)]
pub fn panel_title_style(platform: Platform) -> Style {
    Style::default()
        .fg(accent_for(platform))
        .add_modifier(Modifier::BOLD)
}

/// Get accent color by platform display name (legacy string-based API).
#[allow(dead_code)]
pub fn platform_color(platform: &str) -> Color {
    match platform.to_lowercase().as_str() {
        "claude" | "claude code" => palette().claude,
        "codex" => palette().codex,
        "gemini" | "gemini cli" => palette().gemini,
        "droid" | "factory droid" => palette().droid,
        _ => palette().codex,
    }
}

/// Get style by platform display name (legacy string-based API).
#[allow(dead_code)]
pub fn platform_style(platform: &str) -> Style {
    Style::default()
        .fg(platform_color(platform))
        .add_modifier(Modifier::BOLD)
}

// ═══════════════════════════════════════════════════════════
// Text & chrome style functions
// ═══════════════════════════════════════════════════════════

/// Primary body text style.
pub fn primary_text_style() -> Style {
    Style::default().fg(text())
}

/// Primary emphasized body text style.
pub fn primary_text_emphasis_style() -> Style {
    primary_text_style().add_modifier(Modifier::BOLD)
}

/// Secondary body text style.
pub fn secondary_text_style() -> Style {
    Style::default().fg(subtext())
}

/// Secondary emphasized body text style.
pub fn secondary_text_emphasis_style() -> Style {
    secondary_text_style().add_modifier(Modifier::BOLD)
}

/// Muted/hint body text style.
pub fn muted_text_style() -> Style {
    Style::default().fg(muted()).add_modifier(Modifier::ITALIC)
}

/// Title style.
#[allow(dead_code)]
pub fn title_style() -> Style {
    primary_text_emphasis_style()
}

/// Filled chip style for the active tab.
pub fn tab_active_style_for(platform: Platform) -> Style {
    Style::default()
        .fg(selection_fg())
        .bg(accent_for(platform))
        .add_modifier(Modifier::BOLD)
}

/// Tab highlight style (for selected tab).
pub fn tab_highlight_style_for(platform: Platform) -> Style {
    tab_active_style_for(platform)
}

/// Tab normal style.
pub fn tab_normal_style() -> Style {
    secondary_text_style()
}

/// Unfilled style for inactive tabs.
pub fn tab_inactive_style() -> Style {
    tab_normal_style()
}

/// List item selected style (unified Mauve highlight).
#[allow(dead_code)]
pub fn list_selected_style() -> Style {
    selected_row_style()
}

/// List item current-active style.
pub fn list_current_style() -> Style {
    Style::default().fg(success()).add_modifier(Modifier::BOLD)
}

/// List item normal style.
pub fn list_normal_style() -> Style {
    primary_text_style()
}

/// List item description style.
#[allow(dead_code)]
pub fn list_description_style(is_selected: bool, is_current: bool) -> Style {
    if is_selected {
        selected_row_style()
    } else if is_current {
        list_current_style()
    } else {
        muted_style()
    }
}

/// Success message style.
pub fn success_style() -> Style {
    Style::default().fg(success()).add_modifier(Modifier::BOLD)
}

/// Warning message style.
pub fn warning_style() -> Style {
    Style::default().fg(warning()).add_modifier(Modifier::BOLD)
}

/// Error message style.
pub fn error_style() -> Style {
    Style::default().fg(error()).add_modifier(Modifier::BOLD)
}

/// Informational message style.
pub fn info_style() -> Style {
    Style::default().fg(info())
}

/// Muted secondary text style.
pub fn muted_style() -> Style {
    muted_text_style()
}

/// Empty state hint style.
pub fn empty_hint_style() -> Style {
    Style::default()
        .fg(warning())
        .add_modifier(Modifier::ITALIC)
}

/// Global background style — paints the Catppuccin surface so text contrast is
/// controlled regardless of the host terminal being light or dark.
pub fn background_style() -> Style {
    Style::default().bg(bg()).fg(text())
}

/// Claude platform style.
pub fn claude_style() -> Style {
    Style::default().fg(claude()).add_modifier(Modifier::BOLD)
}

/// Codex platform style.
pub fn codex_style() -> Style {
    Style::default().fg(codex()).add_modifier(Modifier::BOLD)
}

/// OpenCode platform style.
pub fn opencode_style() -> Style {
    Style::default().fg(opencode()).add_modifier(Modifier::BOLD)
}

// ═══════════════════════════════════════════════════════════
// Enhanced style functions
// ═══════════════════════════════════════════════════════════

/// Get quota color based on percentage (5-level gradient, high→low).
pub fn quota_color(percentage: i32) -> Color {
    let quota = palette().quota;
    match percentage {
        90..=100 => quota[0],
        70..=89 => quota[1],
        50..=69 => quota[2],
        30..=49 => quota[3],
        _ => quota[4],
    }
}

/// Unified selected-row style (Mauve background, contrast-matched foreground).
pub fn selected_row_style() -> Style {
    Style::default()
        .bg(selection_bg())
        .fg(selection_fg())
        .add_modifier(Modifier::BOLD)
}

/// Card block accent style.
pub fn card_block_style() -> Style {
    Style::default().fg(info())
}

/// Separator style.
#[allow(dead_code)]
pub fn separator_style() -> Style {
    Style::default().fg(border())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forced_variant_from_env_parses_explicit_overrides() {
        assert_eq!(
            forced_variant_from_env(Some("mocha")),
            Some(ThemeVariant::Mocha)
        );
        assert_eq!(
            forced_variant_from_env(Some(" Latte ")),
            Some(ThemeVariant::Latte)
        );
        assert_eq!(forced_variant_from_env(Some("auto")), None);
        assert_eq!(forced_variant_from_env(Some("nonsense")), None);
        assert_eq!(forced_variant_from_env(None), None);
    }

    #[test]
    fn set_and_toggle_theme_switch_active_palette() {
        set_theme(ThemeVariant::Mocha);
        assert_eq!(active_variant(), ThemeVariant::Mocha);
        assert_eq!(palette().bg, MOCHA.bg);

        toggle_theme();
        assert_eq!(active_variant(), ThemeVariant::Latte);
        assert_eq!(palette().bg, LATTE.bg);

        toggle_theme();
        assert_eq!(active_variant(), ThemeVariant::Mocha);

        // 复位,避免污染其它串行测试。
        set_theme(ThemeVariant::Mocha);
    }

    #[test]
    fn body_text_styles_use_explicit_palette_foreground() {
        set_theme(ThemeVariant::Mocha);
        assert_eq!(primary_text_style().fg, Some(MOCHA.text));
        assert_eq!(secondary_text_style().fg, Some(MOCHA.subtext));
        assert_eq!(muted_style().fg, Some(MOCHA.muted));
        assert_eq!(background_style().bg, Some(MOCHA.bg));
        assert_eq!(background_style().fg, Some(MOCHA.text));
    }

    #[test]
    fn background_and_text_track_active_variant() {
        set_theme(ThemeVariant::Latte);
        assert_eq!(background_style().bg, Some(LATTE.bg));
        assert_eq!(primary_text_style().fg, Some(LATTE.text));
        set_theme(ThemeVariant::Mocha);
        assert_eq!(background_style().bg, Some(MOCHA.bg));
    }

    #[test]
    fn selected_row_style_uses_mauve_selection_tokens() {
        set_theme(ThemeVariant::Mocha);
        let style = selected_row_style();
        assert_eq!(style.bg, Some(MOCHA.selection_bg));
        assert_eq!(style.fg, Some(MOCHA.selection_fg));
        assert!(style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn panel_border_uses_accent_only_when_focused() {
        set_theme(ThemeVariant::Mocha);
        assert_eq!(panel_border(Platform::Claude, true).fg, Some(MOCHA.claude));
        assert_eq!(panel_border(Platform::Claude, false).fg, Some(MOCHA.border));
    }

    #[test]
    fn accent_for_distinguishes_platforms() {
        set_theme(ThemeVariant::Mocha);
        assert_eq!(accent_for(Platform::Claude), MOCHA.claude);
        assert_eq!(accent_for(Platform::Codex), MOCHA.codex);
        assert_ne!(accent_for(Platform::Claude), accent_for(Platform::Codex));
    }

    #[test]
    fn page_identity_accents_are_distinct() {
        // 三个页面共用外壳,但身份强调色必须可区分: Claude=Peach, Codex=Blue, OpenCode=Teal。
        set_theme(ThemeVariant::Mocha);
        assert_ne!(claude(), codex());
        assert_ne!(claude(), opencode());
        assert_ne!(codex(), opencode());
        set_theme(ThemeVariant::Latte);
        assert_ne!(claude(), codex());
        assert_ne!(claude(), opencode());
        assert_ne!(codex(), opencode());
        set_theme(ThemeVariant::Mocha);
    }

    #[test]
    fn tab_highlight_style_is_platform_aware_filled_chip() {
        set_theme(ThemeVariant::Mocha);
        let codex = tab_highlight_style_for(Platform::Codex);

        assert_eq!(codex.fg, Some(selection_fg()));
        assert_eq!(codex.bg, Some(accent_for(Platform::Codex)));
        assert_ne!(codex.bg, Some(accent_for(Platform::Claude)));
        assert!(codex.add_modifier.contains(Modifier::BOLD));
        assert!(!codex.add_modifier.contains(Modifier::UNDERLINED));
    }

    #[test]
    fn quota_color_follows_active_palette_gradient() {
        set_theme(ThemeVariant::Mocha);
        assert_eq!(quota_color(95), MOCHA.quota[0]);
        assert_eq!(quota_color(10), MOCHA.quota[4]);
        set_theme(ThemeVariant::Latte);
        assert_eq!(quota_color(95), LATTE.quota[0]);
        set_theme(ThemeVariant::Mocha);
    }
}
