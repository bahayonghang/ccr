# Design

## Architecture

CCR has two relevant usage-pricing paths and they should remain separate.

1. Embedded CCR catalog for legacy/archive/config defaults:
   `crates/ccr-types/src/model_rate_catalog.rs` is the single pricing rule
   owner. `ccr-store` and legacy `ccr-db` consumers should inherit Fable/Mythos
   behavior from this catalog rather than copy pricing constants.
2. Installed `llmusage` projection for desktop Usage Dashboard:
   `ccr-ui` invokes the installed CLI and reads `~/.llmusage/llmusage.db`
   read-only through `crates/ccr-usage`. It must not recalculate upstream row
   pricing and must not link the upstream crate.

This task should therefore add embedded pricing only where CCR already owns
pricing, and add projection-flow tests where `llmusage` already owns pricing.

## Data Flow

### Legacy CCR Pricing

Raw model id
-> `normalize_model_id`
-> `official_rate`
-> `PricingComputation`
-> legacy import/migration/repository/model stats
-> optional `ccr-store` pricing defaults.

Required model aliases:

- Canonical: `claude-fable-5`, `claude-mythos-5`
- Short: `fable-5`, `mythos-5`
- Provider-prefixed: `anthropic/claude-fable-5`,
  `anthropic.claude-fable-5`, `anthropic-claude-fable-5`, and matching Mythos
  variants when they appear in OpenCode/Anthropic logs.

Normalization should be exact enough to avoid broad substring matches. A good
shape is to strip known provider prefixes first, then match exact normalized
model ids.

### Desktop Usage Dashboard

Installed `llmusage sync`
-> `llmusage.db` stores bucket rows with model, costs, `pricing_status`,
`pricing_source`, `pricing_rate`
-> `crates/ccr-usage::Dashboard::model_breakdown`
-> `ccr-ui/src-tauri/src/llmusage_adapter/queries.rs::to_model_stats`
-> generated TS type / `UsageModelsTab.vue`.

For this path CCR should pass through:

- `model = claude-fable-5` or `claude-mythos-5`
- `pricing_status = static`
- `pricing_source = static-v1`
- `pricing_rate = 10/1/50`
- stored cache-aware/cache-free costs.

## Pricing Contract

Use the local upstream `llmusage` and official Claude docs values:

- input: `10.0` USD / million tokens
- cache read: `1.0` USD / million tokens
- cache creation: `12.5` USD / million tokens
- output: `50.0` USD / million tokens
- context window: `1_000_000` tokens, documented in task evidence but not added
  to CCR unless a current CCR API consumes context-window data.

Cost sample to lock:

- input: `1_000_000`
- cache read: `200_000`
- cache creation: `300_000`
- output: `400_000`
- cost with cache: `33.95`
- cost without cache: `35.0`

Calculation:

- with cache: `10 + 0.2 + 3.75 + 20`
- without cache: `(1_000_000 + 200_000 + 300_000) * 10 / 1M + 20`

## File-Level Plan

- `crates/ccr-types/src/model_rate_catalog.rs`
  - Add canonical Fable/Mythos defaults to `official_model_rate_overrides()`.
  - Extend normalization or official matching for short/provider-prefixed aliases.
  - Add exact positive and negative tests plus the `33.95`/`35.0` sample.
- `crates/ccr-store/src/models/pricing.rs`
  - Add or update default-pricing tests to assert both canonical rows are exposed.
- `crates/ccr-store/src/models/stats.rs`
  - Usually no code change; it derives from `official_model_rate_overrides()`.
    Add coverage only if the existing tests do not exercise the defaults path.
- `crates/ccr-db/src/services/usage_import_service.rs`
  - Add a focused legacy import parse/cost test for Fable/Mythos if the legacy
    test harness still covers model-specific pricing.
- `crates/ccr-db/src/database/migrations.rs`
  - Extend migration v13 repricing coverage with a Fable/Mythos row so upgraded
    historical records are not left unpriced.
- `crates/ccr-db/src/database/repositories/usage_repo.rs`
  - Add model-stat/rate-summary coverage if existing tests make this cheap;
    otherwise rely on catalog tests plus import/migration tests.
- `crates/ccr-usage/src/fixtures.rs` and/or `crates/ccr-usage/src/db.rs`
  - Add a projection test only if it improves coverage of pass-through
    `static`/`static-v1` fields without duplicating Tauri service tests.
- `ccr-ui/src-tauri/src/services/usage.rs`
  - Seed Fable/Mythos `SeedBucket` rows through `ccr_usage::fixtures` and assert
    `usage_by_model`/dashboard DTOs preserve model names, costs, status, source,
    and rate.
- `ccr-ui/tests/usage-models-tab.smoke.test.ts`
  - Add a smoke row if the fixture is easy to update, proving the table accepts
    long model names and `static` status without UI special casing.
- `crates/ccr-cli/src/services/install_*`
  - Inspect before editing. Current flow detects versions and installs latest
    package-manager versions but has no minimum-version policy. Prefer no hard
    gate unless the open question is answered differently.

## Compatibility And Migration

- Existing CCR archive rows are only recalculated by legacy migration/import
  paths that already use `ModelRateCatalog`. Existing `llmusage.db` rows are not
  rewritten by CCR.
- If a user has an older installed `llmusage`, the desktop dashboard will show
  whatever that database stored. CCR should not fabricate static-v1 pricing for
  upstream projection rows.
- Existing `pricing_status = priced` from CCR legacy catalog and
  `pricing_status = static` from `llmusage` projection are both valid but belong
  to different boundaries.

## Trade-Offs

- Adding exact alias support in `ccr-types` is preferable to adding source-aware
  pricing there. The current catalog API has no source parameter, and source
  semantics already belong to `llmusage`/`ccr-usage` for desktop projections.
- Avoid a hard installed-version gate for now. It would block users whose
  current database is otherwise readable and would require product policy around
  package-manager release timing.
- Do not add context-window storage unless a current CCR feature consumes it.
  The upstream fact should be captured as evidence, not as unused API surface.
