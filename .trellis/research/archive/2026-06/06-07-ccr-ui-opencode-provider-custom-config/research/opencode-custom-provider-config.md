# OpenCode custom provider config research

Date: 2026-06-07

## Sources

- OpenCode providers docs: https://opencode.ai/docs/providers/
- OpenCode config schema: https://opencode.ai/config.json
- OpenCode docs source link from the providers page: https://github.com/anomalyco/opencode/edit/dev/packages/web/src/content/docs/providers.mdx

## Findings

OpenCode's official providers page documents custom providers under the top-level `provider` object. The provider object key is the provider ID used by OpenCode, and its value can include `npm`, `name`, `options`, and `models`.

For OpenAI-compatible endpoints, the official example uses:

```json
{
  "$schema": "https://opencode.ai/config.json",
  "provider": {
    "myprovider": {
      "npm": "@ai-sdk/openai-compatible",
      "name": "My AI Provider",
      "options": {
        "baseURL": "https://api.example.com/v1",
        "apiKey": "{env:OPENAI_API_KEY}"
      },
      "models": {
        "my-model": {
          "name": "My Model",
          "limit": {
            "context": 200000,
            "output": 65536
          }
        }
      }
    }
  }
}
```

The provider page also states that custom OpenAI-compatible providers should use `@ai-sdk/openai-compatible` for `/v1/chat/completions` style endpoints, and `@ai-sdk/openai` when the model uses `/v1/responses`. Mixed setups can override the package per model through `provider.npm`.

The published schema confirms:

- `Config.provider` is an object whose values are `ProviderConfig`.
- `ProviderConfig` supports `api`, `name`, `env`, `id`, `npm`, `whitelist`, `blacklist`, `options`, and `models`.
- `ProviderConfig.options` supports `apiKey`, `baseURL`, `timeout`, `headerTimeout`, `chunkTimeout`, `setCacheKey`, and related provider options.
- `ProviderConfig.models` values support `name`, `limit`, `options`, `headers`, `variants`, and a per-model `provider` object with `npm` and `api`.
- `Config.agent` supports agent-level `options`, which allows the user's `agent.build.options.store` and `agent.plan.options.store` shape.

## Sanitized target shape

Use placeholder secrets only:

```json
{
  "$schema": "https://opencode.ai/config.json",
  "provider": {
    "openai": {
      "npm": "@ai-sdk/openai-compatible",
      "name": "OpenAI Compatible",
      "options": {
        "baseURL": "https://api.example.com/v1",
        "apiKey": "<YOUR_API_KEY>"
      },
      "models": {
        "gpt-5.2": {
          "name": "GPT-5.2",
          "limit": {
            "context": 400000,
            "output": 128000
          },
          "options": {
            "store": false
          },
          "variants": {
            "low": {},
            "medium": {},
            "high": {},
            "xhigh": {}
          }
        }
      }
    }
  },
  "agent": {
    "build": {
      "options": {
        "store": false
      }
    },
    "plan": {
      "options": {
        "store": false
      }
    }
  }
}
```

## Implications for ccr-ui

- `options.apiKey`, `options.baseURL`, and `models` are valid, but `npm` must be a top-level provider field, not a nested option.
- The current `openai-compatible` preset ID is likely misleading as an OpenCode provider ID. The provider ID can be arbitrary, while the OpenAI-compatible implementation is selected by `npm: "@ai-sdk/openai-compatible"`.
- CCR UI should either expose `npm` in the provider editor or auto-fill it for OpenAI-compatible presets. The safer MVP is to support both: a visible `npm package` field plus preset defaults.
- The user-provided real API key must never be copied into fixtures, tests, task files, screenshots, or docs.
