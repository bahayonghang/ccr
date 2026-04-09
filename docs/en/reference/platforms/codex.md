# Codex Platform Configuration Guide

## Overview

In CCR, Codex platform profiles are translated into Codex CLI runtime files.

- Input: `~/.ccr/platforms/codex/profiles.toml`
- Output: `~/.codex/config.toml` and `~/.codex/auth.json`

> Note: this repository also ships GitHub Copilot for VS Code workspace assets under `.github/*` plus shared `.claude/skills/`. That workspace surface is separate from the Codex CLI runtime configuration described on this page.

Switching uses two modes:

1. `official_relay` (Official mode): reset `config.toml` and `auth.json`
2. Any other type (Third-party mode): read-modify-write; only provider-related keys are updated

CCR reserves `custom` as the runtime provider namespace for third-party Codex profiles. When writing a third-party profile, the top-level `model_provider` key is always set to `"custom"`, and the real upstream identity comes from the profile metadata or `[model_providers.custom].name` instead of the root field.

## Configuration Flow

### 1. Profile -> Codex write path

After `ccr switch <profile>`, CCR writes:

- Top-level keys in `~/.codex/config.toml`: `model`, `model_provider`, `model_reasoning_effort`, `approval_policy`, `sandbox_mode`, etc.
- The active provider table is always `[model_providers.custom]`: `name`, `base_url`, `wire_api`, `requires_openai_auth`, `env_key`
- Secrets in `~/.codex/auth.json`: `OPENAI_API_KEY` or `<env_key>`

### 2. Runtime provider namespace

For every third-party profile:

1. The root `model_provider` key is always written as `"custom"`
2. The active provider config is always written to `[model_providers.custom]`
3. The real upstream provider identity still comes from profile metadata (such as `provider_id` / `provider`) or `[model_providers.custom].name`

That means `model_provider` represents CCR's reserved runtime namespace in third-party mode, not the upstream vendor name. Legacy `[model_providers.<legacy_id>]` tables may still remain in the file, but the active runtime path is `custom`.

If you have already switched the runtime provider but old history is still invisible in Codex CLI / App, use `ccr codex sync-history` to sync rollout, SQLite, and sidebar metadata.

## Key Fields (model / effort / url / key)

### model

| Field | Location | Type | Required | Description |
|------|------|------|----------|------|
| `model` | profile top-level | string | No | Primary model; written to top-level `model` in `~/.codex/config.toml` |

Example:

```toml
model = "gpt-5-codex"
```

### model_reasoning_effort

| Field | Location | Type | Required | Allowed values |
|------|------|------|----------|--------|
| `model_reasoning_effort` | platform_data (flattened profile key) | string | No | `minimal` / `low` / `medium` / `high` / `xhigh` |

Behavior:

- Strict enum validation during profile validation.
- Case-insensitive input; written in lowercase.

Example:

```toml
model_reasoning_effort = "high"
```

### base_url

| Field | Location | Type | Required | Description |
|------|------|------|----------|------|
| `base_url` | profile top-level | string | Required in third-party mode | Provider endpoint; must start with `http://` or `https://` |

Example:

```toml
base_url = "https://api.example.com/v1"
```

### key (auth_token / env_key / OPENAI_API_KEY)

| Field | Location | Type | Required | Description |
|------|------|------|----------|------|
| `auth_token` | profile top-level | string | Depends on auth intent | Source token used when writing `auth.json` |
| `env_key` | platform_data | string | Required for provider-key mode | Key name written into `auth.json` |
| `OPENAI_API_KEY` | `~/.codex/auth.json` | string | Written in OpenAI API-key flow | OpenAI key entry |

Auth intent rules:

- If `requires_openai_auth = true`: OpenAI-auth semantics; `env_key` is ignored.
- If `requires_openai_auth = false` and `env_key` is set: `auth_token` is required and written to `auth.json[env_key]`.
- If `requires_openai_auth` is omitted: inferred from `env_key` presence.

## Other Common Fields

| Field | Location | Description |
|------|------|------|
| `wire_api` | platform_data | `responses` or `chat`; default is `responses` |
| `provider_type` | profile top-level | `official_relay` => official mode; otherwise third-party mode |
| `approval_policy` | platform_data | passed through to top-level `config.toml` |
| `sandbox_mode` | platform_data | passed through to top-level `config.toml` |
| `network_access` | platform_data | passed through to top-level `config.toml` |
| `disable_response_storage` | platform_data | passed through to top-level `config.toml` (bool) |
| `provider_model` | platform_data | optional; writes `[model_providers.custom].model` |

## Recommended Profiles

### Official mode (reset to Codex defaults)

```toml
[official]
description = "Codex official mode"
provider = "openai"
provider_type = "official_relay"
```

### Third-party mode (provider env key)

```toml
[duckcoding]
description = "DuckCoding OpenAI compatible"
base_url = "https://jp.duckcoding.com/v1"
auth_token = "sk-..."
model = "gpt-5-codex"
provider = "duckcoding"
provider_type = "third_party_model"
wire_api = "responses"
env_key = "DUCKCODING_API_KEY"
requires_openai_auth = false
model_reasoning_effort = "high"
approval_policy = "on-request"
sandbox_mode = "workspace-write"
network_access = "enabled"
disable_response_storage = true
```

### Third-party mode (OpenAI-auth semantics)

```toml
[openai-proxy]
description = "OpenAI auth via proxy"
base_url = "https://proxy.example.com/v1"
model = "gpt-5-codex"
provider = "proxy"
provider_type = "third_party_model"
wire_api = "responses"
requires_openai_auth = true
model_reasoning_effort = "medium"
```

## Generated Output Example

For the `duckcoding` profile, expected `~/.codex/config.toml` looks like this (note that the runtime namespace is fixed to `custom`):

```toml
model_provider = "custom"
model = "gpt-5-codex"
model_reasoning_effort = "high"
approval_policy = "on-request"
sandbox_mode = "workspace-write"
network_access = "enabled"
disable_response_storage = true

[model_providers.custom]
name = "DuckCoding OpenAI compatible"
base_url = "https://jp.duckcoding.com/v1"
wire_api = "responses"
requires_openai_auth = false
env_key = "DUCKCODING_API_KEY"
```

Expected `~/.codex/auth.json`:

```json
{
  "DUCKCODING_API_KEY": "sk-..."
}
```

## Validation and Troubleshooting

### Common validation failures

1. Invalid `wire_api`
- Only `responses` / `chat` are allowed.

2. Invalid `model_reasoning_effort`
- Only `minimal/low/medium/high/xhigh` are allowed.

3. Missing `base_url` in third-party profile
- URL must start with `http://` or `https://`.

4. Missing `auth_token` in provider-key mode
- When `env_key` is active, `auth_token` is required.

### Useful commands

```bash
ccr platform switch codex
ccr list
ccr validate
ccr switch <profile>
```

## Security Notes

1. Never commit real tokens to Git.
2. Use `--no-secrets` when exporting shareable configs.
3. Protect local permissions:

```bash
chmod 600 ~/.ccr/platforms/codex/profiles.toml
chmod 600 ~/.codex/auth.json
```

## See Also

- [GitHub Copilot Workspace Support](/en/guide/github-copilot-workspace)
- [Platform Overview](./index)
- [Platform Migration](./migration)
- [Examples](../../examples/)
