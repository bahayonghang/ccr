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
- defaults to configured Claude/Codex/Grok runtime targets
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

## Output format

Terminal results use `✓` for passed checks, `! 警告:` for warnings, `× 错误:` for failed checks, and `- 跳过:` for skipped checks. Redirected output and `TERM=dumb` use `成功:`, `警告:`, `错误:`, and `跳过:`. `NO_COLOR` disables styling and retains the symbols in a regular terminal.

Results, paths, and recommendations remain on stdout. `--json` retains the existing structure and status values.

## Capability and Exit Contract

Platform choices come from the shared `Platform` enum and include `grok`. Claude, Codex, and Grok support auth/profile commands. Gemini and Droid retain read-only compatibility checks with a `legacy_adapter` label. Qwen remains unimplemented. No additional legacy writer is called.

Grok checks cover config TOML and managed route consistency. Grok owns official session authentication; offline checks do not prove a valid login.

Failed checks return `1`. Warnings alone return `0`. The binary chooses process status from the typed report. Embedded services never terminate the host process. Local diagnostics do not create log directories or mutate configuration files.
