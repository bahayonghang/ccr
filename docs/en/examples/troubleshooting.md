# Troubleshooting Guide

> This page summarizes common recovery and debugging paths for CCR 7.x SQLite catalogs (`profiles.db`), runtime routing, and host file permissions. For first-time setup, start with [Quick Start](/en/guide/quick-start).

## Common Issues and Solutions

### 1. Catalog Not Initialized or Profile Not Found

**Symptom**: Switching or inspecting a profile reports a missing catalog or an unknown profile name.

**Solution**:

```bash
# 1. Initialize the target platform's SQLite catalog if needed
ccr claude profile init
ccr codex profile init
ccr grok profile init

# 2. List saved profiles for each platform
ccr claude profile list
ccr codex profile list
ccr grok profile list

# 3. Create a missing profile safely via stdin
printf '%s' "$ANTHROPIC_AUTH_TOKEN" | ccr claude profile create work \
  --base-url https://api.example.com \
  --auth-token-stdin
```

---

### 2. Manually Edited or Restored `profiles.toml` Has No Effect

**Cause**: CCR 7.x stores live profiles in `~/.ccr/platforms/<platform>/profiles.db` (SQLite). Manually creating or copying a `profiles.toml` into the platform folder will not be read by the 7.x binary.

**Solution**:

```bash
# Option A: Edit the live catalog projection in $VISUAL / $EDITOR
ccr claude profile open

# Option B: Import an external ExchangeV1 TOML file (preview with --dry-run first)
ccr claude profile import ./my-profiles.toml --dry-run
ccr claude profile import ./my-profiles.toml --on-conflict overwrite

# Re-apply and validate
ccr claude profile switch <name>
ccr validate
```

---

### 3. Runtime Drift or Stale Codex `app-server` Processes

**Symptom**: A host CLI keeps using old routing after switching profiles, shell environment variables override host settings, or a stale Codex `app-server` stays alive.

**Solution**:

```bash
# 1. Inspect runtime state and unified diagnostics
ccr current --verbose
ccr validate
ccr doctor --verbose

# 2. Check whether shell environment variables override host config files
env | grep -E 'ANTHROPIC_|OPENAI_|XAI_|CODEX_'

# 3. For Codex: clean up lingering app-server processes and replay the active profile if needed
ccr codex fix
ccr codex fix --repair-runtime

# 4. Exit profile mode to return to official login state
ccr claude profile off
ccr codex profile off
ccr grok profile off
```

---

### 4. API Connectivity Failure or Expired Credentials

**Symptom**: Connection timeouts, `401 Unauthorized`, or `INVALID_API_KEY` errors in the host CLI.

**Solution**:

```bash
# 1. Run online probing or targeted provider health checks
ccr doctor --online
ccr provider test <name> --verbose

# 2. Rotate the credential safely via stdin and re-apply
printf '%s' "$NEW_AUTH_TOKEN" | ccr claude profile edit <name> --auth-token-stdin
ccr claude profile switch <name>
```

> For Grok profiles, rotate credentials with `printf '%s' "$NEW_KEY" | ccr grok profile edit <name> --api-key-stdin` or `--set env_key=XAI_API_KEY`.

---

### 5. Permission Denied or Stale File Locks

**Symptom**: `Permission denied` errors or lock acquisition timeouts.

**Solution**:

```bash
# 1. Fix directory and host settings file permissions
chmod 755 ~/.ccr ~/.ccr/platforms
chmod 600 ~/.claude/settings.json

# 2. Remove stale lock files only after verifying no CCR process is running
ls -la ~/.claude/.locks/
rm -rf ~/.claude/.locks/*
```

---

### 6. Moving Configurations Across Machines or OS Users

**Cause**: CCR stores state under each user's `~/.ccr/` home directory by default. Use `ExchangeV1` export/import or WebDAV sync rather than copying raw SQLite files across live sessions.

**Solution**:

```bash
# Export from the source environment (always pass --no-secrets when sharing)
ccr claude profile export --no-secrets -o /tmp/claude-profiles.toml

# Preview and import on the destination environment
ccr claude profile import /tmp/claude-profiles.toml --dry-run
ccr claude profile import /tmp/claude-profiles.toml --on-conflict skip
```

## Debug Logging and Reporting Issues

```bash
# Run diagnostics with debug logging enabled
CCR_LOG_LEVEL=debug ccr doctor --all-platforms --verbose

# Inspect daily UTC-rotated log files (retained for 14 days)
ls -lh ~/.ccr/logs/

# Collect version details for bug reports (never attach unredacted host config or backup files)
ccr --version
ccr version
```

## See Also

- [Examples Overview](./index)
- [Multi-Platform Setup](./multi-platform-setup)
- [Quick Start](/en/guide/quick-start)
- [Configuration Model](/en/guide/configuration)
- [Platform Support](/en/reference/platforms/)
