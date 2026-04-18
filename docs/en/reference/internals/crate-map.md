# Crate Map

This page expands the `architecture` page by describing the current responsibilities at the crate and module level.

## `crates/ccr`

### Entry and dispatch

- `main.rs`: process entrypoint, logger setup, top-level error handling
- `cli/definitions.rs`: CLI structure and the `Commands` enum
- `cli/dispatch.rs`: command routing, no-subcommand behavior, and paths such as `ccr ui`, `sync`, and `codex`

### User-facing command layer

- `commands/platform/`: platform registry and platform switching
- `commands/profile/`: profile lifecycle
- `commands/lifecycle/`: init, clear, clean, optimize, validate
- `commands/data/`: history, export, import, stats, budget, pricing
- `commands/codex/`: Codex-specific commands, especially auth plus env/quota
- `commands/sessions_cmd.rs`, `skills_cmd.rs`, `prompts_cmd.rs`: sessions, skills, and prompt management

### Orchestration layer

- `services/config_service.rs`: CRUD, enable/disable, import/export for config sets
- `services/settings_service.rs`: apply settings, back up settings, restore settings, list backups
- `services/codex_auth_service.rs`: Codex multi-account auth, backups, switching, import/export
- `services/sync_service.rs`: WebDAV sync execution
- `services/ui_service.rs`: detection, update, download, and launch behavior for `ccr ui`

### Persistence and configuration layer

- `managers/config/` and `platform_config.rs`: registry and unified config persistence
- `managers/settings.rs`: settings file read/write
- `managers/history.rs`: operation history
- `managers/pricing_manager.rs`, `budget_manager.rs`, `cost_tracker.rs`: cost and budget state
- `managers/skills_manager.rs` plus `services/skills_service.rs`: skill sources, installs, inventory, and caches
- `managers/mcp_preset_manager.rs`: MCP preset installation and cross-platform sync

### Sessions and sync

- `sessions/`: session parsing and index models
- `storage/session_store.rs`: local session storage and queries
- `sync/`: WebDAV config, folder registration, batch push/pull/status

### Shared infrastructure

- `platforms/`: platform implementations for Claude, Codex, Gemini, Droid, and Qwen
- `core/`: errors, locks, atomic writes, logging, HTTP helpers, and related foundations
- `utils/`: masking, validation, and shared format helpers

## `crates/ccr-db`

### Database entry

- `database/mod.rs`: database path resolution, global pool, migration startup, transaction wrapper
- `database/repositories/`: repository layer
- `database/schema.rs` and `migrations.rs`: schema and data migration

### Business domains

- `models/checkin/`: check-in domain models
- `managers/checkin/`: providers, accounts, records, balances, exports, WAF cookies
- `services/checkin_service.rs`: check-in execution and query flow
- `services/usage_import_service.rs`: usage import from session files
- `services/log_persistence.rs`: log persistence

## `crates/ccr-types`

### Current public surface

- `ClaudeSettings` and related nested types for settings, hooks, MCP, slash commands, agents, and plugins
- `LoginState` for Codex auth state
- `MonitoringEntry`, `FrontendLogInput`, and `MonitoringFeedQuery` for monitoring data

### Design focus

- every type is shaped around serde compatibility
- older field names and input layouts stay accepted
- unknown fields are preserved through `flatten`/`other`

## Test Layout

- `crates/ccr/tests/commands.rs`
- `crates/ccr/tests/managers.rs`
- `crates/ccr/tests/platforms.rs`
- `crates/ccr/tests/workflows.rs`

Those files fan out into capability-oriented subdirectories instead of keeping one flat pile of integration tests.
