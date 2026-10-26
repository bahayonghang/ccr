# 02 runtime / native read chain implementation evidence

Date: 2026-10-18. Role: services_runtime, dispatched trellis-implement. Scope is 02 only; this is not parent migration acceptance. Shared workspace changes from other owners were retained. Cargo manifests/locks/config/domain/Tauri files are owned by services_domain/check and are not claimed here.

## Implemented boundary

- `crates/ccr-app/src/runtime.rs` and `runtime/tests.rs`: one long-lived Tokio owner, typed environment/domain Platform/generation/request key, current-response gate, 128 admitted operations and four concurrent blocking workers. Each subscription has a bounded 128 notification queue. Progress replaces one pending value and is published at intervals of at least 100ms. Authoritative immutable terminal is retained independently of the progress queue, prioritized by `next()`, and wakes readers even when the queue is full. Logs are not transported in 02; no unused log buffer framework was introduced.
- Pure reads cancel and close their receiver on subscription drop. Already-started blocking I/O is joined before a cancelled terminal appears. Application-owned jobs detach on subscription drop and return their own outcome after cleanup; cancellation cannot rewrite a terminal. This is a lifecycle primitive, not the complete command/usage job migration or registry from 08.
- `ccr-desktop/src/runtime/mod.rs`: explicit local/fixture read source, service creation and actual domain list on the worker, worker-side name filtering. Non-Local or non-Claude requests return typed unsupported errors and never fall back to local data. Domain errors remain typed; UI text never formats raw paths/parser/credential payloads.
- `ccr-desktop/src/app.rs`: real read-only Claude list, stable name-derived IDs, GPUI uniform-list virtualization, loading/empty/error/retry, real filter input, local notifications, list keyboard selection and masked detail. A completed result installs an Arc rather than cloning all rows on the UI thread. There is no placeholder navigation or mock data.
- `ccr-desktop/src/lib.rs`: default configuration view, retained `--probe` input view, runtime lifetime outside the event loop, explicit quit cancellation/join. GPUI's exact-version `SHUTDOWN_TIMEOUT` is 200ms (`gpui-pre-0.3.9/src/app.rs:79`), so hook completion is not the sole cleanup proof: after `Application::run` returns the outer owner waits for all runners. An uncooperative blocking domain read can delay process return; no false cancellation terminal is synthesized. A visible full exit flow belongs to 09.
- `ccr-desktop/src/theme.rs`: only four existing DESIGN tokens added (36px rows, 12px metadata, 8px gap, 4px selection radius). The complete shell/theme/IME/accessibility contract remains 03.
- `ccr-desktop/tests/native_ui.rs`: retained probe test plus actual Kit filter/loading/disabled-refresh/window-close behavior, driven by barriers and the real Tokio worker pool. Read generation is observable for action/runtime evidence; clicking disabled refresh does not advance it.
- `ccr-desktop/README.md`: describes the actual default read behavior, explicit safe fixture flags, controlled delay and native test actions. It does not present the probe as business completion.

## Actual commands and results

Environment: Windows x86_64 MSVC; `rustc 1.98.0 (88d9e12ae 2026-08-18)` / `cargo 1.98.0 (797e8a9bc 2026-08-05)`. All behavioral Rust tests retain default parallelism and `--skip export_bindings`.

| Command | Actual result |
| --- | --- |
| `cargo test -p ccr-app --locked --offline runtime:: -- --skip export_bindings` | 0, initially 8 tests, final 9 tests after actual late-worker-result test added; 0 failed, measured test duration 0.00s. Six config integration tests were filtered out by this command; their owner runs them separately. |
| `cargo clippy -p ccr-app --locked --offline --all-targets -- -D warnings` | 0, includes runtime tests. |
| `just desktop-check` | First code run 101: missing explicit Range type, focus App argument and borrowed scroll handle. Fixed all three. Next run 0. Final-lock rerun 0, all targets, build time 39.00s. |
| `just desktop-lint` | Final-lock rerun 0, all targets and `-D warnings`, 3.56s. |
| `just desktop-test` | First run 101 at an incorrect accessibility assertion, after Python 6 tests and desktop unit 2 tests passed. See failure analysis below. Final full rerun 0: Python 6 tests (41.841s), Rust 3 unit tests including remote fail-closed, 2 real Kit headless tests, no failures; Rust test build 21.01s. |
| `just desktop-test-ui` | Final behavior revision 0, 2 real Kit headless tests passed in 0.03s, including disabled refresh not advancing generation, Chinese input/backspace/focus, close and zero runtime tasks. OS screenshots/IME are separate main-session evidence. |
| `just desktop-build-debug` | 0, linked native binary using final lock. No window was started by this agent. |
| `just desktop-fmt-check` | 0, independent Cargo format validation, no implicit repair. |
| `rustfmt --edition 2024 --check crates/ccr-app/src/runtime.rs ccr-desktop/src/app.rs ccr-desktop/src/runtime/mod.rs ccr-desktop/src/lib.rs ccr-desktop/src/theme.rs ccr-desktop/tests/native_ui.rs` | 0 at checkpoint; explicit formatting repairs were separate commands, not hidden in validation. |
| `git diff --check -- crates/ccr-app ccr-desktop` | 0 for tracked diffs; these new directories remain untracked, so Cargo/rustfmt checks above provide the actual new-source checks. No commit was made. |

