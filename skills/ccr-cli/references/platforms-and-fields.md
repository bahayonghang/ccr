# Platforms, Auth Modes, Writable Fields & Secret Storage

## Table of Contents

- [1. Canonical Storage & Runtime Paths](#1-canonical-storage--runtime-paths)
- [2. Create vs Edit Flag Surfaces & Common Editable Fields](#2-create-vs-edit-flag-surfaces--common-editable-fields)
- [3. Claude Platform Contract](#3-claude-platform-contract)
- [4. Codex Platform Contract & Sidecar Secrets](#4-codex-platform-contract--sidecar-secrets)
- [5. Grok Platform Contract & Credential Exclusivity](#5-grok-platform-contract--credential-exclusivity)
- [6. Environment Variable Name vs Secret Value Boundary](#6-environment-variable-name-vs-secret-value-boundary)

---

## 1. Canonical Storage & Runtime Paths

Always inspect state through `ccr <platform> profile list --json` and `ccr <platform> profile current --json`. Never read `~/.ccr/platforms/*/profiles.toml`, `profiles.db`, `*.bak` backup files, `~/.ccr/platforms/codex/profile_secrets.json`, or `~/.codex/auth.json` directly via file-reading tools or shell commands.

| Platform | Catalog Storage (CCR Managed, Never Read Directly) | Secret Sidecar / Entry State | Host CLI Live Runtime Target |
| --- | --- | --- | --- |
| `claude` | `~/.ccr/platforms/claude/profiles.toml` + `profiles.db` | Inline in catalog (`0600` backups) | `~/.claude/settings.json` |
| `codex` | `~/.ccr/platforms/codex/profiles.toml` + `profiles.db` | `~/.ccr/platforms/codex/profile_secrets.json` (`0600`) | `~/.codex/config.toml` + `~/.codex/auth.json` |
| `grok` | `~/.ccr/platforms/grok/profiles.toml` + `profiles.db` | `~/.ccr/platforms/grok/profile_entry_config_state.json` | `$GROK_HOME/config.toml` (defaults to `~/.grok/config.toml`) |

Run `ccr <claude|codex|grok> profile <action> --help` before composing unfamiliar flags; `--help` is the authoritative flag reference for the installed `ccr` binary.

---

## 2. Create vs Edit Flag Surfaces & Common Editable Fields

`profile create` and `profile edit` expose distinct CLI flags (`profile_args.rs` / `grok.rs`):

- **`profile create <NAME>` surface**:
  - Dedicated flags on all 3 platforms: `--description <TEXT>`, `--base-url <URL>`, `--model <MODEL>`, `--provider <NAME>`, `--provider-type <official_relay|third_party_model>`, `--account <LABEL>`, repeatable `--tag <TAG>`, `--disabled`, `--expected-revision <REV>`, `--json`, plus repeatable `--set FIELD=VALUE` for non-secret fields permitted by `create_field_allowed`.
  - Additional dedicated flags on `claude` and `codex` `create` only: `--small-fast-model <MODEL>` and `--auth-mode <MODE>`.
  - `profile create` does **not** accept `--clear` (clearing fields only applies to `profile edit`).
- **`profile edit <NAME>` surface**:
  - Dedicated flags on all 3 platforms: `--rename <NEW_NAME>`, `--description <TEXT>`, `--model <MODEL>`, `--base-url <URL>`, `--provider <NAME>`, repeatable `--tag <TAG>`, `--enable` / `--disable`, `--auth-token-stdin` / `--clear-auth-token`, `--expected-revision <REV>`, `--no-apply`, `--json`, repeatable `--set FIELD=VALUE`, and repeatable `--clear FIELD`.
  - There is **no** `--clear-tags` flag and **no** `--small-fast-model`, `--provider-type`, or `--account` dedicated flag on `profile edit` — use `--clear tags`, `--set small_fast_model=<MODEL>` / `--clear small_fast_model`, `--set provider_type=<official_relay|third_party_model>` / `--clear provider_type`, and `--set account=<LABEL>` / `--clear account`.
  - `enabled` is controlled via `--disabled` on `create` and `--enable` / `--disable` on `edit` (`--clear enabled` is rejected because `enabled` is non-nullable).

---

## 3. Claude Platform Contract

### Auth Modes (`claude`)

- `api_key`: Third-party relay or Anthropic API key mode; requires `base_url` and credential input (via `--auth-token-stdin`) to be runnable.
- `subscription`: Official Claude subscription pass-through (no inline token required; if a non-empty inline token is supplied while `auth_mode` is `subscription`, CCR normalizes `auth_mode` to `api_key`).

### Platform-Specific Writable Fields (`claude`)

- `auth_mode`: `subscription` | `api_key` (`--auth-mode` on `create`, or `--set auth_mode=...` / `--clear auth_mode` on `edit`)
- `small_fast_model`: `--small-fast-model <MODEL>` on `create`; `--set small_fast_model=<MODEL>` or `--clear small_fast_model` on `edit`
- Additional `--set <field>=<value>` / `--clear <field>` fields: `default_opus_model`, `default_sonnet_model`, `default_haiku_model`, `default_fable_model`, `subagent_model`, `effort_level` (`low` | `medium` | `high` | `xhigh` | `max`), `api_timeout_ms` (non-negative integer ms), `claude_code_auto_compact_window` (non-negative integer token window)

### Secret & Suppression Notes (`claude`)

- Feed credentials only via non-interactive stdin pipe on both `create` and `edit`:
  - Create: `printf '%s' "$MY_CLAUDE_KEY" | ccr claude profile create <name> ... --auth-token-stdin --json`
  - Edit: `printf '%s' "$MY_CLAUDE_KEY" | ccr claude profile edit <name> --auth-token-stdin --json`
- Remove stored inline credentials with `ccr claude profile edit <name> --clear-auth-token --json`.
- When turning off CCR routing via `ccr claude profile off --json`, inspect `remaining_suppressors` in the JSON response: external shell environment variables (such as `ANTHROPIC_API_KEY` or `ANTHROPIC_AUTH_TOKEN` set in shell profiles) or unmanaged settings outside CCR's control are reported there rather than silently deleted.

---

## 4. Codex Platform Contract & Sidecar Secrets

### Auth Modes (`codex`)

- `openai_chatgpt`: Official ChatGPT OAuth session managed outside profile inline secrets (`ccr codex auth` owns OAuth account snapshots; `ccr-cli` only switches/inspects compatible profiles).
- `openai_api_key`: Direct OpenAI API key mode; secret stored in sidecar `profile_secrets.json`.
- `provider_env_key`: Custom provider referencing an environment variable name via `env_key` (`requires_openai_auth` typically `false`); optional managed secret value stored in sidecar `profile_secrets.json`.
- `provider_bearer_token`: Custom provider bearer token mode; secret stored in sidecar `profile_secrets.json`.
- `no_auth`: Unauthenticated local/proxy endpoint requiring no secret.

### Sidecar Secret Architecture (`profile_secrets.json`)

- For `openai_api_key`, `provider_env_key`, and `provider_bearer_token`, CCR stores the secret payload inside `~/.ccr/platforms/codex/profile_secrets.json` (`0600` permissions) rather than writing plaintext tokens into `profiles.toml`.
- CCR overlays sidecar secrets in memory during `profile switch` / `enable` and `profile export` (when not redacted), and updates/deletes sidecar entries atomically during `profile create`, `edit`, `delete`, and `import`.
- Agents must **never** read `profile_secrets.json`, `profiles.toml`, `profiles.db`, `*.bak`, or `~/.codex/auth.json`.

### Platform-Specific Writable Fields (`codex`)

- `auth_mode` (`--auth-mode` or `--set auth_mode=...` on `create` only): `openai_chatgpt` | `openai_api_key` | `provider_env_key` | `provider_bearer_token` | `no_auth` (`codex profile edit` does not accept `auth_mode` in `--set`; recreate or import to change `auth_mode`).
- `small_fast_model`: `--small-fast-model <MODEL>` on `create`; `--set small_fast_model=<MODEL>` or `--clear small_fast_model` on `edit`.
- `model_reasoning_effort` (`--reasoning-effort <EFFORT>` on `edit`, or `--set model_reasoning_effort=...` / `--clear model_reasoning_effort`): `minimal` | `low` | `medium` | `high` | `xhigh`.
- `wire_api` (`--set wire_api=responses` / `--clear wire_api`): Only `responses` is accepted.
- `env_key` (`--set env_key=<ENV_NAME>` / `--clear env_key`): Single environment variable **name** string (for example `OPENROUTER_API_KEY`); commas and array syntax are rejected.
- `requires_openai_auth` (`--set requires_openai_auth=true|false` / `--clear requires_openai_auth`): `true` | `false` (`1` | `0` accepted).

---

## 5. Grok Platform Contract & Credential Exclusivity

### Derived Auth Modes & Mutual Exclusivity (`grok`)

Grok derives `auth_mode` from three mutually exclusive credential representations:

1. `env_key` (**Preferred zero-secret-storage mode**): References an environment variable name via `--env-key <ENV_NAME>`. Setting `--env-key` clears any stored inline key or session token.
2. `inline_api_key`: Stores an inline key fed via `--api-key-stdin` (visible alias `--auth-token-stdin` on `create`; both `--api-key-stdin` and `--auth-token-stdin` exist on `edit`). Setting an inline key clears `env_key` and session token.
3. `session`: Session token representation managed outside standard inline API key creation (`grok auth` interactive flows are out of scope for `ccr-cli`).

### Platform-Specific Flags & Fields (`grok`)

- `--env-key <ENV_NAME>`: Environment variable name holding the xAI / compatible API key (on `create`, always use `--env-key <ENV_NAME>` because `--set env_key=...` is rejected on `grok profile create`; on `edit`, prefer `--env-key <ENV_NAME>` or `--clear env_key`).
- `--api-key-stdin` (or `--auth-token-stdin`): Read API key from non-interactive stdin pipe (`printf '%s' "$MY_XAI_KEY" | ccr grok profile create <name> ... --api-key-stdin --json`); `--clear-auth-token` clears stored credentials on `edit`.
- `--api-backend <chat_completions|responses|messages>` (or `--set api_backend=...` / `--clear api_backend`)
- `--context-window <POSITIVE_INTEGER>` (or `--set context_window=...` / `--clear context_window`)
- `--supports-backend-search [true|false]` (or `--set supports_backend_search=...` / `--clear supports_backend_search`)
- `--reasoning-effort <none|minimal|low|medium|high|xhigh|max>` (or `--set reasoning_effort=...` / `--clear reasoning_effort`; exact allowed values: `none`, `minimal`, `low`, `medium`, `high`, `xhigh`, `max`)

### Entry State Protection (`grok`)

Grok tracks native `$GROK_HOME/config.toml` entry snapshots in `profile_entry_config_state.json` (`Inactive`, `Active`, `Drifted`, `UnsafeMissingEntryState`). If a profile enters `UnsafeMissingEntryState`, even `grok profile delete --force` is blocked so CCR will not corrupt native Grok settings; run `ccr validate --json`, `ccr doctor --json --platform grok`, and `ccr grok profile off --json` (with operator approval) before deleting.

---

## 6. Environment Variable Name vs Secret Value Boundary

Keep these two mechanisms strictly separate when constructing commands:

| Concept | Safe to Appear in CLI Flags? | Canonical Flag / Pattern | Stored on Disk in CCR Catalog? |
| --- | --- | --- | --- |
| **Environment Variable Name** (e.g. `OPENROUTER_API_KEY`, `XAI_API_KEY`) | **Yes** (metadata identifier, not a secret) | `grok`: `--env-key XAI_API_KEY`<br>`codex`: `--set env_key=OPENROUTER_API_KEY` | Yes (`env_key` string in `profiles.toml`) |
| **Secret Value** (actual API key / bearer token) | **Never** in args or literal shell strings | `printf '%s' "$OPENROUTER_API_KEY" \| ccr <platform> profile <create\|edit> <name> --auth-token-stdin --json` | Stored only via CCR's protected catalog / `0600` sidecar |
