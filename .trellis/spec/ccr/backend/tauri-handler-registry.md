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
- Build-time ACL entry:
  ```rust
  let app_manifest = tauri_build::AppManifest::new().commands(registry_commands()?);
  tauri_build::try_build(tauri_build::Attributes::new().app_manifest(app_manifest))?;
  ```
- Generated application permission sets live in
  `ccr-ui/src-tauri/permissions/command-inventory.toml`:
  `main-command-inventory` contains every manifest command, while
  `codex-tray-command-inventory` contains only the commands invoked by the tray panel.
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
- Count assertions intentionally freeze the current handler surface: 330 base commands and 338 commands on Windows across 37 base modules.
- Capability descriptors cover every command ID and include risk, input/output schema, timeout, concurrency, confirmation, authorization, and audit policy.
- Every generated command row owns its handler path, exact TypeScript input/output type names, and client declaration. The manifest v2 exact count is 267/267 typed commands; generated-client functions may contain imports/type aliases only and must append declarations from the registry.
- Risk inference recognizes action verbs both at the start of an ID and after domain prefixes. For example, `claude_get_settings` is read-only and `codex_delete_session` is destructive; the module default still controls secret/system authorization and audit redaction.
- Generated inventory artifacts are `docs/{en/,}reference/tauri-command-inventory.md`, `ccr-ui/src/api/generated/command-manifest.json`, `commandCapabilities.ts`, and the generated domain clients.
- The manifest is an authoritative backend allowlist: an app command without a descriptor is rejected before the generated handler runs. Audit logging records descriptor metadata only and never logs the invoke payload.
- `build.rs` structurally parses manifest schema v2, checks `windows_command_count`, rejects empty/duplicate IDs, and registers all commands through `AppManifest::commands`. Do not parse the JSON with line matching or maintain a second Rust command array.
- The main window receives `main-command-inventory`. Auxiliary windows receive an explicit least-privilege set; `codex-tray-panel` currently has only snapshot, auth switch, main-window show, quit, and drag lifecycle commands. Tauri ACL rejection is the runtime authorization boundary, before the generated handler.
- A cloned `InvokeResolver` timeout only races the response; it does not cancel a backend mutation or release a concurrency permit. Timeout/concurrency enforcement must live at an execution boundary with cooperative cancellation and completion-aware permit release.

### 4. Validation & Error Matrix

- New command added outside `handler_registry.rs` -> reject in review.
- Duplicate command path -> `command_registry_paths_are_unique` fails.
- Empty domain module metadata -> `command_registry_modules_are_well_formed` fails.
- Command count changes without intentional test update -> `command_registry_shape_matches_current_handler_surface` fails.
- A typed row lacks an exact wire contract -> exact count falls below typed count and the registry test fails.
- An app command reaches `generate_handler` without a descriptor -> reject with `command is not registered in the capability manifest`; do not dispatch it.
- Manifest schema/count/ID drift -> fail `build.rs` before Tauri code generation.
- Auxiliary window invokes a command outside its permission set -> Tauri rejects it with `Command <id> not allowed by ACL`; the application handler is not called.
- Removing `generate_handler` re-export from `commands::mod` -> desktop `main.rs` check fails.

### 5. Good / Base / Bad Cases

- Good: add a new Codex command with handler path, exact types, and client declaration in one registry row, then regenerate all artifacts.
- Good: add a tray-panel command to the registry and the explicit tray command slice, regenerate inventory permissions, and prove the resolved ACL contains it.
- Base: move no commands; only maintain module declarations in `commands::mod`.
- Bad: give an auxiliary window `main-command-inventory` to make an ACL error disappear.
- Bad: keep the handler in the registry but maintain the same client's `invoke()` string in a separate generator function.
- Bad: reject only the frontend Promise after a timeout while the backend mutation keeps running.
- Bad: reintroduce a long `tauri::generate_handler![...]` list directly in `commands::mod`.
- Bad: add a Windows-only command to the base registry instead of the Windows module list.

### 6. Tests Required

- Run `cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml commands::handler_registry -- --nocapture`.
- The registry tests must read Tauri's generated `acl-manifests.json` and `capabilities.json`, assert 338 main permissions, assert the exact tray subset, and prove the tray does not inherit the main permission set.
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

