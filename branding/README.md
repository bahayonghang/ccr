# CCR Brand Assets

This directory is the single source of truth for CCR brand iconography.

## Source Files

- `master.svg`: editable reference master for the rounded-square prism badge system
- `app-icon.svg`: simplified production icon for app shortcuts, favicons, and marketplace tiles
- `display-logo.svg`: richer presentation variant for docs hero, about panels, and branded surfaces
- `vscode-icon.svg`: monochrome monogram variant for VS Code activity/view containers

## Export Policy

Generated artifacts are owned by `ccr-ui/scripts/generate_icons.py`.

The script exports assets for:

- `icon.png`
- `ccr-ui/src-tauri/icons/*`
- `ccr-ui/public/icons/*`
- `ccr-ui/src/assets/favicon.svg`
- `ccr-ui/src/assets/logo.png`
- `ccr-vscode/icon.png`
- `ccr-vscode/resources/icons/ccr.svg`
- `docs/public/logo.svg`
- `docs/public/favicon.svg`
- `docs/public/favicon.png`

## Design Notes

- The root `icon.png` remains a visual reference only.
- The production brand system prefers a rounded-square prism badge plus a two-stroke CCR monogram.
- Small-size legibility takes priority over decorative rendering.
