# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build and verification

- Prefer root `just` recipes over ad-hoc commands so local checks stay aligned with CI.
- Standard repo verification is `just lint-strict`, `just test`, and `just frontend-check`; use `just ci` for the full pipeline.
- `just ci` pipeline: version-sync → fmt → fmt-check → lint-strict → workspace-check → test → release → audit → frontend-check → vscode-ci.
- If you run Rust tests directly instead of `just test`, include `-- --test-threads=1` to avoid concurrent-conflict flakes.
- Set `CCR_LOG_LEVEL=debug` (or `trace|info|warn|error`) for runtime debug output.

## Tooling

- `ccr-ui` uses `bun` (`packageManager: bun@1.3.10`) as the primary frontend package manager. Use npm only as a compatibility fallback when Bun is unavailable.
- `docs/` is a separate VitePress package. When touching docs, verify it with `cd docs && npm run build` or via `just frontend-check`.

## Codebase-specific rules

- Internal implementation comments stay in Chinese; public API docs stay in English.
- When changing config, settings, or sync persistence flows, preserve masking of secrets, backup-before-destructive-change behavior, file locking, and atomic writes.

## Version management

- Version source of truth is `workspace.package.version` in root `Cargo.toml`. After changing it, run `just version-sync` to propagate to all crates, `ccr-ui/package.json`, `ccr-ui/src-tauri/`, and `ccr-vscode/package.json`.
- Use `just version-check` to verify version consistency without modifying files.

## Commit conventions

- Follow Conventional Commits with scope and optional emoji: `feat(tauri): ✨ add profile switch`, `fix(core): 🐛 handle missing config`.
- Branch strategy: `main` (production), `dev` (development), `feature/*`, `bugfix/*`.
- Keep commits scoped to one concern. PRs should include summary, impacted areas, verification commands, and linked issues.

## Scoped guidance

- `@crates/ccr/src/CLAUDE.md` for core CLI/library details.
- `@ccr-ui/CLAUDE.md` for the Vue/Tauri UI.
- `@AGENTS.md` when working in the OpenSpec workflow.

## Skill routing

When the user's request matches an available skill, ALWAYS invoke it using the Skill
tool as your FIRST action. Do NOT answer directly, do NOT use other tools first.
The skill has specialized workflows that produce better results than ad-hoc answers.

Key routing rules:
- Product ideas, "is this worth building", brainstorming → invoke office-hours
- Bugs, errors, "why is this broken", 500 errors → invoke investigate
- Ship, deploy, push, create PR → invoke ship
- QA, test the site, find bugs → invoke qa
- Code review, check my diff → invoke review
- Update docs after shipping → invoke document-release
- Weekly retro → invoke retro
- Design system, brand → invoke design-consultation
- Visual audit, design polish → invoke design-review
- Architecture review → invoke plan-eng-review
- Save progress, checkpoint, resume → invoke checkpoint
- Code quality, health check → invoke health
