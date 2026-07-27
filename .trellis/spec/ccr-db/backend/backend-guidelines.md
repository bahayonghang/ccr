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

## Scenario: Transactional migrations and v3 usage repair

### 1. Scope / Trigger

- Trigger: changing ccr-db schema migrations, data backfills, migration
  postconditions, or history markers.
- Published migration numbers are immutable. Historical v3 backfill gaps are
  repaired by v16; never rewrite an existing v3 marker in place.

### 2. Signatures

- `apply_migration(conn, version, name, migrate, validate_postconditions, marker_name)`
- `run_migration_v16(conn: &Connection) -> MigrationResult<()>`
- `migration_rejections(version, row_id, error_code, created_at)` with primary
  key `(version, row_id)`.

### 3. Contracts

- One SQLite transaction contains schema/data work, postcondition validation,
  count-bearing marker insertion, and commit.
- v16 processes every row whose extracted usage fields are missing. Each
  candidate ends as repaired or as one coded rejection; raw JSON is never
  stored in rejection details.
- Row decode and UPDATE errors propagate and roll back rejections, data, schema,
  and marker. Malformed business JSON is the only expected row rejection.
- A real file database with repair candidates is copied with `VACUUM INTO`
  before v16; the backup must pass `PRAGMA integrity_check`. In-memory databases
  skip this file-only preflight.

### 4. Validation & Error Matrix

- Row decode or UPDATE fails -> transaction rolls back; no marker.
- Malformed `record_json` -> `malformed_record_json` rejection; no raw payload.
- `processed != repaired + rejected` -> fail postcondition and roll back.
- Remaining inconsistent rows do not equal rejection rows -> fail and roll
  back.
- Backup creation/open/integrity check fails -> do not begin v16 mutation.

### 5. Good/Base/Bad Cases

- Good: one valid and one malformed candidate produce
  `repair_usage_v3_backfill[processed=2,repaired=1,rejected=1]`.
- Base: no candidates records a zero-count marker without a backup.
- Bad: `.filter_map(|row| row.ok())`, ignored UPDATE results, or inserting the
  marker outside the data transaction.

### 6. Tests Required

- Inject a BLOB row id and a failing UPDATE trigger; assert no marker or partial
  accounting, then prove retry succeeds.
- Fail after transactional DDL and assert both table and marker are absent.
- Upgrade a file-backed historical-v3 fixture twice; assert exactly one
  pre-migration backup, backup `integrity_check = ok`, v16 idempotence, and
  stable repaired/rejected counts.
- Run `cargo test -p ccr-db migration -- --test-threads=1`, full ccr-db tests,
  `just lint-strict`, and `just test`.

### 7. Wrong vs Correct

#### Wrong

```rust
for row in rows.filter_map(|row| row.ok()) {
    let _ = update.execute(params![row]);
}
conn.execute(INSERT_MIGRATION_SQL, params![version, name, now])?;
```

#### Correct

```rust
apply_migration(conn, version, name, migrate, validate_postconditions, marker_name)?;
```

Success means committed schema/data plus verified postconditions and a marker,
not merely that a backfill loop returned.

## Scenario: UTF-8 migration source comments

### 1. Scope / Trigger

- Trigger: editing comments or string-adjacent documentation in
  `crates/ccr-db/src/database/migrations.rs`.

### 2. Signatures

- Source encoding: UTF-8.
- Corruption search:
  `rg -n '鍔|浠|璇|鐨|鏁|鏍|锛|鈥|�' crates/ccr-db/src/database/migrations.rs`.

### 3. Contracts

- Preserve migration code and published SQL while repairing mojibake comments.
- Reconstruct comment meaning from the adjacent operation; do not guess data or
  change executable strings as part of an encoding-only repair.

### 4. Validation & Error Matrix

- Known corruption marker remains -> acceptance fails.
- Executable SQL or migration behavior changes in an encoding-only diff ->
  split and review as a migration change.

### 5. Good/Base/Bad Cases

- Good: replace a corrupted comment above the pricing query with
  `// 加载定价表`.
- Base: leave valid English or Chinese comments unchanged.
- Bad: re-encode the whole file blindly or modify SQL while claiming a comment
  repair.

### 6. Tests Required

- Run the corruption search and assert zero matches.
- Run `cargo test -p ccr-db migration -- --test-threads=1` and `just fmt-check`.

### 7. Wrong vs Correct

#### Wrong

```rust
// 鍔犺浇瀹氫环琛?
```

#### Correct

```rust
// 加载定价表
```
