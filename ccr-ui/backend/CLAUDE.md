# CCR UI Backend Module

[Root Directory](../../CLAUDE.md) > [ccr-ui](../CLAUDE.md) > **backend**

## Change Log (Changelog)
- **2025-10-22 10:39:28 CST**: Initial backend module documentation created

## Module Responsibilities

The CCR UI backend is an Axum-based REST API server that provides comprehensive management interfaces for multiple AI CLI tools. Key responsibilities include:

1. **Multi-Platform Config Management** - Claude Code, Codex, Gemini CLI, Qwen, iFlow
2. **MCP Server Management** - List, add, update, delete, toggle MCP servers
3. **Agent Management** - Manage AI agents for each platform
4. **Slash Command Management** - Custom slash commands configuration
5. **Plugin Management** - Plugin installation and configuration
6. **Multi-Folder WebDAV Sync** - Independent folder management with batch operations
7. **Config Conversion** - Convert configs between different platforms
8. **Command Execution** - Execute CCR CLI commands via API
9. **System Information** - Provide system metrics and status

The backend runs as a standalone Axum server on port 8081 and communicates with the frontend via RESTful JSON APIs.

## Entry and Startup

### Main Entry Point

**File**: `/home/lyh/Documents/Github/ccr/ccr-ui/backend/src/main.rs`

**Startup Sequence**:
```rust
#[tokio::main]
async fn main() -> std::io::Result<()> {
    // 1. Parse CLI arguments
    let args = Args::parse();

    // 2. Initialize logging with file rotation
    setup_logging()?;

    // 3. Bind to address
    let bind_addr: SocketAddr = format!("{}:{}", args.host, args.port).parse()?;

    // 4. Verify CCR availability
    executor::execute_command(vec!["version".to_string()]).await?;

    // 5. Create router with all routes
    let app = create_router();

    // 6. Start server
    let listener = tokio::net::TcpListener::bind(bind_addr).await?;
    axum::serve(listener, app).await?;
}
```

### Command-Line Arguments

```bash
# Default (binds to 127.0.0.1:8081)
ccr-ui-backend

# Custom port
ccr-ui-backend --port 8082

# Custom host and port
ccr-ui-backend --host 0.0.0.0 --port 8081
```

### Module Structure

```
backend/
├── src/
│   ├── main.rs                     - Entry point and router setup
│   ├── handlers/                   - API endpoint handlers
│   │   ├── mod.rs                  - Handler module exports
│   │   ├── config.rs               - CCR config management
│   │   ├── command.rs              - Command execution
│   │   ├── system.rs               - System information
│   │   ├── version.rs              - Version and updates
│   │   ├── sync.rs                 - WebDAV sync
│   │   ├── mcp.rs                  - Claude MCP servers
│   │   ├── agents.rs               - Claude agents
│   │   ├── slash_commands.rs       - Claude slash commands
│   │   ├── plugins.rs              - Claude plugins
│   │   ├── codex.rs                - Codex management
│   │   ├── gemini.rs               - Gemini CLI management
│   │   ├── qwen.rs                 - Qwen management
│   │   ├── iflow.rs                - iFlow management (stub)
│   │   ├── converter.rs            - Config conversion
│   │   └── claude/                 - Claude-specific handlers
│   │       ├── mod.rs
│   │       ├── mcp.rs
│   │       ├── agents.rs
│   │       ├── slash_commands.rs
│   │       └── plugins.rs
│   ├── claude_config_manager.rs    - Claude config reader/writer
│   ├── codex_config_manager.rs     - Codex config reader/writer
│   ├── gemini_config_manager.rs    - Gemini config reader/writer
│   ├── qwen_config_manager.rs      - Qwen config reader/writer
│   ├── config_reader.rs            - Generic config reader
│   ├── config_converter.rs         - Config format converter
│   ├── models.rs                   - Data models
│   ├── codex_models.rs             - Codex-specific models
│   ├── gemini_models.rs            - Gemini-specific models
│   ├── qwen_models.rs              - Qwen-specific models
│   ├── converter_models.rs         - Conversion models
│   ├── plugins_manager.rs          - Plugin management
│   ├── markdown_manager.rs         - Markdown file handling
│   ├── settings_manager.rs         - Settings management
│   └── executor/                   - Command execution
│       └── mod.rs                  - CCR command executor
├── Cargo.toml                      - Dependencies
└── logs/                           - Log files (auto-created)
```

