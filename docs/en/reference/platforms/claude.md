# Claude Platform Guide

Claude (`claude`) is a first-class platform in CCR's explicit runtime and profile catalog model. For the full CLI syntax and JSON contracts, see the [`claude` command reference](/en/reference/commands/claude).

## Main current path

```bash
# Initialize and edit the Claude profile catalog (~/.ccr/platforms/claude/profiles.db)
ccr claude profile init
printf '%s' "$ANTHROPIC_AUTH_TOKEN" | ccr claude profile create work \
  --base-url https://api.example.com \
  --auth-token-stdin
ccr claude profile open

# Inspect, switch, and leave profile mode
ccr claude auth current
ccr claude profile list
ccr claude profile switch work
ccr claude profile current
ccr claude profile off

# Export and import ExchangeV1 TOML
ccr claude profile export --no-secrets -o claude-profiles.toml
ccr claude profile import claude-profiles.toml --dry-run
```

## Model

- `ccr claude auth ...`: manage official-auth account snapshots and login state
- `ccr claude profile ...`: manage the `profiles.db` catalog and apply a saved profile into `~/.claude/settings.json`
- `ccr claude profile off`: leave profile mode, clear managed environment variables, and return to the official-auth runtime

## Third-party models (GLM / DeepSeek / Kimi, etc.)

To run a third-party model in Claude Code, the profile must use **api_key** mode (not `subscription`):

- `auth_mode = "api_key"`: writes `ANTHROPIC_BASE_URL` / `ANTHROPIC_AUTH_TOKEN` plus the model mappings.
- `auth_mode = "subscription"`: clears all `ANTHROPIC_*` on apply and falls back to official login — the third-party config is discarded.

To prevent misconfiguration, any profile that has both `base_url` and `auth_token`, or `provider_type = third_party_model`, is automatically corrected to `auth_mode = api_key` on save and on apply (`ccr-ui` also shows an inline warning).

Model-mapping fields → environment variables:

| Profile field | Environment variable |
| --- | --- |
| `default_opus_model` | `ANTHROPIC_DEFAULT_OPUS_MODEL` |
| `default_sonnet_model` | `ANTHROPIC_DEFAULT_SONNET_MODEL` |
| `default_haiku_model` | `ANTHROPIC_DEFAULT_HAIKU_MODEL` |
| `default_fable_model` | `ANTHROPIC_DEFAULT_FABLE_MODEL` |
| `default_opus_model_name` | `ANTHROPIC_DEFAULT_OPUS_MODEL_NAME` |
| `default_sonnet_model_name` | `ANTHROPIC_DEFAULT_SONNET_MODEL_NAME` |
| `default_haiku_model_name` | `ANTHROPIC_DEFAULT_HAIKU_MODEL_NAME` |
| `default_fable_model_name` | `ANTHROPIC_DEFAULT_FABLE_MODEL_NAME` |
| `subagent_model` | `CLAUDE_CODE_SUBAGENT_MODEL` |
| `custom_model_option` | `ANTHROPIC_CUSTOM_MODEL_OPTION` |
| `custom_model_option_name` | `ANTHROPIC_CUSTOM_MODEL_OPTION_NAME` |
| `effort_level` | `CLAUDE_CODE_EFFORT_LEVEL` |
| `claude_code_auto_compact_window` | `CLAUDE_CODE_AUTO_COMPACT_WINDOW` |
| `api_timeout_ms` | `API_TIMEOUT_MS` |
| `claude_code_disable_nonessential_traffic` | `CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC` |

### Z.AI / GLM template

CCR's built-in GLM Claude Code template uses the Z.AI Anthropic-compatible endpoint:

```toml
base_url = "https://api.z.ai/api/anthropic"
provider = "glm"
provider_type = "third_party_model"
auth_mode = "api_key"
default_opus_model = "glm-5.2[1m]"
default_sonnet_model = "glm-5.2[1m]"
default_haiku_model = "glm-4.7"
default_fable_model = "glm-5.2[1m]"
claude_code_auto_compact_window = "1000000"
api_timeout_ms = "3000000"
claude_code_disable_nonessential_traffic = "1"
```

Templates never include a real API key. After creating the profile, pipe your Z.AI / GLM key via stdin, switch to the profile, and verify with `doctor`:

```bash
printf '%s' "$ZAI_API_KEY" | ccr claude profile edit <name> --auth-token-stdin
ccr claude profile switch <name>
ccr doctor --platform claude
```

When applying an API-key profile, CCR updates the `env` block in `~/.claude/settings.json` with the profile-managed environment variables and tries to write `hasCompletedOnboarding = true` in `~/.claude.json`. If `~/.claude.json` is corrupted or not writable, profile switching still continues and `ccr doctor` reports an onboarding warning.

Notes:

- The Claude Code `/model` picker still shows the Opus/Sonnet/Haiku labels — it does not rename built-in aliases to third-party IDs, but the model you mapped is used under the hood.
- Suffixes like `[1m]` in `glm-5.2[1m]` require a recent Claude Code version.
- `ccr doctor --platform claude` reports common issues such as placeholder tokens, `settings.json` environment mismatches against the active profile, missing GLM 1M compact-window settings, and missing onboarding state.

## Key paths

- Host runtime settings: `~/.claude/settings.json`
- Claude Code state file: `~/.claude.json`
- Authoritative profile catalog: `~/.ccr/platforms/claude/profiles.db` (`profiles.toml` is used only for `ExchangeV1` export/import and `profile open` projections)
- Registry pointer: `[claude].current_profile` in `~/.ccr/config.toml`

## Related docs

- [`claude` command reference](/en/reference/commands/claude)
- [Platform Support](./index)
- [Platform Command Migration](./migration)
