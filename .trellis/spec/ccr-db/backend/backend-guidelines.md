# ccr-db Backend Guidelines

> Desktop/check-in/usage SQLite storage and data services.

## Scope

`crates/ccr-db` owns the desktop database layer: SQLite connection pools, schema/migrations, repositories, check-in models, usage import, and monitoring log persistence. It is independent of Axum/Tauri response types.

Reference files:

- `crates/ccr-db/src/lib.rs`
- `crates/ccr-db/src/database/mod.rs`
- `crates/ccr-db/src/core/error.rs`
- `crates/ccr-db/src/services/usage_import_service.rs`

## Structure

Keep the existing storage split:

- `database/` for pool setup, schema, migrations, and repositories.
- `models/` for check-in, platform, usage, and monitoring data.
- `services/` for import, conversion, and log persistence.
- `core/` for database/migration errors.

Do not add UI command handlers or HTTP response conversion in this crate.

## Database Patterns

Use `rusqlite` and `r2d2_sqlite` through `DbPool`. Pool construction and PRAGMAs (`WAL`, `busy_timeout`, `foreign_keys`, ...) are owned by `ccr_core::core::sqlite` — the single SQLite seam for the whole workspace; `ccr_db::database` re-exports `DbPool`/`DbConnection`/`PoolConfig` from it and adds the `DbError` boundary. Do not add another pool factory or a forwarding `pool` submodule.

The Tauri app initializes `~/.ccr-ui/ccr-ui.db` through `initialize_app_pool()`: one pool (max_size=8) is created, migrations run once, and the SAME instance is registered as `GLOBAL_POOL` and returned for `AppState.db_pool`. Never open a second pool on `ccr-ui.db`. Usage archive uses a separate pool via `create_usage_archive_pool()`.

Repository/manager code reaches the database either through the global-path free functions `with_connection`/`transaction`, or through an injected `DbAccess` handle (`Global` | `Pool(DbPool)`, same `DbError` surface). Managers that need unit tests should take `DbAccess` (constructor keeps a `Global` default so callers don't change); see `ccr-checkin` `AccountManager::with_db` for the pilot pattern.

Usage archive storage lives under `~/.ccr/analytics/usage.db`, honoring `CCR_DATA_DIR`/`CCR_ROOT`. Desktop UI state uses `~/.ccr-ui/ccr-ui.db`.

For read-only upstream usage databases, open with `SQLITE_OPEN_READ_ONLY` as `UsageImportService` does. Do not migrate or mutate upstream tool databases.

### Convention: Additive column migrations

**What**: Adding a column to an existing table requires updating both `schema.rs` `CREATE_TABLES_SQL` (so fresh databases get the column directly) and a guarded migration in `migrations.rs` (`is_migration_applied` + `table_has_column` guard around `ALTER TABLE ... ADD COLUMN`, so existing databases upgrade idempotently).

**Why**: Either half alone diverges fresh-install schema from upgraded schema; the `table_has_column` guard keeps the migration safe to re-run and safe against fresh databases that already carry the column.

**Example**: migration v15 `checkin_providers_builtin_id` adds `checkin_providers.builtin_id TEXT NULL` — a nullable link to the providers-catalog entry id. Old rows stay NULL (consumers fall back to name matching); `set_provider_builtin_id_if_missing` writes only NULL rows and never overwrites an existing value.

## Decision Record: SQLite Seam Shape (2026-07-05, 07-03-arch-sqlite-seam)

Evidence: `.trellis/tasks/archive/2026-07/07-03-arch-sqlite-seam/research/veto-research.md`.

- **Three database files stay separate — do not propose merging.** `~/.ccr/data.db` (CLI process, ccr-store) vs `~/.ccr-ui/ccr-ui.db` (desktop process) vs `~/.ccr/analytics/usage.db` (durable archive, deliberately split out of ccr-ui.db in aa5af6c1). Different processes, roots, and lifecycles.
- **The two migration runners stay separate.** ccr-store uses a name-based `migrations(id, name, applied_at)` table; ccr-db uses a version-based `migrations(version, name, applied_at)` table — same table name, different schema, both with published history. Merging runners would rewrite published migration semantics for no benefit.
- **No cross-stack seam crate.** CLI-side crates (ccr/ccr-cli → ccr-store) do not depend on ccr-db; a shared seam inside ccr-db would drag migrations + check-in models into every CLI build. The seam is `ccr_core::core::sqlite`, which both stacks already consume.
- Error direction follows the CcrError Freeze ADR: the seam speaks primitives, ccr-db wraps into `DbError`, ccr-store wraps into `CcrError` at its own boundary; no `impl From<DbError> for CcrError`.

## Error Handling

Use `DbError` and `MigrationError`. Convert `rusqlite`, IO, and serialization errors at the database boundary and include operation context. (`ExecutorError` and `core/executor.rs` were deleted in 07-03-arch-sqlite-seam: zero callers workspace-wide, a leftover from the pre-Tauri web backend.)

Do not use catch-all strings when the caller needs to distinguish pool, migration, query, or serialization failures.

## Logging

Use `tracing::info!` for database initialization and import summaries; `debug!`/`warn!` for import details and skipped records. Avoid logging raw request/response bodies or sensitive account fields.

## Testing

Use `database::initialize_for_test()` for in-memory SQLite tests and local mutex guards where global database state is involved. Use `TempDir` for imported usage files and fixture databases.

## Verification

For database changes, run:

- `just fmt-check`
- `cargo test -p ccr-db -- --test-threads=1`
- `just lint-strict`

Run UI/Tauri checks as well when `ccr-ui/src-tauri` consumers are affected.
