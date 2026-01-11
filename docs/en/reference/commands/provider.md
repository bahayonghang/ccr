# provider

Provider health check commands for testing API endpoint connectivity and validating API Keys.

## Overview

The `ccr provider` command group provides health check functionality for configured Providers:

- **Connectivity Test**: Check if API endpoint is reachable
- **API Key Validation**: Verify if the key is valid
- **Latency Measurement**: Measure API response time
- **Model Availability**: Check if configured model is available

## Subcommands

### test

Test Provider connectivity.

```bash
ccr provider test <NAME|--all> [OPTIONS]
```

**Arguments:**

| Argument | Description |
|----------|-------------|
| `<NAME>` | Name of configuration to test |

**Options:**

| Option | Description |
|--------|-------------|
| `-a, --all` | Test all configurations |
| `-v, --verbose` | Show detailed information |

**Output Information:**

- Provider name
- Base URL
- Health status (Healthy/Degraded/Unhealthy/Unknown)
- Latency (milliseconds)
- Error message (if any)

**Examples:**

```bash
# Test single configuration
ccr provider test my-provider

# Test all configurations
ccr provider test --all

# Show detailed information
ccr provider test my-provider --verbose
```

### verify

Verify API Key validity.

```bash
ccr provider verify <NAME>
```

Verifies API Key validity by calling the `/v1/models` endpoint and lists available models.

**Output:**

- Key validity status
- Available models list
- Whether configured model is in the list

**Examples:**

```bash
# Verify API Key
ccr provider verify my-provider
```

## Health Status Description

| Status | Description |
|--------|-------------|
| `Healthy` | Endpoint reachable, API Key valid |
| `Degraded` | Endpoint reachable but slow response or limited functionality |
| `Unhealthy` | Endpoint unreachable or API Key invalid |
| `Unknown` | Unable to determine status |

## Test Flow

1. **Connectivity Test**: Request `{base_url}/v1/models` endpoint
2. **Authentication Test**: Check HTTP status code (401/403 = authentication failed)
3. **Latency Test**: Record request round-trip time
4. **Model Validation**: Confirm configured model is in returned list

## Use Cases

### Diagnose Connection Issues

```bash
# Test specific configuration
ccr provider test my-provider --verbose

# View detailed error information
```

### Batch Check All Configurations

```bash
# Test all configuration status at once
ccr provider test --all
```

### Verify New Configuration

```bash
# Verify after adding new configuration
ccr add
ccr provider verify new-config
```

### CI/CD Integration

```bash
# Check Provider status in scripts
if ccr provider test my-provider; then
    echo "Provider is healthy"
else
    echo "Provider check failed"
    exit 1
fi
```

## Web API

Provider health check is also accessible via Web API:

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/provider-health/test` | POST | Test single Provider |
| `/api/provider-health/test-all` | GET | Test all Providers |

## Version Information

- **Introduced in**: v3.12.0
- **Feature Dependency**: Requires `web` feature (network requests)
