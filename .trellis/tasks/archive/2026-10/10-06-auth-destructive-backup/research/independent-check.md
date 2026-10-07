# P3 independent check: saved auth backups and destructive preconditions

- Task: `.trellis/tasks/10-06-auth-destructive-backup`.
- Reviewer: `trellis-check` (`/root/check_p1`).
- Status: **SCOPED_CHECK_PASS / WORKSPACE_NOT_RUN**.
- Date: 2026-10-06 America/Chicago. Evidence timestamps retain UTC.
- Product scope: `services/{mod,codex_auth_backup,codex_registry_store,codex_auth_service}.rs`.
- Product edits by final reviewer: 0. P1/P2 changes remain intact.
- Inputs: latest check.jsonl → listed specs/research → PRD → design → implement, current frozen source, owner report and raw receipts.
- Spec sync: the main session added `Saved Auth Backup Pools And Destructive Preconditions` at `backend-guidelines.md:313`. The reviewer read the added contract and checked the frozen implementation against all seven sections.

## Acceptance criteria

The row numbers follow the five clauses in the current PRD.

| AC | Result | Evidence |
| --- | --- | --- |
| 1. Identical registry content adds no backup; different content in one second creates separate files | PASS | `codex_registry_store.rs:183` moves the existing backup to an older timestamp filename before invoking the public backup entry point. The assertion requires that exact older path and refreshed mtime. It cannot pass through same-second overwrite. `codex_auth_backup.rs:229` verifies latest-only dedup; :162 checks exact first/second filenames and every preimage's bytes. |
| 2. All old backups remain; no count/age cleanup | PASS | `codex_auth_backup.rs:38` matches full alias plus timestamp/optional sequence. :162 retains 16 old foo backups, foo_bar and registry files, then checks the original bytes and final file count. There is no rotation/deletion branch in the new helper. Runtime auth/config keep-10 pools are unchanged. |
| 3. Delete backup restores exact snapshot bytes | PASS | `codex_auth_service.rs:1426` acquires credential resources before the locked destructive callback. It completes registry and existing snapshot backups before removal. The regression at :3554 verifies an absent snapshot and a retained backup with the exact source bytes. The blocked-directory regression at :3651 preserves snapshot and registry bytes on error. |
| 4. Force rename backup failure returns error without deleting either snapshot | PASS | `codex_auth_service.rs:1608` rereads under the registry lock and validates both locked source resources. Target, source and registry backups all precede removal/move. The test at :3572 injects each of the three backup failures and checks source, target and registry bytes after every failure. :3611 blocks the backup directory after preparation and proves final publication does not invoke a second registry backup. |
| 5. Codex tests, workspace strict lint and workspace tests | PACKAGE_PASS / WORKSPACE_NOT_RUN | Independent final-source package test: 353 passed / 0 failed / 2 existing ignored; native and shell exit 0. Owner strict clippy and fmt receipts match unchanged final sources. Workspace `just lint-strict` / `just test` remain deferred under the approved parent sequence. |

## Additional required checks

