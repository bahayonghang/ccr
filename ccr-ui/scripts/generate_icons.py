# /// script
# requires-python = ">=3.10"
# dependencies = ["cairosvg", "pillow"]
# ///
"""Generate CCR brand assets for the whole repository from SVG sources."""

from __future__ import annotations

from io import BytesIO
from pathlib import Path

from PIL import Image, ImageChops, ImageDraw, ImageFilter

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

APP_START = (0xF2, 0x9A, 0x68, 0xFF)
APP_END = (0xD9, 0x65, 0x43, 0xFF)
APP_STROKE = (0xFF, 0xF8, 0xF2, 0xFF)
SHADOW_COLOR = (0xA9, 0x4A, 0x2D, 0xFF)


def ensure_parent(path: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)


def copy_text_asset(source: Path, target: Path) -> None:
    ensure_parent(target)
    target.write_text(source.read_text(encoding="utf-8"), encoding="utf-8")
    print(f"Updated {target.relative_to(REPO_ROOT)}")


def scale(value: float, size: int, base_size: int = 512) -> int:
    return int(round(value * size / base_size))


def scale_points(points: list[tuple[float, float]], size: int) -> list[tuple[int, int]]:
    return [(scale(x, size), scale(y, size)) for x, y in points]


def diagonal_gradient(size: int, start: tuple[int, int, int, int], end: tuple[int, int, int, int]) -> Image.Image:
    mask = Image.linear_gradient("L").rotate(-45, expand=True).resize((size, size), Image.Resampling.BICUBIC)
    start_layer = Image.new("RGBA", (size, size), start)
    end_layer = Image.new("RGBA", (size, size), end)
    return Image.composite(end_layer, start_layer, mask)


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


def render_native_brand_icon(size: int, variant: str = "app") -> Image.Image:
    render_size = size * 2 if size < 512 else size
    canvas = Image.new("RGBA", (render_size, render_size), (0, 0, 0, 0))

    badge_mask = Image.new("L", (render_size, render_size), 0)
    badge_draw = ImageDraw.Draw(badge_mask)
    inset = scale(88, render_size)
    badge_draw.rounded_rectangle(
        (inset, inset, render_size - inset, render_size - inset),
        radius=scale(122, render_size),
        fill=255,
    )

    shadow = Image.new("RGBA", (render_size, render_size), SHADOW_COLOR)
    shadow_alpha = badge_mask.filter(ImageFilter.GaussianBlur(scale(22, render_size)))
    shadow_alpha = shadow_alpha.point(lambda alpha: min(255, int(alpha * 0.22)))
    shadow.putalpha(shadow_alpha)
    canvas.alpha_composite(shadow, (0, scale(18 if variant == "app" else 22, render_size)))

    badge = diagonal_gradient(render_size, APP_START, APP_END)
    badge.putalpha(badge_mask)
    canvas.alpha_composite(badge)

    highlight = Image.new("RGBA", (render_size, render_size), (255, 255, 255, 0))
    highlight_draw = ImageDraw.Draw(highlight)
    highlight_draw.ellipse(
        (
            scale(112, render_size),
            scale(102, render_size),
            scale(392, render_size),
            scale(270, render_size),
        ),
        fill=(255, 255, 255, 84 if variant == "app" else 96),
    )
    highlight = highlight.filter(ImageFilter.GaussianBlur(scale(18, render_size)))
    highlight.putalpha(ImageChops.multiply(highlight.getchannel("A"), badge_mask))
    canvas.alpha_composite(highlight)

    outer_points = scale_points(
        [(164, 292), (236, 190), (332, 190), (388, 258), (318, 356), (216, 356), (164, 292)],
        render_size,
    )
    inner_points = scale_points([(236, 190), (294, 258), (216, 356)], render_size)

    monogram = Image.new("RGBA", (render_size, render_size), (0, 0, 0, 0))
    monogram_draw = ImageDraw.Draw(monogram)
    monogram_draw.line(outer_points, fill=APP_STROKE, width=scale(32, render_size), joint="curve")
    rounded_line(monogram_draw, inner_points, width=scale(28, render_size), color=APP_STROKE)
    canvas.alpha_composite(monogram)

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

    background_xml = """<resources>\n  <color name=\"ic_launcher_background\">#E67E58</color>\n</resources>\n"""
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
