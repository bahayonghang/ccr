# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

CCR UI is a full-stack web application for managing CCR (Claude Code Configuration Switcher) configurations and executing commands through a visual interface.

**Tech Stack:**
- **Frontend**: Next.js 16 Beta, React 19, TypeScript, Turbopack, Tailwind CSS
- **Backend**: Actix Web (Rust), Tokio async runtime
- **Task Runner**: Just command runner (recommended workflow tool)

**Key Architecture Pattern:**
The backend serves as a secure gateway that spawns CCR CLI as subprocesses. All commands are validated against a whitelist before execution. Frontend communicates with backend via REST API at `/api/*`.

## Development Workflow

### Prerequisites Check
```bash
just check-prereqs    # Verify Rust, Node.js, npm, and CCR are installed
```

### Essential Commands

**Quick Start (Most Common):**
```bash
just s                # Start development servers (alias for 'just dev')
just i                # Install all dependencies (alias for 'just install')
```

**Development:**
```bash
just dev              # Start both frontend (port 3000) and backend (port 8081)
just dev-frontend     # Start only Next.js dev server
just dev-backend      # Start only Actix Web server
just dev-clean        # Clean ports, caches, and processes before starting
```

**Building:**
```bash
just b                # Build production version (alias for 'just build')
just build            # Build both frontend and backend
just build-backend    # Cargo build --release
just build-frontend   # next build
```

**Code Quality:**
```bash
just c                # Check code (alias for 'just check')
just check            # Run cargo check + npm lint
just t                # Run tests (alias for 'just test')
just f                # Format code (alias for 'just fmt')
just clippy           # Run Rust linter with strict warnings
```

**Maintenance:**
```bash
just clean            # Clean build artifacts
just clean-logs       # Remove logs older than 14 days
just update           # Update all dependencies
```

**Running Production:**
```bash
just run-prod         # Start production backend binary
just serve-frontend   # Serve frontend static files (Python HTTP server)
```

### Manual Commands (Without Just)

**Backend:**
```bash
cd backend
cargo build --release       # Build production binary
cargo run                   # Start dev server
cargo run -- --port 8082    # Custom port
cargo test                  # Run tests
cargo fmt                   # Format code
```

**Frontend:**
```bash
cd frontend
npm install                 # Install dependencies
npm run dev                 # Start Next.js dev server
npm run build               # Build production bundle
npm run lint                # ESLint check
```

## Architecture Highlights

### Backend Structure (`backend/src/`)

- **handlers/** - API route handlers organized by domain:
  - `config.rs` - Config management (list, switch, validate, export/import)
  - `command.rs` - CCR command execution with subprocess spawning
  - `system.rs` - System information endpoints
  - `version.rs` - Version checking and updates

- **executor/** - CLI subprocess executor with:
  - Command validation against whitelist
  - 30-second timeout per command
  - Proper stdout/stderr capture
  - Exit code and duration tracking

- **models.rs** - Request/response models and API types
- **config_reader.rs** - TOML config file parsing
- **main.rs** - Server initialization, CORS, routing, logging setup

**Logging System:**
- Uses `tracing` + `tracing-appender` for structured logging
- Logs automatically rotate daily in `logs/` directory
- Retention period: 14 days (run `just clean-logs` to clean)
- Both console and file output supported

### Frontend Structure (`frontend/src/`)

**Next.js App Router** (`app/`):
- `configs/` - Configuration management page
- `commands/` - Command executor interface
- `agents/`, `mcp/`, `plugins/`, `slash-commands/` - Extended functionality pages
- `layout.tsx` - Root layout with sidebar and global providers
- `page.tsx` - Homepage

**Components** (`components/`):
- `layout/` - Layout components (navbar, sidebar)
- `providers/` - Context providers for state management
- `ui/` - Reusable UI components (buttons, cards, etc.)
- `sidebar/` - Sidebar navigation components
- `history/` - Operation history display

**Utilities** (`lib/`):
- `api/client.ts` - Axios-based API client for backend communication
- `types/` - TypeScript type definitions

### Request Flow

1. User interacts with React component
2. Component calls API client method (`lib/api/client.ts`)
3. API client sends HTTP request to backend (`/api/*`)
4. Backend handler validates request
5. Executor spawns CCR CLI subprocess with validated args
6. CCR CLI executes command and modifies `~/.claude/settings.json`
7. Output captured and returned as JSON
8. Frontend displays result with syntax highlighting

### Security Model

- **Command Whitelist**: Only 13 predefined CCR commands allowed
- **No Shell Injection**: Arguments passed as array to `tokio::process::Command`
- **Timeout Protection**: 30-second max execution per command
- **CORS**: Configured for localhost development (restrictive in production)
- **Path Safety**: All file operations use validated paths

## Common Development Patterns

### Adding a New API Endpoint

1. Define request/response models in `backend/src/models.rs`
2. Implement handler function in appropriate `handlers/*.rs` file
3. Register route in `backend/src/main.rs` under `HttpServer::new()`
4. Add corresponding method in `frontend/src/lib/api/client.ts`
5. Create or update page component in `frontend/src/app/`

### Testing Backend Changes

```bash
cd backend
cargo test                    # Run all tests
RUST_LOG=debug cargo run      # Verbose logging for debugging
```

### Testing Frontend Changes

```bash
cd frontend
npm run lint                  # Check for linting errors
npm run type-check            # TypeScript validation (if available)
```

## Troubleshooting

**Port conflicts:**
- Run `just dev-clean` to kill processes on ports 3000 and 8081
- Or manually: `lsof -ti:3000 | xargs kill -9`

**Backend won't start:**
- Ensure CCR is installed: `ccr --version`
- Check logs in `logs/backend.log` for errors

**Frontend can't connect:**
- Verify backend is running on port 8081
- Check browser console for API errors
- Ensure API requests proxy correctly in Next.js config

**Build failures:**
- Clean and rebuild: `just clean && just build`
- Update dependencies: `just update`

## Important Notes

- **Next.js Version**: Using Next.js 16 Beta with Turbopack (bleeding edge, expect occasional issues)
- **React Version**: React 19 (stable release)
- **Node Version**: Requires Node.js >= 20.9.0
- **CCR Dependency**: Backend requires `ccr` binary in PATH
- **Log Rotation**: Logs auto-rotate daily, clean old logs with `just clean-logs`

## Project Context

This UI project is a separate interface layer for the main CCR CLI tool. It does not modify CCR's core code but acts as a frontend wrapper that:
- Executes CCR commands via subprocess
- Visualizes CCR configurations
- Provides real-time command output
- Offers a user-friendly alternative to CLI usage

Related projects: CCR (main CLI), CCS (shell version), CCR Tauri (desktop app).
