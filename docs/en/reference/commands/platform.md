# platform - Platform Registry

Manage the platform registry (`~/.ccr/config.toml`): list, switch, inspect, and initialize platforms.

**Version**: v3.6.0+

## Subcommands

### list

List all supported platforms and highlight the current one.

```bash
ccr platform list [--json]
```

- `--json`: JSON output for scripting.

### switch

Switch the active platform (does not modify profiles; only updates the registry pointer).

```bash
ccr platform switch <platform>
```

Examples:

```bash
ccr platform switch codex
ccr platform switch gemini
```

### current

Show the current platform.

```bash
ccr platform current [--json]
```

### info

Show detailed info of a specific platform (status, paths, profiles).

```bash
ccr platform info <platform> [--json]
```

### init

Create directory structure and `profiles.toml` template for a platform (if missing).

```bash
ccr platform init <platform>
```

## Platform Keys

- Implemented: `claude`, `codex`, `gemini`
- Stub/Reserved: `qwen`, `iflow`

## Typical Workflow

```bash
ccr platform list
ccr platform switch claude
ccr add                     # add profile for current platform
ccr list && ccr switch xxx  # list/switch within current platform
```
