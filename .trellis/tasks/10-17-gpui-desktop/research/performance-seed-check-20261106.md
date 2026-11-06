# Paired benchmark seed — independent check

Scope: `ccr-app/src/benchmark_fixture.rs`, its test-only module declaration, shared fixture contract and the existing ignored output `outputs/desktop/perf/fixtures/paired-20261106-v1`. No generator rerun, UI launch, network/SSH/WSL operation or existing fixture mutation was performed by this reviewer.

## Findings (fixed)

The generator created the allowed outputs-directory prefix before checking its ancestors for symlinks/reparse points. With an existing redirected ancestor and a missing suffix, rejection occurred only after creating a directory at the redirected destination.

- Extracted only that private filesystem boundary as `prepare_allowed_directory`; its generator behavior was initially unchanged.
- Added a regular regression using one owned temporary tree, a Windows junction (Unix symlink on Unix) and a missing suffix. The initial test **failed** because the redirected target already contained the new suffix when validation rejected the link.
- Moved ancestor validation before `create_dir_all`, retained the post-creation check/canonicalization, and reran the same test: **1/1 passed**. The target stays untouched. No existing paired output or global directory was replaced, no assertion weakened and no ignore added.
- Updated the seed receipt and shared fixture spec with this ordering contract. This guard-only change does not alter scenario data or hashes; the earlier 99-second generator receipt is not claimed to have tested this later guard.

## Source and artifact review

- The module is `cfg(test)` and the large generator remains explicitly ignored. It introduces no production dependency or app-startup behavior.
- Destination authority remains a new absolute directory with a canonical existing parent confined to the benchmark output root. Existing destinations are refused; metadata failures other than missing ancestors fail closed. Relative/traversal components cannot redirect the final canonical parent outside the allowed tree.
- Profiles use `ConfigManager` save/snapshot under captured locks. Usage schema/events/buckets come from `ccr-usage::fixtures`; the existing Dashboard/read owner checks aggregate values and pages all records. Raw Codex transcripts are parsed and archived through the real provider registry/shared refresh, then summaries and one actual detail are read through their owners. No adapter duplicates usage SQL or directly fabricates archive rows.
- S0 represents missing profiles with empty usage/session data. S1 creates 100 profiles, 10,000 sessions and 100,000 usage events per end. S2's static data has 10,000 profiles, 10,000 sessions and 100,000 usage events per end. These projection counts come from the generator's owner-backed assertions and its supplied successful run; this reviewer did not rerun the expensive generator or add ad hoc usage SQL.
- Logical profile hashing intentionally covers non-secret presentation fields. Usage hashes every owner-paged event and summary projections. Session logical hashes include source names/bytes and public summaries while excluding physical-root-bound archive IDs. Original opaque IDs stay unchanged and are separately hashed; nonempty paired identity hashes differ as required.
- Independently read all three per-scenario manifests and checked every recorded physical artifact hash (catalog/usage/archive/main for both ends): all **24 role slots match**, including S0's intentionally absent catalog. Existing catalog WAL files are empty; this is not a hash of an omitted nonempty WAL payload.
- Independently enumerated and hashed all actual transcript files for both ends: S0 has 0/0; S1 and S2 each have 10,000/10,000 files. Filename/byte hashes match the recorded `raw_sources_sha256` values. Current artifact bytes therefore still match the source-backed generation receipt.
- All manifests retain the fixed `2026-11-06T12:00:00Z` cutoff, UTC reporting zone and matching paired logical hashes. S2 stream workload explicitly remains `not_emitted`; its 1,000 events/s × 60 seconds descriptor is not emitted traffic, a persisted substitute or a performance sample.

## Verification

- Reviewer-run path regression: **red 0/1 → green 1/1**, pinned Rust/Cargo 1.98, locked/offline, default parallelism, `--skip export_bindings`. Only the narrow regular test ran; the ignored generator was not invoked.
- Reviewer-run `cargo clippy --locked --offline -p ccr-app --all-targets --all-features -- -D warnings`: **exit 0** after the ordering fix and regression.
- Scoped Rust formatting and diff checks: **pass**.
- Supplied original generator receipt: **1 passed**, 99.17 seconds, plus its pre-review strict check. It verifies actual owner-backed dataset counts/hashes; it does not cover the later ordering guard or native loading.

## Findings (not fixed) / limits

No additional source defect was found in this seed slice. This is a paired static dataset, not an accepted benchmark baseline: native benchmark source/loading, common query-clock injection, actual dynamic S2 emission/consumption, process-tree attribution and true process-create/first-present/action-ready trials remain unfinished. Every existing manifest still marks `performance=not_run` and `comparison=not_comparable`.

Parent integrated gates, native acceptance and task archival remain outside this review. The previously reviewed production fixture received its final supplied trap/core-child pass; that receipt is recorded separately in `performance-fixture-check-20261106.md`.
