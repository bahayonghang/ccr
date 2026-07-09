# Design — Per-provider token/cost usage stats

> Scope: technical design + the exact **llmusage upstream contract** (§2) the user
> implements in the llmusage repo. Requirements/acceptance live in `prd.md`.

## §1 Architecture & data flow

```
profile switch (CLI / TUI / ccr-ui)
        │  hook at ccr-config apply choke point
        ▼
 [C1] append-only activation log  ~/.ccr/analytics/provider_activation.jsonl
        │  (platform, profile, provider, base_url_host, account, activated_at, event)
        ▼
 llmusage sync --provider-map <log>      ← [C2] wiring passes the path
        │  [UPSTREAM] build per-platform intervals, stamp provider_label
        ▼
 ~/.llmusage/llmusage.db  usage_event / usage_bucket_30m  (+ provider_label)
        │  read-only SQL projection
        ▼
 [C2] llmusage_adapter: provider filter + provider_breakdown()  (schema-gated)
        │  Tauri command get_usage_by_provider_v2 (+ provider on dashboard)
        ├──────────────► [C3] ccr-ui per-provider view
        └──────────────► [C4] ccr-tui usage tab
```

Key invariant: **CCR owns the "who was active when" truth (C1); llmusage owns
tokens/cost/pricing and applies the label; the adapter only reads.** Neither
Claude nor Codex logs carry base_url, so the timeline is the only source of the
provider dimension.

## §2 llmusage upstream contract (the "missing content" — user implements)

Target repo: `bahayonghang/llmuasage` (currently 0.5.3 @ `9bdac14`). Three
additions. **Target `schema_version = 14`** — verified against the real llmusage
tree: current is 13 (migrations 1..13), the new migration is v14; CCR adapter's
current min-supported is 10. (Grounded llmusage task:
`D:\Documents\Code\CLI\llmusage` → `07-01-provider-label-dimension`.)

### 2.1 Schema

- Add column `provider_label TEXT NOT NULL DEFAULT ''` to **both** `usage_event`
  and `usage_bucket_30m`; `''` = **unattributed**.
- **Critical:** the `usage_bucket_30m` PRIMARY KEY MUST include `provider_label`
  (alongside source/model/hour_start/project_hash). Otherwise two providers active
  within the same 30-minute window collapse into one bucket and per-provider sums
  are wrong. Use `NOT NULL DEFAULT ''`, **not** NULL: a NULL PK column compares as
  distinct in SQLite and would silently break the upsert dedup (mirrors the
  existing `project_hash NOT NULL DEFAULT ''`).
- Bump the stored `meta.schema_version` to `14`. Existing rows backfill to
  `provider_label = ''` (→ "unattributed").

### 2.2 Ingest input — the provider-map (activation timeline)

- New sync flag `--provider-map <path>` (JSONL; also honor a default path if
  absent so cron sync still works — TBD with user, default path recommended:
  `$CCR_ROOT/analytics/provider_activation.jsonl`).
- Each line is one activation event (see §3.2 for the exact producer format):
  ```json
  {"platform":"claude","profile":"anyrouter3","provider":"anyrouter",
   "base_url_host":"anyrouter.top","activated_at":"2026-07-01T12:03:44Z","event":"activate"}
  {"platform":"claude","profile":null,"provider":null,
   "activated_at":"2026-07-01T13:20:00Z","event":"clear"}
  ```
- **Interval construction:** for each `platform`, sort events by `activated_at`
  ascending; event _i_ defines the half-open window `[activated_at_i,
activated_at_{i+1})`; the last window extends to `+∞`. `platform` maps to
  llmusage `source` (`claude`→claude, `codex`→codex).
- **Stamping:** for each usage event, set `provider_label = window.provider` where
  `event.source == window.platform` and `event.event_at ∈ window`. A `clear`
  window and any event before the first window → `provider_label = ''`.
- **Idempotent / rebuildable:** stamping is a pure function of (events, map), so
  `llmusage sync --rebuild` re-derives labels. Times are **UTC RFC3339** on both
  sides (matches existing `event_at`).

