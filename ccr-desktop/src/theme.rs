//! The single native semantic token mapping from DESIGN.md.
use gpui_kit::component::{Theme, ThemeMode};
use gpui_kit::{
    App, Background, BoxShadow, Hsla, Pixels, Point, Size, Window, linear_color_stop,
    linear_gradient, point, px, rgb,
};

pub const WINDOW_WIDTH: f32 = 1280.;
pub const WINDOW_HEIGHT: f32 = 800.;
pub const MIN_WIDTH: f32 = 960.;
pub const MIN_HEIGHT: f32 = 640.;
pub const CONTENT_PADDING: f32 = 24.;
pub const NARROW_CONTENT_PADDING: f32 = 16.;
pub const SECTION_GAP: f32 = 16.;
pub const BODY_SIZE: f32 = 14.;
pub const TITLE_SIZE: f32 = 20.;
pub const DISPLAY_SIZE: f32 = 24.;
pub const ROW_HEIGHT: f32 = 36.;
pub const META_SIZE: f32 = 12.;
pub const CONTROL_GAP: f32 = 8.;
pub const SELECTION_RADIUS: f32 = 4.;
pub const INPUT_RADIUS: f32 = 8.;
pub const PANEL_RADIUS: f32 = 12.;
pub const PILL_RADIUS: f32 = 9999.;
/// 分段胶囊容器角：32px 选项 + 3px 内边距 + 1px 亮边的一半，换行时与内侧 pill 同心。
pub const CAPSULE_RADIUS: f32 = 20.;
pub const CAPSULE_PADDING: f32 = 3.;
pub const SIDEBAR_WIDTH: f32 = 216.;
pub const NARROW_SIDEBAR: f32 = 184.;
pub const NARROW_WIDTH: f32 = 1040.;
pub const SPLIT_WIDTH: f32 = 1200.;
/// 列表/详情并排时的最小宽度（DESIGN.md Layout：列表 ≥360、详情 ≥320）。
pub const LIST_MIN_WIDTH: f32 = 360.;
pub const DETAIL_MIN_WIDTH: f32 = 320.;
pub const STATUS_HEIGHT: f32 = 28.;
pub const NAV_ICON_SIZE: f32 = 16.;
pub const NAV_ROW_GAP: f32 = 4.;
pub const NAV_ROW_HEIGHT: f32 = 32.;
pub const NAV_PADDING: f32 = 12.;
pub const NAV_BRAND_HEIGHT: f32 = 40.;
pub const SECTION_TITLE_SIZE: f32 = 16.;
pub const DATE_COLUMN_WIDTH: f32 = 96.;
pub const DAILY_VALUE_WIDTH: f32 = 112.;
/// Overview KPI band: cost is the primary reading, other counts are secondary.
pub const KPI_PRIMARY_SIZE: f32 = 32.;
pub const KPI_SIZE: f32 = 24.;
/// 读数为文字状态（未索引 / 不可用）时的字号，避免 24px 的大号词语。
pub const KPI_STATE_SIZE: f32 = 16.;
pub const OVERVIEW_CHART_HEIGHT: f32 = 216.;
pub const CHART_BAR_RADIUS: f32 = 2.;
/// 图表系列相对平台身份色的饱和度系数；身份色相不变，只降低饱和度。
pub const CHART_SERIES_SATURATION: f32 = 0.72;
pub const SPARK_CELL_MIN_WIDTH: f32 = 158.;
pub const SPARK_HEIGHT: f32 = 34.;
/// 非交互状态标签高度；4px 角矩形，与 pill 形状的筛选 chip 区分可点与不可点。
pub const TAG_HEIGHT: f32 = 22.;
pub const FILTER_CHIP_HEIGHT: f32 = 28.;
pub const DOT_SIZE: f32 = 8.;
pub const SEARCH_WIDTH: f32 = 300.;
pub const EMPTY_ICON_SIZE: f32 = 24.;
/// 两行列表行定高：Agent sessions 60px；MCP Manager 56px。
pub const TWO_LINE_ROW_HEIGHT: f32 = 60.;
pub const MCP_ROW_HEIGHT: f32 = 56.;
/// 概览右栏占内容宽的比例；取 8 的倍数后限制在 MIN..=MAX。
pub const OVERVIEW_RAIL_SHARE: f32 = 0.24;
pub const OVERVIEW_RAIL_MIN: f32 = 280.;
pub const OVERVIEW_RAIL_MAX: f32 = 360.;
/// 热力图格子随主列宽度取值的上下界（窄于下界时在面板内横向滚动）。
pub const HEATMAP_CELL_MIN: f32 = 8.;
pub const HEATMAP_CELL_MAX: f32 = 14.;
pub const HEATMAP_GAP: f32 = 2.;
/// 星期标签列：12px 文字约 24px 宽，另留 4px 与网格的间距。
pub const HEATMAP_LABEL_WIDTH: f32 = 28.;
pub const DISTRIBUTION_HEIGHT: f32 = 160.;
/// 排行行高：8 行时与分布面板大致等高，不用列表页的 36px。
pub const OVERVIEW_LEADER_ROW_HEIGHT: f32 = 28.;
pub const SETTINGS_NAV_WIDTH: f32 = 176.;

