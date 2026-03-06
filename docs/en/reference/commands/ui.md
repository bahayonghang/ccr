# ui - Launch CCR UI

`ccr ui` is the recommended browser-oriented entrypoint for CCR.

## Usage

```bash
ccr ui [-p <frontend-port>] [--backend-port <port>]
ccr ui update
ccr ui help
```

## Defaults

- frontend port: `15173`
- backend port: `38081`

## Startup Order

1. `ccr-ui/` in the current or parent directory
2. `~/.ccr/ccr-ui/`
3. prompted GitHub download on first use

## Examples

```bash
ccr ui
ccr ui -p 15173 --backend-port 38081
ccr ui update
```

## When To Use It

- You want the main browser-based CCR experience
- You need the full module surface: skills, sessions, monitoring, statusline, provider health, checkin, opencode, and more
- You want CCR to auto-detect a local `ccr-ui/` checkout during development

## How It Differs from `ccr web`

- `ccr ui`: recommended entrypoint tied to the full `ccr-ui` product surface
- `ccr web`: legacy/programmatic path kept for compatibility and HTTP automation

See [Choosing `ccr ui` vs `ccr web`](/en/guide/web-guide).
