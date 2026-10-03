---
name: ccr-gate-recovery
description: CCR-specific local gate recovery workflow. Use when asked to fix failing CCR checks, just ci, Rust workspace gates, ccr-ui checks, VS Code extension tests, docs build/audit failures, or noisy aggregate CI output. Applies to Claude Code, Codex, Grok, Kimi, and OMP. Keep existing parallel tests; do not invent a new recovery engine.
---

# CCR Gate Recovery

Recover local verification gates from the CCR repository root by finding the first real failing step, fixing narrowly, and escalating validation only after the narrow gate is green.

This skill applies to Claude Code, Codex, Grok Build, Kimi Code, and OMP. Use existing repository gates and default test parallelism. Isolate shared environment state in the owning fixture. Keep binding exports in the separate generation gate.

Before implementation approval, inspect and run only authorized checks. After approval, implement and self-fix check roles may edit the approved files. Keep native client, hosted CI, and local test evidence separate.

## Workflow

1. Confirm the current working tree with `git status --short`; preserve unrelated user changes.
2. Reproduce the narrowest failing gate when the user gave one. For aggregate output, identify the first real failure and ignore downstream noise until it is fixed.
3. Inspect only the owning surface:
   - Rust workspace: `crates/**`, `Cargo.toml`, root `justfile`.
   - UI/Tauri: `ccr-ui/src/**`, `ccr-ui/src-tauri/**`, `ccr-ui/tests/**`.
   - VS Code extension: `ccr-vscode/src/**`, `ccr-vscode/package.json`.
   - Docs: `docs/**`, especially `.vitepress/config.mjs` and `scripts/audit-docs.mjs`.
4. Make the smallest behavior-preserving or bug-fixing edit. Do not add dependencies unless explicitly requested.
5. Re-run the same failing gate. When green, run the next broader gate that proves the touched surface.
6. Finish with `git diff --check` and report exact commands run plus any skipped expensive gates.

## Gate Map

### Rust

```powershell
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features -- --skip export_bindings
```

Use package-scoped `cargo test -p <crate-name> -- --skip export_bindings` when the failing crate is clear. Independent UI, docs, and extension gates may run in parallel with Rust.

### ccr-ui

```powershell
cd ccr-ui
bun run type-check
bun run lint
bun run test
```

Use root `just ui-check` for the broader UI gate and `just frontend-check` when docs also need to be included.

### VS Code extension

```powershell
cd ccr-vscode
npm run build
npm test
```

Use `npm run lint` or `just ci` inside `ccr-vscode/` when TypeScript config or packaging surfaces changed.

### Docs

```powershell
cd docs
bun run build
bun run audit
```

Fix documentation audits at the source Markdown/config level; do not edit `.vitepress/dist/`.

### Full repository

```powershell
just ci
```

Run this only after narrow gates are green or when the user explicitly asks for the full repo gate.

## Reporting

Keep the final report short:

- failing gate and root cause
- files changed
- validation commands and pass/fail status
- remaining risks or skipped broader gates
