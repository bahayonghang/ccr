# Icon Integration Inventory

Date: 2026-07-14

## Input Package

`ref/ccr-icon-redesign` contains:

- production candidates: `sources/master.svg`, `app-icon.svg`, `display-logo.svg`, `vscode-icon.svg`;
- an extra `glyph.svg` with no current repository ownership slot;
- app PNG exports at 16, 24, 32, 48, 64, 128, 256, 512 and 1024 px;
- ICO, ICNS and Tauri top-level/Windows square assets;
- `preview/ccr-icon-preview.png` and a standalone reference generator.

All inspected PNG exports are RGBA, have alpha range `(0, 255)`, and have a transparent `(0, 0, 0, 0)` top-left pixel. The four production SVG hashes differ from the current `branding/` files.

## Current Repository Ownership

The repository already has a source/generator model:

- `branding/README.md` declares `branding/*.svg` as the source of truth.
- `ccr-ui/package.json` exposes `icons:generate` and `icons:ensure`; prebuild hooks call the ensure command.
- `ccr-ui/scripts/generate_icons.py:272-397` owns Web, docs, VS Code, Tauri, Android and iOS outputs.
- `ccr-ui/src-tauri/tauri.conf.json` consumes the stable `icons/...` paths.
- `ccr-vscode/package.json` consumes `icon.png` for Marketplace packaging and `resources/icons/ccr.svg` for the Activity Bar/view container.
- `ccr-ui/src/components/layout/Titlebar.vue:255` consumes `/icons/logo.svg`.
- `ccr-ui/index.html:5` consumes `src/assets/favicon.svg`.
- `docs/index.md:9` and `docs/en/index.md:9` consume `/logo.svg`.

The 2026-06 archived task `06-08-replace-ccr-icon-assets` established the same source/generator boundary and explicitly kept reference repositories outside production changes.

## Generator Gap

The current SVG sources and native renderer describe the previous circuit-link/prism identity:

- old colors are hard-coded at `ccr-ui/scripts/generate_icons.py:30-34`;
- the old Pillow geometry occupies `:114-188`;
- `REFERENCE_ICON_PNG` points to root `icon.png` at `:23`;
- `load_reference_icon()` reads that output at `:190-205`;
- `export_runtime_assets()` rewrites root `icon.png` at `:286`.

This creates both a stale-brand fallback and an input/output cycle.

## Windows Renderer Evidence

Read-only import check:

```powershell
uv run --with cairosvg python -c "import cairosvg; print(cairosvg.__version__)"
```

Result: Python packages installed, but import failed because no `cairo-2`, `cairo`, `libcairo-2`, `libcairo.so.2`, `libcairo.2.dylib`, or `libcairo-2.dll` could be loaded. Therefore the native fallback is required on this Windows host and must be migrated with the SVG sources.

## Scope Decision

Use one task, not a parent/child tree. Every visible surface is a deterministic output of one generator, so independent child execution would increase partial-update risk without creating a useful verification boundary.

Promote only the four source files named by the reference README. Keep `ref/` as local evidence and regenerate every existing repository-owned output, including Android/iOS assets absent from the supplied export pack.
