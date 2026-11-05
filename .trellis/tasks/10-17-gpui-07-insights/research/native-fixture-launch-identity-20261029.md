# Native fixture launch identity investigation, 2026-10-29

Scope: read-only investigation requested by main after launching the immutable
integrated candidate `1157f008a5793668447092c94cbb1a49ff938d48dbb991bd97f918c01b64ef7f`.
No product source, fixture database, real configuration, HOME, account, or network
was changed by this checker. No Cargo gates or UI operations were run here;
main owns the native run and serialized gates. The saved hook selected task08,
so the explicit task07 artifacts/context and current source were used instead.

## Finding

The v2 seed and native launcher supplied different spellings of the same Windows
provider root. `fixture.json` records `provider_home` beginning with `\\?\D:\`.
`outputs/desktop/continuation-20261028/run_native.py` ignores that manifest and
reconstructs `ROOT / outputs/desktop/insights / fixture-name / home`, producing
ordinary `D:\` spelling in the actual `integrated-resume29/launch.json`.

`AgentSessionProviderRegistry::discover_for_refresh` retains the root's spelling
in discovered paths. The archive physical identity and ID intentionally use
the exact `(platform, file_path, source_member_id)` strings. The seed's same-
registry refresh assertions therefore pass, but do not test this launcher seam.
On launch, refresh discovers 205 ordinary-path sources, inserts those identities,
and correctly reconciles the different prefixed identities as unseen/missing.

Read-only SQLite inspection after main's native refresh found 205 `live` rows
with ordinary `D:\` paths and 205 `missing` rows with `\\?\D:\` paths. For the
same `rollout-native-000.jsonl`, the ordinary path has archive ID
`as-6fe463aec51d25f47f64f7890a348765` and the original prefixed path has ID
`as-829ac5c57282d92f3908b3a290bf7bae`. The original row now has the reconciliation
time in `updated_at`, explaining why missing rows appear ahead of the newly
parsed rows and order by opaque ID rather than the fixture's 204-to-000 times.

This is a confirmed fixture-launch identity mismatch. It does not demonstrate
a stale candidate, a wrong provider root, source-file deletion, or a new
requirement to redesign product identity normalization.

## Candidate identity

All five targeted current source hashes equal the hashes frozen in
`outputs/desktop/continuation-20261028/integrated-source-before.json`:

- `crates/ccr-app/src/sessions/refresh.rs`
- `crates/ccr-app/src/usage/native_fixture.rs`
- `ccr-desktop/src/runtime/insights.rs`
- `ccr-desktop/src/features/insights/mod.rs`
- `crates/ccr-store/src/sessions/providers.rs`

The integrated candidate includes the v2 seed implementation and current refresh
generation guards; it does not predate them. The earlier baseline candidate is
separate evidence and must not be substituted.

## Minimal next action

Main should have the fixture launcher read the exact three paths from the
fixture's manifest, validate their ownership/existence, and pass them unchanged.
Preserve this run and the modified v2 archive as failed seam evidence. Generate
a new owned fixture and retry; do not edit historical DB rows or source IDs to
manufacture a pass. The new run should verify 205 unchanged/zero upserts and
205 total archive rows after the native refresh, plus stable first/last pages.

No product repair is proposed from this finding. Full task07 and parent
acceptance remain open. Lint/type-check/tests were not rerun because no product
source changed and main already owns the active gates.
