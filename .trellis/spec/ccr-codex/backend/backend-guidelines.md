# ccr-codex Backend Guidelines

> Dedicated Codex and OpenCode domain crate.

## Scope

`crates/ccr-codex` owns Codex/OpenCode auth, profiles, session visibility, quota, runtime services, usage records, and history sync. CLI commands and UI backends should call this crate instead of reading `~/.codex` or OpenCode files directly.

Reference files:

- `crates/ccr-codex/src/lib.rs`
- `crates/ccr-codex/src/utils.rs`
- `crates/ccr-codex/src/services/`
- `crates/ccr-codex/src/managers/codex_config.rs`

## Structure

Keep this split:

- `models/` for serialized Codex/OpenCode contracts.
- `managers/` for local config/auth stores.
- `platforms/` for `CodexPlatform`.
- `services/` for auth, runtime, quota, session, usage, and history workflows.
- `utils.rs` for path resolution, private permissions, and small encoding helpers.

## Filesystem And Security

Use `CodexPaths` instead of direct Codex home-directory joins. The retired OpenCode Auth `OpenCodePaths` helper has been removed; this does not remove other OpenCode configuration or usage consumers. Preserve `CCR_CODEX_DIR`, `CCR_DATA_DIR`, and `CCR_LOCK_DIR` overrides for tests and controlled environments.

Auth files and exported account snapshots are security-sensitive. Preserve masking, private-file permissions, backup-before-destructive-change behavior, and repair/sync flows.

Pending OAuth login credentials are an explicit no-backup domain.
`services::CodexOAuthPendingStore` owns `oauth_pending.json` under
`CodexPaths::ccr_codex_dir`. CLI/desktop consumers use `save`, `load(now)`, and
`clear`; the desktop owns its listener and events. The existing desktop
adapter passes its published `PlatformPaths` / `CCR_ROOT` path to `with_path`
so a distinct `CCR_DATA_DIR` does not silently move pending credentials.
No credential copies or path migration are part of this change. A store operation lock
serializes save with cancellation and expiry cleanup; the guarded path lock
remains a leaf lock. Saves explicitly use `secret: true` and
`BackupPolicy::None`. Never copy verifier/state to a backup or history file.
Creation/replacement permissions are established by the existing secret writer
before payload bytes; read, permission, write, and cleanup errors propagate.
Callers publish an in-memory state only after persistence succeeds.

`CodexOAuthPendingState` uses `Secret` for verifier/state and URLs that contain
credentials. Default Debug/serialization is masked. Only the private disk DTO
opts into plaintext serialization; the login-start response explicitly exposes
the browser authorization URL. Parse errors report line/column without quoting
input values. Preserve the existing camelCase disk format.

Pending-store tests cover create/replace/cancel/expiry, permission denial,
no-copy filesystem scans, legacy plaintext load, and redacted errors. Run
native Windows ACL/cleanup-denial tests and Unix mode tests on their respective
platforms; source checks cannot replace native permission checks.

The desktop OAuth controller is `commands/codex_auth/oauth.rs`. It owns the
already-bound listener, pending state, cancel token, one completion request,
and terminal cleanup. Bind and secret persistence must succeed before an ID
or authorization URL is published. Startup restore follows the same path.
An active ID can be reused without binding or saving again.

One monotonic deadline and cancellation token cover accept, accepted socket
reads/writes, the token request, and the complete HTTP body. A callback must
match the current ID, state, loopback port, and callback path; duplicate or
expired callbacks cannot replace the accepted callback. A restored callback
is validated before listener publication. Account commit is completion-aware:
the controller cannot detach a secret writer through cancellation. A cancel
request during commit returns `oauth_commit_in_progress`; admission stays held
until the commit and cleanup finish. Pending
cleanup failure is visible and blocks a new login until cleanup succeeds.
Cleanup retry first verifies the same owner still occupies the slot and claims
one retry. Disk cleanup runs without holding the slot or login-state mutex;
a concurrent retry returns `oauth_cleanup_in_progress`. A stale retry must not
clear a replacement login's pending record. Restore re-reads persisted state
after admission and rejects a record cancelled during queueing with
`oauth_saved_login_missing`; restore never recreates a captured old snapshot.

OAuth token errors report status or a fixed diagnostic, never response bodies
or credential values. Existing callback-received and timeout events retain
their payload shape. A callback-received event does not claim token exchange
or account commit succeeded. Port release cancels the controller's own
listener and proven ProcessGateway owners only; unrelated PIDs remain
report-only. Tests use synthetic accounts and loopback servers, never real
OpenAI endpoints or user credentials.

## Error Handling

Return `ccr_core::Result<T>` and map missing auth/config/session state to the existing actionable `CcrError` variants — the variant set is frozen, do not add new ones (see `../../ccr-core/backend/ccr-error-freeze.md`). Avoid panics in runtime discovery and session restore paths; unreadable records should become diagnostics or skipped records with context.

## Logging

Use `tracing` for diagnostics. Never log access tokens, refresh tokens, provider API keys, OAuth payloads, or raw auth JSON.

## Testing

Tests that mutate Codex-related env vars must use `test_support::TestCodexEnv`. Prefer temp homes and fixture files over touching real `~/.codex` state.

## Scenario: Codex Auth Token Cost And Quota Capacity

### 1. Scope / Trigger

