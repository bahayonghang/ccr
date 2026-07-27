# WebDAV sync hardening implementation plan

## Ordered work

- [x] Add `RemoteEntryName` and exact percent-decoding/one-component validation;
  cover the full malicious href corpus and dependency normalization behavior.
- [x] Add `SyncBudget`, streaming GET/write, visited-set cycle detection, and
  typed limit failures.
- [x] Implement `PullTransaction` with staging, validation, fsync, swap,
  parent-fsync, rollback, and failpoints at every I/O stage.
- [x] Replace boolean sync branching with an exhaustive typed action/conflict
  state and four-combination-by-force tests.
- [x] Make folder-manager config canonical and implement idempotent legacy
  read-through migration under the existing locking/guarded-write contract.
- [x] Enforce HTTPS/loopback-development policy both when saving and connecting;
  test redirects and credential-safe errors.
- [x] Implement the authenticated v2 envelope, per-operation passphrase input,
  operation-scoped key lifetime, plaintext v1 explicit migration, and UI
  encryption-state DTO.
- [x] Add fake-DAV adversarial/fault fixtures for list, GET, stream, mkdir,
  write, fsync, rename, parent-fsync, restore, oversized input, depth, cycles,
  and hostile names.
- [x] Update ccr-sync and desktop sync specs after the implementation contract is
  proven.

## Focused validation

```powershell
cargo test -p ccr-sync -- --test-threads=1
cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml sync -- --test-threads=1
just frontend-check-quick
just lint-strict
just test
```

The final child check must also inspect a fake remote fixture and prove that no
plaintext secret asset bytes are present after a v2 push.

## Risk and rollback checks

- Keep secret masking, file locks, backups, and atomic replacement intact.
- A failed pull at every injected stage must preserve exact active bytes.
- v1 migration must be idempotent and leave an exportable backup.
- Do not log URLs with credentials, passphrases, keys, plaintext payloads, or
  decrypted filenames beyond normalized asset identifiers.
- Prove passphrases and derived keys are not serialized to config, persisted in
  the local secret store, or retained after the sync operation completes.
