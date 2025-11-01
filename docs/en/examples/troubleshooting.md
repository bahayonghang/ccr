# Troubleshooting Guide

> **Note:** This documentation is being translated. Please refer to the [Chinese version](../examples/troubleshooting) for complete information.

## Common Issues

### Config File Not Found

```bash
ccr init
```

### Lock Timeout

```bash
# Check for stale locks
ls -la ~/.claude/.locks/

# Clean if no CCR process running
rm -rf ~/.claude/.locks/*
```

### Permission Denied

```bash
chmod 600 ~/.claude/settings.json
chmod 644 ~/.ccs_config.toml
```

## See Also

- [Examples Overview](./index)
- [Quick Start](../quick-start)
- [Configuration Guide](../configuration)

---

**Translation in progress.** See [Chinese version](../examples/troubleshooting) for complete troubleshooting guide.
