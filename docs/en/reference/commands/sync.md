# sync - WebDAV Configuration Sync

`ccr sync` provides WebDAV-backed cloud synchronization for CCR configurations and registered host directories, with built-in defaults for Jianguoyun and compatibility with standard WebDAV servers such as Nextcloud and ownCloud.

## Usage

```bash
# Interactive setup and core sync actions
ccr sync config
ccr sync status
ccr sync push [--force] [-i]
ccr sync pull [--force]

# Registered sync-folder management
ccr sync folder list
ccr sync folder add <name> <local-path> -r <remote-path>
ccr sync folder remove <name>
ccr sync folder enable <name>
ccr sync folder disable <name>
ccr sync folder info <name>

# Batch or single-folder sync operations
ccr sync all push|pull|status
ccr sync <folder> push|pull|status
```

## Sync Modes and Smart Filtering

CCR supports two WebDAV synchronization scopes:

- **Unified directory sync (recommended default)**: syncs the `~/.ccr/` (or `CCR_ROOT`) platform registry (`config.toml`) and platform configuration assets while excluding heavy caches and temporary directories.
- **Registered folder sync (`sync folder` / `sync all`)**: manages host or custom directories in parallel (such as `~/.claude`, `~/.codex`, or `~/.gemini`).

### Automatically Excluded Paths

Sync skips local-only artifacts, SQLite WAL/SHM sidecars, non-authoritative projections, and build caches automatically:

- Directories: `backups/`, `history/`, `ccr-ui/`, `.locks/`, `.git/`
- Files and patterns: `profiles.toml`, `profiles.db-wal`, `profiles.db-shm`, `*.tmp`, `*.lock`, `*.bak`, `.DS_Store`, `Thumbs.db`, `desktop.ini`

> [!WARNING]
> **Sensitive Credentials**: Sensitive sync assets that contain plaintext bearers (such as `codex-config` for `~/.codex/config.toml`) wrap their payload inside an encrypted v2 envelope requiring an independent operation passphrase. Always use `https://` WebDAV endpoints and dedicated app passwords rather than primary account passwords.

## Subcommands

| Subcommand | Description |
|------------|-------------|
| `sync config` | Interactively configure the WebDAV URL, username, app password, and remote path, then test connectivity |
| `sync status` | Display active sync settings and remote path availability |
| `sync push [--force] [-i]` | Upload local configs to WebDAV; `-f/--force` skips overwrite confirmation, `-i` selects items interactively |
| `sync pull [--force]` | Download remote configs locally (automatically creates a local `before_pull` backup before overwriting) |
| `sync folder ...` | Register, inspect, enable/disable, or remove named sync folders |
| `sync all ...` | Run `push`, `pull`, or `status` across all enabled registered folders |

## Common Scenarios

### Scenario 1: Sync Multi-Platform Setup Across Machines

```bash
# Machine A: initialize platform catalogs and push to WebDAV
ccr claude profile init
ccr codex profile init
ccr grok profile init

ccr sync config
ccr sync push

# Machine B: configure the same WebDAV endpoint, pull, and verify
ccr sync config
ccr sync pull
ccr platform list
ccr current
ccr validate
```

### Scenario 2: Register and Batch-Sync Host Directories

```bash
ccr sync folder add claude ~/.claude -r /ccr-sync/claude
ccr sync folder enable claude
ccr sync folder list

# Inspect status and push all enabled folders
ccr sync all status
ccr sync all push
```

### Scenario 3: Offline Snapshot Archives

Alongside WebDAV sync, periodically export `ExchangeV1` snapshots for offline archiving:

```bash
ccr claude profile export --no-secrets -o ~/backups/claude-profiles.toml
ccr codex profile export --no-secrets -o ~/backups/codex-profiles.toml
```

## Current Contract Boundary

- `ccr sync` executes only the `config`, `status`, `push`, `pull`, `folder`, `all`, and registered-folder shortcuts exposed in CLI help.
- Background auto-sync daemons and server-side version history are not part of the CLI contract; run `status` before destructive `pull` or `push` operations.

## Related Commands

- [`claude`](./claude) / [`codex`](./codex) / [`grok`](./grok) - platform profile catalogs and `ExchangeV1` export/import
- [`current`](./current) - inspect current runtime status
- [`clean`](./clean) - clean up old local backups and caches
