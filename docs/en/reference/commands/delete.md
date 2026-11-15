# `delete` - Deprecated Top-Level Delete Command

> [!WARNING]
> **Deprecated & Hidden (7.x Compatibility)**: Top-level `ccr delete <config_name> [--force]` is **Deprecated** and **Hidden** from `--help` in CCR `7.x`. Invoking it emits a deprecation warning to `stderr` (never polluting `stdout`), and it is scheduled for removal in `8.0`.

## Canonical Replacements

| Scenario | Canonical replacement |
|---|---|
| Delete a Claude profile | `ccr claude profile delete <name> [--force] [--json]` |
| Delete a Codex profile | `ccr codex profile delete <name> [--force] [--json]` |
| Delete a Grok profile | `ccr grok profile delete <name> [--force] [--json]` |

```bash
ccr claude profile delete temp-relay
ccr codex profile delete active-relay --force --json
```

## Key Behavioral Differences

- **Explicit platform scope**: Legacy `ccr delete` only targets the legacy Claude profile catalog; canonical commands explicitly target `claude`, `codex`, or `grok`.
- **Active-profile protection & runtime cleanup**: Legacy `ccr delete` left stale overrides in host runtime files after deleting the active profile; `ccr <platform> profile delete <name>` rejects deleting the active profile by default and, when `--force` is passed, atomically runs `profile off` to clean up the active runtime before removing the entry from `profiles.db`.

## Related Pages

- [`claude`](./claude)
- [`codex`](./codex)
- [`grok`](./grok)
