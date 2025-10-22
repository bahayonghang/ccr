# CCR UI Module

[Root Directory](../CLAUDE.md) > **ccr-ui**

## Change Log (Changelog)
- **2025-10-22 00:04:36 CST**: Initial module documentation created

## Module Responsibilities

The `ccr-ui/` module is a full-stack web application providing visual management interfaces for multiple AI CLI tools. It consists of:

1. **Backend** (`ccr-ui/backend/`) - Axum-based REST API server with 129 endpoints
2. **Frontend** (`ccr-ui/frontend-vue/`) - Vue.js 3 SPA with liquid glass design
3. **Documentation** (`ccr-ui/docs/`) - VitePress documentation site

The UI supports configuration management for:
- **Claude Code** - MCP servers, agents, slash commands, plugins
- **Codex** - Profiles, MCP servers, agents, slash commands, plugins
- **Gemini CLI** - Configuration, MCP servers, agents, slash commands, plugins
- **Qwen** - Configuration, MCP servers, agents, slash commands, plugins
- **iFlow** - Basic support (stub implementation)

## Entry and Startup

### Quick Start Methods

**Method 1: Via CCR CLI (Recommended)**
```bash
ccr ui                    # Auto-detects or downloads ccr-ui
ccr ui -p 3000           # Custom frontend port
ccr ui --backend-port 8081  # Custom backend port
```

**Method 2: Direct Development**
```bash
cd ccr-ui
just s                   # Start development (backend + frontend)
just quick-start         # First-time setup + start
```

### Three-Tier Detection Priority

When running `ccr ui`, CCR searches in this order:

1. **Development Environment** - `./ccr-ui/` or `../ccr-ui/` (for developers)
2. **User Directory** - `~/.ccr/ccr-ui/` (for daily use)
3. **GitHub Download** - Prompts to download from GitHub (first-time users)

### Startup Flow

```
1. Detect ccr-ui location (or download)
2. Check Node.js/npm availability
3. Install frontend dependencies (if needed)
4. Build frontend (production) or start dev server
5. Start backend server (Axum)
6. Open browser to http://localhost:3000
```

## External Interfaces

### Backend API (129 Endpoints on Port 8081)

#### Claude Code API (33 endpoints)
```
# MCP Servers
GET    /api/mcp                     - List MCP servers
POST   /api/mcp                     - Add MCP server
PUT    /api/mcp/:name               - Update MCP server
DELETE /api/mcp/:name               - Delete MCP server
PUT    /api/mcp/:name/toggle        - Toggle MCP server

# Agents (5 endpoints)
GET/POST/PUT/DELETE /api/agents
PUT    /api/agents/:name/toggle

# Slash Commands (5 endpoints)
GET/POST/PUT/DELETE /api/slash-commands
PUT    /api/slash-commands/:name/toggle

# Plugins (5 endpoints)
GET/POST/PUT/DELETE /api/plugins
PUT    /api/plugins/:name/toggle

# Configuration
GET    /api/config                  - Get Claude config
PUT    /api/config                  - Update Claude config
```

#### Codex API (33 endpoints)
```
# MCP, Profiles, Agents, Slash Commands, Plugins
# Same pattern as Claude with /api/codex/ prefix
GET    /api/codex/config
PUT    /api/codex/config
```

#### Gemini CLI API (28 endpoints)
```
# MCP, Agents, Slash Commands, Plugins, Config
# Prefix: /api/gemini-cli/
```

#### Qwen API (28 endpoints)
```
# MCP, Agents, Slash Commands, Plugins, Config
# Prefix: /api/qwen/
```

#### iFlow API (5 endpoints - stub)
```
GET    /api/iflow/mcp
POST   /api/iflow/mcp
GET    /api/iflow/agents
GET    /api/iflow/slash-commands
GET    /api/iflow/plugins
```

#### Utility APIs
```
POST   /api/converter/convert       - Convert config formats
POST   /api/sync/claude-to-codex    - Sync Claude to Codex
POST   /api/command/execute         - Execute CCR CLI command
GET    /api/system/info             - Get system information
GET    /api/version                 - Get backend version
```