/// One process-local directory shared by the default resolver and both pickers.
pub struct FontCatalog(pub Vec<String>);
impl gpui_kit::Global for FontCatalog {}

pub fn font_catalog(cx: &mut App) -> &[String] {
    if cx.try_global::<FontCatalog>().is_none() {
        let mut names = cx.text_system().all_font_names();
        names.retain(|name| {
            !name.trim().is_empty() && name.len() <= 256 && !name.chars().any(char::is_control)
        });
        names.sort();
        names.dedup();
        cx.set_global(FontCatalog(names));
    }
    &cx.global::<FontCatalog>().0
}

/// Text roles use the current UI baseline; geometry remains independently owned.
pub fn text_size(base: f32) -> gpui_kit::Rems {
    gpui_kit::rems(base / BODY_SIZE)
}

/// Virtual rows must admit the independently sized code/path line as well.
pub fn data_row_height(base: f32, cx: &App) -> Pixels {
    let theme = Theme::global(cx);
    px(base
        * (f32::from(theme.font_size) / BODY_SIZE).max(f32::from(theme.mono_font_size) / META_SIZE))
}

pub fn apply_typography(value: &crate::runtime::preferences::TypographyPreferences, cx: &mut App) {
    let names = font_catalog(cx).to_vec();
    let available = |name: &Option<String>| {
        name.as_ref()
            .filter(|name| names.contains(name))
            .cloned()
            .map(gpui_kit::SharedString::from)
    };
    let ui = available(&value.ui_font).unwrap_or_else(|| native_font(cx));
    let mono = available(&value.mono_font).unwrap_or_else(|| {
        [
            "Consolas",
            "Menlo",
            "DejaVu Sans Mono",
            "Liberation Mono",
            "Cascadia Mono",
        ]
        .into_iter()
        .find(|candidate| names.iter().any(|name| name == candidate))
        .map(gpui_kit::SharedString::from)
        .unwrap_or_else(|| Theme::global(cx).mono_font_family.clone())
    });
    Theme::update(cx, |theme| {
        theme.font_family = ui;
        theme.mono_font_family = mono;
        theme.font_size = px(f32::from(value.ui_size));
        theme.mono_font_size = px(f32::from(value.mono_size));
    });
}

