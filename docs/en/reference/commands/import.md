# `import` - Deprecated Top-Level Import Command

> [!WARNING]
> **Deprecated & Hidden (7.x Compatibility)**: Top-level `ccr import <file>` is **Deprecated** and **Hidden** from `--help` in CCR `7.x`. Invoking it emits a deprecation warning to `stderr` (never polluting `stdout`), and it is scheduled for removal in `8.0`.

## Canonical Replacements

| Scenario | Canonical replacement |
|---|---|
| Non-destructive merge import (default) | `ccr <claude\|codex\|grok> profile import <FILE> [--on-conflict skip\|overwrite\|rename] [--dedup] [--dry-run] [--json]` |
| Import from stdin | `ccr <claude\|codex\|grok> profile import --stdin [--dry-run]` (or pass `-`) |
| Destructive full-catalog replacement | `ccr <claude\|codex\|grok> profile import <FILE> --replace [-y\|--yes]` |

```bash
ccr claude profile import claude-team.toml --dry-run
ccr claude profile import claude-team.toml --on-conflict overwrite --dedup --json
ccr -y codex profile import codex-full.toml --replace --json
```

## Key Behavioral Differences

- **Inverted default mode (Merge vs Replace)**: Legacy `ccr import <file>` defaulted to **destructive full Replace** unless `--merge` was passed (and defaulted implicitly to `--platform claude`). Canonical `ccr <platform> profile import <file>` defaults to **non-destructive Merge** (`--on-conflict skip`) and requires explicit `--replace` (plus interactive confirmation or `-y/--yes` in non-TTY, `--stdin`, or `--json` mode) for full replacement.
- **3-format support & strict platform matching**: Canonical import accepts `ExchangeV1` (rejecting cross-platform mismatches), `LegacyCatalog`, and `LegacySimplified`, while rejecting masked secret placeholders.
- **Credential preservation, active-profile protection & 6+1 bucket receipts**: Missing-secret additions are imported with `enabled = false` (`missing_secrets`), compatible overwrites preserve local secrets (`preserved_local_secrets`), active profiles cannot be removed or disabled, commits write an owner-only (`0600`) backup first, and `--dry-run`, `--no-apply`, `--expected-revision`, and 6+1 bucket `--json` reports are supported.

## Related Pages

- [`claude`](./claude)
- [`codex`](./codex)
- [`grok`](./grok)
- [`export`](./export)
