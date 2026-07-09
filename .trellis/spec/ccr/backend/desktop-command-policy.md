# Desktop Command Policy

> Request-level validation for desktop command passthrough.

---

## Scenario: Tauri command policy for CCR CLI passthrough

### 1. Scope / Trigger
- Trigger: changing `execute_ccr_command`, `start_ccr_command_job`, `get_ccr_command_help`, or command catalog metadata.
- Applies when the desktop UI invokes `ccr` through Tauri.
- The backend owns policy; the UI mirrors catalog metadata and is not the security boundary.

### 2. Signatures
- `execute_ccr_command(command: String, args: Option<Vec<String>>, confirmation_token: Option<String>) -> Result<Value, String>`
- `start_ccr_command_job(app_handle: AppHandle, command: String, args: Option<Vec<String>>, confirmation_token: Option<String>) -> Result<Value, String>`
- `get_ccr_command_help(command: String) -> Result<Value, String>`
- `CommandInfo.requires_confirmation` marks destructive commands.
- `CommandFlagSchema.aliases` shares canonical and short flag names between backend and UI.

### 3. Contracts
- `command` must resolve to an executable catalog entry.
- `args` is tokenized input; backend validates positional arity, allowed flags, and flag values.
- `confirmation_token` is required only when `requires_confirmation = true`.
- Background jobs must use the same validator as synchronous execution.
- Flag aliases are accepted only when declared in the catalog.
- Confirmation token format: `desktop-confirm:<command>`; it is an anti-bypass token, not a compromise defense.

### 4. Validation & Error Matrix
- Unknown command -> reject with executable allowlist error.
- Unknown flag -> reject.
- Missing value for a value flag -> reject.
- Extra positional argument -> reject.
- Missing required positional argument -> reject.
- Destructive command without matching confirmation token -> reject.
- Background job path must never be looser than sync execution.

### 5. Good/Base/Bad Cases
- Good: `status --json`
- Base: `history -l 20`
- Bad: `delete old` without confirmation token
- Good: `delete old --force` with `desktop-confirm:delete`
- Bad: `history --limit` or `status --delete-everything`
- Bad: a job path that accepts input rejected by sync validation

### 6. Tests Required
- Unit tests for safe-command allowlist, unknown command, unknown flag, missing positional, missing value, destructive confirmation, flag aliases, and sync/job parity.
- Smoke test for the dangerous command flow to ensure the UI only sends the confirmation token after explicit acknowledgement.
- Existing job snapshot serialization tests must remain green.

### 7. Wrong vs Correct
#### Wrong
```rust
validate_command(command)?;
cmd.args(&args);
```

#### Correct
```rust
let request = CommandExecutionRequest::foreground(command, args, confirmation_token);
validate_command_request(&request)?;
cmd.args(&request.args);
```

---

## Scenario: CCR CLI binary resolution for desktop passthrough

### 1. Scope / Trigger
- Trigger: changing desktop passthrough process creation, desktop bundling expectations, `execute_ccr_command`, `start_ccr_command_job`, or `get_ccr_command_help`.
- Applies before any desktop path executes an external `ccr` binary.
- The resolver is a short-term hardening layer until high-frequency commands migrate to shared typed use-case services.

### 2. Signatures
- `resolve_checked_ccr_binary() -> Result<String, String>`
- `probe_ccr_binary_version(binary: &str) -> Result<Option<String>, String>`
- `parse_ccr_version_output(output: &str) -> Option<String>`
- `build_ccr_command(binary: &str) -> tokio::process::Command`

### 3. Contracts
- Candidate order is stable:
  1. current desktop executable directory plus `ccr` / `ccr.exe`;
  2. repo-root `target/debug/ccr(.exe)`;
  3. repo-root `target/release/ccr(.exe)`;
  4. PATH fallback name `ccr`.
- PATH fallback is for diagnostics and development only; bundled desktop execution should prefer the same-directory sidecar.
- Every passthrough execution path must call `resolve_checked_ccr_binary()` before spawning the user-requested command.
- The resolver must probe `ccr --version` and require the parsed CLI version to equal the desktop crate `CARGO_PKG_VERSION`.
- Version probe failures, timeouts, unparsable output, missing binaries, and version mismatches are terminal errors for that request.

### 4. Validation & Error Matrix
- Same-directory sidecar exists -> use it before dev targets and PATH.
- Dev debug binary exists and sidecar does not -> use debug binary before release and PATH.
- No file candidates exist -> try PATH name `ccr`, then report not-found if spawn fails.
- `--version` times out -> reject with a probe timeout error.
- `--version` exits non-zero -> reject with the probe exit/error details.
- Version token missing -> reject as invalid CCR CLI.
- Version differs from desktop version -> reject as version mismatch.