### 2.3 Query surface (optional upstream; CCR can also do it adapter-side)

Minimal upstream requirement is only 2.1 + 2.2 (column + stamping); CCR's adapter
runs its own `GROUP BY provider_label` SQL (§4). If the user also wants
llmusage's own dashboard/CLI to split by provider, add there:

- `provider` to the query filter, and
- a `provider_breakdown` mirroring the existing `source_breakdown`.

## §3 [C1] CCR profile activation timeline

### 3.1 Write hook (single choke point, covers all callers)

`ccr-config` has **no `ccr-db` dependency** (verified), so the timeline is an
append-only file written from ccr-config, not a DB table. Hook where the active
profile actually changes:

- `crates/ccr-config/src/managers/platform_config.rs:237-238` (apply sets
  `current_profile` + `last_used`) and the `set_current_profile()` setter at
  `:250` → emit an `activate` event.
- `crates/ccr-config/src/platforms/base.rs:534-535` and `:567-568`
  (`current_profile = None`) → emit a `clear` event.

Because CLI, TUI, and ccr-ui all funnel profile application through this
ccr-config layer, hooking here satisfies FR1/AC2 without touching each caller.
Resolve provider fields from the applied `ProfileConfig`
(`crates/ccr-config/src/models/platform.rs:145-255`: `provider`, `provider_type`,
`base_url`, `account`).

### 3.2 Format & location

- Path: `$CCR_ROOT/analytics/provider_activation.jsonl` (CCR root already known
  to ccr-config). Append-only, one JSON object per line.
- Fields: `platform, profile, provider, provider_type, base_url_host, account,
activated_at (UTC RFC3339), event ∈ {activate, clear}`.
- **No secrets (FR2/AC4):** store `base_url_host` (host only, via url parse), never
  `auth_token`/api key. `account` is a label already shown in the UI, safe.

### 3.3 Robustness (NFR2)

- Write is best-effort and atomic-append; a failure is logged and **must not**
  fail or roll back the profile switch. (Append with a short file lock consistent
  with existing config-write conventions; no rewrite of prior lines.)
- Dedup: if the applied profile equals the currently-active one, still emit
  (re-apply is a legitimate boundary) — keep it simple, let llmusage's interval
  logic dedup identical consecutive windows.

## §4 [C2] Adapter + sync wiring (CCR side)

Files: `ccr-ui/src-tauri/src/llmusage_adapter/{db.rs,queries.rs,capabilities.rs,
cli.rs,source.rs}`, `ccr-ui/src-tauri/src/commands/usage.rs`,
`commands/handler_registry.rs`.

- **db.rs:** add a `provider_breakdown(&QueryFilter) -> Vec<ProviderBreakdownDto>`
  mirroring the existing `source_breakdown` (GROUP BY `provider_label` over
  `usage_bucket_30m`; `'' → unattributed`). Add `provider_label` to required
  columns for the new feature; add a `provider` predicate to the filter builder.
- **queries.rs:** `ProviderBreakdownDto { provider: Option<String>, input_tokens,
output_tokens, cache_read_tokens, cache_creation_tokens, reasoning_tokens,
requests, cost_with_cache_usd, cost_without_cache_usd }`.
- **capabilities.rs:** add `FeatureKey::ProviderBreakdown`, gated on
  `schema_version >= 14` AND presence of `provider_label`. Reuse the existing
  `ensure_feature` degrade path (NFR3/AC3).
- **cli.rs:** extend `SyncCommandOptions` (`:47-52`) with
  `provider_map: Option<PathBuf>` → append `--provider-map <path>` to the sync
  invocation in `run_sync_stream`/`run_sync_collect`. The import commands in
  `commands/usage.rs` resolve `$CCR_ROOT/analytics/provider_activation.jsonl` and
  pass it.
