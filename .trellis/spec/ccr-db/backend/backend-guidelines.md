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
- `core/` for database/executor errors and command execution helpers.

Do not add UI command handlers or HTTP response conversion in this crate.

## Database Patterns

Use `rusqlite` and `r2d2_sqlite` through `DbPool`. Initialize app pools through `create_app_pool()` or `create_usage_archive_pool()`, which run migrations. Use `with_connection`/`transaction` for global-pool operations.

Usage archive storage lives under `~/.ccr/analytics/usage.db`, honoring `CCR_DATA_DIR`/`CCR_ROOT`. Desktop UI state uses `~/.ccr-ui/ccr-ui.db`.

For read-only upstream usage databases, open with `SQLITE_OPEN_READ_ONLY` as `UsageImportService` does. Do not migrate or mutate upstream tool databases.

## Error Handling

Use `DbError`, `MigrationError`, and `ExecutorError`. Convert `rusqlite`, IO, and serialization errors at the database boundary and include operation context.

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
