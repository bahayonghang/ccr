# brainstorm: ccr-vscode update and optimization

## Goal

Bring `ccr-vscode` back in line with the current CCR codebase, then identify the highest-value follow-up optimizations so the extension stops drifting from `ccr-ui` and `crates`.

## What I already know

* `ccr-vscode` currently supports profiles, Codex auth, a status bar, and a webview-style profile editor.
* The manifest still describes the extension mainly as a Claude/Codex sidebar tool, but the codebase already has platform capability metadata for Gemini, Qwen, and Droid as well.
* `ccr-vscode` has a `switchProfileForPlatform` command registered in code, but it is not contributed in `package.json`.
* The status bar settings already support `pinned`, `current`, and `hidden` modes plus per-platform toggles.
* Current tests cover services and presentation helpers, but there is no broad end-to-end coverage around activation, command contribution, or manifest consistency.
* The repo has recent changes in `ccr-ui` and `crates`, so the extension likely needs a parity pass instead of only a version bump.

## Assumptions (temporary)

* The main goal is update and optimization analysis for the extension, not a full rewrite.
* We should preserve existing command IDs and settings keys unless a change is clearly needed for parity or cleanup.
* Any work should stay backward compatible with current extension users.
* The user wants all three areas in scope: sync fixes, activation/startup optimization, and platform expansion exposure.

## Open Questions

* What exactly counts as "platform expansion exposure"?
  * surface existing platform capability metadata in the manifest/docs/UI
  * add actual extension actions for more platforms beyond Claude/Codex
  * both

## Requirements (evolving)

* Audit `ccr-vscode` against the current CCR platform/command/settings surface.
* Identify missing or stale extension metadata, commands, docs, or tests.
* Decide which improvements belong in this task and which should stay out of scope.
* Surface already-known platform capability metadata for Gemini, Qwen, and Droid in the extension-facing surfaces where that data already fits.

## Acceptance Criteria (evolving)

* [ ] We have a scoped list of `ccr-vscode` updates to make.
* [ ] We have an explicit list of optimizations that are in scope.
* [ ] We have an explicit list of optimizations that are out of scope.

## Definition of Done (team quality bar)

* Tests added/updated where behavior changes.
* Lint / typecheck / CI green.
* Docs/notes updated if behavior or usage changes.
* Rollout/rollback considered if risky.

## Out of Scope (explicit)

* Rewriting the extension architecture.
* Changing public command IDs or settings keys without a clear reason.
* Touching generated artifacts or unrelated workspace areas.
* Adding new platform actions beyond exposing existing capability metadata.

## Technical Notes

* `ccr-vscode/package.json` still has a narrow marketplace description and may lag the current feature set.
* `ccr-vscode/src/models/platformCapabilities.ts` exposes capability metadata for more platforms than the manifest currently highlights.
* `ccr-vscode/src/extension.ts` registers a few commands that are not surfaced consistently in the manifest.
* `ccr-vscode/AGENTS.md` and `ccr-vscode/code_map.md` define the local conventions for this subtree.
