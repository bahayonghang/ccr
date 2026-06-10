# Brooks Sweep Crates Optimization

## Goal

Run a focused Brooks full sweep on the Rust workspace under `crates/` to find and safely fix maintainability, test quality, debt, and architecture issues without changing unrelated repository areas.

## Requirements

* Scope is limited to tracked files under `crates/`.
* Use the `brooks-sweep` flow in Full Sweep mode: review, test, debt, then audit.
* Apply only Safe and Extended-Safe fixes:
  * Safe: local single-file changes that do not alter exported/public APIs.
  * Extended-Safe: small multi-file fixes with existing verification coverage and no public interface changes.
* Record high-risk findings as residual items instead of applying them:
  * public API breaks;
  * cross-crate structural rewrites;
  * ambiguous architecture changes;
  * changes without meaningful verification coverage.
* Preserve unrelated work and do not commit, push, amend, reset, or stash.
* Keep the worktree reviewable by using narrow, evidence-backed edits.

## Acceptance Criteria

* [x] User gives the one-time Brooks sweep consent before any code edit.
* [x] Scope enumeration records the `crates/` tracked file count.
* [x] The sweep report lists dimension summaries, fixes applied, verification commands, and residual items.
* [x] Each applied change has a direct Brooks finding and a verification result.
* [x] Rust formatting, lint, and tests are run at the narrowest useful level, escalating to the repo gate as needed.
* [x] No frontend, VS Code extension, docs, generated, or runtime-output files are changed unless needed for verification metadata.

## Definition of Done

* `git status --short` reviewed before and after the sweep.
* At minimum, run `just fmt-check`, `just lint-strict`, and `just test` for crates changes.
* Run `just ci` if changes touch cross-crate behavior, workspace dependencies, release-sensitive code, or if narrower gates leave uncertainty.
* Final response includes changed files, verification results, and residual findings.

## Completion Notes

* User consent: `Y`.
* Scope: 402 tracked files under `crates/`, including 384 Rust files.
* Applied fixes:
  * `crates/ccr-checkin/src/managers/checkin/waf_cookie_manager.rs` replaces an unimplemented runtime panic path with an explicit error and adds regression coverage.
  * `crates/ccr-db/src/database/repositories/checkin_repo.rs` removes impossible midnight `expect` usage by using `NaiveTime::MIN`.
* Verification passed: targeted package tests and clippy for `ccr-checkin` and `ccr-db`, then `just fmt-check`, `just lint-strict`, and `just test`.
* `just ci` was not run because the applied fixes are internal Rust-only local changes with no workspace dependency, frontend, docs, release, or cross-crate behavior impact.
* Residual architecture/debt items are recorded in the final sweep report rather than auto-applied.

## Technical Approach

1. Preflight: request the exact `brooks-sweep` consent for the `crates/` scope.
2. Baseline: enumerate tracked crate files, inspect package boundaries, and run/read the narrow baseline checks needed to classify safe fixes.
3. Review pass: scan production Rust for local decay risks, then apply safe cleanup only where the remedy is obvious and covered.
4. Test pass: scan tests and test gaps, adding or adjusting tests only for local pure behavior or already-covered surfaces.
5. Debt pass: identify repeated local patterns and stale workaround clusters; apply only bounded consolidation.
6. Audit pass: identify architectural risks, but keep module-boundary or public API changes as residual findings.
7. Iterate on modified files and direct consumers, then report applied fixes and residuals.

## Decision (ADR-lite)

**Context**: The user requested a deep crates analysis plus optimization, and named `brooks-sweep`, which is an auto-fix workflow.

**Decision**: Use a crates-only Brooks full sweep with auto-fix limited to low-risk changes. Do not broaden to `ccr-ui/`, `ccr-vscode/`, docs, or generated outputs.

**Consequences**: This favors safe, reviewable cleanup over large structural rewrites. Some architecture or public API findings may remain as residual recommendations.

## Out of Scope

* Frontend or Tauri UI optimization.
* VS Code extension optimization.
* Documentation rewrites unrelated to the sweep report or task notes.
* Dependency upgrades unless a crate issue cannot be fixed otherwise.
* Public API redesigns or crate boundary rewrites.
* Git commits or pushes.

## Technical Notes

* Repo root: `D:\Documents\Code\Github\ccr`.
* Current branch from Trellis context: `dev`.
* Initial working tree from Trellis context: clean.
* Workspace members under `crates/`: `ccr`, `ccr-core`, `ccr-config`, `ccr-sync`, `ccr-skills`, `ccr-store`, `ccr-codex`, `ccr-db`, `ccr-checkin`, `ccr-cli`, `ccr-tui`, `ccr-types`.
* Tracked files in scope at task creation: 402 under `crates/`.
* Relevant repo commands from `justfile`: `just fmt-check`, `just lint-strict`, `just test`, `just ci`.
* `brooks-sweep` local package is missing the referenced sibling `_shared` files; use the available `sweep-guide.md` plus repo/Trellis rules and record this as a process limitation.
