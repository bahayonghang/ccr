# fix-just-ci-failures

## Goal

Restore the repository `just ci` gate by identifying the first real failure, applying the smallest appropriate repair, and continuing until the full gate is green or a non-code external blocker is proven.

## What I Already Know

* The user asked to analyze and fix all `just ci` failures.
* The repo root `just ci` is the full local acceptance gate.
* The first observed failure is in `Version Sync`, before formatting, lint, tests, build, audit, or frontend checks.
* `scripts/version-sync.ps1` currently fails because it requires `ccr-ui/src/layouts/MainLayout.vue`, which no longer exists in the working tree.

## Assumptions

* Scope is limited to fixing actual local gate failures surfaced by `just ci`.
* Unrelated existing dirty work should be preserved.
* If a broad gate reveals multiple failures, handle them in order and prefer the narrowest owning gate for iteration.

## Requirements

* Preserve existing behavior unless the failing check proves the behavior is stale.
* Do not add dependencies.
* Keep changes surgical and aligned with current repo layout.
* Re-run the failing narrow gate after each fix.
* Finish with the full `just ci` gate when feasible.

## Acceptance Criteria

* [ ] The initial `version-sync` failure is fixed.
* [ ] Any later `just ci` failures are analyzed and repaired in order.
* [ ] `just ci` passes, or any remaining blocker is documented with exact evidence.
* [ ] `git diff --check` is run before final reporting.

## Definition of Done

* Narrow checks prove each changed surface.
* Full `just ci` is green or blocked by an external condition.
* The final response reports root cause, changed files, and exact verification commands.

## Technical Approach

Run the aggregate gate, inspect the owning script/code for the first failure, remove stale assumptions about deleted files only if current repo checks show they are no longer authoritative, then rerun the same gate and continue through the next failure.

## Out of Scope

* Refactoring unrelated scripts or UI architecture.
* Changing release/version policy beyond what the failed gate requires.
* Committing or pushing changes unless explicitly requested.

## Technical Notes

* `code_map.md` identifies `justfile` as the root verification anchor and `scripts/` as repo automation.
* Initial log: `just-ci-initial.log`.
* First failure: `scripts/version-sync.ps1:68` calls `Test-RequiredFile $LEGACY_MAIN_LAYOUT`, which resolves to missing `ccr-ui/src/layouts/MainLayout.vue`.
