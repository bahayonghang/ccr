# CCR Web Frontend

This directory contains the source code for the CCR web interface.

## Development

The project uses [Vite](https://vitejs.dev/) for development and building.

### Prerequisites

- Node.js (Latest LTS recommended)
- npm (comes with Node.js)

### Setup

Install dependencies:

```bash
cd web
npm install
```

### Development Server

Start the development server with hot module replacement (HMR):

```bash
npm run dev
```

The dev server proxies API requests to `http://localhost:19527` (the Rust backend) by default.
If you start the backend on a different port, set `CCR_WEB_PORT`:

```bash
# PowerShell
$env:CCR_WEB_PORT=5645
npm run dev
```

### Building for Production

To build the frontend for embedding into the Rust binary:

```bash
npm run build
```

This command generates the following files in `dist/`:
- `index.html`
- `script.js`
- `style.css`

**Important**: You MUST run `npm run build` before compiling the Rust project if you have made changes to the frontend code. The Rust server (`src/web/server.rs`) embeds these built files at compile time.

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
