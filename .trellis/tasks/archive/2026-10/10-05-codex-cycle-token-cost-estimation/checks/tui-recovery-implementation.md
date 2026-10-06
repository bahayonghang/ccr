# TUI recovery implementation

Date: 2026-10-06

## Scope

This record covers the recovery of the Codex Auth TUI implementation after the
first TUI implementer stopped with a partial DTO migration. Product ownership
remained limited to `crates/ccr-tui/src/tui/codex_auth/{app,ui}.rs` and the
direct Codex composition regression in `crates/ccr-tui/src/tui/ui.rs`.

## Changes verified

- `CodexUsageDataset` now carries the domain `CodexAuthUsageSnapshot`.
- Usage loading receives `CodexAuthService::usage_paths()` and runs in the
  blocking loader. The TUI does not read the user home directly.
- Usage messages carry account name and generation. A late result is ignored.
  A newer request is queued while an older request is still running.
- Display uses `CodexAuthUsageSnapshot::for_display`, so an expired estimate
  keeps its history while remaining capacity becomes `N/A`.
- Compact and wide composition preserve quota, local token totals, record
  counts, API equivalent cost status, attribution scope, and estimate status.
  Wide composition includes classification, price basis, capacity ranges, and
  sample span. English and Simplified Chinese use the existing TUI catalog.
- Test fixtures use synthetic records and injected directories. No personal
  auth, session, usage cache, or quota endpoint was read.

## Checks

| Command | Result |
| --- | --- |
| `cargo check -p ccr-tui --all-features` | PASS |
| `cargo test -p ccr-tui tui::codex_auth::ui::tests:: --all-features -- --skip export_bindings` | PASS (19 tests) |
| `cargo test -p ccr-tui tui::codex_auth::app::tests:: --all-features -- --skip export_bindings` | PASS (16 tests) |
| `cargo test -p ccr-tui tui::ui::tests::codex_auth_composed_layout_matrix_preserves_scope_quota_and_errors --all-features -- --skip export_bindings` | PASS (1 test) |
| full `cargo test -p ccr-tui --all-features -- --skip export_bindings` | PASS (245 tests) |

The focused UI suite covered 19 tests. The focused app suite covered 16 tests.
The existing composed matrix test was updated to accept the additional price
basis occurrence of the visible token total while retaining the requirement of
at least three visible totals. The complete package suite passed after the
shared Cargo process released the test executable.

## Follow-up

Continue with the independent Trellis check and the repository `just ci` gate.
Native terminal verification remains a separate evidence boundary.
