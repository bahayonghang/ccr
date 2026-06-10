# Recover just ci gate

## Goal

Run the repository-wide `just ci` gate from the CCR root, identify the first real failing step, and make the smallest necessary fixes until the gate passes or a true external blocker is identified.

## Requirements

* Use `just ci` as the aggregate source of truth.
* Preserve unrelated existing dirty-tree changes.
* Fix only verified CI failures; avoid speculative refactors or scope expansion.
* Prefer narrow reproduction commands for the failing subsystem before rerunning the full gate.
* Report the root cause, changed files, and validation commands.

## Acceptance Criteria

* [ ] `just ci` has been run from the repository root.
* [ ] Each real failure is reproduced narrowly where practical.
* [ ] Necessary fixes are implemented with minimal file changes.
* [ ] The relevant narrow gate passes after each fix.
* [ ] Final `just ci` passes, or any remaining blocker is clearly documented as external/non-code.
* [ ] `git diff --check` passes.

## Definition of Done

* The full repository gate is green or blocked by a clearly stated external condition.
* No unrelated dirty-tree changes are reverted.
* No new dependencies are added unless required by the failing gate.
* Notes are synchronized with this task if the failure reveals durable project guidance.

## Technical Approach

Follow the CCR gate recovery workflow: inspect status, run `just ci`, isolate the first real failure, fix narrowly in the owning surface, rerun the narrow gate, then rerun the aggregate gate.

## Out of Scope

* Refactoring adjacent code unrelated to the gate failure.
* Cleaning or restoring pre-existing Playwright output deletions unless they are the actual CI failure.
* Committing changes unless explicitly requested later.

## Technical Notes

* Existing dirty tree before work includes root and Tauri `Cargo.toml` modifications plus deleted `ccr-ui/output/playwright/*` artifacts.
* Relevant repo guidance: root `AGENTS.md`, `code_map.md`, `.codex/skills/ccr-gate-recovery/SKILL.md`, and Trellis specs loaded before code edits.
