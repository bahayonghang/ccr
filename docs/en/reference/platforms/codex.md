# Codex Platform Configuration Guide

Codex now uses a split model: one surface for official-auth accounts and another for runtime/profile routing.

## Main current path

```bash
ccr codex auth current
ccr codex auth list
ccr codex profile list
ccr codex profile switch <name>
ccr codex profile current
ccr codex profile off
ccr codex fix
```

## `auth` vs `profile`

- `ccr codex auth ...`: save / switch / import / export official-auth accounts
- `ccr codex profile ...`: write a CCR profile into `~/.codex/config.toml` and `~/.codex/auth.json`
- `ccr codex profile off`: leave profile mode and restore the official-auth runtime

## Key paths

- Runtime config: `~/.codex/config.toml`
- Runtime auth: `~/.codex/auth.json`
- Profiles: `~/.ccr/platforms/codex/profiles.toml`
- Registry pointer: `[codex].current_profile` in `~/.ccr/config.toml`

## DeepSeek Responses API

DeepSeek integration requires **Codex >= 0.144.0**. The currently supported model is
`deepseek-v4-flash`. CCR does not download or overwrite the `~/.codex/models.json` catalog;
generate it first with the
[official DeepSeek Codex guide](https://api-docs.deepseek.com/quick_start/agent_integrations/codex/),
the official [shell setup script](https://cdn.deepseek.com/api-docs/codex-deepseek-setup.sh),
or the official [PowerShell setup script](https://cdn.deepseek.com/api-docs/codex-deepseek-setup-en.ps1).

In the ccr-ui Codex Profiles page, select the DeepSeek provider template, then:

1. Select `Provider Bearer Token` and enter the DeepSeek API key.
2. Keep `deepseek-v4-flash` as the model and set the model catalog to `~/.codex/models.json`.
3. Select `high` reasoning effort, save, and apply the profile.

The template fills non-secret provider, endpoint, and model fields only. It never stores or
overwrites the API key. The stored profile contains the following non-secret fields; CCR keeps
the bearer in its runtime secret store, so do not add it manually to `profiles.toml`:

```toml
[deepseek]
description = "DeepSeek"
base_url = "https://api.deepseek.com/"
model = "deepseek-v4-flash"
provider = "deepseek"
provider_type = "third_party_model"
wire_api = "responses"
auth_mode = "provider_bearer_token"
model_catalog_json = "~/.codex/models.json"
model_reasoning_effort = "high"
enabled = true
```

When applied, CCR keeps the provider id fixed at `[model_providers.custom]` and derives
`preferred_auth_method = "apikey"` plus `forced_login_method = "api"`. See the resulting runtime
shape in [`codex-cli-config.toml`](https://raw.githubusercontent.com/bahayonghang/ccr/main/docs/examples/codex-cli-config.toml).
Switching to another profile or running `ccr codex profile off` removes these root fields and
`experimental_bearer_token`.

::: warning Credential and sync boundary
`~/.codex/config.toml` and CCR-created `~/.codex/backups/config.*.bak` files contain the bearer in
plaintext. Do not commit, share, or attach them as ordinary diagnostic files. `config.toml` is also
the sensitive `codex-config` sync asset: WebDAV sync includes the bearer inside the encrypted v2
envelope and requires an independent operation passphrase; the bearer is not omitted from the
synced content.
:::

## Runtime diagnosis and repair

Switch to the profile you intend to diagnose before running `fix`:

```bash
ccr codex profile switch future
ccr codex fix
```

`ccr codex fix` cleans up stale app-server processes and compares the registry pointer, `profiles.toml`, `config.toml`, `auth.json`, and the current process environment at invocation time. It reports `process_state`, `runtime_consistency`, and `provider_auth_validity` separately.

Process discovery explicitly loads command lines and owners and only handles Codex `app-server`
processes owned by the current user. Cleanup identifies processes by `PID + start_time`, discovers
replacement PIDs throughout the TERM grace window, and revalidates owner and argv before every
signal. Output contains redacted summaries only. If a safe snapshot cannot be established, CCR
reports `process_state = unavailable` instead of treating the unknown state as `clean`.

The bare command is diagnostic only. To replay the saved profile through the existing atomic apply path, opt in explicitly:

```bash
ccr codex fix --repair-runtime
ccr codex fix --dry-run --repair-runtime
```

`--repair-runtime` does not change or rotate the saved secret. Combined with `--dry-run`, it neither terminates processes nor writes `config.toml` or `auth.json`.

Process cleanup, runtime inspection/repair, and doctor are independent stages. When the runtime
stage is unavailable, CCR still runs doctor when possible and exits with code `1`; an app-server
that remains or unavailable process discovery takes precedence with exit code `2`.

CCR's reconciliation adds no third-party credential probe. The command still runs upstream `codex doctor`, whose checks depend on the installed Codex version. Even when `runtime_consistency = match`, `provider_auth_validity` remains `not_checked`. If the provider still returns `INVALID_API_KEY`, verify or update the key saved in that profile instead of repeatedly cleaning app-server processes.

## History sync note

`ccr codex sync-history ...` still repairs history visibility after provider-namespace changes. When moving between official and third-party profiles, prefer:

```bash
ccr codex sync-history --bridge official-custom --dry-run
ccr codex sync-history --bridge official-custom --all-history
```

Bridge mode repairs list visibility only. If a history contains `encrypted_content`, CCR warns that it cannot re-encrypt it, so continue/compact may still be constrained by the original account/provider encryption boundary.
