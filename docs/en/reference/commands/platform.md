# platform - Registry Compatibility View

`ccr platform` now acts as a registry-facing compatibility surface rather than the main auth/profile routing path.

## Recommended current usage

```bash
ccr platform list
ccr current
```

- `ccr platform list`: inspect registry entries, enabled state, and each platform's `current_profile`
- `ccr current`: inspect real Claude Runtime / Codex Runtime state

## Retired subcommands

These subcommands now return migration guidance instead of acting as the main path:

- `ccr platform switch <platform>`
- `ccr platform current`
- `ccr platform info <platform>`
- `ccr platform init <platform>`
- `ccr platform profile ...`

## Migration map

| Legacy path | Current path |
|---|---|
| `ccr platform switch claude` | `ccr claude profile switch <name>` or `ccr claude auth ...` |
| `ccr platform switch codex` | `ccr codex profile switch <name>` or `ccr codex auth ...` |
| `ccr platform current` | `ccr current` |
| `ccr platform profile create claude ...` | `ccr claude profile create ...` |
| `ccr platform profile create codex ...` | `ccr codex profile create ...` |

## Notes

- old `default_platform` / `current_platform` fields can still be read from older registries
- the routing truth is now each platform's own `current_profile`