Apply this contract to Codex JSONL usage, Auth API equivalent prices, quota
observations, and empirical capacity. `CodexUsageService` owns parsing and scan
quality. `CodexUsageEstimationService` owns attribution, prices, and estimates.
`CodexQuotaObservationStore` owns bounded private metadata. Callers consume typed
results. Keep the existing `ccr-usage` SQL projection and llmusage database intact.

### 2. Signatures

```rust
CodexUsageService::scan(as_of: DateTime<Utc>) -> Result<CodexUsageScan>
CodexUsageService::compute_rolling_usage_at(records: &[CodexUsageRecord], as_of: DateTime<Utc>) -> CodexRollingUsage
CodexUsageEstimationService::load(registry: &CodexAuthRegistry, account_name: &str, as_of: DateTime<Utc>) -> Result<CodexAuthUsageSnapshot>
price_record(record: &CodexUsageRecord) -> CodexRecordPrice
estimate_window(observations: &[CodexQuotaObservation], scan: &CodexUsageScan, records: &[CodexUsageRecord], ledger: &[CodexUsageActivation], account_id: &str, duration: i64, as_of: DateTime<Utc>) -> CodexCapacityEstimate
CodexQuotaObservationStore::record(observations: Vec<CodexQuotaObservation>, now: DateTime<Utc>) -> Result<()>
CodexAuthUsageSnapshot::with_quota(quota: &CodexAccountQuota) -> Self
CodexAuthUsageSnapshot::for_display(now: DateTime<Utc>) -> Self
```

### 3. Contracts

Usage input `I` includes cache read `R` and cache write `W`. Output `O` includes
reasoning `Q`. Require `R + W <= I` and `Q <= O`. Total Token count is `I + O`.
Price the exclusive input `I - R - W`, cache read, cache write, and inclusive
output once. Preserve explicit zero and missing classification as different
states. Invalid classification adds diagnostics and partial status.

Choose one authoritative measurement per proven request/turn identity. Preserve
unrelated turns in files that contain completed events. The persisted
`token_usage_record` uses its request `usage` and response ID; turn/thread
cumulative fields cannot add another charge. Both `context_compacted` and
top-level `compacted` mark unproven epoch boundaries. Use cumulative deltas
only within a proven monotonic epoch. Unproven rollback, model/tier changes, and
compaction retain partial status. Source copies use deterministic session/event
identity. A fork prefix requires matching parent records and a proven boundary.
All same-request merge paths preserve explicit scope evidence. Conflicting
account/model/provider/speed/tier/bucket metadata marks partial; completed Token
authority cannot clear that conflict or a known route mismatch.
Missing, invalid, inferred, and future event times cannot calibrate capacity.
Upgrade the rebuildable usage cache; preserve source JSONL and activation history.

Compute rolling windows against fixed UTC `as_of`. Account attribution uses the
stable account ID and local activation timeline, with inferred scope. Global
fallback never calibrates a selected account. Explicit API/custom route, account,
provider, bucket, or pre-activation session evidence invalidates calibration.
Provider visibility repair cannot prove original billing ownership.

Price each request before aggregation. Canonical `gpt-6.1-sol` uses the verified
2026-10-06 USD/MTok rates: input 2, cache read 0.10, cache write 2.50, output 10.
Inclusive request input above 272000 doubles all input-side rates and multiplies
output by 1.5. Fast doubles the request cost. Period totals cannot select a
request context tier. Preserve model match, price version/source, tier, context
assumption, and record/Token coverage. Unknown models remain unpriced. Older
catalog entries retain catalog-estimate provenance. Missing tier or request
length requires an explicit assumption. Missing classification excludes USD
calibration while Token calibration can remain valid.

Quota acquisition time is fixed when the network response completes. A 30-second
cache hit preserves that time and does not create a new observation. The additive
`CodexAccountQuota.observation` envelope carries provenance and history warnings.
Map main windows by duration: 300 minutes means 5h; 10080 means 7d. Preserve main
and explicit limit-ID buckets separately. Missing percent keeps window presence
unknown. A reset countdown alone cannot identify a stable reset generation.

Store `quota_observations.json` under `CodexPaths::ccr_codex_dir`. Preserve
`CCR_CODEX_DIR`, `CCR_ROOT`/`CCR_DATA_DIR`, and `CCR_LOCK_DIR` path resolution.
Use a normalized operation lock before the guarded writer's leaf lock. Writes
use `secret: true` and `BackupPolicy::None`. Retain at most 35 days, 4096 rows,
and 8 MiB. Reject corrupt, oversized, or unknown-schema input without replacing
the old file. The schema contains quota metadata only. Credentials, emails,
response bodies, prompts, headers, and source paths cannot enter the schema.
History write failure retains successful quota and reports a warning. Keep
sampling on the existing quota query path.

For each interval `[acquisition0, acquisition1)`, require a complete scan whose
watermark covers both endpoints, one account activation, one plan/bucket/window/
reset/model/tier/price basis, a positive Token delta, and at least 5 percentage
points of raw usage increase. Recompute intervals from each completed scan so
late records cannot retain stale capacity. Require three independent intervals;
use at most the last 20. Capacity is `local_delta / (percent_delta / 100)`.
Compute Token and USD median/min/max independently. A max/min ratio above 2
suppresses the affected metric. Remaining capacity scales by `1-used_percent/100`.
Current acquisition must be within five minutes and before reset. Joint remaining
capacity uses the lower compatible window value. The result remains a local
workload estimate with unavailable consumption from other devices and cloud jobs.

### 4. Validation & Error Matrix

