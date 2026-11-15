# check - Conflict Detection

Detect potential environment variable and key conflicts across platform configurations so they do not silently overwrite each other.

For a unified health summary across your entire setup, run [`ccr doctor`](./doctor) first; `check` focuses specifically on local key-collision scanning.

**Version**: v3.6.0+

## Subcommands

### conflicts

Scan platform configurations and report potential key collisions.

```bash
ccr check conflicts
```

Sample output:

```text
⚠️ Conflicts detected:
- CLAUDE_API_KEY appears in claude / codex
```

> Run after multi-platform setup or migration to ensure distinct key naming per platform.

## Diagnostic Boundaries

- [`doctor`](./doctor): unified diagnostics entrypoint that includes conflict scanning in its global summary
- [`validate`](./validate): static validation of catalogs and host settings
- [`provider`](./provider): reachability-oriented provider health checks
