# Platform Support

This page describes the current CLI platform domain. CLI support and CCR UI route registration are separate contracts.

## Support Matrix

| Platform | Status | Profile file | Settings target |
|---|---|---|---|
| Claude Code | Implemented | `~/.ccr/platforms/claude/profiles.toml` | `~/.claude/settings.json` |
| Codex | Implemented | `~/.ccr/platforms/codex/profiles.toml` | `~/.codex/config.toml` |
| Antigravity CLI | Implemented | `~/.ccr/platforms/gemini/profiles.toml` | `~/.gemini/antigravity-cli/settings.json` |
| Factory Droid | Implemented | `~/.ccr/platforms/droid/profiles.toml` | `~/.factory/settings.json` |
| Qwen CLI | Stub / not implemented | reserved platform directory | platform operations report unsupported |

Antigravity keeps the persisted `gemini` key; `agy` and `antigravity` are input aliases.

## Current Command Boundary

```bash
ccr platform list
ccr platform list --json

ccr claude profile list
ccr codex profile list
ccr current
```

`platform switch`, `current`, `info`, `init`, and `profile` remain parseable so CCR can return explicit migration errors; they are not current execution paths. Claude and Codex use explicit commands. Confirm other platform state through `platform list`, configuration, and the concrete implementation.

## Platform Guides

- [Claude Code](./claude)
- [Codex](./codex)
- [Antigravity CLI](./gemini)
- [Factory Droid](./droid)
- [Platform Command Migration](./migration)

Qwen has only a reserved key and partial data paths; do not present it as a switchable runtime. The current UI has platform workspaces for Claude, Codex, and Antigravity; OpenCode uses a separate tool entrypoint.

## Related Pages

- [`platform` command](/en/reference/commands/platform)
- [UI Module Map](/en/guide/ui-modules)
- [CLI Workflows](/en/guide/cli-workflows)
