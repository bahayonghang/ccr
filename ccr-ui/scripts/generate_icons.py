# /// script
# requires-python = ">=3.10"
# dependencies = ["cairosvg", "pillow"]
# ///
"""Generate CCR brand assets for the whole repository from SVG sources."""

from __future__ import annotations

from io import BytesIO
from pathlib import Path

from PIL import Image, ImageDraw

try:
    from cairosvg import svg2png as cairosvg_svg2png
except Exception:
    cairosvg_svg2png = None


SCRIPT_DIR = Path(__file__).resolve().parent
REPO_ROOT = SCRIPT_DIR.parents[1]
BRANDING_DIR = REPO_ROOT / "branding"
REFERENCE_ICON_PNG = REPO_ROOT / "icon.png"

MASTER_SVG = BRANDING_DIR / "master.svg"
APP_ICON_SVG = BRANDING_DIR / "app-icon.svg"
DISPLAY_LOGO_SVG = BRANDING_DIR / "display-logo.svg"
VSCODE_ICON_SVG = BRANDING_DIR / "vscode-icon.svg"

BRAND_BACKGROUND = (0xFF, 0xFF, 0xFF, 0xFF)
BRAND_DARK = (0x13, 0x22, 0x35, 0xFF)
BRAND_SIDE = (0x11, 0x20, 0x31, 0xFF)
BRAND_BLUE = (0x0E, 0x9F, 0xF3, 0xFF)
BRAND_ORANGE = (0xFF, 0x58, 0x00, 0xFF)


def ensure_parent(path: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)


def copy_text_asset(source: Path, target: Path) -> None:
    ensure_parent(target)
    target.write_text(source.read_text(encoding="utf-8"), encoding="utf-8")
    print(f"Updated {target.relative_to(REPO_ROOT)}")


def scale(value: float, size: int, base_size: int = 1024) -> int:
    return int(round(value * size / base_size))


def scale_points(points: list[tuple[float, float]], size: int) -> list[tuple[int, int]]:
    return [(scale(x, size), scale(y, size)) for x, y in points]


def scale_box(box: tuple[float, float, float, float], size: int) -> tuple[int, int, int, int]:
    return tuple(scale(value, size) for value in box)


def append_cubic(
    points: list[tuple[float, float]],
    control_1: tuple[float, float],
    control_2: tuple[float, float],
    end: tuple[float, float],
    steps: int = 24,
) -> None:
    start = points[-1]
    for step in range(1, steps + 1):
        t = step / steps
        inverse = 1 - t
        x = (
            inverse**3 * start[0]
            + 3 * inverse**2 * t * control_1[0]
            + 3 * inverse * t**2 * control_2[0]
            + t**3 * end[0]
        )
        y = (
            inverse**3 * start[1]
            + 3 * inverse**2 * t * control_1[1]
            + 3 * inverse * t**2 * control_2[1]
            + t**3 * end[1]
        )
        points.append((x, y))


def rounded_line(
    draw: ImageDraw.ImageDraw,
    points: list[tuple[int, int]],
    width: int,
    color: tuple[int, int, int, int],
) -> None:
    draw.line(points, fill=color, width=width, joint="curve")
    radius = width // 2
    for x, y in (points[0], points[-1]):
        draw.ellipse((x - radius, y - radius, x + radius, y + radius), fill=color)


def draw_ring(
    draw: ImageDraw.ImageDraw,
    center: tuple[float, float],
    radius: float,
    width: float,
    color: tuple[int, int, int, int],
    size: int,
) -> None:
    cx, cy = center
    half_width = width / 2
    draw.ellipse(
        scale_box((cx - radius - half_width, cy - radius - half_width, cx + radius + half_width, cy + radius + half_width), size),
        outline=color,
        width=scale(width, size),
    )


