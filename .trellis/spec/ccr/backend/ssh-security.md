# SSH Trust and Transport

> Backend-owned OpenSSH capability and host-trust contract for the desktop app.

## Scenario: trusted SSH config access

### 1. Scope / Trigger

- Applies when changing `ccr-ui/src-tauri/src/ssh/`, `platform/ssh.rs`, SSH Tauri commands, or their frontend DTOs.
- The renderer may request a probe and approve an opaque challenge. It never owns executable arguments, trusted fingerprints, or known-hosts content.
- Config files use system OpenSSH SFTP. Fixed CLI detection and the nonce handshake are the only remote shell commands.

### 2. Signatures

- `ssh_probe_host_fingerprint(request: { env_id?, host?, port? }) -> { challenge_id, host, port, key_type, public_key, fingerprint, status, stored_fingerprint? }`
- `ssh_confirm_host_fingerprint(request: { challenge_id }) -> ()`
- `SshTarget::new(host, port, user?, identity_file?) -> Result<SshTarget, String>`
- `RemotePosixPath::root(remote_home?) -> Result<RemotePosixPath, String>`
- `SshConnectionManager::test_connectivity(config) -> Result<SshConnectResult, String>`
- `SshConnectResult` includes `success`, `latency_ms`, `error_code`, and `error`.
- Trust file: `<CCR_ROOT>/ssh/known_hosts`, defaulting to `~/.ccr/ssh/known_hosts`.

### 3. Contracts

- Host values are normalized bracket-free DNS names or IP literals. User values use `[A-Za-z0-9._-]+`; neither may be empty, option-shaped, or contain whitespace/control characters.
- Identity files are one local OS argument and reject empty, control, surrounding-whitespace, and leading-dash values.
- `remote_home` is exactly `~`, `/`, or an absolute POSIX path whose non-root segments use `[A-Za-z0-9._-]+`. Relative config paths pass the shared traversal normalizer and the same segment grammar.
- Probe validates one requested-host key, computes the OpenSSH-compatible SHA-256 fingerprint, and registers a backend-held UUID challenge for 120 seconds.
- Confirm consumes only `challenge_id`. Consumption is atomic and single-use. The backend-held key is atomically written with owner-only permissions before its fingerprint is recorded in SQLite.
- A matching stored fingerprint may reconstruct a missing app known-hosts entry. New and mismatched keys require explicit confirmation.
- Every connection `ssh` and `sftp` command forces `BatchMode=yes`, `StrictHostKeyChecking=yes`, `GlobalKnownHostsFile=none`, the app-owned `UserKnownHostsFile`, and finite timeouts.
- Connection becomes active only after exact nonce stdout. Any failed connect switches an already-active target back to local and records it disconnected.
- SFTP writes upload to a UUID sibling first, run rename only after upload success, and issue best-effort cleanup after upload, rename, or transport failure.

### 4. Validation & Error Matrix

| Condition | Required result |
| --- | --- |
| Invalid host/user/identity/remote path | `ssh_invalid_*` rejection before process creation |
| Malformed/unknown/expired/replayed challenge | `ssh_challenge_invalid` / `unknown` / `expired` / `reused` |
| Changed host key | `ssh_host_key_mismatch`; connection remains inactive |
| Unconfirmed host key | `ssh_host_key_untrusted`; connection remains inactive |
| Password-only or interactive authentication | `ssh_auth_unsupported` |
| Spawn, timeout, or transport failure | `ssh_network_error` at the connection API boundary |
| SFTP upload or rename failure | target is not promoted; staged sibling cleanup is attempted |

### 5. Good / Base / Bad Cases

- Good: `/home/deploy`, `~`, `deploy-user`, and a separately passed `C:/Keys/id key` argument.
- Base: a stored matching fingerprint reconstructs a deleted app known-hosts entry from the freshly scanned matching key.
- Bad: `$()`, backticks, backslashes, quotes, CR/LF, traversal, repeated separators, or `--option` in trust or path fields.
- Bad: frontend confirmation sends host, fingerprint, key data, or OpenSSH options.
- Bad: config read/write uses `cat`, redirection, `mkdir -p`, or another renderer-influenced shell fragment.

### 6. Tests Required

- Unit/property tests for hostile boundaries, deterministic path grammar, strict OpenSSH arguments, and OpenSSH-compatible fingerprints.
- State tests for new/match/mismatch plus malformed, expired, and replayed challenges.
- Fake OpenSSH tests proving upload failure skips rename and rename failure follows upload; both must attempt staged cleanup.
- Frontend smoke proving confirm sends only `challenge_id`.
- Run `cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml ssh -- --test-threads=1`, SSH/API smoke tests, `bun run type-check`, `just frontend-check-quick`, `just lint-strict`, and `just test`.

### 7. Wrong vs Correct

#### Wrong

```rust
command.arg(format!("cat \"{remote_home}/.claude/settings.json\""));
```

```typescript
await confirm({ host, fingerprint })
```

#### Correct

```rust
let target = SshTarget::new(host, port, user, identity_file)?;
let path = RemotePosixPath::root(remote_home)?.join_relative(".claude/settings.json")?;
```

```typescript
await sshConfirmHostFingerprint(probe.challenge_id)
```

## Rollback

Disable SSH remote writes and leave local profiles available. Never restore `accept-new`, renderer-provided fingerprints, or shell-interpolated file operations.
