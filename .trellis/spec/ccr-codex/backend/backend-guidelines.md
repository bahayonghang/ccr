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

- Identity: match the runtime to a saved account by `account_id` only (from `tokens.account_id`, else the access-token JWT). If several accounts share the `account_id`, select `current_auth` first, then the latest `last_used`. A missing or unknown `account_id` is `NoOp`.
- Freshness: `effective_ts = last_refresh`, else file mtime. Equal tokens (trimmed refresh/access/id/account_id) are `Unchanged` and write nothing. If the runtime is not older than the snapshot, write the snapshot. If the snapshot is newer and the account is `current_auth`, write the runtime through `CodexRuntimeService::commit_plan` (backup + atomic write) and keep the other runtime keys. If the snapshot is newer and the account is not `current_auth`, skip.
- Observation points run the best-effort sync: TUI load and reload, `switch_account` before the switch-out, quota routing for the active account, `ccr codex auth sync`, and the file-store branch of `auth off` before it deletes the runtime file.
- Non-file credential stores are `NoOp`. The keyring/auto branch of `auth off` still spawns `codex logout`, which revokes.
- Quota for the account that is `current_auth` and owns the runtime uses the runtime file as its credential source, then syncs the snapshot. Other accounts use their snapshot.
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
| Runtime `account_id` missing or not saved  | `NoOp`                                      |
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

## Verification

For Codex/OpenCode domain changes, run:

- `just fmt-check`
- `cargo test -p ccr-codex --all-features -- --skip export_bindings`
- Relevant `cargo test -p ccr --test commands -- --skip export_bindings` when CLI surfaces change
- `just lint-strict`
- `just ci` for cross-crate final acceptance