### Frontend Routes (Vue Router)

```
/                        - Home/Dashboard
/configs                 - CCR configuration management
/commands                - CCR command executor
/converter               - Config format converter
/sync                    - Sync management

# Claude Code
/claude                  - Claude overview
/mcp                     - MCP servers
/agents                  - Agents
/slash-commands          - Slash commands
/plugins                 - Plugins

# Codex
/codex                   - Codex overview
/codex/profiles          - Profiles
/codex/mcp               - MCP servers
/codex/agents            - Agents
/codex/slash-commands    - Slash commands
/codex/plugins           - Plugins

# Gemini CLI
/gemini-cli              - Gemini overview
/gemini-cli/mcp          - MCP servers
/gemini-cli/agents       - Agents
/gemini-cli/slash-commands
/gemini-cli/plugins

# Qwen
/qwen                    - Qwen overview
/qwen/mcp                - MCP servers
/qwen/agents             - Agents
/qwen/slash-commands
/qwen/plugins

# iFlow
/iflow                   - iFlow overview
/iflow/mcp               - MCP servers
/iflow/agents            - Agents
/iflow/slash-commands
/iflow/plugins
```

## Key Dependencies and Configuration

### Backend Dependencies (Rust)

**Cargo.toml** (`ccr-ui/backend/Cargo.toml`):
```toml
[dependencies]
# Web framework
axum = "0.7"
tower = "0.5"
tower-http = "0.6"        # CORS, compression, trace
tokio = { version = "1.42", features = ["full"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }

# Error handling
anyhow = "1.0"
thiserror = "2.0"

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"
tracing-appender = "0.2"

# CLI & System
clap = { version = "4.5", features = ["derive"] }
whoami = "1.5"
num_cpus = "1.16"
sysinfo = "0.32"

# Config parsing
toml = "0.9"
dirs = "6.0"
tempfile = "3.10"
serde_yaml = "0.9"

# HTTP client
reqwest = { version = "0.12", features = ["json"] }
```

### Frontend Dependencies (Vue.js)

**package.json** (`ccr-ui/frontend-vue/package.json`):
```json
{
  "dependencies": {
    "vue": "^3.5.22",
    "vue-router": "^4.4.5",
    "pinia": "^2.2.6",
    "axios": "^1.7.9",
    "lucide-vue-next": "^0.468.0",
    "tailwindcss": "^3.4.17"
  },
  "devDependencies": {
    "@vitejs/plugin-vue": "^5.2.1",
    "vite": "^7.1.11",
    "typescript": "^5.7.3",
    "vue-tsc": "^2.2.0",
    "eslint": "^9.19.0"
  }
}
```

### Environment Configuration

**Backend** (Port 8081):
- `RUST_LOG` - Logging level (info, debug, trace)
- Runs on `127.0.0.1:8081`

**Frontend** (Port 3000):
- `VITE_API_BASE_URL` - Backend API URL (default: `http://localhost:8081`)
- Development server on `localhost:3000`

## Data Models

### Backend Models

Located in `ccr-ui/backend/src/models/`:

**Claude Models** (`models/claude.rs`):
```rust
pub struct McpServer {
    pub command: String,
    pub args: Vec<String>,
    pub env: Option<HashMap<String, String>>,
}

pub struct Agent {
    pub name: String,
    pub description: String,
    pub instructions: String,
}

pub struct SlashCommand {
    pub name: String,
    pub description: String,
    pub command: String,
}

pub struct Plugin {
    pub name: String,
    pub enabled: bool,
    pub config: serde_json::Value,
}
```

**Codex Models** (`models/codex.rs`):
```rust
pub struct CodexProfile {
    pub name: String,
    pub description: String,
    pub settings: serde_json::Value,
}

pub struct CodexPlugin {
    pub id: String,
    pub name: String,
    pub enabled: bool,
}
```

**Converter Models** (`models/converter.rs`):
```rust
pub enum ConversionFormat {
    ClaudeToCodex,
    CodexToClaude,
    ClaudeToGemini,
    // ... more formats
}

pub struct ConversionRequest {
    pub format: ConversionFormat,
    pub source_config: serde_json::Value,
}
```