| Condition | Domain outcome |
| --- | --- |
| No valid independent intervals or fewer than three | `insufficient_samples` |
| Incomplete scan, ambiguous usage, or invalid classification | `partial_usage` for calibration |
| Known account/route/bucket mismatch | `invalid_scope` |
| Missing duration/reset identity or unsupported window | `unsupported_window` |
| Percent rollback or incompatible segment | Reset baseline; do not use the crossing interval |
| Positive quota delta without local Tokens | `unexplained_quota_change` |
| Unknown or incomplete prices | USD `unpriced`; Token validity remains independent |
| Empirical max/min exceeds 2 | Affected metric `unstable` |
| Acquisition expires or reset passes | Remaining capacity `stale` |
| History read/write fails | Visible `history_error`; preserve successful quota and old bytes |
| Displayed quota differs from the estimate acquisition | Suppress remaining capacity |

### 5. Good/Base/Bad Cases

- Good: three synthetic intervals yield total medians 10M Token and USD 5,
  ranges 8M–12M and USD 4–6, and 50% remaining medians 5M and USD 2.50.
- Base: the first network observation provides a baseline and no capacity.
- Bad: a cached reread has a later return time. Preserve its original acquisition
  and leave the independent sample count unchanged.

### 6. Tests Required

Assert flat/nested inclusive fields, explicit zero, missing fields, invalid
subsets, mixed completions, cumulative repeats, source copies, fork proof,
rollback/model boundaries, and UTC time bounds. Verify original bytes and legacy
cache rebuilds. Assert Standard USD 5.15 and Fast USD 10.30 for the synthetic
10.2M Token example, plus 272000/272001 per-request boundaries and cache-write
rates. Cover every estimate status, independent Token/USD validity, joint bucket
constraints, scan generations, and stale/history reconciliation.

Store tests must exercise concurrent writes, retention/row/byte bounds, unknown
fields, replacement failure, and native Windows create/replace private ACLs.
Unix permission tests run on Unix. All fixtures use temporary directories and
`TestCodexEnv`; preserve default parallelism and `--skip export_bindings`. Run
`just ci` for cross-crate acceptance. Real-account and native-terminal evidence
require separate receipts.

### 7. Wrong vs Correct

```rust
// Wrong: inclusive input plus cache read bills cached input twice.
catalog.calculate(model, input, output, cache_read, cache_write);

// Correct: validate the subsets and pass exclusive input.
let uncached = input.checked_sub(cache_read.checked_add(cache_write)?)?;
catalog.calculate(model, uncached, output, cache_read, cache_write);
```

## Scenario: Codex Auth Runtime And Snapshot Token Sync

### 1. Scope / Trigger

Apply this section when a change touches `CodexAuthService::switch_account`, `sync_runtime_with_saved_account*`, `CodexOAuthTokenService::plan_runtime_sync` / `repair_saved_account`, the quota fetch path for saved accounts, or `ccr codex auth off` / TUI `o`.

OpenAI OAuth refresh tokens are single-use. Every refresh rotates the token, and the old value is rejected with `refresh_token_reused`. `codex login` and `codex logout` call `/oauth/revoke` on the credentials already in the runtime file. A revoked token returns `refresh_token_invalidated`. No local copy can recover a revoked token.

### 2. Signatures

- `CodexOAuthTokenService::plan_runtime_sync(&self) -> Result<RuntimeSyncPlan>` is read-only.
- `RuntimeSyncPlan`: `NoOp`, `Unchanged`, `WriteSnapshot`, `WriteRuntime`, `SkipStaleRuntime`.
- `CodexAuthService::sync_runtime_with_saved_account(&self) -> Result<RuntimeSyncOutcome>`.
- `CodexAuthService::sync_runtime_with_saved_account_best_effort(&self, context: &str) -> RuntimeSyncOutcome` logs a warning and returns `NoOp` on error.
- `codex_quota_service::RELOGIN_REQUIRED_PREFIX` and `relogin_required_detail(error) -> Option<&str>`.

### 3. Contracts

