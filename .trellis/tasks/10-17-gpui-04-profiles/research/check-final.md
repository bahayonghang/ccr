# 04 independent domain/shared check evidence

Date: 2026-10-18. Role: dispatched trellis-check; no recursive implement/check
dispatch. Shared workspace preserved. Stage 1 owns frozen domain/core files;
stage 2 shared/Tauri ownership transferred explicitly, native remains its
implementer's until frozen. No commit, push, publish, signing or archive.

Read task PRD/design/implementation, curated check refs including synchronous
lock and shared configuration policy, parent architecture/performance and
requirements/design, 02 42-command/5-route source map, 03 usable-core check
evidence, actual frozen domain/services reports, receipts and source paths.
This report does not accept native UI from compilation or mark the complete
migration done. User deferred geometry/zoom/DPI checks.

## Findings fixed so far

1. `crates/ccr-cli/src/platforms/grok.rs` staged save checked the editor revision
   in a completed no-op mutation before patch composition, then saved without
   that baseline. A nested catalog mutation in the callback demonstrated the
   stale save could overwrite the intervening field. The owner now binds its
   original snapshot revision inside the actual destination-save catalog
   transaction. The callback executes before that transaction; its side effects
   are not replayed. Grok operation -> catalog -> leaf order is retained.
2. The same staged rename created a new profile from a JSON projection without
   seeding the original TOML section. A stored datetime became another type.
   Seed the exact source section in the destination-save transaction, then merge
   the existing platform delta. Commit destination before the existing staged
   apply/delete policy; existing real apply/cleanup failure outcomes remain.
3. `profile_off.rs` omitted Codex's legacy `profile_entry_auth_state.json` from
   its backup. The domain removes this file before registry update; a one-shot
   registry write failure restored runtime/auth/registry but lost the snapshot.
   Add the existing owner-managed file to the same backup/rollback set. This is
   a pre-existing defect exposed by the actual injected 04 off path. No active
   SQLite file is copied/deleted, and no domain cleanup rules are duplicated.
4. First-create compensation was incomplete. On an absent Codex catalog, a
   synthetic external runtime write after credential publication made journal
   verification fail. Rollback removed credentials but left the newly published
   catalog. The main session explicitly expanded ownership to the catalog owner.
   Added `CatalogTransaction::execute_or_create(legacy, db, expected, body)`:
   existing catalogs use the original transaction with an in-transaction version
   check; absent catalogs use the existing private TempSnapshot/schema/Session
   and commit helpers, then the existing create-new installer. All platform
   reads/writes join the same live-path Session while its connection is staged.
   Error, commit failure and panic remove only owned staging. Live database and
   late legacy-file collisions reject without overwriting foreign data. The
   existing execute method uses its unchanged flow factored into execute_session.
   Lifecycle creation now always calls this owner, without an absence bypass.

Focused failure regressions are in `profile_lifecycle_tests.rs`. The initial
Grok two new tests both failed, while the two existing stage tests passed.
After repair all four staged tests passed. The off regression failed before
repair and passed in the subsequent full 16-test injected suite.

## Findings not fixed / remaining verification

- **Intermittent binding drift remains unexplained.** First original-checker
  invocation with locked export steps passed; a subsequent independent proof run
  failed with 11 generated type differences, then the instrumented run and three
  cached repetitions passed without any source/generated change. The failing
  run did not yet capture before/after contents, only the 11 paths. Later runs
  capture exact regenerated diffs and report empty arrays. Do not claim a fixed
  root cause or stable overall gate. Fail paths were observer HeatmapCell and
  InsightDto; sync SyncStatusInfo and WebDavConfigInput; usage DailyTrendDto,
  HomeInsightsResponse, InsightsDay/Tally/Totals/TrendSeries and
  UsageImportResultV2. No tracked binding was rebaselined or generator altered.
- Generator inspection: original runBindingStep uses spawnSync; each export
  child alone receives RUST_TEST_THREADS=1, and ordinary behavior tests retain
  their default parallelism. The actual ts-rs 12.0.1 macro generates cfg(test)
  export functions; compiler execution alone is not proof of a second writer.
  A concurrency/order/normalization explanation remains unproven. Main owns any
  necessary generator/DTO change after a concrete reproducer, not speculative
  changes to unrelated files.
