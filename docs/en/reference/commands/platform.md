# platform - Platform Registry

Manage platform state, the current-platform pointer, and platform initialization in `~/.ccr/config.toml`.

## Usage

```bash
ccr platform <ACTION> [OPTIONS]
```

## Subcommands

### list

```bash
ccr platform list [--json]
```

Lists known platforms and their status.

### switch

```bash
ccr platform switch <platform>
```

Switches the active platform without modifying profiles on other platforms.

### current

```bash
ccr platform current [--json]
```

Shows the active platform.

### info

```bash
ccr platform info <platform> [--json]
```

Shows status, paths, and descriptive metadata for the requested platform.

### init

```bash
ccr platform init <platform>
```

Creates the directory structure and template files for a platform.

## Current Platform Keys

| Key | Status | Notes |
|-----|--------|-------|
| `claude` | Implemented | Mainline default platform |
| `codex` | Implemented | Also exposes `ccr codex auth` |
| `gemini` | Implemented | Managed in Unified Mode |
| `droid` | Implemented | Writes to `~/.factory/settings.json` |
| `qwen` | Reserved / Stub | Core implementation currently reports unsupported |
| `iflow` | Reserved / Stub | Core implementation currently reports unsupported |

## Common Examples

```bash
ccr platform list
ccr platform switch claude
ccr platform info droid
ccr platform init gemini
```

## Related Docs

- [Platform Support](/en/reference/platforms/)
- [Quick Start](/en/guide/quick-start)
