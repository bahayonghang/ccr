# ui - Launch CCR UI

`ccr ui` is the recommended graphical entrypoint for CCR.

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

- You want the main CCR graphical experience
- You need the full module surface: skills, monitoring, statusline, checkin, opencode, and more
- You want CCR to auto-detect a local `ccr-ui/` checkout during development

