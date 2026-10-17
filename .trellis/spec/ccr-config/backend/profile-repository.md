# Profile Repository Transactions

## Scope and ownership

`ccr-config` owns profile document reads, resource locks, partial edits, and persistence. Platform adapters own auth-mode validation and runtime operations. The repository does not depend on CLI, Tauri, or Codex services.

Sources: `managers/config/catalog/`, `managers/config/repository.rs`, `services/config_service.rs`, and `platforms/base.rs`. Paths are relative to `crates/ccr-config/src/`.

## Live catalog

Claude, Codex, Gemini, Grok, and Droid store profiles in `profiles.db` next to the legacy `profiles.toml` path. `PlatformPaths::profiles_file` stays the legacy path for migration and the "file removed" check. `document.revision` is the compare-and-swap token. SQL `NULL` means the marker was absent. An empty string means the key exists and is empty. Direct `load`, `snapshot`, list, current, and validate do not migrate and do not create a database. Legacy TOML alone returns `配置尚未迁移` and creates nothing. Both files absent returns `ConfigMissing` and creates nothing. Import and export exchange canonical TOML and do not create a live `profiles.toml`. Live `load`, `snapshot`, and canonical reads open the database read-only so a commit that is still only in the WAL stays visible. `immutable=1` is only for a checkpointed snapshot file. A checkpoint error after `COMMIT` is logged and does not fail the catalog transaction. Import and restore with backup write a secret canonical TOML backup before overwriting the catalog, including when the legacy TOML is already gone. Comments and whitespace are not preserved. A restored backup TOML is not read. Rollback: stop the new binary, restore the TOML from `backups/<platform>/`, and delete `profiles.db` with its `-wal` and `-shm` files. Usage counts written after migration exist only in the database.

## Interfaces

- `ConfigManager::for_platform(name)` opens an explicit platform path without I/O.
- `ConfigManager::with_default()` retains the legacy Claude domain. Compatibility evidence: `crates/ccr/tests/managers/legacy_registry.rs::config_manager_default_ignores_legacy_current_platform_routing` and the Claude-only lifecycle clear command. New callers must use `for_platform`.
- `load()` and `snapshot()` are pure reads of the catalog. `ConfigSnapshot::version` is `document.revision`.
- `mutate(closure)` requires an existing catalog. `mutate_or_create(closure)` permits explicit first creation.
- `mutate_versioned(expected, create, closure)` checks the revision token before mutation and commits the catalog transaction when the token matches.
- `ConfigPatch` maps field names to `FieldPatch::Set(value)` or `FieldPatch::Remove`. An omitted field is retained. Patch values have no Debug implementation.
- `patch(name, new_name, patch, expected, validator)` checks existence, target-name collision, field types, and the supplied platform validator under the resource lock.
- `clear_current_if_present()` is an explicit mutation. Missing files remain absent; corrupt or unreadable files return errors.
- `ensure_initialized()` and `load_with_autofix()` are explicit write workflows. Query services must not call them.
- `base::mutate_profiles` passes the latest profile map to a domain closure. `mutate_profiles_with_current` also permits the platform to preserve an inactive marker.

## Lock and write contract

1. A caller may hold a platform operation lock.
2. The repository derives a resource name from `ccr_core::core::guarded_write::normalized_resource_path`. The core helper owns absolute normalization. Windows names are case-folded; ordinary and verbatim drive/UNC prefixes identify the same resource. An existing ancestor's 8.3 short name identifies that same resource. The helper does not follow symlinks and does not require the final target to exist. The FNV-1a hash is stable across processes. No platform label or adapter-specific lock name selects the resource.
3. The repository acquires the resource lock before reading.
4. The closure runs once. The repository does not replay domain or external effects. Cross-file recovery remains an application responsibility.
5. The catalog transaction checks `document.revision`, writes the rows, and commits SQLite before runtime journal commit. It verifies the declared runtime files after mutation and before that commit. It does not replace a live `profiles.toml`. A revision mismatch writes nothing and creates no backup. A checkpoint error after `COMMIT` does not roll back the runtime journal.

The order is **operation lock → resource lock → catalog transaction**. Do not re-enter mutation while holding the same resource guard. `CONFIG_LOCK` and the old `ccr_config` / `platform_profiles_<name>` locks do not protect repository transactions.

Paths can be absent. Resource identity follows `normalized_resource_path` of `profiles.db`; callers must not introduce multiple symlink aliases for one managed target. A non-cooperating external process can still write after CCR commits. Observed revision conflicts return an actionable error; the repository never falls back to an unconditional replacement.

All profile content is secret. `save`, `save_config`, and `save_profiles_to_toml` write the catalog and do not create a live `profiles.toml`. Ordinary edits use the catalog transaction. Migration moves the legacy TOML into the platform backup directory. On Windows, case-only path aliases keep the same resource identity.

