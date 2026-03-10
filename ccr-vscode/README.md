# CCR VSCode Extension

Manage AI CLI tool configurations (Claude, Codex, Gemini, Qwen, iFlow, Droid) directly from the VSCode sidebar.

## Features

- **Profile Viewer** — TreeView showing all platforms and profiles with current-profile indicator
- **Profile Switcher** — Switch profiles via TreeView click or QuickPick (uses `ccr` CLI)
- **Profile Editor** — Edit profile fields directly from VSCode
- **Status Bar** — Current platform and profile shown in status bar
- **Auto Refresh** — File watcher detects external config changes and updates UI

## Requirements

- [CCR](https://github.com/bahayonghang/ccr) CLI installed for profile switching
- Config files at `~/.ccr/` (or `$CCR_ROOT`)

## Usage

1. Install the `.vsix` file: `code --install-extension ccr-vscode-<version>.vsix`
2. Open the CCR sidebar panel in the Activity Bar
3. View, switch, and edit profiles

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
