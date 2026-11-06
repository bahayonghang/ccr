//! The single native semantic token mapping from DESIGN.md.
use gpui_kit::component::{Theme, ThemeMode};
use gpui_kit::{
    App, Background, BoxShadow, Hsla, Pixels, Size, Window, linear_color_stop, linear_gradient, px,
    rgb,
};

pub const WINDOW_WIDTH: f32 = 1280.;
pub const WINDOW_HEIGHT: f32 = 800.;
pub const MIN_WIDTH: f32 = 960.;
pub const MIN_HEIGHT: f32 = 640.;
pub const CONTENT_PADDING: f32 = 24.;
pub const SECTION_GAP: f32 = 16.;
pub const BODY_SIZE: f32 = 14.;
pub const TITLE_SIZE: f32 = 20.;
pub const DISPLAY_SIZE: f32 = 24.;
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
pub const NAV_ICON_SIZE: f32 = 16.;
pub const NAV_ROW_GAP: f32 = 4.;
pub const NAV_ROW_HEIGHT: f32 = 32.;
pub const NAV_PADDING: f32 = 12.;
pub const NAV_BRAND_HEIGHT: f32 = 40.;
pub const SECTION_TITLE_SIZE: f32 = 16.;
pub const DATE_COLUMN_WIDTH: f32 = 96.;
pub const DAILY_VALUE_WIDTH: f32 = 112.;

/// Choose an installed sans family, rather than trusting the virtual system
/// alias (which can resolve to a serif on Windows). No font file is bundled.
fn native_font(cx: &App) -> gpui_kit::SharedString {
    let names = cx.text_system().all_font_names();
    #[cfg(target_os = "windows")]
    let candidates = [
        "Segoe UI Variable",
        "Segoe UI",
        "Microsoft YaHei UI",
        "Arial",
    ];
    #[cfg(target_os = "macos")]
    let candidates = [
        ".AppleSystemUIFont",
        "Helvetica Neue",
        "PingFang SC",
        "Arial",
    ];
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let candidates = ["Noto Sans", "DejaVu Sans", "Liberation Sans", "Arial"];
    candidates
        .into_iter()
        .find(|candidate| names.iter().any(|name| name == candidate))
        .map(gpui_kit::SharedString::from)
        .unwrap_or_else(|| Theme::global(cx).font_family.clone())
}

pub fn body_font(cx: &App) -> gpui_kit::Font {
    let mut font = gpui_kit::font(Theme::global(cx).font_family.clone());
    font.fallbacks = Some(gpui_kit::FontFallbacks::from_fonts(vec![
        "Microsoft YaHei UI".into(),
        "PingFang SC".into(),
        "Noto Sans CJK SC".into(),
    ]));
    font
}

/// App-specific roles stay here; Kit reading/input/popover roles remain opaque.
#[derive(Clone, Copy)]
pub struct Palette {
    pub field: [u32; 2],
    pub sheet: u32,
    pub surface: u32,
    pub neutral: u32,
    pub elevated: u32,
    pub overlay: u32,
    pub text: u32,
    pub secondary_text: u32,
    pub muted: u32,
    pub subtle: u32,
    pub control: u32,
    pub primary: u32,
    pub primary_hover: u32,
    pub primary_active: u32,
    pub on_primary: u32,
    pub output: u32,
    pub success: u32,
    pub warning: u32,
    pub danger: u32,
    pub info: u32,
}

