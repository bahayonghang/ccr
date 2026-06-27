# Claude Platform Guide

Claude is a first-class platform in the current explicit runtime/profile model.

## Main current path

```bash
ccr claude auth current
ccr claude profile list
ccr claude profile switch <name>
ccr claude profile current
ccr claude profile off
```

## Model

- `ccr claude auth ...`: official-auth account management
- `ccr claude profile ...`: apply a saved profile into `~/.claude/settings.json`
- `ccr claude profile off`: leave profile mode and return to the official-auth runtime

## Third-party models (GLM / DeepSeek / Kimi, etc.)

To run a third-party model in Claude Code, the profile must use **api_key** mode (not subscription):

- `auth_mode = "api_key"`: writes `ANTHROPIC_BASE_URL` / `ANTHROPIC_AUTH_TOKEN` plus the model mappings.
- `auth_mode = "subscription"`: clears all `ANTHROPIC_*` on apply and falls back to official login — the third-party config is discarded.

To prevent misconfiguration, any profile that has both `base_url` and `auth_token`, or `provider_type = third_party_model`, is automatically corrected to `auth_mode = api_key` on save and on apply (ccr-ui also shows an inline warning).

Model-mapping fields → environment variables:

| Profile field                              | Environment variable                          |
| ------------------------------------------ | --------------------------------------------- |
| `default_opus_model`                       | `ANTHROPIC_DEFAULT_OPUS_MODEL`                |
| `default_sonnet_model`                     | `ANTHROPIC_DEFAULT_SONNET_MODEL`              |
| `default_haiku_model`                      | `ANTHROPIC_DEFAULT_HAIKU_MODEL`               |
| `default_fable_model`                      | `ANTHROPIC_DEFAULT_FABLE_MODEL`               |
| `default_opus_model_name`                  | `ANTHROPIC_DEFAULT_OPUS_MODEL_NAME`           |
| `default_sonnet_model_name`                | `ANTHROPIC_DEFAULT_SONNET_MODEL_NAME`         |
| `default_haiku_model_name`                 | `ANTHROPIC_DEFAULT_HAIKU_MODEL_NAME`          |
| `default_fable_model_name`                 | `ANTHROPIC_DEFAULT_FABLE_MODEL_NAME`          |
| `subagent_model`                           | `CLAUDE_CODE_SUBAGENT_MODEL`                  |
| `custom_model_option`                      | `ANTHROPIC_CUSTOM_MODEL_OPTION`               |
| `custom_model_option_name`                 | `ANTHROPIC_CUSTOM_MODEL_OPTION_NAME`          |
| `effort_level`                             | `CLAUDE_CODE_EFFORT_LEVEL`                    |
| `claude_code_auto_compact_window`         | `CLAUDE_CODE_AUTO_COMPACT_WINDOW`             |
| `api_timeout_ms`                          | `API_TIMEOUT_MS`                              |
| `claude_code_disable_nonessential_traffic` | `CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC`    |

### Z.AI / GLM template

CCR's built-in GLM Claude Code template currently uses the Z.AI Anthropic-compatible endpoint:

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

Templates never include a real API key. After creating the profile, set `auth_token` to your own Z.AI / GLM key, then run:

```bash
ccr claude profile switch <name>
ccr doctor --platform claude
```

When applying an API-key profile, CCR updates `~/.claude/settings.json.env` with the profile-managed environment variables and tries to write `hasCompletedOnboarding = true` in `~/.claude.json`. If `~/.claude.json` is corrupted or not writable, profile switching still continues and `ccr doctor` reports an onboarding warning.

Notes:

- The Claude Code `/model` picker still shows the Opus/Sonnet/Haiku labels — it does not rename built-in aliases to third-party IDs, but the model you mapped is used under the hood.
- Suffixes like `[1m]` in `glm-5.2[1m]` require a recent Claude Code version.
- `ccr doctor --platform claude` reports common issues such as placeholder tokens, `settings.json.env` mismatches against the active profile, missing GLM 1M compact-window settings, and missing onboarding state.

## Key paths

- Runtime settings: `~/.claude/settings.json`
- Claude Code state: `~/.claude.json`
- Profiles: `~/.ccr/platforms/claude/profiles.toml`
- Registry pointer: `[claude].current_profile` in `~/.ccr/config.toml`
