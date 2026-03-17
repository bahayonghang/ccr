# CCR Web Frontend (Legacy)

This directory contains the legacy web frontend used by `ccr web`.

If you want the primary graphical experience, use `ccr ui` / the main `ccr-ui/` app instead. Keep this directory for compatibility, automation, embedded legacy browser flows, and the Rust binary's built-in web assets.

## Development

The project uses [Vite](https://vitejs.dev/) and follows the repository's Bun-first frontend workflow.

### Prerequisites

- Node.js 18+
- Bun 1.3+

### Setup

```bash
cd web
bun install --frozen-lockfile
```

### Development Server

```bash
bun run dev
```

The dev server proxies API requests to `http://localhost:19527` (the Rust backend) by default.
If you start the backend on a different port, set `CCR_WEB_PORT`:

```bash
# PowerShell
$env:CCR_WEB_PORT=5645
bun run dev
```

### Building for Embedded Use

To build the legacy frontend for embedding into the Rust binary:

```bash
bun run build
```

This command generates the following files in `dist/`:
- `index.html`
- `script.js`
- `style.css`

**Important**: Run `bun run build` before compiling the Rust project if you changed this frontend. The Rust server (`src/web/server.rs`) embeds these built files at compile time.

## Port Binding Notes (Windows)

On some Windows environments, binding to `0.0.0.0:19527` (or a nearby range) may fail with `os error 10013` due to system/security policies or port reservations.

Current behavior:
- CCR tries `0.0.0.0:19527..19536`
- If that is blocked, it falls back to `localhost` or an OS-assigned random port and prints the final URL in the logs.

If you need a stable port, start with an explicit port:

```bash
ccr web --port 18080
```

## Project Structure

- `src/js/`: JavaScript source modules
  - `main.js`: Entry point
  - `api.js`: API client
  - `state.js`: State management
  - `ui.js`: UI logic
  - `render.js`: DOM rendering
- `src/css/`: CSS source files
  - `style.css`: Main stylesheet (imports others)
  - `variables.css`: CSS variables and themes
- `dist/`: Production build output (embedded by Rust)
- `vite.config.js`: Vite configuration
