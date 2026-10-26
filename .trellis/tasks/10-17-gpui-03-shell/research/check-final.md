# 03 independent check

Date: 2026-10-18. Windows/MSVC, development Rust 1.98.0; current complete
GPUI graph also checked with Rust 1.95.0. Shared workspace preserved. No native
launch, resize/DPI operation, real account/configuration test, commit or publishing
by this reviewer. Main owns native evidence, task status and parent inventory.

## Findings (fixed)

- File: `ccr-desktop/src/shell/command_state.rs:217`.
  Issue: Remove favorite and Clear recent commands immediately dispatched
  irreversible deletion. Approved DESIGN.md requires an object/consequence
  confirmation for irreversible actions. Main confirmed the scoped correction.
  Fix: shared typed pending confirmation, explicit Confirm deletion and Cancel,
  Escape cancellation, actual captured focus restoration on cancellation, and a
  retained rendered content focus after confirmation disappears. Navigation clears
  pending confirmation. No storage job is admitted before Confirm. While pending,
  conflicting actions are disabled and the existing mutation guard is retained.
  Clear text explicitly includes **all** desktop history, including older records
  outside the default 50-row list. Favorite target uses its stable UUID and safe
  projected display label and explains deletion of its stored arguments.
  Regression: actual favorite controls preserve records on Cancel/Escape then
  remove only on Confirm; actual seeded history controls preserve data and return
  keyboard focus on Escape, then clear and reread the private database after
  Confirm. No fake execution/history or direct SQL was added.

No frozen domain/runtime source fix was necessary.

## Findings (not fixed) / remaining acceptance

- Native role/name/state delivery and OS screen-reader behavior remain
  unverified. Source installs disabled/busy on the original Button node through
  the exact public synthetic callback seam. Kit snapshots observe before that
  callback and the public debug tree omits those flags. Neither None nor a safe
  label is treated as proof. Main's first actual03 UIA capture is null.
- Main's first03 native run is explicitly partial: real catalog/physical-key
  character/exit0 passed for the former `0efb...` artifact, but packet text
  injection did not appear. This is not yet attributed to the application or
  the tool; actual OS IME/core continuation belongs to main. Kit input was not
  rewritten speculatively.
- Original layout geometry is not fully implemented: config split children use
  equal `flex_1` rather than approved 3:2 (`app.rs:334,352`); narrow content still
  uses 24px instead of the approved 16px (`app.rs:400`, shell/mod.rs:161 and
  command_state.rs content). These are bounded design/geometry follow-ups, not
  source evidence of a measured clipping defect. This reviewer left those
  interaction/layout decisions for main rather than silently altering geometry
  while user explicitly deferred size/DPI verification. Full original AC3.1
  cannot be claimed. The responsive drill-in/minimum-window boundary is present.
- Native OS appearance events under System are source implemented but not
  independently simulated by external TestAppContext (exact test_window API is
  private). Existing actual theme controls/palette synchronization pass. An
  explicit Light/Dark choice uses the retained observer's System-only guard.
- No macOS/Linux native, performance samples, complete parent AC1-AC8 or final
  just ci result is claimed. User-deferred matrix and external acceptance stay
  separate from an engineering dependency handoff.

## Source/spec review

Read saved complete hook context plus all nine check.jsonl entries, current
PRD/design/implementation, domain/ui evidence and main's initial native-shell.md.
Checked actual storage -> typed adapter -> bounded runtime -> rendered controls:

- Existing ccr-db repositories remain sole SQL owner. No schema, migrations,
  UUID, command construction or retention logic was copied into desktop.
- DbAccess injection/fallible reads preserve native errors; existing default
  constructor/read wrappers preserve Tauri API and empty-on-read-error behavior.
  No Tauri DTO/export change in03.
- Native constructors are path/cache-only; actual initialization/migrations/
  queries run in02's bounded background executor. Services clones share one lazy
  manager cache. Private fixtures never register GLOBAL_POOL or modify HOME.
