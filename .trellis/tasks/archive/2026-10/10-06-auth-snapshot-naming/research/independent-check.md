# P5 Independent Assessment Check

Status: **ASSESSMENT_SCOPE_PASS / PRODUCT_MIGRATION_NOT_IMPLEMENTED**.

- Task: `.trellis/tasks/10-06-auth-snapshot-naming`.
- Reviewer: `trellis-check`, `/root/check_p4`.
- Review freeze: 2026-10-06 21:10:26 America/Chicago; 2026-10-07 02:10:26 UTC.
- Approved scope: evaluate snapshot naming and write task evidence. The product whitelist is empty. Migration requires a separate user decision and approved implementation design.
- Inputs: current PRD/design/implement/check context, design research, assessment and validation record, applicable Codex/core specs, archived F13 audit, P3 independent finding and current source blocks.
- Product, spec, task pointer and PRD edits: none. This reviewer added this report only.

## Findings (fixed)

No documentation defect required a correction. The frozen owner assessment and validation record remain unchanged.

The owner validation record preserves the first whitespace failure: a blank line at EOF caused wrapper exit 3; the owner removed the blank line and retested both files. P5 has no separate raw first-failure log. The record is owner-reported evidence; the reviewer independently checked the final files.

## Findings (not fixed)

No unresolved documentation finding blocks the assessment scope.

- `codex_auth_service.rs:1679` still permits separate `Foo`/`foo` registry keys. On a case-insensitive Windows filesystem, forced target deletion can remove the same physical source before the later source-existence check. The assessment cites P3 and current code, labels P5 native reproduction NOT_RUN, and makes physical-path overlap a migration constraint. The approved evaluation does not authorize a product fix.
- Migration trigger, shared versus per-alias storage, duplicate-identity deletion, non-OAuth/missing-identity naming and downgrade compatibility remain product decisions. The defer-migration conclusion identifies those decisions; the evaluation does not select them implicitly.
- Migration, rollback execution, synthetic migration failure matrix, native Windows case-only rename, Unix naming/permissions, real accounts, user directories, installed binaries and hosted CI remain NOT_RUN.

## Requirement And Source Review

| Contract | Result and evidence |
| --- | --- |
| Evaluation-only authorization | PASS. PRD Goal, design and implement whitelist require an assessment. The report recommends deferral and reserves implementation for a separate user decision. R2/R3 remain conditional and unimplemented. |
| R1 conclusion and benefits | PASS. The report compares retained alias paths, base64url complete identity and opaque IDs. Decoupling can remove rename file movement only after reference semantics are defined. The current P1 association fix does not depend on renamed snapshots. |
| Three current resolvers | PASS. AuthService `:191`, OAuthTokenService `:153` and QuotaService `:854` all build `auth/<name>.json`. None derives the filename from identity_key. |
| Naming constraints | PASS. `validate_account_name` at `codex_auth_service.rs:1734` accepts 1-32 ASCII alphanumeric/underscore/hyphen characters, permits case variants, and reserves exact lowercase default. The report separates character validation from physical-path uniqueness and flags case/length constraints for a new encoding. |
| OAuth/API/provider/missing identity | PASS. `codex_auth_identity.rs:20-65` builds user/account association from supported JWT claims and token account context. Conflicting/missing user/account values cannot form the key; API keys have no OAuth identity. API/provider account_id fingerprints remain separate. JWT decoding does not verify signatures. |
| Duplicate aliases | PASS. Save checks alias keys, not identity uniqueness. Per-alias metadata and files remain separate. Selection prefers current_auth, latest last_used and insertion order. Alias token synchronization keeps identity/freshness checks; the source contains no shared snapshot reference/deletion rule. |
| Save/rename/delete | PASS. The report matches captured source/target locking, per-alias writes, rename prebackups and metadata/ledger updates, and delete backup/removal. Shared targets require a new reference policy; P4 fallback and later registry failures can have earlier file effects. |
| Import/export | PASS. Exports remain keyed by alias; include_secrets selects auth_data and the DTO omits identity_key/extras. Legacy provider records are skipped. Import handles Merge/Replace/force, optional auth_data, existing paths and registry publication. The report does not claim that export metadata alone can name an OAuth snapshot. |
| Repair/quota/backup | PASS. Repair candidates use complete identity and freshness, while the target remains an alias path. Quota persistence verifies expected tokens, known identity and raw content version; missing-identity same-file refresh remains available. Backup pools remain alias based; Windows case folding coordinates backup matching/locks without rejecting registry aliases. |
| R1 migration cost and rollback | PASS. The report requires dual reads and priority, preimage backups, a path/reference manifest, per-item content versions and state, interruption recovery and commit compensation. Rollback preserves external new credentials on conflict. The report covers old-file retirement, old-client writes and server-side token invalidation limits. |
| No migration acceptance claim | PASS. The report marks layout changes and no-move rename unimplemented. Historical P1-P4 receipts are not presented as migration acceptance. |

