# Public API Boundary

> Executable contract for the root `ccr` crate public import surface.

## Scenario: stable prelude and deprecated legacy root modules

### 1. Scope / Trigger

- Trigger: Phase 6/A8 architecture convergence for `crates/ccr/src/lib.rs`.
- Applies when adding new public application-facing Rust API from the `ccr` crate.
- Goal: keep the historical root re-exports compatible while giving new consumers a smaller, intentional `ccr::prelude` path.

### 2. Signatures

- Stable prelude module:
  ```rust
  pub mod prelude {
      pub use crate::{
          CcrError, Result,
          Platform, PlatformConfig, PlatformConfigEntry, PlatformConfigManager, PlatformPaths,
          ProfileConfig, UnifiedConfig,
          ConfigManager, SettingsManager, HistoryManager,
          ConfigService, SettingsService, HistoryService, BackupService, ValidateService,
          create_platform,
      };
  }
  ```
- Compatibility guard test:
  ```rust
  cargo test -p ccr --test public_api_compat -- --nocapture
  ```

### 3. Contracts

- `ccr::prelude` is the preferred stable import surface for application and integration consumers.
- Existing root modules such as `ccr::application`, `ccr::commands`,
  `ccr::managers`, `ccr::models`, `ccr::platforms`, `ccr::services`, and
  `ccr::sync` are deprecated in 7.x with the owning `ccr_cli` module named as
  the replacement. They remain callable until 8.0.0 at the earliest.
- `ccr::sessions`, the stable prelude, and intentional root-level type exports
  are not part of the broad-module deprecation.
- New repository code must use the owning domain crate or `ccr::prelude` and
  must not suppress the deprecation warning globally. The compatibility test
  alone uses `#![allow(deprecated)]` to prove old imports still compile.
- Rust reports the module-level deprecation when the module symbol itself is
  imported (for example `use ccr::managers;`). It does not reliably propagate
  that warning through every nested glob-re-exported item path. The manifest
  dependency guard is therefore the fail-closed repository rule; the doctest
  proves the public module still carries its warning metadata.
- New stable public API should be added to `ccr::prelude` first when it fits the public error/platform/config/service contract.
- New broad root `pub use` or `pub mod` lines require an intentional snapshot update in `crates/ccr/tests/public_api_compat.rs`.
- The `ccr` crate is a thin facade (since 2026-07-05, task 07-03-arch-ccr-facade): `main.rs` + frozen `lib.rs` bridge + `cli/mod.rs` forwarding only. Command dispatch lives in `ccr_cli::cli::dispatch`; `CommandDispatcher::dispatch(cli, Option<&TuiLaunchers>)` takes injected TUI launchers because `ccr-cli` must never depend on `ccr-tui` (`ccr-tui` already depends on `ccr-cli`; adding the reverse edge is a dependency cycle). `crates/ccr/src/main.rs` is the only place that constructs `TuiLaunchers`.
- `crates/ccr/Cargo.toml` `[dependencies]` stays converged to the real import set (`ccr-cli`, `ccr-core`, `ccr-store`, `ccr-tui` optional, `clap`, `tokio`); test-only deps belong in `[dev-dependencies]`. Do not re-add pass-through dependencies "for type reachability" — reachability comes through `ccr-cli`.
- Re-export wall rule for `ccr-cli/src/{models,managers,services}/mod.rs`: every entry must have a real consumer through a wall path (`crate::X::`, `ccr_cli::X::`, or the `ccr::` bridge). Adding a new re-export entry requires naming its consumer in the PR/commit. 59 consumer-less entries were removed 2026-07-05; the per-symbol audit lives in the task archive (`07-03-arch-ccr-facade/research/inventory.md` C8).

### 3b. Breaking-change candidates (next major, 8.0 or later)

- Remove the seven deprecated broad modules only after a separately reviewed
  8.0 breaking inventory proves repository consumers have migrated. Keep
  `ccr::prelude`, `ccr::sessions`, and intentional root-level types unless that
  review explicitly supersedes their contracts.
- See also `ccr-core/backend/ccr-error-freeze.md` for the registered `CcrError` decomposition candidate.

### 4. Validation & Error Matrix

- New root `pub use` / `pub mod` without snapshot update -> `crate_root_public_reexport_snapshot_is_intentional` fails.
- Removing legacy paths used by downstream callers -> `legacy_public_paths_remain_available` fails.
- Removing a deprecated broad module before 8.0.0 -> reject in review even if
  repository-local callers have migrated.
- Adding a new repository import through a deprecated `ccr::<broad-module>`
  path -> `-D warnings` fails; use `ccr_cli` or the owning domain crate.
- Removing the compatibility-only bridge documentation around the broad root `ccr_cli` re-export -> `crate_root_public_reexport_snapshot_is_intentional` fails.
- Removing or renaming stable prelude exports -> `stable_prelude_paths_remain_available` fails.
- Adding internal-only implementation details to `prelude` -> reject in review; keep them behind explicit modules or domain crates.

### 5. Good / Base / Bad Cases

- Good: add a stable DTO or service type to `ccr::prelude`, update the prelude import test, and explain why it is public.
- Base: keep a deprecated compatibility module callable while migrating
  repository code to the narrow crate.
- Bad: add `pub use ccr_cli::some_internal_module::*;` at the crate root to make one caller compile.

### 6. Tests Required

- Run `cargo test -p ccr --test public_api_compat -- --nocapture`.
- Run `cargo clippy -p ccr --all-targets --all-features -- -D warnings` for code changes.
- Run `just fmt-check` before committing Rust public API boundary changes.

### 7. Wrong vs Correct

#### Wrong

```rust
pub use ccr_cli::commands::internal_helper::*;
```

This expands the root API with implementation details and bypasses the compatibility snapshot.

#### Correct

```rust
pub mod prelude {
    pub use crate::{CcrError, Result, Platform, create_platform};
}
```

Expose small, intentional contracts through `prelude`, keep legacy root exports stable, and update the snapshot only with a rationale.
