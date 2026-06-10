# Codex Session Trash Restore - Codebase Notes

## Reference Behavior

- `ref/repo/cockpit-tools/src/components/codex/CodexSessionManager.tsx` exposes:
  - move selected sessions to trash
  - open restore dialog
  - restore selected trashed sessions
- `ref/repo/cockpit-tools/src-tauri/src/modules/codex_session_manager.rs` implements:
  - `move_sessions_to_trash_across_instances`
  - `list_trashed_sessions_across_instances`
  - `restore_sessions_from_trash_across_instances`
- The reference trash manifest records enough information to restore a session to its original instance:
  - session id, title, cwd
  - instance id/name/root
  - original rollout path and relative path
  - original `session_index.jsonl` entry
  - deleted timestamp
- Restore checks for target rollout path and `session_index.jsonl` conflicts, copies the rollout back, restores file modified time, appends the saved session index entry, asks official Codex to rebuild metadata, and rolls back on failure.

## Current CCR State

- `crates/ccr-codex/src/services/codex_session_service.rs` lists/details/exports Codex session JSONL files under `~/.codex/sessions`.
- `crates/ccr-codex/src/services/codex_history_sync_service.rs` already contains reusable parsing concepts for rollout metadata, file times, and SQLite thread reconstruction.
- There is no crate-level Codex session trash service yet.
- There is no current CCR CLI command for moving Codex sessions to a recoverable trash or listing/restoring trashed Codex sessions.

## Gaps

- Need a stable CCR-owned trash root and manifest format.
- Need conflict-safe restore behavior.
- Need session index helpers reusable with visibility repair work.
- Need CLI entrypoints and summaries for list/trash/restore operations.
- Need tests that prove the restored session can be made visible through `session_index.jsonl` and follow-up `sync-history`/metadata repair.

## Implementation Constraints

- Implement the service in `ccr-codex`; expose CLI plumbing in `ccr-cli`.
- Keep destructive operations explicit; no hidden deletion without a named command.
- Preserve user files: move/copy with rollback where practical; never overwrite an existing rollout or existing session index entry during restore.
- Prefer restoring rollout + session index first, then rely on `sync-history`/visibility repair for SQLite consistency rather than inventing a separate SQLite writer.
- Use atomic writes for `session_index.jsonl` updates.
- Do not require the Tauri UI for the crate implementation.

## Verification Anchors

- Targeted service tests in `ccr-codex` for trash, list, restore, conflict handling, and rollback.
- CLI parse tests in `ccr-cli`.
- Targeted commands: `cargo test -p ccr-codex codex_session_trash`, `cargo test -p ccr-cli codex_session`.
