# codex - Codex Multi-Account Management

`ccr codex` is the Codex-specific command group. Its main user-facing surfaces today are `auth` and `sync-history`.

## Usage

```bash
ccr codex
ccr codex auth <ACTION> [OPTIONS]
ccr codex sync-history [--provider <ID>] [--keep <N>] [--codex-home <PATH>]
ccr codex sync-history status
ccr codex sync-history restore <BACKUP_DIR>
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

Supported subcommands:

| Subcommand | Purpose |
|------------|---------|
| `sync-history` | Sync to the current root `model_provider`, or to `--provider` if provided |
| `sync-history status` | Show the current provider plus rollout / SQLite distribution |
| `sync-history restore <backup-dir>` | Restore from a backup directory |
| `sync-history prune-backups` | Remove old backups |

Examples:

```bash
# Sync to the current root model_provider in ~/.codex/config.toml
ccr codex sync-history

# Explicitly sync to custom
ccr codex sync-history --provider custom

# Inspect current state
ccr codex sync-history status

# Restore from a previous backup
ccr codex sync-history restore C:\Users\you\.codex\backups_state\sync-history\20260409T101530123Z

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
