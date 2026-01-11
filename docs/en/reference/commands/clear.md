# clear - Clear CCR Configuration

Clear environment variables written by CCR from Claude Code's `settings.json`, restoring it to default state.

## Usage

```bash
ccr clear [OPTIONS]
```

## Options

- `-f, --force`: Skip confirmation prompt, clear directly (dangerous operation)

## Features

- 🧹 **Clear Environment Variables** - Remove `ANTHROPIC_*` and other platform-related environment variables
- 🔒 **Safe Confirmation** - Requires user confirmation by default (unless using --force)
- 💾 **Auto Backup** - Automatically backup `settings.json` before clearing
- 📋 **History Logging** - Record clear operations to audit log
- ⚠️ **Impact Notice** - Clearly inform about post-clear effects

## Examples

### Basic Usage

```bash
# Clear configuration (requires confirmation)
ccr clear

# Force clear (skip confirmation)
ccr clear --force

# Use short option
ccr clear -f
```

## Sample Output

### Normal Clear Flow

```bash
$ ccr clear

Clear CCR Configuration
══════════════════════════════════════════════════════════════

▶ Current Configuration Info

Platform: Claude
Current Profile: anthropic
Environment Variables: 3

Environment variables to be cleared:
  • ANTHROPIC_API_KEY
  • ANTHROPIC_BASE_URL
  • ANTHROPIC_MODEL

⚠ Warning
══════════════════════════════════════════════════════════════

Post-clear effects:
  • Claude Code will not be able to use API (until reconfigured)
  • Environment variables in settings.json will be cleared
  • Profiles in configuration file are not affected

Recommended actions:
  • Export configuration backup before clearing
  • Run 'ccr switch <name>' to reapply configuration after clearing

? Confirm clear CCR configuration? [y/N]: y

▶ Execute Clear
══════════════════════════════────────────────────────────────

✓ Created backup: ~/.claude/backups/settings.json.20250110_143022.bak
✓ Cleared environment variables: 3
✓ Saved settings.json

──────────────────────────────────────────────────────────────

✓ CCR configuration cleared

ℹ Next steps:
  • Run 'ccr list' to view available configurations
  • Run 'ccr switch <name>' to apply new configuration
  • Or directly edit settings.json for manual configuration
```

### Force Clear (--force)

```bash
$ ccr clear --force

Clear CCR Configuration
══════════════════════════════════════════════════════════════

▶ Execute Clear (--force mode)
⚠ Skipping confirmation, clearing directly

✓ Created backup: ~/.claude/backups/settings.json.20250110_143522.bak
✓ Cleared environment variables: 3
✓ Saved settings.json

──────────────────────────────────────────────────────────────

✓ CCR configuration cleared

⚠ Important notice:
  • Claude Code cannot use API now
  • Run 'ccr switch <name>' to reapply configuration
```

### Nothing to Clear

```bash
$ ccr clear

Clear CCR Configuration
══════════════════════════════────────────────────────────────

▶ Check Current Configuration

ℹ No CCR-written environment variables in settings.json

✓ Nothing to clear
```

### Cancel Clear

```bash
...
? Confirm clear CCR configuration? [y/N]: n

ℹ Clear cancelled
```

## Use Cases

### Case 1: Reset Claude Code Configuration

```bash
# Clear all CCR configuration
ccr clear

# Reapply new configuration
ccr switch new_config
```

### Case 2: Troubleshooting

```bash
# Clear potentially corrupted configuration
ccr clear

# Verify clear result
cat ~/.claude/settings.json

# Reapply configuration
ccr switch anthropic
```

### Case 3: Switch Work Environment

```bash
# Clear current configuration
ccr clear

# Switch to new environment
ccr platform switch codex
ccr switch codex_config
```

### Case 4: Cleanup Before Uninstalling CCR

```bash
# Export configuration backup
ccr export -o backup.toml

# Clear CCR configuration
ccr clear --force

# Now safe to uninstall CCR
```

## What Gets Cleared

### Environment Variables Cleared (varies by platform)

**Claude Code Platform:**
- `ANTHROPIC_API_KEY`
- `ANTHROPIC_BASE_URL`
- `ANTHROPIC_MODEL`
- Other `ANTHROPIC_*` variables

**Codex Platform:**
- `OPENAI_API_KEY`
- `OPENAI_BASE_URL`
- `OPENAI_MODEL`
- Other `OPENAI_*` variables

**Gemini Platform:**
- `GOOGLE_API_KEY`
- `GOOGLE_BASE_URL`
- `GOOGLE_MODEL`
- Other `GOOGLE_*` variables

### What's NOT Cleared

- ✅ CCR configuration file (`~/.ccr/config.toml` or `~/.ccs_config.toml`)
- ✅ Profile configurations (`~/.ccr/platforms/*/profiles.toml`)
- ✅ History records (`~/.ccr/history/`)
- ✅ Backup files (`~/.ccr/backups/`)
- ✅ Other settings in `settings.json`

## Before and After Comparison

### settings.json Before Clear

