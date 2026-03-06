# Troubleshooting Guide

> This page lists a few quick recovery paths. For the main onboarding flow, start with the English quick start guide.

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
- [Quick Start](/en/guide/quick-start)
- [Configuration Guide](/en/guide/configuration)

---

For platform-specific details, continue with [Platform Support](/en/reference/platforms/).
