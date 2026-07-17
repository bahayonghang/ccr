# Tauri Handler Registry

> Executable contract for `ccr-ui/src-tauri/src/commands/handler_registry.rs`.

## Scenario: domain command registry for Tauri invoke handlers

### 1. Scope / Trigger

- Trigger: adding, moving, renaming, or removing a Tauri `#[tauri::command]` function.
- Applies to `ccr-ui/src-tauri/src/commands/mod.rs` and `ccr-ui/src-tauri/src/commands/handler_registry.rs`.
- Goal: keep `commands::mod` as module wiring only, and keep invoke handler growth inside a domain-shaped registry with metadata and tests.

### 2. Signatures

- Public handler entry:
  ```rust
  pub use handler_registry::generate_handler;
  ```
- Registry module shape:
  ```rust
  pub(crate) struct CommandModule {
      pub(crate) key: &'static str,
      pub(crate) title: &'static str,
      pub(crate) commands: &'static [&'static str],
  }
  ```
- Required focused test:
  ```powershell
  cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml commands::handler_registry -- --nocapture
  ```

### 3. Contracts

- `commands::mod` owns only submodule declarations plus `pub use handler_registry::generate_handler`.
- Base commands live in `COMMAND_MODULES` through `define_command_registry!`.
- Windows-only WSL commands live in `WINDOWS_COMMAND_MODULES` and the Windows `generate_handler()` arm.
- Each command path must appear once; duplicate command paths fail `command_registry_paths_are_unique`.
- Registry metadata must remain non-empty and domain-keyed; empty module keys, titles, or command lists are invalid.
- Count assertions intentionally freeze the current handler surface: 315 base commands and 323 commands on Windows across 30 base modules.

### 4. Validation & Error Matrix

- New command added outside `handler_registry.rs` -> reject in review.
- Duplicate command path -> `command_registry_paths_are_unique` fails.
- Empty domain module metadata -> `command_registry_modules_are_well_formed` fails.
- Command count changes without intentional test update -> `command_registry_shape_matches_current_handler_surface` fails.
- Removing `generate_handler` re-export from `commands::mod` -> desktop `main.rs` check fails.

### 5. Good / Base / Bad Cases

- Good: add a new Codex command under the existing `codex` registry group and update the command count test.
- Base: move no commands; only maintain module declarations in `commands::mod`.
- Bad: reintroduce a long `tauri::generate_handler![...]` list directly in `commands::mod`.
- Bad: add a Windows-only command to the base registry instead of the Windows module list.

### 6. Tests Required

- Run `cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml commands::handler_registry -- --nocapture`.
- Run `cargo check --manifest-path ccr-ui/src-tauri/Cargo.toml --bin ccr-desktop`.
- Run `git diff --check`.
- `cargo clippy --manifest-path ccr-ui/src-tauri/Cargo.toml --bin ccr-desktop -- -D warnings` is the intended full lint gate; if it fails on unrelated existing warnings, record the blocker rather than broadening this slice.

### 7. Wrong vs Correct

#### Wrong

```rust
// commands/mod.rs
tauri::generate_handler![
    codex::new_command,
]
```

#### Correct

```rust
// commands/handler_registry.rs
define_command_registry! {
    codex: "Codex" => [
        super::codex::new_command,
    ],
}
```

Keep command registration domain-shaped and testable, while `commands::mod` remains a small module index.
