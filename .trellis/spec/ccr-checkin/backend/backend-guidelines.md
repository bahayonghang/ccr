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

## Testing

Use `TempDir` plus `database::initialize_for_test()` for service tests. Prefer deterministic timestamps for calendar/stat logic, following existing `checkin_service.rs` tests.

## Verification

For check-in changes, run:

- `just fmt-check`
- `cargo test -p ccr-checkin -- --test-threads=1`
- `cargo test -p ccr-db -- --test-threads=1` when shared database models or repositories change
- `just lint-strict`
