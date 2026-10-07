# Independent check: Codex identity and account lifecycle

- Task: `.trellis/tasks/10-06-auth-identity-key` (P1).
- Reviewer: `trellis-check` (`/root/check_p1`).
- Status: **SCOPED_CHECK_PASS / WORKSPACE_NOT_RUN**.
- Date: 2026-10-06 America/Chicago. Receipt and hash timestamps retain their UTC values.
- Scope: approved P1 R1–R5, P2 compatibility, affected CLI/TUI construction and behavior. Product files are frozen.
- Inputs: latest `check.jsonl`, PRD, design, implement, listed specs, implementation reports, and `cockpit-account-lifecycle.md`.
- The earlier `independent-check.md` remains unchanged. Its 328 passed / 2 ignored result is historical. The independent final package result is 340 passed / 2 ignored.
- The lifecycle review found eight issues. The implementation owner fixed those issues before final verification. The reviewer made no product edits during the final lifecycle verification.

## Acceptance criteria

The row numbers follow the eight clauses in the current PRD.

| AC | Result | Code and evidence |
| --- | --- | --- |
| 1. Same account_id, different user_id | PASS | `codex_auth_identity.rs:26` supplies one complete identity parser. Full-key comparisons isolate users in sync, reconciliation, repair and quota selection. The shared-workspace tests in `codex_auth_service.rs:4184` and quota/core tests pass in the independent Codex gate. |
| 2. Bidirectional sync, repair, switch-out, quota/cache, alias priority | PASS | `codex_oauth_token_service.rs:556` plans synchronization; saved identity and candidate selection at :741/:792 require the complete key. `codex_auth_service.rs:1227` revalidates runtime identity and replans before the auth-only CAS at `codex_runtime_service.rs:322`. Snapshot writes recheck identity, version and freshness. Quota callbacks validate both the expected destination and the refreshed identity. Same-key alias priority remains current_auth, last_used, then registry order; the tests at `codex_auth_service.rs:4664` retain that rule. Cache tests cover different users in one workspace. |
| 3. Legacy backfill, read-only planning and P2 | PASS | Backfill at `codex_oauth_token_service.rs:765` reads each record's own snapshot. Runtime identity does not fill an unrelated record. Account names and current_auth remain stable. Planner/read paths do not persist. P2 read-only registries skip backfill and reject save/switch before operation-lock creation. Safe bidirectional token synchronization remains available. P2 command/tree and current_auth regressions pass in the independent Codex gate. |
| 4. Missing/conflicting identity, provider compatibility, identity privacy | PASS within the approved contract | Cross-file synchronization and repair return NoOp when a complete identity is missing, invalid or conflicting. Incomplete runtime reconciliation retains current_auth. Unknown sources retain same-file quota refresh, as approved; they do not associate separate files. API key/provider branches and command behavior retain their regressions. The new complete identity has no public DTO field and is excluded from its Debug/error/log paths. Existing raw-token Debug/body-preview work remains assigned to P4. |
| 5. Synthetic claims and CLI/TUI/P2 regression | PASS | Successful OAuth fixtures use complete user claims. Missing/conflicting identity NoOp tests remain. Independent gates: Codex 340 passed / 2 ignored; TUI 252 passed; CLI auth-off 11 passed and current JSON 2 passed. CLI auth-off rotation writeback and current JSON privacy assertions remain covered. Default test parallelism and `--skip export_bindings` are retained. |
| 6. Manual quota query and refresh behavior | PASS | `openai_quota_core.rs` uses force_refresh only to bypass quota cache. A live access token issues GET before any refresh. Five manual-query loopback tests at :907/:932/:968/:990/:1012 cover successful live-token GET without POST/persist, cache bypass, expiry, remote authentication rejection and permanent refresh rejection. The 340-test gate includes those tests. Failed authentication does not become a successful quota result. |
| 7. Account lifecycle and cross-process lock evidence | PASS for approved entry points | `codex_auth_refresh_lock.rs:84` holds a stable source-path resource and an additional complete-identity resource when known. Per-path expected resources are verified at :102. Lock holders reread candidate documents. Switch holds source/target resources in stable order; held-lock internal helpers avoid reentrant acquisition. Save derives metadata and snapshot from one captured read. Freshness retains last_refresh-first/mtime-fallback semantics at full timestamp precision. Unknown-to-known refresh exits before GET/retry/sync, releases the old locks, and retries at most once with the persisted identity as its expectation. Eight lifecycle regressions and four lock regressions pass in the full independent gate. A real child process proves LockTimeout before ready, then acquires and reads the new version after release. Same-process loopback tests separately prove HTTP/refresh/switch behavior. |
| 8. Codex test, workspace strict lint and workspace tests | PACKAGE_PASS / WORKSPACE_NOT_RUN | Independent Codex package test passes. Strict clippy covers ccr-codex, ccr-cli and ccr-tui. `just lint-strict` and `just test` remain deferred under the main session's approved P3–P6 gate sequence. Package checks do not mark those workspace commands PASS. |

