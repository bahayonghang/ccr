# Web Guide

CCR has two browser-related entrypoints, but they are not equal:

| Command | Role | Recommendation |
|---------|------|----------------|
| `ccr ui` | full graphical interface | default recommendation |
| `ccr web` | legacy lightweight API / compatibility surface | use only for scripting, CI, or compatibility |

## Prefer `ccr ui`

```bash
ccr ui -p 15173 --backend-port 38081
```

Use it for:
- daily browser-based management
- platform module navigation
- usage / monitoring / skills / provider health style views
- the complete CCR module map

See [UI Overview](/en/guide/ui-overview) for the runtime model.

## Use `ccr web` only when needed

```bash
ccr web --host 127.0.0.1 --port 19527 --no-browser
```

Use it for:
- scripting against the HTTP API
- CI or automation
- legacy compatibility flows

Defaults:
- host: `127.0.0.1`
- port: `19527`
- port binding falls back automatically when occupied

## How to choose

Choose `ccr ui` when:
- you want CCR to be the main browser experience
- you need the full module surface and visual workflows

Choose `ccr web` when:
- you only need HTTP endpoints
- you are working in CI, shell automation, or remote environments
- you must preserve older scripts

## Related Pages
- [UI Overview](/en/guide/ui-overview)
- [UI Modules](/en/guide/ui-modules)
- [Web API Reference](/en/reference/api)
- [`web` command](/en/reference/commands/web)
