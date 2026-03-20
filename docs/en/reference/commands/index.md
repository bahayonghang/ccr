# Command Overview

CCR's CLI currently falls into five groups: platform and initialization, profiles and temporary overrides, data and sync, interfaces and cost controls, and extensions and maintenance.

## Command List

| Command | Purpose | Notes |
|---------|---------|-------|
| [`init`](./init) | Initialize the configuration root | Default entry for Unified Mode |
| [`platform`](./platform) | Manage the platform registry | list / switch / current / info / init |
| [`codex`](./codex) | Manage Codex multi-account auth | `ccr codex auth *` |
| [`migrate`](./migrate) | Legacy → Unified migration | Multi-platform migration entry |
| [`add`](./add) / [`delete`](./delete) | Create or remove profiles | Operates on the current platform |
| [`list`](./list) / [`current`](./current) / [`switch`](./switch) | Inspect or switch profiles | `ccr <name>` is the shortcut form of `switch` |
| [`temp`](./temp) / [`temp-token`](./temp-token) | Temporary overrides for the active settings | `temp` is interactive, `temp-token` is command-line driven |
| [`validate`](./validate) / [`enable`](./enable) / [`disable`](./disable) / [`clear`](./clear) / [`optimize`](./optimize) | Validate and tidy configuration | |
| [`history`](./history) / [`export`](./export) / [`import`](./import) / [`clean`](./clean) | Audit, export, import, cleanup | |
| [`sync`](./sync) | WebDAV sync | folder registry, push/pull/status |
| [`sessions`](./sessions) / [`provider`](./provider) / [`check`](./check) | Session search, provider health, conflict checks | Diagnostic command groups |
| [`ui`](./ui) / [`tui`](./tui) | Full UI and terminal UI | `ui` is the recommended graphical entry |
| [`stats`](./stats) / [`budget`](./budget) / [`pricing`](./pricing) | Cost and budget controls | Built on usage and pricing data |
| [`skills`](./skills) / [`prompts`](./prompts) | Extension management | |
| [`update`](./update) / [`version`](./version) | Version maintenance | |

## Recommended Starting Sequence

```bash
ccr init
ccr platform list
ccr add
ccr list
ccr switch <name>
ccr validate
```

If you prefer a browser entrypoint:

```bash
ccr ui -p 15173 --backend-port 38081
```

## Commands by Task

### Platform and init

- [`init`](./init)
- [`platform`](./platform)
- [`migrate`](./migrate)

### Profiles and overrides

- [`add`](./add)
- [`delete`](./delete)
- [`list`](./list)
- [`current`](./current)
- [`switch`](./switch)
- [`temp`](./temp)
- [`temp-token`](./temp-token)
- [`validate`](./validate)
- [`enable`](./enable)
- [`disable`](./disable)
- [`clear`](./clear)
- [`optimize`](./optimize)

### Data, sync, and diagnostics

- [`history`](./history)
- [`export`](./export)
- [`import`](./import)
- [`clean`](./clean)
- [`sync`](./sync)
- [`sessions`](./sessions)
- [`provider`](./provider)
- [`check`](./check)

### Interfaces and cost controls

- [`ui`](./ui)
- [`tui`](./tui)
- [`stats`](./stats)
- [`budget`](./budget)
- [`pricing`](./pricing)

### Extensions and maintenance

- [`codex`](./codex)
- [`skills`](./skills)
- [`prompts`](./prompts)
- [`update`](./update)
- [`version`](./version)

## Related Docs

- [CLI Workflows](/en/guide/cli-workflows)
- [UI Overview](/en/guide/ui-overview)
