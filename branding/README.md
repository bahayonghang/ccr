# CCR Brand Assets

This directory is the single source of truth for CCR brand iconography.

## Brand System

The Dual Runtime Router mark encodes CCR without a text wordmark:

- the outer cream `C` is the shared control plane for CLI, TUI, and CCR UI;
- the clay upper route represents Claude Runtime profile, config, and auth state;
- the sage lower route represents Codex Runtime profile, auth, and sync state;
- the CLI chevron shows both runtime paths converging on an explicit execution route.

The production palette is:

| Role | Hex |
| --- | --- |
| Dark base | `#17120F` |
| Dark surface | `#2A221E` |
| Warm cream | `#F3EADF` |
| Claude clay | `#E79A77` |
| Codex sage | `#7CAB82` |

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