- Native projection excludes raw args/full_command, reuses current log redaction
  on metadata, and replaces storage error payloads with typed safe values.
  Arbitrary free-text redaction is not guaranteed by that existing owner; its
  actual limits are documented, not waived by a fabricated sentinel test.
- Reads cancel/detach on view drop. Mutations remain app-owned. A committed
  mutation returns Success despite late cancellation. UI loading/error/generation
  gates and no automatic mutation replay are preserved.
- Real capability-only Local navigation, English/Chinese replacement, session
  theme settings, stable-name virtual rows, actual F6/Tab/list/back controls and
  no decorative continuous animation are implemented. Render does not perform
  synchronous I/O or create another runtime.
- Existing backend spec already prescribes DbAccess injection and private fixture
  ownership; current app spec already prescribes bounded runtime and cleanup.
  No new spec layer or general framework abstraction is needed for this fix.

## Actual reviewer verification

All behavior tests use default Rust parallelism and `--skip export_bindings`.
No validation command performed fmt/version-sync repairs. One explicit scoped
rustfmt repair was separate before validation. Root/Cargo commands requiring the
existing external mbx cache used authorized cache access; no global setting change.

| Command | Exit | Result / exact raw log |
| --- | --- | --- |
| `cargo test -p ccr-db --locked -- --skip export_bindings` | 0 | 127 passed, 0 failed; doc-tests0. Actual check tool transcript. |
| `cargo clippy -p ccr-db --locked --all-targets --all-features -- -D warnings` | 0 | Actual check tool transcript. |
| `just version-check` | 0 | `outputs/desktop/shell/check-root-version.log` |
| `just fmt-check` | 0 | `outputs/desktop/shell/check-root-fmt.log` |
| `just desktop-test` after fix | 0 | Python12, exact-source AccessKit3, Rust lib8, native_shell2, native_ui2; main/doc0. `check-desktop-test-final.log` |
| `just desktop-test-ui` after native closed | 0 | Real native_shell2 + native_ui2. `check-desktop-ui-final.log` |
| `just desktop-lint` | 0 | Strict locked all-target clippy. `check-desktop-lint-final.log` |
| `just desktop-check` | 0 | Locked all-target type check. `check-desktop-check-final.log` |
| `just desktop-fmt-check` | 0 | Read-only formatting check. `check-desktop-fmt-final.log` |
| child `RUSTUP_TOOLCHAIN=1.95` then `just desktop-check` | 0 | Actual matching1.95.0 Cargo/compiler, full graph with ccr-db,39.70s. `check-desktop-msrv-final.log` |
| `just desktop-build-debug` | 0 | Normal1.98.0 build7.23s. `check-desktop-build-debug-final.log` |

All short log names above are under `outputs/desktop/shell/`.

## Failed attempts retained

- Initial desktop-test-ui sandbox exit1: could not verify mbx shim identity from
  restricted cache status. Authorized retry reached actual Cargo.
- Authorized retry exit101: concurrent main native process held ccr-gpui.exe,
  while Cargo test tried to remove/rebuild it. Windows denied access; measured
  running artifact hash stayed unchanged. This was coordination, not a product
  pass/fail. Main closed normally before further Cargo activity; final gate0.
- First new favorite test exit101: plain Div group was not registered by Kit's
  component observation. Corrected to require the actual visible Confirm button,
  then exercise real Cancel/Escape/Confirm and persistent data.
- First history test exit101: standalone feature window had no shell activation
  observer establishing initial focus, so unfocused Tab did not reach controls.
  Corrected setup uses native window.focus_next once, then real Tab/Enter and
  exact restored trigger assertions. Focused final2/2 and full suite8/8 passed.
- One explicit rustfmt attempt used a root-relative path from desktop cwd and
  returned missing-file (no source change). Correct root invocation succeeded;
  later read-only desktop-fmt-check0 is the actual formatting evidence.

## Frozen native handoff

