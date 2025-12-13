# Platform Migration Guide

## Overview

This guide covers migrating configurations and profiles between different AI CLI platforms supported by CCR: Claude Code, Codex (GitHub Copilot), and Gemini CLI.

## Table of Contents

- [Why Migrate?](#why-migrate)
- [Migration Strategies](#migration-strategies)
- [Legacy to Unified Mode](#legacy-to-unified-mode)
- [Cross-Platform Migration](#cross-platform-migration)
- [Migration Workflows](#migration-workflows)
- [Troubleshooting](#troubleshooting)
- [Rollback Procedures](#rollback-procedures)

## Why Migrate?

### Use Cases

1. **Trying Different Platforms**: Test multiple AI assistants without losing configurations
2. **Cost Optimization**: Switch between platforms based on pricing
3. **Feature Requirements**: Use platform-specific features when needed
4. **Organizational Changes**: Company switches to different AI provider
5. **Unified Management**: Manage all AI CLI tools from a single interface

### Migration Goals

- ✅ Preserve profile configurations
- ✅ Maintain operation history
- ✅ Minimize downtime
- ✅ Provide rollback capability
- ✅ Validate migrated configurations

## Migration Strategies

### Strategy 1: Platform Coexistence (Recommended)

Keep all platforms active and switch between them:

```bash
# Initialize all platforms
ccr platform init claude
ccr platform init codex
ccr platform init gemini

# Switch as needed
ccr platform switch claude   # Use Claude
ccr platform switch codex    # Use Codex
ccr platform switch gemini   # Use Gemini
```

**Pros:**
- No data loss
- Quick switching
- Independent configurations
- Easy rollback

**Cons:**
- Uses more disk space
- Requires managing multiple profile sets

### Strategy 2: Profile Migration

Copy profiles from one platform to another:

```bash
# Migrate profiles from Claude to Codex
ccr platform migrate claude codex

# Or use automatic migration
ccr platform init codex --migrate-from claude
```

**Pros:**
- Preserves profile structure
- Automated process
- Maintains history

**Cons:**
- Requires manual token replacement
- May need configuration adjustments

### Strategy 3: Manual Migration

Manually recreate configurations on new platform:

```bash
# Switch to new platform
ccr platform switch gemini

# Read old config for reference
cat ~/.ccr/claude/profiles.toml

# Create new profiles manually
ccr add  # Enter Gemini-specific details
```

**Pros:**
- Full control over migration
- Opportunity to clean up old configs
- Platform-specific optimizations

**Cons:**
- Time-consuming
- Error-prone for many profiles

## Legacy to Unified Mode

### Understanding the Transition

**Legacy Mode (CCS Compatible):**
```
~/.ccs_config.toml          # Single config file
~/.claude/settings.json     # Claude settings only
~/.claude/ccr_history.json  # Shared history
```

**Unified Mode (Multi-Platform):**
```
~/.ccr/                     # Root directory
  ├── config.toml           # Platform registry
  ├── claude/               # Claude-specific
  │   ├── profiles.toml
  │   ├── settings.json
  │   └── history.json
  ├── codex/                # Codex-specific
  │   └── ...
  └── gemini/               # Gemini-specific
      └── ...
```

### Migration Steps

#### Step 1: Backup Legacy Configuration

```bash
# Backup your existing config
cp ~/.ccs_config.toml ~/.ccs_config.toml.backup
cp ~/.claude/settings.json ~/.claude/settings.json.backup

# Backup history if exists
cp ~/.claude/ccr_history.json ~/.claude/ccr_history.json.backup 2>/dev/null || true
```

#### Step 2: Initialize Unified Mode

```bash
# Initialize Claude platform (migrates Legacy config automatically)
ccr platform init claude

# CCR will:
# 1. Create ~/.ccr/claude/ directory
# 2. Convert Legacy profiles to Unified format
# 3. Preserve existing settings
# 4. Migrate history records
```

#### Step 3: Verify Migration

```bash
# Check Claude platform is initialized
ccr platform list

# Verify profiles were migrated
ccr platform switch claude
ccr list

# Compare with legacy config
cat ~/.ccs_config.toml
cat ~/.ccr/claude/profiles.toml
```

#### Step 4: Test Migrated Configuration

```bash
# Switch to a migrated profile
ccr platform switch claude
ccr switch <profile-name>

# Verify settings are correct
ccr current

# Check environment variables
env | grep ANTHROPIC
```

#### Step 5: (Optional) Remove Legacy Files

```bash
# Only after confirming migration success!
# Keep backups for safety

# Legacy config can coexist with Unified mode
# Remove only if you're sure you don't need CCS anymore
# mv ~/.ccs_config.toml ~/.ccs_config.toml.legacy
```

### Automatic Migration Features

CCR automatically detects and migrates:

- ✅ Profile names and descriptions
- ✅ API endpoints (base_url)
- ✅ Model configurations
- ✅ Small/fast model settings
- ✅ Default profile selection
- ✅ Operation history (preserves timestamps and UUIDs)

**Not Automatically Migrated:**
- ❌ Custom environment variables (must be set manually)
- ❌ WebDAV sync settings (reconfigure after migration)
- ❌ Platform-specific features (only applies to new platforms)

## Cross-Platform Migration

### Claude → Codex Migration

#### Preparation

1. **Understand Platform Differences**:
   - Claude uses Anthropic API keys (`sk-ant-api03-...`)
   - Codex uses GitHub tokens (`ghp_...`)
   - Different API endpoints
   - Different model names

2. **Obtain Codex Credentials**:
   ```bash
   # Generate GitHub Personal Access Token
   # https://github.com/settings/tokens
   # Scopes needed: copilot, read:org (optional)
   ```

#### Migration Process

```bash
# Step 1: Initialize Codex platform
ccr platform init codex

# Step 2: Migrate profile structure
ccr platform migrate claude codex

# This creates Codex profiles with:
# - Same profile names
# - Same descriptions
# - Placeholder tokens (need replacement)
# - Codex-appropriate default models

# Step 3: Replace tokens manually
vim ~/.ccr/platforms/codex/profiles.toml

# Update auth_token for each profile:
# Before: auth_token = "sk-ant-api03-..."
# After:  auth_token = "ghp_YOUR_GITHUB_TOKEN"

# Step 4: Validate migrated config
ccr platform switch codex
ccr validate

# Step 5: Test with profile switch
ccr switch <profile-name>
ccr current
```

#### Model Mapping Suggestions

| Claude Model | Codex Equivalent |
|--------------|------------------|
| `claude-sonnet-4-5-20250929` | `gpt-4` |
| `claude-3-5-sonnet-20241022` | `gpt-4` |
| `claude-3-5-haiku-20241022` | `gpt-3.5-turbo` |
| `claude-opus-4` | `gpt-4` (or wait for GPT-4.5) |

### Claude → Gemini Migration

#### Preparation

1. **Understand Platform Differences**:
   - Claude uses Anthropic API keys
   - Gemini uses Google API keys (`AIzaSy...`)
   - Different API endpoints
   - Different model names

2. **Obtain Gemini Credentials**:
   ```bash
   # Get Google API Key
   # https://makersuite.google.com/app/apikey
   # Or Google Cloud Console
   ```

#### Migration Process

```bash
# Step 1: Initialize Gemini platform
ccr platform init gemini

# Step 2: Migrate profile structure
ccr platform migrate claude gemini

# Step 3: Replace API keys manually
vim ~/.ccr/gemini/profiles.toml

# Update auth_token for each profile:
# Before: auth_token = "sk-ant-api03-..."
# After:  auth_token = "AIzaSyXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"

# Step 4: Update base URLs (if needed)
# Default: https://generativelanguage.googleapis.com/v1beta

# Step 5: Validate migrated config
ccr platform switch gemini
ccr validate

# Step 6: Test with profile switch
ccr switch <profile-name>
ccr current
```

#### Model Mapping Suggestions

| Claude Model | Gemini Equivalent |
|--------------|-------------------|
| `claude-sonnet-4-5-20250929` | `gemini-2.0-flash-exp` |
| `claude-3-5-sonnet-20241022` | `gemini-1.5-pro` |
| `claude-3-5-haiku-20241022` | `gemini-1.5-flash` |
| `claude-opus-4` | `gemini-1.5-pro` (or `gemini-2.0-flash-exp`) |

### Codex → Gemini Migration

```bash
# Initialize Gemini
ccr platform init gemini

# Migrate profiles
ccr platform migrate codex gemini

# Update API keys
vim ~/.ccr/gemini/profiles.toml

# Replace GitHub tokens with Google API keys
# Update models (gpt-4 → gemini-2.0-flash-exp, etc.)

# Validate and test
ccr platform switch gemini
ccr validate
ccr switch <profile-name>
```

## Migration Workflows

### Workflow 1: Testing New Platform

**Scenario**: Want to try Gemini while keeping Claude active

```bash
# Current state: Using Claude
ccr platform current  # Shows "claude"

# Initialize Gemini (Claude untouched)
ccr platform init gemini

# Add test Gemini profile
ccr platform switch gemini
ccr add  # Enter test API key

# Test Gemini
ccr switch test-gemini-profile

# Switch back to Claude anytime
ccr platform switch claude
ccr switch my-claude-profile

# Both platforms coexist independently
```

### Workflow 2: Permanent Platform Switch

**Scenario**: Migrating from Claude to Codex permanently

```bash
# Step 1: Backup everything
ccr export -o full-backup.toml

# Step 2: Initialize new platform
ccr platform init codex

# Step 3: Migrate profiles
ccr platform migrate claude codex

# Step 4: Update tokens
vim ~/.ccr/platforms/codex/profiles.toml
# Replace all Anthropic keys with GitHub tokens

# Step 5: Validate
ccr platform switch codex
ccr validate

# Step 6: Set as default
vim ~/.ccr/config.toml
# Change: current_platform = "codex"
# Change: default_platform = "codex"

# Step 7: Test thoroughly
ccr switch primary-profile
ccr current

# Step 8: (Optional) Archive Claude platform
# Keep for rollback, or delete if confident
# rm -rf ~/.ccr/claude/  # Only if absolutely sure!
```

### Workflow 3: Multi-Platform Daily Use

**Scenario**: Use different platforms for different tasks

```bash
# Morning routine setup
alias work-claude='ccr platform switch claude && ccr switch work'
alias home-gemini='ccr platform switch gemini && ccr switch personal'
alias dev-codex='ccr platform switch codex && ccr switch development'

# Usage
work-claude   # Switch to Claude with work profile
home-gemini   # Switch to Gemini with personal profile
dev-codex     # Switch to Codex with dev profile

# Or use platform-profile combo (if implemented)
ccr switch claude:work
ccr switch gemini:personal
ccr switch codex:development
```

## Troubleshooting

### Issue: Migration Command Not Found

**Symptoms:**
```
Error: Unknown command 'platform migrate'
```

**Solution:**
Manual migration - copy profiles and replace tokens:

```bash
# Create new platform
ccr platform init gemini

# Copy profile structure
cat ~/.ccr/claude/profiles.toml > ~/.ccr/gemini/profiles.toml.template

# Edit and update tokens
vim ~/.ccr/gemini/profiles.toml.template
# Replace tokens and models manually

# Move to correct location
mv ~/.ccr/gemini/profiles.toml.template ~/.ccr/gemini/profiles.toml

# Validate
ccr platform switch gemini
ccr validate
```

### Issue: Profiles Lost After Migration

**Symptoms:**
Migrated platform has no profiles or missing profiles

**Solution:**

```bash
# Check if backup exists
ls -la ~/.ccr/backups/

# Restore from backup
ccr restore ~/.ccr/backups/config_20250125_120000.toml.bak

# Or recreate profiles manually from legacy config
cat ~/.ccs_config.toml.backup
ccr platform switch <platform>
ccr add  # Recreate each profile
```

### Issue: Token Validation Fails

**Symptoms:**
```
❌ Invalid token format for platform
```

**Solution:**

```bash
# Check token format for platform:

# Claude: sk-ant-api03-XXXX... (104 chars)
# Codex:  ghp_XXXX...          (40 chars)
# Gemini: AIzaSyXXXX...        (39 chars)

# Verify your tokens match these formats
# Regenerate tokens if needed
```

### Issue: Settings Not Applied

**Symptoms:**
Profile switched but environment variables not updated

**Solution:**

```bash
# Check platform directory exists
ls -la ~/.ccr/<platform>/

# Verify settings file
cat ~/.ccr/<platform>/settings.json

# Check permissions
chmod 600 ~/.ccr/<platform>/profiles.toml
chmod 600 ~/.ccr/<platform>/settings.json

# Force re-apply settings
ccr platform switch <platform>
ccr switch <profile-name>

# Verify environment
env | grep -E "(ANTHROPIC|GITHUB|GOOGLE|GEMINI)"
```

### Issue: History Not Migrated

**Symptoms:**
Old operation history missing after migration

**Solution:**

```bash
# Legacy history location
cat ~/.claude/ccr_history.json

# Unified history location (per-platform)
cat ~/.ccr/claude/history.json
cat ~/.ccr/history/codex.json

# If history missing, manually copy
cp ~/.claude/ccr_history.json ~/.ccr/claude/history.json

# Or accept that unified mode starts fresh history per platform
```

## Rollback Procedures

### Rollback: Unified → Legacy Mode

If you need to go back to Legacy mode:

```bash
# Step 1: Restore legacy config from backup
cp ~/.ccs_config.toml.backup ~/.ccs_config.toml
cp ~/.claude/settings.json.backup ~/.claude/settings.json

# Step 2: Restore legacy history if needed
cp ~/.claude/ccr_history.json.backup ~/.claude/ccr_history.json

# Step 3: (Optional) Remove unified config
# mv ~/.ccr ~/.ccr.backup  # Keep backup just in case

# Step 4: Verify legacy mode works
ccr list
ccr switch <profile-name>

# You're back to Legacy mode
# Note: CCR will still detect ~/.ccr/ if it exists
```

### Rollback: Failed Cross-Platform Migration

If migration to new platform fails:

```bash
# You can always switch back to original platform
ccr platform switch claude  # Or whichever was original

# Original platform is untouched by migration
ccr list  # Original profiles intact

# Delete failed migration if needed
rm -rf ~/.ccr/<failed-platform>/

# Or fix the migration issues and try again
vim ~/.ccr/<failed-platform>/profiles.toml
ccr validate
```

### Emergency Rollback

If something goes seriously wrong:

```bash
# Step 1: Stop all CCR operations
killall ccr 2>/dev/null || true

# Step 2: Restore from full backup
ccr import full-backup.toml --force

# Step 3: Restore settings from backup
cp ~/.claude/settings.json.backup ~/.claude/settings.json

# Step 4: Clear locks
rm -rf ~/.ccr/*/.locks/
rm -rf ~/.claude/.locks/

# Step 5: Verify restoration
ccr platform list
ccr current
```

## Best Practices

### Before Migration

1. **Create Full Backup**:
   ```bash
   ccr export -o migration-backup-$(date +%Y%m%d).toml
   ```

2. **Document Current State**:
   ```bash
   ccr platform current > current-state.txt
   ccr list >> current-state.txt
   ```

3. **Test New Platform First**:
   - Don't migrate production profiles immediately
   - Create test profile on new platform
   - Verify functionality before full migration

### During Migration

1. **Validate Each Step**:
   ```bash
   ccr validate  # After every major change
   ```

2. **Keep Backups**:
   - CCR auto-creates backups before major operations
   - Don't delete backups until migration confirmed successful

3. **Test Thoroughly**:
   - Switch to migrated profiles
   - Verify environment variables
   - Check actual API functionality

### After Migration

1. **Monitor for Issues**:
   - Check logs for first few days
   - Monitor API usage and errors

2. **Update Documentation**:
   - Document which platforms you're using
   - Note any platform-specific configurations

3. **Clean Up Gradually**:
   - Keep old platform for 1-2 weeks
   - Only delete after confirming everything works

4. **Set Up Sync**:
   ```bash
   ccr sync config
   ccr sync push  # Backup to cloud
   ```

## See Also

- [Codex Platform Guide](./codex.md) - GitHub Copilot CLI configuration
- [Gemini Platform Guide](./gemini.md) - Gemini CLI configuration
- [Main README](../../README.md) - CCR overview
- [CLAUDE.md](../../CLAUDE.md) - Multi-platform architecture details
