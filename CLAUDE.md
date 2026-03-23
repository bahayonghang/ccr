# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build and verification

- Prefer root `just` recipes over ad-hoc commands so local checks stay aligned with CI.
- Standard repo verification is `just lint-strict`, `just test`, and `just frontend-check`; use `just ci` for the full pipeline.
- If you run Rust tests directly instead of `just test`, include `-- --test-threads=1` to avoid concurrent-conflict flakes.

## Tooling

- `ccr-ui` uses `bun` (`packageManager: bun@1.3.10`) as the primary frontend package manager. Use npm only as a compatibility fallback when Bun is unavailable.
- `docs/` is a separate VitePress package. When touching docs, verify it with `cd docs && npm run build` or via `just frontend-check`.

## Codebase-specific rules

- Internal implementation comments stay in Chinese; public API docs stay in English.
- When changing config, settings, or sync persistence flows, preserve masking of secrets, backup-before-destructive-change behavior, file locking, and atomic writes.

## Scoped guidance

- `@crates/ccr/src/CLAUDE.md` for core CLI/library details.
- `@ccr-ui/CLAUDE.md` for the Vue/Tauri UI.
- `@AGENTS.md` when working in the OpenSpec workflow.