`ccr-desktop/target/debug/ccr-gpui.exe`, 56,220,672 bytes, SHA256
`f69266960cfad286007a4184c42cda8276b8b6987186051683aa3e1b44d99af4`.
Cargo.lock SHA256
`9c8f5b816b01a2db2285d0092fa223e791ffe3f48f02657ec4edad1708c7216f`.
Final command_state.rs SHA256
`57e8b8d6c50fedcd01c63fbc019fc6219725350bdbc0eb357d2958f67a5780ff`.

Main was told this exact artifact is safe for native continuation; no further
Cargo invocation or product write by reviewer while the next native run is active.
This source/core engineering handoff is green. Native acceptance remains pending
main's actual final receipts; original03 and parent completion are not implied.

## Native-driven core repair: full-row hitbox (2026-10-18)

Main supplied a concrete actual-window observation: clicking the blank part of
configuration row only focused its list, while clicking the short label opened
detail. The selected background also stopped at the text-width boundary. This
was a core pointer behavior defect, not a requested size/DPI test.

Fixed `src/app.rs` only: add `.w_full()` to the actual keyed row. Transparent
Kit `.test_support()` observation is attached to the existing row/list elements;
normal non-test-support builds return the exact original element, adding no
layout/semantic node. The meaningful rendered regression measures the actual
first-row/list bounds, clicks the list's far-right blank area at that row's
center, requires correct detail/selection, then checks Back/keyboard navigation,
10k-row visible-range count and stable domain-name selection after reorder.

- Before fix: the regression failed exit101, measured row193px/list1488px.
  `outputs/desktop/shell/check-row-hitbox-before.log` retains that actual failure.
- After the single width correction: identical regression exit0, 1/1 passed.
  `check-row-hitbox-after.log`.
- `just desktop-test-ui`: exit0, real native_shell2 + native_ui2. `check-row-ui.log`.
- `just desktop-lint`, `just desktop-check`, `just desktop-fmt-check`: each exit0.
  `check-row-{lint,type,fmt}.log`.
- `just desktop-build-debug`: exit0, normal1.98 build8.19s. `check-row-build.log`.
- Unchanged domain/root/full-suite/MSRV checks above were not unnecessarily
  repeated; no dependency/lock/runtime/domain change occurred in this repair.

Superseding frozen native artifact: `ccr-desktop/target/debug/ccr-gpui.exe`,
56,220,672 bytes, SHA256
`6e89a0990861055ad2a1c766dd9f0d5bf860f2fdd79bcdfabbf772acffb4886d`.
app.rs SHA256
`1ed22428c796964fcdbc7906df669a19f0e7f006d25826347cd41fdcf5a748f5`.
Lock remains `9c8f5b816b01a2db2285d0092fa223e791ffe3f48f02657ec4edad1708c7216f`.
Main was notified no more Cargo/product writes will occur during its next native
row/error-retry/cancel/close validation.

## Independent consumption of actual native evidence

Read/asserted receipts for `shell-final-core` (PID76196) and `shell-final-probe`
(PID47972): both actualnative_exit_code0, same former f692 artifact hash,
HOME override=false, performance_sample=false and trimmed stderr empty.
Viewed the actual PNGs `final-filter-match`, `final-filter-empty`,
`final-masked-detail-row-hitbox-gap` and `final-favorite-reread` under
`outputs/desktop/shell/native/windows-core/`. They support real hit/empty
filtering, masked detail and stored favorite reread, and independently show the
pre-fix short selected-row background. These screenshots do not establish OS
UIA, actual IME composition or the repaired row's native behavior. Main reports
packet text injection also absent in the unchanged --probe while real key input
works; no speculative Kit editor rewrite was made. IME remains not_run/unverified.
The row-fixed native result still belongs to main's next actual receipt.

## Final independent native evidence conclusion (2026-10-18)

Read `outputs/desktop/shell/native/core-summary.json`, all six actual completed
run receipts and stderr logs, the original fixture-copy manifest and row retry's
fixture-recovery receipt. Independently recomputed every lifetime from exact
RFC3339 strings and matched the summary within its recorded rounding. All six
native_exit_code values are0, each artifact hash matches its run, trimmed stderr
is empty, HOME override=false and performance_sample=false. These are actual
native process receipts, not only launcher exit codes.