### 5. Good/Base/Bad Cases
- Good: bundled desktop app executes the adjacent `ccr.exe` whose version matches `ccr-desktop`.
- Base: local development uses `target/debug/ccr.exe` when no adjacent sidecar exists.
- Base: PATH fallback is attempted only after local candidates are absent.
- Bad: directly calling `tokio_command("ccr")` from a desktop command path.
- Bad: executing a PATH `ccr` whose version does not match the desktop crate.

### 6. Tests Required
- Unit test that same-directory sidecar wins.
- Unit test that repo debug binary wins before PATH fallback.
- Unit test that absent candidates fall back to PATH name only.
- Unit test that semver-like version output is parsed and invalid output is rejected.
- Focused command-exec test target must remain green after resolver changes.

### 7. Wrong vs Correct
#### Wrong
```rust
let mut cmd = tokio_command("ccr");
cmd.arg(&request.command).args(&request.args);
```

#### Correct
```rust
let binary = resolve_checked_ccr_binary().await?;
let mut cmd = build_ccr_command(&binary);
cmd.arg(&request.command).args(&request.args);
```

---

## Scenario: Command job lifecycle bounds

### 1. Scope / Trigger
- Trigger: changing `CommandJobSnapshot`, `start_ccr_command_job`, `get_ccr_command_job_status`, `cancel_ccr_command_job`, output streaming, or background maintenance cleanup.
- Applies to desktop background command jobs stored in the process-local command job registry.
- The backend owns lifecycle bounds; the UI consumes snapshot metadata and must not assume complete historical output is retained forever.

### 2. Signatures
- `start_ccr_command_job(app_handle: AppHandle, command: String, args: Option<Vec<String>>, confirmation_token: Option<String>) -> Result<Value, String>`
- `get_ccr_command_job_status(job_id: String) -> Result<Value, String>`
- `cancel_ccr_command_job(app_handle: AppHandle, job_id: String) -> Result<Value, String>`
- `prune_command_jobs() -> usize`
- `CommandJobSnapshot.truncated: bool`
- `CommandJobSnapshot.dropped_lines: usize`

### 3. Contracts
- The registry keeps at most `COMMAND_JOB_MAX_JOBS` snapshots.
- Terminal snapshots expire after `COMMAND_JOB_TTL_SECS`; queued/running snapshots are not TTL-pruned.
- Each output channel is capped by both `COMMAND_JOB_MAX_LINES_PER_CHANNEL` and `COMMAND_JOB_MAX_BYTES_PER_CHANNEL`.
- When output is clipped, `truncated` becomes `true`; when stored lines are dropped, `dropped_lines` increments.
- Capacity pruning removes oldest terminal snapshots first. If the store is full of active jobs, starting a new job returns an error instead of deleting active snapshots.
- Background maintenance calls `prune_command_jobs()` periodically; status lookup may also prune before returning a snapshot.
- Removed job snapshots must have their cancel tokens removed so lifecycle state stays aligned.

### 4. Validation & Error Matrix
- Terminal job older than TTL -> remove snapshot and cancel token.
- Registry over capacity with terminal snapshots -> remove oldest terminal snapshots first.
- Registry at capacity with only active snapshots -> reject new background job.
- Channel over line or byte cap -> drop/trim retained output and expose truncation metadata.
- Unknown pruned job id on status lookup -> return not found.

### 5. Good/Base/Bad Cases
- Good: a completed job older than TTL disappears from status lookup.
- Good: a large stdout stream keeps bounded retained lines and reports `truncated = true`.
- Base: active jobs remain visible even if their started timestamp is old.
- Bad: pruning a running job just to make room for a new one.
- Bad: dropping output without setting snapshot metadata.

### 6. Tests Required
- Unit test that output line caps are enforced per channel.
- Unit test that a single oversized line is byte-clipped without invalid UTF-8.
- Unit test that TTL pruning removes only terminal jobs.
- Unit test that capacity pruning removes oldest terminal snapshots first.
- Unit test that active snapshots are preserved when capacity pruning runs.
- Serialization test that `truncated` and `dropped_lines` are present.

### 7. Wrong vs Correct
#### Wrong
```rust
jobs.insert(job_id, snapshot);
snapshot.stdout_lines.push(line);
```

#### Correct
```rust
insert_job(snapshot, cancel_token).await?;
job.push_line(OutputChannel::Stdout, line);
```
