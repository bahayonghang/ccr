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
      pub(crate) wire_contracts: &'static [Option<CommandWireContract>],
      pub(crate) default_risk: CommandRisk,
      pub(crate) schema: CommandSchema,
      pub(crate) platform: CommandPlatform,
  }

  pub(crate) struct CommandWireContract {
      pub(crate) input_type: &'static str,
      pub(crate) output_type: &'static str,
      #[cfg(test)]
      pub(crate) client_declaration: &'static str,
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
- Count assertions intentionally freeze the current handler surface: 315 base commands and 323 commands on Windows across 36 base modules.
- Capability descriptors cover every command ID and include risk, input/output schema, timeout, concurrency, confirmation, authorization, and audit policy.
- Every generated command row owns its handler path, exact TypeScript input/output type names, and client declaration. The manifest v2 exact count is 252/252 typed commands; generated-client functions may contain imports/type aliases only and must append declarations from the registry.
- Risk inference recognizes action verbs both at the start of an ID and after domain prefixes. For example, `claude_get_settings` is read-only and `codex_delete_session` is destructive; the module default still controls secret/system authorization and audit redaction.
- Generated inventory artifacts are `docs/{en/,}reference/tauri-command-inventory.md`, `ccr-ui/src/api/generated/command-manifest.json`, `commandCapabilities.ts`, and the generated domain clients.
- The manifest is an authoritative backend allowlist: an app command without a descriptor is rejected before the generated handler runs. Audit logging records descriptor metadata only and never logs the invoke payload.
- A cloned `InvokeResolver` timeout only races the response; it does not cancel a backend mutation or release a concurrency permit. Timeout/concurrency enforcement must live at an execution boundary with cooperative cancellation and completion-aware permit release.

### 4. Validation & Error Matrix

- New command added outside `handler_registry.rs` -> reject in review.
- Duplicate command path -> `command_registry_paths_are_unique` fails.
- Empty domain module metadata -> `command_registry_modules_are_well_formed` fails.
- Command count changes without intentional test update -> `command_registry_shape_matches_current_handler_surface` fails.
- A typed row lacks an exact wire contract -> exact count falls below typed count and the registry test fails.
- An app command reaches `generate_handler` without a descriptor -> reject with `command is not registered in the capability manifest`; do not dispatch it.
- Removing `generate_handler` re-export from `commands::mod` -> desktop `main.rs` check fails.

### 5. Good / Base / Bad Cases

- Good: add a new Codex command with handler path, exact types, and client declaration in one registry row, then regenerate all artifacts.
- Base: move no commands; only maintain module declarations in `commands::mod`.
- Bad: keep the handler in the registry but maintain the same client's `invoke()` string in a separate generator function.
- Bad: reject only the frontend Promise after a timeout while the backend mutation keeps running.
- Bad: reintroduce a long `tauri::generate_handler![...]` list directly in `commands::mod`.
- Bad: add a Windows-only command to the base registry instead of the Windows module list.

### 6. Tests Required

- Run `cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml commands::handler_registry -- --nocapture`.
- Run `just tauri-command-inventory-check` after regeneration with `just tauri-command-inventory`.
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
    codex: "Codex" [SecretMutation, Generated] => [
        super::codex::new_command => [
            "NewCommandInput",
            "NewCommandOutput",
            "export const newCommand = (input: NewCommandInput): Promise<NewCommandOutput> => invoke('new_command', { input })\n",
        ],
    ],
}
```

Keep command registration domain-shaped and testable, while `commands::mod` remains a small module index.
