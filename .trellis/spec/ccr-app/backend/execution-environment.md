# Execution Environment Decisions

## 1. Scope / Trigger

Applies when changing `crates/ccr-app/src/environments/` or the GPUI settings consumer in `ccr-desktop/src/features/environments/` and `ccr-desktop/src/runtime/environments.rs`.

The original decision helpers remain pure. The separate `ssh::Workspace` owns saved-host sessions and SFTP operations, using the extracted `ssh_security` contract also re-exported by Tauri. This does not replace `.trellis/spec/ccr/backend/ssh-security.md` or change Tauri command payloads. Do not describe `boundary_before_connect` as a completed socket connection, and do not describe `plan_wsl_file` as a WSL file read or write.

## 2. Signatures

- `ssh::Workspace::retained_session_count() -> usize` reads only the optional-session lock, returns0/1, and does not construct/inspect a host.


- `plan_wsl_file(intent: WslFileIntent, distro: Option<&str>, linux_path: &str) -> WslFileTarget`
- `resolve_wsl_file(distro, linux_path)` is `plan_wsl_file(WslFileIntent::Read, ...)`
- `WslFileIntent::{Detect, Read, Write}`
- `WslFileTarget::Unavailable` or `WslFileTarget::Remote { distro, path }`
- `boundary_before_connect(kind: ConnectKind, facts: ConnectFacts) -> Result<(), &'static str>`
- `ConnectKind::{Open, Again}`
- `ConnectFacts { cancel_requested, password_only, stored_fingerprint, observed_fingerprint, process_trusted }`
- `password_only(password: Option<&str>) -> bool` is true only when the caller already holds a non-empty trimmed value. It does not open an identity file.
- `EnvironmentSurface<T>::{new, switch, note_cache, note_draft, note_job, visible, accepts}`
- `admit_local_only(environment: &EnvironmentId) -> Result<(), &'static str>`
- `read_local_source(caller, owner, read) -> Result<T, &'static str>`
- Static codes: `ssh_cancelled`, `ssh_auth_unsupported`, `ssh_host_key_untrusted`, `ssh_host_key_mismatch`, `ssh_not_connected`, `local_only_environment`

## 3. Contracts

Native Monitoring observes an already captured Workspace's retained session capability without any IO. No owner means0; one stored Session means1 even when lookup authority is unavailable. This is not a live socket/reachability probe or Tauri's remembered SSH-state/password map. Do not scan identities or initialize Workspace/host storage to supply this metric. `retained_session_metrics_observe_only_memory_and_follow_real_owner_transitions` proves0/1/disconnect0 with poisoned transport/removed lookup authority.


`plan_wsl_file` does not spawn `wsl.exe`, does not create a distribution, and does not open the path. Detect, read, and write use the same rule. A missing distro, a blank distro, a drive letter, a UNC path, or a `..` segment returns `Unavailable`. A usable distro name plus an absolute POSIX path returns `Remote` with that distro and that POSIX path. `Unavailable` and `Remote` never contain a Windows drive or UNC path.

A live `wsl -l -v` / `wsl -e true` classification is separate. On the current Windows machine the recorded result is `Listed { names: ["docker-desktop"], exec: PathNotFound }`. That is a failed default distribution, not a usable read/write target. Non-Windows `Unsupported` stays `not_run` until that host executes it.

`boundary_before_connect` returns before any socket, `ssh-keyscan`, known_hosts write, or identity-file read. `Ok` means this decision found no refusal. It does not open a connection. Cancel returns `ssh_cancelled`. A caller-held non-empty password returns `ssh_auth_unsupported`. A supplied fingerprint that does not match returns `ssh_host_key_mismatch`. A new fingerprint, an unknown fingerprint, or a match without process trust returns `ssh_host_key_untrusted`. `decide_host_key` keeps `may_store`, `may_trust`, and `may_accept` false. A matching fingerprint alone does not call `note_confirmed`. Reconnect (`ConnectKind::Again`) also requires process trust for the already loaded host and port. Never-connected disconnect stays `ssh_not_connected`, not a successful remote disconnect.

Reconnect also refuses absent or blank current key material even when `process_trusted` is true. Remembered process trust cannot stand in for a current matching host key. The only production `Open` caller supplies empty facts for the local settings inspection; its `Ok` is not SSH authorization and must not be used to activate a connection.

Error text is the static code only. It must not contain a host, fingerprint, password, identity path, or `ssh-keyscan`.

`EnvironmentSurface` stores cache, draft, and job labels per `EnvironmentId`. `switch` hides the previous slot and bumps generation. `visible` returns only the active slot. `accepts` requires the same environment, generation, and `ResponseGate`. These helper tests alone are not product environment-switch evidence.

The native Settings `RemoteView` manages saved hosts and explicit Local/SSH selection. Selection is not connection success: only the strict OpenSSH nonce handshake activates the session. Remote source drafts belong to `(host ID, fixed config file)` and each loaded `Document` carries an opaque session identity. Reconnect invalidates old write authority even for the same host ID. Remote selection blocks Local-only page loads and actions; it never substitutes Local data. Existing Local jobs remain owned by their original runtime, not relabeled as remote work.

