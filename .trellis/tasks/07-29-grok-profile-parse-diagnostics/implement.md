# Implementation Plan

## 1. Shared Parser Diagnostics

- [x] Add focused regression tests in `crates/ccr-config/src/platforms/base.rs` for the live failure shape (`provider_type = "relay"`), malformed TOML, typed secret-field failure, full/simplified compatibility, and empty input.
- [x] Add private helpers for full-vs-simplified shape selection, span-to-line/column conversion, and safe TOML error summarization.
- [x] Refactor `parse_profiles_from_str` to separate syntax parsing from selected-shape deserialization and preserve the relevant typed error.
- [x] Refactor `load_profiles_from_toml` to attach the path once without nesting `CcrError` display text.
- [x] Verify error output never contains sentinel tokens or raw source lines.

## 2. TUI Error Layout

- [x] Update `format_issue` in `crates/ccr-tui/src/tui/app.rs` to render label/value blocks and remove one duplicated path fragment from `What`.
- [x] Extend TUI tests to assert the path occurs once, `Where`/`What` remain distinct under long-path layout, and the actionable parse reason survives.
- [x] Keep the issue paragraph rendering unchanged; `ui.rs` changes are limited to the separately required Grok reasoning-effort detail field.

## 3. Grok Reasoning-Effort Flow

- [x] Extend Grok editable/create/set-field handling with a `reasoning_effort` platform field; trim and normalize the 7 canonical Grok Build enum levels and reject all other values.
- [x] Extend `GrokProfileSummary` and current/list JSON tests with `reasoning_effort` while preserving credential redaction.
- [x] Add Grok runtime helpers that map the field to `model.custom.supports_reasoning_effort`, `model.custom.reasoning_effort`, and `models.default_reasoning_effort` according to official/third-party semantics.
- [x] Extend the entry-state schema with backward-compatible capture/restore of `models.default_reasoning_effort`; cover entry value present, absent, legacy JSON, profile-to-profile switching, `off`, and drift detection.
- [x] Add the existing `ReasoningEffort` detail line to Grok TUI output and cover valid, missing, and invalid stored shapes without displaying secrets.
- [x] Update `examples/grok/profiles.toml` and its byte-identical docs mirror, the Grok runtime example, Chinese/English command docs, and `.trellis/spec/ccr-cli/backend/grok-profile-runtime.md`.

## 4. Validation

- [x] `cargo test -p ccr-config platforms::base::tests -- --test-threads=1`
- [x] `cargo test -p ccr-cli grok -- --test-threads=1`
- [x] `cargo test -p ccr --test commands grok_profile -- --test-threads=1`
- [x] `cargo test -p ccr-tui tui::app::tests`
- [x] `cargo test -p ccr-tui tui::ui::tests::grok -- --test-threads=1`
- [x] `cargo test -p ccr-config -- --test-threads=1` (73 passed)
- [x] `cargo test -p ccr-cli -- --test-threads=1` (252 unit + 11 dispatch passed)
- [x] `cargo test -p ccr-tui -- --test-threads=1` (214 passed)
- [x] `just fmt-check`
- [x] `just lint-strict` (workspace all-target/all-feature Clippy with warnings and production unwrap denied)
- [ ] `just ci` (all stages through CI Governance passed; `TS Bindings Drift` then regenerated unrelated `ccr-ui/src/types/generated/usage/DailyTrendDto.ts` whitespace and failed. The generated drift was restored and excluded from this task.)
- [x] `git diff --check` and final scoped diff review

## 5. Acceptance Evidence

- [x] Capture the fixture error at line 13 and the current real-file diagnostic at line 14; both identify `provider_type` and list `official_relay` / `third_party_model` without exposing rejected values.
- [x] Capture an isolated profile switch proving all three `high` runtime values, then switch/off and prove the entry default is restored without unrelated TOML changes.
- [x] Verify SHA-256 before/after the live diagnostic: the real `profiles.toml` and `~/.grok/config.toml` were unchanged.
- [x] Update the executable Grok runtime, shared parser-diagnostic, and TUI issue-layout specs.
- [ ] Commit the scoped code, tests, spec, and Trellis artifacts only after checks pass.
