# Platform Support

This page is the single platform-status reference for CCR. Home pages, command docs, and platform guides should agree with it.

## Support Matrix

| Platform | Status | Profile file | Settings target | Notes |
|----------|--------|--------------|-----------------|-------|
| Claude Code | Implemented | `~/.ccr/platforms/claude/profiles.toml` | `~/.claude/settings.json` | default primary platform |
| Codex | Implemented | `~/.ccr/platforms/codex/profiles.toml` | `~/.codex/config.toml` | also exposes `ccr codex auth` |
| Gemini CLI | Implemented | `~/.ccr/platforms/gemini/profiles.toml` | `~/.ccr/platforms/gemini/settings.json` | managed in Unified Mode |
| Factory Droid | Implemented | `~/.ccr/platforms/droid/profiles.toml` | `~/.factory/settings.json` | separate settings structure |
| Qwen CLI | Reserved / Stub | `~/.ccr/platforms/qwen/profiles.toml` | `~/.ccr/platforms/qwen/settings.json` | core implementation currently reports unsupported |
| iFlow CLI | Reserved / Stub | `~/.ccr/platforms/iflow/profiles.toml` | `~/.ccr/platforms/iflow/settings.json` | core implementation currently reports unsupported |

> Platform status follows `Platform::is_implemented()` and the concrete platform implementation, not merely whether a UI entry exists.

## Quick Commands

```bash
ccr platform list
ccr platform switch claude
ccr platform info droid
ccr platform init gemini
```

## Implemented Platform Guides

- [Claude Code](./claude)
- [Codex](./codex)
- [Gemini CLI](./gemini)
- [Factory Droid](./droid)

## Reserved Platforms

- `qwen`
- `iflow`

Today that means:

- the keys already exist in Unified Mode
- the UI may expose reserved module groups
- the docs keep stable locations for future support

## Related Docs

- [platform command](/en/reference/commands/platform)
- [CLI Workflows](/en/guide/cli-workflows)
- [Migration Guide](/en/reference/migration)
- [Multi-Platform Setup](/en/examples/multi-platform-setup)