## External Interfaces

### API Endpoints (141 Total)

#### Health Check
```
GET /health - Health check endpoint
```

#### CCR Config Management (10 endpoints)
```
GET    /api/configs              - List all CCR configurations
POST   /api/switch               - Switch active configuration
POST   /api/configs              - Create new config section
PUT    /api/configs/:name        - Update config section
DELETE /api/configs/:name        - Delete config section
GET    /api/validate             - Validate configurations
POST   /api/export               - Export configuration
POST   /api/import               - Import configuration
POST   /api/clean                - Clean old backups
GET    /api/history              - Get operation history
```

#### Command Execution (3 endpoints)
```
POST /api/command/execute         - Execute CCR CLI command
GET  /api/command/list            - List available commands
GET  /api/command/help/:command   - Get command help
```

#### System & Version (3 endpoints)
```
GET  /api/system                  - Get system information
GET  /api/version                 - Get CCR version
GET  /api/version/check-update    - Check for updates
POST /api/version/update          - Update CCR
```

#### Claude Code Management (33 endpoints)

**MCP Servers (5)**:
```
GET    /api/mcp                   - List MCP servers
POST   /api/mcp                   - Add MCP server
PUT    /api/mcp/:name             - Update MCP server
DELETE /api/mcp/:name             - Delete MCP server
PUT    /api/mcp/:name/toggle      - Toggle MCP server
```

**Agents (5)**:
```
GET    /api/agents                - List agents
POST   /api/agents                - Add agent
PUT    /api/agents/:name          - Update agent
DELETE /api/agents/:name          - Delete agent
PUT    /api/agents/:name/toggle   - Toggle agent
```

**Slash Commands (5)**:
```
GET    /api/slash-commands                     - List slash commands
POST   /api/slash-commands                     - Add slash command
PUT    /api/slash-commands/:name               - Update slash command
DELETE /api/slash-commands/:name               - Delete slash command
PUT    /api/slash-commands/:name/toggle        - Toggle slash command
```

**Plugins (5)**:
```
GET    /api/plugins               - List plugins
POST   /api/plugins               - Add plugin
PUT    /api/plugins/:id           - Update plugin
DELETE /api/plugins/:id           - Delete plugin
PUT    /api/plugins/:id/toggle    - Toggle plugin
```

**Sync (17)**:
```
# Basic Sync Operations (5)
GET  /api/sync/status             - Get sync status
POST /api/sync/push               - Push config to cloud
POST /api/sync/pull               - Pull config from cloud
GET  /api/sync/info               - Get sync configuration
POST /api/sync/config             - Configure sync

# Multi-Folder Management (6)
GET    /api/sync/folders          - List all sync folders
POST   /api/sync/folders          - Add new sync folder
DELETE /api/sync/folders/:name    - Remove folder registration
GET    /api/sync/folders/:name    - Get folder details
PUT    /api/sync/folders/:name/enable  - Enable folder
PUT    /api/sync/folders/:name/disable - Disable folder

# Folder-Specific Operations (3)
POST /api/sync/folders/:name/push   - Push specific folder
POST /api/sync/folders/:name/pull   - Pull specific folder
GET  /api/sync/folders/:name/status - Get folder status

# Batch Operations (3)
POST /api/sync/all/push            - Push all enabled folders
POST /api/sync/all/pull            - Pull all enabled folders
GET  /api/sync/all/status          - Get all folders status
```

#### Codex Management (33 endpoints)

**MCP Servers (4)**:
```
GET    /api/codex/mcp             - List Codex MCP servers
POST   /api/codex/mcp             - Add Codex MCP server
PUT    /api/codex/mcp/:name       - Update Codex MCP server
DELETE /api/codex/mcp/:name       - Delete Codex MCP server
```

**Profiles (4)**:
```
GET    /api/codex/profiles        - List Codex profiles
POST   /api/codex/profiles        - Add Codex profile
PUT    /api/codex/profiles/:name  - Update Codex profile
DELETE /api/codex/profiles/:name  - Delete Codex profile
```

