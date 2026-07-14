# Crate Map

This page lists every workspace member in the root `Cargo.toml`. The documentation audit compares this inventory with the workspace automatically.

## Entrypoints And Interaction

### `crates/ccr`

- Builds the installable `ccr` binary.
- Assembles the `ccr-cli` dispatcher and optional `ccr-tui` launcher.
- Keeps limited compatibility re-exports; it is no longer the owner of every domain.

### `crates/ccr-cli`

- `src/cli/definitions.rs`: top-level Clap `Commands`.
- `src/cli/subcommands/`: nested Claude, Codex, OpenCode, platform, and sync commands.
- `src/cli/dispatch.rs`: routing, TUI launchers, and legacy-path handling.
- `src/commands/`: user-visible handlers; `services/`, `managers/`, and `platforms/` own CLI-specific orchestration.

### `crates/ccr-tui`

- Ratatui rendering, interaction state, and platform tabs.
- Reuses `ccr-cli`, `ccr-codex`, and `ccr-usage` instead of duplicating their domains.

## Shared Foundation And Contracts

### `crates/ccr-core`

- Shared errors, paths, locks, atomic writes, logging, HTTP, and foundation models.
- Provides safe writes and common infrastructure to upper layers.

### `crates/ccr-types`

- Shared serde types for Claude settings, Codex login state, monitoring, and log payloads.
- Preserves old data through aliases, flattened unknown fields, and compatibility shapes.

## Configuration And Persistence

### `crates/ccr-config`

- Platform enum, profile/settings contracts, registry, and format conversions.
- Unified configuration boundary for Claude, Codex, Antigravity, Droid, and the Qwen stub.

### `crates/ccr-store`

- CLI history and session SQLite storage.
- Session indexing, search, statistics, and pruning queries.

### `crates/ccr-db`

- Desktop SQLite connections, migrations, repositories, and transactions.
- Data layer for check-in, logs, and other desktop services.

## Domain Crates

### `crates/ccr-codex`

- Codex auth snapshots, profile/runtime, quota, usage, and session domain.

### `crates/ccr-sync`

- WebDAV configuration, folder registry, push/pull/status, and batch-sync foundation.

### `crates/ccr-skills`

- Skill sources, inventory, installs, caches, builtin prompts, and MCP presets.

### `crates/ccr-checkin`

- Check-in business facade combining providers, accounts, balances, records, and execution services.
- Reuses `ccr-db` for persistence and `ccr-types` for shared contracts.

### `crates/ccr-usage`

- Reads llmusage SQLite/CLI capabilities and exposes stable read-only projections.
- The `ts` feature exports TypeScript types for the Tauri/Vue boundary.
- Does not parse raw transcripts directly or own upstream schema migrations.

## Frontend Consumer

`ccr-ui/src-tauri` directly depends on the workspace domain crates except `ccr-tui` and exposes invoke handlers through `commands/handler_registry.rs`. Vue consumes them through `src/api/domains/*`.

## Test Entry Points

- Crate unit tests live beside modules or under each crate's `tests/`.
- CLI integration tests live under capability directories such as `crates/ccr/tests/commands/`.
- TUI tests belong to `ccr-tui`.
- Usage projection and TypeScript binding tests belong to `ccr-usage`.
- Tauri integration tests live under `ccr-ui/src-tauri/tests/`.

## Related Pages

- [Architecture](../architecture)
- [Runtime Flows](./runtime-flows)
