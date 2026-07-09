# Codex Session Trash Restore

## Goal

Add a CCR crate/CLI implementation of Codex session trash and restore behavior similar to `cockpit-tools`, so selected Codex conversations can be moved out of the active session list and later restored safely with rollout files, session index entries, and file timestamps intact.

## What I Already Know

- The user wants the second red-box feature, "恢复对话" / "Restore Sessions", implemented in CCR crates.
- `cockpit-tools` treats restore as the counterpart to move-to-trash: sessions are moved to a managed trash with manifest metadata, then restored back to the original Codex instance.
- CCR currently has session listing/export/details in `CodexSessionService`, but no recoverable trash service.
- CCR already has visibility repair in `CodexHistorySyncService`; restored sessions can use that path for SQLite consistency rather than duplicating SQLite writes in the trash service.

## Assumptions

- The MVP is a CLI/crate implementation for the active Codex home, not full Tauri UI.
- Restore should be conflict-safe and should not overwrite an existing rollout file or duplicate session index entry.
- The trash format should be CCR-owned and stable enough for future UI support.

## Requirements

- Add a `CodexSessionTrashService` under `ccr-codex`.
- Support moving selected session IDs to a managed trash root.
- Support listing trashed sessions with session id, title/preview, cwd, deleted time, original relative path, and original Codex home.
- Support restoring selected session IDs from trash.
- Trash manifest must record:
  - session id
  - title or preview
  - cwd
  - deleted timestamp
  - original rollout path and relative path
  - original Codex home
  - saved `session_index.jsonl` entry when available
  - rollout file modified time
- Move-to-trash must:
  - find matching rollout files under `sessions` and `archived_sessions`
  - copy or move rollout into trash
  - remove the rollout from active location only after trash write succeeds
  - remove matching `session_index.jsonl` entry when present
  - avoid deleting unrelated session files
- Restore must:
  - reject restore if the target rollout path already exists
  - reject restore if target `session_index.jsonl` already has the same session id
  - copy rollout back
  - restore modified time
  - append saved or reconstructed `session_index.jsonl` entry
  - remove the trash entry only after restore succeeds
- Provide CLI commands for trash/list/restore.
- Add tests for happy path, conflict path, missing index file, multiple sessions, and rollback on index write failure where practical.

## Acceptance Criteria

- [ ] `ccr-codex` exposes a service that can move a fixture Codex rollout to trash and list it.
- [ ] Restoring the trashed session recreates the rollout file at the original path.
- [ ] Restoring appends the expected session id to `session_index.jsonl`.
- [ ] Restoring preserves or restores the rollout modified time.
- [ ] Restore refuses to overwrite an existing rollout file.
- [ ] Restore refuses to duplicate an existing session index entry.
- [ ] Trash cleanup happens only after successful restore.
- [ ] CLI commands are parsed and invoke the service.
- [ ] Targeted tests pass.

## Definition of Done

- New service and public types are exported from `ccr-codex::services`.
- CLI handlers are added in `ccr-cli`.
- Tests cover service behavior and CLI parsing.
- Formatting and targeted tests pass.
- The trash format is documented in code comments or tests clearly enough for future UI work.

## Technical Approach

### Recommended Approach: New Trash Service, Reuse History Repair Helpers

1. Create `crates/ccr-codex/src/services/codex_session_trash_service.rs`.
2. Keep trash behavior separate from `CodexSessionService` because it mutates filesystem state.
3. Add public types:
   - `CodexTrashedSessionRecord`
   - `CodexSessionTrashSummary`
   - `CodexSessionRestoreSummary`
   - optional manifest type if public exposure is useful
4. Implement helper functions:
   - collect rollout files from `sessions` and `archived_sessions`
   - parse first-line session metadata
   - read/remove/append session index entries
   - write/read trash manifest
   - copy/move with rollback
5. Reuse or duplicate narrowly-scoped parsing helpers from `codex_history_sync_service` only if extraction would not cause a large refactor. Prefer a small shared private helper module only if both tasks need the same session index operations.
6. Add CLI surface under `ccr codex sessions` or `ccr codex session-trash`; recommended CLI shape:
   - `ccr codex sessions trash <session-id>...`
   - `ccr codex sessions trash-list`
   - `ccr codex sessions restore <session-id>...`
   - `--codex-home <path>` for fixture/manual targeting

### Rejected Alternative

Implement restore directly inside `CodexSessionService`. That service currently reads and summarizes sessions; adding destructive move/restore behavior would blur read-only inventory behavior with state mutation.

## Decision (ADR-lite)

**Context**: Trash/restore is stateful and riskier than listing sessions. It needs a manifest and rollback boundaries.

**Decision**: Add a dedicated `CodexSessionTrashService` and keep CLI as a thin wrapper.

**Consequences**: This keeps mutation logic isolated and future UI can call one service. It adds a new service file, but avoids overloading existing session listing code.

## Implementation Plan

### PR1: Trash Format and Service Skeleton

- Add service file and public summary/record types.
- Define trash root, manifest filename, and manifest schema.
- Implement list of trash entries and empty-state behavior.
- Export service types from `ccr-codex::services`.
- Add tests for manifest read/write and empty list.

### PR2: Move to Trash

- Implement active session discovery by session id.
- Save manifest and rollout copy into trash.
- Remove active rollout and session index entry after trash write succeeds.
- Add tests for single session, multiple sessions, missing session, and index removal.

### PR3: Restore From Trash

- Implement conflict checks.
- Restore rollout, mtime, and session index entry.
- Remove trash entry after success.
- Add tests for happy path, rollout conflict, index conflict, missing index file, and cleanup.

### PR4: CLI Wiring and Verification

- Add CLI subcommands and handlers.
- Add parse tests.
- Add concise user-facing output.
- Run targeted tests and formatting.

## Out of Scope

- Cockpit-style cross-instance session sync/copy.
- Tauri UI restore dialog.
- Direct official Codex app metadata rebuild API integration.
- Direct SQLite mutation during trash restore; follow-up `sync-history` handles visibility/SQLite consistency.
- Permanent deletion/purge command unless explicitly requested later.

## Research References

- `research/codebase-notes.md` - Current CCR and `cockpit-tools` behavior comparison.

## Technical Notes

- Reference module: `ref/repo/cockpit-tools/src-tauri/src/modules/codex_session_manager.rs`.
- Existing CCR session read service: `crates/ccr-codex/src/services/codex_session_service.rs`.
- Existing repair service for follow-up visibility: `crates/ccr-codex/src/services/codex_history_sync_service.rs`.
- Existing command definition area: `crates/ccr-cli/src/cli/subcommands/codex.rs`.
- Test fixture guidance: `.trellis/spec/ccr-codex/backend/test-fixtures.md`.
