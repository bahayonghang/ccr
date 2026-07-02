# Implement — C2 llmusage provider ingest + adapter provider dimension

Precondition is satisfied: local llmusage has schema 14 and `--provider-map`.

## Steps

1. Adapter filter and DTO:
   - Add `provider: Option<String>` to `QueryFilter`.
   - Update `build_filter` and SQL predicate construction to support provider
     filtering.
   - Add `ProviderBreakdownDto` in `queries.rs`.

2. Capability gate:
   - Add `FeatureKey::ProviderBreakdown`.
   - Require schema `>= 14` and `provider_label` on both `usage_event` and
     `usage_bucket_30m`.
   - Preserve the existing unsupported/degrade path for old DBs.

3. Read-only projection:
   - Add `Dashboard::provider_breakdown(&QueryFilter)`.
   - Group `usage_bucket_30m` by `provider_label`.
   - Return `provider = null` for empty provider labels.
   - Include requests, token splits, and cost totals.

4. Sync wiring:
   - Add `provider_map: Option<PathBuf>` to `SyncCommandOptions`.
   - Append `--provider-map <path>` when present.
   - In usage import paths, pass `$CCR_ROOT/analytics/provider_activation.jsonl`.

5. Tauri command surface:
   - Add `get_usage_by_provider_v2(platform?, start?, end?)`.
   - Add an optional `provider` parameter to `get_usage_dashboard_v2`.
   - Register the new command.

6. Tests:
   - Keep `tests/llmusage_no_crate_guard.rs` green.
   - Add schema-14 fixture coverage for provider breakdown and provider filter.
   - Add schema <14 / missing-column degrade coverage for `ProviderBreakdown`.
   - Add sync CLI argument coverage for `--provider-map`.

## Validation

- `cd ccr-ui/src-tauri && cargo check`
- `cd ccr-ui/src-tauri && cargo test llmusage`
- `just lint-strict`
- `just test`
- `git diff --check`

## Rollback

Remove the provider filter, provider breakdown command, and `--provider-map`
wiring. Existing activation logs and llmusage schema 14 data remain harmless.
