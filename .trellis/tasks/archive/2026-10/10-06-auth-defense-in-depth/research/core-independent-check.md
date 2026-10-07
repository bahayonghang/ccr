# P4 independent core review — PART 1

- Task: `.trellis/tasks/10-06-auth-defense-in-depth`.
- Reviewer: `trellis-check` (`/root/check_p1`).
- Status: **CORE_REVIEW_PASS / P4_PART2_PENDING / WORKSPACE_NOT_RUN**.
- Date: 2026-10-06 America/Chicago. Raw receipt timestamps remain unchanged.
- Scope: frozen `crates/ccr-core/src/core/{atomic_writer,guarded_write}.rs`; read-only product review.
- No product edits, Cargo invocation, workspace format operation, credential operation, installation, commit or archive.
- Codex/TUI implementation belongs to the other active owner. Consumer behavior and complete P4 acceptance remain pending PART 2.
- Inputs: latest check.jsonl, PRD/design/implement, listed applicable specs and design research, core-implementation.md, core-implementation-hashes.json, first-failure identity records and raw core receipts. Previously read unchanged error-freeze/audit contracts were reused.
- The reviewer read the new seven-section `Versioned owner-only permission hardening` spec at `atomic-writer.md:321`.

## Core contract review

| Contract | Result | Code and receipt evidence |
| --- | --- | --- |
| Exact public entry point | STATIC_REVIEW_PASS | `guarded_write.rs:197` implements the approved `enforce_owner_only_permissions_versioned(path, expected_token, lock_timeout) -> Result<bool>`. Existing `core::guarded_write` is public at `core/mod.rs:18`. core/mod is unchanged. |
| Raw content version | STATIC_REVIEW_PASS | The helper opens the target under the existing guarded leaf lock, reads the original bytes from that handle and compares content_version_token. No JSON parser or reserialization supplies the version. |
| Verified Windows handle | STATIC_REVIEW_PASS | The open access mask includes GENERIC_READ and WRITE_DAC. The same open File that supplies bytes is passed to `atomic_writer.rs:721`; SetSecurityInfo updates that HANDLE. A later path lookup does not choose the metadata target. |
| Process-token owner-only policy | STATIC_REVIEW_PASS / NATIVE_RECEIPT_PASS | `owner_only_windows_dacl` at `atomic_writer.rs:633` obtains the current process token user SID and builds protected D:P(A;;FA;;;SID). The new helper applies only DACL_SECURITY_INFORMATION plus PROTECTED_DACL_SECURITY_INFORMATION. Owner/group/SACL fields are not requested. No USERNAME lookup or icacls process exists in the production helper. |
| Missing/conflict/error behavior | STATIC_REVIEW_PASS / NATIVE_RECEIPT_PASS | Missing opens return false. After a successful readable open, a raw-byte mismatch returns false before permissions change. Lock/open/read/DACL failures propagate existing errors. Windows WRITE_DAC failure can precede the version comparison; I/O failure is not converted to conflict. |
| Bytes, mtime and file identity | STATIC_REVIEW_PASS / NATIVE_RECEIPT_PASS | No payload write, replacement or backup occurs. The Windows regression checks bytes, modification time and volume/file-index identity after the first and repeated call; it also checks directory entries for no backup. |
| Journal checks | STATIC_REVIEW_PASS / NATIVE_RECEIPT_PASS | before_write runs before metadata changes. The helper does not call committed and adds no content rollback entry. Fault and stale journal-version tests preserve old metadata; the success test has empty changed_paths and rollback lists. |
| Unix mode rule | STATIC_REVIEW_PASS / NATIVE_NOT_RUN | The helper reuses secret_unix_mode. Test source covers 0644→0600, 0600→0600 and 0400→0400, with bytes/mtime/inode checks. Unix metadata changes call before_write and sync_all. Native Unix and cross-compilation were not run. |
| Existing preservation APIs | STATIC_REVIEW_PASS / REGRESSION_RECEIPT_PASS | The old enforce_secret_permissions_versioned body is unchanged. secret_windows_dacl still captures an existing target's DACL; only its previously existing new-file SID construction was extracted. Existing AtomicWriter metadata restoration/preservation paths remain unchanged. |
| Dependency/error/scope | STATIC_REVIEW_PASS | The diff adds code in two approved files. No CcrError variant, dependency, module export or journal protocol changed. |

## Native test review

The Windows filtered group reports eight tests. Seven execute distinct operations in the normal host run; the eighth is a child-only entry that returns in the normal host. The parent USERNAME test launches a real test process, removes USERNAME only from that child environment, and makes the child execute the new API and SID assertion. The child-only entry is not counted as a second independent policy proof.

Reviewed coverage:

- Wide Everyone DACL becomes protected, with one noninherited Allow FullControl ACE.
- ACE SID bytes equal an independent OpenProcessToken/GetTokenInformation/GetLengthSid result. Assertions do not print SID bytes.
- Repeated matching calls retain payload, mtime and file identity; no backup is created.
- Missing file and readable version mismatch have no permission change.
- A preheld guarded leaf lock returns LockTimeout without changing payload.
- Native OWNER_RIGHTS denial of WRITE_DAC returns PermissionDenied. The test retains a previously granted restoration handle and restores the synthetic fixture policy after checking unchanged metadata.
- Injected journal policy failure and expected-version conflict precede metadata changes.
- Successful metadata hardening adds no content journal entry and is not automatically widened by journal rollback.

The USERNAME child is native process evidence for the environment-independent policy. The core review does not claim a complete Codex/TUI credential lifecycle or cross-application transaction.

