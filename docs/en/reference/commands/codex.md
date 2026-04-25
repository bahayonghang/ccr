# codex - Codex Multi-Account Management

`ccr codex` is the Codex-specific command group. Its main user-facing surfaces today are `auth` and `sync-history`.

## Usage

```bash
ccr codex
ccr codex auth <ACTION> [OPTIONS]
ccr codex sync-history --provider <ID> [--keep <N>] [--max-age-days <DAYS>] [--dry-run] [--codex-home <PATH>]
ccr codex sync-history status
ccr codex sync-history restore <BACKUP_DIR> [--restore-state]
ccr codex sync-history prune-backups [--keep <N>]
```

## Supported Subcommands

### `ccr codex`

Without a subcommand, CCR enters the default Codex interaction path; when the TUI feature is enabled, it can act as the Codex account-management entrypoint.

### `ccr codex auth`

| Subcommand | Purpose |
|------------|---------|
| `save <name>` | Save the current `~/.codex/auth.json` as a named account |
| `list` | List saved accounts |
| `switch <name>` | Switch to a saved account |
| `delete <name>` | Delete a saved account |
| `current` | Show the current account |
| `export` | Export accounts to encrypted JSON |
| `import` | Import accounts from JSON (auto-detects encrypted/plaintext) |

### `ccr codex sync-history`

Repairs Codex history visibility after switching between provider namespaces such as official `openai` and third-party `custom`.

Scope:

- Rewrites `session_meta.payload.model_provider` in rollout files under `~/.codex/sessions` and `~/.codex/archived_sessions`
- Syncs `threads.model_provider` inside `~/.codex/state_5.sqlite`
- Conservatively restores missing sidebar project entries in `.codex-global-state.json`
- Creates recoverable backups under `.codex/backups_state/sync-history/`
- Processes only the latest 7 days by default; use `--max-age-days` to change the window
- Preserves rollout mtime after rewriting so Codex Resume `Updated` sorting is not bulk-refreshed

Supported subcommands:

| Subcommand | Purpose |
|------------|---------|
| `sync-history --provider <ID>` | Sync to an explicit target provider; if the root `model_provider` is missing, `--provider` is required |
| `sync-history --dry-run` | Preview rollout / SQLite / sidebar changes without creating backups or writing state |
| `sync-history status` | Show the current Codex runtime provider, rollout / SQLite distribution, and recent 7-day provider distribution |
| `sync-history restore <backup-dir>` | Restore only rollout provider fields recorded in the manifest by default |
| `sync-history restore <backup-dir> --restore-state` | Also restore the old `state_5.sqlite` and global state; this can overwrite thread metadata created after the backup |
| `sync-history prune-backups` | Remove old backups |

Examples:

```bash
# Preview the recent 7-day changes first
ccr codex sync-history --provider custom --dry-run

# CCR URL+Key profiles use custom as the Codex runtime provider
# Use custom to make recent openai sessions visible under a URL+Key profile
ccr codex sync-history --provider custom

# Use openai to restore the official profile view
ccr codex sync-history --provider openai

# Inspect current state
ccr codex sync-history status

# Normal restore only rolls back rollout provider fields from the manifest
ccr codex sync-history restore C:\Users\you\.codex\backups_state\sync-history\20260409T101530123Z

# Restore SQLite / global state only when you need the older full snapshot
ccr codex sync-history restore C:\Users\you\.codex\backups_state\sync-history\20260409T101530123Z --restore-state

# Keep only the latest 3 backups
ccr codex sync-history prune-backups --keep 3
```

## Examples

```bash
# Save the current login
ccr codex auth save work

# Save with description and expiration
ccr codex auth save personal -d "Personal GitHub account" --expires-at 2026-02-01T00:00:00Z

# List and switch
ccr codex auth list
ccr codex auth switch work
ccr codex auth current

# Import and export (auto-encrypts when secrets are included)
ccr codex auth export
ccr codex auth export --no-secrets
ccr codex auth import --replace
```

### Export Encryption

When exporting with secrets (OAuth tokens, API keys), the system automatically prompts for a password and encrypts the file using AES-256-GCM with Argon2id key derivation.

**Encryption Scheme:**

| Item | Details |
|------|---------|
| Cipher | AES-256-GCM (authenticated encryption) |
| Key Derivation | Argon2id (64 MB / 3 iterations / 1 parallelism) |
| Export Format | JSON envelope: readable header (version, timestamp, count) + encrypted payload |
| AAD Protection | Envelope header fields are bound as GCM authenticated data to prevent metadata tampering |
| Backward Compat | Import auto-detects old plaintext files |

**Encrypted export format (v2.0 envelope):**

```json
{
  "version": "2.0",
  "format": "encrypted",
  "exported_at": "2026-04-15T12:00:00Z",
  "account_count": 5,
  "encryption": {
    "algorithm": "aes-256-gcm",
    "kdf": "argon2id",
    "kdf_params": { "m_cost": 65536, "t_cost": 3, "p_cost": 1 },
    "salt": "<base64>",
    "nonce": "<base64>"
  },
  "encrypted_payload": "<base64>"
}
```

Readable without decryption: export time, account count, encryption parameters.
All sensitive account data (tokens, API keys) is protected inside `encrypted_payload`.

## Migrate Saved Accounts into OpenCode

If you already have a saved set of Codex accounts in CCR and want OpenCode to reuse them, move through the `opencode` command group:

```bash
# Preview which saved Codex accounts can be imported
ccr opencode auth import-codex --dry-run

# Import compatible accounts
ccr opencode auth import-codex
```

Migration guarantees:

- reads only Codex accounts already saved in CCR, not an unsaved runtime login
- imports only compatible ChatGPT OAuth-backed accounts
- never overwrites existing OpenCode accounts
- never switches the current OpenCode runtime login
- reports skipped accounts by reason

## When to Use It

- One developer manages multiple GitHub / Codex identities
- A shared machine needs explicit account switching
- You want to export or import Codex auth state for backup or migration (cross-device transfers are encrypted by default)

## Related Docs

- [`opencode`](./opencode)
- [Platform Support](/en/reference/platforms/)
- [UI Module Map](/en/guide/ui-modules)