| Contract | Result and evidence |
| --- | --- |
| Exact registry/account pools | PASS: foo/foo_bar separation is tested with equal payloads, so an erroneous cross-pool dedup would fail. Registry suffix/prefix cannot match account snapshots. Timestamp recognition accepts old names and numeric suffixes. |
| Windows Foo/foo physical aliases | PASS: lowercased pool identity and pool matching select one resource. The Windows test at `codex_auth_backup.rs:372` preholds that resource, verifies another case alias waits, then confirms two distinct payload versions after release. File layout and display alias spelling remain unchanged. |
| Cross-process same-second preimages | PASS: :333 launches three actual child test processes with one lock root and one fixed timestamp. The parent verifies all three different payloads remain. :281 covers eight threads. These are backup-file tests; they do not claim cross-process HTTP lifecycle coverage. |
| Native new-backup privacy | PASS on Windows: :372 runs a native Windows PowerShell/.NET ACL probe. The DACL is protected and contains exactly one Allow FullControl rule for the current process user SID. The probe does not print the SID. Secret guarded atomic publication establishes policy before payload writes. |
| Ordinary registry save | PASS: `codex_registry_store.rs:116` propagates `self.backup()?` before private publication. :202 blocks the backup directory and checks unchanged registry bytes. The old public load/save pattern is not promoted to a general read-modify-write transaction. |
| Latest registry preimage and readonly | PASS: `update_with_prepared_backup` at :128 holds the existing registry write lock, loads the latest registry, checks its version, runs the callback and publishes without another backup. Service preflight rejects read-only commands before credential-operation lock creation. Existing P2 delete/rename tree-preservation tests pass in the independent package gate. |
| Credential source stability | PASS: delete/rename use P1 stable source-path and known-identity resources. Each path is verified against its own captured resource inside the locked callback. Lock acquisition is sorted/deduplicated; helpers do not reacquire held credential or registry locks. |
| Refresh overlapping rename | PASS: `codex_auth_service.rs:3671` holds the quota refresh POST on a loopback server. Rename waits, then moves the snapshot containing the newly persisted refresh token. The test also checks the renamed registry record. The HTTP test uses a synthetic same-process server. |
| Lock order | STATIC_REVIEW_PASS: credential resources → registry lock → backup pool → guarded writer leaf. Independent backup takes pool/leaf only. No pool-held path acquires credential/registry locks; destructive callbacks call private write_locked instead of ordinary save. |
| Public behavior and errors | STATIC_REVIEW_PASS: no public bypass-backup switch or new CcrError variant. Existing current_auth, insertion order and usage_ledger rename rules remain; their package regressions pass. New backup errors use existing variants and fixed prefixes without payload or identity values. |
| P1/P2 regression | PASS: full final Codex gate includes identity isolation, freshness/CAS, lifecycle, manual-query, registry extras/version gates and safe token synchronization under read-only metadata. |

## Findings (fixed)

No new product finding required reviewer self-fix.

Owner fixes confirmed by source and regressions:

- File: `codex_auth_service.rs`. Issue: delete removed the snapshot without a recoverable preimage. Fix: snapshot and registry backups precede deletion under the credential and registry locks.
- File: `codex_auth_service.rs`. Issue: force rename ignored required backup errors. Fix: target/source/registry backup errors abort before either file is removed or moved.
- File: `codex_registry_store.rs`. Issue: ordinary save ignored automatic backup failure. Fix: the failure propagates before registry replacement.
- Files: `codex_auth_backup.rs`, `codex_registry_store.rs`. Issue: second-level names could overwrite distinct preimages, and a count-only dedup assertion could miss that overwrite. Fix: locked pool allocation uses unused numeric suffixes; tests require exact old path reuse and retained payload bytes.
- File: `codex_auth_service.rs`. Issue: delegated registry backup made registry_path production code unused. Fix: the helper is test-only. The owner strict clippy receipt passes.
- File: `codex_auth_backup.rs` tests. Issue: the Get-Acl probe could not load its module in the test environment. Fix: the native probe uses Windows .NET File.GetAccessControl. The independent full gate executes the corrected probe successfully.

## Findings (not fixed)

No unresolved finding blocks the approved P3 contract.

1. `codex_auth_service.rs:1675`: on Windows, a registry with separate Foo/foo entries can make force rename delete the same physical source/target file. HEAD had the same cleanup order. This is an existing account-name/physical-path boundary; the new backup-pool case test does not validate case-only rename. The main session explicitly directed the reviewer to leave this path unchanged and to assess the issue under P4 rename safety and P5 naming policy.
2. `codex_auth_service.rs:1689`: the existing rename copy/remove fallback and post-copy permission helper remain P4 scope. The reviewer did not widen P3 into permission replacement or fallback compensation.
3. `codex_model_provider_store.rs:63`: its separate provider-store pool still ignores its backup error. The main session excluded that pool from P3. It was reported and not changed.
4. Ordinary registry save still accepts caller-constructed whole-registry state. Other old load/save entry points have no new general transaction guarantee. The prepared destructive callback owns the required lock-held reread; unrelated callers are unchanged.
5. External editors/Codex writers do not join CCR locks. The final move or registry publication can fail after earlier effects. P3 guarantees unchanged source/target/registry bytes before required backup failure; it does not guarantee general multi-file rollback.
6. Identical older backups are reused as stored and receive an mtime touch. The new-backup Windows DACL test does not establish ACL repair for every historical file.

## Verification

