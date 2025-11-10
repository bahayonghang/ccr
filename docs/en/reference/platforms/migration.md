# Platform Migration Guide

> **Note:** This documentation is being translated. Please refer to the [Chinese version](../platforms/migration) for complete information.

## Overview

Guide for migrating configurations between platforms and modes.

## Migration Types

### Legacy to Unified Mode

```bash
ccr migrate --check
ccr migrate
```

### Between Platforms

```bash
# Export from Claude
ccr platform switch claude
ccr export -o claude-profiles.toml

# Import to Codex
ccr platform switch codex
ccr import claude-profiles.toml --merge
```

## See Also

- [Platform Overview](./index)
- [Configuration Guide](../configuration)
- [Migration Guide](../migration)

---

**Translation in progress.** See [Chinese version](../platforms/migration) for details.
