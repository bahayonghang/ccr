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

In existing store paths, map `rusqlite` failures to `CcrError::DatabaseError` with operation context. Do not expose raw SQL errors without saying which operation failed. Shared SQLite seam code speaks `ccr-db`'s `DbError` and bridges with `map_err` at the ccr-store boundary — do not add `From<DbError>` impls or new `CcrError` variants (see `../../ccr-core/backend/ccr-error-freeze.md`). A domain projection that needs typed consumer branching may use a local self-owned error under that ADR: [Recorded Profile Activity](./profile-activity.md) uses fixed safe categories and adds no ccr-db dependency.

## Logging

Use `tracing::debug!` for per-record failures and `tracing::info!` for completed indexing/import counts. Avoid logging full prompt/session content; log ids, paths, counts, and summaries instead.

## Testing

Use `tempfile` for SQLite and filesystem fixtures. Prefer explicit paths over process-wide environment changes. Preserve root/desktop default test parallelism and `--skip export_bindings`; the separate binding-generation gate owns export serialization.

## Verification

For store changes, run:

- `just fmt-check`
- `cargo test -p ccr-store --locked -- --skip export_bindings`
- `just lint-strict`

## Budget/pricing extension preservation

1. **Scope:** `BudgetManager` and `PricingManager` saves consumed by CLI/Tauri/GPUI. Public typed config/projection shapes stay unchanged; unknown persisted TOML is not added to wire results.
2. **Signatures:** existing manager setters and `save_config` remain the write owner. `ccr_app::usage::policies::update_budget(PathBuf, Value)` validates the entire supplied patch before calling any setter.
3. **Contracts:** managers retain private original TOML, replace known top-level keys and keep unknown extensions. Pricing retains extension fields of surviving model/default entries, while explicitly removed models/default pricing stay removed. Update private original only after successful existing atomic write. Budget valid-field writes retain legacy sequential persistence; this is not a multi-field transaction or CAS guarantee.
4. **Errors:** invalid enabled type, negative/non-number limit, or non-integer/out-of-range `warnAtPercent` rejects before any setter and leaves original bytes unchanged. Never narrow300 to44 with `as u8`. A failed save leaves the private original unchanged; it must not cause a later save to resurrect deliberately removed entries.
5. **Cases:** good: update daily limit while retaining an unknown nested owner extension; base: absent optional budget limit remains absent; bad: reserialize only the typed config and silently discard extensions, or remerge deleted models from an outdated original.
6. **Tests:** `budget_mutation_preserves_unknown_toml_extensions`, `pricing_mutations_preserve_unknown_top_level_and_untouched_model_extensions`, and shared all-input validation fixtures. Check repeated successful mutations, nested surviving price fields, deliberate remove/reset, failure byte preservation and no public DTO expansion.
7. **Wrong/correct:** wrong: deserialize typed config → serialize typed config as whole file. Correct: preserve original document inside the existing manager → replace intentional known fields → existing guarded atomic write → advance private original on success.
