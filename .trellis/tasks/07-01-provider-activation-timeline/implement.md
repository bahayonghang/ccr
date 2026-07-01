# Implement — C1 provider activation timeline

Design: parent `design.md` §3. Single-crate change in `ccr-config` (no new deps,
no `ccr-db` dependency added).

## Steps

1. New module `crates/ccr-config/src/managers/provider_activation.rs`:
   - `ActivationEvent` + `ActivationKind {activate, clear}` (serde, all fields
     always serialized for a stable cross-tool contract).
   - `record_activation(root, platform, profile_name)` — resolve provider fields
     from `root/platforms/<platform>/profiles.toml` via `load_profiles_from_toml`,
     append an `activate` event. Best-effort (`()` return; errors logged).
   - `record_clear(root, platform)` — append a `clear` event.
   - internals: `append_deduped` (skip if identical to the platform's last event),
     `last_event_for_platform`, `append_line` (LockManager + `OpenOptions::append`),
     `host_only` (strip scheme + path + userinfo → host[:port], no secrets),
     `default_ccr_root` (CCR_ROOT or ~/.ccr), `activation_log_path`.
2. `managers/mod.rs`: `pub mod provider_activation;`.
3. `platforms/base.rs` hooks (after the registry save succeeds, best-effort):
   - `update_registry_current_profile` + `_with_paths` → `record_activation`.
   - `reconcile_registry_current_profile_after_delete` + `_with_paths` → capture
     outcome, then `record_activation(next)` or `record_clear`.

## Validation

- `cargo test -p ccr-config -- --test-threads=1`
- `just fmt-check && just lint-strict`

## Tests (in the new module)

- `host_only` parsing (scheme/port/userinfo/no-scheme).
- activation writes provider from profiles.toml; log contains no `auth_token`.
- dedup: identical consecutive activation → 1 line; different provider → 2 lines.
- `record_clear` writes a `clear` event.
- missing profiles.toml → still logs `activate` with `provider=null`, no panic.

## Rollback

Remove the `record_*` calls in `base.rs`; the module is inert. Append-only log is
harmless if left on disk.

## Status (2026-07-01)

Implemented + verified. `cargo test -p ccr-config` (43 pass), `just fmt-check`,
`just lint-strict` (workspace, `-D warnings -D clippy::unwrap_used`) all green.

Entry-point coverage confirmed: ccr-ui (`claude_apply_profile`/`codex_apply_profile`),
TUI (`app.rs:784`), and CLI all funnel through the platform trait `apply_profile`
→ `base::update_registry_current_profile*`, which is hooked.

### Known follow-up (minor, deferred)

- `crates/ccr-cli/src/platforms/claude.rs:162 clear_current_profile_registry`
  writes `current_profile = None` directly (a drift self-heal in
  `stable_current_profile`), bypassing `base.rs` — so that rare self-heal clear is
  not logged. Not a user-initiated switch; left unhooked to keep the change inside
  ccr-config (single choke point). Revisit if attribution shows stale windows
  after a self-heal.
