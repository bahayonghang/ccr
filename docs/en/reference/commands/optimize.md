# `optimize` - Deprecated Top-Level Config Sorting Command

> [!WARNING]
> **Deprecated & Hidden (7.x Compatibility)**: Top-level `ccr optimize` is **Deprecated** and **Hidden** from `--help` in CCR `7.x`. Invoking it emits a deprecation warning to `stderr` (never polluting `stdout`), and it is scheduled for removal in `8.0`.

## Canonical Replacements

| Scenario | Canonical replacement |
|---|---|
| Day-to-day catalog ordering | No command needed: SQLite `~/.ccr/platforms/<platform>/profiles.db` maintains canonical ordering automatically |
| Inspect or edit canonical TOML projection in `$VISUAL` / `$EDITOR` | `ccr <claude\|codex\|grok> profile open [--json]` |
| Export deterministically ordered interchangeable TOML | `ccr <claude\|codex\|grok> profile export [--no-secrets] [-o <FILE>]` |
| Read-only cross-platform configuration validation | `ccr validate [--json]` |

## Key Behavioral Differences

- **SQLite catalogs eliminate manual sorting**: Legacy `ccr optimize` rewrote legacy Claude TOML files on disk to sort section headers alphabetically; in `7.x`, authoritative profile catalogs live in SQLite (`profiles.db`), which maintains canonical structure automatically.
- **Deterministic TOML projections**: Both `ccr <claude|codex|grok> profile open` and `profile export` always render deterministic canonical TOML without requiring a separate optimization step.

## Related Pages

- [`claude`](./claude)
- [`codex`](./codex)
- [`grok`](./grok)
- [`validate`](./validate)
