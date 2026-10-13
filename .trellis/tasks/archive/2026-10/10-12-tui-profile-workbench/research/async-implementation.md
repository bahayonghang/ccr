# Profile background implementation evidence

Implemented after candidate A approval. Scope: `tui/app.rs`, `profile_tasks.rs`, `runtime.rs`, `event.rs`, `mod.rs`, and the existing `usage/app.rs` source list. No lifecycle, persistence or usage SQL was copied into the TUI.

## Behavior and boundaries

- App construction creates a lightweight six-tab shell, then submits only the configured first Profile tab. Other Profile tabs load on activation. Platform construction, config/current/runtime reads and the batched activity facade execute in a blocking worker. Missing/failed runtime reads remain visible rather than becoming an active profile.
- ProfileTasks owns at most one read per platform and one mutation globally. Repeated reloads coalesce into one next read. Starting a write advances that platform's read generation; obsolete pre-write snapshots are discarded even if they finish after the mutation. A fresh read is queued for the affected platform only. Mutation results are never discarded by read generations.
- Apply freezes platform, profile and operation ID; Off freezes platform. The existing complete synchronous owner executes inside one `spawn_blocking` closure, including lock, thread-local journal, writes, verification and ancillary work. No off-then-apply composition or automatic write replay was added.
- Profile selection, pagination, resize and language state remain responsive during writes. Auth tab entry is gated. Duplicate Apply/Off is rejected. q, Esc and Ctrl+C retain exit intent until the result is consumed; Enter/Space continue applying without exiting.
- Read failures retain the previous list with an explicit error. Current markers are cleared when current-state lookup fails. Initialized Codex/Claude Auth runtime summaries are invalidated at write submission and replaced from the fresh typed read; the Auth app and account selection are retained.
- Grok uses `inspect_activation_state`: only `Active` sets an effective current marker. Drifted and unsafe missing-entry states reach the view as their typed state. Existing UsageApp now requests Grok alongside Claude/Codex through the same shared source-tagged projection and single-flight cache.
- Runtime drains Profile/usage results after every event, including Ctrl+T and ignored mouse motion. Pending waits are capped at 50ms. Tick-based Auth gates retain their 250ms elapsed-clock cadence; frequent input cannot accelerate or starve them. Idle events request no draw when nothing changed. Relative history labels schedule their next actual label boundary; old absolute dates have no periodic redraw.
- Mouse profile hit testing consumes the exact table body rectangle supplied by the visual implementation. Header hit testing now consumes the visual module's actual label/padding/divider geometry instead of equal-width guesses.

## Controlled measurements

Baseline revision: `8115cd116632ae2d897401b1de4e8385d9b19d6b`. Same machine and Cargo test profile before/after: Windows NT 10.0.26300, 24 logical processors, Rust 1.98.0 (`88d9e12ae`, 2026-08-18), `profile.test` inherits dev (`opt-level=0`, dependencies `opt-level=1`, line-table debug info). CPU model lookup was unavailable in the sandbox. These measurements concern worker scheduling and injected delays; they are not filesystem throughput or actual terminal flush timings.

| Fixture and samples | p50 | p95 | Work count |
|---|---:|---:|---:|
| Before: synchronous reload, 3 fake platforms, 10ms delayed load each, 30 samples | 30.896ms | 31.524ms | 90 platform reads |
| After: current-platform reload submission, same 10ms load delay, 30 samples | 0.013ms | 0.029ms | 30 platform reads |
| After: current-platform data ready including worker delay and 1ms test polling, 30 samples | 11.226ms | 12.089ms | same 30 reads |
| After: Apply submission with a 1s delayed writer, 30 samples | 0.022ms | 0.078ms | exactly 30 writes |
| After: writer-finished to result-consumed with 50ms polling, 30 samples | 6.643ms | 7.771ms | same 30 writes |
| After: whole 1s synthetic operation to result-consumed, 30 samples | 1006.913ms | 1008.276ms | same 30 writes |

The one-second fixture retains 1,000 profiles in every sample and handles 1,000 selection actions while each writer is blocked (30,000 events). The observed selection p50/p95 was 100ns/100ns, at the clock's measurement granularity; treat that only as evidence that these in-memory operations are below useful microbenchmark resolution, not an end-to-end latency claim. Resize page capacity and language state also changed while pending. No wall-clock threshold was added to ordinary CI.

Baseline was recorded before production edits using the original synchronous `App::reload_profiles` and three injected `PlatformConfig` backends. Their fake platform identity avoided auth-summary discovery, and each `load_profiles` only slept 10ms and returned an empty fixture map. The temporary baseline test was removed after measurement; reproducible after-change harnesses are ignored tests `profile_async_responsiveness_benchmark` and `profile_one_second_writer_responsiveness_benchmark`. No actual user profile application, personal history database, account or remote service was exercised by these benchmarks. Real durable transaction performance is not claimed faster; its semantics remain unchanged.

## Verification

- `cargo test -p ccr-tui profile_tasks -- --skip export_bindings`: 4 passed, 1 timing harness ignored, before final additional tests.
- `cargo test -p ccr-tui profile_ -- --skip export_bindings`: 60 passed, 3 ignored; includes the existing actual lifecycle success/replay, preflight, ancillary-warning, each-write failure, post-publication failure and external-version contracts.
- `cargo test -p ccr-tui -- --skip export_bindings`: 276 passed, 3 ignored, 35.59s at the integrated revision before final mouse/timer fixture refinements. Main integration gate owns the final full-scope rerun.
- Final focused `cargo test -p ccr-tui tui::app::tests::profile_ -- --skip export_bindings`: 8 passed, 1 timing harness ignored. Covers active-first/lazy loading; no I/O from 1,000 navigation/scroll events; no draw or reload from 1,000 settled ticks; first-current focus after switching away during loading; frozen write identity/duplicate suppression; all three deferred quit keys; Auth cache update without selection reset; read-error retention; exact relative-label timer boundaries.
- Runtime/event tests cover completion pumping through continuous key/resize/Ctrl+T events, deferred exit without another key, 50ms pending cap and unchanged idle interval, and elapsed-clock Auth tick cadence. They passed in the full TUI run.
- Both ignored timing harnesses passed explicitly with `--ignored --nocapture --skip export_bindings`.
- Final `cargo clippy -p ccr-tui --all-targets --all-features -- -D warnings`: passed.
- Owned Rust files formatted with `rustfmt --edition 2024`; scoped `git diff --check`: passed (only repository line-ending notices).

Cargo initially could not open `target/debug/.cargo-build-lock` under sandbox permissions. The same scoped commands succeeded with approved build-cache access; no cache deletion or permission rewrite was used.

## Remaining evidence limits

Native PTY/terminal flush and first-visible-frame timing were not measured here. Store query sizes/plans are owned by the activity worker; render timings and actual TestBackend cell artifacts are owned by the visual worker. The final repo gate and spec/task synchronization belong to the main session. No commit or release was made.
