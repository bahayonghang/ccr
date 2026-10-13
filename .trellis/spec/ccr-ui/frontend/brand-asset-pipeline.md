# Brand Asset Pipeline Contract

> Source ownership, cross-platform rendering, generated outputs, and verification for CCR brand icons.

## Scenario: Updating CCR Brand Assets

### 1. Scope / Trigger

- Trigger: changing `branding/icon-source.png`, the theme-aware glyph under `branding/`, the renderer in `ccr-ui/scripts/generate_icons.py`, or a generated brand asset owned by that script.
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

Production input set:

```text
branding/icon-source.png
branding/vscode-icon.svg
```

Generated SVG compatibility carriers: `branding/master.svg`, `branding/app-icon.svg`, `branding/display-logo.svg`. ImageGen provenance: `branding/provenance/approved-blue-a.png` and `branding/provenance/imagegen-prompts.md` (original generation, production preparation, source hashes).

Renderer signatures: `render_brand_master(variant: str)`, `render_brand_icon(size: int, variant: str = 'app')`, `write_brand_svg(target: Path, variant: str)`.

### 3. Contracts

- `branding/icon-source.png` is the version-controlled, full-canvas ImageGen production master. Reference folders, provenance images, task previews, root `icon.png`, and generated PNG/ICO/ICNS outputs are never generator inputs.
- `app-icon.svg` is an embedded-PNG compatibility carrier. The app variant owns favicons, marketplace images, Tauri raster assets, Windows tiles, Android launchers, and iOS AppIcons.
- `display-logo.svg` is the display-variant carrier for CCR UI presentation surfaces and the docs home logo.
- `vscode-icon.svg` must use `currentColor`; fixed brand colors are invalid for Activity Bar/view container rendering.
- Pillow is the deterministic raster exporter; Cairo is not required. SVG carriers embed the same canonical PNG bytes and apply matching clip paths. Do not hand-redraw the colored symbol as a substitute for the approved ImageGen asset.
- The blue double-loop symbol uses a cool light base. Full-canvas image placement preserves the approved symbol proportions: scale the source to the full 1024 canvas, then mask the exterior; do not shrink it again into the tile rectangle.
- Normalized app mask: `(x=48, y=48, width=928, height=928, radius=224)`. Display mask: `(32, 32, 960, 960, 236)`. Desktop raster outer corners remain transparent with smooth antialiasing; the production master itself is opaque to avoid generated alpha-fringe artifacts.
- The generator must preserve existing filenames, dimensions, RGBA transparent edges, ICO sizes, ICNS readability, and Android/iOS output coverage.
- Lanczos can leave alpha=1 at a 16 px outer corner. Raster export explicitly clears all four corner pixels; ICO uses the exact prepared size frames instead of relying on Pillow to resize the large master again. Validate each embedded ICO frame, not just the default largest image.
- Tauri `bundle.icon` includes both `icon.ico` and `icon.icns` alongside the existing PNG targets.
- A second generator run without source changes must produce identical content hashes.

### 4. Validation & Error Matrix

| Condition | Required behavior |
| --- | --- |
| Canonical PNG or required currentColor glyph is missing | `generate_icons.py` raises `FileNotFoundError` before export |
| Canonical PNG is not decodable | Fail generation; do not reuse a previous generated output |
| A generated SVG carrier is missing | Recreate it from the canonical PNG and variant mask |
| A required committed icon is missing during prebuild | `icons:ensure` regenerates with `uv`, or fails with the missing path list |
| VS Code container glyph lacks `currentColor` | Fail asset validation before packaging |
| Second generation changes hashes | Treat as generator drift and stop delivery |

### 5. Good/Base/Bad Cases

- Good: update the canonical PNG and provenance, align the currentColor glyph, run the generator, then commit all owned outputs together.
- Base: rebuild with no source changes; `icons:ensure` leaves the same deterministic asset set.
- Bad: manually replace only `ccr-ui/src-tauri/icons/icon.png`; the next prebuild overwrites it and other surfaces retain the old brand.
- Bad: use root `icon.png` or a home-directory ImageGen file as input; output then depends on prior workspace state or one developer's filesystem.
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
- verify SVG carriers embed the canonical source bytes and match the app/display clipping geometry;
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
ICON_SOURCE = REPO_ROOT / "branding" / "icon-source.png"

# Load the version-controlled source, never a generated output.
with Image.open(ICON_SOURCE) as image:
    source = image.convert("RGBA")
# Apply the documented full-canvas variant mask in render_brand_master.
app_icon = render_brand_icon(512, variant="app")
```

The master is a real ImageGen raster asset under version control. Masks and SVG carriers are deterministic export geometry, not a second hand-drawn brand implementation.
