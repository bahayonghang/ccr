# Responsibility-Based Module Decomposition

> Extraction contract for oversized authoritative modules. Line count alone is
> not permission to move code.

## Scenario: split only at an implemented ownership boundary

### 1. Scope / Trigger

- Trigger: moving code out of `command_exec.rs`, `migrations.rs`,
  `codex_auth.rs`, or desktop `commands/sync.rs`.
- Goal: reduce mixed ownership without changing public command paths,
  persistence semantics, or rollback behavior.

### 2. Signatures

- `command_exec.rs` target modules: `policy`, `descriptor`, `resolver`,
  `foreground`, `job`, `events`, and platform `process_tree`.
- `migrations.rs` target modules: runner/registry, release migration families,
  repair migrations, fixtures, and postconditions.
- `codex_auth.rs` target modules: DTO/commands, OAuth flow, external navigation,
  process ownership, import/export, and tests.
- `sync.rs` target modules: asset catalog, config ownership, action truth table,
  pull transaction, encryption adapter, and Tauri DTO/commands.

### 3. Contracts

- Extract only after the owning gateway or policy has a named interface and
  focused tests. Preserve existing public command/module paths with narrow
  re-exports where compatibility requires them.
- Keep policy decisions separate from OS/process adapters, DTO serialization,
  persistence, and runtime state. A new module owns one responsibility and its
  tests; it is not an arbitrary line range.
- Do not combine the ccr-store and ccr-db migration runners. Their published
  marker schemas and process/database ownership remain distinct.

### 4. Validation & Error Matrix

- Extraction changes a Tauri command name or handler registration -> reject or
  provide a reviewed compatibility bridge and inventory update.
- Migration move changes published version/marker behavior -> reject; preserve
  transaction and postcondition contracts.
- Module introduces a dependency cycle or umbrella `ccr` dependency ->
  dependency governance fails.
- Move has no named responsibility or focused regression test -> defer it.

### 5. Good/Base/Bad Cases

- Good: move process-tree termination behind the existing gateway interface and
  keep cancellation regression tests with the adapter.
- Base: leave an oversized file intact after documenting its future boundaries.
- Bad: split every 500 lines, create `part1.rs`, or mix DTOs and OS process
  mutation merely to reduce file length.

### 6. Tests Required

- Run the owning gateway's focused regression target before and after a move.
- Run handler inventory checks for Tauri command moves.
- Run `cargo test -p ccr-db migration -- --test-threads=1` for migration moves.
- Run `just fmt-check`, the affected subsystem gate, and `just lint-strict`.

### 7. Wrong vs Correct

#### Wrong

```rust
mod command_exec_part2;
```

#### Correct

```rust
mod process_tree;
mod policy;
```

Names expose ownership, and extraction waits until those contracts and tests
exist.
