# Diagnostics, Dual-Check Interpretation, CAS & Runtime Recovery

## Table of Contents

- [1. Dual Exit-Code & JSON Health Verification Matrix](#1-dual-exit-code--json-health-verification-matrix)
- [2. Cross-Platform Read-Only Diagnostics](#2-cross-platform-read-only-diagnostics)
- [3. Optimistic Concurrency (`--expected-revision` CAS Loop)](#3-optimistic-concurrency---expected-revision-cas-loop)
- [4. Handling `profile edit --json` Partial-Commit Receipts](#4-handling-profile-edit---json-partial-commit-receipts)
- [5. Claude `remaining_suppressors` on `profile off`](#5-claude-remaining_suppressors-on-profile-off)
- [6. Operator-Confirmed Codex Runtime Recovery (`ccr codex fix`)](#6-operator-confirmed-codex-runtime-recovery-ccr-codex-fix)

---

## 1. Dual Exit-Code & JSON Health Verification Matrix

Never rely on process exit code alone. Status-card commands intentionally exit `0` while encoding broken runtime health inside JSON.

| Command | Exit Code on Broken Config | Exact JSON Fields to Inspect | Required Follow-Up When Unhealthy |
| --- | --- | --- | --- |
| `ccr current --json` | **`0`** (status-card contract) | `claude.health`, `codex.health` (`"ready"` \| `"needs_login"` \| `"invalid"` \| `"unsupported"` \| `"error"`), plus `claude.note` / `codex.note` | If `"error"` or `"invalid"`, run `ccr validate --json` |
| `ccr <claude\|codex> profile current --json` | **`0`** (status-card contract) | Top-level `PlatformStatusCard`: `platform`, `display_name`, `profile`, `provider?`, `base_url?`, `model?`, `small_fast_model?`, `auth`, `auth_kind`, `health`, `note` | If `"health"` in `["error", "invalid"]`, run `ccr validate --json` |
| `ccr grok profile current --json` | `0` (non-zero on unreadable TOML) | `platform`, `profile`, `runtime_mode` (`"profile"` \| `"grok_native"`), `details?` | Run `ccr validate --json` and `ccr doctor --json --platform grok` to check Grok drift / entry state |
| `ccr validate --json` | Non-zero on first `severity == "error"` (`0` on `info`/`warning`) | `checks[]: { platform, target, severity, category, message, exit_code }` | Report failing `target` + `message`; zero disk writes |
| `ccr doctor --json [--platform <p>]` | Non-zero if `summary.failed > 0` | `summary: { passed, warnings, failed, skipped }`, `checks[]: { id, status, summary, detail, recommendation }` | Surface `recommendation` without mutating unapproved files |
| `ccr <p> profile switch <name> --json` | Non-zero on failure | `ok`, `platform`, `profile`, `previous_profile`, `status`, `revision`, `warnings` | Verify `ok == true`; review `warnings` |
| `ccr <p> profile edit <name> ... --json` | Non-zero on failure **or** partial/rollback failure | `ok`, `platform`, `old_name`, `new_name`, `changed_fields`, `reapplied`, `revision`, `status`, `warnings` | On non-zero exit, parse **both** `stderr` and `stdout` JSON receipt |

---

## 2. Cross-Platform Read-Only Diagnostics

All four cross-platform diagnostic commands belong to **L0** (zero disk writes, auto-allowed):

1. `ccr platform list --json`
   - Lists registered platforms, enabled/current profile summary, and availability. Remember: `ccr-cli` mutates **only** `claude`, `codex`, and `grok` profiles.
2. `ccr current --json` (alias `ccr status --json`)
   - Returns `RuntimeOverview` (`schema_version: 2`, `generated_at`, `claude`, `codex`) where each `PlatformStatusCard` reports `platform`, `display_name`, `profile`, optional `provider` / `base_url` / `model` / `small_fast_model`, `auth`, `auth_kind` (`"official_auth"` | `"third_party_api"` | `"provider_key"` | `"no_auth"` | `"missing"` | `"unknown"`), `health` (`"ready"` | `"needs_login"` | `"invalid"` | `"unsupported"` | `"error"`), and `note`. For Grok's active runtime mode (`"profile"` vs `"grok_native"`), run `ccr grok profile current --json`.
3. `ccr validate --json`
   - Fast, offline, zero-write structural validation across catalog files, active pointers, required fields, Grok entry state, and runtime config parsability. Always run this first when a status card reports `"error"` or `"invalid"`.
4. `ccr doctor --json [--platform <claude|codex|grok>]`
   - Deeper environment and runtime inspection (directory permissions, CLI binary presence, config drift, stale state, actionable `recommendation` strings). Keep `--online` off unless the operator specifically asks for network endpoint probing.

---

## 3. Optimistic Concurrency (`--expected-revision` CAS Loop)

`profile create`, `profile edit`, and `profile import` support compare-and-swap (CAS) protection against concurrent edits from another terminal, `ccr` TUI, or `ccr-ui` desktop:

1. Read `ccr <platform> profile list --json` and extract top-level `"revision"` (a content hash string, or `null` when the platform catalog has not been initialized yet).
2. Pass `--expected-revision <REV>` on `ccr <platform> profile edit <name> ... --expected-revision <REV> --json` or `ccr <platform> profile import <file> ... --expected-revision <REV> --json`.
3. If CCR returns a `RevisionConflict` error:
   - **Do not blindly re-run** the write with the new revision.
   - Re-read `ccr <platform> profile list --json`, inspect what changed externally, and confirm with the operator if the target profile was modified concurrently.

---

## 4. Handling `profile edit --json` Partial-Commit Receipts

When `profile edit` modifies the catalog and re-applies an active profile to the live host runtime, `EditProfileOutcome::to_json_value()` writes `{ ok, platform, old_name, new_name, changed_fields, reapplied, revision, status, warnings }` to `stdout` while detailed error diagnostics go to `stderr`:

- **`claude` and `codex` `status` values**:
  - `"applied"` / `"applied_with_warning"` (`"ok": true`, exit `0`): Update succeeded; inspect `warnings`.
  - `"unchanged"` (`"ok": false`, non-zero exit): Re-applying live runtime failed, and CCR automatically restored the previous configuration. Inspect `stderr` and run `ccr validate --json`.
  - `"recovery_required"` (`"ok": false`, non-zero exit): Re-applying failed and external runtime changes were preserved rather than overwritten. Stop further mutations immediately, report `revision`, `warnings`, and `stderr` diagnostics, and let the operator choose recovery.
- **`grok` `status` values**:
  - `"updated"` / `"renamed"` (`"ok": true`, exit `0`): Catalog and runtime succeeded.
  - `"updated_apply_failed"` / `"rename_apply_failed"` / `"rename_cleanup_failed"` (`"ok": false`, non-zero exit): Profile catalog saved, but live `$GROK_HOME/config.toml` re-apply or old-name cleanup failed. Inspect `stderr` and run `ccr validate --json` plus `ccr doctor --json --platform grok` before retrying.

---

## 5. Claude `remaining_suppressors` on `profile off`

Running `ccr claude profile off --json` clears CCR-managed routing keys from `~/.claude/settings.json` so Claude Code returns to native subscription mode. However, external environment variables or unmanaged settings can still override subscription auth.

- Always inspect `"remaining_suppressors"` in the `claude profile off --json` output.
- If `"remaining_suppressors"` is non-empty, list the reported suppressor names/sources clearly and advise the operator to unset them in their shell startup file (`~/.zshrc`, `~/.bashrc`, PowerShell `$PROFILE`, or system environment) rather than editing user shell dotfiles without permission.

---

## 6. Operator-Confirmed Codex Runtime Recovery (`ccr codex fix`)

Codex runs background `app-server` processes that can cache stale runtime configuration or drift from `~/.ccr/platforms/codex/profiles.toml`. `ccr codex fix` is a **specialist text-only recovery tool** (it does **not** accept `--json`, and is outside the standard 12-action `profile` allowlist):

1. **Operator Confirmation Required**:
   - Before invoking `ccr codex fix`, explain the detected Codex drift (from `ccr current --json`, `ccr validate --json`, or `ccr doctor --json --platform codex`) and obtain explicit operator approval.
   - Once approved, preview first with `ccr codex fix --dry-run` (inspects `~/.codex/config.toml` + `auth.json` alignment and stale `codex app-server` processes in text output), then execute `ccr codex fix --repair-runtime` only when the operator confirms terminating stale `codex app-server` processes and replaying the active Codex profile (`repairable == true`).
2. **Exit Code Semantics**:
   - `0`: Runtime already clean or repaired successfully.
   - `1`: Unexpected runtime/IO error.
   - `2`: Process discovery or respawn failure.
   - `3`: Unrepairable configuration mismatch (manual conflict in `~/.codex/` that CCR refuses to overwrite blindly) or drift remaining in `--dry-run`.
   - `127`: `codex` binary not found when `--doctor` check is requested.
