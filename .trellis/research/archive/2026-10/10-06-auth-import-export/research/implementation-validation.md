# P6 implementation validation

Status: SCOPED_IMPLEMENTATION_DONE; CURRENT_PARENT_GATE_PENDING.

Product source status: CHECK_FINAL_SOURCE_FROZEN at 2026-10-07 02:33:43 UTC.
No product source changes are pending in the owner session.
Parent owns task state, acceptance criteria, specs and the final workspace gate.

## Scope and source versions

The approved behavior rejects known import identity conflicts and preserves
existing encrypted export rules. The owner changed four product files:

| File | SHA256 at source freeze |
| --- | --- |
| `crates/ccr-codex/src/services/codex_auth_service.rs` | `d0ba7f0924fdfdb6e714476624a3f69d3d25dd630fe1be95598c08aa5fa3112b` |
| `crates/ccr-codex/src/services/codex_auth_identity.rs` | `22fccbeb10626998637bef7925a4f11bae3f66ff18d881d949e5fe3b22a770aa` |
| `crates/ccr-codex/src/services/codex_auth_refresh_lock.rs` | `fd1fbb64d499a896e2087a848634cd96fa6f15bfec5c0ed1cda3bd76df8cfd3d` |
| `crates/ccr-cli/src/commands/codex/auth/import.rs` | `11197b8cdb56709233defb5bab70340d22cbd5fa9a3d733077fd18dfcc507b42` |

These files also contain earlier approved P1-P4 changes. P6 ownership does not
authorize reverting those changes. P6 added no public DTO, crypto format,
export behavior, Tauri implementation or general transaction API.

## Implemented behavior

`codex_auth_service.rs:2058` retains typed `CodexAuthExport` parsing. Parse
diagnostics contain line/column only. A malformed field in a skipped entry
still fails typed parsing. Merge/no-force skips existing accounts after
parsing, and the legacy provider skip remains unchanged.

Preflight at `codex_auth_service.rs:2084` validates all pending entries before
directory creation, backups, lock files or writes. Known token account IDs
must equal the raw metadata value. Leading/trailing metadata whitespace is a
conflict. API fingerprints and incompatible auth methods are checked.
`codex_auth_identity.rs:53` supplies account context; `:66` detects
contradictory account/user claims. Missing complete claims retain P1 unknown
identity behavior. Metadata-only entries retain legacy behavior.

Preflight captures snapshot bytes and versions. Admission at
`codex_auth_service.rs:2167` acquires stable path, old identity and incoming
identity resources in sorted order through
`codex_auth_refresh_lock.rs:88`. The service re-reads the registry and checks
writable schema, source identity, snapshot content version and account
existence. Same-identity rotation during lock wait returns a retry error and
retains the rotated snapshot. Locks remain held through registry save at
`codex_auth_service.rs:2274`.

Existing targets use `enforce_owner_only_permissions_versioned` at
`codex_auth_service.rs:2207`. A false result returns a retry error. Payload
replacement at `:2215` uses a secret versioned guarded write, without
pre-deleting the snapshot. The private admission callback follows the
existing backup callback pattern and supports deterministic tests.

The plaintext CLI branch at `import.rs:137` now returns the service error.
The CLI test calls that internal function and checks a nonzero error code.
Encrypted import continues to call the same typed service import path.
Encrypted export entry assessment is in `encrypted-export-assessment.md`.

## Owner checks before reviewer test addition

All Rust tests use default parallelism with `--skip export_bindings`.

The owner checks below used source freeze `2026-10-07 02:30:35 UTC` with
service SHA256 `228624f62c08990887442e636c5ea02d9f8bd073bf3c1ab528274597d75f923f`.
The other three source hashes equal the final source versions. The reviewer
then added two entries to the existing claim matrix without changing
production behavior. These owner receipts remain evidence for the earlier
source version.

| Command | Result | Receipt |
| --- | --- | --- |
| `cargo test -p ccr-codex -p ccr-cli --all-features p6_ -- --skip export_bindings` | PASS; 13 Codex + 1 CLI tests; native exit 0 | `owner-final-targeted.log` |
| `cargo clippy -p ccr-codex -p ccr-cli --all-targets --all-features -- -D warnings` | PASS; native exit 0 | `clippy-corrected-validation.log` |
| `just fmt-check` | PASS; exit 0 | `owner-final-fmt.log` |
| `python scripts/quality/check_secret_writes.py` | PASS; exit 0 | `owner-final-secret-writes.log` |

