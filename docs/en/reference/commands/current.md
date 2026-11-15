# `current` - Runtime Overview

`ccr current` displays Claude Runtime and Codex Runtime status side by side rather than assuming a single global "current platform". It also provides two visible top-level aliases: **`status`** (recommended shorthand) and **`show`**.

## Usage and Visible Aliases

```bash
ccr current [--verbose | --json]
ccr status [--verbose | --json]
ccr show [--verbose | --json]
```

`--verbose` and `--json` are mutually exclusive; `ccr status` and `ccr show` accept the exact same flags and produce identical output as `ccr current`. To inspect a single platform directly (including Grok), run `ccr claude profile current [--json]`, `ccr codex profile current [--json]`, or `ccr grok profile current [--json]`.

## Output Model

Default terminal output includes:
- Claude Runtime status card
- Codex Runtime status card
- Active profile, provider, authentication source, and health summary per platform

`--verbose` additionally shows:
- Registry target metadata
- Platform catalog and host runtime file paths
- Full non-secret fields of the active profile
- Environment variable and settings override diagnostics

`--json` returns:
- `schema_version`
- `generated_at`
- `claude`
- `codex`

> The top-level payload no longer includes the legacy `current_platform` field.

## Common Scenarios

```bash
ccr status
ccr current --verbose
ccr status --json
```

- Confirm whether Claude and Codex are running under official login or a CCR profile
- Verify resolved runtime paths and auth sources after switching profiles
- Feed a unified structured runtime snapshot into desktop tooling, VS Code extensions, or scripts

## Related Pages

- [`claude`](./claude) / [`codex`](./codex) / [`grok`](./grok)
- [`platform`](./platform)
- [`validate`](./validate)
- [Migration Guide](/en/reference/migration)
