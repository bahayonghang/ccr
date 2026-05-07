# current - Runtime Overview

`ccr current` now shows Claude Runtime and Codex Runtime side by side instead of a single “current platform”.

## Usage

```bash
ccr current
ccr current --verbose
ccr current --json
```

## Output model

Default output includes:

- a Claude Runtime status card
- a Codex Runtime status card
- current profile / provider / auth / health summary for each platform

`--verbose` additionally shows:

- registry target details
- platform paths
- current profile details
- environment/settings diagnostics

`--json` returns:

- `schema_version`
- `generated_at`
- `claude`
- `codex`

> The top level no longer includes `current_platform`.
