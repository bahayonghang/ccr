# P3 maintainability cleanup design

## Facade governance for 7.x

The working tree is preparing version 7.0.0, so the compatibility facade can
begin a warning-producing deprecation cycle without pretending the change is
still in 6.x. Broad root/module re-exports receive deprecation notes naming the
narrow domain crate or `ccr::prelude` replacement and the intended removal
window. Stable prelude items remain supported.

A manifest-policy script rejects new workspace dependencies on the umbrella
`ccr` crate from internal crates unless an explicit compatibility fixture is
allowlisted. Existing external compatibility is tested with compile fixtures.

## Responsibility-based module decomposition

Decomposition follows the owning gateway work and preserves public command
paths:

- `command_exec.rs`: `policy`, `descriptor`, `resolver`, `foreground`, `job`,
  `events`, and platform `process_tree` modules;
- `migrations.rs`: registry/framework, one module per release migration family,
  repair migrations, fixtures, and postconditions;
- `codex_auth.rs`: DTO/commands, OAuth flow, external navigation, process
  ownership, import/export, and tests;
- `sync.rs`: asset catalog, config ownership, action/truth table, pull
  transaction, encryption adapter, and Tauri DTO/commands.

Files move only when the corresponding policy boundary is implemented and
tested. Pure line-count splitting is rejected. The module map is recorded in a
repository architecture note and linked from the affected specs.

## Encoding and JSON formatting

Mojibake comments are replaced with verified UTF-8 Chinese without changing
code. A dependency-free repository script parses and deterministically formats
the selected human-authored JSON configuration files (including
`tauri.conf.json`) with two-space indentation and trailing newline. Lockfiles,
generated bindings, fixtures whose whitespace is semantic, and third-party
assets are explicitly excluded.

The check mode fails on a diff and is called by `just fmt-check`/hosted CI. The
repair mode is an explicit formatting command and its diff is inspected.

## Compatibility and rollback

- Deprecations supply actionable replacements; removals do not occur again in
  the same 7.x cycle without a separately reviewed breaking change.
- Module moves preserve exported paths through narrow re-exports during the
  migration.
- Formatting changes are mechanical and isolated from semantic changes in
  review/commit boundaries.
