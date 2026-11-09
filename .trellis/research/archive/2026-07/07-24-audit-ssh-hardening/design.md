# SSH trust and transport technical design

## Boundary

All SSH connection arguments are backend-owned structured values. Remote file
operations use the OpenSSH `sftp` subsystem rather than a remote shell. Trust is
anchored in an app-owned known-hosts file, and the renderer can approve only a
backend-issued challenge ID.

## Validation model

- `SshHost`, `SshUser`, and `RemotePosixPath` are validated newtypes.
- Host and user reject empty values, control characters, whitespace, and a
  leading `-`; host accepts DNS names, IP literals, and bracket-free normalized
  values only.
- `RemotePosixPath` accepts `~` or an absolute path whose non-empty segments
  match `[A-Za-z0-9._-]+`. It rejects backslashes, traversal, quotes, shell
  metacharacters, controls, CR/LF, and option-shaped values.
- Identity paths are local `PathBuf` values passed as one OS argument after an
  explicit `--`/option boundary where supported; they are never interpolated.

## Trust state machine

1. Probe runs `ssh-keyscan` through the backend process helper, parses a single
   host/key record, computes the fingerprint through `ssh-keygen`, and stores a
   short-lived `HostKeyChallenge` keyed by opaque `challenge_id`.
2. The renderer receives host, port, key type, and display fingerprint. It does
   not receive an API that can submit a fingerprint string.
3. Confirm consumes the challenge and atomically updates the app-owned
   known-hosts file under a file lock. A changed/replayed/expired challenge is
   rejected.
4. Every `ssh` and `sftp` command supplies `UserKnownHostsFile`,
   `StrictHostKeyChecking=yes`, and `BatchMode=yes`. A mismatch is classified as
   a blocking security error.
5. Connection state becomes `connected` only after a real nonce handshake
   succeeds with the same trust options.

## File transport

- Reads use `sftp -b -` with a generated batch containing only commands and
  validated paths. Writes upload to a sibling temporary path, then use SFTP
  rename after upload completion.
- Batch generation has a dedicated encoder for the SFTP command grammar; it is
  not reused as shell escaping.
- The legacy shell transport is removed for writes. A temporary read-only
  compatibility feature may be retained only if its input passes the same
  path grammar and standard POSIX single-quote encoding.

## Compatibility and dependencies

The design reuses installed OpenSSH tools and does not add a production Rust
SSH dependency. Existing key/agent authentication remains supported. Password
automation is not weakened by embedding credentials in command arguments; an
environment without key/agent support returns a typed unsupported-auth result.

## Rollback

The safe rollback is to disable remote writes and keep local configuration
available. It is not acceptable to restore `accept-new`, renderer-provided
fingerprints, or shell-interpolated write paths.
