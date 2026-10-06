# Domain recovery implementation evidence

Date: 2026-10-06

Scope: M1–M3 implementation owned by `crates/ccr-codex/**` and
`crates/ccr-types/src/model_rate_catalog.rs`.

Implemented contracts:

- Usage records now retain cache read, cache write, reasoning, request and
  session metadata, measurement basis, attribution hints, and scan diagnostics.
- Cumulative token events use epoch-aware deltas. Completed responses are
  selected by response or turn identity. Duplicate source copies are removed
  by event identity. Fork records before a known fork boundary are excluded;
  unknown fork boundaries remain partial.
- Rolling statistics use a fixed `as_of`. Future events and records without an
  event timestamp are excluded from rolling totals. Explicit zero token
  classifications remain distinct from missing classifications.
- Pricing uses the CCR model catalog per record. `gpt-6.1-sol` uses the
  verified Standard rates and the 272000-token request boundary. Unknown
  models remain unpriced. Missing request classification or tier remains an
  explicit assumption and cannot produce a complete calibration sample.
- Quota results retain network acquisition time and cache-hit provenance.
  Successful network results add only whitelisted account, plan, bucket,
  window, percentage, reset, and scan metadata to the bounded private
  observation file. Cache hits add no sample. Corrupt or failed history writes
  preserve the previous file and attach a warning to the successful quota.
- Capacity estimation uses aligned scan watermarks, stable bucket/reset
  generations, three non-overlapping intervals with at least five percentage
  points, empirical median/min/max, freshness, and independent Token/USD
  validity. Global fallback records do not enter account calibration.

Checks run:

- `cargo check -p ccr-codex --all-features` — PASS.
- `cargo clippy -p ccr-codex --all-features -- -D warnings` — PASS.
- `cargo test -p ccr-codex --all-features -- --skip export_bindings` — PASS,
  225 passed and 1 ignored.
- `cargo test -p ccr-types --all-features -- --skip export_bindings` — PASS,
  47 passed.
- `cargo check -p ccr-tui --all-features` — PASS after the dependent TUI
  changes were present.

Remaining verification:

- Independent `trellis-check`, repository `just ci`, and isolated native TUI
  checks remain pending.
- No real account, personal auth file, session directory, quota endpoint, or
  llmusage database was accessed.
