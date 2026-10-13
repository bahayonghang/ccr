# CCR Brand Assets

This directory owns CCR's blue interlocking-loop identity. Two opposed rounded
loops suggest switching between AI CLI configurations. The cobalt mark sits on
a pale blue-white tile, with generous negative space and transparent outer corners.

## Source Files

- `icon-source.png`: canonical, unmodified 1254 × 1254 ImageGen production master.
- `provenance/approved-blue-a.png`: original ImageGen candidate selected by the user.
- `provenance/imagegen-prompts.md`: original and production-edit prompts and hashes.
- `vscode-icon.svg`: simplified loop silhouette in `currentColor` for theme-aware
  VS Code view containers.

The canonical PNG is the production source. Provenance images, root `icon.png`,
task previews, and developer-local files are never generator inputs. Changes to
the colored emblem use ImageGen; deterministic exports only resize, mask, and encode.

## Export Policy

Run `bun run icons:generate` from `ccr-ui/`. The Pillow-only generator reads
`icon-source.png` without Cairo or any platform-specific rasterizer. It keeps the
source image at full canvas size and applies these masks on a 1024-unit canvas:

| Variant | Tile bounds | Corner radius |
| --- | --- | --- |
| App | `(48, 48)` to `(976, 976)` | `224` |
| Display | `(32, 32)` to `(992, 992)` | `236` |

`master.svg`, `app-icon.svg`, and `display-logo.svg` are generated compatibility
wrappers containing the same PNG as a data URI with a matching rounded clip path.
They are self-contained and preserve existing SVG consumer URLs.

The script owns these derived assets:

- `icon.png`
- `ccr-ui/src-tauri/icons/*` PNG, ICO, ICNS, Windows tiles, Android and iOS images
- `ccr-ui/public/icons/*`
- `ccr-ui/src/assets/favicon.svg` and `logo.png`
- `ccr-vscode/icon.svg`, `icon.png`, and `resources/icons/ccr.svg`
- `docs/public/logo.svg`, `favicon.svg`, and `favicon.png`

`bun run icons:ensure` uses this same generator before builds. Commit the source,
generator, compatibility wrappers, and all derived assets together. Repeating the
generation without source changes must produce identical hashes. Validate SVG
XML, theme-aware glyph color, raster sizes/formats, transparent corners, and
16/32 px plus 512/1024 px appearance before delivery.
