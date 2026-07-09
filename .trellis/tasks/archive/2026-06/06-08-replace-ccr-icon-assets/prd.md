# Replace CCR Icon Assets

## Goal

Replace the old CCR brand icon assets with the newly supplied CCR icon so the desktop app, web UI, docs, and VS Code extension use the same refreshed identity.

## Requirements

- Replace repo-owned CCR brand source assets under `branding/`.
- Regenerate committed runtime icon assets that are derived from the brand sources.
- Cover web, docs, VS Code extension, and Tauri desktop icon outputs.
- Leave reference repositories under `ccr-ui/ref/` untouched.
- Preserve existing build/runtime wiring; this is an asset replacement, not a UI or behavior change.

## Acceptance Criteria

- [x] `branding/*.svg` reflect the new CCR icon style.
- [x] `ccr-ui/scripts/generate_icons.py` completes successfully.
- [x] Generated PNG, ICO, ICNS, docs, web, VS Code, and Tauri icon assets are updated.
- [x] Existing icon integrity checks pass.
- [x] Existing unrelated dirty files are not reverted or committed.

## Definition of Done

- Relevant icon generation/check commands have been run.
- Changed files are limited to task documentation and repo-owned icon assets unless script adjustments are necessary.
- No new product behavior is introduced.

## Technical Approach

Use the existing `ccr-ui/scripts/generate_icons.py` pipeline. Update `branding/master.svg`, `branding/app-icon.svg`, `branding/display-logo.svg`, and `branding/vscode-icon.svg` to match the supplied icon, then run the existing generator to refresh downstream assets.

## Out of Scope

- Redesigning application pages or layouts.
- Editing third-party/reference assets under `ccr-ui/ref/`.
- Changing package metadata, publisher fields, versions, or extension commands.

## Technical Notes

- Existing generator writes assets for root `icon.png`, `ccr-ui/public`, `ccr-ui/src/assets`, `docs/public`, `ccr-vscode`, and `ccr-ui/src-tauri/icons`.
- `ccr-vscode/resources/icons/ccr.svg` is copied from `branding/vscode-icon.svg` and should remain a `currentColor` icon.
- Initial dirty tree already had `ccr-ui/.gitignore`; this task should preserve it.
- On this Windows host, `cairosvg` imports fail without the native Cairo DLL, so the Pillow fallback in `generate_icons.py` must stay aligned with the SVG source.
