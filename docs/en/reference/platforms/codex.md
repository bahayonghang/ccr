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
- `ccr codex profile off`: leave profile mode, remove the CCR profile route and runtime `auth.json`, and prepare a clean official runtime for `codex login`

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
Switching to another profile replaces these fields. Running `ccr codex profile off` removes the
root `model_provider`, the CCR-managed `model_providers.custom` entry, other profile root fields,
and runtime `auth.json`, while preserving `model_reasoning_effort` verbatim. When there is no
profile pointer, legacy entry snapshot, or third-party runtime, the command leaves the existing
official `auth.json` unchanged.

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

`ccr codex fix` cleans up stale app-server processes and compares the registry pointer, `profiles.toml`, `config.toml`, `auth.json`, and the current process environment at invocation time. It reports `process_state`, `runtime_consistency`, and `provider_auth_validity` separately. The default path does not run upstream `codex doctor`.

Process discovery explicitly loads command lines and owners and only handles Codex `app-server`
processes owned by the current user. Cleanup identifies processes by `PID + start_time`. After TERM,
it rediscovers matching targets every 300 ms and ends the grace loop as soon as the target set is
empty. Identities that appear only in the settle snapshot go into `respawned` and do not receive a
deadline KILL. Owner and argv are revalidated before every signal. Output contains redacted
summaries only. If a safe snapshot cannot be established, CCR reports
`process_state = unavailable` instead of treating the unknown state as `clean`.

The bare command is a local diagnosis only. It does not rewrite runtime files and does not run
upstream doctor. To replay the saved profile through the existing atomic apply path, opt in
explicitly:

```bash
ccr codex fix --repair-runtime
ccr codex fix --dry-run --repair-runtime
ccr codex fix --doctor
```

`--repair-runtime` does not change or rotate the saved secret. Combined with `--dry-run`, it neither terminates processes nor writes `config.toml` or `auth.json`. `--repair-runtime` does not imply `--doctor`.

Process cleanup, runtime inspection/repair, and doctor are independent stages. Doctor runs only with
`--doctor`. When the runtime stage is unavailable, CCR exits with code `1`; an app-server that
remains or unavailable process discovery takes precedence with exit code `2`. Exit code `127` is
used only when `--doctor` is passed and `codex` is missing from `PATH`.

CCR's reconciliation adds no third-party credential probe. Run `ccr codex fix --doctor` when you need upstream health checks; those checks depend on the installed Codex version. Even when `runtime_consistency = match`, `provider_auth_validity` remains `not_checked`. If the provider still returns `INVALID_API_KEY`, verify or update the key saved in that profile instead of repeatedly cleaning app-server processes.

## History sync note

`ccr codex sync-history ...` still repairs history visibility after provider-namespace changes. When moving between official and third-party profiles, prefer:

```bash
ccr codex sync-history --bridge official-custom --dry-run
ccr codex sync-history --bridge official-custom --all-history
```

Bridge mode repairs list visibility only. If a history contains `encrypted_content`, CCR warns that it cannot re-encrypt it, so continue/compact may still be constrained by the original account/provider encryption boundary.
