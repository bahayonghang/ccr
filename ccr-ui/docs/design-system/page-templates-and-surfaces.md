# CCR UI Page Templates And Surface Rules

## Purpose

This document records the approved page templates and visual surface contracts introduced during the shell, token, and hotspot-refactor passes. It is the Phase 6 reference for keeping new routes aligned with the current UI system instead of reintroducing page-owned shells or ad hoc glass variants.

## Approved Page Templates

### 1. Dashboard Template

Use for pages that summarize system state, usage, or high-signal operational metrics.

Required structure:
- shared app shell from `MainLayout`
- optional `ModuleSubnav` when the page belongs to a module group
- one header block for title, subtitle, and primary actions
- workspace sections for charts, cards, and log tables

Current examples:
- `/usage`
- `/agents`
- `/market`

### 2. List / Detail Workspace Template

Use for routes that switch between collection browsing and focused record inspection.

Required structure:
- shared shell and subnav
- page-owned workspace header only, never a nested navbar or breadcrumb
- list region, detail region, or detail modal
- async states rendered through shared state primitives

Current examples:
- `/skills`
- `/mcp/unified`
- `/claude-code/profiles`

### 3. Settings / Form Template

Use for configuration-heavy screens and modal editors.

Required structure:
- shared shell and optional module subnav
- page header card or compact workspace intro
- one dominant form region
- secondary guidance or status stacked beside the form when needed

Current examples:
- `/skills/add`
- `/sync`
- `/statusline`

## Surface Contract

New code should prefer semantic surface utilities over legacy glass aliases.

Primary surfaces:
- `surface-shell`: top-level app shell chrome
- `surface-workspace`: standard page workspace panels
- `surface-card`: denser or more elevated content cards
- `surface-modal`: modal containers and modal subregions
- `surface-status`: compact status chips, controls, or inline indicators

Legacy classes:
- `glass-effect`
- `glass-surface`
- `glass-effect-strong`
- `liquid-glass`

These still work as migration aliases, but new components should not introduce additional legacy surface names.

## Shared Utility Contracts

The following utilities are now shared and should not be redefined inside individual pages:

- `glass-panel`
  - lightweight dashboard workspace panel
- `toolbar-select`
  - compact select control used in dense toolbars
- `AsyncStatePanel`
  - shared loading / error / empty-state presentation

## Async State Rules

Pages and components should not hand-roll their own loading, error, or empty-state layouts unless there is a strong visual reason.

Preferred approach:
- use `AsyncStatePanel` for generic async states
- use `EmptyState` when the page needs a more branded empty-state treatment with a primary CTA
- keep raw text-only fallback states limited to small inline regions

## Shell Ownership Rules

Routes must not own viewport-level shell chrome.

Never reintroduce:
- page-local navbars
- page-local breadcrumbs as primary navigation
- route-owned fixed viewport backgrounds
- `min-h-screen` wrappers that compete with `MainLayout`

Use instead:
- `MainLayout`
- `ModuleSubnav`
- `PageHeaderCard`
- contained decorative backgrounds inside the route container

## Verification Rules

Before calling a route visually complete:

- capture a light-theme screenshot
- capture a dark-theme screenshot
- capture a reduced-motion screenshot or verify reduced-motion behavior on the same route
- confirm the route still renders under plain web mode even if some Tauri-only actions fail

The canonical capture script is:

```bash
bun run test:playwright:snapshots
```

Expected output location:

```text
ccr-ui/tests/artifacts/route-snapshots/
```
