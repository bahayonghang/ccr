# doctor - Unified Diagnostics

`ccr doctor` is the high-level diagnostics entrypoint for CCR runtime state.

## Usage

```bash
ccr doctor
ccr doctor --json
ccr doctor --verbose
ccr doctor --online
ccr doctor --all-platforms
ccr doctor --platform codex
```

## Current behavior

- defaults to local-first, read-only checks
- defaults to configured Claude/Codex runtime targets rather than a legacy global current platform
- runs provider online probing only with `--online`

## What it checks

- CCR root and registry readability
- configured platform targets
- current-profile resolution for each inspected runtime target
- platform settings/config readability and validation
- runtime auth health

## Related docs

- [validate](./validate)
- [current](./current)
- [Migration Guide](/en/reference/migration)
