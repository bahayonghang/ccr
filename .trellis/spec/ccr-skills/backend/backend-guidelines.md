# ccr-skills Backend Guidelines

> Skills, builtin prompts, MCP presets, and skill extension metadata.

## Scope

`crates/ccr-skills` owns skills inventory, builtin prompts, MCP presets, and `skills_ext` enhancements such as agent locators, taxonomy, conflicts, toggles, trash, and version history.

Reference files:

- `crates/ccr-skills/src/lib.rs`
- `crates/ccr-skills/src/services/skills_service.rs`
- `crates/ccr-skills/src/skills_ext/mod.rs`
- `crates/ccr-skills/tests/skills_ext_scanner_test.rs`

## Structure

Keep the current split:

- `managers/` for prompt and preset managers.
- `models/` for serialized skill/prompt/MCP contracts.
- `services/` for higher-level skills operations.
- `skills_ext/` for additive skill-hub-style capabilities.

`skills_ext` is explicitly incremental and should not break `services::skills_service` APIs without a migration plan.

## Persistence And Filesystem Rules

Skill filesystem operations must respect the existing source/scope model: global, project, plugin, symlink, and unknown sources. Do not collapse symlinked shared locations into local files. Keep trash/versioning/toggle stores responsible for their own on-disk layout.

Avoid committing user home-directory skill content, generated local caches, or secrets from MCP configuration. Use fixtures under tests.

## Error Handling

Use existing domain errors (`ToggleError`, `TrashError`, `VersioningError`) where they exist. New operations should return structured errors instead of strings when callers need to present different recovery actions.

Do not use `unwrap`/`expect` in production scanning or filesystem traversal; malformed skills should be reported as records or health findings.

## Logging

Use `tracing` for scan/health diagnostics. Do not log raw credential values from MCP presets or copied skill content that may contain local secrets.

## Testing

Use integration tests under `crates/ccr-skills/tests/` for extension behavior. Existing tests are organized by feature area, for example scanner, taxonomy, toggle, trash, versioning, and agents.

## Verification

For skills changes, run:

- `just fmt-check`
- `cargo test -p ccr-skills -- --test-threads=1`
- `just lint-strict`
