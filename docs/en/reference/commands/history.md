# history - Inspect Operation History

Display CCR's masked operation audit log, covering profile switches, backups, validations, exports/imports, and cleanup actions.

## Usage

```bash
ccr history [OPTIONS]
```

## Options

- `-l, --limit <N>`: show the most recent `N` entries (default: `20`)
- `-t, --type <TYPE>`: filter by operation type (such as `switch`, `backup`, `validate`, `export`, `import`, `clean`, `update`)

## Recorded Fields

Each audit entry includes:

- Operation ID (UUID)
- Timestamp (UTC)
- Actor (OS username)
- Operation type (`operation_type`)
- Masked environment/configuration change summary
- Execution status (`success` / `failed`)
- Notes

## Examples

```bash
# Show the last 20 operations
ccr history

# Show the last 50 operations
ccr history --limit 50

# Filter for profile switch events only
ccr history -t switch

# Filter for validation runs only
ccr history -t validate
```

### Sample Output

```text
Operation History
════════════════════════════════════════════════════════════════
Showing last 20 operations
────────────────────────────────────────────────────────────────

[2026-01-10 12:05:30] switch
ID: 550e8400-e29b-41d4-a716-446655440000
Actor: username
From: official -> relay
Changes:
  ANTHROPIC_BASE_URL: https://api.anthropic.com -> https://api.example.com/v1
  ANTHROPIC_AUTH_TOKEN: sk-a...key -> sk-r...xyz
Result: ✓ Success
Notes: Configuration switched successfully
```

## Secret Redaction and Storage

- **Automatic masking**: API keys and auth tokens are masked before any entry is persisted (for example `sk-a...r678`), preventing credential leaks through audit history.
- **Storage location**: Unified Mode stores history under `~/.ccr/history/` (and the local store layer), while Legacy Mode uses `~/.claude/ccr_history.json`.
- **History vs. logs**: `ccr history` records high-level audit events; detailed debug traces and failure diagnostics live in daily rotated log files under `~/.ccr/logs/`.

## Related Commands

- [`current`](./current) - inspect current runtime status
- [`claude`](./claude) / [`codex`](./codex) / [`grok`](./grok) - platform profile management and switching
- [`validate`](./validate) - validate catalogs and host settings
- [`clean`](./clean) - clean up old backups and caches