The 14 owner targeted results are recorded individually below.

| Test | Result and coverage |
| --- | --- |
| `p6_import_rejects_metadata_conflict_before_any_filesystem_change` | PASS; fixed diagnostic and unchanged full tree |
| `p6_import_valid_first_invalid_second_preserves_complete_tree` | PASS; no earlier writes when a later entry fails |
| `p6_import_rejects_conflicting_token_claims` | PASS; contradictory token identity rejected |
| `p6_import_waiting_on_credentials_retains_same_identity_rotation` | PASS; lock wait followed by raw version recheck |
| `p6_import_claim_matrix_preserves_legacy_and_api_compatibility` | PASS; complete, incomplete, absent-context and API cases |
| `p6_import_rejects_whitespace_metadata_identity_before_side_effects` | PASS; raw metadata account comparison for OAuth/API |
| `p6_import_skipped_entries_keep_typed_shape_validation` | PASS; skips preserve typed parsing and redacted errors |
| `p6_import_metadata_only_force_and_replace_keep_existing_semantics` | PASS; force backup/deletion and Replace retention |
| `p6_import_intentional_identity_replacement_and_order_are_preserved` | PASS; replacement, unrelated accounts, counts/order |
| `p6_import_encrypted_conflict_uses_identical_preflight` | PASS; encrypted input uses common preflight |
| `p6_import_locks_incoming_identity_shared_with_another_alias` | PASS; incoming identity contention is serialized |
| `p6_import_force_backup_failure_keeps_snapshot_and_registry` | PASS; backup failure preserves original payloads |
| `p6_import_rechecks_registry_version_after_credential_admission` | PASS; incompatible registry after admission is rejected |
| `p6_plaintext_import_returns_service_rejection` | PASS; internal CLI function returns nonzero error code |

The owner also completed the following broader tests before the three
equivalent Clippy condition collapses in the frozen service source. These
receipts remain historical to that source version. Independent review will
validate the final source separately.

| Command | Result | Receipt |
| --- | --- | --- |
| `cargo test -p ccr-codex --all-features -- --skip export_bindings` | PASS; 381 unit tests, 2 ignored, 0 failed; 0 doc tests; native exit 0 | `codex-validation.log` |
| `cargo test -p ccr-cli --all-features -- --skip export_bindings` | PASS; 347 unit tests, 24 binding tests filtered; 12 integration tests; 1 doc test, 1 ignored; native exit 0 | `cli-validation.log` |

## Independent checks and final test amendment

Independent full Codex tests on owner service hash `228624f6...` passed:
381 unit tests, 2 ignored, 0 failed and 0 doc tests; native exit 0. The receipt
is `independent-codex-validation.log`. That full receipt predates the final
test-only amendment.

The reviewer added `tokens + Api` and `API key + Chatgpt` rejection cases to
`p6_import_claim_matrix_preserves_legacy_and_api_compatibility`. Both cases
assert unchanged full trees. The amendment changes only the existing test
matrix, producing final service hash `d0ba7f09...` above.

The reviewer ran
`cargo test -p ccr-codex --all-features p6_import_claim_matrix_preserves_legacy_and_api_compatibility -- --skip export_bindings`:
1 passed, 0 failed; native exit 0. The receipt is
`independent-method-matrix-validation.log`. Final `just fmt-check` also
passed; receipt `independent-final-fmt-validation.log`. Reviewer scoped
`git diff --check` passed. The parent formal gate owns complete validation of
the final source version.

Scoped tracked `git diff --check` passed for service and CLI source.
Untracked helper and owner report whitespace checks are recorded separately
in `owner-final-whitespace.log`; raw receipts retain original CRLF. The
earlier corrected check is in `owner-whitespace-corrected-validation.log`.
The first whitespace wrapper returned expected no-index difference
exit 1 after checks found no errors; `owner-whitespace-validation.log`
preserves that result. The corrected wrapper returns exit 0 when no check
reports whitespace errors.

