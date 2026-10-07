# Managed Process Tree

> Cross-platform ownership and cleanup for child process trees.

## Scenario: managed child lifecycle

### 1. Scope / Trigger

- Trigger: spawning a child whose descendants must be cancelled, timed out, or reaped as one owned tree.
- Applies to `ccr_core::core::process_gateway::ManagedProcess` and desktop `ProcessGateway` callers.

### 2. Signatures

- `ManagedProcess::spawn(tokio::process::Command) -> io::Result<ManagedProcess>`
- `ManagedProcess::spawn_detached(tokio::process::Command) -> io::Result<ManagedProcess>`: opt-in mode for a command whose spawned product must outlive the command.
- `ManagedProcess::wait(&mut self) -> io::Result<ExitStatus>`
- `ManagedProcess::terminate_tree(&mut self, grace: Duration) -> io::Result<ExitStatus>`
- `ManagedProcess::{take_stdin,take_stdout,take_stderr}` transfer pipe ownership.
- `read_bounded_line(&mut AsyncBufRead, max_bytes) -> io::Result<Option<BoundedLine>>` caps one physical line before allocation grows past `max_bytes`.

### 3. Contracts

- Unix children start in a new process group. Cancellation reserves half of `grace` for graceful direct-child exit, then sends `SIGKILL` to the group even if the direct child has already exited. Forceful termination, reap, and group-exit confirmation use the remaining deadline.
- Windows children are attached to a Job Object with `KILL_ON_JOB_CLOSE`; cancellation terminates the job, reaps the direct child, and confirms `ActiveProcesses == 0` through Job Object accounting.
- Direct-child reap and tree cleanup are separate states. Only successful child wait records reap; only an absent Unix process group or an empty Windows Job Object records completed tree cleanup.
- `wait` also terminates remaining owned descendants after direct-child exit and allows at most five seconds for tree-exit confirmation. An enclosing execution deadline can cancel this wait. A missing cleanup proof returns an error.
- Detached mode (`spawn_detached`) inverts the descendant contract for commands whose product must outlive them: a successful `wait` does not terminate descendants, Drop does not terminate the tree, and the Windows Job Object is created without `KILL_ON_JOB_CLOSE`. Explicit `terminate_tree` still reclaims the whole tree in both modes; the default `spawn` contracts above are unchanged.
- Drop force-terminates an unconfirmed tree even when the direct child was reaped. Drop is a last resort and does not prove successful cleanup; callers must still await a terminal method.
- No production caller may mark a job terminal before `wait` or `terminate_tree` returns.
- Child stdout/stderr that is not consumed by the foreground capped reader must use `read_bounded_line`; bounded queues do not bound `AsyncBufReadExt::lines()` before a newline arrives.

### 4. Validation & Error Matrix

- Spawn fails -> return the OS error; no registry entry is created.
- Job/process-group attachment fails -> return the OS error and do not expose an unmanaged child.
- Graceful termination exceeds `grace` -> force-terminate the tree, then wait/reap.
- Tree termination or wait fails -> caller reports cleanup failure; it must not report cancellation success.
- Direct child exits while a descendant ignores `SIGTERM` -> force the owned group, then confirm group exit before reporting completion.
- Tree-exit confirmation reaches the deadline -> return `process_tree_cleanup_timeout`; leave cleanup unconfirmed for the Drop fallback.
- Unterminated line exceeds `max_bytes` -> consume through newline/EOF with constant retained memory and return `BoundedLine.truncated = true`.

### 5. Good/Base/Bad Cases

- Good: a parent that spawns a grandchild leaves no live descendant after `terminate_tree`.
- Good: `spawn_detached` for a launcher whose daemon descendant must keep running after the launcher exits; cancellation stays explicit through `terminate_tree`.
- Base: a normally exiting child is consumed with `wait`.
- Bad: call `Child::kill` and immediately mark the job cancelled.
- Bad: drop a live `ManagedProcess` as the normal cancellation path.
- Bad: feed `BufReader::lines()` directly from an untrusted child into an otherwise bounded channel.

### 6. Tests Required

- Windows fixture: parent starts a grandchild, `terminate_tree` completes, and the grandchild PID is no longer running.
- Unix CI fixture: the same assertion targets a dedicated process group.
- Unix fixtures: parent exits on `SIGTERM` while a descendant ignores it; parent exits normally before `wait`; Drop follows direct-child reap with outstanding descendants.
- Windows fixture: normal `wait` and Drop both clean descendants after the direct child was reaped.
- Detached fixtures (Windows and Unix): a detached parent exits normally, `wait` and Drop leave the grandchild running, the fixture terminates the grandchild itself, and detached `terminate_tree` still reclaims the whole tree.
- A live-tree confirmation fixture must time out without marking cleanup complete, then explicitly clean up its process.
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

#### Wrong

```rust
// 用默认 spawn 启动其产物必须比命令活得更久的命令：wait/Drop 会清掉后代
let child = ManagedProcess::spawn(command)?;
```

#### Correct

```rust
// 产物需要存活时使用分离模式；取消仍走显式 terminate_tree
let mut child = ManagedProcess::spawn_detached(command)?;
```
