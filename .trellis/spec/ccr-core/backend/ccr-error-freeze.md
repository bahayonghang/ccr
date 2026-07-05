# CcrError Freeze (ADR)

> Status: Accepted · Date: 2026-07-05 · Task: 07-03-arch-ccr-error
> Evidence: `.trellis/tasks/archive/2026-07/07-03-arch-ccr-error/research/` (inventory + options, reproducible `rg` commands)

## Decision

`CcrError` (`crates/ccr-core/src/core/error.rs`) is **frozen at its current 25 variants**:

1. **No new domain variants** (application-layer vocabulary such as History/Sync/Platform/Profile/Database/Ui/Update/Settings). New domain errors live in the crate that owns the domain, as self-owned error types.
2. **Primitive variants** (IO / lock / format tier, e.g. a future `#[from] reqwest::Error`) require case-by-case review and an intentional update of the guard test.
3. Existing `CcrError` construction sites across the legacy eight crates (cli/codex/config/skills/store/sync/tui/core) **stay as they are** — no migration.

The rejected alternative was the architecture-review candidate "move domain variants to their owning crates" (07-03-arch-deepening candidate 8).

## Why the migration was rejected

- **The ownership premise is false on the dependency graph.** The biggest constructors of domain variants are consumer crates, not owners: `UiError` = ccr-cli 38× while ccr-tui depends on ccr-cli (`crates/ccr-tui/Cargo.toml:16`); `DatabaseError` = ccr-codex 32× while ccr-codex does not depend on ccr-store; `SettingsError` = ccr-cli 51×, whose "owner" is ccr-cli itself. Only `HistoryError` (ccr-store, 11×) is fully concentrated. An honest migration is "per-crate error enums + top-level aggregation": 150–180 files, 1030+ construction sites.
- **6.x freeze makes the goal unreachable this major.** `CcrError` is a frozen prelude member (`public-api-boundary.md`) and is not `#[non_exhaustive]`; removing or re-typing public variants is a breaking change. Current line: 6.4.x.
- **The god-enum costs do not occur in practice.** Across 1082 references, production code pattern-matches a variant exactly once — and on a primitive (`codex_history_sync_service.rs` `is_locked_error` → `FileLockError`). `exit_code()/is_fatal()/user_message()` have a single consumer (`ccr::cli::dispatch::handle_error`). Zero variants were added in the 3 months since ccr-core was split out.

## Rules for new code

| Situation                                        | Correct move                                                                                                                                                                                                              |
| ------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| New crate / new subsystem                        | Self-owned error type. Precedents: `ccr-db::DbError`, `ccr-usage::UsageError`, `ccr-checkin` layered errors, `llmusage_adapter::LlmusageAdapterError`.                                                                    |
| Shared SQLite seam code (07-03-arch-sqlite-seam) | Seam speaks `DbError`; `ccr-store` bridges with `map_err(\|e\| CcrError::DatabaseError(...))` at its own boundary. Do **not** add `impl From<DbError> for CcrError`. Existing 27 `DatabaseError` sites in ccr-store stay. |
| ccr prelude shape (07-03-arch-ccr-facade)        | `CcrError`/`Result` unchanged — no dependency on this ADR's outcome.                                                                                                                                                      |
| Existing flows in the legacy eight crates        | Keep constructing the frozen variants; do not migrate, do not extend.                                                                                                                                                     |
| A flow truly needs typed error branching         | Introduce a local self-owned error there (see how ccr-tui branches on `UsageError` variants), pay the cost when the need is real.                                                                                         |

### Wrong vs Correct

```rust
// Wrong: extending the frozen enum for a new domain
// (crates/ccr-core/src/core/error.rs)
#[error("船新模块错误: {0}")]
NewModuleError(String),

// Correct: self-owned error in the owning crate, primitives via #[from]
// (crates/<owner>/src/error.rs)
#[derive(Debug, thiserror::Error)]
pub enum NewModuleError {
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),
    #[error("...: {0}")]
    Parse(String),
}
```

## Guard

- `test_variant_set_is_frozen` in `crates/ccr-core/src/core/error.rs`: an exhaustive `match` plus a `FROZEN_VARIANTS: [&str; 25]` snapshot. Adding a variant fails compilation (missing arm), removing/renaming fails compilation (unknown variant); either way the editor lands on the freeze banner pointing here. Red/green verified 2026-07-05 (commented-out arm → `error[E0004]`).
- The enum's doc comment carries the freeze notice.

## Future

- The full decomposition may be re-evaluated as a **7.0 breaking candidate**; re-run the inventory commands first (construction distribution may have drifted).
- A far cheaper slimming alternative, if ccr-core's "application knowledge" ever needs reducing: move `exit_codes`/`user_message()` rendering concerns next to their single consumer (`dispatch.rs`). Out of scope here; noted for the facade/next-major discussions.
- Non-goal: cleaning up the `ConfigError` catch-all drift (344 uses) — renaming the catch-all was never a benefit of the rejected migration either.