| Check | Status | Exit/count and provenance |
| --- | --- | --- |
| `cargo test -p ccr-codex --all-features -- --skip export_bindings` | INDEPENDENT_PASS | Native 0, shell 0; 353 passed / 0 failed / 2 ignored; doc-tests 0; 21.92 seconds. `independent-codex.log` ends with `NATIVE_EXIT_CODE=0`. Default test parallelism. |
| `cargo clippy -p ccr-codex --all-targets --all-features -- -D warnings -D clippy::unwrap_used` | OWNER_RECEIPT_REUSED_PASS | Owner reported exit 0; `clippy-corrected-validation.log` has successful completion. Final source and receipt hashes match. No source change required another lint run. |
| TypeCheck | PASS | Current package test compiled the frozen source; the reused all-targets/all-features strict clippy receipt covers target construction. |
| `just fmt-check` | OWNER_RECEIPT_REUSED_PASS | Owner reported exit 0; `fmt-validation.log` records Rust/Tauri fmt, five formatter tests and 11 JSON files. Final hashes match. |
| `python scripts/quality/check_secret_writes.py` | OWNER_REPORTED_PASS | Owner recorded exit 0 and the success text in implementation-validation.md. A separate raw receipt was NOT_CAPTURED. The reviewer did not repeat an unchanged successful check under the dispatch instruction. |
| Scoped `git diff --check` | INDEPENDENT_PASS | Native/shell exit 0. Only LF/CRLF advisory warnings. New untracked backup module is covered by the successful source fmt check. |

The owner full package gate preceded the strengthened public dedup test. The reviewer therefore ran the current complete package once. Unchanged successful lint/fmt checks were not repeated.

### NOT_RUN / ignored

- Workspace `just lint-strict`, `just test`, `just ci`: NOT_RUN here; the parent coordinates these after the remaining children.
- TUI, full CLI, provider-store or other runtime backup-pool expansion: NOT_RUN; product changes are confined to Codex service storage.
- Unix owner-only tests: NOT_RUN on this Windows host.
- A separate registry-backup ACL probe, historical-file ACL repair and final-I/O multi-file compensation: NOT_RUN. The actual Windows probe covers a new account backup.
- Cross-process quota HTTP/refresh/rename end-to-end: NOT_RUN. Three real child processes validate backup preimages; the refresh/rename loopback test runs in one process.
- Two existing Codex cache benchmarks remain ignored. Binding generation is skipped under `--skip export_bindings`.
- Real credentials, real-account OAuth validity/recovery, installed binaries and interactive terminal behavior: NOT_RUN.

## Source hashes and preserved receipts

`independent-hashes.json` records final source identity, the independent native/shell result and receipt hash, and the complete retained owner hash map. Rechecks found **0 source/receipt mismatches**. The first-failure receipt SHA256 remains `197756f0fb3793d0b7e115d8a981b4e6a3ac5977bff4bfbfd37de721f9ae890b`.

| Final source | SHA256 |
| --- | --- |
| `services/mod.rs` | `3a31f79e917374e11d60d8e4522c91f5945e9c77aa5563b9e01bec73d42eef97` |
| `services/codex_auth_backup.rs` | `66e094e7ffbc39986cc46d060d2dba63ac9f8666dacefc100200acb66d39bfa1` |
| `services/codex_registry_store.rs` | `ef1cf97764606b3196f42c7264dc19e84986d3c6299f9880af8c9f3e7015957c` |
| `services/codex_auth_service.rs` | `d9250702c8e60f8810d63ee6e19889a89527fcb15c0fc0ba691a596469f36535` |

Preserved failure sequence:

- `first-failure.log`: owner exit 101; 1 passed / 3 failed, covering delete preimage, rename backup failure and registry save backup failure.
- `first-retest.log`: owner exit 0; 7 passed.
- `lifecycle-retest.log`: owner exit 101; 12 passed / 1 failed, caused by the ACL test probe.
- `lifecycle-corrected-retest.log`: owner exit 0; 13 passed.
- `clippy-validation.log`: owner exit 101; production registry_path dead code.
- `clippy-corrected-validation.log`: owner exit 0.
- `public-dedup-retest.log`: owner exit 0; 1 passed after the stronger public-path assertion.

First-failure source bytes/hashes were **NOT_CAPTURED**. Final source hashes are not assigned to the earlier red runs. No receipt was rewritten.

## Rollback and recovery

A source rollback must remove only P3 additions in the four approved files and preserve P1/P2 and other local edits. No rollback was performed.

Recovery uses a matching snapshot preimage and matching registry preimage after identity verification, with private atomic publication. P3 retains all existing backups and creates no schema/path migration. Restoring local bytes does not restore a revoked server authorization.

No real credential operation, installation, commit, archive, push or backup cleanup was performed. Product files remain frozen. The main session can continue P4 under the approved parent gate sequence.