- **Tauri command:** `get_usage_by_provider_v2(platform?, start?, end?)` returning
  `Vec<ProviderBreakdownDto>`, registered in `handler_registry.rs` next to the
  other V2 usage commands; add a `provider` param to `get_usage_dashboard_v2` for
  drill-down. Mirror the existing command/caching patterns.

## §5 [C3] ccr-ui per-provider view

Reuse, don't rebuild: the `/usage` group (`src/router/index.ts`),
`stores/usage.ts`, `components/usage/*` (StatCard, breakdown strips, ApexCharts),
i18n `usage.*` (`i18n/locales/{zh-CN,en-US}.ts`).

- Add a "By provider" surface: either a new tab inside `UsageDashboardView.vue`
  (alongside Overview/Tokens/Cost/Models/Projects/Logs) or a dedicated
  `/usage/providers` view — choose in the child's design.
- Store: add `providerStats` + a `fetchProviderStats()` action calling
  `get_usage_by_provider_v2`; respect the existing platform + time-range filters.
- Cost label: render the "≈ official-equivalent price" note next to any USD (FR5).
- Capability-aware: when `ProviderBreakdown` is unsupported, show the existing
  unsupported/empty pattern with an "update llmusage" hint (AC3).

## §6 [C4] ccr-tui usage tab

Add a tab via the existing system:

- `crates/ccr-config/src/managers/tui_config.rs:16-40`: add a `TuiTabId::Usage`
  variant + include it in `DEFAULT_TAB_ORDER` + `as_str()`.
- `crates/ccr-tui/src/tui/app.rs:33-60`: add `TabVariant::Usage`; build the tab in
  `with_task_executor`; route in `handle_key` and `ui::draw`.
- New module `crates/ccr-tui/src/tui/usage/{app.rs,ui.rs}`. Render a per-provider
  table (provider | requests | input/output/cache tokens | ≈cost) for Claude and
  Codex, following the Codex Auth usage panel pattern
  (`crates/ccr-tui/src/tui/codex_auth/ui.rs:542-607`). Load async via the existing
  `AsyncTaskExecutor` + message channel; key `r` = refresh.
- Data source: read the same per-provider breakdown. Since the TUI does not link
  the tauri adapter, the child decides between (a) a small shared read-only
  projection crate over `llmusage.db` + the activation log, or (b) reading the
  activation log + a lightweight llmusage read. Prefer factoring the adapter's
  read-only projection so TUI and ccr-ui share one implementation — resolve in the
  child design.

## §7 Attribution semantics & edge cases

- **One active profile per platform:** CCR writes a single `~/.claude/settings.json`
  / `~/.codex/config.toml`, so at any instant exactly one provider is active per
  platform; concurrent sessions share it. Wall-clock window attribution is
  therefore correct for CCR's model (R1).
- **Pre-timeline data:** events before the first activation window → `''` →
  "unattributed" bucket. Expected and surfaced, not an error.
- **Manual env override** (user exports `ANTHROPIC_BASE_URL` etc. outside CCR):
  attribution will be wrong — documented limitation, out of scope.
- **Clock skew / DST:** all timestamps UTC RFC3339 → no DST issues (R3).

## §8 Compatibility, rollout, rollback

- Adapter is additive + schema-gated: older llmusage DB (no `provider_label`,
  schema < N) → `ProviderBreakdown` reports unsupported; `/usage` and all existing
  commands keep working (NFR3/NFR4/AC3).
- C1 timeline is inert until llmusage consumes it; shipping C1 first is safe.
- Rollback: remove the `--provider-map` wiring + hide the provider surfaces; the
  activation log is harmless if left in place. No destructive migration on CCR's
  side. (Upstream llmusage column add is additive/nullable.)

## §9 Cost caveat (FR5)

llmusage cost = official model price × tokens (`cost_with/without_cache_usd`).
For third-party relays the real bill differs (often cheaper / packaged / free).
The UI/TUI must label provider cost as **"≈ official-equivalent price"** and never
present it as the actual amount billed. Real per-profile pricing is a possible
follow-up (was offered and deferred).
