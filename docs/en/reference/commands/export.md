# `export` - Deprecated Top-Level Export Command

> [!WARNING]
> **Deprecated & Hidden (7.x Compatibility)**: Top-level `ccr export` is **Deprecated** and **Hidden** from `--help` in CCR `7.x`. Invoking it emits a deprecation warning to `stderr` (never polluting `stdout`), and it is scheduled for removal in `8.0`.

## Canonical Replacements

| Scenario | Canonical replacement |
|---|---|
| Export Claude profiles (all, by name, or by tag) | `ccr claude profile export [NAME]... [-t\|--tag <TAG>] [-o <FILE>] [--stdout] [--no-secrets] [-y\|--yes] [--json]` |
| Export Codex profiles | `ccr codex profile export [NAME]... [-t\|--tag <TAG>] [-o <FILE>] [--stdout] [--no-secrets] [-y\|--yes] [--json]` |
| Export Grok profiles | `ccr grok profile export [NAME]... [-t\|--tag <TAG>] [-o <FILE>] [--stdout] [--no-secrets] [-y\|--yes] [--json]` |

```bash
# Redacted export (safe to share or commit to version control)
ccr claude profile export --no-secrets -o claude-team.toml

# Private backup (includes inline secrets by default; -o writes owner-only 0600 files)
ccr codex profile export relay -o codex-relay.toml
ccr grok profile export -t work --json
```

> [!IMPORTANT]
> `profile export` includes inline secrets by default (`secrets_mode = "included"`), writing owner-only (`0600`) files intended solely for private local backups. Never redirect unredacted `--stdout` via shell `>` (which uses shell `umask` instead of `0600`), and always pass `--no-secrets` when sharing or committing exported files.

## Key Behavioral Differences

- **Versioned `ExchangeV1` header**: Legacy `ccr export` (which defaulted implicitly to `--platform claude`) emitted unversioned catalog TOML without a platform header; `ccr <platform> profile export` emits `ExchangeV1` (`schema_version = 1`, `platform = "<claude|codex|grok>"`, `secrets_mode = "included" | "redacted"`, `[profiles.<name>]`) so imports enforce strict platform matching.
- **Complete secret scrubbing instead of partial masking**: Canonical `profile export --no-secrets` completely removes inline secrets, sensitive extension fields, and URL userinfo while preserving explicit `auth_mode`, rather than writing masked placeholder strings.
- **Pure read-only subset export & overwrite guard**: Canonical export has zero lock or migration side effects on `profiles.db`, supports `[NAME]...` or `-t/--tag` filtering, writes owner-only `0600` files for secret-bearing private backups (pass `--no-secrets` before sharing or committing), and exits with code `64` instead of overwriting an existing `-o` file in non-TTY or `--json` mode without `-y/--yes`.

## Related Pages

- [`claude`](./claude)
- [`codex`](./codex)
- [`grok`](./grok)
- [`import`](./import)
