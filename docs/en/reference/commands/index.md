# Command Reference

CCR commands are grouped by domain. All commands support `--help` for detailed usage.

## Command List

| Command | Aliases | Description | Version |
|---------|---------|-------------|---------|
| [init](./init) | - | Initialize config (Unified default, Legacy compatible) | v1.0+ |
| [platform](./platform) | - | Platform registry management (list/switch/current/info/init) | v3.6+ |
| [migrate](./migrate) | - | Legacy → Unified migration | v3.6+ |
| [list](./list) | `ls` | List profiles of current platform | v1.0+ |
| [current](./current) | `status`, `show` | Show current profile | v1.0+ |
| [switch](./switch) | - | Switch profile (supports shortcut `ccr <name>`) | v1.0+ |
| [add](./add) | - | Add new profile interactively | v1.0+ |
| [delete](./delete) | - | Delete profile | v1.0+ |
| enable | - | Enable profile (current platform) | v1.0+ |
| disable | - | Disable profile (`--force` supported) | v1.0+ |
| [validate](./validate) | `check` | Validate config and settings | v1.0+ |
| optimize | - | Reorder profiles | v1.0+ |
| clear | - | Clear CCR writes from settings.json | v2.0+ |
| [temp-token](./temp-token) | - | Temporary token/base_url/model override | v2.0+ |
| [history](./history) | - | Operation history | v1.0+ |
| [stats](./stats) | - | Cost/usage stats (web feature) | v2.0+ |
| [budget](./budget) | - | Budgeting (web feature) | v3.16+ |
| [pricing](./pricing) | - | Model pricing (web feature) | v3.16+ |
| [export](./export) | - | Export configuration | v1.0+ |
| [import](./import) | - | Import configuration (merge/replace) | v1.0+ |
| [clean](./clean) | - | Clean old backups | v2.0+ |
| [sync](./sync) | - | WebDAV sync (folder registry/batch/interactive) | v2.0+ |
| [ui](./ui) | - | Launch full CCR UI | v1.4+ |
| [tui](./tui) | - | Terminal UI (tui feature) | v2.0+ |
| [web](./web) | - | Lightweight Web API (compatibility/scripts) | v2.0+ |
| [skills](./skills) | - | Skills management | v3.5+ |
| [prompts](./prompts) | - | Prompt preset management | v3.5+ |
| [check](./check) | - | Config conflict detection | v3.6+ |
| [update](./update) | - | Self update | v1.0+ |
| [version](./version) | `ver` | Show version info | v1.0+ |

## Categories

### Init & Platform

- **[init](./init)** - Initialize (Unified by default)
- **[platform](./platform)** - Platform registry management
- **[migrate](./migrate)** - Legacy → Unified migration

### Profile Management

- **[list](./list)** / **[current](./current)** / **[switch](./switch)** - View & switch
- **[add](./add)** / **[delete](./delete)** - CRUD profiles
- enable/disable - Enable or disable profiles
- **[validate](./validate)** / optimize / clear - Validate, reorder, clear writes
- **[temp-token](./temp-token)** - Temporary token/base_url/model override

### Data & Sync

- **[export](./export)** / **[import](./import)** / **[clean](./clean)** - Export/import/cleanup
- **[sync](./sync)** - WebDAV sync (registry, batch/single, interactive filter)
- **[history](./history)** / **[stats](./stats)** - Audit & metrics
- **[budget](./budget)** / **[pricing](./pricing)** - Budgeting & model pricing (web feature)

### Interfaces

- **[ui](./ui)** - Full CCR UI
- **[tui](./tui)** - Terminal UI (feature gated)
- **[web](./web)** - Lightweight Web API

### Extensions & Maintenance

- **[skills](./skills)** / **[prompts](./prompts)** - Extensions management
- **[check](./check)** - Conflict detection
- **[update](./update)** / **[version](./version)** - Updates & version info

## Quick Snippets

```bash
# Init & platform
ccr init
ccr platform list
ccr platform switch codex

# Profile lifecycle
ccr add && ccr list && ccr switch <name>
ccr validate
ccr export --no-secrets
ccr import configs.toml --merge

# Sync
ccr sync config
ccr sync folder add claude ~/.claude
ccr sync push -i

# Launch CCR UI (full web app)
ccr ui

# Launch legacy Web API (scripting/CI)
ccr web --port 8080

# Budget & pricing (web feature)
ccr budget status
ccr pricing list --verbose
