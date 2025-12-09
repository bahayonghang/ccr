# check - Conflict Detection

Detect potential key conflicts across platform configurations to avoid overwriting each other.

**Version**: v3.6.0+

## Subcommands

### conflicts

Scan platform configs and report suspicious key collisions.

```bash
ccr check conflicts
```

Sample output:

```
⚠️ Conflicts detected:
- CLAUDE_API_KEY appears in claude / codex
```

> Run after platform setup or migration to ensure distinct key naming per platform.
