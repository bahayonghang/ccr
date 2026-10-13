# ccr-tui Backend Spec Index

> Terminal UI crate built on ccr-cli services.

## Guidelines Index

| Guide | Description | Status |
|-------|-------------|--------|
| [Backend Guidelines](./backend-guidelines.md) | TUI state/rendering boundaries, Codex Auth Token/cost/capacity presentation, recoverable errors, logging, tests, and verification | Complete |
| [Profile Workbench](./profile-workbench.md) | Three-platform A layout, shared table geometry, read-only activity, background operations and deferred exit | Complete |

## Pre-Development Checklist

- Read [Backend Guidelines](./backend-guidelines.md) before changing Ratatui state, rendering, key/mouse handling, async task runtime, or embedded auth views.
- Read [Profile Workbench](./profile-workbench.md) before changing Profile views or their background reads/mutations.
