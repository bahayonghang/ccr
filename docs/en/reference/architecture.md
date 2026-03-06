# Architecture

> Canonical guide for the current workspace layout. CCR now uses a Rust 2024 workspace with core crates under `crates/`, while `ccr-ui`, `docs`, `scripts`, and `examples` remain at the repository root.

## Overview

- **Core CLI crate**: `crates/ccr`
- **Database crate**: `crates/ccr-db`
- **Shared types crate**: `crates/ccr-types`
- **UI app root**: `ccr-ui`
- **Root utilities**: `docs/`, `scripts/`, `examples/`
- **Collected artifacts**: optional root `outputs/`

## Workspace Layout

```text
ccr/
├── Cargo.toml                # workspace manifest + shared dependencies
├── crates/
│   ├── ccr/                  # installable CLI crate + shared runtime logic
│   │   ├── src/              # cli / commands / services / managers / sync / web / tui
│   │   └── tests/            # integration tests for the CLI crate
│   ├── ccr-db/               # database-facing services and models
│   └── ccr-types/            # shared types reused across crates and desktop shell
├── ccr-ui/
│   ├── src/                  # Vue 3 application
│   ├── src-tauri/            # Tauri desktop shell
│   └── dist/                 # generated frontend assets after `ccr-ui` build
├── docs/                     # VitePress docs (zh/en)
├── scripts/                  # repository automation and maintenance helpers
├── examples/                 # sample configs and workflows
└── outputs/                  # collected/generated artifacts (optional)
```

## Layering

```text
CLI / Web API / TUI / Desktop shell
                ↓
         Commands / UI bridge
                ↓
          Services (orchestration)
                ↓
          Managers (persistence)
                ↓
         Core / Utils / Models
```

- **CLI entrypoints** live in `crates/ccr/src/cli/` and `crates/ccr/src/main.rs`.
- **Command handlers** live in `crates/ccr/src/commands/`.
- **Service orchestration** lives in `crates/ccr/src/services/`.
- **Persistence and data access** live in `crates/ccr/src/managers/`.
- **Platform implementations** live in `crates/ccr/src/platforms/`.
- **Infrastructure helpers** live in `crates/ccr/src/core/` and `crates/ccr/src/utils/`.
- **Desktop integration** in `ccr-ui/src-tauri` depends directly on `crates/ccr`, `crates/ccr-db`, and `crates/ccr-types`.

## Dependency Direction

- Interfaces depend on Commands.
- Commands depend on Services.
- Services depend on Managers.
- Managers depend on Core/Utils.
- Shared models/platform traits can be reused upward, but UI code does not own the business logic.

## Key Flows

### Profile switching

1. `crates/ccr/src/cli/` parses a command or shorthand `ccr <name>`.
2. `ConfigService` loads `~/.ccr/config.toml` and `platforms/<name>/profiles.toml`.
3. `SettingsService` acquires locks, creates backups, and atomically writes the target `settings.json`.
4. `HistoryService` records masked diffs.
5. `TempOverrideManager` applies temporary token/base_url/model overrides when requested.

### WebDAV sync

1. `sync config` stores connection data.
2. `sync folder ...` registers and enables sync targets.
3. `SyncService` handles push/pull/all recursion while filtering backups, history, lock files, and UI cache.

### CCR UI bootstrap

1. `UiService` probes local `./ccr-ui` first.
2. It falls back to `~/.ccr/ccr-ui` if needed.
3. It can finally prompt for a GitHub download.

## Reliability & Quality

- File locks, in-process mutexes, and atomic writes protect config files.
- Destructive operations create backups first.
- Logs are controlled via `CCR_LOG_LEVEL` and stored under `~/.ccr/logs/`.
- CLI integration tests live under `crates/ccr/tests/`.
- New commands and platform work should extend `crates/ccr/src/`, not the workspace root.

## Extension Paths

- Add commands under `crates/ccr/src/commands/<domain>/`.
- Wire CLI definitions in `crates/ccr/src/cli/definitions.rs` and routing in `crates/ccr/src/cli/dispatch.rs`.
- Add platforms under `crates/ccr/src/platforms/` and register them in `crates/ccr/src/platforms/mod.rs`.

## See Also

- [Quick Start](/en/guide/quick-start)
- [Command Reference](/en/reference/commands/)
- [Migration Guide](/en/reference/migration)
