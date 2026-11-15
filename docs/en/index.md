---
layout: home

hero:
  name: "CCR"
  text: "Unified management for AI CLI configuration and runtime state"
  tagline: "CLI-first, explicit Claude / Codex / Grok runtimes backed by SQLite catalogs, with TUI and CCR UI"
  image:
    src: /logo.svg
    alt: CCR
  actions:
    - theme: brand
      text: Quick Start
      link: /en/guide/quick-start
    - theme: alt
      text: CLI Workflows
      link: /en/guide/cli-workflows
    - theme: alt
      text: CCR UI
      link: /en/guide/ui-overview
---

<script setup>
const choosePaths = [
  {
    icon: '⚡',
    title: 'Start with the CLI',
    details: 'Day-to-day runtime work centers on ccr current and ccr <claude|codex|grok> profile.',
    link: '/en/guide/cli-workflows'
  },
  {
    icon: '🖥️',
    title: 'Use CCR UI visually',
    details: 'The Tauri UI shares the same registry, profiles.db catalogs, history, and backups.',
    link: '/en/guide/ui-overview'
  }
]

const capabilityCards = [
  {
    icon: '🔀',
    title: 'Explicit multi-runtime model',
    details: 'Inspect active runtime state with ccr current instead of relying on an implicit global platform switch.',
    link: '/en/reference/commands/current'
  },
  {
    icon: '🔐',
    title: 'Platform-scoped profile & auth commands',
    details: 'Use ccr <claude|codex|grok> profile ... and auth ... for runtime routing and official accounts.',
    link: '/en/reference/commands/'
  },
  {
    icon: '☁️',
    title: 'WebDAV sync',
    details: 'Folder registration plus single-folder and batch push/pull/status flows.',
    link: '/en/reference/commands/sync'
  },
  {
    icon: '📚',
    title: 'Doctor / Sessions / Skills',
    details: 'Unified read-only diagnostics, session indexing, provider health, and extensions share one CLI.',
    link: '/en/reference/commands/doctor'
  },
  {
    icon: '🛡️',
    title: 'SQLite catalogs & safe writes',
    details: 'Live profiles.db storage paired with ExchangeV1 TOML exchange, file locking, atomic writes, and backups.',
    link: '/en/guide/configuration'
  },
  {
    icon: '🏗️',
    title: 'Architecture and migration references',
    details: 'Workspace layering, runtime flows, and 7.x command/storage migration mappings are documented together.',
    link: '/en/reference/migration'
  }
]
</script>

<HomeFeatures badge="Choose Your Path" title="How To Use CCR" :features="choosePaths" />
<HomeFeatures badge="Capability Map" badge-type="info" title="What This Project Covers" :features="capabilityCards" />

## Five-Minute Start

```bash
ccr claude profile init
printf '%s' "$ANTHROPIC_AUTH_TOKEN" | ccr claude profile create work \
  --base-url https://api.example.com \
  --auth-token-stdin
ccr claude profile list
ccr claude profile switch work
ccr current
ccr validate
```

## Support Matrix

| Platform | Status | Notes |
|----------|--------|-------|
| Claude Code (`claude`) | ✅ Implemented | dual `auth` + `profile` runtime surface backed by `profiles.db` |
| Codex (`codex`) | ✅ Implemented | `auth`, `profile`, `fix`, and `sync-history` backed by `profiles.db` |
| Grok (`grok`) | ✅ Implemented | full CLI/TUI/UI `auth` + `profile` support backed by `profiles.db` |
| Antigravity CLI (`gemini`) | 🔄 Compatibility Adapter | `legacy_adapter` (no CLI `profile`/`auth` subcommands); supports `doctor`, ecosystem tools, and `/antigravity` UI |
| Factory Droid (`droid`) | 🔄 Compatibility Adapter | `legacy_adapter` (no CLI `profile`/`auth` subcommands); supports `doctor` and ecosystem tools |
| Qwen CLI (`qwen`) | 🚧 Reserved / Stub | reserved platform key plus partial data-domain support |

## Common Entry Points

```bash
ccr current --verbose
ccr codex auth current
ccr grok profile list
ccr doctor
ccr ui -p 15173 --backend-port 38081
```
