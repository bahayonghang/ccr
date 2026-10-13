# Recorded Profile Activity

## 1. Scope / Trigger

Use this contract when reading last-used timestamps or recent activations for Profile views. CCR operation history belongs to ccr-store; llmusage provider analytics remain in ccr-usage. This projection never initializes history storage or repairs records.

## 2. Signatures

```rust
ccr_store::Database::default_path() -> ccr_core::Result<PathBuf>
ccr_store::read_profile_activity(
    path: &Path, platform: Platform, names: &[String],
) -> Result<ProfileActivitySnapshot, ProfileActivityError>
ccr_cli::application::profile_activity::load_profile_activity(
    platform: Platform, names: &[String],
) -> Result<ProfileActivitySnapshot, ProfileActivityError>
```

Serializable DTOs live in ccr-types and are re-exported through the store/facade: `ProfileActivitySnapshot { availability, profiles }`, `ProfileActivity { recent_activations, invalid_timestamps }`, and `ProfileActivityAvailability::{Ready, MissingDatabase, MissingSchema}`. Default availability is MissingDatabase. The owning store's fixed `ProfileActivityError::{PathUnavailable, Open, Query}` carries no paths, row contents or raw SQL errors; the CcrError freeze permits this local typed boundary.

## 3. Contracts

- Resolve `CCR_DATA_DIR`, then `CCR_ROOT`, then the existing home fallback through the same pure resolver used by `Database::init_default()`. Resolving is distinct from creating directories/pools or running migrations.
- Open existing storage with `SQLITE_OPEN_READ_ONLY`. Do not add write pragmas, indexes, checkpoints or `immutable=1`; active WAL commits must remain visible. SQLite owns WAL shared-memory reader bookkeeping, so do not claim every SHM byte is immutable.
- Check required columns, then perform one filtered history SELECT for a nonempty names batch. Select only `to_config, timestamp`; require exact `operation = 'Switch'`, `result = 'Success'`, and `extra = platform.short_name()`.
- Match exact requested names and parse timestamps strictly with RFC3339 into UTC. Do not use the permissive HistoryEntry decoder or lexical MAX(timestamp). Keep at most the newest three instants per key, including duplicate recorded instants. Retain valid future instants for a clock warning; count invalid eligible timestamps separately.
- No credentials, env changes, arbitrary notes or complete history records cross this boundary. Memory is bounded by the requested names, not history length.
- Last used means the last recorded successful CCR activation, not last API usage. Switch metadata count is independent. Cleared/legacy/unattributed history, rename/name reuse and ancillary-write failures limit completeness; absent records never imply never used.

## 4. Validation & Error Matrix

| Input/state | Result |
| --- | --- |
| Missing database/path parents | MissingDatabase; no creation |
| Existing database missing required history columns | MissingSchema; no migration |
| Corrupt, locked or unreadable storage | Typed Open/Query error, not an empty success |
| Same profile name on other platform or failed/unknown operation | Excluded |
| Valid mixed-offset timestamps | Newest three by UTC instant |
| Malformed/null/non-text timestamp | Per-key invalid count; never now |
| Future timestamp | Preserved for UI warning |
| No eligible row | Ready with no matching profile entry |

## 5. Good / Base / Bad Cases

- Good: a background refresh requests all visible platform names once and renders from the cached DTO.
- Base: a fresh installation displays unavailable history without creating its data directory.
- Bad: calling `HistoryManager::with_default()` from a view, deriving time from mtime, or using provider activity as profile activity.

## 6. Tests Required

Run default-parallel `cargo test -p ccr-store --test profile_activity -- --skip export_bindings` and facade/DTO tests filtered by `profile_activity` in ccr-cli/ccr-types. Assert exact filters, UTC order/top-three bounds, malformed/future timestamps, missing schema/storage, retention and safe errors. Closed rollback-journal fixtures must retain file bytes/mtime; active WAL fixtures must expose latest commits without application writes to database/WAL. Keep fixtures isolated from user data.

The ignored `benchmark_profile_activity_owned_history` reports 0/10k/100k-row fixture timings. It is observational and must not become a flaky wall-clock CI assertion or a claim of cold-disk latency.

## 7. Wrong vs Correct

Wrong: initialize a writable HistoryManager, load all secret-bearing records, and use its timestamp fallback or SQL text MAX for each displayed row.

Correct: background-load one read-only projection through the facade, preserve its availability/error state, and consume exact-name UTC timestamps from the cached snapshot without rendering I/O.