def render_native_brand_icon(size: int, variant: str = "app") -> Image.Image:
    del variant

    render_size = 1024 if size < 512 else size
    canvas = Image.new("RGBA", (render_size, render_size), BRAND_BACKGROUND)
    draw = ImageDraw.Draw(canvas)

    left_side = [(426, 258), (323, 258)]
    append_cubic(left_side, (221, 258), (148, 335), (148, 443))
    append_cubic(left_side, (148, 551), (221, 628), (323, 628))
    left_side.extend([(426, 628), (359, 561), (322, 561)])
    append_cubic(left_side, (256, 561), (211, 512), (211, 443))
    append_cubic(left_side, (211, 374), (256, 325), (322, 325))
    left_side.extend([(359, 325)])

    right_top = [(598, 258), (701, 258)]
    append_cubic(right_top, (803, 258), (876, 335), (876, 443))
    right_top.append((813, 443))
    append_cubic(right_top, (813, 374), (768, 325), (702, 325))
    right_top.extend([(665, 325)])

    right_bottom = [(665, 561), (702, 561)]
    append_cubic(right_bottom, (768, 561), (813, 512), (813, 443))
    right_bottom.append((876, 443))
    append_cubic(right_bottom, (876, 551), (803, 628), (701, 628))
    right_bottom.extend([(598, 628)])

    for shape in (left_side, right_top, right_bottom):
        draw.polygon(scale_points(shape, render_size), fill=BRAND_SIDE)

    blue_width = scale(31, render_size)
    orange_width = scale(31, render_size)

    draw_ring(draw, (512, 197), 30, 20, BRAND_BLUE, render_size)
    draw_ring(draw, (293, 443), 24, 20, BRAND_BLUE, render_size)

    blue_left = [(512, 227), (512, 275)]
    append_cubic(blue_left, (512, 296), (500, 305), (482, 316))
    append_cubic(blue_left, (435, 342), (405, 383), (405, 433))
    blue_left.extend([(405, 604), (430, 631)])
    rounded_line(draw, scale_points(blue_left, render_size), blue_width, BRAND_BLUE)

    blue_right = [(512, 275)]
    append_cubic(blue_right, (589, 303), (632, 358), (632, 429))
    append_cubic(blue_right, (632, 478), (603, 508), (558, 542))
    append_cubic(blue_right, (535, 559), (522, 579), (522, 603))
    rounded_line(draw, scale_points(blue_right, render_size), blue_width, BRAND_BLUE)
    rounded_line(draw, scale_points([(317, 443), (405, 443)], render_size), blue_width, BRAND_BLUE)

    draw_ring(draw, (735, 443), 24, 20, BRAND_ORANGE, render_size)
    draw_ring(draw, (522, 666), 24, 20, BRAND_ORANGE, render_size)
    rounded_line(draw, scale_points([(632, 443), (711, 443)], render_size), orange_width, BRAND_ORANGE)
    rounded_line(draw, scale_points([(522, 604), (522, 642)], render_size), orange_width, BRAND_ORANGE)
    rounded_line(draw, scale_points([(522, 604), (656, 734)], render_size), orange_width, BRAND_ORANGE)

    draw.polygon(scale_points([(449, 443), (501, 404), (501, 482)], render_size), fill=BRAND_DARK)
    draw.polygon(scale_points([(575, 443), (523, 404), (523, 482)], render_size), fill=BRAND_DARK)

    for offset in (0, 168):
        draw.rectangle(scale_box((286 + offset, 740, 418 + offset, 762), render_size), fill=BRAND_DARK)
        draw.rectangle(scale_box((286 + offset, 740, 306 + offset, 850), render_size), fill=BRAND_DARK)
        draw.rectangle(scale_box((286 + offset, 828, 418 + offset, 850), render_size), fill=BRAND_DARK)

    draw.rectangle(scale_box((622, 740, 652, 850), render_size), fill=BRAND_DARK)
    draw.rectangle(scale_box((622, 740, 712, 762), render_size), fill=BRAND_DARK)
    draw.rectangle(scale_box((622, 801, 716, 823), render_size), fill=BRAND_DARK)
    draw.rectangle(scale_box((704, 740, 754, 801), render_size), fill=BRAND_DARK)
    draw.polygon(scale_points([(674, 823), (718, 823), (760, 850), (712, 850)], render_size), fill=BRAND_DARK)
    draw.rectangle(scale_box((652, 762, 704, 801), render_size), fill=BRAND_BACKGROUND)

    if render_size != size:
        canvas = canvas.resize((size, size), Image.Resampling.LANCZOS)

    return canvas


