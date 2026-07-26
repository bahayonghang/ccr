# Managed Process Tree

> Cross-platform ownership and cleanup for child process trees.

## Scenario: managed child lifecycle

### 1. Scope / Trigger

- Trigger: spawning a child whose descendants must be cancelled, timed out, or reaped as one owned tree.
- Applies to `ccr_core::core::process_gateway::ManagedProcess` and desktop `ProcessGateway` callers.

### 2. Signatures

- `ManagedProcess::spawn(tokio::process::Command) -> io::Result<ManagedProcess>`
- `ManagedProcess::wait(&mut self) -> io::Result<ExitStatus>`
- `ManagedProcess::terminate_tree(&mut self, grace: Duration) -> io::Result<ExitStatus>`
- `ManagedProcess::{take_stdin,take_stdout,take_stderr}` transfer pipe ownership.
- `read_bounded_line(&mut AsyncBufRead, max_bytes) -> io::Result<Option<BoundedLine>>` caps one physical line before allocation grows past `max_bytes`.

### 3. Contracts

- Unix children start in a new process group; cancellation signals the group, waits for `grace`, then escalates to `SIGKILL` and reaps the direct child.
- Windows children are attached to a Job Object with `KILL_ON_JOB_CLOSE`; cancellation terminates the job and waits for the direct child.
- `wait` and successful `terminate_tree` set the reaped state. Dropping an unreaped process force-terminates the owned tree as a last resort, but callers must still await a terminal method.
- No production caller may mark a job terminal before `wait` or `terminate_tree` returns.
- Child stdout/stderr that is not consumed by the foreground capped reader must use `read_bounded_line`; bounded queues do not bound `AsyncBufReadExt::lines()` before a newline arrives.

### 4. Validation & Error Matrix

- Spawn fails -> return the OS error; no registry entry is created.
- Job/process-group attachment fails -> return the OS error and do not expose an unmanaged child.
- Graceful termination exceeds `grace` -> force-terminate the tree, then wait/reap.
- Tree termination or wait fails -> caller reports cleanup failure; it must not report cancellation success.
- Unterminated line exceeds `max_bytes` -> consume through newline/EOF with constant retained memory and return `BoundedLine.truncated = true`.

### 5. Good/Base/Bad Cases

- Good: a parent that spawns a grandchild leaves no live descendant after `terminate_tree`.
- Base: a normally exiting child is consumed with `wait`.
- Bad: call `Child::kill` and immediately mark the job cancelled.
- Bad: drop a live `ManagedProcess` as the normal cancellation path.
- Bad: feed `BufReader::lines()` directly from an untrusted child into an otherwise bounded channel.

### 6. Tests Required

- Windows fixture: parent starts a grandchild, `terminate_tree` completes, and the grandchild PID is no longer running.
- Unix CI fixture: the same assertion targets a dedicated process group.
- Bounded-line fixture: an unterminated input larger than the cap returns only the capped prefix with `truncated = true`.
- Run `cargo test -p ccr-core process_gateway -- --test-threads=1`.
- Run `cargo clippy -p ccr-core --all-targets --all-features -- -D warnings`.

### 7. Wrong vs Correct

#### Wrong

```rust
child.kill().await?;
job.status = Cancelled;
```

#### Correct

```rust
let mut child = ManagedProcess::spawn(command)?;
child.terminate_tree(Duration::from_secs(5)).await?;
job.status = Cancelled;
```
