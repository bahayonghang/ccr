# Configuration Model

This page explains CCR's current configuration modes, directory layout, platform status, and shared resources.

## Modes

| Mode | When to use it | Primary location | Notes |
|------|----------------|------------------|-------|
| Unified Mode | Multi-platform workflows, recommended default | `~/.ccr/` | Platform registry, profiles, history, backups, and logs are organized by platform. |
| Legacy Mode | CCS compatibility, single-platform Claude workflows | `~/.ccs_config.toml` | Keeps the old single-file configuration path. |

Detection order:

1. `CCR_ROOT` is set
2. `~/.ccr/config.toml` exists
3. Fall back to Legacy Mode

## Unified Mode Layout

```text
~/.ccr/
├── config.toml
├── platforms/
│   ├── claude/
│   ├── codex/
│   ├── gemini/
│   ├── droid/
│   ├── qwen/
├── history/
├── backups/
├── logs/
└── ccr-ui/
```

Key points:

- `config.toml`: platform registry and current-platform pointer.
- `platforms/<name>/profiles.toml`: profile set for that platform.
- `history/` and `backups/`: global records and rollback assets.
- `ccr-ui/`: downloaded or cached UI project used by `ccr ui`.

## Platform Status

| Platform | Status | Profile file | Settings target |
|----------|--------|--------------|-----------------|
| Claude | Implemented | `~/.ccr/platforms/claude/profiles.toml` | `~/.claude/settings.json` |
| Codex | Implemented | `~/.ccr/platforms/codex/profiles.toml` | `~/.codex/config.toml` |
| Gemini | Implemented | `~/.ccr/platforms/gemini/profiles.toml` | `~/.ccr/platforms/gemini/settings.json` |
| Droid | Implemented | `~/.ccr/platforms/droid/profiles.toml` | `~/.factory/settings.json` |
| Qwen | Reserved / Stub | `~/.ccr/platforms/qwen/profiles.toml` | `~/.ccr/platforms/qwen/settings.json` |
> Qwen remains a reserved/stub platform until the core implementation says otherwise.

## Common Lifecycle

### Initialize and switch platforms

```bash
ccr init
ccr platform list
ccr platform switch claude
```

### Manage profiles

```bash
ccr add
ccr list
ccr switch <name>
ccr enable <name>
ccr disable <name> --force
```

### Validate, inspect history, and clean up

```bash
ccr validate
ccr history --limit 20
ccr optimize
ccr clean backups --days 30 --dry-run
```

### Import, export, and restore

```bash
ccr export -o configs.toml --no-secrets
ccr import configs.toml --merge --backup
```

## Temporary Overrides and Immediate Writes

CCR separates three write paths:

- `ccr switch`: reads a profile and writes the target settings.
- `ccr temp`: interactively writes a temporary configuration without relying on an existing profile.
- `ccr temp-token`: applies command-line token / base URL / model overrides to the active settings.

Those commands do not change CLI defaults, but they do change the currently active settings file.

## Relationship to CCR UI

- CLI and `ccr-ui` share the same `~/.ccr/` data source.
- The UI can expose more surfaces than the CLI, but it must not become a second source of truth.
- Platform status, defaults, and API routes still come from the codebase definitions.

## Related Docs

- [CLI Workflows](/en/guide/cli-workflows)
- [UI Overview](/en/guide/ui-overview)
- [Platform Support](/en/reference/platforms/)
