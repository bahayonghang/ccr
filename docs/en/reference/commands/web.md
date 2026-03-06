# web - Legacy Web API

`ccr web` starts CCR's lightweight built-in HTTP service. Its role is compatibility and programmatic access, not the main graphical experience.

## Usage

```bash
ccr web [--host <host>] [--port <port>] [--no-browser]
```

## Defaults

- host: `127.0.0.1`
- port: `19527`

## Examples

```bash
# default localhost-only listener
ccr web

# explicit host / port
ccr web --host 127.0.0.1 --port 19527 --no-browser

# expose on a trusted LAN
ccr web --host 0.0.0.0 --port 19527 --no-browser
```

## Best Use Cases

- `curl`, CI, and shell automation
- legacy HTTP integrations
- lightweight API usage without the full `ccr-ui` surface

## What It No Longer Represents

This page should not describe:

- a “modern full web UI”
- the `ccr-ui` module map
- desktop-shell workflows

Those belong in [UI Overview](/en/guide/ui-overview) and [UI Module Map](/en/guide/ui-modules).

## Related Docs

- [Web API Reference](/en/reference/api)
- [Choosing `ccr ui` vs `ccr web`](/en/guide/web-guide)
