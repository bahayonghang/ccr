# CCR VSCode Extension

Manage AI CLI tool configurations (Claude, Codex, Gemini, Qwen, Droid) directly from the VSCode sidebar.

## Features

- **Profile Viewer** — TreeView showing all platforms and profiles with current-profile indicator
- **Profile Switcher** — Switch profiles via TreeView click or QuickPick (uses `ccr` CLI)
- **Profile Editor** — Control-panel style editor with copy actions for `base_url` and `auth_token`
- **Optional Models** — Leave `model` overrides blank to fall back to platform defaults
- **Status Bar** — Show a pinned platform, follow the current platform, or hide the item entirely
- **Auto Refresh** — File watcher detects external config changes and updates UI

## Requirements

- [CCR](https://github.com/bahayonghang/ccr) CLI installed for profile switching
- Config files at `~/.ccr/` (or `$CCR_ROOT`)

## Usage

1. Install the `.vsix` file: `code --install-extension ccr-vscode-<version>.vsix`
2. Open the CCR sidebar panel in the Activity Bar
3. View, switch, and edit profiles

## Configuration

- `ccr.confirmBeforeSwitch` — show a confirmation dialog before switching profiles
- `ccr.statusBar.mode` — `pinned`, `current`, or `hidden`
- `ccr.statusBar.platform` — platform name used when the status bar is pinned

Use the command palette entry `CCR: Select Status Bar Platform` to pin the status bar without editing settings manually.

## Development

```bash
just install   # Install dependencies
just build     # Build extension + create .vsix package
just watch     # Watch mode (auto-recompile)
just lint      # TypeScript type check
just test      # Run tests
just ci        # Full pipeline: install -> lint -> test -> build
just clean     # Clean build artifacts
```
