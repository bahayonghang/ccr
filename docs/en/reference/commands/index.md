# Command Overview

CCR's CLI now centers on five groups: runtime overview, platform-scoped profile/auth flows, data and sync, diagnostics and interfaces, and extensions and maintenance.

## Recommended starting sequence

```bash
ccr init
ccr current
ccr add
ccr claude profile list
ccr claude profile switch <name>
ccr validate
```

## Main current paths

| Path | Purpose |
|---|---|
| [`current`](./current) | dual Claude Runtime / Codex Runtime overview |
| [`codex`](./codex) | Codex auth, profile, and sync-history |
| `ccr claude profile ...` | Claude runtime/profile routing |
| [`platform`](./platform) | registry compatibility view (mainly `list`) |
| [`validate`](./validate) / [`doctor`](./doctor) | validation and diagnostics based on explicit runtime state |

## Migration quick map

| Legacy command | Current path |
|---|---|
| `ccr switch <name>` | `ccr claude profile switch <name>` or `ccr codex profile switch <name>` |
| `ccr <name>` | shortcut retired |
| `ccr platform switch <platform>` | retired |
| `ccr platform current` | `ccr current` |
| `ccr platform profile ...` | `ccr claude profile ...` / `ccr codex profile ...` |

## Related docs

- [CLI Workflows](/en/guide/cli-workflows)
- [Configuration Model](/en/guide/configuration)
- [Migration Guide](/en/reference/migration)
