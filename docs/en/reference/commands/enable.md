# enable - Enable Configuration

Enable a disabled configuration profile, making it available for normal switching.

## Usage

```bash
ccr enable <config_name>
```

## Arguments

- `<config_name>`: Name of the configuration to enable (required)

## Features

- ✅ **Status Management** - Modify configuration's enabled/disabled status
- 🔍 **Configuration Validation** - Check if configuration exists before enabling
- 📝 **Status Display** - Show configuration's current status
- 💾 **Persistent Save** - Update status marker in configuration file
- 📋 **History Logging** - Record enable operations to audit log

## Examples

### Basic Usage

```bash
# Enable a disabled configuration
ccr enable test_config

# Enable multiple configurations
ccr enable config1
ccr enable config2
```

## Sample Output

### Successful Enable

```bash
$ ccr enable test_provider

Enable Configuration: test_provider
══════════════════════════════════════════════════════════════

▶ Step 1/2: Check Configuration
✓ Configuration 'test_provider' exists

▶ Step 2/2: Update Status
✓ Configuration 'test_provider' enabled

ℹ Next steps:
  • Run 'ccr list' to view all configurations
  • Run 'ccr switch test_provider' to switch to this configuration
```

### Already Enabled

```bash
$ ccr enable anthropic

Enable Configuration: anthropic
══════════════════════════════════════════════════════════════

▶ Step 1/2: Check Configuration
✓ Configuration 'anthropic' exists

▶ Step 2/2: Update Status
ℹ Configuration 'anthropic' is already enabled

✓ No changes needed
```

### Configuration Not Found

```bash
$ ccr enable nonexistent

Enable Configuration: nonexistent
══════════════════════════════════════════════════════════════

▶ Step 1/2: Check Configuration
✗ Configuration section 'nonexistent' does not exist

Suggestion: Run 'ccr list' to view available configurations
```

## Use Cases

### Case 1: Enable Temporarily Disabled Configuration

```bash
# Temporarily disable a configuration (during testing)
ccr disable test_config

# Re-enable after testing
ccr enable test_config

# Switch to use
ccr switch test_config
```

### Case 2: Batch Manage Configuration Status

```bash
# View all configurations
ccr list

# Enable needed configurations
ccr enable prod_config
ccr enable backup_config

# Disable unneeded configurations
ccr disable old_config
```

### Case 3: Switch Work Environment

```bash
# Work environment configuration
ccr enable work_config
ccr switch work_config

# Switch to personal configuration after work
ccr enable personal_config
ccr switch personal_config
ccr disable work_config
```

## Configuration Status

### Enabled Status

- ✅ Can switch via `ccr switch`
- ✅ Displays normally in `ccr list`
- ✅ Can be set as current configuration
- ✅ All CCR commands can operate on it

### Disabled Status

- ❌ Cannot switch via `ccr switch`
- ⚠️ Marked as disabled in `ccr list`
- ❌ Cannot be set as current configuration
- ✅ Still preserved in configuration file

## Enable vs Delete

| Operation | Preserve Config | Recoverable | Use Case |
|-----------|----------------|-------------|----------|
| **Disable** | ✅ Preserved | ✅ Immediately available | • Temporary non-use<br>• Preserve config info |
| **Delete** | ❌ Removed | ⚠️ Need recovery | • Permanent removal<br>• Clean unused configs |

## Working with list Command

```bash
# View all configurations and their status
ccr list

# Output example:
# anthropic    [Enabled] [Current]
# test_config  [Disabled]
# prod_config  [Enabled]

# Enable disabled configuration
ccr enable test_config

# View again
ccr list

# Output example:
# anthropic    [Enabled] [Current]
# test_config  [Enabled]  ← Status changed
# prod_config  [Enabled]
```

## FAQ

### Q: Will enabling automatically switch to the configuration?

**A:** No, enabling only changes the configuration's status marker. You need to manually run:
```bash
ccr enable test_config
ccr switch test_config  # Manual switch
```

### Q: Can I enable the current configuration?

**A:** Yes, but typically the current configuration is already enabled:
```bash
ccr enable current_config  # If already enabled, will prompt no changes needed
```

### Q: Does enabling modify configuration content?

**A:** No, it only modifies the configuration's enabled/disabled status marker, doesn't affect configuration content (API Key, Base URL, etc.).

### Q: What's the difference between enable and switch?

**A:**
- `enable`: Changes configuration's **status marker** (enabled/disabled)
- `switch`: Changes **currently used configuration** (requires configuration to be enabled)

```bash
# Complete flow
ccr enable test_config   # 1. Enable configuration
ccr switch test_config   # 2. Switch to that configuration
```

## Best Practices

### 1. Configuration Category Management

```bash
# Production configurations - keep enabled
ccr enable prod_anthropic
ccr enable prod_openai

# Test configurations - enable when needed
ccr disable test_config1
ccr disable test_config2

# Personal configurations - enable/disable as needed
ccr enable personal_config
```

### 2. Regularly Check Configuration Status

```bash
# View all configurations
ccr list

# Clean up unneeded configurations
ccr disable old_config
# Or permanently delete
ccr delete old_config
```

### 3. Use Descriptive Naming

Although CCR doesn't have built-in tagging, you can distinguish by naming:

```bash
# Use prefixes to identify types
# prod_*    - Production environment
# test_*    - Test environment
# backup_*  - Backup configuration

ccr enable prod_main
ccr disable test_temp
```

### 4. Use with Automation Scripts

```bash
#!/bin/bash
# Switch to work environment

# Disable personal configuration
ccr disable personal_config

# Enable work configuration
ccr enable work_config
ccr switch work_config

echo "✓ Switched to work environment"
```

## Notes

::: tip Tip
- Enable operation is **idempotent**, enabling the same configuration multiple times won't cause errors
- Configuration still needs `switch` to be used after enabling
- Enable operation doesn't validate configuration validity (API Key, etc.)
:::

::: warning Warning
- Enable operation modifies the configuration file
- If configuration is already enabled, operation will prompt but won't error
- Enable doesn't equal switch, still need `ccr switch` to use
:::

## Related Commands

- [disable](./disable) - Disable configuration
- [list](./list) - View all configurations and status
- [switch](./switch) - Switch to specified configuration
- [add](./add) - Add new configuration
- [delete](./delete) - Delete configuration
