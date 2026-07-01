# Per-provider token/cost usage stats (Claude Code + Codex)

## Goal

Give CCR a **per–relay-provider** token and cost view for Claude Code and Codex,
surfaced in both the **TUI** and **ccr-ui**. "Provider" here means the relay a
profile points at (anyrouter / methink / glm / deepseek / …), not just the CLI
source (claude / codex). Token/cost numbers are reused from **llmusage**; the
provider dimension is added natively in llmusage and fed by a new CCR-owned
profile activation timeline.

## Background / current state (verified)

- CCR already has **two** token/cost engines, both only at **source** granularity
  (`{claude, codex, gemini, opencode}`):
  - **llmusage** (`~/.llmusage/llmusage.db`, read-only via `llmusage_adapter`) —
    the modern engine that powers the ccr-ui `/usage` V2 dashboard. Tables
    `usage_event` / `usage_bucket_30m` carry `source, model, event_at, tokens*,
    cost_with/without_cache_usd, project*`. Query API already has
    `source_breakdown`, `trends_daily`, `model_breakdown`, `project_breakdown`,
    `heatmap`, `logs`, filterable by `source + model + date + project`.
  - **CostTracker** (`ccr-store`) — legacy engine behind `stats.rs`
    `get_provider_usage`; its `by_provider` is really `record.platform`
    (claude/codex) **counts**, not relay-provider.
- **Why per-provider is not free today:** Claude's
  `~/.claude/projects/**/*.jsonl` and Codex logs **do not record base_url /
  provider** — llmusage only sees model + tokens. And CCR keeps **no history** of
  which profile was active when (only `current_profile` + a single `last_used` in
  `~/.ccr/config.toml`; `usage_count` is a bare counter). So historical usage
  cannot be reattributed.
- **Display infra already exists:** ccr-ui has the full `/usage` dashboard, Pinia
  `stores/usage.ts`, ApexCharts, i18n (zh-CN/en-US), and ~26 usage/stats
  commands. ccr-tui has a tab system (`TuiTabId` enum + `DEFAULT_TAB_ORDER`) and
  the Codex Auth tab already renders rolling usage tables to copy from.

## Decisions (locked with user, 2026-07-01)

1. **Granularity = per relay-provider.** Attribution via a new activation
   timeline, effective **going forward only**. Source-level rollup is a free
   by-product. Data recorded before the timeline exists stays source-level /
   "unattributed".
2. **Provider dimension lives natively in llmusage.** llmusage gains a
   `provider_label` column + an ingest hook that consumes CCR's activation
   timeline + a `provider` query/breakdown dimension. The user implements the
   upstream llmusage changes; CCR provides the exact contract (design.md §2) and
   feeds the timeline. **CCR must build the activation timeline regardless** —
   llmusage cannot know the provider on its own.
3. **Cost basis = llmusage's official-equivalent cost**, surfaced in the UI with
   an explicit "≈ official-equivalent price" label. Third-party relays may bill
   differently; real per-profile pricing is out of scope.

## Scope

**In scope**
- CCR-owned profile activation timeline (append-only, written at profile apply).
- llmusage upstream contract for native provider support (spec the user builds).
- `llmusage_adapter` read path for `provider_label` + a `provider_breakdown`
  projection + `provider` filter, version-gated by a new schema version.
- Sync wiring so llmusage ingests the timeline (`--provider-map`).
- New Tauri command(s) for per-provider usage; provider filter on the dashboard.
- ccr-ui per-provider usage surface (view or dashboard section) + i18n.
- ccr-tui usage/statistics tab showing per-provider token/cost.

**Out of scope**
- Real third-party relay billing / per-profile custom price rates.
- Historical backfill / heuristic reattribution of pre-timeline data.
- Gemini / OpenCode provider attribution (source-level only for now; design must
  not preclude adding them later).
- Attribution when the user bypasses CCR by exporting base_url env vars manually
  (documented known limitation).

## Requirements

### Functional
- FR1: Every profile apply/switch/clear that changes the active profile for a
  platform records a timeline event `(platform, profile, provider, provider_type,
  base_url_host, account, activated_at)`. This holds across **all** entry points
  (CLI, TUI, ccr-ui) by hooking the shared ccr-config apply path — no caller may
  be missed.
- FR2: The timeline never persists secrets (no auth token / api key); base_url is
  stored as host only.
- FR3: llmusage, given the timeline, stamps each `usage_event` with the
  `provider_label` active for `(source == platform, event_at ∈ [t_i, t_{i+1}))`;
  events with no matching window get NULL → shown as "unattributed".
