# Configuration Model

This page explains CCR's current configuration modes, directory layout, and the new runtime/profile source of truth.

## Modes

| Mode | When to use it | Primary location | Notes |
|------|----------------|------------------|-------|
| Unified Mode | Multi-platform workflows, recommended default | `~/.ccr/` | Registry, profiles, history, backups, and logs are grouped by platform. |
| Legacy Mode | Old CCS compatibility, single-platform Claude flows | `~/.ccs_config.toml` | Keeps the historical single-file path. |

Detection order:

1. `CCR_ROOT` is set
2. `~/.ccr/config.toml` exists
3. fall back to Legacy Mode

## Unified layout

```text
~/.ccr/
├── config.toml
├── platforms/
│   ├── claude/
│   ├── codex/
│   ├── gemini/
│   ├── droid/
│   └── qwen/
├── history/
├── backups/
├── logs/
└── ccr-ui/
```

Key points:

- `config.toml`: platform registry; the routing truth now lives in each platform entry's `current_profile`.
- `platforms/<name>/profiles.toml`: the profile set for that platform.
- `history/` / `backups/`: audit and rollback assets.
- `ccr-ui/`: frontend/runtime assets used by `ccr ui`.

## Runtime/profile source of truth

Auth/profile routing has moved to explicit command families:

- `ccr claude auth ...` / `ccr codex auth ...`: official-auth account surface
- `ccr claude profile ...` / `ccr codex profile ...`: runtime/profile routing surface
- `ccr current`: parallel Claude Runtime + Codex Runtime overview

Inside `~/.ccr/config.toml`, the per-platform `current_profile` field is the routing truth.

Older registries may still contain `default_platform` / `current_platform`, but they are no longer the auth/profile routing truth.

## Platform status

| Platform | Status | Profile file | Settings target |
|----------|--------|--------------|-----------------|
| Claude | Implemented | `~/.ccr/platforms/claude/profiles.toml` | `~/.claude/settings.json` |
| Codex | Implemented | `~/.ccr/platforms/codex/profiles.toml` | `~/.codex/config.toml` |
| Gemini | Implemented | `~/.ccr/platforms/gemini/profiles.toml` | `~/.ccr/platforms/gemini/settings.json` |
| Droid | Implemented | `~/.ccr/platforms/droid/profiles.toml` | `~/.factory/settings.json` |
| Qwen | Reserved / Stub | `~/.ccr/platforms/qwen/profiles.toml` | `~/.ccr/platforms/qwen/settings.json` |

## Common operations

```bash
ccr init
ccr current
ccr platform list
ccr add
ccr claude profile switch <name>
ccr codex profile switch <name>
ccr validate
ccr doctor
```

## Relationship to CCR UI and VS Code

- CLI, `ccr-ui`, and `ccr-vscode` share the same `~/.ccr/` registry and profile files.
- The dual-runtime model shown by `ccr current` is also the model UI and VS Code should reflect.
- `ccr platform list` remains a compatibility registry view rather than a global active-platform switch.

## Related docs

- [CLI Workflows](/en/guide/cli-workflows)
- [Quick Start](/en/guide/quick-start)
- [Platform Support](/en/reference/platforms/)
