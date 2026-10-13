# /// script
# requires-python = ">=3.10"
# dependencies = ["pillow"]
# ///
"""Generate CCR brand assets from the approved ImageGen PNG and theme glyph."""

from __future__ import annotations

import base64
from functools import lru_cache
from pathlib import Path

from PIL import Image, ImageDraw

SCRIPT_DIR = Path(__file__).resolve().parent
REPO_ROOT = SCRIPT_DIR.parents[1]
BRANDING_DIR = REPO_ROOT / "branding"

ICON_SOURCE = BRANDING_DIR / "icon-source.png"
MASTER_SVG = BRANDING_DIR / "master.svg"
APP_ICON_SVG = BRANDING_DIR / "app-icon.svg"
DISPLAY_LOGO_SVG = BRANDING_DIR / "display-logo.svg"
VSCODE_ICON_SVG = BRANDING_DIR / "vscode-icon.svg"

BRAND_BASE = (0xF1, 0xFA, 0xFD, 0xFF)
# x, y, width, height, radius on the shared 1024 canvas.
TILE_VARIANTS = {
    "app": (48, 48, 928, 928, 224),
    "display": (32, 32, 960, 960, 236),
}
RENDER_SCALE = 2
RENDER_SIZE = 1024 * RENDER_SCALE


def ensure_parent(path: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)


def copy_text_asset(source: Path, target: Path) -> None:
    ensure_parent(target)
    target.write_text(source.read_text(encoding="utf-8"), encoding="utf-8")
    print(f"Updated {target.relative_to(REPO_ROOT)}")


@lru_cache(maxsize=2)
def render_brand_master(variant: str) -> Image.Image:
    x, y, width, height, radius = TILE_VARIANTS[variant]
    with Image.open(ICON_SOURCE) as source:
        painted = source.convert("RGBA").resize(
            (RENDER_SIZE, RENDER_SIZE), Image.Resampling.LANCZOS
        )
    tile_mask = Image.new("L", painted.size, 0)
    ImageDraw.Draw(tile_mask).rounded_rectangle(
        tuple(value * RENDER_SCALE for value in (x, y, x + width, y + height)),
        radius=radius * RENDER_SCALE,
        fill=0xFF,
    )
    canvas = Image.new("RGBA", painted.size, (0, 0, 0, 0))
    canvas.paste(painted, mask=tile_mask)
    return canvas


def render_brand_icon(size: int, variant: str = "app") -> Image.Image:
    master = render_brand_master(variant)
    if size == master.width:
        return master.copy()
    icon = master.resize((size, size), Image.Resampling.LANCZOS)
    # Lanczos can leave alpha=1 at 16px corners outside the rounded tile.
    for corner in ((0, 0), (size - 1, 0), (0, size - 1), (size - 1, size - 1)):
        icon.putpixel(corner, (0, 0, 0, 0))
    return icon


def write_brand_svg(target: Path, variant: str) -> None:
    x, y, width, height, radius = TILE_VARIANTS[variant]
    encoded = base64.b64encode(ICON_SOURCE.read_bytes()).decode("ascii")
    svg = f'''<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink"
  viewBox="0 0 1024 1024" role="img" aria-labelledby="title desc">
  <title id="title">CCR blue interlocking loops</title>
  <desc id="desc">Two opposed blue loops on a light tile represent switching configurations.</desc>
  <defs>
    <clipPath id="tile-clip">
      <rect x="{x}" y="{y}" width="{width}" height="{height}" rx="{radius}"/>
    </clipPath>
  </defs>
  <image x="0" y="0" width="1024" height="1024" clip-path="url(#tile-clip)"
    xlink:href="data:image/png;base64,{encoded}"/>
</svg>
'''
    ensure_parent(target)
    target.write_text(svg, encoding="utf-8")
    print(f"Updated {target.relative_to(REPO_ROOT)}")


