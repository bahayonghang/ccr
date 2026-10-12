# /// script
# requires-python = ">=3.10"
# dependencies = ["cairosvg", "pillow"]
# ///
"""Generate CCR brand assets for the whole repository from SVG sources."""

from __future__ import annotations

from functools import lru_cache
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

MASTER_SVG = BRANDING_DIR / "master.svg"
APP_ICON_SVG = BRANDING_DIR / "app-icon.svg"
DISPLAY_LOGO_SVG = BRANDING_DIR / "display-logo.svg"
VSCODE_ICON_SVG = BRANDING_DIR / "vscode-icon.svg"

BRAND_BLUE = (0x1A, 0x73, 0xE8, 0xFF)
BRAND_MARK = (0xFF, 0xFF, 0xFF, 0xFF)
BRAND_BASE = BRAND_BLUE
# 与 branding/app-icon.svg 同一组汇流坐标。
MARK_STROKE = 112
MARK_LEFT = ((156, 848), (200, 640), (370, 420), (512, 292))
MARK_RIGHT = ((868, 848), (824, 640), (654, 420), (512, 292))
MARK_NODE = (512, 228, 72)
NATIVE_RENDER_SCALE = 2
NATIVE_RENDER_SIZE = 1024 * NATIVE_RENDER_SCALE


def ensure_parent(path: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)


def copy_text_asset(source: Path, target: Path) -> None:
    ensure_parent(target)
    target.write_text(source.read_text(encoding="utf-8"), encoding="utf-8")
    print(f"Updated {target.relative_to(REPO_ROOT)}")


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
    points: list[tuple[float, float]],
    width: int,
    color: tuple[int, int, int, int],
) -> None:
    radius = width // 2
    for start, end in zip(points, points[1:]):
        draw.line((start, end), fill=color, width=width)
    for x, y in points:
        draw.ellipse((x - radius, y - radius, x + radius, y + radius), fill=color)


def cubic_points(
    curve: tuple[tuple[float, float], tuple[float, float], tuple[float, float], tuple[float, float]],
    steps: int = 32,
) -> list[tuple[float, float]]:
    start, control_1, control_2, end = curve
    points = [start]
    append_cubic(points, control_1, control_2, end, steps)
    return points


@lru_cache(maxsize=2)
def render_native_brand_master(variant: str) -> Image.Image:
    if variant == "app":
        tile_bounds = (48, 48, 976, 976)
        tile_radius = 224
    elif variant == "display":
        tile_bounds = (32, 32, 992, 992)
        tile_radius = 236
    else:
        raise ValueError(f"Unknown brand icon variant: {variant}")

    def scaled_points(points: list[tuple[float, float]]) -> list[tuple[float, float]]:
        return [(x * NATIVE_RENDER_SCALE, y * NATIVE_RENDER_SCALE) for x, y in points]

    def scaled_box(box: tuple[int, int, int, int]) -> tuple[int, int, int, int]:
        return tuple(value * NATIVE_RENDER_SCALE for value in box)

    canvas = Image.new("RGBA", (NATIVE_RENDER_SIZE, NATIVE_RENDER_SIZE), (0, 0, 0, 0))
    tile_mask = Image.new("L", canvas.size, 0)
    ImageDraw.Draw(tile_mask).rounded_rectangle(
        scaled_box(tile_bounds),
        radius=tile_radius * NATIVE_RENDER_SCALE,
        fill=0xFF,
    )
    painted = Image.new("RGBA", canvas.size, (0, 0, 0, 0))
    draw = ImageDraw.Draw(painted)
    draw.rectangle((0, 0, NATIVE_RENDER_SIZE, NATIVE_RENDER_SIZE), fill=BRAND_BLUE)
    stroke = MARK_STROKE * NATIVE_RENDER_SCALE
    rounded_line(draw, scaled_points(cubic_points(MARK_LEFT)), stroke, BRAND_MARK)
    rounded_line(draw, scaled_points(cubic_points(MARK_RIGHT)), stroke, BRAND_MARK)
    center_x, center_y, radius = (value * NATIVE_RENDER_SCALE for value in MARK_NODE)
    draw.ellipse(
        (center_x - radius, center_y - radius, center_x + radius, center_y + radius),
        fill=BRAND_MARK,
    )
    canvas.paste(painted, mask=tile_mask)
    return canvas


def render_native_brand_icon(size: int, variant: str = "app") -> Image.Image:
    master = render_native_brand_master(variant)
    if size == master.width:
        return master.copy()
    return master.resize((size, size), Image.Resampling.LANCZOS)


def render_svg(svg_path: Path, size: int) -> Image.Image:
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
    raise RuntimeError(f"SVG rasterizer unavailable for {svg_path}")


def render_with_padding(
    svg_path: Path,
    size: int,
    padding_ratio: float,
) -> Image.Image:
    canvas = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    inner_size = int(round(size * (1 - padding_ratio * 2)))
    inner = render_svg(svg_path, inner_size)
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
) -> None:
    save_png(render_svg(svg_path, size), target)


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
    save_svg_preview(APP_ICON_SVG, REPO_ROOT / "icon.png", 512)
    save_svg_preview(APP_ICON_SVG, REPO_ROOT / "ccr-ui" / "public" / "icons" / "icon.png", 512)
    save_svg_preview(DISPLAY_LOGO_SVG, REPO_ROOT / "ccr-ui" / "public" / "icons" / "logo.png", 1024)
    save_svg_preview(DISPLAY_LOGO_SVG, REPO_ROOT / "ccr-ui" / "src" / "assets" / "logo.png", 640)
    save_svg_preview(APP_ICON_SVG, REPO_ROOT / "docs" / "public" / "favicon.png", 256)
    save_svg_preview(APP_ICON_SVG, REPO_ROOT / "ccr-vscode" / "icon.png", 256)


def export_tauri_bundle_assets() -> None:
    tauri_icons = REPO_ROOT / "ccr-ui" / "src-tauri" / "icons"
    app_1024 = render_svg(APP_ICON_SVG, 1024)

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
        save_png(render_svg(APP_ICON_SVG, size), tauri_icons / name)

    save_ico(app_1024, tauri_icons / "icon.ico")
    save_icns(app_1024, tauri_icons / "icon.icns")


def export_android_assets() -> None:
    android_root = REPO_ROOT / "ccr-ui" / "src-tauri" / "icons" / "android"
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
        image = render_svg(APP_ICON_SVG, size)
        save_png(image, android_root / bucket / "ic_launcher.png")
        save_png(image, android_root / bucket / "ic_launcher_round.png")

    for bucket, size in foreground_sizes.items():
        foreground = render_with_padding(APP_ICON_SVG, size, padding_ratio=0.14)
        save_png(foreground, android_root / bucket / "ic_launcher_foreground.png")

    background_color = "#{:02X}{:02X}{:02X}".format(*BRAND_BASE[:3])
    background_xml = f"""<resources>\n  <color name=\"ic_launcher_background\">{background_color}</color>\n</resources>\n"""
    background_path = android_root / "values" / "ic_launcher_background.xml"
    ensure_parent(background_path)
    background_path.write_text(background_xml, encoding="utf-8")
    print(f"Updated {background_path.relative_to(REPO_ROOT)}")


def export_ios_assets() -> None:
    ios_root = REPO_ROOT / "ccr-ui" / "src-tauri" / "icons" / "ios"
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
        save_png(render_svg(APP_ICON_SVG, size), ios_root / name)


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
