# llmusage Fable Mythos stats adoption

## Goal

Bring CCR's usage-pricing/statistics surfaces into alignment with the local
`llmusage` checkout at `D:\Documents\Code\CLI\llmusage`, where Claude Fable 5
and Claude Mythos 5 were added to the static catalog. CCR should price these
models correctly in Rust legacy/catalog paths and should display already-priced
`llmusage` projection rows correctly in `ccr-ui`.

User value: after upgrading/syncing `llmusage`, Fable/Mythos usage should not
show up as unknown/unpriced in CCR's pricing defaults, legacy archive views, or
desktop Usage model breakdowns.

## Confirmed Facts

- Upstream/local `llmusage` commit `311e9bc` added Fable/Mythos catalog coverage
  in `pricing/static-v1.json`, `src/query/pricing*.rs`, `src/query/mod.rs`, and
  `tests/local_flow.rs`.
- The upstream static model entries cover:
  - Claude source aliases: `claude-fable-5`, `fable-5`,
    `claude-mythos-5`, `mythos-5`.
  - OpenCode/Anthropic aliases: `anthropic-claude-fable-5`,
    `anthropic-claude-mythos-5`; upstream tests also normalize dotted/slashed
    provider prefixes.
  - Rates: input `10.0`, cached read `1.0`, cache creation `12.5`, output
    `50.0`, context window `1_000_000`.
- Official Claude Platform docs, checked 2026-07-06, list API IDs
  `claude-fable-5` and `claude-mythos-5`, shared 1M context, and pricing at
  `$10` input / `$50` output per million tokens. Anthropic's pricing page states
  prompt-cache multipliers: 5-minute cache writes `1.25x` base input and cache
  reads `0.1x` base input, matching `12.5` and `1.0`.
- CCR currently has no `fable` or `mythos` hits in the pricing catalog path;
  current `fable` strings are unrelated Claude profile env fields.
- CCR must not link the upstream `llmusage` Rust crate. The project spec and
  `ccr-ui/src-tauri/tests/llmusage_no_crate_guard.rs` require CCR to invoke the
  installed CLI and read the SQLite projection read-only through `crates/ccr-usage`.
- `crates/ccr-types/src/model_rate_catalog.rs` is the embedded pricing source
  consumed by legacy CCR archive/import paths.
- `crates/ccr-store/src/models/stats.rs` and
  `crates/ccr-store/src/models/pricing.rs` derive built-in pricing defaults from
  `official_model_rate_overrides()`.
- `crates/ccr-db/src/services/usage_import_service.rs`,
  `crates/ccr-db/src/database/migrations.rs`, and legacy
  `usage_repo::get_model_stats` use `ModelRateCatalog::official()`.
- `crates/ccr-usage` is a read-only SQL projection over `llmusage.db`; it should
  pass through stored `pricing_status`, `pricing_source`, and `pricing_rate`
  rather than recalculate upstream pricing.
- `ccr-ui/src-tauri/src/services/usage.rs` maps `crates/ccr-usage` model rows
  into `ModelStatDto`; `UsageModelsTab.vue` already supports `static`,
  `snapshot`, `mixed`, `unpriced`, `legacy_alias`, and arbitrary model names.

## Requirements

- Add CCR embedded pricing coverage for `claude-fable-5` and
  `claude-mythos-5` with rates input `10.0`, cache read `1.0`, cache creation
  `12.5`, output `50.0`.
- Support the alias forms seen in local `llmusage` for pricing lookup without
  overmatching unrelated names such as `not-fable-5` or
  `claude-mythos-preview`.
- Keep canonical built-in default rows in CCR as `claude-fable-5` and
  `claude-mythos-5`; aliases should resolve for calculation, not necessarily
  duplicate every default-config row.
- Preserve CCR's no-upstream-crate architecture. Do not add `llmusage = ...` to
  any manifest and do not import `llmusage::...`.
- Keep all usage SQL centralized in `crates/ccr-usage`; Tauri/UI code should
  only map DTOs and display fields.
- Add focused tests proving:
  - CCR catalog cost with cache for the upstream sample token mix is `33.95`
    and cost without cache is `35.0`.
  - CCR built-in pricing defaults include both canonical Fable/Mythos rows.
  - Legacy ccr-db import/reprice paths no longer leave Fable/Mythos as
    `unpriced`.
  - `ccr-ui` read-only projection rows for Fable/Mythos with
    `pricing_status = static`, `pricing_source = static-v1`, and
    `pricing_rate = 10/1/50` flow into `ModelStatDto` and, if practical, the
    model tab smoke payload.
- Keep context-window handling explicit: if no current CCR surface consumes a
  per-model context-window catalog, document that this task adopts pricing/stat
  coverage only and does not invent a new context-pressure subsystem.
- Inspect the install/detection path before implementation. Do not add a hard
  minimum-installed-`llmusage` gate unless it is deliberately chosen; stale local
  `llmusage` installs should continue to use existing capability/diagnostic
  behavior.

## Acceptance Criteria

- [ ] `ModelRateCatalog::official()` prices `claude-fable-5`,
      `claude-mythos-5`, `fable-5`, `mythos-5`, and supported Anthropic-prefixed
      aliases at `10/1/50` with cache-write `12.5`.
- [ ] Negative alias tests prove broad substring matching is not used for
      Fable/Mythos.
- [ ] `official_model_rate_overrides()` and `PricingConfig::with_claude_defaults()`
      expose canonical rows for both models.
- [ ] Legacy `ccr-db` import/migration/model-stat tests cover at least one
      Fable/Mythos record and preserve cache-aware pricing fields.
- [ ] `crates/ccr-usage`/`ccr-ui/src-tauri` tests show a `llmusage.db`
      projection row for Fable/Mythos remains `static`/`static-v1` and carries
      its rate string through to the desktop DTO.
- [ ] The no-crate guard still passes.
- [ ] No new Usage Dashboard SQL appears outside `crates/ccr-usage` except
      comments/docs.
- [ ] Validation commands in `implement.md` have been run or explicitly marked
      not run with a reason before finishing implementation.

## Out of Scope

- Modifying the upstream `llmusage` repository.
- Adding the upstream `llmusage` Rust crate as a dependency.
- Reworking provider attribution, schema 14 gating, or `--provider-map` sync
  semantics.
- Redesigning Usage Dashboard UI.
- Backfilling/repricing existing `llmusage.db` rows inside CCR. Users must rerun
  or upgrade/sync `llmusage` for upstream projection data to change.

## Open Question

- Should CCR warn or require a minimum installed `llmusage` version that contains
  Fable/Mythos pricing? Recommended answer: do not hard-block existing usage
  flows in this task; rely on existing capability/diagnostic surfaces and only
  add a soft note if the current install UI already has a natural place for it.

## Notes

- Primary evidence:
  - Local upstream catalog:
    `D:\Documents\Code\CLI\llmusage\pricing\static-v1.json`
  - Local upstream commit:
    `311e9bc feat(定价): [AI] ✨ 添加 Claude Fable/Mythos 静态统计覆盖`
  - Official model docs:
    `https://platform.claude.com/docs/en/about-claude/models/introducing-claude-fable-5-and-claude-mythos-5`
  - Official prompt-cache multiplier docs:
    `https://docs.claude.com/en/docs/about-claude/pricing`
