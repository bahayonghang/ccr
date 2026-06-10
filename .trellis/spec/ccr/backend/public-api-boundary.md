# Public API Boundary

> Executable contract for the root `ccr` crate public import surface.

## Scenario: stable prelude and legacy root re-export freeze

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
- Existing root exports such as `ccr::application`, `ccr::commands`, `ccr::managers`, `ccr::models`, `ccr::services`, `ccr::sync`, `ccr::sessions`, and root-level legacy type re-exports remain available until an explicit breaking release plan exists.
- In the 6.x line, do not add `#[deprecated]` to the broad root re-export bridge merely to signal preference; warning-producing deprecations would break `-D warnings` downstream builds and need a next-major breaking-change list first.
- New stable public API should be added to `ccr::prelude` first when it fits the public error/platform/config/service contract.
- New broad root `pub use` or `pub mod` lines require an intentional snapshot update in `crates/ccr/tests/public_api_compat.rs`.

### 4. Validation & Error Matrix
- New root `pub use` / `pub mod` without snapshot update -> `crate_root_public_reexport_snapshot_is_intentional` fails.
- Removing legacy paths used by downstream callers -> `legacy_public_paths_remain_available` fails.
- Removing the compatibility-only bridge documentation around the broad root `ccr_cli` re-export -> `crate_root_public_reexport_snapshot_is_intentional` fails.
- Removing or renaming stable prelude exports -> `stable_prelude_paths_remain_available` fails.
- Adding internal-only implementation details to `prelude` -> reject in review; keep them behind explicit modules or domain crates.

### 5. Good / Base / Bad Cases
- Good: add a stable DTO or service type to `ccr::prelude`, update the prelude import test, and explain why it is public.
- Base: keep a compatibility-only legacy root export unchanged while adding no new surface or warnings.
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
