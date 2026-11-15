# `platform` - Platform Registry View and Migration Map

`ccr platform` provides only a read-only registry listing and help entrypoint. Platform switching, catalog initialization, and profile CRUD/exchange live exclusively under explicit per-platform command groups.

## Supported Subcommands

```bash
ccr platform list
ccr platform list --json
ccr platform help
```

- `ccr platform list [--json]`: list recognized AI CLI platforms, their registry enabled status, and each platform's recorded `current_profile` (supports `--json` for scripts and integrations).
- `ccr platform help` (or `ccr platform --help`): display help for `ccr platform`.

## Removed Subcommands

The former `switch`, `current`, `info`, `init`, and `profile` subcommands under `platform` have been removed from the Clap parser altogether. Invoking any of those removed subcommands now produces a standard Clap unknown-subcommand parse error (with a non-zero exit code) rather than custom runtime migration guidance.

## Complete Migration Map (All 13 `profile` Actions)

Use the explicit platform-scoped commands on `claude`, `codex`, or `grok` instead:

| Removed `platform` subcommand form | Canonical replacement |
|---|---|
| Removed `platform` subcommand `info <platform>` | `ccr platform list [--json]` or `ccr <claude\|codex\|grok> profile current [--json]` |
| Removed `platform` subcommand `current` | `ccr current [--verbose \| --json]` (aliases `ccr status` / `ccr show`) |
| Removed `platform` subcommand `switch <platform>` | `ccr <claude\|codex\|grok> profile switch <name> [--json]` or `ccr <claude\|codex\|grok> auth switch <name>` |
| Removed `platform` subcommand `init <platform>` | `ccr <claude\|codex\|grok> profile init [--json]` |
| Removed `platform` subcommand `profile open <platform>` | `ccr <claude\|codex\|grok> profile open [--json]` |
| Removed `platform` subcommand `profile current <platform>` | `ccr <claude\|codex\|grok> profile current [--json]` |
| Removed `platform` subcommand `profile list <platform>` | `ccr <claude\|codex\|grok> profile list [--json]` |
| Removed `platform` subcommand `profile switch <platform> <name>` | `ccr <claude\|codex\|grok> profile switch <name> [--json]` |
| Removed `platform` subcommand `profile create <platform> <name> ...` | `ccr <claude\|codex\|grok> profile create <name> [...] [--json]` |
| Removed `platform` subcommand `profile edit <platform> <name> ...` | `ccr <claude\|codex\|grok> profile edit <name> [...] [--json]` |
| Removed `platform` subcommand `profile enable <platform> <name>` | `ccr <claude\|codex\|grok> profile enable <name> [--json]` |
| Removed `platform` subcommand `profile disable <platform> <name>` | `ccr <claude\|codex\|grok> profile disable <name> [--force] [--json]` |
| Removed `platform` subcommand `profile delete <platform> <name>` | `ccr <claude\|codex\|grok> profile delete <name> [--force] [--json]` |
| Removed `platform` subcommand `profile off <platform>` | `ccr <claude\|codex\|grok> profile off [--json]` |
| Removed `platform` subcommand `profile export <platform> ...` | `ccr <claude\|codex\|grok> profile export [NAME]... [...] [--json]` |
| Removed `platform` subcommand `profile import <platform> ...` | `ccr <claude\|codex\|grok> profile import [<FILE>\|-] [...] [--json]` |

## Registry Compatibility Notes

- Legacy `default_platform` and `current_platform` keys in older `~/.ccr/config.toml` files remain readable for backward compatibility.
- Authoritative auth and profile routing truth lives in each platform's own `profiles.db` catalog and host runtime configuration.

## Related Pages

- [`claude`](./claude) / [`codex`](./codex) / [`grok`](./grok)
- [`current`](./current)
- [CLI Workflows](/en/guide/cli-workflows)
- [Migration Guide](/en/reference/migration)
