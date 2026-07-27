# ProcessGateway implementation plan

## Ordered work

- [x] Add URL parsing/host policies and replace `open_external_url`; cover
  allowed OAuth/loopback cases and file/custom/user-info/host-confusion cases.
- [x] Replace generic port killing with an owned-process lookup and explicit
  unknown-owner response.
- [x] Change job output storage to `VecDeque` and preserve serialization/order.
- [x] Define `ProcessDescriptor`, `TrustedExecutable`, typed execution result,
  and the central registry; add packaged hash-manifest resolution and explicit
  development resolution.
- [x] Implement capped streaming foreground execution and timeout/truncation
  behavior.
- [x] Implement bounded delta batching, sequence/dropped metrics, snapshot
  query behavior, and producer-greater-than-consumer soak tests.
- [x] Implement Unix process-group and Windows Job Object lifecycle handling;
  make terminal state wait for cleanup/reap.
- [x] Migrate command passthrough, install, OAuth helpers, update, and SSH/system
  callers in reviewed capability-sized increments; delete old raw spawn seams.
- [x] Add endless process, output flood, descendant, kill-denied, stalled
  consumer, spoofed binary, and PATH-precedence regression fixtures.
- [x] Update desktop command policy and process observability specs.

## Focused validation

```powershell
cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml command_exec -- --test-threads=1
cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml codex_auth -- --test-threads=1
just frontend-check-quick
just lint-strict
just test
```

Platform-owned process-tree tests run on Windows, Linux, and macOS CI. A child
task cannot claim cross-platform acceptance from unit tests on one OS.

## Rollback checks

- Disabling a migrated capability must fail closed with an actionable error.
- Cancellation failure keeps the job in a cleanup-failed terminal state and
  reports remaining ownership; it must not claim success.
- Audit and UI events must not contain full environments, tokens, or URL
  credentials.