- FR4: `llmusage_adapter` exposes a per-provider breakdown (tokens split:
  input/output/cache-read/cache-creation/reasoning, request count,
  cost_with/without_cache_usd) and a `provider` filter on existing queries,
  gated by the new schema version and degrading gracefully on older DBs.
- FR5: ccr-ui shows per-provider token/cost for Claude Code and Codex with
  time-range filtering, an "unattributed" bucket, and the "≈ official-equivalent
  price" cost label. Reuses existing usage components/store where possible.
- FR6: ccr-tui shows a per-provider token/cost table for Claude Code and Codex
  with refresh, respecting the existing tab/keys/status conventions.

### Non-functional
- NFR1: llmusage stays read-only from CCR's side except the one sanctioned
  ingest input (the provider-map file); the adapter must not link the upstream
  crate, migrate, or parse raw provider logs (existing contract preserved).
- NFR2: Timeline writes are atomic and append-only; a write failure must not
  break or roll back the profile switch itself (best-effort, logged).
- NFR3: Adapter changes must not break older llmusage DBs lacking
  `provider_label` (capability gate + graceful "provider unsupported" state).
- NFR4: No regression to the existing `/usage` dashboard or stats commands.
- NFR5: Secret masking, atomic writes, backup-before-destructive-change, and
  file-locking behavior for config/sync paths remain intact.

## Deliverables — child task map

| Child | Slug | Owns | Depends on |
|------|------|------|-----------|
| C1 | `07-01-provider-activation-timeline` | ccr-config apply hook → append-only activation JSONL; format = the provider-map contract | — |
| C2 | `07-01-llmusage-provider-ingest-adapter` | Upstream llmusage contract (user builds) + adapter `provider_label`/`provider_breakdown`/`provider` filter + sync `--provider-map` wiring + Tauri command(s) | C1 (map format), upstream llmusage |
| C3 | `07-01-ccr-ui-provider-usage-view` | ccr-ui per-provider surface + store + i18n | C2 commands |
| C4 | `07-01-ccr-tui-usage-tab` | ccr-tui usage/statistics tab | C2 (or timeline+adapter read) |

Sane build order (not a hard dependency engine): C1 → C2 → {C3, C4}. Each child
carries its own acceptance criteria and is verified independently; see each
child's `prd.md`.

## Acceptance criteria (cross-cutting / parent integration)

- AC1: After switching Claude and Codex profiles a few times and generating real
  usage, both TUI and ccr-ui attribute new token/cost to the correct provider,
  with an "unattributed" bucket for the remainder. Verified end-to-end.
- AC2: Switching a profile from CLI, TUI, and ccr-ui each produce a timeline
  event (no entry point missed).
- AC3: On an llmusage DB without `provider_label`, ccr-ui and TUI degrade to a
  clear "provider attribution unavailable / update llmusage" state instead of
  erroring; the source-level `/usage` dashboard still works.
- AC4: No secret ever appears in the activation log.
- AC5: `just version-check` → `just fmt-check` → `just lint-strict` → `just test`
  (Rust) and `just frontend-check-quick` (UI) pass. Full `just ci` for final
  acceptance.
- AC6: The llmusage upstream contract (design.md §2) is precise enough that the
  user can implement it in the llmusage repo without further questions, and CCR's
  adapter/sync consumes exactly that shape.

## Risks / open items

- R1: Wall-clock window attribution assumes one global active profile per
  platform (true for CCR's model — it writes a single settings.json). Manual env
  overrides break this — documented limitation (out of scope).
- R2: Schema-version coordination between the user's llmusage build and CCR's
  adapter gate — pin an agreed `provider_label` schema version in the contract.
- R3: Timezone consistency between `activated_at` and llmusage `event_at` — the
  contract fixes both to UTC RFC3339 for the join.
- R4: Cost meaning for third-party relays is only "official-equivalent" — must be
  labeled to avoid misleading users (FR5).

## Appendix: the "missing content" for llmusage (summary)

llmusage itself lacks nothing for token/cost/model/project/source stats. To carry
**provider** natively it needs three additions (full contract in design.md §2):
1. `provider_label` (nullable TEXT) on `usage_event` (+ `usage_bucket_30m`), new
   schema version.
2. An ingest input (`--provider-map <path>`) = CCR's activation timeline, used to
   stamp `provider_label` by `(source, event_at ∈ window)`.
3. `provider` as a query filter + a `provider_breakdown` (mirroring
   `source_breakdown`).
