# ccr-checkin Backend Guidelines

> Check-in business facade over ccr-db storage and provider services.

## Scope

`crates/ccr-checkin` owns check-in orchestration, CDK redemption, encrypted check-in credentials, and a facade over `ccr-db` check-in models/repositories.

Reference files:

- `crates/ccr-checkin/src/lib.rs`
- `crates/ccr-checkin/src/core/error.rs`
- `crates/ccr-checkin/src/core/crypto.rs`
- `crates/ccr-checkin/src/services/checkin_service.rs`
- `crates/ccr-checkin/src/services/cdk_service.rs`

## Structure

Keep the split:

- `core/` for crypto and check-in-specific errors.
- `managers/checkin/` for account and record management.
- `services/` for network orchestration and dashboard/stat computations.
- `models::checkin` re-exports come from `ccr-db`.

Do not duplicate check-in database models in this crate; use `ccr_db::models::checkin`.

## Database And Crypto Boundaries

Database initialization is owned by `ccr-db`; tests call `database::initialize_for_test()`. Check-in account secrets must pass through `CryptoManager` and should never be logged or stored in plaintext outside intentional encrypted fields.

Network services use `reqwest` clients and proxy detection. Preserve explicit WAF/Cloudflare/cookie-expired error classification in `CheckinServiceError::error_code()`.

## Error Handling

Use `CheckinServiceError` for provider/account/crypto/network/API/record/balance/database failures. Convert database failures through `Database(#[from] DbError)`.

Do not use `unwrap`/`expect` in production check-in flows. Missing cookies, tokens, balances, or DB rows should become typed service errors or absent optional values.

## Logging

Use `tracing` for provider progress, proxy mode, balance/check-in milestones, and failures. Never log cookies, bearer tokens, encrypted payload plaintext, or full auth headers.

## Scenario: WAF Cookie Recovery Contracts

### 1. Scope / Trigger

- Trigger: changing WAF recovery, Tauri command payloads, cached WAF cookie validation, or provider WAF policy.
- Scope: `WafCookieManager`, `CheckinService::validate_cached_waf_access`, and the Tauri WAF commands that bridge desktop WebView recovery to the Vue check-in flow.
- Reason: the user-facing recovery flow spans provider policy, WebView cookie extraction, encrypted account cookies, cache writes, validation requests, and frontend retry state.

### 2. Signatures

- `WafCookieManager::policy_for_provider(&CheckinProvider) -> Option<WafCookiePolicy>`
- `WafCookieManager::select_required_cookies(&HashMap<String, String>, &[String]) -> WafCookieSelection`
- `CheckinService::validate_cached_waf_access(&self, account_id: &str) -> Result<WafCookieValidationResult>`
- Tauri `open_waf_login(login_url: String, provider_id: String) -> Result<WafCookieRecoveryResult, String>`
- Tauri `validate_waf_cookie_for_account(account_id: String) -> Result<WafCookieValidationResult, String>`

### 3. Contracts

- AnyRouter WAF recovery requires `acw_tc`, `cdn_sec_tc`, and `acw_sc__v2`; keep these names in backend policy, not duplicated as frontend-only constants.
- `open_waf_login` may read cookie values from the WebView runtime cookie store and `document.cookie`, but its response must expose only cookie names, missing names, status, source, provider id/name, and a diagnostic message.
- Cache writes are allowed only after `WafCookieSelection::is_complete()` is true for the provider policy.
- Retry must be gated by `validate_cached_waf_access` using merged account cookies plus cached WAF cookies, and the validation method must not update balance history or account timestamps.
- Validation URL selection must prefer `WafCookiePolicy.validation_path` and fall back to the provider's `user_info_path`.

### 4. Validation & Error Matrix

- Required cookie missing -> return `persisted=false` with `missing_cookie_names`; do not cache partial cookies.
- Cookie cache absent -> validation returns `success=false`, `challenge="none"`, and a cache-missing message.
- Validation response is WAF HTML -> `success=false`, `challenge="waf"`, and no retry.
- Validation response is Cloudflare challenge -> `success=false`, `challenge="cf"`, and no retry.
- Validation HTTP status is not success -> `success=false` with the status code and no cookie or auth values.

### 5. Good/Base/Bad Cases

- Good: all required AnyRouter cookie names are present from the WebView store, cached once, validation passes, then only previously WAF-blocked accounts are retried.
- Base: `document.cookie` reports a non-empty value but `acw_sc__v2` is missing; the UI shows the missing name and stops before retry.
- Bad: caching any non-empty cookie string or retrying after validation still returns an HTML/WAF page.

### 6. Tests Required

- Unit test provider policy matching for built-in AnyRouter and `anyrouter.top`.
- Unit test cookie parsing and required-cookie selection, asserting only names are surfaced and unrelated cookie values are not selected.
- Frontend smoke test missing-cookie diagnostics and retry gating after validation.
- Run `cargo test -p ccr-checkin -- --test-threads=1`, `bun run type-check`, `bun run test`, and `bun run tauri:check` for cross-layer changes.

### 7. Wrong vs Correct

#### Wrong

```rust
// Any non-empty cookie is not proof that the provider WAF challenge is solved.
if !cookie_str.trim().is_empty() {
    waf_cookie_manager.save(provider_id, parse_cookie_pairs(cookie_str))?;
}
```

#### Correct

```rust
let selection = WafCookieManager::select_required_cookies(&cookies, &policy.required_cookie_names);
if selection.is_complete() {
    waf_cookie_manager.save(provider_id, selection.cookies)?;
}
```

## Testing

Use `TempDir` plus `database::initialize_for_test()` for service tests. Prefer deterministic timestamps for calendar/stat logic, following existing `checkin_service.rs` tests.

## Verification

For check-in changes, run:

- `just fmt-check`
- `cargo test -p ccr-checkin -- --test-threads=1`
- `cargo test -p ccr-db -- --test-threads=1` when shared database models or repositories change
- `just lint-strict`
