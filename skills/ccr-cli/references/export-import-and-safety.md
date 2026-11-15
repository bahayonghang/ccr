# ExchangeV1 Export/Import, 6+1 Buckets & Transactional Safety

## Table of Contents

- [1. ExchangeV1 File Format Contract](#1-exchangev1-file-format-contract)
- [2. Redaction Rules (`--no-secrets`) & Export Flags](#2-redaction-rules---no-secrets--export-flags)
- [3. Three-Format Import Compatibility & Validation](#3-three-format-import-compatibility--validation)
- [4. The 6+1 Bucket Conservation Model (`ProfileImportReport`)](#4-the-61-bucket-conservation-model-profileimportreport)
- [5. Conflict, Deduplication & Replace Strategies](#5-conflict-deduplication--replace-strategies)
- [6. Active Profile Protection, Backups & Rollback](#6-active-profile-protection-backups--rollback)

---

## 1. ExchangeV1 File Format Contract

`ccr <claude|codex|grok> profile export` emits `ExchangeV1` (`schema_version = 1`) TOML designed for cross-machine sharing and team templates:

```toml
# CCR Profile Exchange File
schema_version = 1
platform = "claude"
exported_at = "2026-04-19T08:00:00Z"
secrets_mode = "redacted"

[profiles.openrouter-sonnet]
description = "Team OpenRouter relay"
base_url = "https://openrouter.ai/api"
model = "anthropic/claude-sonnet-4"
auth_mode = "api_key"
provider = "openrouter"
provider_type = "third_party_model"
tags = ["team", "relay"]
enabled = true
```

Key format guarantees:

- Catalog-local state (`current_config`, `default_config`, `[settings]`, `usage_count`) is stripped on export so importing never hijacks the receiver's active profile pointer unexpectedly.
- Official OAuth session files (such as `~/.codex/auth.json` ChatGPT tokens) are **never** embedded in profile exports.
- Profile tables are serialized in deterministic key order under `[profiles.<name>]`.

---

## 2. Redaction Rules (`--no-secrets`) & Export Flags

### Export Command Patterns

- Stream redacted TOML to stdout for inspection (L0 auto-allowed; note `--stdout` and `--json` are mutually exclusive):
  `ccr <platform> profile export --no-secrets --stdout`
- Write redacted bundle to a file with structured JSON receipt (`ExportProfilesJsonReport`, L0 auto-allowed when target file does not exist):
  `ccr <platform> profile export --no-secrets -o ./team-profiles.export.toml --json`
- Filter specific profiles by positional `NAME...` or by repeatable `-t` / `--tag <TAG>` (positional `NAME...` and `--tag` are mutually exclusive; there is no `--profile` flag):
  - By name: `ccr <platform> profile export work-a work-b --no-secrets -o ./subset.export.toml --json`
  - By tag: `ccr <platform> profile export --tag team --tag relay --no-secrets -o ./tagged.export.toml --json`

### What `--no-secrets` Redacts

1. **Preserves explicit `auth_mode` first**: Before stripping credentials, CCR materializes `auth_mode` (for example `api_key` on `claude` or `inline_api_key` / `env_key` on `grok`) so the receiving machine knows whether credentials are required.
2. **Removes credential fields & unknown extensions**: Deletes inline keys/tokens (including Codex sidecar overlays) and drops non-whitelisted `extra` keys so custom fields cannot smuggle secrets.
3. **Sanitizes `base_url`**: Strips `userinfo@` credentials, `#fragment` tokens, and sensitive query parameters matching `SENSITIVE_QUERY_KEYS` (`key`, `api_key`, `apikey`, `x-api-key`, `token`, `auth`, `auth_token`, `access_token`, `refresh_token`, `id_token`, `client_secret`, `secret`, `password`, `passwd`, `pwd`, `sig`, `signature`), while preserving non-secret query parameters such as `?api-version=2024-10-21`.

### Secret-Bearing Export Restrictions

- Omit `--no-secrets` **only** when the operator explicitly asks for a full-credential migration file written directly to disk (`-o <FILE>` or default `./ccr-<platform>-profiles-<YYYYMMDD-HHMMSS>.toml`, written with `0600` permissions on Unix).
- **Never** combine secret-bearing export with `--stdout`, and **never** read the resulting file with agent file tools or shell commands.
- In `--json` or non-TTY mode, `-o <EXISTING_FILE>` fails closed with exit code `64` unless the operator explicitly authorizes overwrite (`ccr -y <platform> profile export ...`).

---

## 3. Three-Format Import Compatibility & Validation

`ccr <claude|codex|grok> profile import <FILE> --dry-run --json` auto-detects three TOML layouts (`source_format` in JSON output):

1. `exchange_v1`: Canonical `schema_version = 1` files (`platform` must match target `<claude|codex|grok>`; cross-platform imports fail immediately).
2. `legacy_catalog`: Historical `profiles.toml` files containing top-level `default_config` / `current_config` / `[settings]` alongside profile tables.
3. `legacy_simplified`: Flat `[<profile_name>]` tables without `ExchangeV1` headers.

Hard input safety checks performed before any write:

- **Symlink / reparse-point rejection**: `import <FILE>` inspects `symlink_metadata` and refuses symbolic links or Windows reparse points (use `--stdin` only from trusted non-secret content).
- **Masked placeholder rejection**: Any credential field containing legacy masked strings (`***`, `...`, `redacted`, `<redacted>`, `[redacted]`, `sk-ant-...***`, etc.) is rejected deterministically so corrupted placeholders never overwrite working credentials.

---

## 4. The 6+1 Bucket Conservation Model (`ProfileImportReport`)

Every `profile import --dry-run --json` and `profile import --json` returns `ProfileImportReport`. Numeric counters live under `counts.*`, paired with top-level name/item detail arrays:

### Primary 6 Input-Partition Buckets (Merge Mode Conservation)

In default `merge` mode (without `--replace`), every profile in the input file lands in **exactly one** of these 6 `counts.*` buckets:

```text
counts.added + counts.updated + counts.unchanged + counts.renamed + counts.skipped_conflict + counts.skipped_duplicate == counts.total_input
```

(`counts.total_incoming` records normalized incoming candidates, and `counts.skipped == counts.skipped_conflict + counts.skipped_duplicate`.)

- `counts.added` (`added_names: string[]`): New profile names inserted into the catalog.
- `counts.updated` (`updated_names: string[]`): Existing profile names overwritten via `--on-conflict overwrite`.
- `counts.unchanged` (`unchanged_names: string[]`): Input entry is already semantically identical to the existing same-named local entry.
- `counts.renamed` (`renamed_items: [{ from, to }]`): Name collision resolved via `--on-conflict rename` into `<name>-imported`, then `<name>-imported-2`, `<name>-imported-3`, etc.
- `counts.skipped_conflict` (`skipped_conflicts: string[]`): Same-named local entry differs and `--on-conflict skip` (default) kept the local profile untouched.
- `counts.skipped_duplicate` (`skipped_duplicates: [{ name, duplicate_of }]`): `--dedup` detected an existing local profile (`duplicate_of`) under a **different name** with the identical canonical configuration fingerprint and skipped creating a duplicate.

### Plus Auxiliary Safety Buckets (`+1` / Side Counters & Receipts)

- `counts.removed` (`removed_names: string[]`): Local profiles deleted when running `--replace` full-catalog replacement (`0` in merge mode).
- `counts.disabled_missing_secret` (`missing_secrets: string[]`): Imported profiles whose `auth_mode` requires a secret that was omitted (`secrets_mode = "redacted"`). CCR imports them safely with **`enabled = false`** so broken credentials cannot be switched on accidentally.
- `preserved_local_secrets` (`string[]`): When `--on-conflict overwrite` updates an existing local profile from a redacted file and `auth_mode` remains compatible, CCR preserves the local secret rather than wiping it.
- Metadata & rollback fields: `ok`, `platform`, `mode` (`"merge"` | `"replace"`), `on_conflict`, `dedup`, `dry_run`, `changed`, `source_format`, `reapplied_profile`, `backup_path`, `base_revision`, `revision`, `warnings`.

---

## 5. Conflict, Deduplication & Replace Strategies

1. **Mandatory L1 Dry-Run First**:
   `ccr <platform> profile import <FILE> --dry-run --json`
2. **Conflict Resolution (`--on-conflict <skip|overwrite|rename>`, default `skip`)**:
   - `skip`: Safest default; keeps existing local profiles unchanged on name collision (`skipped_conflicts`).
   - `rename`: Allocates `<name>-imported` first, then `<name>-imported-2`, `<name>-imported-3`, ... (`renamed_items`) without touching existing profiles.
   - `overwrite`: Updates matching local profiles. **Requires explicit operator confirmation** after reviewing `updated_names` in dry-run output, followed by global `-y` (`ccr -y <platform> profile import <FILE> --on-conflict overwrite --json`).
3. **Content Deduplication (`--dedup`)**:
   - Computes a canonical normalized fingerprint (`base_url` with trailing slashes normalized, `model`, `provider`, `auth_mode`, and platform routing fields) and skips entries already present under another profile name (`skipped_duplicates`).
4. **Full Replacement (`--replace`)**:
   - Mutually exclusive with `--on-conflict` and `--dedup`. Deletes local profiles absent from `<FILE>`.
   - **Requires explicit operator confirmation** after listing `removed_names` and `updated_names` from `--dry-run --json`, then requires global `-y` in `--json` mode (`ccr -y <platform> profile import <FILE> --replace --json`).
5. **Catalog-Only Import (`--no-apply`)**:
   - Pass `--no-apply` when `overwrite` touches the currently active profile name if the operator wants to stage catalog updates without rewriting live host CLI runtime files yet.

---

## 6. Active Profile Protection, Backups & Rollback

- **Active profile guard**: `profile import` refuses to `--replace`-remove the currently active profile and refuses to flip the currently active profile to `enabled = false` due to missing secrets. Switch to another profile or run `ccr <platform> profile off --json` (with operator approval) first.
- **CAS concurrency guard**: Pass `--expected-revision <REV>` (obtained from `ccr <platform> profile list --json`) so concurrent edits fail closed with `RevisionConflict` before touching disk.
- **Transactional rollback**: Every real import acquires `operation_lock`, records a `WriteJournal`, creates a `0600` backup (`backup_path` returned in JSON), and executes inside `CatalogTransaction` (covering `profiles.db`, `profiles.toml`, and Codex `profile_secrets.json`). If runtime re-application fails mid-flight, CCR rolls the catalog transaction back automatically.