## Failure and receipt audit

The first Codex probe failures remain in `first-failure.log` and their source
versions in `first-failure-source-hashes.json`. The swallowed CLI error is in
`cli-first-failure.log`. `first-retest.log` preserves the invalid
`OpenAiAuthMethod::OAuth` compilation attempt; the corrected source uses
`Chatgpt`. Corrected compilation, matrix and targeted receipts remain beside
the first failures. `clippy-validation.log` preserves three P6
`collapsible_if` failures. The three conditions were collapsed and strict
Clippy was rerun successfully.

| Receipt | SHA256 |
| --- | --- |
| `owner-final-targeted.log` | `64d3914441dff6777c8b8bce600c375c0622b8c632198861bda65eff4259ae99` |
| `clippy-corrected-validation.log` | `4e5ae76df834e36fd487cafbf532a5e3879ff4c9df9e196ea2f1ac12411f8feb` |
| `owner-final-fmt.log` | `9d12c08e230ab6a9e5edfe2e6646bcfea41d5db000ea1f5d522c37fd6b2d3516` |
| `owner-final-secret-writes.log` | `fe0a988cf45b910c72c002035170958dae88690e56a42299659975baa7637a51` |
| `codex-validation.log` | `a365b555c8b1cd43853ed2d0552e730314f87637f1ada128c684152648d79212` |
| `cli-validation.log` | `70ecae121dec1d0e7ed680892d46b44fc116cebc9e8a2ae9e98b64f1071abe91` |
| `clippy-validation.log` | `ba49f520f7426d64c73ecf4a9170c27712fb4b2c16ce6c54279bb67236f8b43a` |
| `first-failure.log` | `35d241a55348fb8ecf528dff3a0e8f3f2fc07335153ff3e4cd537d7a114f416c` |
| `cli-first-failure.log` | `d5bf112ed39c44eeb9a659ebb618a604db8eacdca97b65eed2dae30e46b63220` |
| `first-retest.log` | `e04913b72b3ad06671e443d28380cd3195110664371e87ab57f237c1ef246f3a` |
| `independent-codex-validation.log` | `6a720a0209526bfab45b14e8f5c582721f0c72c79f27584ca4dc31e71f459d9c` |
| `independent-method-matrix-validation.log` | `5b57b69bf2bb04ca98d5ad1a809c36197f099228b553423e5086134ea035e6c0` |
| `independent-final-fmt-validation.log` | `d452e7f4494e729b37f00afecad196f68bde6b3a0ec56cbd78af4d4ee170e379` |

`implementation-hashes.json` records the frozen source, owner reports and
all owner receipt hashes. The manifest excludes its own hash.

## Safety and rollback boundaries

- Input validation failure leaves snapshot, registry, backup and lock trees
  unchanged. The synthetic tests scan the complete temporary home.
- Force replacement of an existing account backs up registry and snapshot
  before destructive work. A backup failure returns an error before payload
  replacement. Metadata-only force retains historical snapshot deletion.
- Replace/no-force retains historical behavior: no snapshot preimage backup
  is added. Metadata-only Replace/no-force retains old credentials, while
  imported metadata may differ. Unrelated accounts remain present.
- A completed explicit force retry may replace rotated credentials. P6 does
  not invent a token age rule for imported inputs.
- Snapshot/registry I/O failures can partially complete a multi-entry import.
  Registry commit failure has no general rollback. P6 adds no multi-file
  transaction or guarantee against writers that bypass these locks.
- Tauri object/array imports retain per-item commits. A later failure can
  leave earlier items committed. No Tauri behavior change was implemented.
- Data recovery for force replacement uses the captured account and registry
  backups. Replace/no-force does not provide a captured old snapshot.
  Source rollback must remove only P6 hunks from the four listed files and
  preserve earlier approved changes and unrelated dirty files.

## Unrun boundaries

NOT_RUN in the owner session: real accounts, real network endpoints, user
credential import/export, installed binary behavior, hosted CI and native
Unix behavior. Synthetic Windows tests do not establish those results.

Formal `just lint-strict`, `just test`, and `just ci` are pending in the parent
workflow. Scoped owner and reviewer checks do not replace those formal gates.
No commit, archive, push or installation occurred.
