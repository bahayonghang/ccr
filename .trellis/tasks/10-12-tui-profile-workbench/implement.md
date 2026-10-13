# Execution plan

Status: implementation and independent review complete; all standard CI recipes verified (aggregate plus resumed frontend suffix). The user approved revision-2 candidate A for all three platforms. Shared files had fixed owners; platform presentations were integrated sequentially.

## Preconditions

- [x] Final review confirms last-use = recorded successful activation, platform name fallback, runtime scope and safe async-exit behavior.
- [x] Read `trellis-before-dev`, curated implement/check manifests and task research. Preserve pre-existing build-cache/journal changes.
- [x] Record baseline revision, Rust/build mode, viewport/theme/locale and fixture sizes. No profile writes against user home directories.

## Ordered work

1. Baseline and fixtures
   - [x] Reuse TestBackend, injected loaders and lifecycle fixture seams; generate 25/1000 profiles, history at 0/10k/100k rows, and absent/malformed data fixtures.
   - [x] Measure explicit reload acknowledgement/data readiness, selection/render, Apply submission/completion and warm-cache history queries with documented fixtures and sample counts. See the three implementation reports.
   - [ ] Remaining measurement limits: native startup-to-shell/first-frame/flush, cold-disk queries and real durable-write before/after timings were not measured; no performance claim is made for them.
   - [x] Record query/read counts and confirm which suspected costs matter. Do not change code just to satisfy a speculative bottleneck.
2. Read model
   - [x] Add ccr-store read-only, batched activation projection with explicit platform/name key and strict timestamp parsing; ccr-cli facade and typed unavailable/error state.
   - [x] Return last recorded successful activation and bounded recent timestamps; select no credential fields; missing DB cannot be created by this read.
   - [x] Tests: no path/database creation/migration, same-name platforms, timezone offsets, invalid/future/missing timestamps, legacy unattributed rows, failed operation, retention, schema unavailable.
3. Background Profile operations
   - [x] Lightweight App shell; active tab background load, lazy inactive Profile tabs, existing runtime executor/channel.
   - [x] One in-flight Apply/Off, original platform/profile/operation identity, stale-read suppression, queued q/Esc/Ctrl+C exit; prevent entry to Auth tabs while a Profile write is pending.
   - [x] Preserve Enter/Space staying in TUI; drain messages after every event and timeout, not just idle ticks; pending poll cap 50 ms, no continuous redraw.
   - [x] Add runtime exit-state check separate from on_tick's redraw bool; test deferred quit exits on completion without another key and Ctrl+T cannot skip pumping.
   - [x] Invalidate/update matching initialized Auth runtime summaries after Profile mutation; entering Codex Auth must show fresh runtime while preserving account selection.
   - [x] Refresh only affected state after mutation; preserve current marker, selected name, per-tab snapshot, failure/recovery/warning outcomes.
   - [x] Delayed-loader/worker tests prove input remains responsive, repeat action does not write twice, platform switches do not reroute results, pending exit drains write.
4. Shared visual primitives
   - [x] Shared table header/body geometry, capacity/hit testing, column budget, display-width formatting and aligned time column.
   - [x] Runtime strips, Focus identity summary, thin guides, localized semantic styles, full-row selection.
   - [x] Implement the revision-2 candidate-A right panel: compact connection rows, provider totals and aligned category bars/counts, then vertical activation history. No repeated Last used/Recorded at/UTC-offset fields or healthy-state unset summaries.
   - [x] Bars use cached independent category counts and their common maximum; preserve zero/unknown and partial-cell magnitudes, with no new query or sum-to-100% assumption. Narrow layouts keep numeric rows and scope; measured labels and long values reflow.
   - [x] Correct wrapped scroll extent; no hidden final detail lines; no duplicate shortcut hints.
   - [x] TUI-local brand presentation removes generic emoji without modifying global Platform::icon consumers.
5. Platform children, sequential
   - [x] Execute Codex child using shared layouts; verify all runtime/auth mode cases.
   - [x] Execute Claude child with exact title and official/API-key distinction.
   - [x] Execute Grok child; extend existing UsageApp source/platform mapping for Grok.
   - [x] Never dispatch simultaneous edits to ui.rs/app.rs; use fixed ownership and concrete acceptance per child.
6. Measured render/query improvement
   - [x] Re-measure the final renderer. Its richer layout is slower than baseline but remains below 1 ms p95 in the documented debug fixture; no additional cache or speculative layout optimization was justified.
   - [x] Preserve existing idle redraw suppression; prove continuous key input cannot starve worker completion and toast/time-label timers still update.
   - [x] Preserve default test parallelism. Do not add timing-sensitive microbenchmark pass/fail assertions to ordinary CI; deterministic job/query counters prove the main contracts.
7. Verification and integration review
   - [x] Capture actual TestBackend cells and colored cell-derived images for EN/ZH, Mocha/Latte, all three profiles and the viewport/state matrix. Include revision-2 long model/endpoint, 74-cell rail, 60×20 short rail, empty usage/history and zero/unknown category cases; short views must scroll to the final line.
   - [ ] Native-terminal/font evidence remains unverified; cell-derived images are labeled accordingly and are not native screenshots.
   - [x] Report geometry before/after in cells, checked consumers and remaining visual gaps. Design mockup is not implementation evidence.
   - [x] Re-run delayed mutation, lifecycle compensation/replay and secret-sentinel tests on actual adapter paths.
   - [x] Update impacted TUI/lifecycle specs for shared table, readonly activity and new async adapter contracts; preserve unrelated spec content.
   - [x] Trellis check sub-agent reviews final implementation with curated context; main session resolves findings and reports measured results.

## Validation commands

Run narrow checks as work lands; only final integrated checks need broad scope.

```powershell
just version-check
just fmt-check
cargo test -p ccr-store -- --skip export_bindings
cargo test -p ccr-cli -- --skip export_bindings
cargo test -p ccr-usage -- --skip export_bindings
cargo test -p ccr-tui -- --skip export_bindings
cargo test -p ccr-config -- --skip export_bindings
just lint-strict
just test
just ci
```

`ccr-config` is a compatibility regression suite, not permission to change its schema. Full `just ci` is final acceptance because the planned change crosses crates. Local specs containing `--test-threads=1` are superseded by root AGENTS.md's default-parallelism contract. Preserve export binding separation and existing coverage thresholds. Do not run full gates repeatedly without new changes/failures.

- [x] Final integrated CI acceptance: all 16 standard recipes passed. `just ci` passed its first 12 recipes and stopped on existing Tailwind lint grammar; the reviewed configuration repair was verified by the complete `just frontend-check frontend-audit frontend-coverage vscode-ci` suffix (exit 0). See the integration report; the initial aggregate exit remains recorded as a failure.

## Rollback and delivery

- Pure read-model changes require no migration rollback; never rewrite history during display.
- Revert background scheduling separately from lifecycle internals; lifecycle correctness is not optional for performance.
- Revert layout independently while keeping typed fields, if necessary.
- Revision-2 candidate A is implemented, independently reviewed and visually checked with actual renderer cells. All standard CI recipes have passed; see `research/integration-validation.md`.
- No automatic commit/push/release. Final report states changed files, test results, measured responsiveness, known legacy-history and terminal-icon limits.
