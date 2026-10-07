# P6 Independent Check

Status: `SCOPED_PASS / PARENT_WORKSPACE_GATE_RUNNING`.

Reviewer: `/root/check_p4` (`trellis-check`).
Task: `.trellis/tasks/10-06-auth-import-export`.
Current source freeze: `CHECK_FINAL_SOURCE_FROZEN`, 2026-10-07 02:33:43 UTC.

## Scope Review

The frozen P6 source preserves typed `CodexAuthExport` parsing, the existing
Merge/no-force and provider skip counts, and the existing encrypted export
envelope. Pending entries are preflighted before operation locks, directories,
backups, or writes. Known OAuth and API identity conflicts, contradictory
claims, incompatible methods, and metadata whitespace are rejected with fixed
diagnostics. Existing auth data is published with a secret versioned writer;
the old file is not removed before publication. Metadata-only force keeps the
existing backup/delete behavior. Replace/no-force keeps the old snapshot and
does not add a snapshot backup guarantee. Unrelated Replace accounts remain.

Admission captures raw target bytes and content versions, locks target paths,
captured identities, and incoming complete identities, then re-reads the
registry and validates each source before writing. The lock remains held
through registry save. Plaintext CLI import returns the service error. Encrypted
import reuses the same service preflight.

The current source hash for `codex_auth_service.rs` is
`d0ba7f0924fdfdb6e714476624a3f69d3d25dd630fe1be95598c08aa5fa3112b` after the
reviewer added two test-only method-conflict cases. The other three P6 source
hashes remain the owner freeze values in `implementation-validation.md`.

## P1-P6 Integration Static Review

The review covers the complete current product change set: 19 tracked Rust
files and three new crate-private helpers. The source list is recorded in
`independent-hashes.json`. Every row below is a current source review. Earlier
P1-P4 package receipts retain their original source boundaries; these rows do
not convert those receipts into final workspace results.

