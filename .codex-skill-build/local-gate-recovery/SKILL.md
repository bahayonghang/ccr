---
name: local-gate-recovery
description: Diagnose and fix local repository verification gates such as just ci, pnpm sizecheck, cargo clippy, cargo fmt, cargo test, bun run test, just ui-check, docs builds, and Windows shell warning noise. Use when the user reports a local gate failure, asks to fix all just ci errors, says continue after interrupted CI debugging, or provides failing command output. Focuses on the actual failing command as source of truth instead of generic CI or GitHub Actions workflows.
---

# Local Gate Recovery

Recover local repo gates from evidence. This skill is for the repeated pattern where the user wants the failing local command closed, not merely explained.

## Workflow

1. Treat the latest failing output as authoritative.
   - Read the exact command, exit code, and first real failure.
   - If the user supplies newer logs, re-evaluate older hypotheses against them.
   - Do not diagnose app code when the failure is clearly shell/bootstrap/tooling noise.

2. Reproduce narrowly.
   - Run the same command when cheap enough.
   - If an aggregate gate like `just ci` times out, is opaque, or hides the blocker, run constituent gates directly in order.
   - Stop at the first real blocker; avoid collecting extra logs that do not change the fix.

3. Locate the owning constraint.
   - Examples from memory: file line budgets surfaced by `pnpm sizecheck`, Windows `PSReadLine` warning configuration, repo version checks, docs builds, Rust fmt/clippy/tests, Tauri UI checks.
   - Prefer fixing the exact violated constraint over broad refactors.
   - Preserve unrelated dirty changes.

4. Make the smallest scoped fix.
   - Use existing repo patterns and utilities.
   - Do not add dependencies for gate recovery unless the user explicitly asks.
   - If the failure is a warning that pollutes local output but does not block correctness, fix the shell/bootstrap layer when that is the source.

5. Verify in a ladder.
   - Re-run the failing narrow gate first.
   - Then run neighboring gates that could be affected.
   - Finish with the aggregate gate when feasible and expected by the repo.
   - Read outputs before claiming success.

## Common Ladders

- Rust CLI repo: `cargo fmt --check`, targeted `cargo test`, `cargo clippy -- -D warnings`, then `just ci`.
- Vue/Tauri repo: targeted Vitest, `pnpm typecheck`, `pnpm lint`, `pnpm test`, `pnpm sizecheck`, Rust targeted tests or clippy, then `just ci` or `just ui-check`.
- Docs/reporting repo: targeted tests, docs build, screenshot or browser sanity check when the plan requires visual evidence.

## Failure Patterns To Avoid

- Calling a local gate fixed after only surrounding tests passed.
- Re-running `just ci` repeatedly when it times out before revealing the failing sub-step.
- Mistaking pre-existing warnings for new regressions.
- Treating a shell warning during `just ci` as an application bug before checking the command wrapper and shell profile behavior.

## Output

Report the original failing gate, root cause, files changed, verification ladder, and any remaining non-blocking warnings.
