# Monitoring Log Contracts

> Desktop bridge, persist deadline, and frontend IPC limits.

## Scenario: Runtime channel

### 1. Scope / Trigger

- Changing `ccr-ui/src-tauri/src/bridge.rs` or `monitoring.rs` persist rules.

### 2. Contracts

- Bridged `ccr_*` warn/error become `channel=runtime`.
- `runtime.error` persists. `runtime.warn` is live-only.
- `frontend.warn` / `frontend.error` still persist.
- `record_monitoring_entry` does not `force_flush` per event.
- Emit/flush failures increment an internal counter. They must not emit bridgeable `ccr_*` tracing.

### 3. Tests Required

- `runtime_entry_maps_error_and_warn`
- `should_persist_warn_and_whitelisted_events`
- `reentrant_consumer_does_not_enqueue`

## Scenario: Flush deadline

- Threshold 20 or 2s ticker, whichever first.
- Exit: `close_bridged_log_sender`, then `force_flush` with 500ms timeout, then `database::shutdown`.

## Scenario: Frontend IPC

- `sanitize_frontend_log` is the server boundary. Batch 32. See parent design IPC table.
- Tests: `ccr-ui/src-tauri/src/log_sanitize.rs`, `ccr-ui/tests/shell/logger.smoke.test.ts`.