**Base Config (2)**:
```
GET /api/codex/config              - Get Codex base config
PUT /api/codex/config              - Update Codex base config
```

#### Gemini CLI Management (28 endpoints)

**MCP Servers (4)**:
```
GET    /api/gemini/mcp            - List Gemini MCP servers
POST   /api/gemini/mcp            - Add Gemini MCP server
PUT    /api/gemini/mcp/:name      - Update Gemini MCP server
DELETE /api/gemini/mcp/:name      - Delete Gemini MCP server
```

**Base Config (2)**:
```
GET /api/gemini/config             - Get Gemini config
PUT /api/gemini/config             - Update Gemini config
```

#### Qwen Management (28 endpoints)

**MCP Servers (4)**:
```
GET    /api/qwen/mcp              - List Qwen MCP servers
POST   /api/qwen/mcp              - Add Qwen MCP server
PUT    /api/qwen/mcp/:name        - Update Qwen MCP server
DELETE /api/qwen/mcp/:name        - Delete Qwen MCP server
```

**Base Config (2)**:
```
GET /api/qwen/config               - Get Qwen config
PUT /api/qwen/config               - Update Qwen config
```

#### iFlow Management (5 endpoints - stub)
```
GET /api/iflow/mcp                 - List iFlow MCP servers (stub)
POST /api/iflow/mcp                - Add iFlow MCP server (stub)
GET /api/iflow/agents              - List iFlow agents (stub)
GET /api/iflow/slash-commands      - List iFlow slash commands (stub)
GET /api/iflow/plugins             - List iFlow plugins (stub)
```

## Key Dependencies and Configuration

### Dependencies (Cargo.toml)

```toml
[dependencies]
# Web framework
axum = { version = "0.7", features = ["macros"] }
tower = { version = "0.5", features = ["timeout", "limit"] }
tower-http = { version = "0.6", features = ["fs", "trace", "cors", "compression-full"] }
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
tracing-subscriber = { version = "0.3", features = ["env-filter", "json", "fmt", "ansi"] }
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

### Middleware Stack

```rust
ServiceBuilder::new()
    // Logging
    .layer(TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
        .on_response(DefaultOnResponse::new().level(Level::INFO)))
    // CORS
    .layer(CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
        .max_age(Duration::from_secs(3600)))
    // Compression
    .layer(CompressionLayer::new())
```

### Environment Variables

- `RUST_LOG` - Logging level (trace, debug, info, warn, error)
  - Default: `info`
  - Example: `RUST_LOG=debug ccr-ui-backend`

### Configuration Files Managed

- **Claude Code**: `~/.claude/settings.json`
- **Codex**: `~/.codex/config.json`, `~/.codex/profiles/`, `~/.codex/mcp_servers/`
- **Gemini CLI**: `~/.gemini-cli/config.json`
- **Qwen**: `~/.qwen/config.json`
- **CCR Config**: `~/.ccs_config.toml`

## Data Models

### Core Models (`src/models.rs`)

```rust
// MCP Server
#[derive(Debug, Serialize, Deserialize)]
pub struct McpServer {
    pub command: String,
    pub args: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<HashMap<String, String>>,
}

// Agent
#[derive(Debug, Serialize, Deserialize)]
pub struct Agent {
    pub name: String,
    pub description: String,
    pub instructions: String,
    #[serde(default)]
    pub enabled: bool,
}

// Slash Command
#[derive(Debug, Serialize, Deserialize)]
pub struct SlashCommand {
    pub name: String,
    pub description: String,
    pub command: String,
    #[serde(default)]
    pub enabled: bool,
}

// Plugin
#[derive(Debug, Serialize, Deserialize)]
pub struct Plugin {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub enabled: bool,
    pub config: Value,
}
```

### Platform-Specific Models

**Codex Models** (`src/codex_models.rs`):
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct CodexProfile {
    pub name: String,
    pub description: String,
    pub settings: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CodexMcpServer {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
}
```

**Gemini Models** (`src/gemini_models.rs`):
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct GeminiConfig {
    pub api_key: String,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
}
```

**Qwen Models** (`src/qwen_models.rs`):
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct QwenConfig {
    pub api_key: String,
    pub model: String,
    pub base_url: String,
}
```

### Request/Response Models

