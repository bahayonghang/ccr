# CLAUDE.md

This file provides guidance when working with `ccr-vscode`.

## Project Overview

CCR VSCode Extension — a sidebar extension for managing Claude and Codex runtime/profile state directly from VSCode. It reads CCR's TOML registry and profile files, and delegates runtime switching to the `ccr` CLI.

## Data Flow

1. **Read path:** `ccrPaths` resolves `~/.ccr/` (or `$CCR_ROOT`) → `tomlReader` parses registry + profiles → providers render UI.
2. **Write path (edit):** `tomlReader.writeProfileField()` edits `profiles.toml` directly while preserving top-level metadata.
3. **Write path (switch):** `ccrCli.execProfileSwitch(platform, name)` shells out to `ccr claude profile switch <name>` or `ccr codex profile switch <name>`.
4. **Refresh:** file watchers trigger debounced reloads.

## Runtime Model

- No global `current_platform` routing truth.
- Per-platform `current_profile` is the routing truth.
- Status UI should reflect Claude Runtime + Codex Runtime, not one global active platform.

## Config Layout

```text
~/.ccr/
├── config.toml                      # registry metadata; old default/current_platform may still appear in older files
└── platforms/
    ├── claude/profiles.toml
    ├── codex/profiles.toml
    └── ...
```
