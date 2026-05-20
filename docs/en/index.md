---
layout: home

hero:
  name: "CCR"
  text: "Unified management for AI CLI configuration and runtime state"
  tagline: "CLI-first, explicit Claude Runtime / Codex Runtime, with TUI and CCR UI"
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
    details: 'Day-to-day runtime work centers on ccr current, ccr claude profile, and ccr codex profile.',
    link: '/en/guide/cli-workflows'
  },
  {
    icon: '🖥️',
    title: 'Use CCR UI visually',
    details: 'The Vue + Tauri UI shares the same registry, profiles, history, and backups.',
    link: '/en/guide/ui-overview'
  }
]

const capabilityCards = [
  {
    icon: '🔀',
    title: 'Explicit dual runtime model',
    details: 'Claude and Codex are shown side by side instead of relying on a global current_platform switch.',
    link: '/en/reference/commands/current'
  },
  {
    icon: '🔐',
    title: 'Platform-scoped profile commands',
    details: 'Use ccr claude profile ... and ccr codex profile ... for runtime routing.',
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
    title: 'Sessions / Provider / Skills',
    details: 'Session indexing, provider health, and extension surfaces share one CLI.',
    link: '/en/reference/commands/'
  },
  {
    icon: '🛡️',
    title: 'Safe writes',
    details: 'File locking, atomic writes, backups, and audit history protect configuration changes.',
    link: '/en/guide/configuration'
  },
  {
    icon: '🏗️',
    title: 'Architecture and migration references',
    details: 'Workspace layering, runtime flows, and migration mappings are documented together.',
    link: '/en/reference/migration'
  }
]
</script>

<HomeFeatures badge="Choose Your Path" title="How To Use CCR" :features="choosePaths" />
<HomeFeatures badge="Capability Map" badge-type="info" title="What This Project Covers" :features="capabilityCards" />

## Five-Minute Start

```bash
ccr init
ccr current
ccr add
ccr claude profile list
ccr claude profile switch <name>
ccr validate
```

## Support Matrix

| Platform | Status | Notes |
|----------|--------|-------|
| Claude Code | ✅ Implemented | dual official-auth + profile runtime surface |
| Codex | ✅ Implemented | auth, profile, and sync-history all supported |
| Antigravity CLI | ✅ Implemented | internal key remains `gemini`; legacy Gemini session import stays compatible |
| Factory Droid | ✅ Implemented | still present in the broader platform domain |
| Qwen CLI | 🚧 Reserved / Partial | reserved key plus partial data-domain support |

## Common Entry Points

```bash
ccr current --verbose
ccr codex auth current
ccr codex profile list
ccr ui -p 15173 --backend-port 38081
```
