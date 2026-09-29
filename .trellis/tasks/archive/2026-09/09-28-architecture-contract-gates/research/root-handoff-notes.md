# T10 handoff notes

Date: 2026-09-28. Read-only preflight and research; T10 is not yet activated.

## Verified commands

- `just version-check`: passed in root.
- `just lint-strict`: passed; all workspace targets/features, deny warnings and unwrap. Evidence: `root-strict-preintegration.{json,log}`. This run precedes the T10 lock-test correction.

## Concrete gate gap

Root `just ci` dispatches three platform-specific timed lists. All omit `tauri-ci`. Existing `tauri-ci` already owns desktop fmt/check/strict clippy/full tests/bindings/inventory. Keep hosted required lanes and process matrix. Add the existing owner to the aggregate and verify a desktop child failure reaches the root result. Do not lower coverage or replace actual Rust tests with recipe text checks.

## Bindings boundary

All three `ccr-ui/justfile` bindings implementations delete the real generated directory first. The check script snapshots only files, normalizes the live directory, invokes generation, and returns immediately on failure. Its success/drift paths can also rewrite the worktree. Direct generation should retain successful outputs but restore the prior directory on export/normalizer/spawn failure. A check should compare outputs and preserve the original directory on every result, with nonzero propagated for failures/drift. Prefer one small shared snapshot/restore owner and narrow test injection, not a generic recovery framework. Relative ts-rs export destinations currently target the real directory; changing only process cwd is not a safe staging design. Test partial writes, new files, deletions, bytes, nested paths, initially absent directory, and throw/nonzero failures. Failure to restore must be visible.

## Evidence limits

- A08 has old real Rust 0/2 counterexamples in T06 `baseline-tests.log`. A09 deadline/cleanup old paths are source-proven; most new executor tests were not run on an old binary. Do not call those red-to-green without additional evidence.
- Final P1 mapping must link original evidence plus actual behavior tests and precise result logs. `fix_commit` stays null because no commit is authorized. Source/diff fingerprints identify the reviewed version.
- T07 independent final run: 19 files /102 tests; two late-unmount cases and one manual-Retry/timer race fixed with red/green evidence. Read final report when available.
- T09 remains in progress. No T10 product edits until prerequisite contracts are accepted.


## T09 backend scope accepted by independent source review

Claude settings expected-environment arguments now preserve one captured target; read/update zero-I/O mismatch and switch-during-read regressions pass (implementation 39 Claude tests, 21 registry tests). Generated window released; only two input schemas/three generated API files changed, no risk/permission/concurrency changes. Formal Tauri `--bin ccr-desktop -- -D warnings` passes. An additional `--all-targets` diagnostic fails four baseline test-lint items in main.rs, state.rs, Claude bool assertion and Codex test lock across await. Their unchanged baseline source was checked; do not describe the all-targets diagnostic as passed or perform unrelated cleanup. Evidence is T09 `backend-implementation-report.md` and `research/backend-verification.json`. Independent backend rerun is in progress as of this note.
