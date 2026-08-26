# CCR UI Verification Guide

Run commands from `ccr-ui/` unless noted otherwise. Start with the narrowest check that covers the changed contract, then widen only when the change crosses boundaries.

## Documentation

```powershell
bun run docs:audit
```

This checks the maintained document structure, archive lifecycle, local Markdown links, and known stale command references.

## Frontend

| Change | Focused checks |
|---|---|
| TypeScript, Vue, router, store, or composable | `bun run type-check`, `bun run lint`, `bun run test:smoke` |
| Visible or localized copy | `bun run test:i18n`, `bun run check:i18n` |
| API wrapper or invoke boundary | `bun run test:smoke -- tests/api/api-facade-boundary.smoke.test.ts`, then type-check and lint |
| Theme, tokens, fonts, or semantic surfaces | relevant theme smoke tests, type-check, lint, and browser inspection |
| Production bundling or dependency shape | `bun run build`, optionally `bun run check:bundle-budget` |

`bun run lint` is a no-fix verification path. Use `bun run lint:fix` only when formatting changes are intentional and reviewed.

## Tauri Backend

```powershell
bun run tauri:check
bun run tauri:test
bun run tauri:clippy
```

Use the focused command that matches the Rust change, then run the broader UI gate for cross-layer work.

## Browser And Visual Checks

Start the web preview without invoking the desktop shell:

```powershell
bun run dev:web -- --host 127.0.0.1 --strictPort
```

Open `http://127.0.0.1:5173/` and inspect the affected routes at desktop and mobile widths. For visual changes, verify light and dark themes, reduced motion, loading/error/empty states, and text fit.

Web mode cannot complete Tauri-only `invoke()` operations. Distinguish that runtime limitation from rendering, routing, or frontend-state failures. There is no supported route-snapshot package command; do not cite the old route-snapshot artifacts as a live gate.

## Full Gates

```powershell
bun run check:all
```

From the repository root, use `just ui-check` for the complete UI surface or `just frontend-check` when the change also affects the published documentation site.

Do not commit `dist/`, `src-tauri/target/`, `test-results/`, caches, local logs, or dependency output created by these commands.
