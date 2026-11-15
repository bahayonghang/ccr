# `list` - Deprecated Top-Level Profile Listing Command

> [!WARNING]
> **Deprecated & Hidden (7.x Compatibility)**: Top-level `ccr list` (and its alias `ccr ls`) is **Deprecated** and **Hidden** from `--help` in CCR `7.x`. Invoking it emits a deprecation warning to `stderr` (never polluting `stdout`), and it is scheduled for removal in `8.0`.

## Canonical Replacements

| Scenario | Canonical replacement |
|---|---|
| List Claude profiles | `ccr claude profile list [--json]` |
| List Codex profiles | `ccr codex profile list [--json]` |
| List Grok profiles | `ccr grok profile list [--json]` |
| Inspect cross-platform runtime state | `ccr current [--verbose \| --json]` (alias `ccr status`) |
| Inspect registered platforms and pointers | `ccr platform list [--json]` |

```bash
ccr claude profile list
ccr codex profile list --json
ccr grok profile list
```

## Key Behavioral Differences

- **Explicit platform scope**: Legacy `ccr list` only inspects the legacy Claude profile scope, whereas `ccr <claude|codex|grok> profile list` targets each platform's own `profiles.db` catalog explicitly.
- **Structured JSON & catalog revision**: `profile list --json` returns `{ "current_profile": "..." | null, "revision": "..." | null, "profiles": [...] }`, exposing top-level `revision` for optimistic concurrency via `--expected-revision`.

## Related Pages

- [`claude`](./claude)
- [`codex`](./codex)
- [`grok`](./grok)
- [`current`](./current)
- [`platform`](./platform)