def load_reference_icon(svg_path: Path) -> Image.Image:
    if not REFERENCE_ICON_PNG.exists():
        return render_svg(svg_path, 1024)

    image = Image.open(REFERENCE_ICON_PNG).convert("RGBA")
    alpha_bbox = image.getchannel("A").getbbox()
    bbox = alpha_bbox or image.getbbox()
    if not bbox:
        raise ValueError(f"Reference icon has no visible pixels: {REFERENCE_ICON_PNG}")
    cropped = image.crop(bbox)
    side = max(cropped.size)
    pad = int(round(side * 0.12))
    canvas = Image.new("RGBA", (side + pad * 2, side + pad * 2), (0, 0, 0, 0))
    offset = ((canvas.width - cropped.width) // 2, (canvas.height - cropped.height) // 2)
    canvas.alpha_composite(cropped, offset)
    return canvas


def render_svg(svg_path: Path, size: int, fallback_image: Image.Image | None = None) -> Image.Image:
    if cairosvg_svg2png is not None:
        png_bytes = cairosvg_svg2png(
            url=str(svg_path),
            output_width=size,
            output_height=size,
            background_color=None,
        )
        return Image.open(BytesIO(png_bytes)).convert("RGBA")
    if svg_path in {APP_ICON_SVG, DISPLAY_LOGO_SVG}:
        variant = "display" if svg_path == DISPLAY_LOGO_SVG else "app"
        return render_native_brand_icon(size, variant=variant)
    if fallback_image is None:
        raise RuntimeError(f"SVG rasterizer unavailable and no fallback image for {svg_path}")
    return fallback_image.resize((size, size), Image.Resampling.LANCZOS)


def render_with_padding(
    svg_path: Path,
    size: int,
    padding_ratio: float,
    fallback_image: Image.Image | None = None,
) -> Image.Image:
    canvas = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    inner_size = int(round(size * (1 - padding_ratio * 2)))
    inner = render_svg(svg_path, inner_size, fallback_image=fallback_image)
    offset = ((size - inner_size) // 2, (size - inner_size) // 2)
    canvas.alpha_composite(inner, offset)
    return canvas


def save_png(image: Image.Image, target: Path) -> None:
    ensure_parent(target)
    # On Windows, Pillow may fail opening paths with its default `w+b` mode in
    # rare cases (seen as Errno 22). Provide our own write handle instead.
    with target.open("wb") as handle:
        image.save(handle, format="PNG", optimize=True)
    print(f"Generated {target.relative_to(REPO_ROOT)} ({image.width}x{image.height})")


def save_svg_preview(
    svg_path: Path,
    target: Path,
    size: int,
    fallback_image: Image.Image | None = None,
) -> None:
    save_png(render_svg(svg_path, size, fallback_image=fallback_image), target)


def save_ico(image: Image.Image, target: Path) -> None:
    ensure_parent(target)
    ico_sizes = [(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (256, 256)]
    with target.open("wb") as handle:
        image.save(handle, format="ICO", sizes=ico_sizes)
    print(f"Generated {target.relative_to(REPO_ROOT)}")


def save_icns(image: Image.Image, target: Path) -> None:
    ensure_parent(target)
    with target.open("wb") as handle:
        image.save(handle, format="ICNS")
    print(f"Generated {target.relative_to(REPO_ROOT)}")


def export_brand_sources() -> None:
    copy_text_asset(APP_ICON_SVG, REPO_ROOT / "ccr-ui" / "src" / "assets" / "favicon.svg")
    copy_text_asset(APP_ICON_SVG, REPO_ROOT / "ccr-ui" / "public" / "icons" / "icon.svg")
    copy_text_asset(DISPLAY_LOGO_SVG, REPO_ROOT / "ccr-ui" / "public" / "icons" / "logo.svg")
    copy_text_asset(DISPLAY_LOGO_SVG, REPO_ROOT / "docs" / "public" / "logo.svg")
    copy_text_asset(APP_ICON_SVG, REPO_ROOT / "docs" / "public" / "favicon.svg")
    copy_text_asset(VSCODE_ICON_SVG, REPO_ROOT / "ccr-vscode" / "resources" / "icons" / "ccr.svg")
    copy_text_asset(APP_ICON_SVG, REPO_ROOT / "ccr-vscode" / "icon.svg")


def export_runtime_assets() -> None:
    app_fallback = load_reference_icon(APP_ICON_SVG)
    display_fallback = load_reference_icon(DISPLAY_LOGO_SVG)

    save_svg_preview(APP_ICON_SVG, REPO_ROOT / "icon.png", 512, app_fallback)
    save_svg_preview(APP_ICON_SVG, REPO_ROOT / "ccr-ui" / "public" / "icons" / "icon.png", 512, app_fallback)
    save_svg_preview(DISPLAY_LOGO_SVG, REPO_ROOT / "ccr-ui" / "public" / "icons" / "logo.png", 1024, display_fallback)
    save_svg_preview(DISPLAY_LOGO_SVG, REPO_ROOT / "ccr-ui" / "src" / "assets" / "logo.png", 640, display_fallback)
    save_svg_preview(APP_ICON_SVG, REPO_ROOT / "docs" / "public" / "favicon.png", 256, app_fallback)
    save_svg_preview(APP_ICON_SVG, REPO_ROOT / "ccr-vscode" / "icon.png", 256, app_fallback)


def export_tauri_bundle_assets() -> None:
    tauri_icons = REPO_ROOT / "ccr-ui" / "src-tauri" / "icons"
    app_fallback = load_reference_icon(APP_ICON_SVG)
    app_1024 = render_svg(APP_ICON_SVG, 1024, fallback_image=app_fallback)

    png_targets = {
        "32x32.png": 32,
        "64x64.png": 64,
        "128x128.png": 128,
        "128x128@2x.png": 256,
        "icon.png": 512,
        "icon@2x.png": 1024,
        "Square30x30Logo.png": 30,
        "Square44x44Logo.png": 44,
        "Square71x71Logo.png": 71,
        "Square89x89Logo.png": 89,
        "Square107x107Logo.png": 107,
        "Square142x142Logo.png": 142,
        "Square150x150Logo.png": 150,
        "Square284x284Logo.png": 284,
        "Square310x310Logo.png": 310,
        "StoreLogo.png": 50,
    }

    for name, size in png_targets.items():
        save_png(render_svg(APP_ICON_SVG, size, fallback_image=app_fallback), tauri_icons / name)

    save_ico(app_1024, tauri_icons / "icon.ico")
    save_icns(app_1024, tauri_icons / "icon.icns")


def export_android_assets() -> None:
    android_root = REPO_ROOT / "ccr-ui" / "src-tauri" / "icons" / "android"
    app_fallback = load_reference_icon(APP_ICON_SVG)
    launcher_sizes = {
        "mipmap-mdpi": 48,
        "mipmap-hdpi": 72,
        "mipmap-xhdpi": 96,
        "mipmap-xxhdpi": 144,
        "mipmap-xxxhdpi": 192,
    }
    foreground_sizes = {
        "mipmap-mdpi": 108,
        "mipmap-hdpi": 162,
        "mipmap-xhdpi": 216,
        "mipmap-xxhdpi": 324,
        "mipmap-xxxhdpi": 432,
    }

    for bucket, size in launcher_sizes.items():
        image = render_svg(APP_ICON_SVG, size, fallback_image=app_fallback)
        save_png(image, android_root / bucket / "ic_launcher.png")
        save_png(image, android_root / bucket / "ic_launcher_round.png")

    for bucket, size in foreground_sizes.items():
        foreground = render_with_padding(APP_ICON_SVG, size, padding_ratio=0.14, fallback_image=app_fallback)
        save_png(foreground, android_root / bucket / "ic_launcher_foreground.png")

    background_xml = """<resources>\n  <color name=\"ic_launcher_background\">#FFFFFF</color>\n</resources>\n"""
    background_path = android_root / "values" / "ic_launcher_background.xml"
    ensure_parent(background_path)
    background_path.write_text(background_xml, encoding="utf-8")
    print(f"Updated {background_path.relative_to(REPO_ROOT)}")


def export_ios_assets() -> None:
    ios_root = REPO_ROOT / "ccr-ui" / "src-tauri" / "icons" / "ios"
    app_fallback = load_reference_icon(APP_ICON_SVG)
    ios_targets = {
        "AppIcon-20x20@1x.png": 20,
        "AppIcon-20x20@2x.png": 40,
        "AppIcon-20x20@2x-1.png": 40,
        "AppIcon-20x20@3x.png": 60,
        "AppIcon-29x29@1x.png": 29,
        "AppIcon-29x29@2x.png": 58,
        "AppIcon-29x29@2x-1.png": 58,
        "AppIcon-29x29@3x.png": 87,
        "AppIcon-40x40@1x.png": 40,
        "AppIcon-40x40@2x.png": 80,
        "AppIcon-40x40@2x-1.png": 80,
        "AppIcon-40x40@3x.png": 120,
        "AppIcon-60x60@2x.png": 120,
        "AppIcon-60x60@3x.png": 180,
        "AppIcon-76x76@1x.png": 76,
        "AppIcon-76x76@2x.png": 152,
        "AppIcon-83.5x83.5@2x.png": 167,
        "AppIcon-512@2x.png": 1024,
    }

    for name, size in ios_targets.items():
        save_png(render_svg(APP_ICON_SVG, size, fallback_image=app_fallback), ios_root / name)


def main() -> None:
    required = [MASTER_SVG, APP_ICON_SVG, DISPLAY_LOGO_SVG, VSCODE_ICON_SVG]
    for path in required:
        if not path.exists():
            raise FileNotFoundError(f"Missing brand source: {path}")

    export_brand_sources()
    export_runtime_assets()
    export_tauri_bundle_assets()
    export_android_assets()
    export_ios_assets()


if __name__ == "__main__":
    main()
