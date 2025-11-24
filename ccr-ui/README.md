# CCR UI - Vue 3 + Axum + Tauri Full-Stack Application

A graphical/desktop interface for CCR (Claude Code Configuration Switcher). Frontend: Vue 3 + Vite + Pinia, Backend: Axum. Supports both Web mode and Tauri desktop mode. Version 3.6.2.

## Features Overview

### Core Features
- **Configuration Management**: Visual view, switch, validate, import/export, history, backup management - covering all CLI capabilities
- **Command Execution**: Graphical execution of all CCR commands with real-time output
- **Multi-Platform Support**: Unified management of AI tool configurations (Claude Code, Codex, Gemini CLI, Qwen)
- **WebDAV Sync**: Multi-directory sync, registration management, batch and single directory push/pull/status
- **Format Conversion**: Convert configurations between different platforms (Claude ↔ Codex ↔ Gemini)
- **Desktop Support**: Native desktop app built with Tauri 2.0

### Supported AI Platforms
| Platform | Status | Description | Config File Path |
|----------|--------|-------------|------------------|
| Claude Code | ✅ Fully Supported | Anthropic's official CLI | `~/.claude/settings.json` |
| Codex | ✅ Fully Supported | GitHub Copilot CLI | `~/.codex/config.json` |
| Gemini CLI | ✅ Fully Supported | Google Gemini CLI | `~/.gemini/settings.json` |
| Qwen | ✅ Fully Supported | Alibaba Tongyi Qianwen CLI | `~/.qwen/config.json` |
| iFlow | 🚧 Basic Support | iFlow CLI | In development |

## Quick Start

### Recommended Method (Using CCR CLI)
```bash
ccr ui                  # Auto-detect local source, user directory, or download from GitHub
ccr ui -p 3000          # Custom frontend port
ccr ui --backend-port 8081  # Custom backend port
# Default ports: Frontend 3000, Backend 8081
```

CCR CLI searches for ccr-ui in the following priority order:
1. **Development Environment** - `./ccr-ui/` or `../ccr-ui/` (for developers)
2. **User Directory** - `~/.ccr/ccr-ui/` (for daily use)
3. **GitHub Download** - Prompts to download from GitHub (first-time users)

### Development Environment (Using just)
```bash
cd ccr-ui
just quick-start        # First-time: check dependencies → install → start
just s                  # Start frontend + backend development mode (most common)
```

**Common just commands**:
- `just i` - Install frontend and backend dependencies
- `just b` - Build production version (backend + frontend)
- `just c` - Code check (clippy + format check + TypeScript check)
- `just t` - Run tests
- `just f` - Format code
- `just s` - Start development environment (frontend 5173 + backend 8081)
- `just run-prod` - Run production environment
- `just tauri-dev` - Tauri desktop development
- `just tauri-build` - Build desktop installer
- `just --list` - View all available commands

## Prerequisites

- **Rust 1.85+** (workspace shares dependencies)
- **Node.js 18+** (npm package manager)
- **CCR CLI** (installed and visible in PATH)
- **just** (optional but recommended: `cargo install just`)
- **Tauri Dependencies** (optional, for desktop mode):
  - Linux: `libwebkit2gtk-4.0-dev build-essential`
  - macOS: Xcode Command Line Tools
  - Windows: Visual Studio C++ Build Tools

## Project Architecture

### Workspace Structure
```
ccr/ (workspace root)
├── Cargo.toml          # Shared dependencies configuration
├── src/                # CCR CLI + core library
├── ccr-ui/            # CCR UI full-stack application
│   ├── backend/        # Axum REST API server (129 endpoints)
│   ├── frontend/       # Vue 3 + Vite + Pinia frontend
│   └── docs/           # VitePress documentation site
└── justfile            # Development task automation
```

### Frontend-Backend Communication
- **Frontend → Backend**: Axios HTTP requests (REST API)
- **Backend → CLI**: Direct CCR command invocation + file system operations
- **State Management**: Pinia stores (client-side state)
- **API Endpoint**: Default `http://localhost:8081`

