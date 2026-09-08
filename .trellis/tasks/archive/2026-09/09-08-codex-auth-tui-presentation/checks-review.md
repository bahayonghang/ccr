# Implementation review

- Reviewer: independent `trellis-check` agent, 2026-09-08.
- Scope: Codex Auth `app.rs` / `ui.rs`, main `tui/ui.rs` test additions, and TUI spec; approved AC1–AC5.
- Authority: implementation and self-fix approved. No private auth/usage data, live quota API, desktop operation, dependencies, commit, push, or release.

## Findings (fixed)

1. **Cached quota disappeared on actual failure messages (AC1).** `drain_task_messages` replaced the last successful preview with a no-quota error result for batch previews, `Quota(Ok(error_snapshot))`, and outer `Quota(Err(...))`. The original render regression manually set `QuotaState::Error` and did not exercise those paths. Added one cache-write helper that preserves the successful quota, email, and original `fetched_at`, while attaching the new error. It uses only the existing in-memory cache and leaves requests, authentication, and persistence unchanged.
2. **A successful batch retry retained an old selected-account error (AC1).** A `QuotaState::Error` could outlive a later successful preview. A valid batch result now clears that error only for the matching account. Regression consumes real synthetic channel messages, covers preview-only and quota-state-only starting caches, all three failure kinds, batch recovery, and selected-quota recovery.
3. **Cached batch refresh lacked a visible refresh marker (AC1).** Added a read-only accessor for the existing batch activity flag and the localized `Refreshing previews` / `速览刷新中` status. This describes batch activity without claiming that the selected account must participate.
4. **The normal coverage note was needlessly clipped at a supported wide size (AC2/AC3).** Shortened it to `Excludes other accounts and unattributed records` / `不计入其他账号及未归属记录`. The preceding scope line continues to identify the selected account and CCR ledger. Successful attribution remains neutral; global fallbacks retain warning styling and scope.
5. **Strict lint rejected a test-only production import and new test unwraps (AC5).** Moved `QuotaState` into the test module and replaced the rejected new test `unwrap` calls with context-bearing `expect` assertions. No lint allowance or production panic path was added. Targeted strict Clippy and the complete TUI tests then passed.

## Review conclusions

- Quota bars and percentages use remaining quota, preserve missing/unknown windows, and keep reset information; no server-window remapping was introduced.
- Account filtering and activation-window arithmetic are unchanged. Global fallback numbers remain paired with an explicit global scope and warning.
- Compact rendering budgets against the real composed content area, with a no-statistics omission state below the supported budget. Bilingual fixtures cover the supported viewport matrix, CJK/long identifiers, actual cells and colors, and list selection preservation.
- Changes to main `tui/ui.rs` are tests only; both real auth entry routes still share `draw_embedded`. No production shell or neighboring auth surface was modified.
- TUI spec includes successful-cache retention at message consumption and same-account error recovery; no new source of truth, state machine, or configuration was added.

## Verification

| Check | Result |
| --- | --- |
| Message-level failure regression before fix | FAIL as expected: `failure kind 0, cache only false lost quota` |
| Cached batch refresh regression before fix | FAIL as expected: absent `Refreshing previews` |
| Batch recovery regression before fix | FAIL as expected: old `fixture network failure` remained |
| `rtk cargo test -p ccr-tui codex_auth -- --test-threads=1` | PASS, 38 tests |
| `rtk cargo fmt -p ccr-tui` | PASS |
| `rtk cargo test -p ccr-tui -- --test-threads=1` | PASS, 228 tests / 2 suites |
| `rtk cargo clippy -p ccr-tui --all-targets --all-features -- -D warnings -D clippy::unwrap_used` | PASS, no issues; complete TUI tests re-run afterward: 228 passed |
| Composed layout matrix with `--nocapture` | PASS; final fixture-only output in `research/render-snapshots.txt` |
| Required repository fmt/lint and final binary integration | Main-session gate owner; final result recorded in `checks.md` |

No unresolved source defect found within the approved scope. Native Windows terminal appearance, screenshot-account statistics, real API reset times, and private runtime correctness remain **UNVERIFIED**. Synthetic buffer evidence does not establish them.
