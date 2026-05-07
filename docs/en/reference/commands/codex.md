# codex - Codex Runtime and Multi-Account Management

`ccr codex` is the Codex-specific command group. Its main user-facing surfaces are:

- `ccr codex auth ...`: official-auth multi-account management
- `ccr codex profile ...`: runtime/profile routing
- `ccr codex sync-history ...`: history visibility repair after provider-namespace changes

## Common commands

```bash
ccr codex auth current
ccr codex auth list
ccr codex profile list
ccr codex profile switch <name>
ccr codex profile current
ccr codex profile off
```

## `auth` vs `profile`

| Command family | Purpose |
|---|---|
| `ccr codex auth ...` | save, switch, export, and import official auth accounts |
| `ccr codex profile ...` | apply a CCR profile into the Codex runtime or exit back to official-auth runtime |

## Current `profile` surface

- `list`
- `current`
- `switch <name>`
- `off`
- `create`
- `set-field`
- `enable`
- `disable`
- `delete`

## `sync-history`

Keeps its existing role: repairing old-history visibility after `openai` / `custom` namespace changes.
