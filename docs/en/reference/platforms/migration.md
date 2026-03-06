# Platform Migration Guide

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
- [Configuration Guide](/en/guide/configuration)
- [Migration Guide](/en/reference/migration)

---

Continue with [Platform Support](/en/reference/platforms/) for the current support matrix.
