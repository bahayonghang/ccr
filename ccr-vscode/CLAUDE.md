# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

CCR VSCode Extension — a sidebar extension for managing AI CLI tool configurations (Claude, Codex, Gemini, Qwen, Droid) directly from VSCode. It reads/writes CCR's TOML config files and delegates switching operations to the `ccr` CLI binary.

## Build & Development Commands

```bash
# Using Just (recommended)
just install    # npm install
just build      # esbuild bundle + vsce package → ccr-vscode.vsix (copied to ../outputs/ccr-vscode/)
just watch      # esbuild watch mode with sourcemaps
just lint       # TypeScript type check (tsc --noEmit)
just test       # Node built-in test runner (node --import tsx --test)
just ci         # Full pipeline: install → lint → test → build
just clean      # Remove dist/ and node_modules/

# Using npm directly
npm run build     # esbuild bundle only (no .vsix)
npm run watch     # Watch mode
npm run package   # esbuild --minify + vsce package
npm run lint      # tsc --noEmit
npm test          # node --import tsx --test src/**/*.test.ts
```

**Install .vsix locally:** `code --install-extension ccr-vscode.vsix`

**Debug in VSCode:** Open `ccr-vscode/` as workspace root, press F5 (uses default Extension Development Host).

## Architecture

Three-layer design: **Providers** (UI) → **Services** (data/IO) → **Models** (types).

```
src/
├── extension.ts                      # Entry point: registers providers, commands, watcher
├── models/
│   └── types.ts                      # TS interfaces mirroring Rust structs (UnifiedConfig, CcsConfig, ProfileConfig)
├── providers/
│   ├── profileTreeProvider.ts        # TreeDataProvider: Platform → Profile two-level hierarchy
│   ├── profileEditorPanel.ts         # WebviewPanel: section-card visual editor with auto-save on blur
│   └── statusBarProvider.ts          # StatusBarItem: current platform/profile indicator
└── services/
    ├── ccrCli.ts                     # CLI interaction: spawns `ccr` binary for switch operations
    ├── ccrPaths.ts                   # Path resolution: mirrors Rust PlatformPaths::get_ccr_root()
    ├── ccrWatcher.ts                 # FileSystemWatcher: monitors config.toml + profiles.toml (300ms debounce)
    ├── tomlReader.ts                 # TOML read/write via smol-toml: registry + profiles parsing
    └── ccrPaths.test.ts              # Unit tests for paths and TOML operations
```

### Data Flow

1. **Read path:** `ccrPaths` resolves `~/.ccr/` (or `$CCR_ROOT`) → `tomlReader` parses TOML → providers render UI
2. **Write path (edit):** `tomlReader.writeProfileField()` directly writes profiles.toml (preserving top-level fields)
3. **Write path (switch):** `ccrCli.execProfileSwitch()` spawns `ccr switch <name>` (atomic writes + file locking handled by Rust CLI)
4. **Refresh:** `ccrWatcher` detects file changes → fires debounced callbacks → providers re-read and re-render

### Key Design Decisions

- **Read-only TOML for profiles, CLI for switching:** Profile field edits write TOML directly (simple field updates). Profile *switching* delegates to `ccr` CLI because it requires atomic writes, file locking, and audit trail — all handled by the Rust binary.
- **Synchronous file reads:** `tomlReader` uses `fs.readFileSync` since config files are small and reads happen on UI refresh events.
- **CcsConfig flat structure:** TOML profiles use a flat layout where top-level keys (`default_config`, `current_config`, `settings`) are filtered out; remaining object sections are profile entries.
- **WebviewPanel singleton pattern:** `ProfileEditorPanel` maintains an `activePanels` map keyed by `"platform/profile"` to reuse existing panels.

## Config File Layout

```
~/.ccr/                              # CCR_ROOT (overridable via $CCR_ROOT env var)
├── config.toml                      # UnifiedConfig: default_platform, current_platform, [platform] entries
└── platforms/
    ├── claude/profiles.toml         # CcsConfig: default_config, current_config, [profile] sections
    ├── codex/profiles.toml
    └── ...
```

## Code Style

- **TypeScript strict mode**, target ES2022, bundled with esbuild (CJS output for VSCode)
- **Comments:** Chinese for internal logic, English for public API docs (consistent with parent project)
- **TOML field names:** snake_case in files, camelCase in TypeScript interfaces (`ProfileEditorPanel.saveField()` maps between them)
- **No runtime dependencies except `smol-toml`** — all VSCode API interactions use built-in `vscode` module
- **Tests:** Node built-in test runner (`node:test` + `node:assert/strict`), run via `tsx` for TypeScript support. Tests use temp directories with `$CCR_ROOT` override and dynamic imports for module re-loading.

## VSCode Extension Points

- **Activity Bar view container:** `ccr` with TreeView `ccr-profiles`
- **Commands:** `ccr.refreshProfiles`, `ccr.switchProfile`, `ccr.editProfileVisual`, `ccr.editProfileField`, `ccr.toggleProfileEnabled`, `ccr.openProfilesFile`
- **Activation:** `onStartupFinished` (non-blocking CLI availability check)

## Type Alignment with Rust

Types in `models/types.ts` mirror Rust structs — see the mapping comment at the top of that file. When modifying Rust models in the parent workspace, update the TS interfaces accordingly:

| TypeScript | Rust Source |
|---|---|
| `UnifiedConfig` | `crates/ccr/src/managers/platform_config.rs:63` |
| `PlatformConfigEntry` | `crates/ccr/src/managers/platform_config.rs:21` |
| `CcsConfig` | `crates/ccr/src/managers/config/ccs_config.rs:13` |
| `ProfileConfig` | `crates/ccr/src/models/platform.rs:143` |