- Identity: match OAuth runtime and saved snapshots by the complete `chatgpt_user_id::chatgpt_account_id` identity. Read the user from id/access JWT claims and prefer `tokens.account_id` for the account context. Missing or conflicting user claims cannot select an OAuth target. If aliases share one complete identity, select `current_auth` first, then the latest `last_used`, preserving insertion order on ties. Validate the selected snapshot identity before token comparison or writes. API key/provider matching retains its existing credential fingerprint rule.
- Freshness: `effective_ts = last_refresh`, else file mtime. Equal tokens (trimmed refresh/access/id/account_id) are `Unchanged` and write no payload. If the runtime is not older than the snapshot, write the snapshot after identity/freshness revalidation and a version check. If the snapshot is newer and the account is `current_auth`, replan at execution and write through crate-private `CodexRuntimeService::commit_synced_auth_versioned` (runtime_switch backup + secret versioned write), keeping other runtime keys. A conflict leaves the changed runtime intact; do not restore an old auth/config backup. If the snapshot is newer and the account is not `current_auth`, skip.
- Observation points run the best-effort sync: TUI load and reload, `switch_account` before the switch-out, quota routing for the active account, `ccr codex auth sync`, and the file-store branch of `auth off` before it deletes the runtime file.
- Non-file credential stores are `NoOp`. The keyring/auto branch of `auth off` still spawns `codex logout`, which revokes.
- Quota for the account that is `current_auth` and owns the runtime uses the runtime file as its credential source, then syncs the snapshot. Other accounts use their snapshot.
- Manual quota refresh bypasses the quota cache. Query with an unexpired access token first; refresh OAuth credentials only when the token is expired or the quota endpoint rejects authentication. A saved snapshot can remain present while the server rejects its credentials.
- Repair writes the snapshot only when the source is newer (`(refresh_changed && latest_ts >= current_ts) || latest_ts > current_ts`). The quota path retries only after a repair that updated the snapshot.
- `refresh_token_reused`, `refresh_token_invalidated`, `refresh_token_expired`, and `invalid_grant` without a newer repair source return an error prefixed with `RELOGIN_REQUIRED_PREFIX`. The account and the snapshot stay. The TUI shows "re-login required" / 「需重新登录」 with the hint to press `o` before `codex login`.
- Backups share the auth prefix pool of 10 files across labels. Do not assume `runtime_switch` backups are kept separately.
- `CodexConfigManager::backup_file` deduplicates by content. If the newest backup with the same prefix has the same bytes as the source, it refreshes that file's mtime and returns that existing path. It writes no new file and runs no cleanup. `commit_plan` rollback restores from the returned path, so a dedup hit must always return a path whose bytes equal the pre-write source. If the read or the mtime refresh fails, it creates a new backup as before.
- A new backup name is `{prefix}.{label}.{YYYYmmdd_HHMMSS}.{ext}.bak`. If that name exists (two writes in the same second), the name becomes `{prefix}.{label}.{ts}_{N}.{ext}.bak` with the first free `N >= 1`. Never overwrite an existing backup. Backup readers filter by prefix/suffix and sort by mtime, then by name; they must not parse the timestamp from the name.
- Account snapshots and their backups (`save_current`, import, `backup_account_auth`, rollback `restore_optional_backup`) and the registry file are written with `AtomicWriter::secret(true)`. Do not use `fs::copy` or `fs::write` followed by chmod for files that hold tokens.

### 4. Validation & Error Matrix

| Condition                                  | Result                                      |
| ------------------------------------------ | ------------------------------------------- |
| Runtime newer than snapshot                | `SnapshotUpdated`                           |
| Snapshot newer, account is `current_auth`  | `RuntimeUpdated`, runtime backup created or newest identical backup reused |
| Snapshot newer, account not `current_auth` | `SkippedStaleRuntime`, no write             |
| Tokens equal                               | `Unchanged`, bytes and mtime unchanged      |
| Runtime complete OAuth identity missing or not saved | `NoOp`; credential files unchanged |
| Target snapshot corrupt during switch      | error; runtime and `current_auth` unchanged |
| Permanent refresh failure, no newer source | `需重新登录：` + original error             |

### 5. Good/Base/Bad Cases

- Good: codex rotates A, CCR observes it, an external login writes B, switch back to A, quota succeeds.
- Base: codex rotates A, no observation, `codex login B` revokes A. Quota for A reports re-login; nothing is overwritten.
- Bad: refreshing quota for the active account from its snapshot. The snapshot consumes the token and the runtime keeps the consumed value.

### 6. Tests Required

- `codex_auth_service` tests for switch-out write-back, unchanged sync, both newer-snapshot branches, identity rules, duplicate `account_id`, failed switch, and the non-file store.
- `codex_oauth_token_service` test: an older repair source does not overwrite a newer snapshot.
- `codex_config` backup tests: identical content returns the existing path with a refreshed mtime (`backup_reuses_latest_identical_backup_path`); changed content in the same second creates a distinct file (`backup_creates_distinct_file_when_content_changes`, `unique_backup_path_appends_counter_on_collision`); the pool keeps 10 (`backup_retention_pool_keeps_ten_distinct_versions`).
- `codex_runtime_service` rollback test: a failed auth write restores the config from the deduplicated backup path (`commit_plan_rollback_restores_config_from_deduplicated_backup`).
- `codex_quota_service` end-to-end tests with a loopback stub (`std::net::TcpListener`) and `openai_quota_core::TEST_ENDPOINTS.scope(...)`. Use unique `account_id` values because `QUOTA_CACHE` is process-wide.
- `ccr-cli` `auth_off` sync-before-delete test and the `ccr-tui` EN/ZH re-login rendering test.

### 7. Wrong vs Correct

```rust
// Wrong: the snapshot refresh consumes the token that the runtime still holds.
let auth_path = self.account_auth_path(account_name);

// Correct: route the active account to the runtime, then sync the snapshot.
let auth_path = if self.route_active_account_to_runtime(account_name).await {
    self.current_auth_path()
} else {
    self.account_auth_path(account_name)
};
```

## Scenario: Saved Auth Backup Pools And Destructive Preconditions

### 1. Scope / Trigger

Apply to auth registry saves, explicit registry/account backups, saved-account deletion, and forced rename.

### 2. Signatures

- Crate-private `backup_auth_file(source: &Path, directory: &Path, pool: AuthBackupPool) -> Result<Option<PathBuf>>`.
- `AuthBackupPool::{Registry, Account(&str)}` selects the exact backup pool.
- `CodexRegistryStore::update_with_prepared_backup<T>(update: impl FnOnce(&mut CodexAuthRegistry) -> Result<T>) -> Result<T>` owns the locked destructive callback and final private publication.

### 3. Contracts

Keep every existing auth backup. The registry has one pool; each account has a separate exact pool. Match the full alias before the timestamp and optional numeric sequence, so foo and foo_bar cannot share a pool. Windows physical case aliases share the pool resource and matching rule; Unix names remain case-sensitive. Preserve the existing backup directory, prefixes, and extensions.