- Existing off backup still lists legacy profiles.toml instead of live catalog;
  no actual catalog-loss failure was established in this review. Do not claim
  this observation is a proven failure or widen the off design speculatively.
- Native 42-command/5-route mapping, rendered actions/teardown, actual CU,
  final affected gates and parent acceptance remain open.

## Actual commands and results

All root/Tauri commands use the recorded rustup-resolved Cargo 1.98.0 binary
with its matching toolchain bin prepended to PATH only in the child process.
No global HOME, USERPROFILE, PATH, MBX config or cache cleanup changes. Default
Rust test parallelism and `--skip export_bindings` retained. Explicit rustfmt
repair was separate from validation. Existing managed target access needs
approved filesystem access; the initial default-sandbox cache denial is not
a product test failure.

| Actual command | Exit/result | Raw output under outputs/desktop/profiles/ |
| --- | --- | --- |
| `cargo test -p ccr-cli --all-features --locked grok_staged_ -- --skip export_bindings` default sandbox | 101, managed target Access denied before tests | check-grok-regressions-before.log |
| same with cache access, before repair | 101, 2 pass/2 fail | check-grok-regressions-before-elevated.log |
| same after repair | 0, 4 pass | check-grok-regressions-after.log |
| `cargo test -p ccr-cli --all-features --locked injected_codex_off_registry_failure -- --skip export_bindings` before repair | 101, 1 fail | check-off-regression-before.log |
| `cargo test -p ccr-cli --all-features --locked application::profile_lifecycle::injected_tests -- --skip export_bindings` after off repair | 0, 16 pass | check-domain-injected-final.log |
| `cargo test -p ccr-cli --all-features --locked injected_first_create_external -- --skip export_bindings` before fix | 101, 1 fail | check-first-create-before.log |
| injected lifecycle suite after owner staging fix | 0, 17 pass, before the additional marker/commit tests | check-first-create-after.log |
| `cargo test -p ccr-config --all-features --locked first_create_transaction -- --skip export_bindings` final | 0, 3 pass including readonly/late legacy collision | check-catalog-first-create-final.log |
| `cargo test -p ccr-cli -p ccr-config -p ccr-codex -p ccr-app --all-features --locked -- --skip export_bindings` | 0: app23 pass/2 ignored; CLI409 pass/1 ignored/24 filtered; dispatch12 pass; Codex392 pass/2 ignored; config126 pass/1 ignored; doctests1 pass/2 ignored | check-domain-shared-regression-final.log |
| `cargo clippy -p ccr-core -p ccr-config -p ccr-codex -p ccr-cli -p ccr-app --all-features --all-targets --locked -- -D warnings` | 0 | check-domain-shared-clippy.log |
| `python scripts/quality/check_secret_writes.py` | 0, Sensitive persistence policy check passed | tool output |
| same strict Clippy after final owner guard/tests | 0 | check-domain-shared-clippy-frozen.log |
| `cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml commands::config:: --locked -- --skip export_bindings` frozen adapters | 0, 18 pass/1 ignored child-only harness | check-tauri-config-frozen.log |
| `cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml commands::grok::tests --locked -- --skip export_bindings` after domain changes | 0, 16 pass | check-tauri-grok-after-domain.log |
| precompiled Tauri `ccr_desktop-923e4ec82b23d93c.exe commands::handler_registry::tests::command_inventory_document_matches_registry --exact --skip export_bindings` | 0, 1 pass (static registry source unchanged) | check-command-inventory.log |
| owned Rust `rustfmt --edition 2024 --check` and scoped `git diff --check` | both0, validation only | check-owned-fmt.log, check-owned-diff.log, check-frozen-receipt.json |

The broad regression preceded the final late-legacy-file publication guard and
readonly assertion; the final three-test catalog suite executes both additions,
and final strict Clippy compiles all affected targets/features afterward. No
unnecessary broad repeat was performed. The broad CLI suite includes all 19
injected tests after the two extra first-create marker/commit cases: actual
Claude/Codex/Grok first-create current/default equals the direct platform owner;
an existing inactive empty catalog stays inactive; first commit failure leaves
no secret/catalog; late journal verification leaves no orphan and retains the
external runtime. The catalog tests also verify panic clears Session and a later
operation succeeds, two first creators yield one winner, read-only legacy and
foreign database/late-legacy collisions retain their bytes.

