# Global monitoring implementation receipt — 2026-11-06

Scope: existing GPUI desktop task, HEAD `96376961cd49b7914e3457efba2fc92e9d8ae076` plus current uncommitted batch. This receipt covers implemented source and isolated/headless checks; native visual acceptance remains user-deferred. It does not claim the whole migration is complete.

## Delivered behavior

- Overview exposes a nested global-monitoring lens while preserving all 13 top-level destinations (parent-owned Shell integration). It consumes actual main-database logs and actual application lifecycle events; Claude Observer remains a separate capability.
- `runtime/monitoring.rs` is a Services-owned, application-lifetime owner with a captured main DB path. Production uses `ccr_db::database::ui_database_path`; fixture uses the captured registry parent's `observer.db`, explicitly distinct from `archive.db`.
- DB reads open read-only; writes use read-write without CREATE. Neither read nor write initializes/migrates a database or falls back to a global pool. Missing DB, unsafe path, read/decode failure, and persistence failure are explicit. In-memory events survive persistence/read failure.
- Existing `ccr-db` log persistence/repository own conversion and SQL. New connection-bound query/append entry points use the caller's connection. `query_logs_strict` shares SQL construction with legacy `query_logs`, rejects invalid RFC3339 time and malformed rows, and preserves existing legacy skip/fallback behavior for Tauri callers.
- Static lifecycle event labels contain no command arguments, provider names, hostnames, raw errors, credentials, or arbitrary payloads. Captured values are redacted before persistence and display. Arbitrary legacy metadata/correlation fields are omitted from this feed; message and identity fields are individually bounded.
- Memory feed retains at most 500 events and 512 KiB of UTF-8 payload. It reports retained count/bytes and evicted count. Reads merge persisted and in-memory actual events, filter by level/channel, deduplicate by entry ID, and cap visible output at 500.
- UI offers explicit refresh, level/channel filters, virtualized event rows, keyboard/a11y selection and safe detail. Suspend drops the subscription, increments generation, and clears retained page details; late results cannot repopulate it. There is no timer or hidden polling.
- Runtime facts are actual tracked-operation count, current-process RSS from a shared minimal sysinfo projection, and real ring capacity counters. Ring mutex is released before RSS sampling. Command/database p95 is explicitly unavailable because no histogram exists.
- `ccr-app` owns the shared RSS helper; Tauri reuses it. sysinfo pins the already locked 0.39.6 version with default features disabled and only `system` enabled. The three existing lockfiles were reconciled without package/checksum upgrades. GPUI now declares its direct `ccr-types` dependency.

## Actual lifecycle seams

| Owner | Static events |
| --- | --- |
| Command retained terminal/finalizer | `command.completed`, `command.cancelled`, `command.failed` |
| Usage import retained worker after cleanup | `usage.import.completed`, `usage.import.cancelled`, `usage.import.failed` |
| Tools/manual CLI update retained worker | `tools.operation.completed`, `tools.operation.failed`, `cli.update.completed`, `cli.update.failed` |
| Checkin operation owner | `checkin.operation.completed`, `checkin.operation.failed` |
| Sync operation owner | `sync.operation.completed`, `sync.operation.failed` |
| SSH operation owner | `environment.operation.completed`, `environment.operation.failed` |
| Actual saved-host/local selection transition | `environment.changed` |

Selection is recorded only when selected identity changes in select/save/delete/Use Local callbacks, not generic view observations. A retained runtime finalizer joins the event write even if its receiver detaches. Pure fixture synthetic tools operations do not pretend to have run an installed CLI.

## Current checks

Pinned Rust 1.98.0 toolchain; Git Bash path scoped to check processes; each command uses the package's real working directory and default Rust test parallelism.

- `ccr-desktop`: `cargo test --locked --offline --lib --no-run` — exit 0, actual resolved graph/test executable built (metadata without deps is not counted as proof).
- `ccr-desktop`: `cargo test --locked --offline --lib monitoring::tests -- --skip export_bindings` — exit 0, **5 passed** after strict-time fix. Covers main/archive separation, missing DB no-create, write failure retention, persisted redaction, capacity eviction, malformed row/time errors, detached selection event joined on shutdown, remote refusal before path resolution, rendered filters/detail, suspend/late-result/no-poll behavior, and nested Overview navigation.
- `ccr-desktop`: `cargo test --locked --offline --lib environment_views_fit_native_windows_stack_with_isolated_locks -- --skip export_bindings` — exit 0. Guard now runs **12 children**, including both monitoring UI tests plus requested Codex rolling and Claude common-form controls, with child-only `RUST_MIN_STACK=1048576` and an invalid default lock path. Product stack size was not enlarged.
- Scoped rustfmt on owned Rust sources/tests — passed.
- Root `ccr-db` strict regression: coordinator independently ran `strict_query_rejects_corrupt_rows_and_time_without_changing_legacy_reads` with pinned Rust 1.98.0 — **1 passed**. It verifies strict/legacy behavior separation.
- A final scoped a11y change adds an explicit labelled `ListBox` parent around virtualized `ListBoxOption` rows. Scoped rustfmt passed; the coordinator's upcoming aggregate GPUI tests will cover this final wrapper (the prior 5/12 receipts predate that wrapper).

Compiler cache warning: hard links unavailable for incremental cache, so Cargo copied files. This is not a Rust diagnostic or failed test.

## Remaining evidence/boundaries

- Independent source review, final aggregate fmt/lint/test/full CI, and packaging belong to the coordinator/checker and are not claimed here.
- User deferred actual native interactions; no newly launched app, real SSH/network, user identity file, HOME/USERPROFILE change, or user business-data write was used. Headless GPUI control checks do not constitute native UIA/IME/visual acceptance.
- No measured latency histogram was introduced, so p95 remains unavailable. RSS is a real snapshot rather than a benchmark or performance acceptance result.
- No commit was created by this implementer; coordinator owns atomic batch commit.
