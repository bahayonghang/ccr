# Profile activity implementation evidence

Date: 2026-10-12. Baseline checkout: `8115cd116632ae2d897401b1de4e8385d9b19d6b` plus the current task's uncommitted work. All database data below is synthetic and owned by tempfile fixtures; no personal history database or real profile is opened or changed.

## Interface and ownership

`ccr_cli::application::profile_activity::load_profile_activity(platform, names)` returns `Result<ProfileActivitySnapshot, ProfileActivityError>` for a single explicit platform and a batch of exact names. The facade performs no writes. Rendering consumes the cached result through the separately implemented TUI snapshot loader.

- Shared serializable data lives in `ccr-types`, per `crates/AGENTS.md`: `ProfileActivitySnapshot { availability, profiles }`, `ProfileActivityAvailability::{Ready, MissingDatabase, MissingSchema}`, and `ProfileActivity { recent_activations, invalid_timestamps }`. Snapshot default is MissingDatabase, not a fabricated ready state.
- The storage projection and local safe error categories live in `ccr-store::storage::profile_activity`. `ProfileActivityError::{PathUnavailable, Open, Query}` contain no underlying SQL error, row contents, or credentials. This follows the local typed-error exception in the CcrError freeze; no `ccr-db`/desktop dependency or CcrError variant was added.
- `Database::default_path()` extracts the existing `CCR_DATA_DIR > CCR_ROOT > dirs::home_dir()/.ccr` resolution without initialization. `init_default()` reuses that resolver and retains its original write/migration behavior for actual writers.
- SQL selects only `to_config, timestamp`, filters `operation = 'Switch'`, `result = 'Success'`, and the exact platform string from `extra`. Neither the permissive HistoryEntry decoder nor any credential-bearing column is loaded.
- One schema-capability query plus one filtered history SELECT per nonempty refresh; no per-profile query, sort of all records, schema migration, new index, checkpoint, or write PRAGMA. A HashSet checks requested names and retains at most three UTC instants per matching key. Duplicate instants remain distinct recorded activations. Eligible invalid/null/non-text timestamps increment a per-key count; valid future timestamps remain future values for the renderer's clock warning.
- No record is an absent map key (or empty timestamps with an invalid count), never proof of no historical usage. Retention, unattributed legacy rows, rename/name reuse, and post-commit history failures retain the approved semantic limitations.

## Read-only guarantees

Uses `SQLITE_OPEN_READ_ONLY`, never a write-capable pool or `immutable=1`. Missing paths stay missing, missing/old schema stays unchanged, and corrupt/locked reads return typed errors rather than masquerading as empty data. Closed rollback-journal fixtures retain every file's bytes and modification time. Active WAL fixtures expose current committed history while database/WAL bytes stay unchanged and no new files appear in that tested state.

SQLite may maintain shared-memory read bookkeeping for WAL; the test does not claim that every SHM byte is immutable. This implementation does not force an immutable snapshot or ignore live WAL commits to avoid that bookkeeping. No blanket claim is made about WAL sidecar creation under every external-process/crash state.

## Measured batch reads

Command:

```text
cargo test -p ccr-store --release --test profile_activity benchmark_profile_activity_owned_history -- --ignored --nocapture --skip export_bindings
```

Environment: Windows NT 10.0.26300.0, Intel Core Ultra 9 275HX, 24 logical processors; Rust 1.98.0 (`88d9e12ae`, 2026-08-18). Repository release profile `opt-level=3`, LTO, one codegen unit. Fixture databases on local C: NTFS temp directory; existing Cargo target cache on D: NTFS. Other agents may compile concurrently. These are workstation observations, not a universal latency guarantee.

Each dataset has 1,000 requested names; rows distributed across Codex/Claude/Grok/unattributed metadata, successful/failed operations, malformed timestamps and offset-bearing timestamps. 100k rows yield at most 250 Codex profile keys due to fixture distribution. Each timed call opens a new read-only connection, inspects required schema and reduces matching history. Filesystem cache is not evicted. "First" means first projection after writing the fixture, not a cold disk measurement. Thirty subsequent samples use fresh connections with a warm OS cache; p95 is nearest-rank sample 29/30.

| History rows | DB bytes | First ms | Median ms | p95 ms | Maximum ms |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 0 | 16,384 | 0.907 | 0.812 | 1.357 | 1.494 |
| 10,000 | 1,224,704 | 6.960 | 6.401 | 8.301 | 8.827 |
| 100,000 | 12,292,096 | 46.247 | 45.422 | 61.017 | 61.090 |

SQLite query plan for each: `SEARCH history USING INDEX idx_history_operation (operation=?)`. The measured batched reads did not justify adding an index in this task. No previous equivalent batched read API existed; therefore there is no before/after speedup claim. The legacy writer-capable full-history load is semantically unsuitable as a baseline. Open/schema/query stage timings and true cold-cache measurements are not separated here. Idle/render I/O guarantees and total UI responsiveness belong to the TUI integration tests.

## Verification

- `cargo test -p ccr-store --test profile_activity -- --skip export_bindings`: 7 passed, benchmark intentionally ignored. Covers exact platform/name/status/operation, invalid/legacy attribution, top-three UTC order with misleading offsets, repeated instants, future and malformed/null/numeric/blob dates, cleared history, missing DB/schema, corrupt/locked storage, no sensitive DTO/error output, and WAL visibility.
- Release benchmark command above: 1 passed, 30 samples for each of three sizes.
- Facade tests exercise the actual HistoryManager writer contract for all three platforms and default-path precedence. The initial home fallback assertion assumed Windows obeyed HOME/USERPROFILE; Windows uses its known-folder API. Corrected to compare the pure resolver with `dirs::home_dir()` without opening that fallback path; fixture reads still use explicit temporary CCR_DATA_DIR.
- Shared-type tests cover missing-field defaults and timestamp/partial-data JSON round trips.
- `cargo test -p ccr-cli -p ccr-types profile_activity -- --skip export_bindings`: final rerun passed, 2 facade tests and 2 type tests; default test parallelism retained.
- `cargo test -p ccr-store -- --skip export_bindings`: 82 existing unit tests, 7 new integration tests and 1 doc test passed; 5 intentional ignores across unit/integration/doc suites.
- `cargo clippy -p ccr-store -p ccr-types -p ccr-cli --all-targets --all-features -- -D warnings`: passed.
- `rustfmt --edition 2024 --config skip_children=true --check` for all nine changed Rust files and owned-scope `git diff --check`: passed.
- Initial sandbox Cargo invocation could not write the existing repository target symlink's external cache. Re-ran approved focused commands with escalation; did not relocate or clear the cache. Integrated TUI/full-repository acceptance remains with the coordinator.