```rust
// Command execution
#[derive(Debug, Deserialize)]
pub struct CommandRequest {
    pub command: String,
    pub args: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct CommandResponse {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub success: bool,
}

// System info
#[derive(Debug, Serialize)]
pub struct SystemInfo {
    pub hostname: String,
    pub os: String,
    pub cpu_count: usize,
    pub total_memory: u64,
    pub used_memory: u64,
    pub uptime: u64,
}
```

## Testing and Quality

### Running Tests

```bash
cd ccr-ui/backend

# Run all tests
cargo test

# Run with logging
RUST_LOG=debug cargo test -- --nocapture

# Check code
cargo clippy
cargo fmt --check

# Build
cargo build
cargo build --release
```

### Development Workflow

```bash
# Development mode with auto-reload
cargo watch -x run

# Run with custom port
cargo run -- --port 8082

# Run with debug logging
RUST_LOG=debug cargo run
```

### Code Quality

- **Linting**: All code passes `cargo clippy` without warnings
- **Formatting**: Formatted with `cargo fmt`
- **Error Handling**: Uses `anyhow` for flexibility and `thiserror` for custom errors
- **Logging**: Structured logging with `tracing` and file rotation

## Frequently Asked Questions (FAQ)

### Q: How does the backend communicate with CCR CLI?

A: The backend uses the `executor` module to spawn CCR as a subprocess and capture its output. This ensures the backend doesn't duplicate CCR's logic and always uses the canonical implementation.

### Q: How are config files modified safely?

A: All config managers use atomic file operations:
1. Read existing config
2. Parse and modify in memory
3. Write to temporary file
4. Atomic rename to target file

This prevents corruption even if the process crashes mid-write.

### Q: What happens if CCR binary is not found?

A: The server logs a warning but continues to start. API endpoints that require CCR will return error responses. This allows the server to run in environments where CCR might be installed later.

### Q: How do I enable CORS for production?

A: By default, CORS allows all origins (`Any`). For production, modify the CORS layer in `main.rs`:

```rust
CorsLayer::new()
    .allow_origin("https://yourdomain.com".parse::<HeaderValue>().unwrap())
    .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
    .allow_headers([CONTENT_TYPE, AUTHORIZATION])
```

### Q: How are logs managed?

A: Logs are written to `logs/` directory with daily rotation. The `tracing-appender` crate handles rotation automatically. Old logs are kept for 7 days by default.

### Q: Can I run the backend standalone?

A: Yes! The backend is fully independent. You can run it without the frontend for API-only usage or integrate it with custom frontends.

### Q: How do I add support for a new platform?

A:
1. Create models in `src/<platform>_models.rs`
2. Create config manager in `src/<platform>_config_manager.rs`
3. Create handlers in `src/handlers/<platform>.rs`
4. Add routes in `src/main.rs`
5. Update converter if needed

## Related File List

### Source Code
- `/home/lyh/Documents/Github/ccr/ccr-ui/backend/src/main.rs` - Entry point and router
- `/home/lyh/Documents/Github/ccr/ccr-ui/backend/src/handlers/` - API handlers (30+ files)
- `/home/lyh/Documents/Github/ccr/ccr-ui/backend/src/claude_config_manager.rs` - Claude config
- `/home/lyh/Documents/Github/ccr/ccr-ui/backend/src/codex_config_manager.rs` - Codex config
- `/home/lyh/Documents/Github/ccr/ccr-ui/backend/src/gemini_config_manager.rs` - Gemini config
- `/home/lyh/Documents/Github/ccr/ccr-ui/backend/src/qwen_config_manager.rs` - Qwen config
- `/home/lyh/Documents/Github/ccr/ccr-ui/backend/src/models.rs` - Core data models
- `/home/lyh/Documents/Github/ccr/ccr-ui/backend/src/executor/mod.rs` - Command executor

### Configuration
- `/home/lyh/Documents/Github/ccr/ccr-ui/backend/Cargo.toml` - Dependencies
- `/home/lyh/Documents/Github/ccr/ccr-ui/backend/.gitignore` - Git ignore

### Build Output
- `/home/lyh/Documents/Github/ccr/ccr-ui/backend/target/` - Build artifacts
- `/home/lyh/Documents/Github/ccr/ccr-ui/backend/logs/` - Log files