/// Choose an installed sans family, rather than trusting the virtual system
/// alias (which can resolve to a serif on Windows). No font file is bundled.
fn native_font(cx: &App) -> gpui_kit::SharedString {
    let names = &cx.global::<FontCatalog>().0;
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

/// Translucent material roles; every value is `(rgb, alpha)`.
#[derive(Clone, Copy)]
pub struct MaterialTokens {
    pub glows: [u32; 2],
    pub glow_alpha: f32,
    pub chrome: (u32, f32),
    pub sheet: (u32, f32),
    pub highlight: (u32, f32),
    pub tab_well: (u32, f32),
    /// 漫射阴影与近影的 alpha（0/20/48 与 0/2/6）。
    pub shadow_alpha: [f32; 2],
}

/// Neutral non-peak sparkline bars retain graphic contrast on the reading surface.
pub fn sparkline_neutral(mode: ThemeMode) -> Hsla {
    rgb(palette(mode).muted).into()
}

/// App-specific roles stay here; Kit reading/input/popover roles remain opaque.
#[derive(Clone, Copy)]
pub struct Palette {
    pub field: [u32; 2],
    pub material: MaterialTokens,
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
            material: MaterialTokens {
                glows: [0x1f4a40, 0x3a2f66],
                glow_alpha: 0.5,
                chrome: (0x1d292c, 0.55),
                sheet: (0x172225, 0.86),
                highlight: (0xffffff, 0.08),
                tab_well: (0x10191b, 0.6),
                shadow_alpha: [0.30, 0.18],
            },
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
            material: MaterialTokens {
                glows: [0xbfeadb, 0xd9cdf5],
                glow_alpha: 0.55,
                chrome: (0xffffff, 0.58),
                sheet: (0xffffff, 0.76),
                highlight: (0xffffff, 0.8),
                tab_well: (0xe7f1ee, 0.7),
                shadow_alpha: [0.05, 0.03],
            },
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

/// Decorative separators only; control boundaries retain the opaque control role.
pub fn hairline(mode: ThemeMode) -> Hsla {
    Hsla::from(rgb(palette(mode).subtle)).opacity(if mode.is_dark() { 0.35 } else { 0.24 })
}

pub struct TabColors {
    /// 胶囊凹槽：纸面上的浅 tint，不是第二层材质。
    pub background: Hsla,
    pub rim: Hsla,
    pub selected: Hsla,
    pub selected_shadow: BoxShadow,
    pub foreground: Hsla,
    pub selected_foreground: Hsla,
}

pub fn tab_colors(mode: ThemeMode) -> TabColors {
    let p = palette(mode);
    TabColors {
        background: alpha(p.material.tab_well),
        rim: alpha(p.material.highlight),
        selected: rgb(tab_selected(mode)).into(),
        selected_shadow: BoxShadow::new(
            px(0.),
            px(1.),
            Hsla::from(rgb(0)).opacity(if mode.is_dark() { 0.32 } else { 0.08 }),
        )
        .blur_radius(px(3.)),
        foreground: rgb(p.secondary_text).into(),
        selected_foreground: rgb(p.primary).into(),
    }
}

/// 选中项是浮起的实色 pill：浅色用 reading 白，深色用更亮一阶的 overlay。
pub fn tab_selected(mode: ThemeMode) -> u32 {
    let p = palette(mode);
    if mode.is_dark() { p.overlay } else { p.surface }
}

fn alpha((color, alpha): (u32, f32)) -> Hsla {
    Hsla::from(rgb(color)).opacity(alpha)
}

/// Sidebar row fills: a translucent reading pill over chrome, never a border.
pub const NAV_SELECTED_ALPHA: f32 = 0.7;
pub const NAV_HOVER_ALPHA: f32 = 0.35;
pub const NAV_PRESSED_ALPHA: f32 = 0.85;

pub struct NavColors {
    pub selected: Hsla,
    pub hover: Hsla,
    pub pressed: Hsla,
    pub rim: BoxShadow,
}

pub fn nav_colors(mode: ThemeMode) -> NavColors {
    let reading = Hsla::from(rgb(palette(mode).surface));
    NavColors {
        selected: reading.opacity(NAV_SELECTED_ALPHA),
        hover: reading.opacity(NAV_HOVER_ALPHA),
        pressed: reading.opacity(NAV_PRESSED_ALPHA),
        rim: rim(mode),
    }
}

/// inset 0 1px 顶部高光。
fn rim(mode: ThemeMode) -> BoxShadow {
    BoxShadow::new(px(0.), px(1.), alpha(palette(mode).material.highlight)).inset()
}

/// Platform identity ink shared by navigation icons, identity dots and usage
/// series. Agents without an approved identity (such as omp) stay muted.
pub fn platform_color(id: &str, mode: ThemeMode) -> Hsla {
    let (light, dark) = match id {
        "claude" => (0xa15840, 0xe3987b),
        "codex" => (0x547458, 0x89bd91),
        "grok" => (0x726985, 0xb9abd7),
        "antigravity" | "gemini" => (0x617081, 0xa9c1dc),
        "opencode" => (0x735f52, 0xc9b6a8),
        // 会话家族 id 是 kimi，用量来源 id 是 kimi_code（概览平台带与图表系列）。
        "kimi" | "kimi_code" => (0xa3467f, 0xd775ae),
        "pi" => (0x7b56b0, 0xac88e7),
        _ => return rgb(palette(mode).muted).into(),
    };
    rgb(if mode.is_dark() { dark } else { light }).into()
}

/// Semantic tone of a status tag; colour always accompanies the tag's text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tone {
    Neutral,
    Success,
    Warning,
    Danger,
    Info,
}

/// Opaque status-tag colours. Fill and edge are pre-composited over the reading
/// surface, so a tag never adds a translucent layer to the material stack.
#[derive(Clone, Copy)]
pub struct ToneColors {
    pub ink: Hsla,
    pub fill: Hsla,
    pub edge: Hsla,
}

/// Ink, fill and edge of one status tone in this theme mode.
pub fn tone_colors(tone: Tone, mode: ThemeMode) -> ToneColors {
    let [ink, fill, edge] = tone_rgb(tone, mode).map(|color| Hsla::from(rgb(color)));
    ToneColors { ink, fill, edge }
}

/// `[ink, fill, edge]`：fill/edge 是 ink 预合成到 reading 上的不透明色。
fn tone_rgb(tone: Tone, mode: ThemeMode) -> [u32; 3] {
    let p = palette(mode);
    let ink = match tone {
        Tone::Neutral => p.secondary_text,
        Tone::Success => p.success,
        Tone::Warning => p.warning,
        Tone::Danger => p.danger,
        Tone::Info => p.info,
    };
    // 标签不是控件，由文字识别状态；edge 只是同色相的装饰描边（与 hairline 同理），
    // 不适用控件边界 3:1。描边过重会让标签读作可点击的描边按钮，削弱“直角标签不可点、
    // pill 可点”的形状区分。
    let (fill, edge) = if mode.is_dark() {
        (0.18, 0.32)
    } else {
        (0.1, 0.24)
    };
    [ink, over(p.surface, ink, fill), over(p.surface, ink, edge)]
}

/// Activity heat-map cell: level 0 is a quiet neutral cell, levels 1 to 4 step
/// up through the primary ink. Levels above 4 render as 4.
pub fn heat_level(level: u8, mode: ThemeMode) -> Hsla {
    rgb(heat_rgb(level, mode)).into()
}

fn heat_rgb(level: u8, mode: ThemeMode) -> u32 {
    let p = palette(mode);
    match level {
        0 => p.neutral,
        1 => over(p.surface, p.primary, 0.25),
        2 => over(p.surface, p.primary, 0.45),
        3 => over(p.surface, p.primary, 0.7),
        _ => p.primary,
    }
}

/// 在 sRGB 空间把 `ink` 以 `alpha` 叠到不透明 `base` 上（与合成对比度单测同一公式）。
fn over(base: u32, ink: u32, alpha: f32) -> u32 {
    [16, 8, 0].into_iter().fold(0, |mixed, shift| {
        let channel = |color: u32| ((color >> shift) & 255) as f32;
        let value = channel(base) * (1. - alpha) + channel(ink) * alpha;
        mixed | ((value.round() as u32) << shift)
    })
}

/// Chart series ink: the platform identity hue with reduced saturation, shared
/// by bars and legend swatches so the legend always matches the plot.
pub fn chart_series(id: &str, mode: ThemeMode) -> Hsla {
    let identity = platform_color(id, mode);
    Hsla {
        s: identity.s * CHART_SERIES_SATURATION,
        ..identity
    }
}

/// OpenType tabular figures on the system UI font (see the task research
/// `numeric-font-probe.md`); fonts without `tnum` keep their default digits.
pub fn tabular_figures() -> gpui_kit::FontFeatures {
    // 渲染路径每帧多处调用：只分配一次，之后仅克隆 Arc。
    static TNUM: std::sync::LazyLock<gpui_kit::FontFeatures> = std::sync::LazyLock::new(|| {
        gpui_kit::FontFeatures(std::sync::Arc::new(vec![("tnum".into(), 1)]))
    });
    TNUM.clone()
}

/// One static soft light spot, painted as a transparent circle with a large
/// same-colour blur shadow (GPUI has no radial gradient).
#[derive(Clone)]
pub struct Glow {
    pub origin: Point<Pixels>,
    pub diameter: Pixels,
    pub shadow: BoxShadow,
}

/// Translucent layered material over the app-painted static low-frequency scene.
///
/// Validity conditions (DESIGN.md Elevation): only the scene sits behind a
/// translucent surface, chrome and sheet are siblings (no glass on glass),
/// and reading surfaces stay opaque. OS window Blurred/Mica are never enabled.
#[derive(Clone)]
pub struct Material {
    pub scene: Background,
    pub glows: [Glow; 2],
    pub chrome: Hsla,
    pub sheet: Hsla,
    pub reading: Hsla,
    pub highlight: Hsla,
    /// 侧栏与纸面共用的 inset 顶部高光。
    pub rim: BoxShadow,
    /// 外层框架的漫射阴影与近影。
    pub shadow: [BoxShadow; 2],
    pub inset: Pixels,
}

pub const SCENE_INSET_MAX: f32 = 20.;

/// Scene margin around the application frame at this viewport.
pub fn scene_inset(viewport: Size<Pixels>) -> f32 {
    let spare = (f32::from(viewport.width) - MIN_WIDTH)
        .min(f32::from(viewport.height) - MIN_HEIGHT)
        .max(0.);
    // 保留原最小内容区；场景边距可收敛到 0。
    (spare / 8.).clamp(0., SCENE_INSET_MAX)
}

/// Sidebar width at this viewport width.
pub fn sidebar_width(viewport_width: Pixels) -> f32 {
    if viewport_width < px(NARROW_WIDTH) {
        NARROW_SIDEBAR
    } else {
        SIDEBAR_WIDTH
    }
}

/// Page padding at this viewport width.
pub fn content_padding(viewport_width: Pixels) -> f32 {
    if viewport_width < px(NARROW_WIDTH) {
        NARROW_CONTENT_PADDING
    } else {
        CONTENT_PADDING
    }
}

pub fn material(mode: ThemeMode, viewport: Size<Pixels>) -> Material {
    let p = palette(mode);
    let m = p.material;
    let (width, height) = (f32::from(viewport.width), f32::from(viewport.height));
    // 光斑半径取视口短边的 0.35；圆盘半径 0.6、模糊 0.4，峰值柔和不出硬边。
    let reach = width.min(height).max(1.) * 0.35;
    let glow = |color: u32, (x, y): (f32, f32)| {
        let radius = reach * 0.6;
        Glow {
            origin: point(px(width * x - radius), px(height * y - radius)),
            diameter: px(radius * 2.),
            shadow: BoxShadow::new(px(0.), px(0.), alpha((color, m.glow_alpha)))
                .blur_radius(px(reach * 0.4)),
        }
    };
    let black = Hsla::from(rgb(0));
    Material {
        scene: field(mode),
        glows: [
            glow(m.glows[0], (0.12, 0.18)),
            glow(m.glows[1], (0.88, 0.85)),
        ],
        chrome: alpha(m.chrome),
        sheet: alpha(m.sheet),
        reading: rgb(p.surface).into(),
        highlight: alpha(m.highlight),
        rim: rim(mode),
        shadow: [
            BoxShadow::new(px(0.), px(20.), black.opacity(m.shadow_alpha[0])).blur_radius(px(48.)),
            BoxShadow::new(px(0.), px(2.), black.opacity(m.shadow_alpha[1])).blur_radius(px(6.)),
        ],
        inset: px(scene_inset(viewport)),
    }
}

pub const SHEET_RADIUS: f32 = 20.;
/// 纸面上、右、下的亮边；左侧与侧栏并列，不画。
pub const SHEET_RIM: f32 = 1.;
pub const SHEET_CONTENT_RADIUS: f32 = SHEET_RADIUS - SHEET_RIM;

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
    font_catalog(cx);
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
        c.border = hairline(theme.mode);
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
        c.secondary_active = color(p.overlay);
        c.accent = color(p.elevated);
        c.accent_foreground = color(p.text);
        c.sidebar = color(p.neutral);
        c.sidebar_foreground = color(p.text);
        c.sidebar_border = hairline(theme.mode);
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
        c.title_bar_border = hairline(theme.mode);
        c.status_bar = color(p.surface);
        c.status_bar_border = hairline(theme.mode);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_kit::size;

    #[gpui_kit::test]
    fn unavailable_families_keep_independent_sizes_and_code_rows_fit(
        cx: &mut gpui_kit::TestAppContext,
    ) {
        cx.update(|cx| {
            gpui_kit::init(cx);
            font_catalog(cx);
            let preferences = crate::runtime::preferences::TypographyPreferences {
                ui_font: Some("Fixture uninstalled UI font".into()),
                mono_font: Some("Fixture uninstalled code font".into()),
                ui_size: 12,
                mono_size: 20,
            };
            apply_typography(&preferences, cx);
            assert_ne!(
                Theme::global(cx).font_family.as_ref(),
                "Fixture uninstalled UI font"
            );
            assert_ne!(
                Theme::global(cx).mono_font_family.as_ref(),
                "Fixture uninstalled code font"
            );
            assert_eq!(Theme::global(cx).font_size, px(12.));
            assert_eq!(Theme::global(cx).mono_font_size, px(20.));
            assert_eq!(data_row_height(TWO_LINE_ROW_HEIGHT, cx), px(100.));
            assert_eq!(
                preferences.ui_font.as_deref(),
                Some("Fixture uninstalled UI font")
            );
        });
    }

    type Rgb = [f64; 3];

    fn channels(color: u32) -> Rgb {
        [16, 8, 0].map(|shift| ((color >> shift) & 255) as f64)
    }

    fn luminance(color: Rgb) -> f64 {
        color
            .into_iter()
            .zip([0.2126, 0.7152, 0.0722])
            .map(|(channel, weight)| {
                let channel = channel / 255.;
                weight
                    * if channel <= 0.04045 {
                        channel / 12.92
                    } else {
                        ((channel + 0.055) / 1.055).powf(2.4)
                    }
            })
            .sum()
    }

    fn contrast(a: Rgb, b: Rgb) -> f64 {
        let (a, b) = (luminance(a), luminance(b));
        (a.max(b) + 0.05) / (a.min(b) + 0.05)
    }

    /// 在 sRGB 空间把 alpha 前景合成到背景上（与 Tauri CSS 合成一致）。
    fn composite(background: Rgb, (color, alpha): (u32, f32)) -> Rgb {
        let alpha = f64::from(alpha);
        let color = channels(color);
        [0, 1, 2].map(|i| background[i] * (1. - alpha) + color[i] * alpha)
    }

    /// 场景两端、中点、两个光斑峰值，以及叠加整组外阴影后的最坏背景。
    fn scene_samples(mode: ThemeMode) -> Vec<Rgb> {
        let p = palette(mode);
        let m = p.material;
        let ends = p.field.map(channels);
        let middle = [0, 1, 2].map(|i| (ends[0][i] + ends[1][i]) / 2.);
        let base = [ends[0], ends[1], middle];
        let mut samples = base.to_vec();
        for glow in m.glows {
            samples.extend(base.map(|sample| composite(sample, (glow, m.glow_alpha))));
        }
        let shadow = 1. - (1. - m.shadow_alpha[0]) * (1. - m.shadow_alpha[1]);
        let shadowed: Vec<Rgb> = samples
            .iter()
            .map(|sample| composite(*sample, (0, shadow)))
            .collect();
        samples.extend(shadowed);
        samples
    }

    fn inks(p: &Palette) -> [(u32, f64); 12] {
        [
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
        ]
    }

    fn assert_inks(label: &str, p: &Palette, surfaces: &[Rgb]) {
        for surface in surfaces {
            for (ink, minimum) in inks(p) {
                let ratio = contrast(channels(ink), *surface);
                assert!(
                    ratio >= minimum,
                    "{label}: {ink:06x} on {surface:?}: {ratio} < {minimum}"
                );
            }
        }
    }

    #[test]
    fn material_reveals_scene_by_default_and_never_squeezes_minimum() {
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            for (width, height, inset) in [
                (1280., 800., 20.),
                (1918., 1374., 20.),
                (1100., 700., 7.5),
                (960., 640., 0.),
                (940., 620., 0.),
            ] {
                let material = material(mode, size(px(width), px(height)));
                assert_eq!(material.inset, px(inset), "{width}x{height}");
                assert!(material.chrome.a < 1. && material.sheet.a < 1.);
                assert!(material.chrome.a < material.sheet.a);
                assert_eq!(material.reading.a, 1., "reading surfaces stay opaque");
                assert!(material.rim.inset && !material.shadow.iter().any(|s| s.inset));
                for glow in &material.glows {
                    assert!(glow.shadow.color.a > 0. && glow.shadow.blur_radius > px(0.));
                }
            }
        }
    }

    #[test]
    fn composited_material_surfaces_preserve_every_ink_contrast() {
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            let p = palette(mode);
            let m = p.material;
            let scene = scene_samples(mode);
            let chrome: Vec<Rgb> = scene.iter().map(|s| composite(*s, m.chrome)).collect();
            let sheet: Vec<Rgb> = scene.iter().map(|s| composite(*s, m.sheet)).collect();
            assert_inks("chrome", &p, &chrome);
            assert_inks("sheet", &p, &sheet);
            for (label, alpha) in [
                ("nav selected", NAV_SELECTED_ALPHA),
                ("nav hover", NAV_HOVER_ALPHA),
                ("nav pressed", NAV_PRESSED_ALPHA),
            ] {
                let pill: Vec<Rgb> = chrome
                    .iter()
                    .map(|s| composite(*s, (p.surface, alpha)))
                    .collect();
                assert_inks(label, &p, &pill);
            }
            // 凹槽 tint 落在纸面或实色阅读面上。
            let mut well: Vec<Rgb> = sheet.iter().map(|s| composite(*s, m.tab_well)).collect();
            well.push(composite(channels(p.surface), m.tab_well));
            assert_inks("tab well", &p, &well);
        }
    }

    #[test]
    fn solid_surface_pairs_preserve_ink_focus_action_output_and_status_contrast() {
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            let p = palette(mode);
            let solid = [p.surface, p.neutral, p.elevated, p.overlay].map(channels);
            assert_inks("solid", &p, &solid);
            for action in [p.primary, p.primary_hover, p.primary_active] {
                assert!(contrast(channels(p.on_primary), channels(action)) >= 4.5);
            }
            assert_ne!(p.primary, p.success);
            assert_ne!(p.primary, p.output);
        }
    }

    #[test]
    fn chart_series_keep_identity_hue_and_graphic_contrast_on_reading_surface() {
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            let reading = channels(palette(mode).surface);
            for id in ["claude", "codex", "antigravity", "opencode"] {
                let identity = platform_color(id, mode);
                let series = chart_series(id, mode);
                assert_eq!(series.h, identity.h, "{id} keeps its identity hue");
                assert!(series.s < identity.s, "{id} is desaturated");
                let rgba = gpui_kit::Rgba::from(series);
                let ink = [rgba.r, rgba.g, rgba.b].map(|c| f64::from(c) * 255.);
                // 非文字图形对比度 ≥ 3:1（WCAG 1.4.11）；精确值仍由表格通道提供。
                assert!(contrast(ink, reading) >= 3., "{id} bar on reading surface");
            }
        }
        assert_eq!(
            tabular_figures().tag_value_list(),
            &[("tnum".to_string(), 1)]
        );
    }

    #[test]
    fn neutral_sparkline_bars_keep_graphic_contrast_on_reading_surface() {
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            let rgba = gpui_kit::Rgba::from(sparkline_neutral(mode));
            let ink = [rgba.r, rgba.g, rgba.b].map(|c| f64::from(c) * 255.);
            assert!(contrast(ink, channels(palette(mode).surface)) >= 3.);
        }
    }

    #[test]
    fn status_tones_keep_text_contrast_wherever_a_tag_is_placed() {
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            let p = palette(mode);
            let reading = channels(p.surface);
            for tone in [
                Tone::Neutral,
                Tone::Success,
                Tone::Warning,
                Tone::Danger,
                Tone::Info,
            ] {
                let [ink, fill, edge] = tone_rgb(tone, mode).map(channels);
                let label = format!("{tone:?} in {mode:?}");
                // fill 不透明，标签放在阅读面或纸面上时文字都只落在 fill 上。
                assert!(contrast(ink, fill) >= 4.5, "{label}: ink on fill");
                // 描边是装饰：只要求比 fill 更清楚地勾出形状，不要求控件边界 3:1。
                assert!(
                    contrast(edge, reading) > contrast(fill, reading),
                    "{label}: edge outlines the fill"
                );
                let colors = tone_colors(tone, mode);
                assert_eq!(colors.fill.a, 1., "{label}: fill stays opaque");
                assert_eq!(colors.edge.a, 1., "{label}: edge stays opaque");
            }
        }
    }

    #[test]
    fn platform_identities_keep_graphic_contrast_and_omp_stays_muted() {
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            let p = palette(mode);
            let reading = channels(p.surface);
            let muted = Hsla::from(rgb(p.muted));
            for id in [
                "claude",
                "codex",
                "grok",
                "antigravity",
                "gemini",
                "opencode",
                "kimi",
                "kimi_code",
                "pi",
            ] {
                let identity = platform_color(id, mode);
                assert_ne!(identity, muted, "{id} has its own identity");
                let rgba = gpui_kit::Rgba::from(identity);
                let ink = [rgba.r, rgba.g, rgba.b].map(|c| f64::from(c) * 255.);
                assert!(contrast(ink, reading) >= 3., "{id} on reading surface");
            }
            assert_eq!(platform_color("omp", mode), muted);
        }
    }

    #[test]
    fn heat_levels_climb_from_a_quiet_zero_cell_to_a_legible_peak() {
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            let reading = channels(palette(mode).surface);
            let ratios: Vec<f64> = (0..=4)
                .map(|level| contrast(channels(heat_rgb(level, mode)), reading))
                .collect();
            assert!(ratios[0] < ratios[1], "zero cell is quieter: {ratios:?}");
            assert!(
                ratios[1..].windows(2).all(|pair| pair[0] < pair[1]),
                "levels 1 to 4 strictly increase: {ratios:?}"
            );
            assert!(ratios[4] >= 3., "peak is legible: {ratios:?}");
            assert_eq!(heat_rgb(9, mode), heat_rgb(4, mode), "levels clamp at 4");
            for level in 0..=4 {
                assert_eq!(heat_level(level, mode).a, 1., "cells stay opaque");
            }
        }
    }

    #[test]
    fn segmented_capsule_keeps_solid_selection_and_translucent_well() {
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            let p = palette(mode);
            let tabs = tab_colors(mode);
            assert_eq!(tabs.selected.a, 1., "selected pill is solid");
            assert!(
                tabs.background.a < 1.,
                "well is a tint, not a second material"
            );
            assert!(!tabs.selected_shadow.inset && tabs.selected_shadow.color.a > 0.);
            assert!(contrast(channels(p.primary), channels(tab_selected(mode))) >= 4.5);
            assert_ne!(tab_selected(mode), p.material.tab_well.0);
            assert_eq!(hairline(mode).a, if mode.is_dark() { 0.35 } else { 0.24 });
            let nav = nav_colors(mode);
            assert!(nav.hover.a < nav.selected.a && nav.selected.a < nav.pressed.a);
            assert!(nav.rim.inset);
        }
    }
}
