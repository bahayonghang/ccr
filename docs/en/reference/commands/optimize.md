# optimize - Optimize Configuration File

Optimize the configuration file structure by rearranging configuration sections in alphabetical order to improve readability and maintainability.

## Usage

```bash
ccr optimize
```

## Features

- 📊 **Auto Sorting** - Rearrange by configuration name alphabetically
- 💾 **Preserve Content** - Only adjust order, no content modification
- 🔒 **Auto Backup** - Automatically backup before optimization
- ✅ **Integrity Validation** - Validate configuration file after optimization
- 📋 **History Logging** - Record optimization operations to audit log

## Examples

### Basic Usage

```bash
# Optimize configuration file
ccr optimize
```

## Sample Output

### Successful Optimization

```bash
$ ccr optimize

Optimize Configuration File
══════════════════════════════════════════════════════════════

▶ Step 1/4: Load Configuration
✓ Read configuration file: ~/.ccs_config.toml
✓ Parse configuration: 5 sections

▶ Step 2/4: Analyze Configuration
Section list (before optimization):
  1. anthropic
  2. test_config
  3. anyrouter
  4. backup_config
  5. openai

Section list (after optimization):
  1. anyrouter
  2. anthropic
  3. backup_config
  4. openai
  5. test_config

ℹ Will rearrange sections in alphabetical order

▶ Step 3/4: Create Backup
✓ Backup created: ~/.ccs_config.toml.20250110_150030.bak

▶ Step 4/4: Optimize and Save
✓ Rearranged configuration sections
✓ Saved optimized configuration
✓ Validated configuration integrity

──────────────────────────────────────────────────────────────

✓ Configuration file optimized

ℹ Optimization details:
  • Section count: 5
  • New order: anyrouter, anthropic, backup_config, openai, test_config
  • Backup location: ~/.ccs_config.toml.20250110_150030.bak

ℹ Next steps:
  • Run 'ccr list' to view optimized configuration list
  • Run 'ccr validate' to verify configuration integrity
```

### Already Optimal

```bash
$ ccr optimize

Optimize Configuration File
══════════════════════════════────────────────────────────────

▶ Step 1/4: Load Configuration
✓ Read configuration file: ~/.ccs_config.toml
✓ Parse configuration: 3 sections

▶ Step 2/4: Analyze Configuration
Section list (current):
  1. anthropic
  2. openai
  3. test_config

ℹ Sections are already in alphabetical order

✓ Configuration file is already optimal, no changes needed
```

### No Sections

```bash
$ ccr optimize

Optimize Configuration File
══════════════════════════════────────────────────────────────

▶ Step 1/4: Load Configuration
✓ Read configuration file: ~/.ccs_config.toml

▶ Step 2/4: Analyze Configuration
ℹ No configuration sections in file

✓ Nothing to optimize
```

## Use Cases

### Case 1: Configuration File Maintenance

```bash
# Periodically optimize configuration file for clarity
ccr optimize
```

### Case 2: After Adding Multiple Configurations

```bash
# Add multiple configurations
ccr add  # Add config1
ccr add  # Add config2
ccr add  # Add config3

# Optimize configuration order
ccr optimize

# View optimized list
ccr list
```

### Case 3: After Importing Configurations

```bash
# Import configuration file
ccr import external-config.toml --merge

# Optimize configuration file
ccr optimize
```

### Case 4: Team Collaboration

```bash
# Optimize to ensure team members see consistent order
ccr optimize

# Export standardized configuration
ccr export -o team-config.toml
```

## Before and After Comparison

### Before Optimization

```toml
[global]
default_config = "anthropic"

[test_config]
base_url = "https://api.test.com"
auth_token = "test-token"

[anthropic]
base_url = "https://api.anthropic.com"
auth_token = "sk-ant-xxx"

[anyrouter]
base_url = "https://anyrouter.top"
auth_token = "any-token"

[backup_config]
base_url = "https://api.backup.com"
auth_token = "backup-token"
```

### After Optimization

```toml
[global]
default_config = "anthropic"

[anyrouter]
base_url = "https://anyrouter.top"
auth_token = "any-token"

[anthropic]
base_url = "https://api.anthropic.com"
auth_token = "sk-ant-xxx"

[backup_config]
base_url = "https://api.backup.com"
auth_token = "backup-token"

[test_config]
base_url = "https://api.test.com"
auth_token = "test-token"
```

