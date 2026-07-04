# ccr-types Backend Guidelines

> Shared serializable contracts for the CCR ecosystem.

## Scope

`crates/ccr-types` owns shared data contracts consumed by the CLI, desktop backend, and frontend-facing APIs. Keep this crate small, serialization-focused, and free of filesystem, network, or database behavior.

Reference files:

- `crates/ccr-types/src/lib.rs`
- `crates/ccr-types/src/claude_settings.rs`
- `crates/ccr-types/src/claude_auth.rs`
- `crates/ccr-types/src/model_rate_catalog.rs`
- `crates/ccr-types/src/monitoring.rs`

## Serialization Contracts

Preserve backward compatibility:

- Derive `Serialize`/`Deserialize` on shared structs and enums.
- Use `#[serde(default)]` for fields that may be absent in older data.
- Use `#[serde(skip_serializing_if = "...")]` for clean output where existing types do.
- Preserve unknown nested fields with `#[serde(flatten)] other` where the local type already does.
- Keep aliases such as `outputStyle` plus legacy `output_style` support.

Changing a serialized field name, enum representation, or default is an API contract change. Add tests before making it.

## Structure

Keep modules private and re-export stable types from `lib.rs`. Add a new module only when a contract family is shared by more than one consumer or needs isolated tests.

Do not add `ccr-core`, `rusqlite`, `reqwest`, or filesystem dependencies here. Shared types should not depend on application services.

## Error Handling

Avoid application error types in this crate. Prefer pure constructors, validation helpers that return simple results, or let callers validate at service boundaries.

## ClaudeSettings Single Shape Contract

`ccr_types::ClaudeSettings` is the **only** `ClaudeSettings` definition in the workspace (`rg 'struct ClaudeSettings'` must hit exactly `crates/ccr-types/src/claude_settings.rs`). `ccr-cli`'s `managers::settings` and the root `ccr::ClaudeSettings` are re-exports of this type; do not reintroduce a parallel shape on either side of the CLI/UI seam.

Ownership split for managed-env behavior:

- **This crate** owns the pure data operations and key registry: `env_keys` constants (including `NON_ANTHROPIC_MANAGED_KEYS`), `clear_anthropic_vars`, `clear_managed_vars`, `apply_managed_env(pairs)` (clear-first, then insert), `anthropic_env_status`, `has_anthropic_overrides`, and validation (`validate`, `validate_api_key_mode`).
- **`ccr-config`** owns the `ConfigSection -> pairs` mapping (`ConfigSection::to_managed_env_pairs`), referencing `env_keys` constants so key names cannot drift.
- **`ccr-cli`** keeps only the IO adapter (`SettingsManager`: load/save/backup/restore).

Validation returns `Result<(), String>` with stable Chinese messages; callers wrap into their own error type (CLI uses `CcrError::ValidationError`). Do not add `Validatable` or other `ccr-core` trait impls here — this crate stays a leaf, and orphan rules prevent downstream impls anyway.

Intentional strictness kept by tests: invalid `hooks` types are a parse error (not tolerated into `other`), legacy array hooks normalize to the canonical object format on write, and empty known containers are dropped on serialization. Unknown fields must survive read→modify→write round-trips at every nesting level.

## Testing

Add serialization round-trip tests and legacy-input tests for contract changes. `model_rate_catalog.rs`, `claude_settings.rs`, and auth modules are good examples of module-local tests.

## Verification

For type-contract changes, run:

- `just fmt-check`
- `cargo test -p ccr-types -- --test-threads=1`
- Downstream targeted tests for each affected consumer
- `just lint-strict`
