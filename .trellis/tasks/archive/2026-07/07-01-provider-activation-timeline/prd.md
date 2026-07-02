# C1 — Provider activation timeline (append-only log at apply)

Parent: `07-01-provider-usage-stats` · Design: parent `design.md` §3 · Order: first (no deps)

## Goal

Record, at every profile apply/switch/clear, an append-only timeline of which
relay provider became active for each platform, so downstream (llmusage ingest)
can attribute token/cost to a provider. This is the foundation the rest depends
on and is inert/harmless until consumed.

## Requirements

- Emit a timeline event at the shared ccr-config choke point so **all** callers
  (CLI, TUI, ccr-ui) are covered without per-caller wiring:
  - `activate` at `platform_config.rs:237-238` and `set_current_profile()` (`:250`).
  - `clear` at `base.rs:534-535` and `:567-568` (`current_profile = None`).
- Event fields: `platform, profile, provider, provider_type, base_url_host,
  account, activated_at (UTC RFC3339), event ∈ {activate, clear}`. Resolve from
  the applied `ProfileConfig` (`models/platform.rs:145-255`).
- Path: `$CCR_ROOT/analytics/provider_activation.jsonl`, append-only, one JSON per
  line. Format is the provider-map contract in parent design §2.2/§3.2 — keep
  byte-compatible with what llmusage will parse.
- **No secrets**: store `base_url_host` (host only), never `auth_token`/api key.
- Best-effort + atomic append: a write failure is logged and must **not** fail or
  roll back the profile switch (NFR2).
- `ccr-config` must gain no `ccr-db` dependency (write a file, not a DB table).

## Acceptance Criteria

- [ ] Switching a Claude profile and a Codex profile from CLI, TUI, and ccr-ui
      each appends exactly one `activate` event with correct fields.
- [ ] Clearing / unsetting a platform's current profile appends a `clear` event.
- [ ] The log never contains an auth token or api key (unit-tested).
- [ ] Injecting a write error does not break the profile switch.
- [ ] `just fmt-check && just lint-strict && just test` pass.

## Notes / dependencies

- No upstream/other-child dependency; can land independently.
- Coordinate the exact JSONL shape with C2 (llmusage `--provider-map` parser) —
  parent design §2.2 is the single source of truth.
