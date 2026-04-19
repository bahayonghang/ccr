# Command Overview

CCR's current CLI falls into five groups: platform and initialization, profiles and temporary overrides, data and sync, interfaces and cost controls, and extensions and maintenance.

## Command List

| Command | Purpose | Notes |
|---|---|---|
| [`init`](./init) | Initialize the configuration root | entrypoint for the current layout |
| [`platform`](./platform) | Manage the platform registry | list / switch / current / info / init |
| [`codex`](./codex) | Manage Codex multi-account auth | `ccr codex auth *` |
| [`opencode`](./opencode) | OpenCode auth migration and entrypoint | `ccr opencode` / `ccr opencode auth import-codex` |
| [`add`](./add) / [`delete`](./delete) | Create or remove profiles | operates on the current platform |
| [`list`](./list) / [`current`](./current) / [`switch`](./switch) | Inspect or switch profiles | `ccr <name>` is the shortcut form of `switch` |
| [`temp`](./temp) / [`temp-token`](./temp-token) | Temporary overrides for the active settings | `temp` is interactive, `temp-token` is command-line driven |
| [`validate`](./validate) / [`enable`](./enable) / [`disable`](./disable) / [`clear`](./clear) / [`optimize`](./optimize) | Validate and tidy configuration | |
| [`history`](./history) / [`export`](./export) / [`import`](./import) / [`clean`](./clean) | Audit, export, import, cleanup | |
| [`sync`](./sync) | WebDAV sync | folder registry, push/pull/status |
| [`sessions`](./sessions) / [`provider`](./provider) / [`check`](./check) | Session search, provider health, conflict checks | diagnostic command groups |
| [`ui`](./ui) / [`tui`](./tui) | Graphical UI and terminal interactive mode | `ui` is the recommended graphical entry; the `tui` page explains the bare `ccr` behavior |
| [`stats`](./stats) / [`budget`](./budget) / [`pricing`](./pricing) | Cost and budget controls | built on usage and pricing data |
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

If you prefer the graphical entrypoint:

```bash
ccr ui -p 15173 --backend-port 38081
```

If you prefer the terminal interactive mode:

```bash
ccr
```

## Global Help and Version

```bash
ccr --help
ccr help platform
ccr help codex auth
ccr help opencode auth
ccr --version
ccr version
```

- `ccr --help` / `ccr help ...`: task-oriented help
- `ccr --version`: short version string for scripts and CI
- `ccr version`: detailed version summary for humans

## Commands by Task

### Platform and init

- [`init`](./init)
- [`platform`](./platform)
- [`codex`](./codex)
- [`opencode`](./opencode)

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

- [`skills`](./skills)
- [`prompts`](./prompts)
- [`update`](./update)
- [`version`](./version)

## Related Docs

- [CLI Workflows](/en/guide/cli-workflows)
- [Entrypoints](/en/guide/entrypoints)
- [UI Overview](/en/guide/ui-overview)
