# Implementation Plan

## Pre-Implementation Checks

- Confirm the task is reviewed and started with `task.py start` before editing
  production code.
- Re-read the relevant specs if the implementation begins in a later session:
  - `.trellis/spec/ccr/backend/llmusage-provider-adapter.md`
  - `.trellis/spec/ccr/backend/dependency-governance.md`
  - `.trellis/spec/ccr-ui/frontend/api-facade-boundary.md`
  - `.trellis/spec/ccr-types/backend/backend-guidelines.md`
  - `.trellis/spec/ccr-db/backend/backend-guidelines.md`
  - `.trellis/spec/ccr-store/backend/backend-guidelines.md`
  - `.trellis/spec/guides/cross-layer-thinking-guide.md`
  - `.trellis/spec/guides/code-reuse-thinking-guide.md`
- Reconfirm local upstream values from
  `D:\Documents\Code\CLI\llmusage\pricing\static-v1.json` if the local checkout
  changed after task creation.

## Ordered Checklist

1. Update `crates/ccr-types/src/model_rate_catalog.rs`.
   - Add canonical default rows for `claude-fable-5` and `claude-mythos-5`.
   - Add exact alias normalization/matching for short and Anthropic-prefixed
     model IDs.
   - Add unit tests for canonical, alias, OpenCode-style prefix, negative
     non-matches, rate labels, and the `33.95`/`35.0` cost sample.

2. Update `ccr-store` coverage.
   - Assert `PricingConfig::with_claude_defaults()` exposes the canonical rows.
   - Add `ModelPricing::default_pricing()` coverage only if not already implied.

3. Update legacy `ccr-db` coverage.
   - Add parse/import pricing coverage in
     `crates/ccr-db/src/services/usage_import_service.rs`.
   - Extend migration v13 tests in
     `crates/ccr-db/src/database/migrations.rs`.
   - Add `usage_repo::get_model_stats` rate-summary coverage if the existing
     fixture can absorb it with little noise.

4. Update read-only projection/Tauri coverage.
   - Seed `claude-fable-5` and/or `claude-mythos-5` buckets with
     `pricing_status = static`, `pricing_source = static-v1`,
     `pricing_rate = 10/1/50`.
   - Assert `usage_by_model` or dashboard response preserves the upstream
     stored fields and costs.
   - Keep SQL in `crates/ccr-usage`; do not duplicate queries in Tauri.

5. Update frontend smoke coverage only if needed.
   - Prefer reusing `tests/usage-models-tab.smoke.test.ts` fixtures.
   - Assert `static` status and long model names are display-safe.
   - Do not add new direct `invoke()` wrappers to `src/api/tauri.ts`.

6. Inspect install/version surfaces.
   - If a soft note is clearly useful, keep it diagnostic/documentary.
   - Do not introduce a hard minimum-version failure without explicit product
     approval.

7. Review for architecture drift.
   - Run a search proving no upstream crate dependency or import was added.
   - Run a search proving usage SQL still lives in `crates/ccr-usage`.

## Validation Commands

Run the narrow checks first, then escalate if touched files require it:

```powershell
just fmt-check
cargo test -p ccr-types -- --test-threads=1
cargo test -p ccr-store -- --test-threads=1
cargo test -p ccr-db -- --test-threads=1
cargo test -p ccr-usage
cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml llmusage_adapter -- --nocapture
cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml --test llmusage_no_crate_guard -- --nocapture
```

If frontend smoke fixtures change:

```powershell
cd ccr-ui
bun run test:smoke -- tests/usage-models-tab.smoke.test.ts
bun run test:smoke -- tests/api-facade-boundary.smoke.test.ts
bun run type-check
```

For broad or release-ready changes:

```powershell
just frontend-check-quick
just ci
```

## Risky Files And Rollback Points

- `crates/ccr-types/src/model_rate_catalog.rs`
  - Risk: overbroad alias matching can price unrelated models. Roll back by
    reverting alias normalization and keep positive/negative tests paired.
- `crates/ccr-db/src/database/migrations.rs`
  - Risk: changing migration behavior instead of test data can affect existing
    user databases. Prefer test-only additions unless a real bug is found.
- `ccr-ui/src-tauri/src/services/usage.rs`
  - Risk: service tests may accidentally assert recalculation. They should
    assert pass-through from fixture rows.
- `crates/ccr-cli/src/services/install_*`
  - Risk: hard version gates can block valid local dashboards. Keep any change
    soft unless explicitly approved.

## Done Review

- Pricing constants match local upstream and official docs as of 2026-07-06.
- Canonical and alias model ids are covered.
- Negative alias tests protect against substring pricing.
- Desktop read-only projection path passes through upstream `static-v1` data.
- No upstream `llmusage` crate dependency/import exists.
- Verification results are recorded before task finish.
