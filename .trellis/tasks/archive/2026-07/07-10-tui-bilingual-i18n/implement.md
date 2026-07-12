# TUI Bilingual Language Switching Implementation Plan

## Review Gate

- [x] Resolve the open MVP coverage question in `prd.md`: complete TUI.
- [x] Confirm `Ctrl+L` as the global immediate switch and English as the
  fallback.
- [x] Review the final PRD/design/implementation artifacts before
  `task.py start`.

## Implementation Checklist

1. [x] Extend `ccr-config` TUI preferences.
   - Add and export the typed language value.
   - Add the backward-compatible defaulted language field.
   - Make invalid language fallback independent from valid tab ordering.
   - Add a guarded `TuiConfigManager::save` path.
   - Cover missing, English, Chinese, unknown, round-trip, and tab-order
     preservation cases with `TestCcrEnv`.
2. [x] Add the `ccr-tui` localization foundation.
   - Add the English/Simplified Chinese catalog and typed interpolation helpers.
   - Add English fallback and catalog-completeness tests.
   - Add a deterministic test strategy for active-language state.
3. [x] Wire startup and switching.
   - Load language with the existing tab-order config.
   - Handle `Ctrl+L` before tab-specific key dispatch.
   - Redraw immediately and persist the full config without losing tab order.
   - Surface save failures as localized, recoverable feedback.
4. [x] Extract user-visible strings for the approved MVP scope.
   - Main profile tabs and details, usage/status text, footers, and placeholders.
   - Embedded Claude/Codex/OpenCode auth apps and renderers.
   - Shared overlays, toasts, and post-exit summaries.
   - Preserve raw dynamic identifiers and lower-layer error details.
5. [x] Add regression coverage.
   - English default and first-frame rendering.
   - Chinese rendering for representative compact/standard/wide states.
   - Immediate switching without state loss on profile and auth tabs.
   - Restart persistence and save-failure behavior.
   - CJK width/truncation assertions for translated labels and footer content.
6. [x] Update the durable ccr-config and ccr-tui specs with the final language,
   persistence, fallback, and global-key contracts.

## Verification

Run from the repository root, escalating only after targeted failures are
resolved:

```powershell
cargo test -p ccr-config -- --test-threads=1
cargo test -p ccr-tui -- --test-threads=1
cargo test -p ccr -- --test-threads=1
just fmt-check
just lint-strict
```

Use Ratatui `TestBackend` render assertions for both languages and compact,
standard, and wide viewports. Run `just ci` only for final cross-workspace
acceptance or release readiness.

## Verification Results

- `cargo test -p ccr-config -- --test-threads=1` — 61 passed.
- `cargo test -p ccr-tui -- --test-threads=1` — 195 passed.
- `cargo test -p ccr -- --test-threads=1` — passed.
- `just version-check` — passed.
- `just fmt-check` — passed.
- `just lint-strict` — passed.
- `just ci` — passed all repository gates in 6m 51s.
- `git diff --check -- . ':(exclude)TODO.md'` — passed; `TODO.md` is a
  pre-existing user change and remains untouched.

## Risk And Rollback Points

- `crates/ccr-config/src/managers/tui_config.rs`: a deserialization change can
  accidentally reset custom tab ordering; preserve it with explicit tests.
- `crates/ccr-tui/src/tui/runtime.rs` and `app.rs`: global key handling must run
  before embedded app delegation without shadowing existing shortcuts.
- Large UI files under `crates/ccr-tui/src/tui/**/ui.rs`: translated CJK labels
  can overflow layouts; retain unicode-width truncation and viewport tests.
- Locale state shared by render helpers can leak between parallel tests; use a
  serialized guard or explicit locale injection in tests.
- Commit configuration contracts separately from broad catalog extraction to
  provide a clean rollback boundary.