Use the shared lock root and a hashed pool resource. Under the pool lock, read the source and compare only the latest matching backup. Matching bytes reuse that path and refresh mtime. If the candidate cannot be read or touched, create another backup. Allocate a timestamp name with an unused numeric suffix and publish through a secret guarded atomic writer. Do not rotate or delete old backups. Missing sources return None; other source/backup errors propagate.

Ordinary registry save propagates prewrite backup failure. Delete and force rename first acquire the P1 source-path/known-identity operation resources. Then the registry write lock covers reread, writable-version validation, all required backups, destructive changes, and final publication. Check each locked source against its captured identity. Delete backs up registry and any existing source snapshot. Force rename backs up any existing target, source, and registry before removing either snapshot. The prepared callback does not perform a second registry backup after destruction.

Lock order is credential resources, registry lock, backup pool, then guarded writer leaf. Independent backup takes pool and leaf locks only. Read-only command preflight precedes operation-lock creation. The final file move or registry publication can fail after earlier effects; these operations do not provide a general multi-file transaction.

### 4. Validation & Error Matrix

| Condition | Result |
| --- | --- |
| Latest backup has equal bytes | Reuse existing path; no new backup |
| Same second, different preimage | Allocate distinct path; preserve each old payload |
| Existing snapshot or registry backup fails | Abort before snapshot removal/move; original bytes remain |
| Registry changes through another cooperating save | Destructive callback rereads under the same registry lock |
| Final move/publication fails | Return error; report possible partial completion and matching backup recovery |

### 5. Good/Base/Bad Cases

- Good: successful OAuth rotation completes before rename moves the source snapshot.
- Base: an explicit backup of a missing source returns None.
- Bad: ignore a target backup error, delete the target, then try to back up the source.

### 6. Tests Required

Use an existing older timestamp filename to prove public backup deduplication; same-second file counts alone cannot distinguish deduplication from overwrite. Check each preimage's bytes, similar aliases, retained old files, thread/process collisions, and Windows case contention/DACL. Inject each force-rename backup failure locally and verify source/target/registry bytes. Block the backup directory after successful prebackups to prove final publication does not repeat backup. Cover quota refresh overlapping rename, P2 read-only refusal, and recoverable delete backups. Unix permission evidence remains separate from Windows native evidence.

### 7. Wrong vs Correct

```rust
// Wrong: a failed backup does not block destruction.
let _ = self.backup_account_auth(new_name);

// Correct: complete all required backups inside the held registry callback.
self.backup_account_auth(new_name)?;
self.backup_account_auth(old_name)?;
self.backup_registry()?;
```

## Scenario: Auth Registry Compatibility And Read-Only Mode

### 1. Scope / Trigger

Apply when changing `CodexAuthRegistry`, `CodexAuthAccount`, registry persistence, or account commands that write the registry.

### 2. Signatures

- `CodexAuthRegistry::is_read_only() -> bool`.
- `CodexRegistryStore::load() -> Result<CodexAuthRegistry>` and `save(&CodexAuthRegistry) -> Result<()>`.
- `registry_read_only_message(version: &str) -> String` and `registry_read_only_version(error: &str) -> Option<&str>` in `codex_registry_store`.
- The service uses `ensure_registry_writable(&CodexAuthRegistry) -> Result<()>` before command side effects.

### 3. Contracts

Both models preserve unrecognized TOML values in flattened `extra: toml::Table`. An ordinary load/save retains unknown top-level keys, tables, and account fields. Replacing an account with `save_current --force` creates a new record and discards that record's old unknown fields. Export DTOs do not include extras.

New registries use `version = "1.0"`. Saves preserve the loaded version. Additive fields do not increase the major version. Parse the first dot-separated component as a trimmed `u32`; a major greater than `SUPPORTED_REGISTRY_MAJOR = 1`, or a parse failure, makes the registry read-only. Missing version uses the existing 1.0 default.

Save, switch, delete, description update, rename, and import must reject read-only registries before snapshot, runtime, backup, profile, or configuration side effects. In the TUI, switch preflight precedes `profile_off_for_platform`. Store save also rejects read-only values inside the registry lock before backup/write. Keep secret atomic writes and the existing error variant set.

Background registry metadata updates warn and skip read-only writes. Runtime/snapshot token synchronization remains available. List and account reads remain available. The service error starts with `REGISTRY_READ_ONLY_PREFIX`; TUI save/switch/delete/rename error toasts localize that error in EN/ZH and include the original version. Parse the prefix inside the full `CcrError` Display text.

### 4. Validation & Error Matrix

| Input | Result |
| --- | --- |
| Missing version or 1.x | Writable; loaded minor version preserved |
| Major greater than 1 or unparseable version | Readable structure remains available; account writes rejected |
| Unsupported version and incompatible structure | Parse error retains `解析注册表失败: ` and includes an upgrade hint |
| Supported version and invalid structure | Existing parse error text retained |
| Read-only background metadata update | Warning and successful return; registry bytes unchanged |

### 5. Good/Base/Bad Cases

- Good: load 1.7 with future account fields and save 1.7 with those fields intact.
- Base: list accounts from a structurally compatible 2.0 registry.
- Bad: clear a profile or write a snapshot before detecting the read-only version.

### 6. Tests Required

Test unknown key/table round trips, account replacement, version preservation, all six command rejection paths, unchanged filesystem trees, background metadata skipping, both token synchronization directions under read-only mode, and EN/ZH error version rendering. Use synthetic isolated filesystem fixtures. Earlier published CCR builds do not implement these protections.