```json
{
  "windows": ["codex-tray-panel"],
  "permissions": ["main-command-inventory"]
}
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

```json
{
  "windows": ["codex-tray-panel"],
  "permissions": ["codex-tray-command-inventory"]
}
```

Keep command registration domain-shaped and testable, while `commands::mod` remains a small module index.

---

## Scenario: completion-aware command runtime policy

### 1. Scope / Trigger

- Trigger: adding or changing a registry command's timeout, concurrency, confirmation, or cancellation behavior.
- Applies to all 336 desktop commands under `src/commands/**`, the local `command-macros` proc-macro crate, and `runtime_policy.rs`.

### 2. Signatures

```rust
#[ccr_tauri_command_macros::command]
pub async fn command_name(...) -> Result<T, String> { ... }

pub(crate) async fn execute<T, F>(
    command: &'static str,
    future: F,
) -> Result<T, String>
where
    F: Future<Output = Result<T, String>>;
```

The attribute macro accepts only `async fn` returning `Result<T, String>`, emits `#[tauri::command]`, and wraps the real command body in `runtime_policy::execute`.

### 3. Contracts

- Every registered command uses `#[ccr_tauri_command_macros::command]`; direct `#[tauri::command]` attributes under `src/commands/**` are forbidden.
- `Parallel` acquires no permit. `ModuleExclusive` uses one permit per registry module. `Singleton` uses one global permit.
- Queue admission is bounded by the descriptor's `timeout_ms`. Once admitted, the permit is held until the real command future completes or a cooperative deadline drops that future.
- `Cooperative` is a hard deadline only for work known to be cancellation-safe at await points; currently only `test_webdav_config` qualifies.
- `CompletionAware` bounds queue admission but awaits admitted work to completion. It must be used when dropping the future could detach `spawn_blocking`, filesystem mutation, or other side effects.
- `BusinessOwned` delegates timeout, cancellation, and cleanup to the owning business boundary, such as `ProcessGateway` or an install attempt.
- Confirmation is checked in `generate_handler` before dispatch. `user_gesture` requires `desktop-confirm:<command>`; opaque capability commands require non-empty backend-issued `planId` or `request.challenge_id`.
- A cloned `InvokeResolver` race is not an execution boundary: it cannot observe or cancel the real handler future and must not release a permit early.

### 4. Validation & Error Matrix

- Descriptor missing -> `command_runtime_policy_missing:<command>`.
- Per-module gate missing -> `command_runtime_module_missing:<module>`.
- Queue wait exceeds `timeout_ms` -> `command_queue_timeout:<command>`.
- Cooperative execution exceeds the deadline -> `command_timeout:<command>`.
- Gate closed -> `command_runtime_gate_closed:<command>`.
- Missing or invalid confirmation, or missing descriptor before dispatch -> reject with `command rejected by the capability manifest`; do not run the command body.
- Non-async command or non-`Result<T, String>` return -> proc-macro compile error.

### 5. Good / Base / Bad Cases

- Good: a module-exclusive config write keeps its permit while its `spawn_blocking` join handle is awaited.
- Good: a process command uses `BusinessOwned` and lets `ProcessGateway` own termination and cleanup.
- Base: a read-only command runs in parallel with a completion-aware admission deadline.
- Bad: wrap a responder in `tokio::time::timeout` and report timeout while the mutation continues.
- Bad: classify a non-cooperative file write as `Cooperative` without proving cancellation safety.

### 6. Tests Required

- Assert exactly 338 managed command attributes and zero direct `#[tauri::command]` attributes under `src/commands/**`.
- `cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml runtime_policy -- --test-threads=1`: prove cooperative timeout and permit lifetime.
- `cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml handler_registry -- --test-threads=1`: prove timeout ownership and confirmation payload validation.
- `cargo clippy --manifest-path ccr-ui/src-tauri/Cargo.toml --bin ccr-desktop -- -D warnings`.

### 7. Wrong vs Correct

#### Wrong

```rust
tokio::select! {
    _ = sleep(deadline) => resolver.reject("timeout"),
    _ = handler_started => {}
}
```

#### Correct

```rust
#[ccr_tauri_command_macros::command]
pub async fn update_config(...) -> Result<ConfigDto, String> {
    tokio::task::spawn_blocking(move || service.update(...))
        .await
        .map_err(|error| format!("Task join error: {error}"))?
}
```

The macro keeps the runtime permit around the actual future, including the join handle.
