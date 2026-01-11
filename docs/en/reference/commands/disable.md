# disable - Disable Configuration

Disable a specified configuration profile, making it temporarily unavailable for switching while preserving the configuration information.

## Usage

```bash
ccr disable <config_name> [OPTIONS]
```

## Arguments

- `<config_name>`: Name of the configuration to disable (required)

## Options

- `-f, --force`: Force disable (even if it's the currently active configuration)

## Features

- 🚫 **Temporary Disable** - Disable configuration while preserving information
- 🔒 **Safety Check** - Detect current configuration to prevent accidental operations
- 💾 **Recoverable** - Can be re-enabled anytime with `enable` command
- 📝 **Status Marking** - Clearly shows disabled status in configuration list
- 📋 **History Logging** - Record disable operations to audit log

## Examples

### Basic Usage

```bash
# Disable specified configuration
ccr disable test_config

# Force disable (even if current configuration)
ccr disable current_config --force

# Use short option
ccr disable current_config -f
```

## Sample Output

### Successful Disable

```bash
$ ccr disable test_provider

Disable Configuration: test_provider
══════════════════════════════════════════════════════════════

▶ Step 1/3: Check Configuration
✓ Configuration 'test_provider' exists

▶ Step 2/3: Safety Check
✓ Safety check completed

▶ Step 3/3: Update Status
✓ Configuration 'test_provider' disabled

ℹ Next steps:
  • Run 'ccr list' to view all configurations
  • Use 'ccr enable test_provider' to re-enable
```

### Disable Current Configuration (Warning)

```bash
$ ccr disable anyrouter

Disable Configuration: anyrouter
══════════════════════════════════════════════════════════════

▶ Step 1/3: Check Configuration
✓ Configuration 'anyrouter' exists

▶ Step 2/3: Safety Check
⚠ Configuration 'anyrouter' is the currently active configuration

ℹ After disabling current configuration:
  1. Current settings.json will retain existing settings
  2. Cannot switch back to this configuration later (unless re-enabled)
  3. Recommend switching to another configuration first

✗ Cannot disable current configuration (use --force to force disable)
```

### Force Disable Current Configuration

```bash
$ ccr disable anyrouter --force

Disable Configuration: anyrouter
══════════════════════════════════════════════════════════════

▶ Step 1/3: Check Configuration
✓ Configuration 'anyrouter' exists

▶ Step 2/3: Safety Check
⚠ Configuration 'anyrouter' is the currently active configuration
ℹ Using --force mode, forcing disable

▶ Step 3/3: Update Status
✓ Configuration 'anyrouter' force disabled

⚠ Important notice:
  • Current settings.json still uses this configuration's settings
  • Cannot switch back to this configuration later (unless re-enabled)
  • Recommend switching to another configuration soon:
    1. ccr list        # View available configurations
    2. ccr switch <name>  # Switch to another configuration
```

### Already Disabled

```bash
$ ccr disable test_config

Disable Configuration: test_config
══════════════════════════════════════════════════════════════

▶ Step 1/3: Check Configuration
✓ Configuration 'test_config' exists

▶ Step 2/3: Safety Check
ℹ Configuration 'test_config' is already disabled

✓ No changes needed
```

### Configuration Not Found

```bash
$ ccr disable nonexistent

Disable Configuration: nonexistent
══════════════════════════════────────────────────────────────

▶ Step 1/3: Check Configuration
✗ Configuration section 'nonexistent' does not exist

Suggestion: Run 'ccr list' to view available configurations
```

## Use Cases

### Case 1: Temporarily Disable Configuration

```bash
# Temporarily disable a configuration
ccr disable test_config

# Re-enable when needed
ccr enable test_config
```

### Case 2: Clean Up Expired Configurations (Preserve Data)

```bash
# View all configurations
ccr list

# Disable expired configuration (preserve information)
ccr disable old_api_2023

# Delete if confirmed not needed
ccr delete old_api_2023
```

### Case 3: Configuration Category Management

```bash
# Keep only production configurations enabled
ccr disable test_config1
ccr disable test_config2
ccr disable dev_config

# Production configuration stays enabled
# prod_config  [Enabled]
```

### Case 4: Prevent Accidental Switching

```bash
# Disable dangerous configuration (prevent accidental switching)
ccr disable production_critical

# Enable when needed
ccr enable production_critical
ccr switch production_critical
```

## Disable vs Delete vs Current Status

| Operation | Preserve Config | Can Switch | Recoverable | Use Case |
|-----------|----------------|------------|-------------|----------|
| **Disable** | ✅ Preserved | ❌ Cannot switch | ✅ Immediately available | • Temporary non-use<br>• Prevent accidents<br>• Category management |
| **Delete** | ❌ Removed | ❌ Cannot switch | ⚠️ Need import or re-add | • Permanent removal<br>• Clean unused configs<br>• Free config space |
| **Current** | ✅ Preserved | ✅ In use | - | • Currently active configuration |

## Safety Check Mechanism

### Check 1: Configuration Existence

Verify the configuration to disable exists in the configuration file.

```bash
# Configuration exists → Continue
✓ Configuration 'test' exists

# Configuration doesn't exist → Abort
✗ Configuration section 'nonexistent' does not exist
```

### Check 2: Current Configuration Protection

By default, disabling the currently active configuration is not allowed:

```bash
⚠ Configuration 'xxx' is the currently active configuration

ℹ After disabling current configuration:
  1. Current settings.json will retain existing settings
  2. Cannot switch back to this configuration later (unless re-enabled)

✗ Cannot disable current configuration (use --force to force disable)
```

**Use --force to force disable:**

```bash
ccr disable current_config --force
```

## Configuration Status

### Before Disable

```toml
[test_config]
enabled = true   # Or no field (defaults to true)
base_url = "https://api.test.com"
auth_token = "test-token"
```

### After Disable

```toml
[test_config]
enabled = false  # Set to false
base_url = "https://api.test.com"
auth_token = "test-token"  # Configuration info preserved
```

### Re-enable

```bash
ccr enable test_config
```

```toml
[test_config]
enabled = true   # Restored to true
base_url = "https://api.test.com"
auth_token = "test-token"
```

## Display in List

```bash
$ ccr list

Configuration List
══════════════════════════════════════════════════════════════

anthropic    [Enabled] [Current]  Anthropic Official API
test_config  [Disabled]           Test Configuration
prod_config  [Enabled]            Production Configuration
```

Disabled configurations are marked as `[Disabled]` and cannot be switched via `ccr switch`.

## Restore Disabled Configuration

### Method 1: Use enable Command (Recommended)

```bash
ccr enable test_config
```

### Method 2: Manually Edit Configuration File

Edit `~/.ccs_config.toml` (Legacy mode) or `~/.ccr/platforms/<platform>/profiles.toml` (Unified mode):

```toml
[test_config]
enabled = true  # Change to true
...
```

## FAQ

### Q: Will Claude Code stop working after disabling current configuration?

**A:** No, the disable operation doesn't modify `settings.json`:
- Claude Code will continue using current settings
- But cannot switch back to this configuration later (unless re-enabled)

### Q: Can disabled configurations be restored?

**A:** Yes, use the `enable` command:
```bash
ccr enable test_config
```

### Q: What's the difference between disable and delete?

**A:**
- **Disable**: Preserves configuration information, can be enabled anytime
- **Delete**: Completely removes configuration, needs import or re-add to restore

### Q: Does disabling affect the configuration file?

**A:** It modifies the configuration's `enabled` field to `false`, but doesn't delete configuration information.

### Q: What are the risks of force disabling current configuration?

**A:** Main risks:
- Cannot switch back to this configuration later (unless re-enabled)
- Need to manually switch to another configuration

Recommendation:
```bash
# Safer approach: switch first, then disable
ccr switch other_config
ccr disable old_config
```

## Best Practices

### 1. Switch Before Disabling

```bash
# Recommended: switch to another configuration first
ccr switch new_config
ccr disable old_config

# Not recommended: force disable current configuration
ccr disable current_config --force
```

### 2. Use Disable for Configuration Categories

```bash
# Production configurations - keep enabled
# prod_anthropic  [Enabled]
# prod_openai     [Enabled]

# Test configurations - disable
ccr disable test_config1
ccr disable test_config2

# Personal configurations - enable/disable as needed
ccr disable personal_backup
```

### 3. Prevent Accidental Operations

```bash
# Disable dangerous configuration
ccr disable critical_production

# Enable when needed
ccr enable critical_production
ccr switch critical_production
```

### 4. Temporarily Unused Configurations

```bash
# Temporarily unused, might use later
ccr disable temp_config

# Confirmed unused, delete
ccr delete useless_config
```

## Notes

::: danger Dangerous Operation
- Using `--force` to force disable current configuration may prevent switching back
- Recommend switching to another configuration before disabling
:::

::: warning Warning
- Disabling configuration doesn't modify `settings.json`
- Cannot switch to disabled configuration via `ccr switch`
- Disabled configurations still occupy configuration file space
:::

::: tip Recommendation
- Use `disable` for temporarily unused configurations
- Use `delete` for permanently unused configurations
- Switch to another configuration before disabling current one
- Periodically check and clean up disabled configurations
:::

## Related Commands

- [enable](./enable) - Enable configuration
- [list](./list) - View all configurations and status
- [switch](./switch) - Switch to specified configuration
- [delete](./delete) - Delete configuration
- [add](./add) - Add new configuration
