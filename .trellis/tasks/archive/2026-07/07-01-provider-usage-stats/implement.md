# Implement — Per-provider token/cost usage stats (parent orchestration)

> Parent-level execution plan and integration gates. Each child has (or will get,
> at activation) its own `implement.md`. This file coordinates order, validation,
> and rollback across children. **No implementation starts until the owning child
> task is activated via `task.py start`.**

## Build order & gates

### Stage 0 — Contract handoff (blocks C2 ingest, not C1)

- [ ] Freeze the provider-map JSONL format (design.md §3.2) and the llmusage
      schema version `N` (design.md §2). Hand design.md §2 to the user for the
      **upstream llmusage** implementation.
- [ ] Confirm default provider-map path with user: `$CCR_ROOT/analytics/provider_activation.jsonl`.
- Gate: user acknowledges §2 is buildable without further questions (AC6).

### Stage 1 — C1 `provider-activation-timeline` (no external deps)

1. [ ] Add append-only writer in `ccr-config` (host-only base_url, no secrets).
2. [ ] Hook `platform_config.rs:237-238` + `set_current_profile()` (`:250`) →
       `activate`; `base.rs:534-535/:567-568` → `clear`.
3. [ ] Best-effort + atomic append; switch never fails on log error (NFR2).
4. [ ] Unit tests: activate/clear events, no-secret assertion, all-callers path.

- Validate: `just fmt-check && just lint-strict && just test`
- Gate: switching from CLI/TUI/ccr-ui each appends exactly one event (AC2/AC4).
- Ship-safe: C1 is inert until llmusage consumes it — may land independently.

### Stage 2 — C2 `llmusage-provider-ingest-adapter`

Precondition: user's upstream llmusage build with `provider_label` + `--provider-map`
available locally (or a fixture DB at schema `N`).

1. [ ] Adapter: `provider_label` read, `provider_breakdown()`, `provider` filter,
       `FeatureKey::ProviderBreakdown` gate (degrade on schema < N).
2. [ ] `SyncCommandOptions.provider_map` → `--provider-map`; import commands pass
       the activation-log path.
3. [ ] `get_usage_by_provider_v2` command + registration; `provider` param on
       dashboard command.
4. [ ] Tests incl. the no-crate guard (`tests/llmusage_no_crate_guard.rs`) and a
       schema-gate/degrade test (AC3).

- Validate: `cd ccr-ui/src-tauri && cargo check`; `just lint-strict`; `just test`.
- Review gates: **tauri-ipc-reviewer** (new command), **sqlite-migration-reviewer**
  (cross-DB read / bucket key), **rust-security-reviewer** (no secret leaks in map
  path).
- Gate: on a real synced DB, breakdown sums per provider + unattributed ==
  source totals.

### Stage 3 — C3 `ccr-ui-provider-usage-view` and C4 `ccr-tui-usage-tab` (parallelizable)

- C3: store action + view/tab + i18n (zh-CN/en-US) + "≈ official-equivalent
  price" label + capability-aware empty state.
  - Validate: `just frontend-check-quick`; review with **frontend-quality-reviewer**.
- C4: `TuiTabId::Usage` + `TabVariant::Usage` + `tui/usage/{app,ui}.rs` + routing.
  - Validate: `just test` (tui crate), manual TUI smoke.
  - Prefer sharing the adapter's read-only projection with C3 (design.md §6).

### Stage 4 — Parent integration

- [ ] End-to-end (AC1): switch Claude + Codex profiles, generate usage, sync, see
      correct per-provider attribution + unattributed bucket in **both** surfaces.
- [ ] Degrade path (AC3) verified against a pre-`N` DB.
- [ ] Full gate: `just ci`.
- [ ] Spec update (Phase 3.3) for touched packages: `ccr-config`, `ccr-ui`,
      `ccr-tui`, `llmusage_adapter` boundary notes.

## Validation command reference

- Rust fast: `just version-check && just fmt-check && just lint-strict && just test`
- Adapter compile: `cd ccr-ui/src-tauri && cargo check`
- Frontend fast: `just frontend-check-quick`
- Full: `just ci`
- Runtime debug: `CCR_LOG_LEVEL=debug`

## Rollback points

- C1: stop writing the log (feature-flag the hook); existing log is harmless.
- C2: drop `--provider-map` wiring + hide command; adapter gate already no-ops on
  old DBs.
- C3/C4: hide the provider surface / tab. None of these are destructive; no CCR
  migration to reverse. Upstream llmusage column is nullable/additive.

## Notes for sub-agent dispatch

Every dispatch prompt must start with `Active task: <path from task.py current>`.
Context order per child: `implement.jsonl`/`check.jsonl` → `prd.md` → `design.md`
(parent design is the shared contract) → child `implement.md`.
