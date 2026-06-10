# Codex Session Visibility Repair

## Goal

Extend CCR's existing Codex history visibility repair so it matches the practical behavior seen in `cockpit-tools`: when Codex account/API provider changes hide existing conversations, `ccr codex sync-history` should repair rollout provider metadata, SQLite thread metadata, and `session_index.jsonl` consistency as one safe, backup-backed operation.

## What I Already Know

- The user wants both red-box features from `cockpit-tools` implemented in CCR crates.
- This task covers only "修复可见性" / "Repair Visibility".
- The reference implementation repairs rollout files, `state_5.sqlite`, and `session_index.jsonl`.
- CCR already has `CodexHistorySyncService` and `ccr codex sync-history`, so the implementation should enhance that service instead of creating a duplicate repair path.
- CCR's current `sync-history` backup format does not include `session_index.jsonl`.

## Assumptions

- The user wants crate/CLI behavior first. Tauri UI wiring can be handled later if needed.
- The operation targets one Codex home by default, using `~/.codex` or `CCR_CODEX_DIR`, not full Cockpit-style multi-instance management.
- `session_index.jsonl` repair should be part of `sync-history`, not a separate hidden background action.

## Requirements

- Add `session_index.jsonl` reconciliation to `CodexHistorySyncService`.
- Reconciliation must append missing session index entries for SQLite thread rows that are missing from the index.
- Use stable entries equivalent to the reference shape: `id`, `thread_name`, and `updated_at`.
- Extend status/dry-run/write result types to report:
  - missing session index entries
  - added session index entries
  - whether session index existed before repair
- Extend managed backups so `restore` and failure rollback can restore `session_index.jsonl` to its previous state or remove it if it did not exist before.
- Preserve existing rollout and SQLite behavior, including provider bridge behavior and `--all-history`.
- Preserve `dry_run`: it may report session index entries to add, but must not create backups or write files.
- Preserve lock/rollback behavior: if a write fails after partial mutation, restore any changed rollout/global-state/session-index state covered by backup.
- Update CLI output for `status`, dry-run, write, and restore summaries.
- Add focused tests for session index diagnostics, dry-run, write, restore, and rollback-safe backup behavior.

## Acceptance Criteria

- [ ] `ccr codex sync-history status` reports session index consistency diagnostics.
- [ ] `ccr codex sync-history --dry-run` reports missing session index entries without modifying `session_index.jsonl`.
- [ ] `ccr codex sync-history` appends missing `session_index.jsonl` rows when SQLite has matching thread rows.
- [ ] Existing rollout provider and SQLite thread repair tests still pass.
- [ ] A sync-history backup captures enough session index state for `restore --restore-state` or standard restore semantics defined by implementation to restore previous index content.
- [ ] If `session_index.jsonl` did not exist before repair, restore removes the created file.
- [ ] CLI parse tests cover any new alias/flag/output path if added.
- [ ] Targeted `ccr-codex` tests pass.

## Definition of Done

- Tests added/updated in `ccr-codex` and, if CLI shape changes, `ccr-cli`.
- `cargo fmt` clean for touched Rust files.
- Targeted tests pass.
- No production `unwrap` / `expect` added.
- No unrelated refactors.

## Technical Approach

### Recommended Approach: Enhance `sync-history`

Use `CodexHistorySyncService` as the single repair engine:

1. Add constants and helpers for `SESSION_INDEX_FILE = "session_index.jsonl"`.
2. Add helper functions:
   - read session index map by `id`
   - build session index entry from a thread/rollout candidate
   - count missing index entries
   - append missing entries atomically
   - restore previous session index content from backup
3. Extend `CodexHistorySyncStatus`, `CodexHistorySyncResult`, and backup metadata with session index fields.
4. During `status`, compute missing index count without writing.
5. During `sync`, include session index changes in the write plan and backup only when changes are needed.
6. During restore, restore session index state alongside rollout metadata and optional SQLite/global-state.
7. Update `crates/ccr-cli/src/commands/codex/sync_history.rs` output.

### Rejected Alternative

Create a separate `repair-visibility` service/command. This duplicates `sync-history` provider resolution, backup, locking, and SQLite behavior, increasing risk without adding capability.

## Decision (ADR-lite)

**Context**: CCR already has `sync-history` for the same user-visible problem, but it currently lacks `session_index.jsonl` parity with `cockpit-tools`.

**Decision**: Extend `sync-history` to include session index reconciliation and reporting.

**Consequences**: The public command remains stable, implementation is smaller, and existing tests/backup flows remain useful. The trade-off is that the command name remains less UI-like than "Repair Visibility"; a CLI alias can be added later if needed.

## Implementation Plan

### PR1: Session Index Diagnostics and Helpers

- Add session index read/count/build helpers in `codex_history_sync_service.rs`.
- Extend status/result structs with index counts.
- Add unit tests for:
  - missing index count
  - valid existing entry skipped
  - invalid index JSON error behavior

### PR2: Backup-backed Write Path

- Extend backup metadata and restore helpers to capture previous `session_index.jsonl`.
- Add write path for appending missing entries.
- Ensure dry-run does not write.
- Add tests for write, restore, and missing-original-file restore.

### PR3: CLI Rendering and Regression Sweep

- Update `sync_history.rs` output labels.
- Add CLI tests only if argument shape changes.
- Run targeted tests and formatting.

## Out of Scope

- Full Cockpit-style cross-instance repair.
- Tauri UI button wiring.
- Decrypting or re-encrypting Codex encrypted conversation content.
- Changing Codex official app internals.
- Recovering corrupted SQLite databases.

## Research References

- `research/codebase-notes.md` - Current CCR and `cockpit-tools` behavior comparison.

## Technical Notes

- Reference module: `ref/repo/cockpit-tools/src-tauri/src/modules/codex_session_visibility.rs`.
- Existing CCR service: `crates/ccr-codex/src/services/codex_history_sync_service.rs`.
- Existing CLI: `crates/ccr-cli/src/commands/codex/sync_history.rs`.
- Existing command definition: `crates/ccr-cli/src/cli/subcommands/codex.rs`.
- Test fixture guidance: `.trellis/spec/ccr-codex/backend/test-fixtures.md`.
