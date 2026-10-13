# Grok execution plan

1. [x] Confirm approved plan and shared foundation; load Grok runtime, provider projection and fixture specs.
2. [x] Add Grok source to existing usage loader/platform mapping and detail rendering with capability/error tests.
3. [x] Implement Grok field grouping/runtime evidence using shared UI and activity snapshots; preserve inspect_activation_state's Inactive/Active/Drifted/UnsafeMissingEntryState and separate login validity; add distinct paired identity token.
4. [x] Test inline/env/session, malformed fields, sanitized URL, secret sentinel, same-provider cross-source rows, missing provider/schema/DB and successful no-record state.
5. [x] Verify no per-selection work and single-flight refresh, apply/off outcomes, pending exit and Profile/Auth guard.
6. [x] Run `cargo test -p ccr-tui -- --skip export_bindings`, `cargo test -p ccr-usage -- --skip export_bindings`, affected `cargo test -p ccr-cli -- --skip export_bindings`, `just fmt-check`, `just lint-strict`.
7. [x] Capture real renders under parent size/locale/theme matrix; Trellis check review then parent integrated acceptance.

No private session reads/tests or new collector. Rollback is source-filter/view wiring and platform presentation; existing domain safety remains intact.

Evidence: parent `research/integration-validation.md` and `research/check-final.md`. Workspace tests and strict lint passed; all standard CI recipes passed as recorded by the parent. Native terminal appearance is not established by TestBackend exports.
