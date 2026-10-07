# P4 Independent Final Check

Status: **P4_SCOPED_PASS / WORKSPACE_PENDING**.

- Task: `.trellis/tasks/10-06-auth-defense-in-depth`.
- Reviewer: `trellis-check`, `/root/check_p4`.
- Source freeze: 2026-10-06 20:47:41 America/Chicago; 2026-10-07 01:47:41 UTC.
- Core review: reused `core-independent-check.md` after checking all 15 core manifest entries; 0 hash mismatches.
- Current complete Codex/TUI gates cover the final registry/account `extra` Debug change. The earlier 367-test Codex receipt remains historical.
- Product edits by this reviewer: none. Other owners' P1/P2/P3 changes were preserved.
- Inputs: native hook context, check.jsonl, PRD/design/implement, listed specs, design research, core review and implementation receipts, current source, `crates/AGENTS.md`, and the Trellis check skill.

## Contract Review

| Contract | Result and evidence |
| --- | --- |
| R1: SID DACL; no USERNAME or external production process | PASS. The separate approved API remains at `guarded_write.rs:197`; core source hashes match PART 1. The native child removes USERNAME and checks the process-token SID policy. `utils.rs:86` calls the API with raw bytes and propagates errors. Production consumers handle the Result. |
| Preserve existing writer APIs | PASS. Core PART 1 confirms that the old preserve-DACL helper remains unchanged. The extracted SID constructor retains the existing new-file policy. No frozen CcrError variant or dependency changed. |
| R2: planner remains read-only | PASS. `plan_runtime_sync` has no permission write. Both execution callers use `harden_unchanged_auth_locked`; the pair helper at `codex_oauth_token_service.rs:567` rechecks complete identity, selected account and tokens. |
| R2: original bytes, mtime, file identity and version guard | PASS. Consumer native tests preserve both files' bytes/mtime; core native tests preserve file identity. The helper uses content tokens from original bytes. Stale runtime/snapshot regression passes. Quota-only preparation also hardens matching files and rejects another identity. |
| Backup before CAS publication | PASS. `codex_runtime_service.rs:348` reads/version-checks, creates the runtime backup, hardens the expected version, then calls versioned secret write. `synced_auth_cas_conflict_after_backup_does_not_restore_old_runtime` passed in the final package. A conflict preserves the external new runtime. |
| R3: private rename fallback | PASS. `codex_auth_service.rs:2195` reads source bytes after injected rename failure, observes/hardens any existing target, publishes through secret versioned write and removes the source only after success. Native tests cover new/existing targets and failure before replacement. The snapshot mover contains no copy-then-harden branch. P3 backups remain before destructive movement. |
| R4: sensitive Debug and disk behavior | PASS. Auth JSON/tokens, profile secret, sync raw maps, runtime raw auth/config and HTTP token request/response are redacted. `CodexAuthAccount` and `CodexAuthRegistry` also hide raw TOML `extra`. Synthetic marker assertions and unchanged plaintext disk round trips passed. Export DTO protection follows CodexAuthJson. |
| R5: fixed diagnostics | PASS. `openai_quota_core.rs:475` and `:513` return status plus an allowed fixed code. `extract_error_code` at `:634` accepts five codes and maps the known invalidation phrase internally. Arbitrary code/message/body, Unicode/long bodies and invalid success values do not enter errors. |
| R5: refresh, repair and relogin classification | PASS. Final suite passed manual cache bypass, expired/rejected-access refresh, known permanent-code repair, relogin marker, and 403 phrase refresh/retry regressions. |
| R5: composed bilingual matrix | PASS. `tui/ui.rs:2634` exercises main `draw` in EN/ZH at 80x24, 100x22, 100x30, 120x22, 140x40, 180x50 and 60x18. Assertions inspect localized buffer text, error color, marker removal and compact omission. The existing full relogin/action-hint/code test also passed. |
| Spec sync | PASS. Core owner-only and Codex permission/diagnostic contracts match the implementation. TUI specs separate TestBackend from native evidence. Main added the explicit raw TOML extension Debug contract before report completion; no product source changed. |

## Findings (fixed)

No new product issue was found after the final lint/type checks passed. This reviewer changed no product source.

