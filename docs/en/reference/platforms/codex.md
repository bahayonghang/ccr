# Codex (GitHub Copilot) Platform

## Overview

Codex platform support for GitHub Copilot CLI configuration management with powerful multi-account management features.

## Quick Start

```bash
ccr platform init codex
ccr platform switch codex
ccr add
```

## Multi-Account Management

CCR provides comprehensive multi-account management for Codex CLI, allowing seamless switching between different GitHub accounts.

### Basic Commands

```bash
# Save current login as a named account
ccr codex auth save work

# Save with description
ccr codex auth save personal -d "Personal GitHub account"

# Save with expiry time
ccr codex auth save temp --expires-at 2026-02-01T00:00:00Z

# Force overwrite existing account
ccr codex auth save work --force

# List all saved accounts
ccr codex auth list

# Switch to a specific account
ccr codex auth switch work

# Show current account info
ccr codex auth current

# Delete an account
ccr codex auth delete old-account

# Delete without confirmation
ccr codex auth delete old-account --force
```

### Export & Import

```bash
# Export all accounts to Downloads folder
ccr codex auth export

# Export without sensitive data (tokens)
ccr codex auth export --no-secrets

# Import accounts from file (interactive)
ccr codex auth import

# Import in replace mode (overwrite existing accounts)
ccr codex auth import --replace

# Import with force (overwrite in merge mode)
ccr codex auth import --force
```

**Import Modes:**
- **Merge (default)**: Skip existing accounts, only add new ones
- **Merge + --force**: Overwrite existing accounts with imported data, show overwritten list
- **Replace (--replace)**: Always overwrite accounts with the same name

**Features:**
- 🟢 Token freshness indicators: Fresh (<1 day) | 🟡 Stale (1-7 days) | 🔴 Old (>7 days)
- 📧 Email masking for privacy (e.g., `use***@example.com`)
- 🔒 Automatic backup rotation, keeps last 10 backups
- ⚠️ Process detection warnings before switching
- 🔐 Auto-set auth file permissions to 0600 on Unix systems

### Interactive TUI

Launch the Codex account management interface:
```bash
ccr codex
```

**Keyboard Shortcuts:**
| Key | Action |
|-----|--------|
| `↑` / `↓` / `j` / `k` | Select account |
| `Enter` | Switch to selected account and exit |
| `Space` | Switch to selected account (stay in TUI) |
| `q` / `Esc` | Quit |

## Configuration

```toml
[profiles.github-copilot]
base_url = "https://api.github.com/copilot"
auth_token = "ghp_..."
model = "gpt-4"
provider = "github"
```

## See Also

- [Platform Overview](./index)
- [Claude Code Platform](./claude)
- [Multi-Platform Setup](../examples/multi-platform-setup)
- [Migration Guide](./migration) - Migrating between platforms

## Related Commands

```bash
# Multi-account management
ccr codex auth save <name>   # Save current login as named account
ccr codex auth list          # List all saved accounts
ccr codex auth switch <name> # Switch to specific account
ccr codex auth current       # Show current account info
ccr codex auth delete <name> # Delete account
ccr codex auth export        # Export accounts to file
ccr codex auth import        # Import accounts from file
ccr codex                    # Launch interactive TUI
```

---

**For complete documentation, see the [Chinese version](../../reference/platforms/codex).**