### 7. Wrong vs Correct

```rust
// Wrong: a side effect precedes the compatibility check.
write_snapshot()?;
ensure_registry_writable(&registry)?;

// Correct: reject the command before its first side effect.
ensure_registry_writable(&registry)?;
write_snapshot()?;
```

## Scenario: Complete OAuth Identity Association

### 1. Scope / Trigger

Apply to saved-account matching, current account reconciliation, token synchronization/repair, and quota credential routing or caching.

### 2. Signatures

- Crate-private `OAuthIdentity::from_tokens(&CodexAuthTokens) -> Option<OAuthIdentity>` and `identity_from_auth(&CodexAuthJson) -> Option<OAuthIdentity>` in `services/codex_auth_identity.rs`.
- `OAuthIdentity::key() -> String`; `CodexAuthAccount.identity_key: Option<String>` is an additive registry field.
- `CodexOAuthTokenService::resolve_latest_oauth_doc(identity_key: &str)` filters repair candidates by the complete key.
- `backfill_identity_keys()` persists legacy metadata only at execution observation points. `plan_runtime_sync` and read-only account snapshots do not persist metadata.

### 3. Contracts

Read nonempty `chatgpt_user_id` from id/access JWT top-level or `https://api.openai.com/auth` claims. Conflicting user claims produce unknown identity. Account context prefers `tokens.account_id`, then supported account claims. Do not infer user identity from email or bare `sub`. JWT decoding provides local association only; the decoder does not verify signatures.

Derive each legacy account's missing key from that account's own saved snapshot. Never copy the current runtime user into another account record. Missing/corrupt snapshots retain unknown identity. A stored key conflicting with its snapshot prevents automatic association. Safe metadata backfill keeps names and current_auth unchanged; P2 read-only registries skip backfill. A synchronization NoOp leaves credential files unchanged, while a permitted execution observation can independently backfill valid snapshot metadata.

Runtime/snapshot sync, repair candidates, display matching, current_auth reconciliation, and quota runtime routing use complete identity. If an OAuth runtime lacks complete identity, reconciliation returns no proven match and preserves the existing current_auth pointer. A known unmatched identity or a logged-out runtime retains the existing pointer-clearing behavior. Recheck snapshot identity before writing tokens. Preserve freshness, runtime extra keys, backup and private atomic writes. Read-only registry metadata does not block safe token synchronization.

Quota cache keys use complete identity, with credential fingerprints when identity is unavailable. An account_id-only key cannot isolate users sharing a workspace. HTTP `ChatGPT-Account-Id`, usage ledger and quota observation schemas keep their existing account_id fields.

Quota refresh persistence verifies all expected token fields and known complete identity against the actual destination, then verifies the refreshed identity. Use a secret versioned guarded write to reject changes between read and replace. Unknown-identity same-file refresh remains available under token/version checks and cannot select another file by workspace identity. Synchronization snapshot writes recheck source freshness and retain the existing no-backup contract.

Do not add the complete key to public CLI/Tauri DTOs, exports, errors or logs. Debug hides identity values. API key/provider matching remains available without an OAuth key.

### 4. Validation & Error Matrix

| Condition | Result |
| --- | --- |
| Same workspace, different user | Separate sync/repair/cache targets |
| Missing/conflicting user identity | OAuth target association unavailable; no credential overwrite |
| Legacy record, valid own snapshot | Read-only in-memory association; permitted execution backfill |
| Registry key conflicts with own snapshot | Skip association; fixed diagnostic without identity values |
| Duplicate complete identity | current_auth, then latest last_used, then insertion order |

### 5. Good/Base/Bad Cases

- Good: user A runtime synchronizes only with user A snapshots in a shared workspace.
- Base: an API key account retains its existing fingerprint identity.
- Bad: use account_id alone for repair or cached quota after fixing only the sync planner.

### 6. Tests Required

Cover top-level/namespaced claims, fallback sources, empty/bad/conflicting user claims, same-workspace user isolation in both sync directions and repair, own-snapshot legacy backfill, read-only planners and P2 read-only registries, alias precedence, quota routing/cache isolation, CLI auth-off rotation, and secret/key output boundaries. Synthetic OAuth success fixtures need complete user claims; keep explicit incomplete-identity NoOp cases.

### 7. Wrong vs Correct

```rust
// Wrong: workspace identity alone selects a user's credentials.
account.account_id == runtime_account_id

// Correct: compare complete identities derived from validated local sources.
saved_identity.as_ref() == Some(&runtime_identity)
```

## Scenario: Credential Operations Across Queries And Saved-Account Switches

### 1. Scope / Trigger

Apply to quota credential selection and refresh, saved-account save/switch, and OAuth synchronization. Delete/rename/import must join the same lock protocol when those operations replace or remove credential files.

### 2. Signatures

- Crate-private `CredentialResource::{path, from_tokens, from_path}` derives hashed source resources.
- `CredentialLocks::acquire_sources` and `acquire_async_sources` hold sorted, deduplicated source-path and known-identity resources.
- `CredentialLocks::verify_path` compares each source with that source's captured identity resource.
- Internal `_locked` helpers run under caller-held operation locks. `prepare_current_quota_locked` prepares current quota credentials through the existing versioned auth commit.

### 3. Contracts

Use `LockManager::with_default_path` and the shared `CCR_LOCK_DIR`. Resource names contain a stable hash, with no credential or complete identity value. Every source keeps a stable path resource across missing-file creation and identity completion. Known complete identities add a shared resource across runtime and saved aliases. Unknown identities permit no cross-file association. Async acquisition uses `spawn_blocking`.

