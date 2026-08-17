# Logging Contracts

> Executable contracts for `init_logger`, daily files, redaction, and the Desktop bridge queue.

## Scenario: Daily file name and permissions

### 1. Scope / Trigger

- Changing `crates/ccr-core/src/core/logging.rs` or `log_writer.rs`.

### 2. Contracts

- Activity file is `~/.ccr/logs/ccr.log.YYYY-MM-DD` (UTC). There is no stable `ccr.log`.
- Unix directory `0o700`. Each day's file `0o600` after create and after date change.
- Directory create failure omits the file layer. Today's chmod failure stops further file writes.

### 3. Tests Required

- `cargo test -p ccr-core log_writer -- --test-threads=1`

## Scenario: Write-boundary redaction

### 1. Scope / Trigger

- Changing `log_redact.rs` or `testdata/log_redaction_vectors.json`.

### 2. Contracts

- `mask_sensitive` only masks a whole value or a matched span. Do not pass a full sentence to it.
- Shared vectors are the alignment lock with `ccr-ui/src/utils/logRedact.ts`.

### 3. Tests Required

- `redact_vectors_from_shared_file`

## Scenario: Bridge queue

### 1. Scope / Trigger

- Changing `log_bridge.rs`.

### 2. Contracts

- `try_enqueue_bridged_log` is sync, capacity 256, never calls `tracing::*`.
- Re-entry via `enter_bridge_consumer` returns `Reentrant`.
- Excluded targets include `ccr_desktop::monitoring`, `ccr_desktop::bridge`, `ccr_db::services::log_persistence`.
- `close_bridged_log_sender` stops later enqueues.

### 3. Tests Required

- `queue_reports_full_after_capacity`
- `reentrant_enqueue_is_dropped`