**Changes:**
- ✅ Sections arranged alphabetically
- ✅ Special sections like `[global]` remain at top
- ✅ Configuration content completely preserved
- ✅ Comments and blank lines preserved (if any)

## Optimization Rules

### Sorting Rules

1. **Special Sections** (remain at top):
   - `[global]`
   - `[settings]`

2. **Regular Sections**:
   - Arranged alphabetically (A-Z)
   - Case-insensitive

### Unchanged

- ✅ Field order within sections
- ✅ Configuration values and content
- ✅ Comments (if any)
- ✅ Blank line formatting (within reason)

## Backup Mechanism

### Auto Backup

Configuration file is automatically backed up before each optimization:

```bash
Backup location: ~/.ccs_config.toml.20250110_150030.bak
          or
          ~/.ccr/platforms/<platform>/profiles.toml.20250110_150030.bak
```

### Backup File Naming

```
<original_filename>.<timestamp>.bak
```

Timestamp format: `YYYYMMDD_HHMMSS`

### Restore Backup

To restore to pre-optimization state:

```bash
# Find the most recent backup
ls -lt ~/.ccs_config.toml.*.bak | head -1

# Restore backup
cp ~/.ccs_config.toml.20250110_150030.bak ~/.ccs_config.toml

# Verify restoration
ccr list
```

## Optimization Benefits

### Improved Readability

Optimized configuration files:
- 📖 Sections arranged alphabetically, easy to find
- 🔍 Clear structure, at a glance
- 🤝 Reduced conflicts in team collaboration

### Easier Maintenance

- 📊 Fixed position when adding new configurations
- 🔄 Fewer conflicts when merging configuration files
- 📋 Easier to identify changes in file comparisons

## Working with Other Commands

### Validate After Optimization

```bash
# Optimize configuration
ccr optimize

# Validate configuration
ccr validate

# View configuration list
ccr list
```

### Export After Optimization

```bash
# Optimize configuration
ccr optimize

# Export standardized configuration
ccr export -o optimized-config.toml
```

### Optimize After Import

```bash
# Import configuration
ccr import external-config.toml --merge

# Optimize order
ccr optimize
```

## FAQ

### Q: Does optimization modify configuration content?

**A:** No, optimization only adjusts section order:
- ✅ Configuration names unchanged
- ✅ Configuration values unchanged
- ✅ Field order unchanged
- ✅ Only section arrangement order changes

### Q: Does optimization affect current configuration?

**A:** No:
- ✅ Currently active configuration unaffected
- ✅ Claude Code continues working normally
- ✅ Only rearranges configuration file order

### Q: Do I need to backup before optimization?

**A:** No manual backup needed, the optimize command backs up automatically:
- ✅ Automatically creates backup file
- ✅ Backup file includes timestamp
- ✅ Can restore anytime

### Q: Do I need to switch configurations after optimization?

**A:** No:
- ✅ Current configuration remains unchanged
- ✅ All configurations still available
- ✅ No additional action required

### Q: What are the sorting rules?

**A:** Alphabetical order (A-Z):
- Sorted by configuration name
- Case-insensitive
- Special sections (like `[global]`) remain at top

## Best Practices

### 1. Regular Optimization

```bash
# Optimize monthly or after adding multiple configurations
ccr optimize
```

### 2. Validate After Optimization

```bash
# Validate configuration integrity after optimization
ccr optimize
ccr validate
```

### 3. Review After Optimization

```bash
# View configuration list after optimization
ccr optimize
ccr list
```

### 4. Team Collaboration

```bash
# Export optimized configuration for team
ccr optimize
ccr export -o team-standard-config.toml
```

### 5. Optimize Before Merging

```bash
# Optimize after importing external configuration
ccr import external.toml --merge
ccr optimize
```

## Notes

::: tip Tip
- Optimization is **idempotent**, multiple executions won't cause errors
- Optimization doesn't modify configuration content, only adjusts order
- Optimization automatically creates backup, can restore anytime
:::

::: warning Warning
- Optimization modifies the configuration file
- Section order will change after optimization
- If using version control (Git), you'll see configuration file changes
:::

::: tip Recommendation
- Run `optimize` regularly to keep configuration file tidy
- Run `optimize` after adding multiple configurations
- Use optimized configuration for team collaboration
- Run `validate` after optimization to ensure configuration validity
:::

## Related Commands

- [validate](./validate) - Validate configuration integrity
- [list](./list) - View all configurations
- [export](./export) - Export configuration
- [import](./import) - Import configuration
- [clean](./clean) - Clean expired backups
