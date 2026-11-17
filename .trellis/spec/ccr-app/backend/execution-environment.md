# Execution Environment Contracts

Applies to the native `ccr-desktop` consumer and `ccr-app::environments`.
The Tauri OpenSSH compatibility API remains governed by
[SSH Trust and OpenSSH Transport](../../ccr/backend/ssh-security.md).
Source: [approved SSH specification #205](https://github.com/bahayonghang/ccr/issues/205)
and the implementation slices #206–#214. Fixture evidence is distinct from real-host acceptance.

## Current environment and admission

- The Shell alone publishes the active `EnvironmentId`. Services held by a feature are captured snapshots, never another global publisher.
- Local and SSH follow-environment operations use `EnvironmentPolicy::FollowsEnvironment` and capture `EnvironmentIo` before queueing. Every SSH operation binds host ID **and session UUID**. Same-host reconnect cannot grant an old request new authority.
- `RequestKey` and `ResponseGate` retain environment/platform/generation/request identity. Switching clears old view data and drafts only through the explicit switch decision. Late results cannot replace the new environment's view.
- Selection persists in desktop preferences; startup attempts a saved SSH connection once. Failed/closed connections retain the SSH selection and show failure/retry. Never fall back to local files, processes or measured zeros.
- Independent check-ins, global settings, operation-record storage, command favorites/history and the text converter remain usable while SSH is offline. WSL is not a switchable environment.
- Platform roots derive from the captured remote POSIX HOME. The process HOME is never changed. Portable configuration mirrors have explicit input/output allowlists, bounded private staging and no ambient local discovery.

## Transport, trust and credentials

- Native Workspace uses one persistent russh 0.65.x connection with ring/RSA/flate2 and russh-sftp 3.x. Authenticate exclusively; admitted operations open independent channels. Connect plus authentication has a ten-second timeout; no automatic reconnect after failure.
- Probe records the observed server key and rejects authentication. Only explicit opaque challenge confirmation persists a trusted key using the existing locked, atomic CAS known-host writer. Connect accepts only the stored matching fingerprint; changed keys are refused.
- `agent`, private-key path and password authentication belong to the desktop-specific boundary. Password/keyboard-interactive and encrypted-key passphrases use keyring 4.x through blocking work. Keys are indexed by host ID, removed with the host, and never stored in SQLite, logs, exports or sync. Private key contents are not copied. Tests use mock credential stores.
- Keep Tauri `ssh_security`/`ssh_session` signatures, file format, strict OpenSSH argument behavior and password refusal semantics unchanged. The pure `boundary_before_connect`/WSL planning helpers do not establish a socket or authorize native I/O.
- Writes serialize per path, compare the captured version, finish the preimage backup, upload a UUID sibling and require `posix-rename@openssh.com`. Unsupported atomic replacement fails closed. This is a pre-save version check, not server-side atomic CAS against unrelated external clients.
- Configuration reads/writes retain 2MiB limits. Session reads have a separate 64MiB limit. Ordinary read cancellation joins the issued operation within its original ten-second deadline; successful read cancellation does not disconnect a shared session. Unconfirmed cleanup revokes only the captured session.
- Exec cancellation/transport failure is fail-closed and cannot claim the remote process tree exited without proof. Shut down SSH before the application runtime.
- Authority rejected before exec issues no command. If authority is lost after exec starts, its result cannot confirm side-effect or private-object cleanup; report cleanup failure without borrowing or revoking a replacement session.

## Sync and usage owner boundaries

- `ccr-sync` owns WebDAV protocol, v2 encryption, explicit plaintext-v1 import, conflict rules and local SQLite transactions. SSH only selects current-host assets and publishes ordinary file changes through the captured I/O. Passphrases are operation-only secrets, absent from snapshots and logs.
- SSH reads `~/.ccr/sync_folders.toml`. Four fixed file assets reuse `AssetService`; ordinary folders stay under captured HOME and allow at most 128 nodes/12 levels, 2MiB per file and 16MiB total. Paths must retain portable identity. Empty source directories are materialized for upload.
- Without a remote transaction owner, SSH `ccr-platforms` and ordinary folders containing SQLite database/sidecar names are explicitly refused. Never SFTP-rename live `profiles.db`. SSH legacy credential clearing/WebDAV disconnect and creating new empty destination directories remain unsupported. Multi-file publication reports partial completion and retains remote preimage backups; it is not a cross-file transaction.
- Installed remote llmusage sync executes through the captured `EnvironmentIo`; a missing CLI never invokes the local executable. The original NDJSON parser owns events. Usage stream execution does not retain whole streams: stdout lines are bounded at 1MiB and stderr retains at most 64 lines of 64KiB. `finished` remains provisional until exec and both streams finish. Preserve the one-hour sync descriptor. Cancellation keeps admission until bounded cleanup returns; unconfirmed remote-tree exit is cleanup failure.
- Remote usage requires Python3 with standard sqlite3. The helper opens `~/.llmusage/llmusage.db` with `mode=ro` and uses SQLite backup API, including committed WAL. Never use `immutable` to ignore WAL or copy a live DB file as a snapshot. Reject source/root links before reading or creating staging; cleanup rejects linked target ancestry too.
- A backup uses a random owner-private remote directory and owner-private file. Helper exec is bounded at ten seconds (eight-second SQLite progress budget). Before-start cancellation issues no helper; an issued private helper joins within its original descriptor deadline so its result path can be cleaned exactly. View cancellation does not apply mutating-exec cancellation to that helper or revoke the shared connection. It prevents the subsequent download/cache publication and becomes terminal after cleanup. Usage SFTP has an independent 64MiB/600-second download budget; configuration/session budgets are unchanged. Cancellation joins the in-flight usage read for five seconds, then revokes only the captured session if cleanup cannot be confirmed (the existing disconnect has a ten-second bound).
- Validate SFTP metadata before/after, local integrity and schema before returning a cache snapshot. Exact remote cleanup runs even after cancelled reading; cleanup failure prevents cache publication. A disconnect may leave a remote `.ccr-snapshot-*` directory requiring manual cleanup; never borrow a reconnected session to remove it.
- Local caches hash host IDs into separate private roots, then use request-private temporary directories with scoped locks. Requests remove their cached snapshot on drop. Stale/failed snapshots and local usage/archive/Observer paths cannot fill remote gaps.
- All usage SQL, integrity/schema checks and read-only projections remain in `ccr-usage`. Minimum schema 10, provider features 14, required-column/feature checks for future additive versions; CCR never migrates usage DBs or links the upstream llmusage Rust crate. Unknown usage/session counts remain unavailable.

## Pure and compatibility helpers

- `plan_wsl_file`/`resolve_wsl_file` only validate and plan: no process, distro creation, file read or write. Missing/blank distro, drive/UNC path, or traversal returns `Unavailable`; a usable distro plus absolute POSIX path returns `Remote`. Neither result implies working WSL I/O or a live probe.
- The compatibility `boundary_before_connect` and reconnect helpers are pure decisions, never socket/trust authorization. Caller-held passwords remain refused by the Tauri boundary; missing/current mismatched keys fail even with remembered process trust. `decide_host_key` never grants implicit trust. Errors remain static codes without credentials or host/key details.
- `Workspace::retained_session_count` and native Monitoring only read retained in-memory authority: no owner means 0, one retained Session means 1, disconnect means 0. This is not reachability. Do not initialize storage, construct hosts, scan identities or perform I/O to supply that metric.
- `EnvironmentSurface` caches/drafts/jobs remain per environment; switching hides the prior slot and increments generation. `read_local_source` runs its closure only when caller and owner are the same named Local environment. Other Local names, SSH and WSL never run that closure. These pure tests do not substitute for native environment-switch acceptance.

## Validation anchors

- `ccr-app` I/O fixtures: session authority, configuration budget, independent usage download deadline and short cancellation cleanup; fixed CLI/transport and credential mock tests.
- `ccr-usage::snapshot` tests: committed WAL backup, source-byte preservation, deterministic close/fsync/cleanup, link refusal, schema 9/10/14/future read-only gates.
- Native `runtime::remote_activity_tests`: SSH usage projection, cache isolation, missing dependencies, cleanup failure, stale authority, multi-megabyte legal NDJSON and provisional terminal behavior.
- Native `runtime::sync::operations::tests`: remote sensitive v2 roundtrip/CAS/preimage backup, ordinary and empty source directories under production metadata semantics, database refusal and no local fallback.
- Shell environment headless fixtures retain independent pages while SSH is offline. Real SSH, native credentials, server rename support, external WebDAV and other operating systems require separate receipts. Default parallel tests and the separate binding-export gate remain unchanged.