### Technology Stack

**Backend (Rust)**
- Web framework: `axum` 0.7 + `tokio` async runtime
- HTTP middleware: `tower` 0.5 + `tower-http`
- Serialization: `serde` + `serde_json` + `toml`
- Error handling: `anyhow` + `thiserror`
- Logging: `tracing` + `tracing-subscriber`
- HTTP client: `reqwest`
- Configuration management: `dirs` + `tempfile`

**Frontend (Vue 3)**
- Framework: `vue` 3.5.22 + `vue-router` 4.4
- State management: `pinia` 2.2.6
- HTTP client: `axios` 1.7.9
- UI components: `lucide-vue-next` (icons)
- Styling: `tailwindcss` 3.4.17
- Build tool: `vite` 7.1.11
- Type checking: `typescript` 5.7 + `vue-tsc` 2.2

## API Endpoints in Detail

### 1. Claude Code API (33 endpoints)

**MCP Server Management** (`/api/mcp`)
- `GET    /api/mcp` - List all MCP servers
- `POST   /api/mcp` - Add MCP server
- `PUT    /api/mcp/:name` - Update MCP server
- `DELETE /api/mcp/:name` - Delete MCP server
- `PUT    /api/mcp/:name/toggle` - Enable/disable MCP server

**Agents** (`/api/agents`)
- `GET    /api/agents` - List all Agents
- `POST   /api/agents` - Add Agent
- `PUT    /api/agents/:name` - Update Agent
- `DELETE /api/agents/:name` - Delete Agent
- `PUT    /api/agents/:name/toggle` - Enable/disable Agent

**Slash Commands** (`/api/slash-commands`)
- `GET    /api/slash-commands` - List all Slash commands
- `POST   /api/slash-commands` - Add Slash command
- `PUT    /api/slash-commands/:name` - Update Slash command
- `DELETE /api/slash-commands/:name` - Delete Slash command
- `PUT    /api/slash-commands/:name/toggle` - Enable/disable Slash command

**Plugins** (`/api/plugins`)
- `GET    /api/plugins` - List all plugins
- `POST   /api/plugins` - Add plugin
- `PUT    /api/plugins/:name` - Update plugin
- `DELETE /api/plugins/:name` - Delete plugin
- `PUT    /api/plugins/:name/toggle` - Enable/disable plugin

**Configuration Management** (`/api/config`)
- `GET    /api/config` - Get Claude configuration
- `PUT    /api/config` - Update Claude configuration

### 2. Codex API (33 endpoints)

**Prefix: `/api/codex/`**
Supports Profiles, MCP, Agents, Slash Commands, Plugins management
- `GET    /api/codex/config` - Get Codex configuration
- `PUT    /api/codex/config` - Update Codex configuration

### 3. Gemini CLI API (28 endpoints)

**Prefix: `/api/gemini-cli/`**
Supports MCP, Agents, Slash Commands, Plugins, Config management

### 4. Qwen API (28 endpoints)

**Prefix: `/api/qwen/`**
Supports MCP, Agents, Slash Commands, Plugins, Config management

### 5. iFlow API (5 endpoints - Basic Support)
- `GET    /api/iflow/mcp` - Get iFlow MCP server
- `POST   /api/iflow/mcp` - Add/update iFlow MCP server
- `GET    /api/iflow/agents` - Get iFlow Agents
- `GET    /api/iflow/slash-commands` - Get iFlow Slash commands
- `GET    /api/iflow/plugins` - Get iFlow plugins

### 6. Utility APIs
- `POST   /api/converter/convert` - Convert config format
- `POST   /api/sync/claude-to-codex` - Sync Claude to Codex
- `POST   /api/command/execute` - Execute CCR CLI command
- `GET    /api/system/info` - Get system information
- `GET    /api/version` - Get backend version

### 7. CCR Core APIs
- `GET    /api/configs` - List all configurations
- `POST   /api/switch` - Switch configuration
- `POST   /api/validate` - Validate configuration
- `POST   /api/export` - Export configuration
- `POST   /api/import` - Import configuration
- `GET    /api/history` - View operation history