## Findings (fixed)

1. File: `codex_auth_refresh_lock.rs`.
   Issue: unknown-to-known identity changed the lock resource; missing-to-created source paths could also change normalization.
   Fix: the owner keeps the normalized source-path lock for every call and adds the identity lock when available. The stable-path regression passes.

2. Files: `codex_auth_refresh_lock.rs`, `codex_auth_service.rs`.
   Issue: checking membership in the whole source/target lock set could accept a target that changed to the outgoing identity while waiting.
   Fix: the owner captures each path's expected resource and validates that path against its own resource after acquisition.

3. File: `codex_oauth_token_service.rs`.
   Issue: second-level timestamp comparison and writeback could select an older runtime document after a same-second alias refresh.
   Fix: the owner preserves full last_refresh/mtime precision, full backup mtime ordering and AutoSi timestamp writeback. Existing freshness priority and candidate limits remain. The same-second behavior regression passes.

4. File: `codex_quota_service.rs`.
   Issue: an unknown source could publish complete identity and continue consuming the new refresh chain while a known alias used another lock.
   Fix: a private callback result stops the core immediately after the identity-completing persist. The outer call releases locks and retries at most once under the new resource set. The retry keeps the successfully persisted identity as its expectation. Unknown-source/known-alias 401 and replacement-user regressions pass.

5. File: `codex_auth_refresh_lock.rs` tests.
   Issue: child ready preceded the first lock attempt and did not prove a held OS lock.
   Fix: the child first asserts zero-timeout LockTimeout, writes ready, then acquires after release and rereads the updated document. The real child-process test passes.

6. File: `codex_auth_service.rs`.
   Issue: public synchronization pre-repair wrote a snapshot before returning the planner's Unchanged result.
   Fix: the owner removed public pre-repair and confines preparation to quota internals with identity/version guards. Existing public planner/outcome assertions pass.

7. File: `codex_quota_service.rs`.
   Issue: force-query documentation still described forced OAuth refresh.
   Fix: documentation now states cache bypass and retains expiry/authentication-rejection refresh behavior.

8. File: `codex_quota_service.rs`.
   Issue: the repair fallback reread a repaired file without checking the request's initial identity before HTTP.
   Fix: the owner reuses the initial expected snapshot and final locked-snapshot verification before retry. Source-drift coverage passes.

The prior P1 review's quota writeback guards, snapshot versioned writer and runtime auth-only CAS fixes remain documented in `independent-check.md`. The final 340-test gate revalidates those paths. The new runtime CAS entry point is crate-private. Public CommitPlan and DTO shapes remain unchanged.

## Findings (not fixed)

No unresolved finding remains within the approved P1 entry points.

- Delete/rename/import do not yet participate in the full credential-operation lock interval. The main session assigned those entry points to P3/P6. This report does not claim every CCR writer is serialized.
- Platform profile application and auth-off release the synchronization lock before final runtime replacement. Full operation coverage for those entry points remains outside P1 R5. The existing auth-off rotation regression passes.
- External Codex/login writers do not honor CCR locks. Identity/version guards and leaf guarded writes remain active. Arbitrary external writes during the final CAS window are not fully serialized.
- Independent CCR lock roots and unknown sources in separate files do not form a known shared identity. The approved no-association behavior remains.
- Existing raw-token Debug/body-preview protection is P4 scope. The newly introduced complete identity is excluded from Debug/errors/logs/public DTOs.
- Real k12 recovery remains unverified. The main session observed a quota GET 401 for k12 and GET 200 for a different runtime identity. The cause remains unknown. No real credential was refreshed, replaced or printed by this reviewer.

## Verification

All listed completed gates returned native and shell exit code 0. Receipts are unchanged and indexed by SHA256 in `account-lifecycle-independent-hashes.json`.

| Gate | Result | Receipt in research/ |
| --- | --- | --- |
| `cargo test -p ccr-codex --all-features -- --skip export_bindings` | PASS: 340 passed, 0 failed, 2 ignored; doc-tests 0 | `account-lifecycle-independent-codex.log` |
| `cargo test -p ccr-tui -- --skip export_bindings` | PASS: 252 passed, 0 failed; doc-tests 0 | `account-lifecycle-independent-tui.log` |
| `cargo test -p ccr-cli --all-features application::auth_off -- --skip export_bindings` | PASS: 11 passed, 0 failed; other tests filtered | `account-lifecycle-independent-cli-auth-off.log` |
| `cargo test -p ccr-cli --all-features commands::codex::auth::current -- --skip export_bindings` | PASS: 2 passed, 0 failed; other tests filtered | `account-lifecycle-independent-cli-current.log` |
| `cargo clippy -p ccr-codex -p ccr-cli -p ccr-tui --all-targets --all-features -- -D warnings` | Lint PASS; TypeCheck PASS for affected packages | `account-lifecycle-independent-clippy.log` |
| `just fmt-check` | PASS: root/Tauri Rust, JSON and formatter tests | `account-lifecycle-independent-fmt.log` |
| `python scripts/quality/check_secret_writes.py` | PASS | `account-lifecycle-independent-secret-writes.log` |
| Scoped `git diff --check` | PASS; LF/CRLF advisory warnings only | `account-lifecycle-independent-diff.log` |

