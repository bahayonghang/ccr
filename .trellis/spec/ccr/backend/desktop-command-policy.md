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
- `ProcessDescriptor::ccr_command() -> ProcessDescriptor`
- `ProcessDescriptor::ccr_version_probe() -> ProcessDescriptor`
- `ProcessGateway::command(&ProcessDescriptor) -> Result<tokio::process::Command, String>`
- `ProcessGateway::execute(&ProcessDescriptor, &[OsString]) -> Result<CappedProcessOutput, String>`

### 3. Contracts
- Release builds resolve only the sidecar adjacent to the desktop executable, require compile-time `CCR_SIDECAR_SHA256`, and verify the file before execution.
- Debug builds may resolve adjacent, repo `target/debug`, or repo `target/release` candidates. PATH fallback is forbidden in every build.
- Every passthrough and help/version path uses a closed descriptor and the gateway's timeout/output bounds.
- `ccr --version` remains a compatibility check; it is not the identity proof.
- Version probe failures, timeouts, truncation, unparsable output, missing binaries, hash failures, and version mismatches are terminal errors for that request.

### 4. Validation & Error Matrix
- Valid adjacent release sidecar + matching hash -> execute it.
- Debug build without an adjacent sidecar -> try repo debug, then repo release.
- No file candidates exist -> report `ccr_sidecar_not_found`; never try PATH.
- Release hash missing/invalid/mismatched -> fail closed before spawn.
- `--version` times out -> reject with a probe timeout error.
- `--version` exits non-zero -> reject with the probe exit/error details.
- Version token missing -> reject as invalid CCR CLI.
- Version differs from desktop version -> reject as version mismatch.

### 5. Good/Base/Bad Cases
- Good: bundled desktop app executes the adjacent `ccr.exe` whose version matches `ccr-desktop`.
- Base: local debug development uses `target/debug/ccr.exe` when no adjacent sidecar exists.
- Bad: directly call `tokio_command("ccr")` from a desktop command path.
- Bad: restore a PATH fallback, even with a version probe.
- Bad: treat self-reported `--version` output as binary identity.

### 6. Tests Required
- Unit test that release resolution requires a hash and never falls back to PATH.
- Unit test that a hash mismatch is rejected.
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
let descriptor = ProcessDescriptor::ccr_command();
let mut cmd = ProcessGateway::command(&descriptor)?;
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

---

## Scenario: ProcessGateway capability, output, and cleanup boundary

### 1. Scope / Trigger

- Trigger: changing desktop foreground/background process execution, CLI probes, install execution, OAuth port discovery/opening, update, llmusage, PATH lookup, or SSH/SFTP helpers.
- Applies to `ccr-ui/src-tauri/src/process/gateway.rs`, `ccr-core::ManagedProcess`, and every migrated caller.

### 2. Signatures

- `ProcessDescriptor` binds a closed `ProcessCapability`, `TrustedExecutable`, timeout, and per-stream byte limit.
- `ProcessGateway::{command,execute,execute_command,spawn}` are the managed execution entry points.
- `read_bounded_line(&mut AsyncBufRead, max_bytes) -> io::Result<Option<BoundedLine>>` caps allocation before a newline is found.
- Background progress is `CommandJobDelta { job_id, seq, channel, lines, dropped_count, status }`.
- Terminal status includes `cleanup_failed` when tree cleanup/reap cannot be proven.

### 3. Contracts

- Foreground output is read concurrently, defaults to 60 seconds and at most 1 MiB per stream, and terminates the owned tree on timeout or overflow.
- Background readers use a bounded channel, at most 50 retained lines per 100 ms tick, independent dropped counters per stream, and delta-only progress events. Full snapshots are query/terminal payloads.
- `AsyncBufReadExt::lines()` is forbidden for untrusted child output: a channel cap does not bound one unterminated line. Use `read_bounded_line` before parsing or batching.
- OAuth URL opening accepts only the exact OpenAI authorize endpoint and fixed loopback callback origin. Port release requests cancellation only for matching registry records; unknown PIDs are report-only.
- WSL's synchronous file adapter and SkillPort's intentional detached GUI handoff are explicit legacy adapters. Do not add callers to them; migrate them under a separately reviewed lifecycle contract rather than pretending a detached app is a reapable child.

### 4. Validation & Error Matrix

- Unknown capability/tool -> reject before spawn.
- Foreground timeout -> terminate tree, reap, return `timed_out = true`.
- Stream byte limit -> terminate tree and return per-stream truncation metadata.
- Unterminated background line exceeds its cap -> retain bounded text, increment dropped/truncation metadata, keep memory bounded.
- llmusage NDJSON line exceeds 1 MiB -> fail with `llmusage_stdout_line_too_long` and terminate/reap the process.
- Cancellation cleanup fails -> terminal `cleanup_failed`, never `cancelled` success.
- OAuth port belongs to an unregistered PID -> return it in `unknown_pids`; never send a kill signal.

### 5. Good/Base/Bad Cases

- Good: a flood producer is reduced to 100 ms deltas and observable dropped counts.
- Good: stdout and stderr account for drops independently.
- Base: a short foreground probe exits normally with complete capped output.
- Bad: `BufReader::lines()` on a child pipe, even behind bounded mpsc.
- Bad: generic `Command::new(renderer_value)`, PATH fallback for CCR, `taskkill /F`, or `kill -9` on a discovered PID.

### 6. Tests Required

- Gateway: timeout, stdout/stderr flood, unterminated bounded line, URL allowlist, sidecar hash, and PATH-precedence rejection.
- Background jobs: stalled consumer, per-stream storage caps, delta sequence, dropped count, and `cleanup_failed` serialization.
- Process tree: descendant termination/reap on Windows, Linux, and macOS CI.
- OAuth: unsafe schemes/host confusion and unknown PID report-only behavior.
- Run focused `command_exec`, `codex_auth`, SSH, install, llmusage, frontend type-check/smoke, then `just lint-strict` and `just test`.

### 7. Wrong vs Correct

#### Wrong

```rust
let mut lines = BufReader::new(child_stdout).lines();
while let Some(line) = lines.next_line().await? {
    tx.send(line).await?;
}
```

#### Correct

```rust
let mut reader = BufReader::new(child_stdout);
while let Some(line) = read_bounded_line(&mut reader, max_bytes).await? {
    bounded_delta.try_send(line)?;
}
```