- File: `research/implementation-validation.md`.
- Issue: the report still marked Clippy RUNNING and format/policy/whitespace checks NOT_RUN; the current-source Codex count omitted the last Debug test.
- Fix: recorded current results, the 368 passed / 2 ignored package receipt, preserved first failures, and the source freeze.

- File: `prd.md`.
- Issue: evidence-backed local criteria still had open checkboxes.
- Fix: checked R1-R5 behavior criteria. The combined workspace criterion remains open because parent workspace gates are pending.

The implementer fixed the earlier synthetic extension Debug failure, five test lint failures, the backup/CAS ordering failure, the Windows loopback blocking fixture and two matrix assertion/build failures. Their first-failure logs remain unchanged. These fixes are owner fixes, not reviewer product edits.

## Findings (not fixed)

No unresolved P4 product finding blocks the scoped handoff.

- The original core native test startup failure `0xc0000005 / STATUS_ACCESS_VIOLATION` has no running-tests output. Later default and no-incremental receipts passed. The cause remains unknown. PART 1 preserves source/EXE identities and the failure receipts.
- Windows case-only account alias behavior remains a pending product decision from the existing task tree. P4 did not change that behavior.
- Two-file permission hardening has no ACL transaction. Runtime permissions can tighten before a later snapshot conflict/error. Native rename success followed by permission failure can already have moved the snapshot. These effects match the approved design and are documented in the implementation report.
- External Codex/editors do not join CCR locks. Verified file handles and content guards do not establish a transaction across external writers.
- Unix native permission and Linux/macOS execution, interactive TUI, installed binary, real account/OAuth and hosted evidence remain NOT_RUN. The task uses synthetic temporary data and loopback HTTP only.
- Workspace `just lint-strict`, `just test`, and `just ci` remain PARENT_PENDING after P6. Scoped package results do not complete that acceptance clause.

## Verification

| Command | Result | Receipt |
| --- | --- | --- |
| `cargo test -p ccr-codex --all-features -- --skip export_bindings` | PASS, exit 0; 368 passed / 0 failed / 2 existing ignored; docs 0. Includes all 16 `p4_` tests. | `independent-codex-validation.log` |
| `cargo test -p ccr-tui --all-features -- --skip export_bindings` | PASS, exit 0; 253 passed / 0 failed / 0 ignored; docs 0. | `independent-tui-validation.log` |
| `cargo clippy -p ccr-codex -p ccr-tui --all-targets --all-features -- -D warnings -D clippy::unwrap_used` | Lint PASS / TypeCheck PASS, exit 0. | `independent-clippy-validation.log` |
| `just fmt-check` | PASS, exit 0; 5 formatter tests, 11 JSON files, workspace/Tauri Rust format. | `independent-fmt-validation.log` |
| `python scripts/quality/check_secret_writes.py` | PASS, exit 0; sensitive persistence policy. | `independent-secret-writes-validation.log` |
| Scoped `git diff --check --` | PASS, exit 0; only LF/CRLF advisory warnings. | `independent-diff-validation.log` |
| Core final source and preserved evidence comparison | PASS; 15 entries / 0 mismatches. No repeated core execution. | `core-implementation-hashes.json`, PART 1 report |

Both final package commands used default parallelism. No coverage threshold, test skip, public DTO, error variant or runtime endpoint was added. Existing two Codex benchmarks and six core doctests remain ignored.

The sensitive-path sweep found no production USERNAME/icacls or body_preview usage in the approved files. All affected production permission calls propagate or explicitly handle failures. The remaining model-provider `fs::copy` is the existing provider metadata backup outside the snapshot mover; P4 did not change that backup.

## Identity And Ownership

`independent-hashes.json` records 13 P4 source files, current relevant specs/task contracts and all P4 raw logs/reports. The manifest excludes itself. Every recorded source must match before reusing these receipts. A later product change requires fresh evidence for the affected gate.

Reviewer-owned edits are limited to `prd.md` acceptance evidence, `research/implementation-validation.md`, this report, `research/independent-hashes.json`, and six `research/independent-*-validation.log` receipts. No source, task pointer, task.json, execution-status, P5/P6, commit, archive, installation or external write was performed.

Source rollback must remove the P4 consumer calls together with the new core API, while retaining P1/P2/P3 hunks. Metadata hardening has no content rollback entry. Data recovery uses matching P3 private backups after complete identity validation; file restoration does not establish server-side token validity. No rollback was executed.