| Contract | Current integration evidence and result |
| --- | --- |
| P1 complete identity and compatibility | SOURCE_REVIEW_PASS. `codex_auth_identity.rs:25` derives local user/account association from id/access claims and explicit account context. `identity_from_auth` excludes API auth. Auth selection, OAuth planner/repair, quota admission/persistence and cache keys use the complete association or the existing unknown-identity fallback. JWT decoding does not establish signature validity. P6 adds conflict detection for imports without changing the incomplete-identity read contract. |
| P1 refresh and mutation admission | SOURCE_REVIEW_PASS. `codex_auth_refresh_lock.rs:69` sorts and deduplicates resources. Source admission includes stable paths and captured identities; `verify_path` checks each path against its captured source. Quota requests retain the expected credential state through persistence. P6 adds incoming complete identity resources and raw content versions, preventing a same-identity rotation during lock wait from being overwritten. The locks remain held through import registry save. |
| P2 registry schema and public boundaries | SOURCE_REVIEW_PASS. Account and registry flatten fields preserve unknown TOML values. The version gate allows reads and blocks unsupported writes. Import checks the gate before admission and after lock acquisition. Other service command checks and TUI preflight retain their earlier order. Force/Replace clears only the replaced account extensions under existing rules. Explicit CLI/Tauri/export mappings omit identity keys and extension maps. No template or generated binding change is required. |
| P3 backup pools and destructive operations | SOURCE_REVIEW_PASS. `codex_auth_backup.rs:68` coordinates exact registry/account pools, latest-content deduplication and unused sequence allocation without cleanup. Delete and force rename use the lock-held prepared callback at `codex_registry_store.rs:128`; required backups precede removal/movement. Ordinary registry save propagates backup failure. P6 force replacement keeps the existing registry/snapshot preimages. Replace/no-force adds no snapshot backup guarantee. |
| P1/P3/P6 lock order | SOURCE_REVIEW_PASS. Credential operation resources precede registry, backup-pool and guarded-writer leaf resources. The prepared destructive callback publishes without calling ordinary save again. Backup-only paths take pool/leaf locks. P6 uses ordinary final registry save while credential locks are held. No inverse acquisition was found in the reviewed paths. The public load/save pattern still has no general registry read-modify-write transaction. |
| P4 permission metadata and guarded publication | SOURCE_REVIEW_PASS. `guarded_write.rs:197` verifies raw content through a file handle before applying owner-only permissions. The Windows helper uses the process-token SID DACL; production code needs no USERNAME or external ACL command. OAuth planners remain read-only. Execution callers harden unchanged matching pairs. Runtime synchronization keeps backup-before-CAS ordering. P6 existing targets use versioned permission hardening before secret publication. Permission changes are not a two-file transaction. |
| P3/P4 snapshot movement | SOURCE_REVIEW_PASS. `codex_auth_service.rs:2281` publishes the private versioned target before removing the source in rename fallback. Required P3 backups precede the mover. A later filesystem/permission/registry failure can retain earlier effects. Successful native rename and fallback publication do not imply whole-operation rollback. |
| P4 diagnostics and TUI | SOURCE_REVIEW_PASS. Sensitive auth, token, raw sync/runtime maps and raw TOML extensions have redacted Debug implementations. HTTP error extraction returns allowed fixed codes and status. TUI localization and the composed EN/ZH size matrix preserve relogin handling. Existing local serde diagnostics remain the separate finding below. P6 parse failures report line/column and identity failures use fixed text. |
| P5 snapshot naming | ASSESSMENT_SOURCE_REVIEW_PASS. All three snapshot resolvers retain `auth/<name>.json`. P6 follows the same paths and alias keys. No naming migration was implemented. P5's deferred migration decision and Windows case-only alias finding still apply. |
| P6 service, CLI and export entry points | SOURCE_REVIEW_PASS. The complete typed bundle is parsed before all pending entries are validated. Merge/no-force skips only identity validation of existing entries; malformed typed shapes still fail. Encrypted import shares the same preflight. Plaintext CLI import returns service Err. Credential-bearing CLI export still encrypts. TUI has no Auth import/export entry; Tauri DTOs and per-item object/array commits remain unchanged. No crypto format, dependency, public error variant or export default changed. |
| Remaining affected files | SOURCE_REVIEW_PASS. `services/mod.rs` registers the three private helpers. `codex_model_provider_store.rs` propagates the changed permission Result. `platforms/codex.rs` contains fixture changes required by complete identity. CLI auth-off/current and the TUI service wrappers preserve the reviewed P1/P2 flow. No unrelated product refactor was found. |

The new P6 Codex full-package receipt includes the existing P1-P4 regressions
on the owner service version. The current-source targeted matrix receipt
covers the reviewer's test-only amendment. The parent workspace gate owns
complete final-source acceptance across packages.

## Findings (fixed)

- File: `crates/ccr-codex/src/services/codex_auth_service.rs`
- Issue: the existing P6 claim matrix did not exercise `auth_method = "api"`
  with token auth or `auth_method = "chatgpt"` with API-key auth.
- Fix: added both incompatible method cases to the existing matrix. Each case
  asserts rejection and complete temporary-tree preservation.

## Findings (not fixed)

- `OpenAiQuotaCore::extract_account_id` at `crates/ccr-codex/src/services/openai_quota_core.rs:348`
  checks a namespaced `account_id` fallback but does not check the namespaced
  `chatgpt_account_id` fallback supported by the P1 identity helper. A claim-only
  import can therefore have a complete local identity while quota account-id
  extraction remains unavailable. This is an inherited P1/quota boundary, not
  a P6 import regression. Status: SOURCE_REVIEWED / RUNTIME_NOT_RUN. Parent
  directed the reviewer to report the finding without expanding P6. A follow-up
  can align the supported claim fallback and add a synthetic quota regression.
