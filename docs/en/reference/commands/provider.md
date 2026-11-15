# provider - Provider Health Checks

`ccr provider` validates whether configured providers are reachable and whether their API keys are valid.

If you want a unified overview first, start with [`ccr doctor`](./doctor); use `provider` for targeted connectivity and credential checks once you suspect an upstream endpoint issue.

## Usage

```bash
ccr provider test <name> [--verbose]
ccr provider test --all [--verbose]
ccr provider verify <name>
```

## Subcommands

### test

- `ccr provider test <name>`: test one configuration
- `ccr provider test --all`: test all configurations
- `--verbose`: include model-level detail

Typical output fields:

- status (`Healthy` / `Degraded` / `Unhealthy` / `Unknown`)
- Base URL
- latency
- error text

### verify

```bash
ccr provider verify <name>
```

Checks whether the configured API key is valid for the chosen provider.

## Examples

```bash
ccr provider test work --verbose
ccr provider test --all
ccr provider verify work
```

## Use Cases

- validate a newly created profile
- run a batch health check across saved configurations
- diagnose model, token, or base URL issues

## Diagnostic Boundaries

- [`doctor`](./doctor): unified diagnostics entrypoint; local read-only by default, with online probing under `--online`
- [`validate`](./validate): static validation of catalogs and host settings
- [`check`](./check): local key-conflict scan only

## Notes

- This is a CLI diagnostics surface, not a standalone `provider-health` HTTP API.
- For graphical inspection, use the platform workspaces and monitoring views in `ccr ui`, which share the same backend capabilities.