def render_with_padding(
    size: int,
    padding_ratio: float,
) -> Image.Image:
    canvas = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    inner_size = int(round(size * (1 - padding_ratio * 2)))
    inner = render_brand_icon(inner_size)
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


def save_brand_preview(
    target: Path,
    size: int,
    variant: str = "app",
) -> None:
    save_png(render_brand_icon(size, variant), target)


def save_ico(image: Image.Image, target: Path) -> None:
    ensure_parent(target)
    ico_sizes = [(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (256, 256)]
    frames = [render_brand_icon(width) for width, _ in ico_sizes]
    with target.open("wb") as handle:
        image.save(handle, format="ICO", sizes=ico_sizes, append_images=frames)
    print(f"Generated {target.relative_to(REPO_ROOT)}")


def save_icns(image: Image.Image, target: Path) -> None:
    ensure_parent(target)
    with target.open("wb") as handle:
        image.save(handle, format="ICNS")
    print(f"Generated {target.relative_to(REPO_ROOT)}")


def export_brand_sources() -> None:
    write_brand_svg(MASTER_SVG, "app")
    write_brand_svg(APP_ICON_SVG, "app")
    write_brand_svg(DISPLAY_LOGO_SVG, "display")
    copy_text_asset(APP_ICON_SVG, REPO_ROOT / "ccr-ui" / "src" / "assets" / "favicon.svg")
    copy_text_asset(APP_ICON_SVG, REPO_ROOT / "ccr-ui" / "public" / "icons" / "icon.svg")
    copy_text_asset(DISPLAY_LOGO_SVG, REPO_ROOT / "ccr-ui" / "public" / "icons" / "logo.svg")
    copy_text_asset(DISPLAY_LOGO_SVG, REPO_ROOT / "docs" / "public" / "logo.svg")
    copy_text_asset(APP_ICON_SVG, REPO_ROOT / "docs" / "public" / "favicon.svg")
    copy_text_asset(VSCODE_ICON_SVG, REPO_ROOT / "ccr-vscode" / "resources" / "icons" / "ccr.svg")
    copy_text_asset(APP_ICON_SVG, REPO_ROOT / "ccr-vscode" / "icon.svg")


def export_runtime_assets() -> None:
    save_brand_preview(REPO_ROOT / "icon.png", 512)
    save_brand_preview(REPO_ROOT / "ccr-ui" / "public" / "icons" / "icon.png", 512)
    save_brand_preview(REPO_ROOT / "ccr-ui" / "public" / "icons" / "logo.png", 1024, "display")
    save_brand_preview(REPO_ROOT / "ccr-ui" / "src" / "assets" / "logo.png", 640, "display")
    save_brand_preview(REPO_ROOT / "docs" / "public" / "favicon.png", 256)
    save_brand_preview(REPO_ROOT / "ccr-vscode" / "icon.png", 256)


def export_tauri_bundle_assets() -> None:
    tauri_icons = REPO_ROOT / "ccr-ui" / "src-tauri" / "icons"
    app_1024 = render_brand_icon(1024)

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
        save_png(render_brand_icon(size), tauri_icons / name)

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
        image = render_brand_icon(size)
        save_png(image, android_root / bucket / "ic_launcher.png")
        save_png(image, android_root / bucket / "ic_launcher_round.png")

    for bucket, size in foreground_sizes.items():
        foreground = render_with_padding(size, padding_ratio=0.14)
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
        save_png(render_brand_icon(size), ios_root / name)


def main() -> None:
    required = [ICON_SOURCE, VSCODE_ICON_SVG]
    for path in required:
        if not path.exists():
            raise FileNotFoundError(f"Missing brand source: {path}")

    if "currentColor" not in VSCODE_ICON_SVG.read_text(encoding="utf-8"):
        raise ValueError("VS Code brand glyph must use currentColor")

    export_brand_sources()
    export_runtime_assets()
    export_tauri_bundle_assets()
    export_android_assets()
    export_ios_assets()


if __name__ == "__main__":
    main()
