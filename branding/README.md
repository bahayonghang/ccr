# CCR Brand Assets

This directory is the single source of truth for CCR brand iconography.

## Brand System

The merge mark shows CCR's configuration route without a text wordmark:

- two white routes are the runtime configurations entering from below;
- they meet at one white point, the configuration that is currently active;
- the field is a saturated blue squircle.

The production palette is:

| Role | Hex |
| --- | --- |
| Blue field | `#1A73E8` |
| Route and node | `#FFFFFF` |

Shared geometry on a `0 0 1024 1024` canvas:

- left route: `M 156 848 C 200 640 370 420 512 292`
- right route: `M 868 848 C 824 640 654 420 512 292`
- stroke width `112`, round caps
- node: center `(512, 228)`, radius `72`

`ccr-ui/scripts/generate_icons.py` draws this same geometry when Cairo is unavailable.

## Source Files

- `master.svg`: editable reference master for the complete mark
- `app-icon.svg`: production icon optimized for shortcuts, favicons, and marketplace tiles
- `display-logo.svg`: presentation variant for docs and branded application surfaces
- `vscode-icon.svg`: monochrome `currentColor` glyph for VS Code view containers

## Export Policy

Generated artifacts are owned by `ccr-ui/scripts/generate_icons.py`.

The script exports assets for:

- `icon.png`
- `ccr-ui/src-tauri/icons/*`
- `ccr-ui/public/icons/*`
- `ccr-ui/src/assets/favicon.svg`
- `ccr-ui/src/assets/logo.png`
- `ccr-vscode/icon.svg`
- `ccr-vscode/icon.png`
- `ccr-vscode/resources/icons/ccr.svg`
- `docs/public/logo.svg`
- `docs/public/favicon.svg`
- `docs/public/favicon.png`

The generator rasterizes SVGs with CairoSVG when the host provides Cairo. Its
Pillow renderer is the required fallback on Windows hosts without Cairo DLLs;
both paths must describe the same brand. Generated outputs must never be used
as fallback inputs.

The root `icon.png` remains a generated visual reference only.
