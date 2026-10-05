# CCR UI Page Templates And Surface Contracts

This document defines the maintained page-level design contracts for CCR UI. It complements the scoped direction in [`ccr-ui/AGENTS.md`](../../AGENTS.md) and the token implementation in [`src/styles/tokens.css`](../../src/styles/tokens.css).

## Product Direction

CCR UI is a calm, precise, editorial workbench for AI CLI power users. Pages prioritize fast scanning, operational clarity, and dense but organized information. Decorative effects must support hierarchy rather than compete with the task.

All templates must support light and dark themes, reduced motion, reduced transparency, keyboard navigation, and text expansion.

## Shell Ownership

[`MainLayout`](../../src/shell/MainLayout.tsx) owns viewport-level navigation and application chrome. Routes own only their workspace content.

Routes must not add:

- a second application navbar or primary breadcrumb;
- fixed viewport backgrounds that compete with the shell;
- `min-h-screen` wrappers that create a second viewport;
- page-local theme or global navigation controls.

Use `ModuleSubnav` for navigation within a platform or module family, `PageHeaderCard` for a page title and primary actions, and a contained route workspace for the body.

## Page Templates

### Operational Dashboard

Use for state summaries, usage, monitoring, and other signal-dense pages.

Structure:

1. optional module sub-navigation;
2. one compact title/action region;
3. high-signal status or metric summary;
4. charts, tables, logs, or diagnostic panels;
5. shared loading, error, and empty states.

Current route examples include `/`, `/usage`, and `/monitoring`.

### Collection And Detail Workspace

Use when users browse a collection and inspect or edit one record.

Keep search, filters, selection, and bulk actions close to the collection. Use a contained detail region or modal; do not create another shell inside the route.

Current route examples include `/mcp-manager`, `/agents`, `/skills`, and platform profile pages.

### Settings And Form Workspace

Use for configuration-heavy pages and focused editors.

Provide one dominant form or configuration surface. Place validation and operational status near the affected control. Secondary guidance can sit beside or below the form when space allows, but it must not become a nested card hierarchy.

Current route examples include `/settings`, `/configs`, `/sync`, and `/statusline`.

## Semantic Surfaces

Components consume role-based surface aliases rather than raw material recipes:

| Alias | Role |
|---|---|
| `--surface-shell-*` | persistent sidebar and topbar chrome |
| `--surface-workspace-*` | standard route workspaces and dense panels |
| `--surface-card-*` | individual repeated or elevated content |
| `--surface-modal-*` | modal containers and floating regions |
| `--surface-status-*` | compact toolbar, status, and sticky controls |

Ordinary content cards and workspaces are opaque surfaces. Glass is limited to its semantic role and must not be nested or placed inside scrolling lists. New components must not consume deprecated `--glass-*` or `--liquid-glass-*` recipes directly.

Before changing a shared alias, search all consumers. Repoint the specific component to an existing role when possible; changing an alias definition can affect unrelated routes.

## Shared Primitives

- `AsyncStatePanel` is the default loading, error, and empty-state primitive for route-sized regions.
- `EmptyState` is appropriate when an empty collection needs a primary recovery action.
- `PageHeaderCard` owns the route title, supporting copy, and primary actions.
- `ModuleSubnav` owns navigation between pages in one module family.
- Shared controls under `src/ui/` own consistent buttons, inputs, cards, dialogs, and feedback.

Small inline regions may use compact text feedback, but route-level states must not be hand-built repeatedly.

## Responsive And Accessibility Rules

- Use stable grid tracks, minimum widths, aspect ratios, or bounded containers for fixed-format UI.
- Allow controls and labels to wrap before text clips or overlaps.
- Preserve visible focus treatment and keyboard order when layouts reflow.
- Keep operational tables scannable on narrow screens through deliberate stacking or horizontal scrolling.
- Disable non-essential motion under `prefers-reduced-motion`.
- Resolve material surfaces to opaque backgrounds under `prefers-reduced-transparency`.
- Verify text and state colors in both light and dark themes.

## Verification

Run the focused static checks for the files changed, then use the web preview for visual work:

```powershell
bun run type-check
bun run lint
bun run test:smoke
bun run dev:web -- --host 127.0.0.1 --strictPort
```

Inspect the affected route at `http://127.0.0.1:5173/` at desktop and mobile widths. Verify light and dark themes, reduced motion, text fit, focus behavior, and loading/error/empty states.

Plain web mode cannot complete every Tauri `invoke()` operation. Treat those failures as runtime limitations unless the route, presentation state, or browser-safe behavior is also broken. See the [verification guide](../development/verification.md) for the full command matrix.