Host CRUD reuses `ccr-db::ssh_repo`; only explicit host writes may initialize its storage schema, with no legacy personal-data import. New hosts use SSH agent authentication; editing preserves existing backend identity settings without reading identity bytes. Trust confirmation consumes an opaque backend challenge and retains strict known-host checking. Probing or confirming a key invalidates the previous connection and requires another nonce handshake; changing known-host trust must never leave an old session or document authorized. `ssh_security` is the single shared owner for target/path validation, challenges, fingerprints, known-host persistence and strict argv.

SFTP saves validate JSON/TOML, check the freshly downloaded version against the loaded snapshot, require a remote backup, then upload to a UUID sibling and rename. This is **pre-save version checking**, not server-side atomic CAS: another client can write between comparison and rename. A reconnect or session replacement rejects stale documents. A failed or cancelled rename whose outcome cannot be established reports `ssh_write_outcome_unknown`, never fabricated cancellation success. Transport failures invalidate the active connection; remote drafts remain available for recovery.

Fixture launches reject SSH network operations even when their isolated database has host rows. Rendered tests explicitly inject a memory transport into the same native action/runtime path. Production Settings refresh disables automatic WSL discovery while WSL remains user-deferred; it does not invent a distribution. Real SSH, OS window interaction and cross-platform acceptance require separate receipts.

The explicit native SSH CLI check uses `Workspace::cli_session` and `detect_cli` for the already trusted, connected saved host. Admission captures its opaque session identity; the worker rejects environment mismatch, disconnect or reconnect before issuing any fixed PATH check. No local CLI lookup or version command is substituted. The Settings consumer clears results on host changes, connection actions and leaving Settings, cancels an in-flight CLI check on leave, and rejects late generations without clearing unrelated SFTP drafts. Fixture and benchmark sources refuse live detection before transport; a test-only injected memory transport exercises the same owner path.

`runtime::environments::load` calls `admit_local_only` before `inspect`. A WSL or SSH key returns `local_only_environment` and does not run the local read. `read_local_source` runs the closure only when caller and owner are the same `EnvironmentId::Local` name. Another local name, WSL, or SSH returns `local_only_environment` and does not call the closure.

GPUI tray and Windows hide/restore stay `Unsupported`: this gpui-kit stack has no tray type, and Windows hide has no restore. Do not add a tray crate or a second window stack. Unsupported is not completion. macOS and Linux native shell evidence stays `not_run` when the command ran on Windows. Paper stays the existing solid sheet. Do not claim blur from this decision layer.

## 4. Validation & Error Matrix

| Condition | Required behavior |
| --- | --- |
| No distro, blank distro, drive letter, UNC, or `..` | `WslFileTarget::Unavailable`; local file bytes unchanged |
| Named distro and absolute POSIX path | `Remote` keeps that distro and path; no `wsl.exe` |
| Live probe lists only `docker-desktop` and exec is `PathNotFound` | Record the failure; do not create a distro or treat it as usable |
| Cancel requested | `ssh_cancelled` before connect |
| Non-empty password already held by the caller | `ssh_auth_unsupported`; do not read an identity file |
| New, unknown, or unmatched host key | `ssh_host_key_untrusted` or `ssh_host_key_mismatch`; nothing stored |
| Matching fingerprint without process trust | Still `ssh_host_key_untrusted` |
| Reconnect with remembered process trust but no current key | Still `ssh_host_key_untrusted` |
| `boundary_before_connect` returns `Ok` | No socket was opened |
| Never connected | `ssh_not_connected` |
| Cache, draft, or job noted on another environment | Hidden after `switch`; visible slot does not render them |
| `load` or `read_local_source` for a non-owner environment | `local_only_environment`; source closure or inspect does not run |

## 5. Good / Base / Bad Cases

- Good: plan `/home/user/.claude/settings.json` on `docker-desktop` and keep the POSIX path without opening it.
- Base: settings refresh on `EnvironmentId::Local("local")` with empty connect facts continues to the existing local inspect.
- Bad: return `C:\Users\...` from `plan_wsl_file`, accept a changed host key because the fingerprint text was non-empty, or treat `Ok` as `connected: true`.
- Bad: run `ssh-keyscan`, open a TCP socket, read `id_rsa`, or install a WSL distribution to satisfy a fixture.

## 6. Tests Required

- `cargo test -p ccr-app --locked --lib environments -- --skip export_bindings` covers path planning, host-key refusal, reconnect, never-connected disconnect, surface isolation, and the local-only gate with default parallelism.
- Desktop settings tests cover the same decisions through the rendered page, one language at a time, the existing 13 destinations, and solid paper.
- Fixture suites keep the live WSL probe ignored. Live probes require separate authorization and receipts; exit 0 of a print-only probe is not read/write acceptance. The historical `PathNotFound` classification is a failure receipt.
- Do not weaken the usage byte comparison. A single file-lock or byte-comparison flake may be rerun once.
- Tauri command signature tests stay on the existing SSH contract. This spec does not authorize a payload change.

## 7. Wrong vs Correct

Wrong:

```text
plan_wsl_file(Read, None, "C:\\Users\\example\\settings.json") -> open that file
boundary_before_connect(Open, new fingerprint) -> store known_hosts and connect
EnvironmentSurface.switch(ssh) -> visible() still returns the local draft
```

Correct:

```text
plan_wsl_file(...) -> Unavailable or Remote { distro, posix path }; file bytes unchanged
boundary_before_connect(...) -> static error, or Ok with no socket
switch -> visible() is only the active environment's slot
admit_local_only(Ssh | Wsl) -> local_only_environment before inspect
```