- `plan_runtime_sync` and quota snapshot parser errors at
  `crates/ccr-codex/src/services/codex_oauth_token_service.rs:681,686,970` and
  `crates/ccr-codex/src/services/codex_quota_service.rs:874,901` still format
  serde parser errors. P4's fixed HTTP diagnostic contract does not cover these
  local parser paths. No P6 change was made because the source paths are outside
  the import contract and no runtime leak was established.
- Force replacement has backup coverage only for an existing account. The
  approved Replace/no-force path retains its previous snapshot behavior and has
  no new snapshot preimage guarantee. Multi-entry I/O and registry publication
  remain non-transactional. External writers do not join CCR locks.
- Windows case-only account aliases remain a product decision. Unix permission,
  installed binary, interactive TUI, real-account/OAuth, hosted CI, and native
  cross-platform evidence remain unverified.

## Verification

- Frozen-source full Codex package test before the test-only matrix extension:
  PASS, exit 0; 381 passed, 0 failed, 2 ignored, 0 doc-test failures.
  Receipt: `research/independent-codex-validation.log`.
- Current-source incompatible-method matrix after the reviewer test extension:
  PASS, exit 0; 1 passed, 0 failed. Receipt:
  `research/independent-method-matrix-validation.log`.
- Current-source format check after the reviewer test extension: PASS, exit 0.
  Receipt: `research/independent-final-fmt-validation.log`.
- Final source/contract/reviewer whitespace check: PASS, wrapper exit 0;
  tracked native exit 0 and untracked expected difference exits 1. The tracked
  output contains LF/CRLF advisory warnings. Receipt:
  `research/independent-whitespace-corrected-validation.log`.
- Owner P6 manifest comparison: PASS; 31 entries, 31 matches, 0 mismatches.
- Preserved core manifest comparison: PASS; 15 entries, 15 matches, 0
  mismatches. The original core startup failure remains preserved; its cause
  remains unknown.
- Owner frozen-source targeted P6 receipt: PASS, 13 Codex tests and 1 CLI test;
  owner report records the matching source hashes before the reviewer test-only
  extension.
- Owner corrected scoped Clippy and secret-write checks: PASS for the owner
  source receipt. Parent must run the formal workspace checks for the current
  test-extended source.
- Premature reviewer test attempt: `INTERRUPTED_UNVERIFIED`, exit 1, no Cargo
  output. Receipt: `research/independent-premature-attempt.log`.
- Reviewer whitespace wrapper failures are recorded in
  `research/independent-whitespace-first-failure.md`. The first wrapper treated
  CRLF as trailing whitespace. A later wrapper had a PowerShell argument error
  and produced an invalid receipt. Its file,
  `independent-final-whitespace-validation.log`, is not acceptance evidence.
  The full original first-failure raw log was overwritten; only the tool
  transcript and the explicit failure note remain. Corrected checks use Git's
  `cr-at-eol` policy and do not rewrite historical raw receipts.

Lint: owner receipt PASS for the owner source version. TypeCheck: owner
Clippy receipt PASS and current-source targeted test compilation PASS.
Tests: independent Codex package PASS before the test-only extension;
current extended matrix PASS. Complete final-source package and workspace
verification remain parent-owned.

Parent started `just ci` after source freeze. Formal workspace results are
pending; this report does not claim those gates. No further Cargo command or
product edit was performed while the parent gate runs.

## Evidence Identity And Rollback

`independent-hashes.json` records final product sources, relevant contracts,
the owner manifest and receipts, earlier scoped review reports, and reviewer
report/logs. The manifest excludes itself. Reuse requires a fresh comparison;
later source or report changes keep these hashes as historical evidence.

The reviewer product change is limited to two entries in the existing P6
method-conflict matrix. Reverting the reviewer amendment removes only those
entries. P6 source rollback must retain earlier P1-P4 changes. Force-replacement
data recovery uses matching registry/snapshot backups. Replace/no-force has no
new snapshot preimage. No rollback, commit, archive, push, installation,
real-account action or task-state change was executed by this reviewer.
