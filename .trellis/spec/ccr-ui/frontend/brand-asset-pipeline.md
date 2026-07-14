# Brand Asset Pipeline Contract

> Source ownership, cross-platform rendering, generated outputs, and verification for CCR brand icons.

## Scenario: Updating CCR Brand Assets

### 1. Scope / Trigger

- Trigger: changing any production SVG under `branding/`, the renderer in `ccr-ui/scripts/generate_icons.py`, or a generated brand asset owned by that script.
- Applies to CCR UI/Web, Tauri desktop/mobile bundles, docs, the VS Code Marketplace icon, and the VS Code Activity Bar glyph.
- Does not apply to feature icons managed by Iconify/Lucide or third-party platform logos.

### 2. Signatures

Primary generation command:

```powershell
cd ccr-ui
bun run icons:generate
```

Build-time existence guard:

```powershell
cd ccr-ui
bun run icons:ensure
```

Production source set:

```text
branding/master.svg
branding/app-icon.svg
branding/display-logo.svg
branding/vscode-icon.svg
```

### 3. Contracts

- `branding/*.svg` is the only production source of truth. Reference folders and generated PNG/ICO/ICNS files are never generator inputs.
- `app-icon.svg` owns favicons, marketplace images, Tauri raster assets, Windows tiles, Android launchers, and iOS AppIcons.
- `display-logo.svg` owns branded presentation surfaces such as the CCR UI titlebar and docs home logo.
- `vscode-icon.svg` must use `currentColor`; fixed brand colors are invalid for Activity Bar/view container rendering.
- CairoSVG is the primary rasterizer when the host provides Cairo. The Pillow renderer must remain behaviorally aligned because Windows hosts may install the Python package but lack `cairo-2`/`libcairo-2` DLLs.
- The generator must preserve existing filenames, dimensions, RGBA transparent edges, ICO sizes, ICNS readability, and Android/iOS output coverage.
- A second generator run without source changes must produce identical content hashes.

### 4. Validation & Error Matrix

| Condition | Required behavior |
| --- | --- |
| A required `branding/*.svg` file is missing | `generate_icons.py` raises `FileNotFoundError` before export |
| CairoSVG import fails because Cairo DLLs are absent | Use the Pillow native renderer for app/display raster outputs |
| A non-app/display SVG requires rasterization without Cairo | Raise `RuntimeError`; do not silently reuse another image |
| A required committed icon is missing during prebuild | `icons:ensure` regenerates with `uv`, or fails with the missing path list |
| VS Code container glyph lacks `currentColor` | Fail asset validation before packaging |
| Second generation changes hashes | Treat as generator drift and stop delivery |

### 5. Good/Base/Bad Cases

- Good: update the four brand SVGs, update the native fallback, run the generator once, then commit all owned outputs together.
- Base: rebuild with no source changes; `icons:ensure` leaves the same deterministic asset set.
- Bad: manually replace only `ccr-ui/src-tauri/icons/icon.png`; the next prebuild overwrites it and other surfaces retain the old brand.
- Bad: use root `icon.png` as fallback input while the same generator overwrites it; output then depends on prior workspace state.
- Bad: copy the colored app SVG into `ccr-vscode/resources/icons/ccr.svg`; it loses theme contrast.

### 6. Tests Required

For every brand-source or renderer change:

```powershell
cd ccr-ui
bun run icons:generate
bun run icons:ensure
bun run build

cd ..
just frontend-check
git diff --check
```

Assertions:

- parse every generated SVG as XML;
- assert the VS Code container glyph contains `currentColor`;
- open every PNG/ICO/ICNS with Pillow and verify expected sizes/formats;
- assert raster outputs have transparent outer corners where required;
- run generation twice and compare hashes of every owned source/output;
- visually inspect at least 16/32 px and 512/1024 px samples.

### 7. Wrong vs Correct

#### Wrong

```python
REFERENCE_ICON_PNG = REPO_ROOT / "icon.png"

def fallback() -> Image.Image:
    return Image.open(REFERENCE_ICON_PNG)
```

#### Correct

```python
def render_svg(svg_path: Path, size: int) -> Image.Image:
    if cairosvg_svg2png is not None:
        return render_with_cairo(svg_path, size)
    if svg_path in {APP_ICON_SVG, DISPLAY_LOGO_SVG}:
        return render_native_brand_icon(size, variant=variant_for(svg_path))
    raise RuntimeError(f"SVG rasterizer unavailable for {svg_path}")
```

The fallback is defined from version-controlled geometry and palette constants, not from a generated output or developer-local reference directory.
