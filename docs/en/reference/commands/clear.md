# `clear` - Deprecated Top-Level Settings Clear Command

> [!WARNING]
> **Deprecated & Hidden (7.x Compatibility)**: Top-level `ccr clear [--force]` is **Deprecated** and **Hidden** from `--help` in CCR `7.x`. Invoking it emits a deprecation warning to `stderr` (never polluting `stdout`), and it is scheduled for removal in `8.0`.

## Canonical Replacements

| Scenario | Canonical replacement |
|---|---|
| Exit Claude profile mode and remove CCR runtime overrides | `ccr claude profile off [--json]` |
| Exit Codex profile mode and remove CCR routing and runtime `auth.json` | `ccr codex profile off [--json]` |
| Exit Grok profile mode and restore pre-profile model/reasoning settings | `ccr grok profile off [--json]` |
| Log out the official runtime session (independent from `profile off`) | `ccr <claude\|codex\|grok> auth off [--json]` |
| Clear a temporary token override | `ccr temp-token clear` |

```bash
ccr claude profile off --json
ccr codex profile off
ccr grok profile off
```

## Key Behavioral Differences

- **Multi-platform pointer & runtime transaction**: Legacy `ccr clear` only scrubbed environment keys in Claude `settings.json` without clearing profile pointers; `ccr <claude|codex|grok> profile off` atomically clears `current_profile` pointers and cleans up host runtime files with transactional backup rollback.
- **External auth suppressor reporting**: `ccr claude profile off --json` returns `{ ok, changed, previous_profile, runtime_mode, remaining_suppressors, warnings }`, surfacing external credential sources (such as process environment variables) that CCR does not delete automatically.

## Related Pages

- [`claude`](./claude)
- [`codex`](./codex)
- [`grok`](./grok)
- [`temp-token`](./temp-token)