## Findings (fixed)

No new core defect was found. The reviewer made no product fix.

The owner corrected the old preserve-helper policy mismatch by adding the separate approved owner-only API. The native first-failure probe and repaired probe remain preserved. No change was made to the old preservation API to force a new policy on existing callers.

## Findings (not fixed)

No unresolved core finding blocks PART 1.

- The original native startup failure `0xc0000005 / STATUS_ACCESS_VIOLATION` remains unexplained. The startup-failing run has no running-tests output. Later temporary no-incremental and default runs passed. The source identity and default EXE identity match the failure record. Cause unknown; no compiler, incremental-cache or corrupt-binary diagnosis is established.
- The leaf lock coordinates cooperating CCR writers. External writers can alter content or replace the path during an operation. Using the verified open handle prevents a later path lookup from selecting a replacement object; it does not provide a general external-write transaction.
- Permissions hardened without a content journal entry are not automatically relaxed by rollback. An active older content entry retains its existing compensation policy.
- Unix native behavior and cross-compilation remain NOT_RUN.
- Codex unchanged-source selection, fallible consumer propagation, rename fallback, Debug/HTTP redaction and TUI matrices are outside this PART 1 review and await PART 2.

## Verification and provenance

No gate was repeated. The main session explicitly required reuse of unchanged successful core receipts while the Codex/TUI owner held Cargo.

| Command/evidence | Reviewed result | Exit provenance |
| --- | --- | --- |
| `cargo test -p ccr-core --all-features owner_only_permissions_tighten_windows_dacl_without_replacement -- --skip export_bindings --nocapture` using the old helper | Preserved red: 0 passed / 1 failed | Owner exit 101; `core-first-failure.log` |
| Repaired single native probe | PASS: 1 passed | Owner exit 0; `core-first-retest.log` |
| Default filtered test EXE before recovery | Preserved startup error; no tests ran | Child `0xc0000005`; `core-metadata-retest.log` |
| Temporary `CARGO_INCREMENTAL=0` list and filtered group | PASS: eight tests listed, then 8 passed | Owner exit 0; `core-native-startup-noincremental.log`, `core-metadata-noincremental-retest.log` |
| Temporary no-incremental full package | OWNER_RECEIPT_REUSED_PASS: unit 130; doc 6 passed / 6 ignored | Owner exit 0; `core-validation-noincremental.log` |
| Default `cargo test -p ccr-core --all-features -- --skip export_bindings` | OWNER_RECEIPT_REUSED_PASS: unit 130 / 0 failed; doc 6 passed / 6 ignored | Owner exit 0; `core-validation-default-first.log` |
| `cargo clippy -p ccr-core --all-targets --all-features -- -D warnings -D clippy::unwrap_used` | Lint/TypeCheck OWNER_RECEIPT_REUSED_PASS | Owner exit 0; successful finish in `core-clippy-validation.log` |
| Owned `rustfmt --check --edition 2024` | OWNER_RECEIPT_REUSED_PASS | Owner exit 0; `core-fmt-validation.log` is the unchanged empty success receipt |
| Scoped two-source `git diff --check` | OWNER_RECEIPT_REUSED_PASS | Owner exit 0; `core-diff-validation.log` contains only LF/CRLF advisory warnings |
| SHA256 comparison by reviewer | PASS: 0 mismatches | Reviewer read-only command exit 0; all 15 manifest entries checked |

The manifest has **2 final source files and 13 evidence files: 10 raw command logs, 2 JSON identity records and 1 implementation report**. All match their stored hashes. These counts do not claim 13 separate raw command logs.

### Final source identity

| File | SHA256 |
| --- | --- |
| `core/atomic_writer.rs` | `bf35a7c546aa08c66f6542479e7e31f9fd5a60e496074fcd9f69090bb4e3108a` |
| `core/guarded_write.rs` | `28c10c615969be537a0fa165f9231df2b37fe75781f0182a75f83879726db00e` |

The reviewer read the current default test EXE hash without launching the EXE. `target/debug/deps/ccr_core-b15e5fcb53867454.exe` is `5d93cfa6611173a8a25ebac7c763cb46ff17aa530fe984f268bc0c24b6ef61b0`, matching `core-native-startup-failure-hashes.json` and the owner's later default PASS report. The recorded Application Error fields identify the same core module, exception c0000005 and offset 0x3b61c0; other application/system log content was not retrieved by this reviewer.

The initial old-helper probe has separate source hashes in `core-first-failure-sources.json`. Those hashes remain distinct from final source identity. The JSON files record identities; they do not establish retention of older source bytes or a copied failing EXE. Raw failures remain unchanged.

### NOT_RUN

- Independent repeated core execution: NOT_RUN under the dispatch's source-matched receipt-reuse instruction.
- Unix native and cross-compilation: NOT_RUN.
- Complete P4 Codex/TUI acceptance, native interactive TUI, installed binary and real credentials: NOT_RUN by this PART 1 reviewer.
- Workspace lint/test/fmt/CI and hosted/cross-platform gates: NOT_RUN here.
- Six ignored doctests remain ignored; they are not counted as passed tests.

## Rollback and handoff

Source rollback must first coordinate removal of the Codex consumer calls to the new API, then revert only the P4 additions in the two core files. Preserve prior core changes. Metadata hardening is not reversed through an empty content journal.

Core products remain frozen. No review-only product mutation was made. PART 1 is complete; the root must dispatch PART 2 after the Codex/TUI owner freezes. Whole P4 is not marked complete.
