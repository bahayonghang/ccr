# CCR UI Engineering Documentation

This directory is the maintained engineering reference for contributors working on the React and Tauri application. User-facing installation, command, and workflow documentation belongs in the repository-level [`docs/`](../../docs/) site.

## Current Documents

- [Architecture overview](architecture/overview.md): runtime layers, module ownership, and the frontend-to-Tauri data path.
- [Page templates and surfaces](design-system/page-templates-and-surfaces.md): current shell, surface, async-state, accessibility, and layout contracts.
- [Verification guide](development/verification.md): focused checks, full gates, and web-preview boundaries.
- [Archive index](archive/README.md): completed plans, unimplemented proposals, and point-in-time analysis.

## Lifecycle Rules

Current documents describe contracts that contributors are expected to follow. They must reference live files, commands, and tests.

Point-in-time plans, generated review artifacts, and comparative reports belong under `archive/<year>/`. An archived file is evidence, not an active requirement. The archive index records whether the work was implemented and identifies the current source of truth.

Do not add new `plans/`, `spark/`, `superpowers/`, or `artifacts/` directories beside the maintained documentation. Create a Trellis task for active work, then archive only the decision material that remains useful after implementation.

## Source Of Truth

- Repository navigation: [`code_map.md`](../code_map.md)
- Scoped contribution rules: [`AGENTS.md`](../AGENTS.md)
- Routes and module registration: [`src/shell/router.tsx`](../src/shell/router.tsx)
- Frontend API boundary: [`src/api/index.ts`](../src/api/index.ts)
- Tauri command registry: [`src-tauri/src/commands/handler_registry.rs`](../src-tauri/src/commands/handler_registry.rs)
- Package scripts: [`package.json`](../package.json)

Run `bun run docs:audit` from `ccr-ui/` after changing this directory.