Acquire operation resources in sorted order before registry, backup-pool, or guarded-write leaf locks. Keep the operation guard through source reread, quota HTTP, refresh persistence, and required snapshot synchronization. Saved-account switch holds outgoing and target resources through the final runtime commit. Recheck each path against the captured source; membership in the complete lock set does not prove that a target still contains the selected account. Reject P2 read-only save/switch before creating operation locks.

Under a known identity lock, quota selection can reuse a newer registered alias with the same complete identity. Preserve the existing last_refresh-first, mtime-fallback freshness rule. Compare full timestamp precision and retain fractional last_refresh values during writeback. Keep public runtime sync outcomes unchanged; quota preparation remains internal.

When same-file refresh first supplies a complete identity, persist the refreshed file and return a private control result before quota GET, another refresh, or cross-file synchronization. Release the old guard and retry at most once under the new source/identity lock set. Keep the newly persisted complete identity as the retry expectation. A replacement with another identity rejects the original request. Do not expose the control result through errors, logs, or DTOs.

Save snapshot bytes and metadata from one captured runtime document. Existing content-version and identity guards remain required because external Codex/login writers do not use CCR locks. These locks do not serialize arbitrary external writers or the complete platform-profile/auth-off replacement intervals.

### 4. Validation & Error Matrix

| Condition | Result |
| --- | --- |
| Concurrent queries of one known token chain | Waiting query rereads persisted tokens; one submission of each consumed refresh token |
| Same workspace, different users | Separate identity resources and credential files |
| Source changes identity while waiting | Fixed conflict before quota HTTP |
| Unknown source gains identity | One bounded retry; no alias work under the old path-only guard |
| Lock acquisition fails | Explicit error; no token POST or credential write |
| External writer changes content | Existing identity/version guards reject unsafe persistence; external operations remain outside lock coverage |

### 5. Good/Base/Bad Cases

- Good: a saved-account switch waits for outgoing refresh persistence before replacing runtime.
- Base: an incomplete OAuth source can refresh the same file without selecting another account by workspace.
- Bad: serialize HTTP but reread an unchanged old alias and submit the consumed token again.

### 6. Tests Required

Cover one-source concurrency, alias reuse, shared-workspace user independence, outgoing refresh/switch overlap, lock-wait source drift, save metadata from captured bytes, same-second rotation, and unknown-to-known identity completion with alias overlap and retry-source replacement. A real child process must first prove OS lock contention, then acquire after release and reread the changed source. Keep the child lock/reread evidence separate from same-process loopback HTTP evidence. Synthetic checks do not establish real-account authentication recovery.

### 7. Wrong vs Correct

```rust
// Wrong: the target changed to the outgoing account, which is also locked.
held_resources.contains(&CredentialResource::from_path(&target))

// Correct: verify the target against the target's captured source resource.
held.verify_path(&target)?;
```

## Scenario: Auth Permission And Diagnostic Boundaries

### 1. Scope / Trigger

Use when observing unchanged OAuth files, moving snapshots, formatting auth Debug values, or reporting quota/refresh failures.

### 2. Signatures

- `utils::ensure_private_permissions(path: &Path) -> Result<()>`.
- Execution uses `enforce_owner_only_permissions_versioned` with raw file bytes and their content token.
- `OpenAiQuotaCore::extract_error_code(body: &str) -> Option<&'static str>` is private and returns a fixed allowed code.

### 3. Contracts

- Permission failures propagate or receive explicit handling. Do not invoke USERNAME/icacls or ignore a failed required permission operation.
- `plan_runtime_sync` stays read-only. Unchanged execution revalidates the selected complete identity and token pair, then hardens the observed runtime and snapshot through versioned metadata operations.
- Preserve credential bytes, mtime, and file identity during metadata hardening. A runtime DACL can already be stricter when the later snapshot operation conflicts or fails; the pair has no ACL transaction.
- For an existing broad target, harden its observed version before a private CAS write. Do not rely on the old secret writer to narrow an existing Windows DACL.
- Rename fallback publishes the private atomic target before removing the source. Publication errors can leave a complete target visible. Native rename success followed by permission failure can already have moved the file; retain the P3 backup recovery boundary.
- Auth JSON/tokens, raw auth maps, registry/account raw TOML extension fields, identity keys, refresh request/response tokens, runtime plan config and provider secrets must have redacted Debug. Keep persistence serialization unchanged.
- HTTP errors expose status and only `token_invalidated`, `refresh_token_reused`, `refresh_token_invalidated`, `refresh_token_expired`, or `invalid_grant`. Do not expose arbitrary code, message or response-body text.
- Map the internally recognized fixed token-invalidated phrase to `token_invalidated` so existing refresh classification remains. Preserve the fixed relogin marker. Forced manual quota bypasses cache; valid access tokens retain the P1 refresh behavior.

### 4. Validation & Error Matrix

| Condition | Result |
| --- | --- |
| Observed content/identity changes | Safe skip or fixed retry error; no stale credential overwrite |
| Required permission operation fails | Existing error; report any earlier operation effects |
| Fallback publication fails before replacement | Source and previous target bytes remain |
| Unknown HTTP code/body, Unicode or long message | Status only; no response content |
| Known refresh code or fixed invalidation phrase | Fixed code retained for refresh/repair/relogin classification |

### 5. Good/Base/Bad Cases

