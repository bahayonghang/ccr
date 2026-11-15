# `switch` - Retired Legacy Profile Switch Entrypoint

> [!WARNING]
> **Retired & Hidden (7.x Compatibility Guard)**: Top-level `ccr switch <config_name>` and the bare positional shortcut `ccr <config_name>` are **Retired** and **Hidden** from `--help` in CCR `7.x`. Invoking either returns a migration error with a non-zero exit code without modifying any platform runtime, and both are scheduled for removal in `8.0`.

## Canonical Replacements

Specify the target platform (`claude`, `codex`, or `grok`) explicitly:

```bash
ccr claude profile switch <name> [--json]
ccr codex profile switch <name> [--json]
ccr grok profile switch <name> [--json]
```

| Retired entrypoint | Canonical replacement |
|---|---|
| `ccr switch <name>` (Claude profile) | `ccr claude profile switch <name> [--json]` |
| `ccr switch <name>` (Codex profile) | `ccr codex profile switch <name> [--json]` |
| `ccr switch <name>` (Grok profile) | `ccr grok profile switch <name> [--json]` |
| Bare positional `ccr <name>` | `ccr <claude\|codex\|grok> profile switch <name>` |

## Key Behavioral Differences

- **No implicit global platform inference**: The retired entrypoint guessed the target CLI from legacy global `current_platform` state, which was error-prone when running Claude Code, Codex, and Grok Build side by side.
- **Atomic per-platform runtime & JSON receipt**: `ccr <claude|codex|grok> profile switch <name>` updates only that platform's `profiles.db` pointer and host runtime files, and supports `--json` single-line structured receipts.

## Related Pages

- [`claude`](./claude)
- [`codex`](./codex)
- [`grok`](./grok)
- [`current`](./current)
- [`platform`](./platform)
- [Migration Guide](/en/reference/migration)