```json
{
  "env": {
    "ANTHROPIC_API_KEY": "sk-ant-xxx",
    "ANTHROPIC_BASE_URL": "https://api.anthropic.com",
    "ANTHROPIC_MODEL": "claude-3-5-sonnet-20241022",
    "OTHER_VAR": "value"
  },
  "other_settings": {
    ...
  }
}
```

### settings.json After Clear

```json
{
  "env": {
    "OTHER_VAR": "value"
  },
  "other_settings": {
    ...
  }
}
```

**Note:** Only CCR-managed environment variables are cleared, other settings remain unchanged.

## Backup Mechanism

### Auto Backup

`settings.json` is automatically backed up before each clear:

```bash
Backup location: ~/.claude/backups/settings.json.20250110_143022.bak
          or
          ~/.ccr/backups/claude/settings.json.20250110_143022.bak
```

### Backup File Naming

```
settings.json.<timestamp>.bak
```

Timestamp format: `YYYYMMDD_HHMMSS`

### Restore Backup

To restore:

```bash
# Find the most recent backup
ls -lt ~/.claude/backups/ | head -5

# Restore backup
cp ~/.claude/backups/settings.json.20250110_143022.bak ~/.claude/settings.json

# Verify restoration
cat ~/.claude/settings.json
```

## Post-Clear Effects

### ⚠️ Immediate Effects

- ❌ Claude Code cannot connect to API (needs reconfiguration)
- ❌ Cannot use `claude` command for conversations
- ✅ Claude Code itself can still run (just can't call API)

### ✅ Not Affected

- ✅ CCR configuration files still exist
- ✅ Profile configurations still available
- ✅ History records fully preserved
- ✅ Can reapply configuration anytime

## Restore Configuration

### Method 1: Reapply Profile (Recommended)

```bash
# View available configurations
ccr list

# Apply configuration
ccr switch anthropic
```

### Method 2: Restore from Backup

```bash
# Restore most recent backup
cp ~/.claude/backups/settings.json.20250110_143022.bak ~/.claude/settings.json
```

### Method 3: Manually Edit settings.json

```bash
# Direct edit
vim ~/.claude/settings.json

# Add necessary environment variables
{
  "env": {
    "ANTHROPIC_API_KEY": "sk-ant-xxx",
    "ANTHROPIC_BASE_URL": "https://api.anthropic.com",
    "ANTHROPIC_MODEL": "claude-3-5-sonnet-20241022"
  }
}
```

## FAQ

### Q: Can Claude Code still work after clearing?

**A:** Claude Code program itself can run, but cannot call API:
- ✅ Can run `claude` command
- ❌ Cannot have AI conversations (needs reconfiguration)

### Q: Will clearing delete my configurations?

**A:** No, clearing only affects `settings.json`:
- ❌ Won't delete CCR configuration files
- ❌ Won't delete Profile configurations
- ❌ Won't delete history records
- ✅ Only clears environment variables in `settings.json`

### Q: How to undo the clear operation?

**A:** Two methods:
```bash
# Method 1: Reapply configuration
ccr switch anthropic

# Method 2: Restore from backup
cp ~/.claude/backups/settings.json.*.bak ~/.claude/settings.json
```

### Q: What preparation is needed before clearing?

**A:** Recommended steps:
```bash
# 1. Export configuration backup (optional)
ccr export -o backup.toml

# 2. Remember current configuration name
ccr current

# 3. Execute clear
ccr clear

# 4. Reapply configuration
ccr switch <previous_config_name>
```

### Q: What's the difference between clear and delete?

**A:**
- **clear**: Clears environment variables in `settings.json`, doesn't affect CCR configuration files
- **delete**: Deletes specific profile from CCR configuration file, doesn't affect `settings.json`

## Best Practices

### 1. Export Backup Before Clearing

```bash
# Good habit: export before clearing
ccr export -o backup-before-clear.toml
ccr clear
```

### 2. Remember Current Configuration

```bash
# Note current configuration before clearing
ccr current  # Remember the output configuration name

# Clear
ccr clear

# Restore
ccr switch <previous_config_name>
```

### 3. Use Force Mode (in scripts)

```bash
#!/bin/bash
# Use --force in automation scripts to avoid interaction

ccr clear --force
ccr switch new_config
```

### 4. Verify After Clearing

```bash
# Check after clearing
cat ~/.claude/settings.json

# Confirm environment variables are cleared
ccr clear
```

## Notes

::: danger Dangerous Operation
- Claude Code will not be able to use API after clearing
- Using `--force` will clear immediately, skipping confirmation
- Clearing is irreversible (unless restored from backup)
:::

::: warning Warning
- Clearing only affects `settings.json`, not CCR configuration
- Need to run `ccr switch` to apply configuration after clearing
- Auto backup files will take up disk space
:::

::: tip Recommendation
- Use `ccr export` to export configuration before clearing
- Reapply configuration immediately after clearing
- Periodically clean expired backup files
- Use `--force` in automation scripts
:::

## Related Commands

- [switch](./switch) - Apply configuration to settings.json
- [current](./current) - View current configuration
- [export](./export) - Export configuration backup
- [clean](./clean) - Clean expired backup files
- [validate](./validate) - Validate configuration integrity