The earlier independent Codex/fmt/secret receipts were checked against unchanged final sources and retained. Successful checks were not repeated after resume. CLI/TUI/clippy completed the missing affected-package coverage.

### NOT_RUN and ignored boundaries

- Workspace `just lint-strict`, `just test`, `just ci`, and the full CLI dispatch suite: NOT_RUN here. The main session owns the parent gate after P3–P6.
- Binding generation: NOT_RUN; export_bindings is skipped under the repository test contract.
- Two Codex cache benchmarks are ignored: `benchmark_list_sessions_inventory_cache` and `benchmark_compute_rolling_usage_cache`. Their benchmark behavior is not a PASS claim.
- Cross-process OAuth HTTP/refresh/switch end-to-end flow: NOT_RUN. The actual child-process evidence covers file-lock blocking, acquisition and locked reread. HTTP concurrency tests use synthetic same-process loopback servers.
- Installed-binary interactive TUI, Unix permission tests, independent production Windows ACL verification, real OAuth/login and real-account recovery: NOT_RUN.

## Source identity, receipts and rollback

The independent manifest `account-lifecycle-independent-hashes.json` records 15 final source files, eight independent gate receipts, the unchanged earlier report hash, actual command exit codes and test counts. Its capture time is `2026-10-07T00:12:03.7210554Z`; HEAD is `c524ac07e77e94587399d969546ed75f13cfaf19`.

Comparison with `account-lifecycle-source-hashes.json` returned **0 mismatches** for the owner's final sources and receipt hashes. Final lifecycle source hashes:

| File | SHA256 |
| --- | --- |
| `codex_auth_refresh_lock.rs` | `F96EDD87B5D5486CF40D530D122F54F38806DF0DEB9E22D42F823842C0EE30FF` |
| `codex_auth_service.rs` | `B5306B70F41C42D4CBB4E8643A27203006BA7BE72CB910048044BB713B488703` |
| `codex_oauth_token_service.rs` | `D5D09966D4B2155C9B14D8D7C1F0485F77D8371F8739551082AEDB5168D64CD2` |
| `codex_quota_service.rs` | `AFF965924E7F5A92991F5BD23F08E31450988826F7E83944E280DDD028957B25` |
| `services/mod.rs` | `33A488CD50E588918D9F74999377C79425F5B9983BBBC36A81C8B0A082197DDE` |
| `codex_runtime_service.rs` | `2FF21705B1280FF73070A4B01C59B2900E2237171B36C270831A9DE4018CD3AE` |
| `openai_quota_core.rs` | `5AB1778F682EF90F552ADC3594267D1C6919921F53DC868A667350C8CA8ED328` |

Retained first-failure receipts include:

- `account-lifecycle-first-failure.log`: 0 passed / 2 failed.
- `account-lifecycle-codex-validation.log`: 330 passed / 6 failed / 2 ignored. Public outcome assertions, a Tokio fixture context failure and the routing fixture were corrected with coverage retained.
- `account-lifecycle-same-second-first-failure.log`: DateTime fixture compile failure.
- `account-lifecycle-same-second-behavior-first-failure.log`: separate behavior failure, 0 passed / 1 failed.
- `account-lifecycle-identity-upgrade-first-failure.log`: 0 passed / 1 failed, with the new refresh token submitted twice.
- `account-lifecycle-clippy-first-failure.log`: two collapsible_if findings; the owner made local let-chain fixes.
- Prior quota/CAS first-failure receipts remain in the task research directory.

First-failure source bytes and source hashes were **NOT_CAPTURED**. Final source hashes are not attributed to those failures. Receipt bytes and the earlier independent report remain unchanged.

Rollback requires a scoped revert of approved P1/R5 source changes while retaining P2 and unrelated edits. There is no registry schema or snapshot-path migration in this extension. Existing secret atomic-write and backup contracts remain. Runtime CAS conflict does not restore an older auth backup over a newer runtime. No rollback, commit, archive or push was performed.

P1 approved-scope review is complete. The main session may proceed to P3 under the accepted workspace-gate sequence.
