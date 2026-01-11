# sessions

Session management commands for parsing, indexing, and managing session history from AI CLI tools.

## Overview

The `ccr sessions` command group provides management functionality for session history from Claude, Codex, Gemini, and other AI CLI tools:

- **Index**: Scan and index local session files
- **List**: View historical sessions
- **Search**: Search sessions by keywords
- **Resume**: Generate commands to resume sessions
- **Statistics**: View index statistics

## Supported Platforms

| Platform | Session Path | Format |
|----------|--------------|--------|
| Claude | `~/.claude/projects/**/*.jsonl` | JSONL |
| Codex | `~/.codex/sessions/*.jsonl` | JSONL |
| Gemini | `~/.gemini/tmp/*` | Custom format |
| Qwen | `~/.qwen/sessions/*.jsonl` | JSONL |
| iFlow | `~/.iflow/sessions/*.jsonl` | JSONL |

## Subcommands

### list

List session history.

```bash
ccr sessions list [OPTIONS]
```

**Options:**

| Option | Description | Default |
|--------|-------------|---------|
| `-p, --platform <PLATFORM>` | Filter by platform (claude/codex/gemini/qwen/iflow) | All |
| `-l, --limit <N>` | Limit display count | 20 |
| `--today` | Show only today's sessions | No |

**Examples:**

```bash
# List most recent 20 sessions
ccr sessions list

# Show only Claude sessions
ccr sessions list --platform claude

# Show today's sessions
ccr sessions list --today

# Show most recent 50 sessions
ccr sessions list --limit 50
```

### search

Search sessions.

```bash
ccr sessions search <QUERY> [OPTIONS]
```

**Arguments:**

| Argument | Description |
|----------|-------------|
| `<QUERY>` | Search keyword |

**Options:**

| Option | Description | Default |
|--------|-------------|---------|
| `-p, --platform <PLATFORM>` | Filter by platform | All |
| `-l, --limit <N>` | Limit result count | 10 |

**Examples:**

```bash
# Search sessions containing "refactoring"
ccr sessions search "refactoring"

# Search in Claude sessions
ccr sessions search "bug fix" --platform claude
```

### show

View session details.

```bash
ccr sessions show <SESSION_ID>
```

**Output Information:**

- Session ID
- Platform
- Title
- Working directory
- File path
- Created/Updated time
- Message statistics (user/assistant/tool calls)
- Resume command

### resume

Generate command to resume session.

```bash
ccr sessions resume <SESSION_ID> [OPTIONS]
```

**Options:**

| Option | Description |
|--------|-------------|
| `--dry-run` | Only print command, don't execute |

**Examples:**

```bash
# Generate resume command
ccr sessions resume abc123

# Only print resume command
ccr sessions resume abc123 --dry-run
```

### reindex

Rebuild session index.

```bash
ccr sessions reindex [OPTIONS]
```

**Options:**

| Option | Description |
|--------|-------------|
| `--force` | Force rebuild (clear existing index) |
| `-p, --platform <PLATFORM>` | Only index specified platform |

**Examples:**

```bash
# Incremental update index
ccr sessions reindex

# Force complete rebuild
ccr sessions reindex --force

# Only rebuild Claude index
ccr sessions reindex --platform claude
```

### stats

Display index statistics.

```bash
ccr sessions stats
```

**Output:**

- Total session count
- Statistics by platform

### prune

Clean up expired session records (files already deleted).

```bash
ccr sessions prune [OPTIONS]
```

**Options:**

| Option | Description |
|--------|-------------|
| `--confirm` | Skip confirmation prompt |

## Data Storage

Session index is stored in SQLite database:

```
~/.ccr/data.db
```

The database is automatically created and initial indexing is performed on first run of `ccr sessions` command.

## Use Cases

### Quickly Resume Interrupted Session

```bash
# View today's sessions
ccr sessions list --today

# Find target session and resume
ccr sessions resume abc123
```

### Search Historical Conversations

```bash
# Search previously discussed topics
ccr sessions search "authentication"

# View details
ccr sessions show <session_id>
```

### Regular Maintenance

```bash
# Clean up invalid records
ccr sessions prune --confirm

# Rebuild index (when encountering issues)
ccr sessions reindex --force
```

## Version Information

- **Introduced in**: v3.12.0
- **Feature Dependency**: Requires SQLite support (enabled by default)