## Frontend Routes

### Main Routes
```
/                        - Home/Dashboard (Platform Overview)
/configs                 - CCR configuration management
/commands                - CCR command executor
/converter               - Config format converter
/sync                    - WebDAV sync management
```

### Claude Code Routes
```
/claude                  - Claude overview
/mcp                     - MCP server management
/agents                  - Agents management
/slash-commands          - Slash commands management
/plugins                 - Plugins management
```

### Codex Routes
```
/codex                   - Codex overview
/codex/profiles          - Profiles management
/codex/mcp               - MCP server management
/codex/agents            - Agents management
/codex/slash-commands    - Slash commands management
/codex/plugins           - Plugins management
```

### Gemini CLI Routes
```
/gemini-cli              - Gemini overview
/gemini-cli/mcp          - MCP server management
/gemini-cli/agents       - Agents management
/gemini-cli/slash-commands
/gemini-cli/plugins
```

### Qwen Routes
```
/qwen                    - Qwen overview
/qwen/mcp                - MCP server management
/qwen/agents             - Agents management
/qwen/slash-commands
/qwen/plugins
```

### iFlow Routes (In Development)
```
/iflow                   - iFlow overview
/iflow/mcp               - MCP server management
/iflow/agents            - Agents management
/iflow/slash-commands
/iflow/plugins
```

## Manual Development (Without just)

### Backend Development
```bash
cd ccr-ui/backend
cargo run -- --port 8081              # Start development server (default 8081)
cargo watch -x run                    # Auto-reload on changes
RUST_LOG=debug cargo run              # Enable debug logging
```

### Frontend Development
```bash
cd ccr-ui/frontend
npm install                           # Install dependencies
npm run dev                           # Start dev server (http://localhost:5173)
npm run build                         # Build production version
npm run type-check                    # TypeScript type checking
npm run lint                          # ESLint checking
```

Frontend accesses backend API at `http://localhost:8081` (configurable via environment variables).

### Environment Variables

**Backend Environment Variables**
```bash
RUST_LOG=info              # Log level: trace, debug, info, warn, error
RUST_BACKTRACE=1          # Enable error backtrace
PORT=8081                 # Server port number
```

**Frontend Environment Variables** (`.env` or `.env.local`)
```bash
VITE_API_BASE_URL=http://localhost:8081    # Backend API URL
VITE_LOG_LEVEL=debug                        # Frontend log level
```

## Production Deployment

### Method 1: Using just Commands
```bash
cd ccr-ui
just build              # Build backend + frontend production version
just run-prod           # Run backend and serve frontend static files
```

### Method 2: Manual Build
```bash
# Build backend
cd ccr-ui/backend
cargo build --release
cp target/release/ccr-ui-backend ../dist/

# Build frontend
cd ../frontend
npm install && npm run build
cp -r dist/* ../dist/static/

# Run
./dist/ccr-ui-backend --port 8081 --static-dir ./dist/static
```

**Build Artifacts**:
- Backend executable: `ccr-ui/backend/target/release/ccr-ui-backend`
- Frontend static files: `ccr-ui/frontend/dist/`

### Docker Deployment (Optional)
```dockerfile
FROM rust:1.85 as backend-builder
WORKDIR /app/ccr-ui/backend
COPY . .
RUN cargo build --release

FROM node:18 as frontend-builder
WORKDIR /app/ccr-ui/frontend
COPY frontend .
RUN npm install && npm run build

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    libssl1.1 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=backend-builder /app/ccr-ui/backend/target/release/ccr-ui-backend /usr/local/bin/
COPY --from=frontend-builder /app/ccr-ui/frontend/dist /usr/local/share/ccr-ui/static
EXPOSE 8081
CMD ["ccr-ui-backend", "--port", "8081", "--static-dir", "/usr/local/share/ccr-ui/static"]
```

## Tauri Desktop Mode

### Development Mode
```bash
cd ccr-ui
just tauri-dev          # Start Tauri development environment
```