## Verification

| Command or check | Result |
| --- | --- |
| `git status --short --branch -uall` | PASS, exit 0; preserved the existing dirty worktree. |
| PowerShell path:line probe plus source-block reads | PASS, exit 0; 26 full-path references across 7 source files, 0 missing/out-of-range references. Shorthand line anchors and cited operations were read separately. The 26 count covers full-path references only. |
| `Get-FileHash -Algorithm SHA256` for cited sources and owner reports | PASS, exit 0; 7 source hashes match the owner validation record and both owner report hashes match the dispatch freeze. |
| `git -c core.autocrlf=false diff --no-index --check -- /dev/null <file>` for both owner documents | PASS; each native exit 1 is the expected no-index difference, with 0 whitespace diagnostics. The wrapper exits 0 only for expected 0/1 and no diagnostics. |
| Rust tests, lint and TypeCheck | NOT_RUN; no product change in the approved P5 scope. |
| Product migration and native/real-account checks | NOT_RUN; no implementation authorized. |

The current assessment satisfies R1. Main owns acceptance/status writeback. The Rust gate clause in the planning-seed PRD has no new P5 execution evidence; the approved implementation plan explicitly avoids repeating Rust gates for this documentation-only assessment.

## Frozen Source And Report Identity

| File | SHA256 |
| --- | --- |
| `crates/ccr-codex/src/models/codex_auth.rs` | `e4ad12f408f270c6b44f64af1753df04d2777ea33ccee1e7c30b6134246b137d` |
| `crates/ccr-codex/src/services/codex_auth_service.rs` | `43301314421e7521a016a4dd15a0936cc0a6ccc92442928931fe05a95c08f96f` |
| `crates/ccr-codex/src/services/codex_oauth_token_service.rs` | `f7453a34c9c13ad7261e616c927aefd90cad8d42b7c050ecb2ffadfffd3d3124` |
| `crates/ccr-codex/src/services/codex_quota_service.rs` | `053013df34b54c603a08d2f40d05e3a8837fe161e35616af053c87e1e7d2aca7` |
| `crates/ccr-codex/src/services/codex_auth_identity.rs` | `2e1cb09becb4a8fc2017d758f2fbf4bfe96ba86d7e666ccc1b74b96af5435345` |
| `crates/ccr-codex/src/services/codex_auth_backup.rs` | `66e094e7ffbc39986cc46d060d2dba63ac9f8666dacefc100200acb66d39bfa1` |
| `crates/ccr-codex/src/services/codex_registry_store.rs` | `d5dcdb9415c537b783aabab3eebf03931a68ab87a3299f4a1aac1c4ea9c14abd` |
| `research/snapshot-naming-assessment.md` | `6990cdd84d13db7366964f3c8cac36e9dc1e335fcf20846f63cf7bde676efc62` |
| `research/implementation-validation.md` | `01153c70a7aa621f31ab9d0745de373f8ba3852698f6b993107211b36b136da8` |

No product migration, spec change, commit, archive, push, PR, installation or real-account access occurred. Later source changes require rechecking the affected assessment claims; these hashes identify the reviewed worktree only.