pub fn palette(mode: ThemeMode) -> Palette {
    if mode.is_dark() {
        Palette {
            field: [0x102b26, 0x25213d],
            sheet: 0x172225,
            surface: 0x1d292c,
            neutral: 0x10191b,
            elevated: 0x172225,
            overlay: 0x233033,
            text: 0xf0f8f5,
            secondary_text: 0xc5d5d0,
            muted: 0x9fafa9,
            subtle: 0x35474a,
            control: 0x737b86,
            primary: 0x85d5b1,
            primary_hover: 0x9ce0c3,
            primary_active: 0x70c49f,
            on_primary: 0x10191b,
            output: 0xb9a4f4,
            success: 0x86efac,
            warning: 0xfcd34d,
            danger: 0xfca5a5,
            info: 0x9bc4f5,
        }
    } else {
        Palette {
            field: [0xe7f6f1, 0xede7f8],
            sheet: 0xf2f7f5,
            surface: 0xffffff,
            neutral: 0xe7f1ee,
            elevated: 0xf2f7f5,
            overlay: 0xe3ece8,
            text: 0x1e2932,
            secondary_text: 0x404e59,
            muted: 0x5c6b76,
            subtle: 0xcedbd5,
            control: 0x787f89,
            primary: 0x156f4e,
            primary_hover: 0x146a4a,
            primary_active: 0x0f5d40,
            on_primary: 0xffffff,
            output: 0x6d4bd6,
            success: 0x166534,
            warning: 0x92400e,
            danger: 0xb91c1c,
            info: 0x245cb3,
        }
    }
}

pub fn field(mode: ThemeMode) -> Background {
    let colors = palette(mode).field;
    linear_gradient(
        160.,
        linear_color_stop(rgb(colors[0]), 0.),
        linear_color_stop(rgb(colors[1]), 1.),
    )
}

pub struct Sheet {
    pub background: Hsla,
    pub inset: Pixels,
    pub shadow: BoxShadow,
}

/// The locked renderer has no in-app backdrop-sampling API. Keep the approved
/// fallback opaque even when native transparency/accessibility preferences are unknown.
/// OS window Blurred/Mica are unrelated and are never enabled by this policy.
pub fn solid_sheet(mode: ThemeMode, viewport: Size<Pixels>) -> Sheet {
    let spare_width = (f32::from(viewport.width) - MIN_WIDTH).max(0.);
    let spare_height = (f32::from(viewport.height) - MIN_HEIGHT).max(0.);
    Sheet {
        background: Hsla::from(rgb(palette(mode).sheet)),
        // Preserve the original minimum content area; scene margins are expendable.
        inset: px((spare_width.min(spare_height) / 2.).min(12.)),
        shadow: BoxShadow::new(
            px(0.),
            px(20.),
            Hsla::from(rgb(0)).opacity(if mode.is_dark() { 0.24 } else { 0.08 }),
        )
        .blur_radius(px(48.)),
    }
}

pub const SHEET_RADIUS: f32 = 20.;
pub const SHEET_CONTENT_RADIUS: f32 = SHEET_RADIUS - 1.;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum Preference {
    #[default]
    System,
    Light,
    Dark,
}

