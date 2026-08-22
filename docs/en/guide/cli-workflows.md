# CLI Workflows

This page groups CCR into task-oriented CLI flows. The recommended auth/profile surface is now explicit Claude Runtime and Codex Runtime state, not the old global “current platform” mental model.

## Workflow 1: Initialize and inspect runtime state

```bash
ccr init
ccr current
ccr platform list
```

Use this for:
- first-run setup of `~/.ccr/`
- checking whether Claude and Codex runtimes are ready
- inspecting registry entries and per-platform `current_profile`

> `ccr platform list` is now a registry compatibility view. Use `ccr current` for actual runtime state.

## Workflow 2: Claude runtime / profile

```bash
ccr add
ccr claude profile list
ccr claude profile switch <name>
ccr claude profile current
ccr claude profile off
```

## Workflow 3: Codex runtime / profile

```bash
ccr codex auth current
ccr codex profile list
ccr codex profile switch <name>
ccr codex profile current
ccr codex profile off
```

## Workflow 4: Validate and diagnose

```bash
ccr current --verbose
ccr validate
ccr doctor
```

## Workflow 5: Sync, history, and cleanup

```bash
ccr history -l 50
ccr sync config
ccr sync all status
ccr clean backups --days 30 --dry-run
```

## Workflow 6: Codex multi-account auth

```bash
ccr codex auth save work
ccr codex auth list
ccr codex auth switch work
ccr codex auth off
```

## Workflow 7: Move into the graphical surface

```bash
ccr ui -p 15173 --backend-port 38081
ccr
```

## Migration quick map

| Legacy command | Current path |
|---|---|
| `ccr switch <name>` | `ccr claude profile switch <name>` or `ccr codex profile switch <name>` |
| `ccr <name>` | same mapping; the shortcut is retired |
| `ccr platform switch <platform>` | retired for auth/profile routing |
| `ccr platform current` | `ccr current` |
| `ccr platform profile ...` | `ccr claude profile ...` / `ccr codex profile ...` |

## Related docs

- [Quick Start](/en/guide/quick-start)
- [Configuration Model](/en/guide/configuration)
- [Entrypoints](/en/guide/entrypoints)
- [Command Reference](/en/reference/commands/)
