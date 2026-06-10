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

## Testing

Add serialization round-trip tests and legacy-input tests for contract changes. `model_rate_catalog.rs`, `claude_settings.rs`, and auth modules are good examples of module-local tests.

## Verification

For type-contract changes, run:

- `just fmt-check`
- `cargo test -p ccr-types -- --test-threads=1`
- Downstream targeted tests for each affected consumer
- `just lint-strict`
