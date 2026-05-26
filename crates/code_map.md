# crates/ code map

Navigation map for `crates/**`. Behavioral rules stay in `./AGENTS.md`; use this file to route Rust workspace edits before broad grep.

## Start here

- Workspace membership and shared dependency versions: `../Cargo.toml`.
- Rust command recipes: `../justfile` (`check-workspace`, `test`, `fmt-check`, `clippy`, `ci`).
- Scoped agent rules for this subtree: `./AGENTS.md`.
- Installable binary wrapper: `ccr/src/main.rs`; public Rust API re-exports: `ccr/src/lib.rs`.

## Layer map

| Layer | Crates | Use when |
|---|---|---|
| Foundation | `ccr-core`, `ccr-types` | Error/result types, file/lock/logging/http/sqlite helpers, masking utilities, shared DTOs and public API structs. |
| Configuration and platforms | `ccr-config`, `ccr-codex` | Platform registries, profile/config files, Codex/OpenCode auth, model providers, quota, runtime/session/usage services. |
| Persistence and analytics | `ccr-db`, `ccr-store` | SQLite schema/migrations/repositories, UI state, usage import, session history, pricing, budget, and cost tracking. |
| Feature domains | `ccr-skills`, `ccr-sync`, `ccr-checkin` | Skills/prompts/MCP presets, skill extension lifecycle, sync folder/WebDAV behavior, check-in provider/account/record flows. |
| User surfaces | `ccr-cli`, `ccr-tui`, `ccr` | Clap command dispatch, command handlers, terminal UI state/rendering, installable package entry point and compatibility tests. |

## Crate index

| Crate | Primary entry points | Notes |
|---|---|---|
| `ccr` | `src/main.rs`, `src/lib.rs`, `tests/` | Default package and binary. Re-exports workspace APIs; keep public compatibility changes covered by `tests/public_api_compat.rs` where applicable. |
| `ccr-cli` | `src/application/`, `src/cli/`, `src/commands/`, `src/platforms/` | CLI orchestration and command handlers. Route command work to `src/commands/<domain>/` before touching shared services. |
| `ccr-tui` | `src/tui/`, `src/tui/*_auth/` | Ratatui-style terminal UI flow, events, overlays, selection, auth screens, theme, and runtime glue. |
| `ccr-core` | `src/core/`, `src/utils/` | Shared infrastructure: atomic writes, locks, file IO, logging, HTTP, SQLite helpers, validation, masking, path utilities. |
| `ccr-types` | `src/*.rs` | Cross-crate data contracts for Claude/Codex auth, model-rate catalog, and monitoring feed types. Avoid duplicating these structs elsewhere. |
| `ccr-config` | `src/managers/`, `src/models/`, `src/platforms/`, `src/services/` | Unified config/profile management, platform config abstractions, validators, and config service orchestration. |
| `ccr-codex` | `src/managers/`, `src/models/`, `src/platforms/`, `src/services/` | Codex/OpenCode-specific config, auth crypto/service, OAuth token handling, quota, usage, sessions, runtime, and history sync. |
| `ccr-db` | `src/database/`, `src/database/repositories/`, `src/models/`, `src/services/` | SQLite pool/schema/migrations and repository layer. Schema changes usually need migrations, repository updates, and service/model checks together. |
| `ccr-store` | `src/sessions/`, `src/storage/`, `src/services/`, `src/*_manager.rs` | Session parsing/indexing/storage, history service, cost/pricing/budget managers. |
| `ccr-skills` | `src/managers/`, `src/models/`, `src/services/`, `src/skills_ext/`, `tests/` | Skills inventory/install/sync models, prompt/MCP preset managers, multi-agent target adapters, taxonomy, trash/toggle/versioning tests. |
| `ccr-sync` | `src/sync/` | Sync config, content selection, folder manager, folder models, and sync service. |
| `ccr-checkin` | `src/managers/checkin/`, `src/services/`, `src/core/` | Check-in account/provider/balance/record/export managers, WAF cookie handling, CDK and execution services. |

## Common edit routes

- CLI command behavior: start in `ccr-cli/src/commands/`; shared command parsing lives under `ccr-cli/src/cli/`.
- Platform/profile behavior: check `ccr-config/src/platforms/` and `ccr-config/src/managers/` first; Codex-only behavior belongs in `ccr-codex/`.
- Codex auth/quota/session/usage: start in `ccr-codex/src/services/`; shared auth types may live in `ccr-types`.
- SQLite schema or persisted UI/usage/check-in data: start with `ccr-db/src/database/schema.rs`, `ccr-db/src/database/migrations.rs`, then the matching repository and service.
- Session history, pricing, costs, or budget behavior: start in `ccr-store/src/sessions/`, `ccr-store/src/storage/`, or the relevant `ccr-store/src/*_manager.rs`.
- Skills/prompts/MCP preset behavior: start in `ccr-skills/src/services/skills_service.rs`, `ccr-skills/src/models/`, and `ccr-skills/src/skills_ext/`.
- Terminal UI behavior: start in `ccr-tui/src/tui/app.rs`, `ccr-tui/src/tui/ui.rs`, and the relevant auth submodule.
- Public API changes: update `ccr/src/lib.rs` re-exports deliberately and run/extend `ccr/tests/public_api_compat.rs`.

## Test and verification routing

- Single crate: `cargo test -p <crate-name>`.
- Main CLI integration tests: `cargo test -p ccr --test commands`, `cargo test -p ccr --test workflows`, or the specific test target under `crates/ccr/tests/`.
- Skills extension changes: `cargo test -p ccr-skills`.
- Cross-crate type drift: `just check-workspace`.
- Final Rust gate for substantive crate changes: `just fmt-check`, `just clippy`, and `just test` from the repository root; use `just ci` when the change also affects frontend/docs/VS Code release surfaces.

## Safety-sensitive paths

- Config/auth/token code writes into user-home tool directories such as `.ccr`, `.claude`, `.codex`, or platform-specific config paths; preserve masking, backups, locks, and atomic writes.
- Database changes in `ccr-db/src/database/` must preserve migrations and repository compatibility.
- Cleanup commands and sync flows can delete or move files; keep dry-run/confirmation behavior and fixture-owned temp paths intact.
- Check-in and quota services may touch external providers; avoid committing real cookies, tokens, account identifiers, or live responses.

## Ignore for guidance/navigation

- `target/`, `.omx/`, and any generated build output are not source guidance targets.
- `crates/ccr/src/CLAUDE.md` is provider-specific reference material, not Codex `AGENTS.md` guidance.
