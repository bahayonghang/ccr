# Sync Security Contracts

## Scenario: Typed encrypted asset sync across renderer and Tauri

### 1. Scope / Trigger

- Trigger: changing fixed sync assets, renderer sync APIs, Tauri sync commands,
  WebDAV configuration ownership, conflict behavior, or passphrase UI.
- This contract keeps renderer, IPC, backend decisions, and persisted config in
  one testable flow.

### 2. Signatures

- Tauri asset commands accept one `payload: SyncAssetOperationInput`:
  `sync_push_asset`, `sync_pull_asset`, and `sync_asset`.
- Batch command: `sync_all_assets(payload: Option<SyncAllAssetsInput>)`.
- Payload fields use renderer camelCase: `id`, `force`, `passphrase`, and
  `migratePlaintextV1`.
- Renderer wrappers pass `{ payload: { id, ...options } }` or
  `{ payload: options }` from `src/api/domains/sync.ts`.
- Asset DTO includes `sensitive` and `encryptionState` (`not_applicable` or
  `v2_required`).

### 3. Contracts

- Sync action truth table:

| Local | Remote | Force | Action |
| --- | --- | --- | --- |
| present | missing | either | push create |
| missing | present | either | pull |
| present | present | false | typed conflict |
| present | present | true | push overwrite remote |
| missing | missing | either | missing error |

- Sensitive operations require a non-empty independent operation passphrase.
  WebDAV Basic Auth credentials are never reused as the envelope passphrase.
- The renderer prompts before each sensitive single/batch operation. The modal
  clears its local passphrase on submit and close. The backend keeps the secret
  only in the command payload and call stack; it is never serialized to config,
  logs, output summaries, events, or a secret store.
- Plaintext v1 read is disabled by default and enabled only by the explicit
  `migratePlaintextV1` checkbox. New sensitive writes remain v2.
- `sync_folders.toml` is the canonical WebDAV configuration. `sync.toml` is
  migration-only and may be read only when the canonical file does not exist.
  Canonical file presence, not credential completeness, closes migration.
- Saving writes only canonical config and validates HTTPS before persistence.
  Disconnect deletes legacy config before clearing canonical credentials.
- Pull delegates replacement and backup handling to `ccr-sync::PullTransaction`;
  Tauri must not rename the active path before download.

### 4. Validation & Error Matrix

- Sensitive operation without passphrase ->
  `sync_envelope_passphrase_required` in the operation failure.
- Both local and remote without force -> `sync_conflict`; renderer offers an
  explicit force retry.
- Neither side exists -> missing error; no implicit create.
- Plaintext v1 without checkbox ->
  `sync_envelope_plaintext_v1_requires_migration`.
- Non-loopback HTTP save/test -> `sync_transport_https_required`; canonical
  config is not written.
- Canonical config parse error -> report canonical error; never fall back to
  legacy credentials.

### 5. Good/Base/Bad Cases

- Good: `pushSyncAsset('codex-config', { force: true, passphrase })` sends one
  typed payload and writes only v2 envelope bytes remotely.
- Good: remote-only non-sensitive asset pulls through the same transaction.
- Base: non-sensitive assets do not prompt and report `not_applicable`.
- Bad: pass separate `id`, `force`, and `passphrase` IPC arguments.
- Bad: persist an operation passphrase in Pinia, local storage, or WebDAV config.
- Bad: read legacy config because an existing canonical config is disabled.

### 6. Tests Required

- Tauri exhaustive presence/force truth-table test.
- Tauri tests for passphrase requirement, canonical-only save, canonical file
  migration closure, legacy one-time migration, and HTTP rejection before write.
- Frontend API smoke test asserting the exact nested payload shape.
- Sync view smoke tests for prompt, explicit migration checkbox, force retry,
  batch operation, passphrase clearing, and v2 status copy.
- Run `cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml sync -- --test-threads=1`,
  `bun run type-check`, `bun run lint`, and `bun run test`.

### 7. Wrong vs Correct

#### Wrong

```typescript
invoke('sync_push_asset', { id, force, passphrase })
```

#### Correct

```typescript
invoke('sync_push_asset', {
  payload: { id, force, passphrase, migratePlaintextV1: false },
})
```


---

## UI Contract: connection-state gating on the Sync page

- `useSyncPage` derives `connectionState` from `sync_status` fields alone:

| configured | remote_accessible | connectionState | Sync actions |
| --- | --- | --- | --- |
| false | any | `unconfigured` | disabled + setup guide card (CTA opens account dialog) |
| true | false | `unreachable` | disabled + `role="alert"` warning banner with retest |
| true | true | `connected` | enabled |
| true | null | `unknown` (untested) | enabled |
| status fetch failed (`syncStatus === null`) | — | `unknown` | enabled — never gate on IPC failure |

- Gating disables per-asset Push/Pull/Sync and header Sync all / Force retry
  all with a localized `title` reason (`sync.gating.disabled*` keys). Account
  configure / test / disconnect stay available in every state.
- Busy attribution: only the card with `busyAssetId === asset.id` shows a
  spinner and busy label; a global batch operation disables all card buttons
  without spinning them, and only the header Sync all shows progress.
- The sidebar order is fixed: WebDAV config card -> operation output panel ->
  collapsed "About sync" `<details>` (safety list, features, supported
  services). New output scrolls into view with `readPrefersReducedMotion()`.
