---
layout: home

hero:
  name: "CCR"
  text: "Unified entrypoint for AI CLI configuration management"
  tagline: "CLI-first workflow, with TUI and the full CCR UI"
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
    details: 'Use CCR as the main entrypoint for profile lifecycle, platform switching, sync, budget, history, and sessions.',
    link: '/en/guide/cli-workflows'
  },
  {
    icon: '🖥️',
    title: 'Use CCR UI for visual workflows',
    details: 'CCR UI is the recommended visual entrypoint for module browsing, dashboards, and platform-oriented management.',
    link: '/en/guide/ui-overview'
  }
]

const capabilityCards = [
  {
    icon: '🔀',
    title: 'Unified platform registry',
    details: 'Manage isolated profiles, history, and backups across claude, codex, gemini, droid, and reserved platform keys.',
    link: '/en/reference/platforms/'
  },
  {
    icon: '☁️',
    title: 'WebDAV sync',
    details: 'Folder registry, batch push/pull, and interactive selection for multi-machine config sync.',
    link: '/en/reference/commands/sync'
  },
  {
    icon: '📚',
    title: 'Sessions / Provider / Skills',
    details: 'CCR groups session indexing, provider health checks, skills, and prompts into the same operational CLI surface.',
    link: '/en/reference/commands/'
  },
  {
    icon: '📊',
    title: 'Cost controls',
    details: 'Stats, pricing, and budget commands share one reporting model for spend-aware workflows.',
    link: '/en/reference/commands/stats'
  },
  {
    icon: '🛡️',
    title: 'Safe writes',
    details: 'Atomic writes, locking, and backups reduce risk during settings updates and imports.',
    link: '/en/guide/quick-start'
  },
  {
    icon: '🏗️',
    title: 'Architecture and integration',
    details: 'Use the architecture and API reference to understand workspace layout and legacy HTTP routes.',
    link: '/en/reference/architecture'
  }
]
</script>

<HomeFeatures badge="Choose Your Path" title="How To Use CCR" :features="choosePaths" />
<HomeFeatures badge="Capability Map" badge-type="info" title="What This Project Covers" :features="capabilityCards" />

## Quick Install
- Rust 1.90+
- Optional: Node.js 18+ and Bun 1.0+ when developing `ccr-ui`
- Recommended: `just`

```bash
cargo install --git https://github.com/bahayonghang/ccr ccr
```

Source install and workspace layout live in [Quick Start](/en/guide/quick-start).

## Five-Minute Start

```bash
ccr init
ccr platform list
ccr add
ccr list
ccr switch <name>
ccr validate
```

Then continue with:
- [CLI Workflows](/en/guide/cli-workflows)
- [UI Overview](/en/guide/ui-overview)
- [Command Reference](/en/reference/commands/)

## Support Matrix

| Platform | Status | Notes |
|----------|--------|-------|
| Claude Code | ✅ Implemented | Default primary platform with direct settings writes |
| Codex | ✅ Implemented | Profile, auth, and MCP-oriented workflows |
| Gemini CLI | ✅ Implemented | Isolated profile, history, and backup structure |
| Factory Droid | ✅ Implemented | Present in both CLI platform docs and CCR UI modules |
| Qwen CLI | 🚧 Reserved / Partial | Platform key and UI grouping exist; treat as reserved/partial in docs |
| iFlow CLI | 🚧 Reserved / Partial | Platform key and UI grouping exist; treat as reserved/partial in docs |

See [Platform Support](/en/reference/platforms/) for the detailed matrix.

## Common Entry Points

```bash
ccr ui -p 15173 --backend-port 38081
ccr sync config
ccr sessions list
ccr provider test --all
ccr stats summary --range week --details
```

## Documentation Map
- [Quick Start](/en/guide/quick-start)
- [CLI Workflows](/en/guide/cli-workflows)
- [UI Overview](/en/guide/ui-overview)
- [UI Modules](/en/guide/ui-modules)
- [Architecture](/en/reference/architecture)

## License
MIT License

## Contributing
Issues and Pull Requests are welcome.