pub fn init(cx: &mut App) {
    apply_palette(cx);
    if std::env::var_os("CCR_GPUI_FONT_DIAGNOSTICS").is_some()
        && !cx.text_system().all_font_names().is_empty()
    {
        let font = body_font(cx);
        let resolved = cx
            .text_system()
            .get_font_for_id(cx.text_system().resolve_font(&font));
        if let Some(resolved) = resolved {
            eprintln!(
                "CCR native UI font: requested={}, resolved={}",
                font.family, resolved.family
            );
        }
    }
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
    let font_family = native_font(cx);
    Theme::update(cx, |theme| {
        theme.font_family = font_family;
        theme.font_size = px(BODY_SIZE);
        theme.mono_font_size = px(META_SIZE);
        theme.radius = px(INPUT_RADIUS);
        theme.shadow = false;
        theme.motion.duration_fast = std::time::Duration::ZERO;
        theme.motion.duration_normal = std::time::Duration::ZERO;
        theme.motion.duration_slow = std::time::Duration::ZERO;
        let p = palette(theme.mode);
        let color = |value| Hsla::from(rgb(value));
        let c = &mut theme.colors;
        c.background = color(p.surface);
        c.foreground = color(p.text);
        c.muted = color(p.neutral);
        c.muted_foreground = color(p.muted);
        c.border = color(p.subtle);
        c.input = color(p.control);
        c.ring = color(p.primary);
        c.caret = color(p.text);
        c.popover = color(p.overlay);
        c.popover_foreground = color(p.text);
        c.primary = color(p.primary);
        c.primary_foreground = color(p.on_primary);
        c.primary_hover = color(p.primary_hover);
        c.primary_active = color(p.primary_active);
        c.secondary = color(p.elevated);
        c.secondary_foreground = color(p.secondary_text);
        c.secondary_hover = color(p.neutral);
        c.secondary_active = color(p.elevated);
        c.accent = color(p.elevated);
        c.accent_foreground = color(p.text);
        c.sidebar = color(p.neutral);
        c.sidebar_foreground = color(p.text);
        c.sidebar_border = color(p.subtle);
        c.sidebar_accent = color(p.elevated);
        c.sidebar_accent_foreground = color(p.text);
        c.sidebar_primary = color(p.primary);
        c.sidebar_primary_foreground = color(p.on_primary);
        c.list = color(p.surface);
        c.list_hover = color(p.neutral);
        c.list_active = color(p.overlay);
        c.list_active_border = color(p.primary);
        c.selection = color(p.overlay);
        c.button = color(p.surface);
        c.button_foreground = color(p.text);
        c.button_hover = color(p.neutral);
        c.button_active = color(p.elevated);
        c.button_primary = color(p.primary);
        c.button_primary_foreground = color(p.on_primary);
        c.button_primary_hover = color(p.primary_hover);
        c.button_primary_active = color(p.primary_active);
        c.button_secondary = color(p.elevated);
        c.button_secondary_foreground = color(p.text);
        c.button_secondary_hover = color(p.neutral);
        c.button_secondary_active = color(p.elevated);
        c.link = color(p.primary);
        c.link_hover = color(p.primary_hover);
        c.link_active = color(p.primary_active);
        c.danger = color(p.danger);
        c.success = color(p.success);
        c.warning = color(p.warning);
        c.info = color(p.info);
        c.title_bar = color(p.surface);
        c.title_bar_border = color(p.subtle);
        c.status_bar = color(p.surface);
        c.status_bar_border = color(p.subtle);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_kit::size;

    fn luminance(color: u32) -> f64 {
        [16, 8, 0]
            .into_iter()
            .zip([0.2126, 0.7152, 0.0722])
            .map(|(shift, weight)| {
                let channel = ((color >> shift) & 255) as f64 / 255.;
                weight
                    * if channel <= 0.04045 {
                        channel / 12.92
                    } else {
                        ((channel + 0.055) / 1.055).powf(2.4)
                    }
            })
            .sum()
    }

    fn contrast(a: u32, b: u32) -> f64 {
        let (a, b) = (luminance(a), luminance(b));
        (a.max(b) + 0.05) / (a.min(b) + 0.05)
    }

    #[test]
    fn solid_material_never_requires_unknown_platform_capabilities_or_squeezes_minimum() {
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            for (width, height, inset) in [(1280., 800., 12.), (960., 640., 0.), (940., 620., 0.)] {
                let sheet = solid_sheet(mode, size(px(width), px(height)));
                assert_eq!(
                    sheet.background.a, 1.,
                    "fallback must never sample the scene"
                );
                assert_eq!(sheet.inset, px(inset));
            }
        }
    }

    #[test]
    fn real_surface_pairs_preserve_ink_focus_action_output_and_status_contrast() {
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            let p = palette(mode);
            // The sheet is alpha=1, so gradient endpoints/midpoint all composite
            // to this same background. Translucent glass is not claimed or enabled.
            for surface in [p.sheet, p.surface, p.neutral, p.elevated, p.overlay] {
                for (ink, minimum) in [
                    (p.text, 12.),
                    (p.secondary_text, 7.),
                    (p.muted, 4.5),
                    (p.primary, 4.5),
                    (p.primary_hover, 4.5),
                    (p.primary_active, 4.5),
                    (p.output, 4.5),
                    (p.success, 4.5),
                    (p.warning, 4.5),
                    (p.danger, 4.5),
                    (p.info, 4.5),
                    (p.control, 3.),
                ] {
                    assert!(
                        contrast(ink, surface) >= minimum,
                        "{ink:06x} on {surface:06x}: {} < {minimum}",
                        contrast(ink, surface)
                    );
                }
            }
            for action in [p.primary, p.primary_hover, p.primary_active] {
                assert!(contrast(p.on_primary, action) >= 4.5);
            }
            assert_ne!(p.primary, p.success);
            assert_ne!(p.primary, p.output);
        }
    }
}
