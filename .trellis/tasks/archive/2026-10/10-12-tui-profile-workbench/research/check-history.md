# Profile activity boundary: independent scoped review

Date: 2026-10-12. Reviewer: dispatched `trellis-check`. This is the intermediate data-boundary review, not final acceptance of the TUI implementation. Other agents were editing TUI files; this review did not change them or any product source.

## Scope and evidence

Reviewed the new `ccr-types::profile_activity` contracts, `ccr-store::storage::profile_activity` projection, `Database::default_path` extraction, CLI facade and their module/re-export integration. Read the parent PRD/design/implementation plan and pertinent native-hook/spec context, then compared the projection against the actual `HistoryManager::add` writer and `profile_lifecycle` post-commit history recording.

- The lifecycle writes `Switch` only for committed activation and `Update` for inactive changes. The projection requires exact `Switch`, `Success` and the requested platform string. `Platform::Display` delegates to the same `short_name` used by the reader. Requested profile names match exactly; unattributed legacy history is not assigned by guesswork.
- Only `to_config` and `timestamp` are selected. The old whole-entry decoder and its fallback-to-now/default-success behavior are bypassed. No `env_changes`, notes, credentials, or raw SQLite errors cross this boundary.
- Timestamps use RFC3339 parsing and UTC instant ordering. The reduction retains at most three timestamps per requested name, including distinct repeated activations. Invalid eligible dates are counted separately; future valid dates remain future instants for the presentation warning. There is no clock substitution or lexical `MAX(timestamp)`.
- The requested-name set and result map bound memory by requested profiles rather than history size. One schema query and one streamed history query serve each nonempty refresh; no per-profile query or history-wide in-memory sort is introduced.
- Missing path returns MissingDatabase; absent required schema returns MissingSchema; corrupt/locked storage returns the safe Query error. An empty valid history remains Ready with no fabricated rows. A failure cannot masquerade as an empty valid result.
- The projection uses `SQLITE_OPEN_READ_ONLY`, never `Database::init`, migration, write PRAGMA, directory creation, checkpoint, or `immutable=1`. The live-WAL test confirms committed records remain visible and DB/WAL bytes remain unchanged. As documented by the implementer, this does not claim SQLite SHM read bookkeeping is immutable in every external-process state.
- `default_path` preserves the writer's prior CCR_DATA_DIR > CCR_ROOT > home/.ccr path resolution. `init_default` still creates the same directory and delegates to the original initialization path; read callers use only the pure resolver.
- Shared types remain serialization-only in `ccr-types`, with backward-safe missing-field defaults and round-trip coverage. The local store-owned `ProfileActivityError` is consistent with `ccr-error-freeze.md`'s explicit exception for flows requiring typed error handling; it adds neither a CcrError variant nor a ccr-db dependency.

## Findings (fixed)

None. No product changes were needed after independent source and test review.

## Findings (not fixed)

No defect found in this scoped data boundary. The existing store guideline overgeneralizes the shared SQLite seam's DbError rule to all new modules; the coordinator owns the already-planned narrow spec clarification and new activity contract. This is a documentation alignment item, not a reason to introduce a desktop database dependency.

TUI formatting of partial/future/missing states, async generation handling, mutation safety, zero-I/O navigation, and final visual acceptance are explicitly reserved for the subsequent full-scope review.

## Verification

Independently rerun in this review, preserving default test parallelism:

- `cargo test -p ccr-store --test profile_activity -- --skip export_bindings`: pass, 7 tests; 1 intentional ignored benchmark. Includes missing DB/schema unchanged-file assertions, exact attribution, offset ordering, malformed/future dates, cleared history, corruption/locking and live WAL visibility.
- `cargo test -p ccr-cli -p ccr-types profile_activity -- --skip export_bindings`: pass, 2 facade tests and 2 type tests. Facade fixtures exercise the actual history writer for all three platforms and pure default-path precedence.
- `cargo clippy -p ccr-store -p ccr-types -p ccr-cli --all-targets --all-features -- -D warnings`: pass. Rust type checking included.
- `rustfmt --edition 2024 --config skip_children=true --check` over all nine scoped Rust files: pass.
- `git diff --check -- crates/ccr-types crates/ccr-store crates/ccr-cli`: pass.

The initial sandboxed Cargo invocation could not write the existing external target-cache lock; approved escalation used the same cache. No cache configuration was changed. All databases came from temporary test fixtures; no real profile activation, private history read, or credential inspection occurred. The release benchmark was not rerun because the code did not change and the existing implementation report already records its measured environment and limitations.