| Run | Actual PID | Lifetime seconds | Artifact |
| --- | --- | --- | --- |
| shell-core-normal | 52368 | 129.4004119 | 0efb8185...95758e8 |
| shell-final-probe | 47972 | 60.5915768 | f6926696...4d99af4 |
| shell-final-core | 76196 | 394.0565391 | f6926696...4d99af4 |
| shell-row-retry | 85336 | 71.4847917 | 6e89a099...fb4886d |
| shell-final-cancel | 100824 | 33.8315001 | 6e89a099...fb4886d |
| shell-timed-cancel | 97924 | 26.3727182 | 6e89a099...fb4886d |

`shell-timed-cancel` explicitly requested30000ms artificial fixture read delay;
its whole process lifetime26.3727182s is shorter than that delay, and its actual
screenshot still shows Reading catalog before close. Combined with the source/
deterministic read-drop/shutdown tests, this establishes cancellation/normal
exit during the pending synthetic read. It does **not** measure a UI cancellation
latency budget. The earlier33.8315001s run cannot establish cancellation before
that delay naturally completed, so it remains timing-inconclusive. The hidden
StartProcess attempt has no native run/window/receipt and is not counted pass.

Viewed the requested real PNGs under `native/windows-core/`:

- `row-final-missing-catalog.png`: safe missing-catalog error and Retry, distinct
  from a successful empty list.
- `row-final-retry-recovered.png`: same shell recovered to actual two records
  after synthetic source recovery and Retry, without application restart.
- `row-final-whole-row-opens-detail.png`: repaired full-width selected row,
  correct a-first details and masked credential. Actual main input sequence plus
  the independently failing-then-passing far-right-click regression establish
  the core pointer repair; a screenshot alone is not a click trace.
- `timed-pending-cancel.png`: controlled slow fixture remains in Reading state.
- `final-filter-match.png` / `final-filter-empty.png`: real hit and no-match
  states through actual character keys.
- `final-favorite-saved.png` / `final-favorite-reread.png`: actual favorite persisted
  and remains visible after Refresh. The native sequence did not execute a stored
  command, delete a favorite or clear history; those mutations' confirmation
  paths are covered by the isolated actual-control/domain regressions above.

Independently hashed original `native-fixture/profiles.db`, `config.toml` and
`fixture-manifest.toml`: all match the original copy receipt. The separate
`ui-state.db` now exists as expected for real favorite persistence. Retry copied
only the existing synthetic profiles.db (receipt hash matches original) into its
owned missing source path. Existing profiles WAL0/SHM32768 auxiliary files are
not confused with business changes. A read-only Get-Process query found no
remaining ccr-gpui process after these runs.

One initial evidence assertion returned exit1 because PowerShell7.6 automatically
converted JSON timestamps to DateTime and implicit reparsing dropped fractional
seconds (129.0 vs129.400412). This was an evidence-check parsing issue, not a
native result mismatch. Corrected to `ConvertFrom-Json -DateKind String`, then
recomputed exact DateTimeOffset durations; the full assertion exited0. No product
code, logs, original receipt or summary was changed to make it pass.

**Verified handoff:** 03's source and core engineering dependency is usable for
approved04 implementation: real configuration read/filter/detail/back, safe
error Retry/recovery, full-row pointer activation, command favorite add/reread,
pending-read cancel/shutdown and native exit are backed by actual evidence plus
passing focused/domain/integration checks. No further Cargo/product edits or
retests occurred during this evidence-consumption step.

**Acceptance remains open:** UIA is null/unavailable; OS IME composition was not
observed and is not passed; window-size/zoom/DPI matrix is user-deferred; native
OS appearance/platform/performance and bounded geometry follow-ups remain as
recorded. Task03 stays in_progress for original full acceptance. Neither full03
nor parent AC1-AC8/migration completion is asserted. Main owns task state and the
normal dependency-ordered continuation to04.
