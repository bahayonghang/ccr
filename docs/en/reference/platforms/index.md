# Platform Support

This page describes the current CCR CLI platform capability matrix. CLI platform support and CCR UI route registration are separate contracts.

## Support Matrix

| Platform | Status & Capability Tier | Authoritative Profile Store | Host Settings Target |
|---|---|---|---|
| [Claude Code](./claude) (`claude`) | Implemented (full CLI/TUI/UI `profile` + `auth`) | `~/.ccr/platforms/claude/profiles.db` | `~/.claude/settings.json`, `~/.claude.json` |
| [Codex](./codex) (`codex`) | Implemented (full CLI/TUI/UI `profile` + `auth`) | `~/.ccr/platforms/codex/profiles.db` | `~/.codex/config.toml`, `~/.codex/auth.json` |
| [Grok](/en/reference/commands/grok) (`grok`) | Implemented (full CLI/TUI/UI `profile` + `auth`) | `~/.ccr/platforms/grok/profiles.db` | `~/.grok/config.toml`, `~/.grok/user-settings.json` |
| [Antigravity CLI](./gemini) (`gemini`) | Retained compatibility adapter (`legacy_adapter`; no CLI/TUI `profile` or `auth` subcommands) | No CLI catalog switching surface | `~/.gemini/antigravity-cli/settings.json` |
| [Factory Droid](./droid) (`droid`) | Retained compatibility adapter (`legacy_adapter`; no CLI/TUI `profile` or `auth` subcommands) | No CLI catalog switching surface | `~/.factory/settings.json` |
| Qwen CLI (`qwen`) | Stub / not implemented | Reserved platform key, no pre-created catalog | Platform operations report unsupported |

- Antigravity CLI keeps the persisted `gemini` key and recognizes the input aliases `gemini-cli`, `antigravity`, `antigravity-cli`, and `agy`.
- Factory Droid uses the persisted `droid` key and recognizes the input aliases `droid` and `factory`.
- Claude, Codex, and Grok store their live profile catalogs in SQLite `profiles.db` (legacy `profiles.toml` files are migrated automatically on first access and backed up under `backups/<platform>/`). `profiles.toml` now serves as the `ExchangeV1` format for `profile export` / `import` and the editor projection for `profile open`; copying a backup TOML back onto the live path is not read by the 7.x binary.

## Current Command Boundary

```bash
# Inspect platform registry
ccr platform list
ccr platform list --json

# Explicit profile management on first-class platforms
ccr claude profile list
ccr codex profile list
ccr grok profile list

# Cross-platform runtime overview and unified diagnostics
ccr current
ccr doctor --all-platforms
```

The `ccr platform` subcommand tree exposes only `list` and `help`; all `profile` and `auth` routing operations use explicit `ccr <claude|codex|grok> profile ...` and `ccr <claude|codex|grok> auth ...` commands. For legacy subcommand replacements, see [Platform Command Migration](./migration).

## Platform Guides and Command References

- [Claude Code Platform Guide](./claude) (command reference: [`claude`](/en/reference/commands/claude))
- [Codex Platform Guide](./codex) (command reference: [`codex`](/en/reference/commands/codex))
- [Grok Command & Profile Reference](/en/reference/commands/grok)
- [Antigravity CLI Platform Guide](./gemini)
- [Factory Droid Platform Guide](./droid)
- [Platform Command Migration](./migration)

Qwen has only a reserved key and partial ecosystem data paths; do not treat it as a switchable runtime. The current CCR UI exposes dedicated platform workspaces for Claude, Codex, and Antigravity (Grok is supported across CLI/TUI and desktop command handlers, while OpenCode uses a standalone workspace entrypoint).

## Related Pages

- [`platform` command](/en/reference/commands/platform)
- [UI Module Map](/en/guide/ui-modules)
- [CLI Workflows](/en/guide/cli-workflows)