## Testing and Quality

### Backend Tests

```bash
cd ccr-ui/backend
cargo test               # Run all tests
cargo clippy            # Lint
cargo fmt               # Format
```

### Frontend Tests

```bash
cd ccr-ui/frontend-vue
npm run type-check      # TypeScript checking
npm run lint            # ESLint
npm test                # Run tests (if configured)
```

### Development Workflow

```bash
# Backend development
cd ccr-ui/backend
cargo run               # Start backend server
cargo watch -x run      # Auto-reload on changes

# Frontend development
cd ccr-ui/frontend-vue
npm run dev             # Start dev server with hot reload

# Full stack (using just)
cd ccr-ui
just s                  # Start both backend and frontend
just i                  # Install dependencies
just b                  # Build production
just c                  # Check code quality
just t                  # Run tests
```

## Frequently Asked Questions (FAQ)

### Q: How does ccr-ui communicate with the backend?

A: The Vue frontend uses Axios to make REST API calls to the Axum backend on port 8081. All state is managed client-side with Pinia stores.

### Q: Can I run backend and frontend on different ports?

A: Yes! Use:
```bash
ccr ui -p 3000 --backend-port 8081  # Frontend 3000, backend 8081
```

### Q: Where are configuration files located?

A:
- Claude: `~/.claude/settings.json`
- Codex: `~/.codex/config.json`
- Gemini: `~/.gemini-cli/config.json`
- Qwen: `~/.qwen/config.json`

### Q: How do I add support for a new CLI tool?

A:
1. Add config reader in `backend/src/config/`
2. Add models in `backend/src/models/`
3. Add handlers in `backend/src/handlers/`
4. Add routes in `backend/src/main.rs`
5. Add frontend views in `frontend-vue/src/views/`
6. Update router in `frontend-vue/src/router/`

### Q: What's the liquid glass design?

A: A modern glassmorphism design with:
- Frosted glass effects (backdrop-filter: blur)
- Subtle gradients and shadows
- Smooth transitions
- Light/dark theme support

### Q: How do I deploy to production?

A:
```bash
cd ccr-ui
just b                   # Build frontend
cargo build --release    # Build backend

# Copy artifacts:
# - frontend: ccr-ui/frontend-vue/dist/
# - backend: ccr-ui/backend/target/release/ccr-ui-backend
```

## Related File List

### Backend Files
- `/home/lyh/Documents/Github/ccr/ccr-ui/backend/src/main.rs` - Backend entry point
- `/home/lyh/Documents/Github/ccr/ccr-ui/backend/src/handlers/` - API handlers (30+ files)
- `/home/lyh/Documents/Github/ccr/ccr-ui/backend/src/config/` - Config readers
- `/home/lyh/Documents/Github/ccr/ccr-ui/backend/src/models/` - Data models
- `/home/lyh/Documents/Github/ccr/ccr-ui/backend/src/services/` - Business logic
- `/home/lyh/Documents/Github/ccr/ccr-ui/backend/Cargo.toml` - Rust dependencies

### Frontend Files
- `/home/lyh/Documents/Github/ccr/ccr-ui/frontend-vue/src/main.ts` - Frontend entry
- `/home/lyh/Documents/Github/ccr/ccr-ui/frontend-vue/src/App.vue` - Root component
- `/home/lyh/Documents/Github/ccr/ccr-ui/frontend-vue/src/views/` - Page components (40+ files)
- `/home/lyh/Documents/Github/ccr/ccr-ui/frontend-vue/src/components/` - Reusable components
- `/home/lyh/Documents/Github/ccr/ccr-ui/frontend-vue/src/router/` - Vue Router config
- `/home/lyh/Documents/Github/ccr/ccr-ui/frontend-vue/src/store/` - Pinia stores
- `/home/lyh/Documents/Github/ccr/ccr-ui/frontend-vue/package.json` - NPM dependencies

### Configuration
- `/home/lyh/Documents/Github/ccr/ccr-ui/justfile` - Just commands for development
- `/home/lyh/Documents/Github/ccr/ccr-ui/README.md` - CCR UI documentation
