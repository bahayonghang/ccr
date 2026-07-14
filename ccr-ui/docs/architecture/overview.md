# CCR UI Architecture Overview

This document describes stable ownership boundaries. Use the [code map](../../code_map.md) for file-level navigation and the [published documentation](../../../docs/) for user-facing behavior.

## Runtime Shape

```text
Vue view or component
  -> composable or Pinia store
  -> domain API module
  -> Tauri invoke command
  -> Rust command/service
  -> CCR crate, local tool config, or external service
```

The browser-only development mode exercises the Vue side of this flow. Calls that require Tauri may fail in a plain browser even when the frontend route is healthy.

## Vue Application

- [`src/main.ts`](../../src/main.ts) bootstraps the application, router, global error handling, and startup behavior.
- [`src/router/index.ts`](../../src/router/index.ts) owns route registration and lazy-loaded page boundaries.
- `src/views/` owns route-level orchestration. Views compose domain components and should not duplicate the application shell.
- `src/components/` owns reusable UI and domain component families. Shared primitives live under `components/ui/`, `components/common/`, and `components/layout/`.
- `src/stores/` owns shared Pinia state; `src/composables/` owns reusable reactive workflows that do not require a global store.
- `src/config/`, `src/types/`, `src/utils/`, `src/i18n/`, and `src/styles/` own shared static configuration, contracts, helpers, localization, and design tokens.

The main route families cover application settings; Claude Code; Codex; OpenCode; Antigravity and Gemini CLI integrations; configuration and sync; usage and monitoring; sessions; MCP; agents; skills; plugins and hooks; output styles and statusline; check-in; WSL; and SSH.

## Frontend API Boundary

New business wrappers belong in `src/api/domains/<domain>.ts` and are exposed through [`src/api/index.ts`](../../src/api/index.ts). [`src/api/tauri.ts`](../../src/api/tauri.ts) is a compatibility facade for legacy imports, not the default location for new `invoke()` calls.

Views and stores should depend on domain APIs instead of backend command strings. This keeps command naming, argument mapping, and compatibility handling out of presentation code.

## Tauri Backend

- `src-tauri/src/main.rs` bootstraps the desktop shell.
- [`src-tauri/src/commands/handler_registry.rs`](../../src-tauri/src/commands/handler_registry.rs) is the command-registration inventory.
- `src-tauri/src/commands/` groups invoke handlers by domain.
- `src-tauri/src/state.rs` and service modules own shared application state and long-lived integration objects.
- `src-tauri/src/events.rs`, monitoring modules, and job modules own asynchronous work and frontend notifications.
- `src-tauri/src/llmusage_adapter/` isolates usage analytics from upstream schema and CLI changes.
- `src-tauri/src/platform/`, `process/`, and `ssh/` own host integration boundaries.

Tauri handlers may touch real user configuration. Changes must preserve masking, backups, atomic writes, and confirmation behavior.

## UI And Design Contracts

[`src/styles/tokens.css`](../../src/styles/tokens.css) is the semantic token source. Components consume role-based aliases such as `--surface-shell-*`, `--surface-workspace-*`, `--surface-card-*`, `--surface-modal-*`, and `--surface-status-*` instead of redefining material recipes locally.

The application targets a calm, precise, editorial workbench for AI CLI power users. Light and dark themes, reduced motion, reduced transparency, information density, and keyboard-accessible controls are product contracts rather than optional polish.

See [page templates and surfaces](../design-system/page-templates-and-surfaces.md) for contributor-facing layout rules.

## Verification Ownership

- `tests/` contains frontend smoke, router, i18n, and contract guards.
- `src-tauri/tests/` contains Rust-side integration and contract checks.
- `scripts/` contains build budgets, development warmup, icon generation, and other repository tooling.
- [`development/verification.md`](../development/verification.md) maps common changes to the narrowest reliable check.