### Build Desktop Application
```bash
just tauri-build        # Build installer
# Artifacts in src-tauri/target/release/bundle/
```

### Platform-Specific Dependencies

**Linux Ubuntu/Debian**
```bash
sudo apt update
sudo apt install libwebkit2gtk-4.0-dev \
    build-essential \
    curl \
    wget \
    file \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev
```

**macOS**
```bash
# Install Xcode Command Line Tools
xcode-select --install
```

**Windows**
- Install [Visual Studio Community](https://visualstudio.microsoft.com/)
- Select "C++ build tools" and Windows 10/11 SDK

## Data Models

### Backend Core Models (Rust)

**MCP Server** (`models/claude.rs`)
```rust
pub struct McpServer {
    pub command: String,
    pub args: Vec<String>,
    pub env: Option<HashMap<String, String>>,
}
```

**Agents** (`models/claude.rs`)
```rust
pub struct Agent {
    pub name: String,
    pub description: String,
    pub instructions: String,
}
```

**Slash Command** (`models/claude.rs`)
```rust
pub struct SlashCommand {
    pub name: String,
    pub description: String,
    pub command: String,
}
```

**Plugins** (`models/claude.rs`)
```rust
pub struct Plugin {
    pub name: String,
    pub enabled: bool,
    pub config: serde_json::Value,
}
```

**Codex Profile** (`models/codex.rs`)
```rust
pub struct CodexProfile {
    pub name: String,
    pub description: String,
    pub settings: serde_json::Value,
}
```

## Testing

### Backend Tests
```bash
cd ccr-ui/backend
cargo test              # Run all tests
cargo test --lib       # Run unit tests only
cargo test -- --nocapture  # Show test output
```

### Frontend Tests
```bash
cd ccr-ui/frontend
npm run type-check     # TypeScript type checking
npm run lint           # ESLint checking
npm test              # Run tests (if test framework is configured)
```

### End-to-End Tests (Optional)
Use Playwright or Cypress for E2E testing:
```bash
npm install -D playwright
npx playwright test    # Run E2E tests
```

## Troubleshooting

### Common Issues

**1. Backend Port Already in Use**
```bash
# Solution 1: Use different port
cargo run -- --port 9090

# Solution 2: Find and kill the process
sudo lsof -i :8081
kill -9 <PID>
```

**2. Frontend Cannot Connect to Backend**
- Confirm backend is running at `http://localhost:8081`
- Check browser console Network panel
- Confirm CORS configuration is enabled
- Check firewall settings

**3. CLI Invocation Error**
- Confirm `ccr` is in PATH
- Check version: `ccr --version` (should be 3.6.2+)
- Enable debug logging: `CCR_LOG_LEVEL=debug ccr ui`
- Check permissions: `chmod +x ~/.ccr/ccr-ui/backend/target/release/ccr-ui-backend`

**4. Node.js or npm Errors**
- Confirm Node.js version: `node --version` (must be ≥ 18)
- Clean npm cache: `npm cache clean --force`
- Delete node_modules and reinstall: `rm -rf node_modules && npm install`

**5. Tauri Build Failure**
- Linux: Confirm `libwebkit2gtk-4.0-dev` is installed
- macOS: Confirm Xcode CLT is installed
- Windows: Confirm Visual Studio C++ Build Tools are installed

**6. CORS Error**
Backend allows all origins by default. To restrict:
```rust
// backend/src/main.rs
.layer(
    CorsLayer::new()
        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
)
```

## Configuration File Paths

### AI Tool Configurations
- **Claude Code**: `~/.claude/settings.json`
- **Codex**: `~/.codex/config.json`
- **Gemini CLI**: `~/.gemini/settings.json`
- **Qwen**: `~/.qwen/config.json`

### CCR UI Related
- **Logs**: `~/.ccr/logs/` or `./ccr-ui/logs/`
- **Backend Logs**: `~/.ccr/logs/ccr-ui-backend.log`
- **Frontend Build Artifacts**: `ccr-ui/frontend/dist/`
- **Tauri Config**: `ccr-ui/src-tauri/tauri.conf.json`

## FAQ

### Q: How does ccr-ui communicate with the backend?
A: The Vue frontend uses Axios to send REST API requests to the Axum backend (port 8081). All state is managed client-side with Pinia stores.

### Q: Can I customize frontend and backend ports?
A: Yes!
```bash
ccr ui -p 3000 --backend-port 8081    # Frontend 3000, backend 8081
```

### Q: Where are configuration files stored?
A:
- Claude Code: `~/.claude/settings.json`
- Codex: `~/.codex/config.json`
- Gemini: `~/.gemini/settings.json`
- Qwen: `~/.qwen/config.json`

### Q: How to add support for a new CLI tool?
A:
1. Add config reader in `backend/src/config/`
2. Add models in `backend/src/models/`
3. Add handlers in `backend/src/handlers/`
4. Add routes in `backend/src/main.rs`
5. Add frontend views in `frontend/src/views/`
6. Update router in `frontend/src/router/`

### Q: What is the liquid glass design style?
A: A modern glassmorphism design featuring:
- Frosted glass effects (backdrop-filter: blur)
- Subtle gradients and shadows
- Smooth transition animations
- Light/dark theme support

### Q: How to deploy to production?
A:
```bash
cd ccr-ui
just b                   # Build frontend
cargo build --release    # Build backend
# Copy artifacts to server and run
```

### Q: What's the difference between Web mode and Tauri mode?
A:
- **Web Mode**: Runs in browser, accessed via HTTP
- **Tauri Mode**: Native desktop app, uses system Webview, better performance, can access system APIs

### Q: Which browsers are supported?
A:
- Chrome/Edge 90+
- Firefox 88+
- Safari 14+
- IE is not supported

### Q: How to contribute code?
A:
1. Fork the repository
2. Create feature branch: `git checkout -b feature/my-feature`
3. Commit changes: `git commit -am 'Add new feature'`
4. Push to branch: `git push origin feature/my-feature`
5. Create Pull Request

### Q: How to debug backend API?
A:
- Start backend with `cargo run -- --port 8081`
- Visit `http://localhost:8081/api/version` to verify
- Test API with Postman or curl
- View logs: `tail -f ~/.ccr/logs/ccr-ui-backend.log`

### Q: Does it support multiple users?
A: Currently ccr-ui is single-user. Each user uses their own config directory (`~/.claude/`, etc.).

## Performance Optimization Tips

### Backend Optimization
- Build with `cargo build --release` for production
- Enable Rust LTO (Link Time Optimization)
- Properly configure tokio thread pool size

### Frontend Optimization
- Build production with `npm run build` (enables Tree Shaking)
- Configure CDN for static assets
- Enable Gzip/Brotli compression
- Use HTTP/2 or HTTP/3

## Security Recommendations

### Backend Security
- Enable HTTPS in production (via Nginx reverse proxy)
- Restrict API access origins
- Add authentication middleware for sensitive operations
- Regularly update dependencies:
  ```bash
  cd backend && cargo update
  cd frontend && npm update
  ```

### Frontend Security
- Validate all user input
- Prevent XSS attacks (Vue escapes content by default)
- Configure CSP (Content Security Policy)
- Don't expose sensitive information in logs

## Contributing Guidelines

### Code Standards
- **Rust**: Follow rustfmt and clippy suggestions
- **Vue**: Use Composition API with `<script setup>` syntax
- **TypeScript**: Enable strict mode
- **Git Commit**: Follow Conventional Commits

### Development Workflow
1. Create feature branch from `dev`
2. Write code and add tests
3. Run tests: `just t`
4. Code review: `just c`
5. Submit PR to `dev` branch

## License

MIT License (same as main project)

---

**Last Updated**: 2025-01-24
**Version**: 3.6.2
**Documentation Status**: ✅ Complete

## Related Links

- [CCR Main Project](../README.md)
- [CCR Documentation](docs/)
- [GitHub Repository](https://github.com/bahayonghang/ccr)
- [Issue Tracker](https://github.com/bahayonghang/ccr/issues)
