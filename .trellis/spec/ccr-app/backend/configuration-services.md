# Shared Configuration Policy

## 1. Scope / Trigger

Applies to `ccr-app::config::{mutations, source, backups, profile_patch}` and
their Tauri/native consumers. These modules contain actual extracted adapter
policy. Catalog SQL, persistence, normalization, auth and process safety remain
with the existing domain owners. ccr-app has no ccr-cli/Tauri/GPUI dependency.

## 2. Signatures

- `mutations::parse_patch(&serde_json::Value) -> Result<ConfigPatch>` implements
  the generic configuration editor's historical ten-field protocol.
- `mutations::{add, update, duplicate, delete}` compose repository/platform
  policy. Update and delete require the caller's matching application FileLock.
- `source::read(&PlatformPaths) -> Result<SourceSnapshot>` returns canonical
  content/token/path/existence; the plaintext value deliberately has no Debug.
- `source::save(paths, current_profile, content, token, force)` returns typed
  `SourceSaveOutcome::{Invalid, ActivationConflict, Conflict, Saved}`.
- `source::export(profiles_file, include_secrets)` is the dedicated canonical
  profile export. Generic redacted export remains `ConfigService::export_config`.
- `backups::{resolve_managed_restore_path, restore, clean}` use a managed leaf,
  existing locked domain restore and explicit cleanup root, respectively.
- `profile_patch::{claude, codex, grok}` owns extracted pure field composition;
  native `validate_fields` is separate from legacy permissive Tauri decoding.

## 3. Contracts

- Generic omitted fields remain unchanged, null removes optional values, and
  enabled must be boolean. Unknown fields/types fail before writes. Dedicated
  platform fields/auth actions/whole-platform-data replacement stay distinct.
- Update/delete acquire application lock before resolving the actual current
  profile. Protect registry-fallback active profiles; a stored display row is
  not a current-state authority. Original editor revision is checked in the
  repository mutation, not replaced with a token read at Save time.
- Source represents the profiles.db canonical projection, not original TOML
  formatting/comments. Missing reads create no catalog. Empty source is invalid.
  Syntax/semantic diagnostics are static with optional position, never parser
  Display containing secret source.
- Force acknowledges active-profile removal only. It never bypasses token CAS.
  Generic redacted export succeeds separately; dedicated export rejects false.
  Tauri retains its original wire DTOs and exact error mapping.
- Restore accepts only a same-directory relative leaf with the current config
  filename prefix and `.bak` suffix. Safe domain parsing precedes existing
  preimage-backup/save; failed backup prevents overwrite.
- Cleanup receives an explicit root, retains depth3 and `.bak`/`.backup`
  deletion/success-count semantics, and does not follow directory symlinks.
  Native fixtures never call an ambient-home cleanup resolver. Tauri's legacy
  root selection is unchanged; do not turn extraction into a compatibility fix.
- Plaintext source/export/editor values remain scoped and excluded from Debug,
  generic errors, lists, logs and preferences. Legacy operation-history failure
  payloads are arbitrary text; native projection uses only the typed status.
- All filesystem/database work runs on02's bounded worker, with the complete
  injected lifecycle using its synchronous lock scope.

## 4. Validation & Error Matrix

| Condition | Required behavior |
| --- | --- |
| Same original revision, two saves | Exactly one commit, one conflict |
| Unknown generic field or invalid type | Safe validation error, no mutation |
| Source removes active profile without force | ActivationConflict, draft retained |
| Forced source with stale token | Conflict, no overwrite |
| Invalid secret-bearing source/backup | Safe diagnostic, no source echo |
| Traversal/absolute/nested restore path | Reject before restore |
| Preimage backup fails | Existing state remains intact |
| Page/context leaves while plaintext result pending | Discard stale result and clear scoped buffers; app-owned cleanup remains tracked |

## 5. Good / Base / Bad Cases

- Good: explicit generic/dedicated policy selection with shared pure patch
  composition, domain-owned writes and original draft revision.
- Base: call ConfigService/converter/history/lifecycle owners directly when no
  actual shared adapter policy needs extraction.
- Bad: one universal patch silently ignores dedicated/unknown fields; a generic
  redacted-export button calls the secret-only dedicated export.

## 6. Tests Required

`ccr-app/tests/profile_services.rs` covers strict generic fields, dedicated
field/credential semantics, same-token clients, canonical source/force/CAS,
source-safe diagnostics, managed restore/preimage and bounded cleanup. Retain
actual Tauri config/profile/Grok/binding regressions and native rendered-control
tests for dirty fields, conflict draft retention, confirmations, navigation
teardown and stale plaintext results. Native CU is separate evidence from
headless tests. Keep default parallelism and `--skip export_bindings`.

## 7. Wrong vs Correct

Wrong: list a masked credential, resend it as replacement, or switch a generic
editor to dedicated semantics without recording the lost behavior.

Correct: retain untouched fields, accept an explicitly entered replacement,
validate the selected protocol and save under the original catalog revision.
