//! The single native semantic token mapping from DESIGN.md.
use gpui_kit::component::{Theme, ThemeMode};
use gpui_kit::{App, Hsla, Window, px, rgb};

pub const WINDOW_WIDTH: f32 = 1280.;
pub const WINDOW_HEIGHT: f32 = 800.;
pub const MIN_WIDTH: f32 = 960.;
pub const MIN_HEIGHT: f32 = 640.;
pub const CONTENT_PADDING: f32 = 24.;
pub const SECTION_GAP: f32 = 16.;
pub const BODY_SIZE: f32 = 14.;
pub const TITLE_SIZE: f32 = 20.;
pub const ROW_HEIGHT: f32 = 36.;
pub const META_SIZE: f32 = 12.;
pub const CONTROL_GAP: f32 = 8.;
pub const SELECTION_RADIUS: f32 = 4.;
pub const INPUT_RADIUS: f32 = 8.;
pub const PILL_RADIUS: f32 = 9999.;
pub const SIDEBAR_WIDTH: f32 = 216.;
pub const NARROW_SIDEBAR: f32 = 184.;
pub const NARROW_WIDTH: f32 = 1040.;
pub const SPLIT_WIDTH: f32 = 1200.;
pub const STATUS_HEIGHT: f32 = 28.;
pub const TOOLBAR_HEIGHT: f32 = 48.;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum Preference {
    #[default]
    System,
    Light,
    Dark,
}

pub fn init(cx: &mut App) {
    apply_palette(cx);
}

pub fn apply(preference: Preference, window: &mut Window, cx: &mut App) {
    let mode = match preference {
        Preference::System => {
            Theme::sync_system_appearance(Some(window), cx);
            Theme::global(cx).mode
        }
        Preference::Light => ThemeMode::Light,
        Preference::Dark => ThemeMode::Dark,
    };
    Theme::change(mode, Some(window), cx);
    // Mode selection reloads Kit's palette. Apply CCR in a separate update,
    // which also synchronizes the Base and Component token owners.
    apply_palette(cx);
}

fn apply_palette(cx: &mut App) {
    Theme::update(cx, |theme| {
        theme.font_size = px(BODY_SIZE);
        theme.mono_font_size = px(META_SIZE);
        theme.radius = px(INPUT_RADIUS);
        theme.shadow = false;
        theme.motion.duration_fast = std::time::Duration::ZERO;
        theme.motion.duration_normal = std::time::Duration::ZERO;
        theme.motion.duration_slow = std::time::Duration::ZERO;
        let (
            surface,
            neutral,
            elevated,
            text,
            muted,
            subtle,
            control,
            focus,
            primary,
            on_primary,
            success,
            warning,
            danger,
        ) = if theme.mode.is_dark() {
            (
                0x151617, 0x1c1e20, 0x232527, 0xe7e9ec, 0xa1a7b0, 0x34373b, 0x737b86, 0xe7e9ec,
                0xe7e9ec, 0x151617, 0x86efac, 0xfcd34d, 0xfca5a5,
            )
        } else {
            (
                0xffffff, 0xf7f7f6, 0xf0f1f2, 0x1f2328, 0x646b74, 0xe6e7e9, 0x787f89, 0x111827,
                0x000000, 0xffffff, 0x166534, 0x92400e, 0xb91c1c,
            )
        };
        let color = |value| Hsla::from(rgb(value));
        let c = &mut theme.colors;
        c.background = color(surface);
        c.foreground = color(text);
        c.muted = color(neutral);
        c.muted_foreground = color(muted);
        c.border = color(subtle);
        c.input = color(control);
        c.ring = color(focus);
        c.caret = color(text);
        c.popover = color(elevated);
        c.popover_foreground = color(text);
        c.primary = color(primary);
        c.primary_foreground = color(on_primary);
        c.primary_hover = color(text);
        c.primary_active = color(muted);
        c.secondary = color(elevated);
        c.secondary_foreground = color(text);
        c.secondary_hover = color(neutral);
        c.secondary_active = color(elevated);
        c.accent = color(elevated);
        c.accent_foreground = color(text);
        c.sidebar = color(neutral);
        c.sidebar_foreground = color(text);
        c.sidebar_border = color(subtle);
        c.sidebar_accent = color(elevated);
        c.sidebar_accent_foreground = color(text);
        c.sidebar_primary = color(primary);
        c.sidebar_primary_foreground = color(on_primary);
        c.list = color(surface);
        c.list_hover = color(neutral);
        c.list_active = color(elevated);
        c.list_active_border = color(focus);
        c.selection = color(elevated);
        c.button = color(surface);
        c.button_foreground = color(text);
        c.button_hover = color(neutral);
        c.button_active = color(elevated);
        c.button_primary = color(primary);
        c.button_primary_foreground = color(on_primary);
        c.button_primary_hover = color(text);
        c.button_primary_active = color(muted);
        c.button_secondary = color(elevated);
        c.button_secondary_foreground = color(text);
        c.button_secondary_hover = color(neutral);
        c.button_secondary_active = color(elevated);
        c.danger = color(danger);
        c.success = color(success);
        c.warning = color(warning);
        c.title_bar = color(surface);
        c.title_bar_border = color(subtle);
        c.status_bar = color(surface);
        c.status_bar_border = color(subtle);
    });
}
