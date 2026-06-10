# ccr-store Backend Guidelines

> CLI-side history, cost, budget, pricing, and session persistence.

## Scope

`crates/ccr-store` owns CLI-side persistent state: history records, budget/cost/pricing managers, session indexing, and the session SQLite store. Keep UI/desktop-specific SQLite concerns in `ccr-db`.

Reference files:

- `crates/ccr-store/src/lib.rs`
- `crates/ccr-store/src/storage/database.rs`
- `crates/ccr-store/src/storage/session_store.rs`
- `crates/ccr-store/src/sessions/indexer.rs`

## Structure

Use the current split:

- Top-level managers for history, pricing, budget, and cost tracking.
- `models/` for persisted data contracts.
- `sessions/` for session parsing/indexing.
- `storage/` for SQLite connection and query code.

Do not put command presentation logic or TUI rendering into this crate.

## Database Patterns

Session persistence uses `rusqlite` and `r2d2_sqlite`. Batch writes should use transactions, as `SessionStore::upsert_sessions` does. Dynamic `IN` queries should chunk inputs, as `get_file_hashes` does with chunks of 500.

Map `rusqlite` failures to `CcrError::DatabaseError` with operation context. Do not expose raw SQL errors without saying which operation failed.

## Logging

Use `tracing::debug!` for per-record failures and `tracing::info!` for completed indexing/import counts. Avoid logging full prompt/session content; log ids, paths, counts, and summaries instead.

## Testing

Use `tempfile` for SQLite and filesystem fixtures. Tests that touch process-wide env should be serialized through the relevant crate fixture. Run Rust tests serially when invoking broader workspace tests.

## Verification

For store changes, run:

- `just fmt-check`
- `cargo test -p ccr-store -- --test-threads=1`
- `just lint-strict`
