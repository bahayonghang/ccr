# doctor - Unified Diagnostics

`ccr doctor` provides a unified, read-only diagnostics entrypoint across CCR's local environment, platform registry, active profiles, host runtime authentication, and optional online provider probing.

## Usage

```bash
ccr doctor
ccr doctor --json
ccr doctor --verbose
ccr doctor --online
ccr doctor --all-platforms
ccr doctor --platform codex
```

## Default Behavior

- Runs **local-first, read-only, side-effect-free** checks by default.
- Default scope: **global CCR status + configured Claude/Codex/Grok platform checks**.
- Never auto-repairs, initializes, or mutates configuration files or log directories.
- Performs online provider probing only when `--online` is passed.

## Options

- `--json`: emit a stable JSON report suitable for scripts and CI
- `--verbose`: display paths, details, and remediation recommendations
- `--online`: probe the active `current_profile` provider endpoint for each inspected target
- `--all-platforms`: inspect all configured platforms
- `--platform <name>`: inspect only the named platform

> `--platform` and `--all-platforms` are mutually exclusive. Platform values derive from the shared `Platform` enum, including `grok`.

Claude, Codex, and Grok support `auth` and `profile` commands. Gemini and Droid retain read-only compatibility checks labeled `legacy_adapter` with no writer calls or CLI `auth`/`profile` commands. Qwen reports unimplemented. Grok checks cover the `profiles.db` catalog, `~/.grok/config.toml`, and managed route consistency; official session validity remains owned by Grok itself.

## Inspection Scope

### Global checks

- CCR root and registry path readability
- Platform capability classification (first-class, `legacy_adapter`, or unimplemented)
- Configured platform registry parsing
- Local conflict scan (reusing the same source of truth as `ccr check conflicts`)

### Deep platform checks

- `profiles.db` readability
- `current_profile` resolution against saved profiles
- Local field validation on the active profile
- Host settings/config path resolution, readability, and schema validation
- Runtime authentication health

### Online checks (`--online`)

When `--online` is enabled:

- Probes only the **current profile** for each inspected platform target
- Does not scan inactive profiles
- Marks non-probable profiles as skipped

## Output Format

Terminal results use status prefixes:

- `✓`: check passed
- `! 警告:`: check raised a warning
- `× 错误:`: check failed
- `- 跳过:`: check skipped

Redirected output and `TERM=dumb` use `成功:`, `警告:`, `错误:`, and `跳过:`. `NO_COLOR` disables ANSI styling while retaining symbols in a normal terminal. Results, paths, and recommendations stay on `stdout`; `--json` preserves its structured schema and status values.

Each run concludes with:

```text
Results: X passed, Y warnings, Z failed, K skipped
```

## Exit Codes

- Returns `1` when one or more checks fail (`failed`)
- Returns `0` when checks pass or raise warnings only (embedded services return typed reports without terminating the host process)

## JSON Output

Under `--json`, the top-level object contains:

- `scope`
- `online`
- `summary`
- `checks`

Each `check` entry includes `id`, `status`, and `summary`, plus optional `path`, `detail`, and `recommendation` fields.

## Relationship to Other Diagnostic Commands

- `ccr doctor`: unified overview entrypoint — run this first
- [`ccr validate`](./validate): static validation of catalogs and host settings
- [`ccr provider`](./provider): targeted provider reachability and API key checks
- [`ccr check conflicts`](./check): focused local key-conflict scan

## Examples

```bash
# Default: global + configured Claude/Codex/Grok read-only checks
ccr doctor

# Full local check on Codex with verbose details
ccr doctor --platform codex --verbose

# Inspect all configured platforms with online probing
ccr doctor --all-platforms --online

# Consume structured JSON in CI
ccr doctor --json
```
