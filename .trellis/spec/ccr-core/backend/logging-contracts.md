# Logging Contracts

> Executable contracts for `init_logger`, daily files, redaction, and the Desktop bridge queue.

## Scenario: Daily file name and permissions

### 1. Scope / Trigger

- Changing `crates/ccr-core/src/core/logging.rs` or `log_writer.rs`.

### 2. Contracts

- Activity file is `<log_dir>/ccr.log.YYYY-MM-DD` (UTC). The destination follows the path contract below. There is no stable `ccr.log`.
- Unix directory `0o700`. Each day's file `0o600` after create and after date change.
- Directory create failure omits the file layer. Today's chmod failure stops further file writes.

### 3. Tests Required

- `cargo test -p ccr-core log_writer -- --skip export_bindings` (default parallelism).

## Scenario: Log destination and cleanup isolation

### 1. Scope / Trigger

- Changing `get_log_dir`, logger initialization, or a process fixture that starts file logging.

### 2. Signatures

- Private `get_log_dir() -> Option<PathBuf>` selects the destination.
- Public `init_logger()` and `init_file_only_logger()` use the same destination.
- `init_logger_at(&Path)` uses only an explicitly captured log directory and
  retains the same writer, redaction, bridge, permissions and cleanup policy.
  It never resolves an ambient fallback destination.

### 3. Contracts

- Nonempty `CCR_ROOT` selects `<CCR_ROOT>/logs`. The value is an OS path.
- Empty or absent `CCR_ROOT` selects `<system home>/.ccr/logs` through `dirs::home_dir()`.
- The writer creates the selected directory and today's UTC file. `CCR_LOG_LEVEL=off` filters events; logger initialization still creates files and runs cleanup.
- Cleanup removes only managed rolling files older than 14 days in the selected directory. Retain recent files, unmanaged files, and files in other roots.
- Windows `dirs::home_dir()` uses the system Known Folder. Child `HOME` and `USERPROFILE` alone do not isolate file logging.
- Keep filter precedence, writer, redaction, bridge, and permission policy. Windows owner-only helpers remain no-op.

### 4. Validation & Error Matrix

| Condition | Result |
| --- | --- |
| Nonempty `CCR_ROOT` | Select its `logs` directory; do not initialize fallback logs |
| Empty or absent `CCR_ROOT` | Preserve the system-home fallback |
| System home unavailable | No file destination |
| Directory creation fails | Omit the file layer |
| Event filter is `off` | Initialize the selected file layer and preserve cleanup rules |

### 5. Good / Base / Bad Cases

- Good: a child process sets `CCR_ROOT` to a temporary directory before logger initialization.
- Base: an ordinary invocation without an override uses the existing system-home directory.
- Bad: a fixture sets only `HOME` or assumes `CCR_LOG_LEVEL=off` prevents file operations.

### 6. Tests Required

- `cargo test -p ccr-core --lib core::logging -- --skip export_bindings`: nonempty, empty, and absent root. Fallback assertions resolve paths without reading real logs.
- `cargo test -p ccr-core --test log_path_isolation -- --skip export_bindings`: both logger entry points, `off` filter, 15-day removal, 13-day retention, unmanaged retention, and cross-root byte/timestamp sentinels.
- The explicit logger case sets a different ambient CCR_ROOT and verifies that
  cleanup and file initialization still affect only its captured destination.
- Keep the ignored child probe separate from the tests that invoke the probe. Ignored probes alone are not acceptance evidence.

### 7. Wrong vs Correct

Wrong: start a CLI child with only a temporary `HOME` and `CCR_LOG_LEVEL=off`.

Correct: set child `CCR_ROOT` to the temporary root, then assert that initialization and cleanup affect only `<CCR_ROOT>/logs`.

## Scenario: Write-boundary redaction

### 1. Scope / Trigger

- Changing `log_redact.rs` or `testdata/log_redaction_vectors.json`.

### 2. Signatures

- `redact_log_text(&str) -> String` handles JSON and free text at the shared write boundary.
- `is_sensitive_log_key(&str) -> bool` owns sensitive key classification.

### 3. Contracts

- `mask_sensitive` only masks a whole value or a matched span. Do not pass a full sentence to it.
- Shared vectors are the alignment lock with `ccr-ui/src/utils/logRedact.ts`.
- Command output also needs sensitive `key=value` and `key:value` masking. Preserve the label and surrounding context; consume the complete quoted/escaped or delimited value, including incomplete final lines. Keep existing Cookie/Bearer/sk/JWT recognizers.
- Command snapshots and emitted deltas call this shared owner; adapters must not invent a second command-output redactor. Output clipping must back up to a UTF8 character boundary before appending an ellipsis.

### 4. Validation & Error Matrix

| Input | Required output |
| --- | --- |
| `api_key=synthetic-private-value` | Preserve key/separator; remove the complete value |
| Quoted sensitive JSON-like assignment with escaped quotes | Remove the complete secret span; keep adjacent non-secret fields |
| Ordinary sentence | Preserve its context rather than masking the whole sentence |
| Oversized four-byte Unicode process line | Bounded valid UTF8 plus visible truncation marker; no panic |

### 5. Good / Base / Bad Cases

- Good: redact each snapshot/delta line through the shared owner before retaining or emitting it.
- Base: structured JSON continues to use the existing sensitive-key traversal.
- Bad: assume arbitrary child output is JSON, or prove clipping only with ASCII.

### 6. Tests Required

- `redact_vectors_from_shared_file`
- `cargo test -p ccr-core --locked log_redact -- --skip export_bindings`: assignments, quoted/escaped values, whitespace, incomplete output and Unicode.
- ccr-app jobs snapshot redaction and oversized-multibyte-stream regressions must assert the sentinel is absent, UTF8 stays valid and truncation remains visible.

### 7. Wrong vs Correct

Wrong: forward `api_key=...` unchanged because only JSON or a known token prefix is recognized.

Correct: recognize a sensitive assignment through the existing key classifier, mask its complete value and retain safe context.

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
