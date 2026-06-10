# Codex Session Visibility Repair - Codebase Notes

## Reference Behavior

- `ref/repo/cockpit-tools/src/components/codex/CodexSessionManager.tsx` exposes the "Repair Visibility" action in the session manager toolbar.
- `ref/repo/cockpit-tools/src-tauri/src/modules/codex_session_visibility.rs` repairs visibility across Codex instances by aligning:
  - rollout `session_meta.payload.model_provider`
  - `state_5.sqlite` `threads` metadata
  - missing `session_index.jsonl` entries
- The reference flow backs up modified files before writing and summarizes changed rollout files, SQLite rows, session index entries, invalid SQLite skips, and running instances.
- Reference behavior treats missing root `model_provider` in `config.toml` as the default official provider `openai`.

## Current CCR State

- `crates/ccr-codex/src/services/codex_history_sync_service.rs` already implements most visibility repair under `CodexHistorySyncService`.
- Existing `sync-history` behavior can:
  - resolve target provider from Codex `config.toml`
  - rewrite rollout first-line provider metadata
  - update and insert `state_5.sqlite` `threads` rows
  - repair SQLite `preview`, `has_user_event`, and `cwd`
  - maintain managed backups for rollout/global-state/SQLite changes
  - expose status diagnostics through `CodexHistoryVisibilityDiagnostics`
- Existing CLI entrypoint is `ccr codex sync-history` in `crates/ccr-cli/src/cli/subcommands/codex.rs` and `crates/ccr-cli/src/commands/codex/sync_history.rs`.

## Gaps

- `session_index.jsonl` is not currently reconciled or reported in `CodexHistorySyncResult` / `CodexHistorySyncStatus`.
- Managed backup metadata does not record whether `session_index.jsonl` existed before repair, so rollback/restore must be extended before writing session index changes.
- CLI output does not mention session index repair counts.
- Existing tests cover rollout and SQLite visibility repair, but not session index reconciliation.

## Implementation Constraints

- Keep the core logic in `ccr-codex`; `ccr-cli` should only parse flags and render results.
- Use existing `LockManager` path and `LOCK_RESOURCE = "codex_sync_history"` to serialize writes.
- Preserve `dry_run` behavior: no backup or state writes during preview.
- Preserve rollback semantics when a write fails after partial mutation.
- Use `TestCodexEnv` for env-dependent `ccr-codex` tests.
- Do not add network dependencies or a new runtime.

## Verification Anchors

- Targeted tests: `cargo test -p ccr-codex codex_history_sync_service`
- CLI parsing/output tests: `cargo test -p ccr-cli codex_sync_history`
- Broader Rust check after implementation: `cargo test -p ccr-codex`, then `just fmt-check` and the relevant clippy command.
