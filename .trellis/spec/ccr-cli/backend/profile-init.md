# Profile Initialization

## 1. Scope / Trigger

- Trigger: changing `ccr claude/codex/grok profile init`, embedded profile
  examples, profile directory creation, or platform registry bootstrap.
- Applies to the three profile command trees, the shared profile-init handler,
  `ccr-config` registry helpers, `examples/`, and migration/help output.

## 2. Signatures

- `ccr <claude|codex|grok> profile init [--json]`
- `platform_profile_init_command(platform_name, template, json) -> Result<()>`
- `register_platform_if_missing(platform_name, description) -> Result<bool>`
- Template paths:
  - `examples/claude/profiles.example.toml`
  - `examples/codex/profiles.toml`
  - `examples/grok/profiles.toml`

## 3. Contracts

- Init creates `PlatformPaths` directories, initializes the SQLite catalog
  `profiles.db` from the embedded canonical TOML template (without creating a
  live `profiles.toml`; if an unmigrated legacy `profiles.toml` is present,
  runs `migrate_catalogs()` first), and registers the platform entry in
  `~/.ccr/config.toml`. It does not apply a profile or read/write the target CLI
  runtime.
- Every init template has `current_config = ""`. Initialization must leave the
  registry entry's `current_profile` unset.
- Templates are embedded from `examples/` with `include_str!` (`profile_open::template_for`); do not maintain
  a second command-only template. The Grok docs example (`docs/examples/grok-profiles.toml`) is a byte-identical
  mirror of the canonical Grok example (`examples/grok/profiles.toml`).
- Catalog creation is create-if-absent via `ConfigManager::replace_toml(template, "")`
  (empty expected revision token), bounded retry for concurrent initialization,
  and owner-only permissions (`0o600` on Unix / protected owner DACL on Windows).
  Version conflict when the catalog now exists returns `created = false` as an
  idempotent no-op.
- Platform registration holds the `platform_registry` named lock across load,
  membership check, backup, and save. An existing entry returns `false` before
  backup/save.
- Human output identifies created versus existing catalog state (`profiles.db` path)
  and suggests `ccr <platform> profile list`. JSON includes `ok`, `platform`,
  `profiles_file` (the `profiles.db` path), `created`, and `registered`.
- Grok copy-ready profiles use session auth or Grok Build's `api_key` field
  with a non-secret placeholder. `env_key` remains supported only for an
  environment variable name.

## 4. Validation & Error Matrix

- Embedded template does not parse -> `ConfigError`; do not write it.
- Profiles file already exists or loses a concurrent create race -> success
  with `created = false`; do not overwrite or back up the file.
- Platform already exists in the registry -> success with `registered = false`;
  do not save or back up the registry.
- Lock or filesystem failure -> propagate the shared error and do not report
  successful initialization.
- Removed `ccr platform init` -> Clap unknown-subcommand parse error; `ccr platform --help`
  points to `ccr <claude|codex|grok> profile init`.

## 5. Tests Required

- Parse all embedded templates, validate every profile through its platform,
  and assert every `current_config` is empty.
- Assert first init, idempotent repeat, inactive registry state, and unchanged
  platform runtime for Claude, Codex, and Grok.
- Assert concurrent Grok init processes both succeed and produce the exact
  canonical template; on Unix assert mode `0o600`.
- Assert JSON state fields, Clap parsing, help discovery, and migration text.