- Good: use the original raw bytes for permission version checks, then retain the original Unchanged result when the pair still matches.
- Base: API key and incomplete-identity matching retain their existing contracts.
- Bad: harden permissions in a planner, copy payload before setting privacy, or format a raw auth map in Debug.

### 6. Tests Required

Cover both Unchanged execution entry points, quota-only preparation, raw-byte conflict, native Windows wide-DACL hardening, deterministic rename fallback and publication-before-failure source preservation. Verify Debug and HTTP errors with synthetic secret markers. Preserve known force/repair/relogin behavior and disk round trips. TestBackend checks must cover EN/ZH relogin status and error colors at the formal six sizes plus compact degradation. Separate those results from interactive or real-account evidence.

### 7. Wrong vs Correct

```rust
// Wrong: response content can contain credentials or personal data.
message.push_str(&body);

// Correct: emit only a fixed code from the private allowlist.
if let Some(code) = Self::extract_error_code(&body) {
    message.push_str(&format!(" [{code}]"));
}
```

## Scenario: Auth Import Identity And Credential Admission

### 1. Scope / Trigger

Apply to plaintext and encrypted saved-account import. Preserve the existing export envelope, ImportResult fields and CLI encrypted-export rule.

### 2. Signatures

- `CodexAuthService::import_accounts(content: &str, mode: ImportMode, force: bool) -> Result<ImportResult>`.
- `import_accounts_encrypted` decrypts and calls the same import method.
- Crate-private `token_account_id` retains the P1 account-source priority. `has_conflicting_claims` detects contradictory known user/account values for import.
- `CredentialLocks::acquire_sources_with_resources` adds incoming identity resources to the existing sorted source/path lock set.

### 3. Contracts

Deserialize the entire typed CodexAuthExport before selecting pending entries. A malformed typed entry remains a parse failure even when its name would be skipped. Parse diagnostics contain line/column, without input values. Keep legacy provider skips and Merge/no-force existing-account skips; skipped entries do not undergo credential-identity validation.

Validate every pending name and credential before creating directories, backups or operation locks. Known OAuth account context must exactly match account_id metadata. Reject contradictory known user/account claims and an incompatible auth_method. API-key metadata must match the existing fingerprint and API method rules. Reject extra metadata whitespace when identity is known. Missing claims and metadata-only entries retain their compatibility behavior; do not infer identity from name/email. Complete keys come from tokens and remain outside public export DTOs/errors.

Capture each target's original bytes/version and source resource before admission. Lock stable paths, original identities and incoming complete identities together. After acquisition, re-read the registry, reject read-only state, and recheck each captured source, raw content version and record presence. A token rotation with the same identity also requires retry. Hold operation locks through credential replacement and registry save. Force/Replace can intentionally change identity; the incoming identity protects its other registered aliases.

For auth_data, retain the target until secret versioned atomic publication succeeds. Harden an existing target using the captured expected version. A permission/version conflict returns the fixed retry error. Metadata-only force retains the existing backup-then-delete behavior. Replace covers matching names and preserves unrelated records; Replace/no-force metadata-only retains the previous snapshot. Only force of an existing account creates the existing snapshot preimage; Replace/no-force has no new snapshot-backup guarantee. Ordinary registry save retains its backup contract.

Input rejection has no file/directory side effects. Later lock/I/O failures can leave lock files, backups, permission changes or earlier imported files. There is no whole-bundle I/O transaction, general registry read-modify-write transaction or external-writer serialization.

### 4. Validation & Error Matrix

| Condition | Result |
| --- | --- |
| Pending metadata/token/claim or method conflict | Fixed ValidationError before any write or lock creation |
| Later entry invalid | Entire pending input rejected; no earlier entry written |
| Existing entry, Merge/no-force | Skip identity validation; preserve typed parse rejection |
| Source/version/presence changes while waiting | Fixed retry error; preserve changed credential |
| Registry becomes read-only during admission | Reject before credential write |
| Missing complete identity or metadata-only | Preserve compatibility; no claimed token validation for missing auth_data |
| Encrypted input conflict | Same preflight rejection after decryption |

### 5. Good/Base/Bad Cases

- Good: an invalid second account leaves the valid first account and all backup directories unchanged.
- Base: a metadata-only force import backs up and removes the old credential under the existing semantics.
- Bad: delete the previous auth_data before the replacement writer succeeds, or compare only identity after a same-user token rotation.

### 6. Tests Required

Cover exact and conflicting metadata, top-level/namespaced claims, contradictory known users, method/API fingerprint mismatches, extra whitespace, missing claims and metadata-only input. Preserve Merge/force/Replace counts and unrelated records. Assert full tree equality after a valid-first/invalid-second failure and encrypted conflict. Verify old/incoming identity contention, lock-wait raw-version change, read-only admission change and force-backup failure. CLI plaintext helper must return the service Err. Keep synthetic loopback/file evidence separate from real-account recovery.

### 7. Wrong vs Correct

```rust
// Wrong: removing the old credential creates a failure window.
fs::remove_file(&auth_path)?;
writer.write_bytes(&payload)?;

// Correct: publish against the captured original version without pre-delete.
write_guarded_versioned(&auth_path, &payload, &expected_version, &secret_options)?;
```

## Verification

For Codex/OpenCode domain changes, run:

- `just fmt-check`
- `cargo test -p ccr-codex --all-features -- --skip export_bindings`
- Relevant `cargo test -p ccr --test commands -- --skip export_bindings` when CLI surfaces change
- `just lint-strict`
- `just ci` for cross-crate final acceptance