## Read and marker contract

- Constructors, list, current, validate, load, and export do not create lock files, initialize profiles, autofix fields, or rewrite current markers.
- Full and simplified TOML use the same safe semantic parser. Invalid TOML and invalid typed fields propagate errors without source lines or credential values.
- `load_current_profile_marker` reads a declared marker only. A simplified profile map has no stored current marker; the compatibility parser's in-memory default does not imply an active profile. Missing files return no marker; corrupt or unreadable files return errors.
- Repository `load`, `snapshot`, and mutation closures project current intent from the same catalog rows. An existing simplified map without a marker exposes an empty current value. Service list/get-by-name results have `is_current = false`; `get_current` uses the existing no-current error. The low-level compatibility parser keeps its original first-profile default.
- An unrelated patch, typed update, or autofix preserves the simplified map's absent markers. Explicit activation writes current/default metadata together. Explicit first creation keeps the legacy initialization defaults; adding to an existing empty inactive document does not activate it.
- Claude retains the valid file-marker → valid registry-marker precedence. `resolve_file_current_profile` returns a current name and an optional repair suggestion. `ClaudePlatform::reconcile_current_profile` performs explicit repair.
- Codex and Grok retain their runtime-match checks and registry-first precedence. A stale marker can yield no active profile while the stored marker remains unchanged. Diagnostics keep the evidence needed for an explicit repair or off operation.
- An empty current marker denotes inactive state. Profile CRUD must preserve that state, including adding the first profile to an existing empty inactive document. Grok save/delete must read the declared marker inside the resource lock; the synthetic parser default for a simplified map is not activation intent.
- `update_current_config` retains the missing-file no-op for legacy apply workflows. Invalid contents and read failures propagate instead of being treated as missing.

## Partial-edit contract

- A patch applies to the current document inside the resource lock. Unknown profile fields and unknown settings are retained unless explicitly edited. TOML extension values such as datetimes remain unchanged through the ProfileConfig JSON projection.
- Renaming requires an existing source and an unused target. Successful rename updates current/default references in the same commit. Empty or reserved profile names are rejected.
- Unsupported provider types fail without echoing the rejected value. The historical `third_party` alias remains accepted.
- Legacy `ConfigService::update_config` replaces typed fields and retains omitted extension fields. New editors use a patch with the read token.
- Auth rules stay in the platform owner. Pass the platform's validator to `patch_config`; do not apply legacy API-key validation to every auth mode.
- Invalid patches, stale versions, missing sources, and rename collisions do not change the target or backups. No-op updates do not rewrite the file.
- A successful explicit no-op mutation rechecks the content token under the guarded leaf lock and enforces the existing secret-file permission policy through the open file handle. Unix broad modes become 0600; stricter owner-only modes and existing Windows DACLs remain unchanged. Bytes, inode, modification time, and backups are preserved. Queries and rejected mutations do not change permissions.
- Metadata-only hardening checks active journal versions before changing permissions and adds no content rollback entry. A journal with no content entries does not undo that repair. Existing content entries retain their original metadata restoration contract, including the recorded prior mode; the no-op helper does not change that contract.

## Required evidence

- Two independent child processes use platform/service and desktop-service/service adapters against the same path. Both wait for a held resource lock; after release, both independent profile changes persist. Repeat each pair three times. Do not replace these tests with a serial test runner.
- Snapshot file paths, bytes, and modification times around list/current/validate on missing, corrupt, incomplete, and read-only fixtures. Queries create no repository lock.
- Reverse registry order and assert explicit platform paths stay fixed; the documented legacy adapter stays Claude.
- Cover wrong field types, missing source, duplicate target, invalid name, stale token, unknown fields, current/default rename, explicit deletion, auth validator failure, and secret-safe errors.
- Force a leaf-only replacement between repository read and commit. CAS rejects the stale mutation and preserves the external bytes without a backup.
- Cover the same leaf conflict during a semantic no-op. Verify 0644 becomes 0600 without content, inode, mtime, or backup changes; retain 0400. Rejected mutations and queries retain existing permissions. An abandoned journal with no content entries retains the hardened mode; a journal version mismatch blocks hardening.
- Cover missing-file deactivation and corruption propagation. Keep existing profile-off behavior tests.
- Query a simplified map through the service and snapshot APIs, then apply an unrelated patch and typed update. Assert no current profile and no new marker keys; explicit activation must still work.
- On Windows, compare ordinary/verbatim drive and UNC resource identities and run the canonical-path/service process pair. Cover Grok edit/delete on simplified maps with no current marker and first creation in an existing empty inactive document.

Run `cargo test -p ccr-config`, package Clippy with all targets/features and warnings denied, affected CLI/Codex platform tests, `just fmt-check`, and `git diff --check`. Broader application/Tauri integration remains part of the parent task.
