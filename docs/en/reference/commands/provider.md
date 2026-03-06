# provider - Provider Health Checks

`ccr provider` validates whether configured providers are reachable and whether their API keys are valid.

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
- `--verbose`: include more model-level detail

Typical output fields:

- status (Healthy / Degraded / Unhealthy / Unknown)
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

- validate a freshly created profile
- run a batch health check over all saved configurations
- diagnose model, token, or base URL issues

## Notes

- This is a CLI diagnostics surface, not a standalone `provider-health` HTTP API.
- If you want a browser-oriented summary, use the Provider Health page in `ccr-ui`; it is backed by the same underlying capability set.