## Binding gate method and retained failure

First invocation in ccr-ui:
`bun -e 'import {checkBindings} from "./scripts/check-generated-bindings.mjs"; import {runBindingStep} from "./scripts/generate-bindings.mjs"; const status = await checkBindings({runStep: step => runBindingStep({...step, args: step.exports ? [...step.args, "--locked"] : step.args})}); process.exit(status)'`
exited0 (`check-tauri-bindings-locked.log`). This is the original drift checker
with only --locked appended to its original Cargo export arguments, not an
unmodified just recipe invocation. Its actual exports were24+9+197 pass.

The independent proof wrapper is
`outputs/desktop/profiles/check-bindings-proof.mjs`; execute with the same child
toolchain PATH using `bun outputs/desktop/profiles/check-bindings-proof.mjs TAG`.
It calls original checkBindings and runBindingStep, retains restoreAlways, and
computes file-name-plus-exact-byte SHA256 before and after, plus three lock hashes.
An assertion proves all230 original files and locks are unchanged. It captures
differences after the second normalization but before original restoration.

- The first proof failed drift (`check-tauri-bindings-proof.log`, actual exit1),
  with all export tests passing and exact restoration assertion passing. Its
  original untagged JSON was superseded by the later instrumented pass; raw
  failure/log/path list remains. Do not present that superseded JSON as its receipt.
- Instrumented untagged run0 (`check-tauri-bindings-drift.log`) and three tagged
  repetitions0 (`check-tauri-bindings-repeat-{1,2,3}.log`). Per-run
  `check-bindings-proof-repeat-*.json` and `check-bindings-drift-repeat-*.json`
  retain exact receipt/diffs. No further blind repetitions were performed.
- The230-file raw-byte hash is
  `f56dc6d0960a4cc83121d878f567112c0d1a41db0ef2493c8998ecae57ae3a92`.
  All three locks retain root `f4a932f8ca98b21ec80b4d0d178e6c3a67f7510993da49b91b03a55c07fa3b15`,
  Tauri `7b4e0585f021f9898400143f712798b868478a6ad6f73b496e693d906a48abaa`,
  desktop `14900fb870ff12effe9129a663aec7fb92d11dd3abef79fa1dce41ecf3c69a56`.
  Final generated gitdiff is empty. All checker-owned Cargo/binding processes
  finished; no native GUI was launched by this checker.

Domain implementer's earlier broad config/CLI/Codex regression, core scoped-lock
tests and final ancillary compatibility test were read with their frozen source
receipt. They are existing evidence, not checker reruns and not verification of
subsequent changes. New owner fixes require the focused/broader checks recorded
above and the final integration gates below.

## Scope and security observations

- Core directory scope is synchronous/thread-local RAII, normalizes using the
  existing resource identity helper, rejects nested directory switching before
  running the body, and restores after success/error/unwind. No domain path
  holds it across await/spawn; it selects the directory for existing locks.
- Explicit context constructors and missing-read tests are path-only. Runtime,
  registry, history, credential and nested lock operations use declared fixture
  owners. Default legacy factories remain distinct. GUI tests must still verify
  native ConfigSource derives every context consistently.
- Grok's already-locked borrowed adapter is private and prevents reacquiring its
  private operation lock inside the catalog transaction. Existing staged
  cleanup can fail after source catalog deletion; callers must refresh actual
  names/runtime and must not universally promise both names survive.
- Source/export values intentionally have no Debug; canonical profiles.db source
  is distinct from verbatim runtime TOML. Force removes only activation-conflict
  protection, never CAS. Generic redacted and dedicated secret-only exports are
  separate protocols.

Domain/shared fixes are frozen with hashes in check-frozen-receipt.json. Native
implementation/check ownership remains separate. No final 04 AC, complete native
surface, performance, other OS, packaging or parent migration acceptance is
claimed while native/integration checks and binding stability remain open.