Sandboxed cargo calls initially returned 1 due to pre-existing mbx cache/temp locations outside writable roots. The same authorized compile/test commands were rerun with cache access approved and then executed normally. This was an execution boundary, not evidence of a product test passing or failing. No global tools were installed and no HOME/user configuration was changed.

An incorrect `cargo fmt --manifest-path crates/ccr-app/Cargo.toml -- src/runtime.rs src/runtime/tests.rs` invocation returned 1 because rustfmt paths were relative to the repository cwd. Corrected to explicit `rustfmt --edition 2024` paths. This was not reported as a successful formatting check.

## Failures retained and resolved

1. Initial headless test asserted `window.find("configuration-refresh").disabled() == Some(true)`. Actual value was `None`, test failed. Exact Kit sources confirm `gpui-base-0.7.3/src/button.rs:235-253` suppress click/focus when disabled but do not set AccessKit Node disabled. The test now checks the real loading accessibility label and clicks the disabled control, asserting read generation does not change. **Native disabled semantics remain unverified/missing for 03 AC3.2**; absence is never treated as true. No upstream code was patched or dependency silently changed.
2. Explicitly draining GPUI test tasks after view drop first failed with `Detected activity on thread Some("ccr-app") ... Your test is not deterministic`. This is a real mixed-executor integration, so it now calls the exact-version supported `cx.executor().allow_parking()` before crossing into actual Tokio OS workers. Work order is still controlled with barriers/channels; no sleep-based timing assertion was added. Shared-runtime algorithm tests remain Tokio virtual time/barrier tests. This does not claim GPUI test clock controls external threads.

## Native handoff

Main session generated the domain-owned fixture at `outputs/desktop/services/native-fixture` with synthetic credentials only; fixture manifest retains hashes/revision. This agent did not create a parallel persistence format or copy personal data.

Native executable checkpoint:

- Path: `ccr-desktop/target/debug/ccr-gpui.exe`
- SHA256: `8F29294241FFB0FBC9B6A26D551C9B8ABB424BDFFF6462185C00159BD3EC8E38`
- Size: 54,440,448 bytes
- Cargo.lock SHA256: `7D6568D91B9EF1944FB92403C4242C0F9152F6A8D69D5693BB1A4BDEE3BDC560`

From repository cwd:

```text
ccr-desktop/target/debug/ccr-gpui.exe --fixture-profiles outputs/desktop/services/native-fixture/profiles.toml --fixture-registry outputs/desktop/services/native-fixture/config.toml
ccr-desktop/target/debug/ccr-gpui.exe --fixture-profiles outputs/desktop/services/native-fixture/profiles.toml --fixture-registry outputs/desktop/services/native-fixture/config.toml --fixture-read-delay-ms 10000
ccr-desktop/target/debug/ccr-gpui.exe --probe
```

Native window operations/screenshots are reserved to the main session. Test IDs: `configuration-filter`, `configuration-refresh`; row IDs use `configuration:<domain-name>`. Fixture UI is marked Demo data; delay additionally marks Controlled slow read. Delay only accepts paired explicit fixture paths and 0..30000ms, so it cannot slow a real user source. Invalid source options fail before any window or runtime starts.

## Not run / not claimed

No actual native window, screenshots, native IME, screen-reader acceptance, DPI/window-size matrix, macOS/Linux build/run, Tauri comparison baseline, full app performance, packaging, signing, publishing, commit, push or task archive was performed by this agent. Main/check own cross-surface regression gates and inventory/task state. Neither 02 nor the complete migration is declared accepted by this file.
