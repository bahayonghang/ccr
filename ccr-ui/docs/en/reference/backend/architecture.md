# Backend Architecture

CCR UI backend is a high-performance REST API server built with Rust and Axum.

## Tech Stack

### Core Framework

- **Rust 2024 Edition** - System programming language
  - Memory safety without garbage collection
  - Zero-cost abstractions
  - Fearless concurrency

- **Axum 0.7** - Modern async web framework
  - Built on Tower and Hyper
  - Type-safe routing
  - Middleware support
  - Excellent performance

- **Tokio 1.42** - Async runtime
  - Multi-threaded work-stealing scheduler
  - Async I/O primitives
  - Timer and interval support

### Serialization

- **Serde 1.0** - Serialization framework
  - Compile-time code generation
  - Zero-overhead serialization
  - Support for JSON, TOML, etc.

- **serde_json** - JSON serialization
- **toml** - TOML parsing

### Error Handling

- **anyhow** - Flexible error handling
- **thiserror** - Custom error types

### HTTP & Networking

- **Tower** - Service middleware
- **Tower-HTTP** - HTTP-specific middleware
  - CORS
  - Compression
  - Request tracing

- **reqwest** - HTTP client

### Logging & Monitoring

- **tracing** - Structured logging
- **tracing-subscriber** - Log subscriber
- **tracing-appender** - Log file appender

### System Integration

- **whoami** - System user information
- **num_cpus** - CPU count detection
- **sysinfo** - System information

## Architecture Overview

```
┌─────────────────────────────────────────────────────┐
│                   HTTP Requests                      │
└──────────────────┬──────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────┐
│              Middleware Layer                        │
│  • CORS         • Tracing        • Compression       │
└──────────────────┬──────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────┐
│               Router Layer                           │
│  • Route matching    • Path parameters              │
└──────────────────┬──────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────┐
│              Handler Layer                           │
│  • Request validation  • Business logic             │
└──────────────────┬──────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────┐
│            Manager Layer                             │
│  • Config Manager  • Settings Manager               │
│  • Plugin Manager  • Markdown Manager               │
└──────────────────┬──────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────┐
│             Executor Layer                           │
│  • CLI Command Execution                            │
│  • Process Management                               │
└──────────────────┬──────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────┐
│            External Systems                          │
│  • CCR CLI      • File System     • Cloud APIs      │
└─────────────────────────────────────────────────────┘
```

## Project Structure

```
backend/
├── src/
│   ├── main.rs                    # Application entry
│   ├── models.rs                  # Data models
│   ├── config_reader.rs           # Config file reader
│   ├── claude_config_manager.rs   # Claude config management
│   ├── markdown_manager.rs        # Markdown file management
│   ├── plugins_manager.rs         # Plugin management
│   ├── settings_manager.rs        # Settings management
│   ├── handlers/                  # Request handlers
│   │   ├── mod.rs
│   │   ├── config.rs              # Config APIs
│   │   ├── command.rs             # Command execution APIs
│   │   ├── system.rs              # System info APIs
│   │   ├── version.rs             # Version APIs
│   │   ├── mcp.rs                 # MCP server APIs
│   │   ├── agents.rs              # Agent APIs
│   │   ├── plugins.rs             # Plugin APIs
│   │   └── slash_commands.rs      # Slash command APIs
│   ├── executor/                  # Command executor
│   │   ├── mod.rs
│   │   └── cli_executor.rs        # CLI execution logic
│   └── config_managers/           # Config managers
│       ├── claude.rs
│       ├── codex.rs
│       └── gemini.rs
├── Cargo.toml                     # Dependencies
├── examples/                      # Example configs
└── README.md                      # Documentation
```

## Core Components

### Application Entry (main.rs)

```rust
use axum::{
    Router,
    routing::{get, post, put, delete},
};
use tower_http::cors::{CorsLayer, Any};
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create router
    let app = Router::new()
        .route("/api/system/info", get(handlers::system::system_info))
        .route("/api/configs", get(handlers::config::list_configs))
        // ... more routes
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    // Start server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8081")
        .await
        .unwrap();
    
    tracing::info!("Server listening on {}", listener.local_addr().unwrap());
    
    axum::serve(listener, app)
        .await
        .unwrap();
}
```

### Data Models (models.rs)

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub name: String,
    pub description: Option<String>,
    pub base_url: String,
    pub auth_token: String,
    pub model: String,
    pub small_fast_model: Option<String>,
    pub provider: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServer {
    pub command: String,
    pub args: Vec<String>,
    pub env: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}
```

### Request Handlers

```rust
use axum::{
    extract::{State, Path, Json},
    http::StatusCode,
    response::IntoResponse,
};

// Handler example
pub async fn list_configs(
    State(state): State<AppState>,
) -> Result<Json<Vec<Config>>, ApiError> {
    let configs = config_reader::read_configs()?;
    Ok(Json(configs))
}

pub async fn switch_config(
    State(state): State<AppState>,
    Json(payload): Json<SwitchRequest>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    execute_ccr_command("switch", &[&payload.name]).await?;
    Ok(Json(ApiResponse {
        success: true,
        data: Some(()),
        error: None,
    }))
}
```

### Error Handling

```rust
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug)]
pub enum ApiError {
    ConfigNotFound,
    InvalidInput(String),
    ExecutionFailed(String),
    Internal(anyhow::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ApiError::ConfigNotFound => {
                (StatusCode::NOT_FOUND, "Config not found")
            }
            ApiError::InvalidInput(msg) => {
                (StatusCode::BAD_REQUEST, msg.as_str())
            }
            ApiError::ExecutionFailed(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, msg.as_str())
            }
            ApiError::Internal(err) => {
                tracing::error!("Internal error: {:?}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}
```

## API Endpoints

### System APIs

```
GET  /api/system/info          - Get system information
GET  /api/version              - Get backend version
```

### Configuration APIs

```
GET    /api/configs            - List all configs
POST   /api/configs/switch     - Switch config
POST   /api/configs/validate   - Validate config
GET    /api/configs/current    - Get current config
```

### Command Execution APIs

```
POST   /api/command/execute    - Execute CCR command
GET    /api/command/list       - List available commands
```

### MCP Server APIs

```
GET    /api/mcp                - List MCP servers
POST   /api/mcp                - Add MCP server
PUT    /api/mcp/:name          - Update MCP server
DELETE /api/mcp/:name          - Delete MCP server
PUT    /api/mcp/:name/toggle   - Toggle MCP server
```

### Agent APIs

```
GET    /api/agents             - List agents
POST   /api/agents             - Add agent
PUT    /api/agents/:name       - Update agent
DELETE /api/agents/:name       - Delete agent
PUT    /api/agents/:name/toggle - Toggle agent
```

### Plugin APIs

```
GET    /api/plugins            - List plugins
POST   /api/plugins            - Add plugin
PUT    /api/plugins/:name      - Update plugin
DELETE /api/plugins/:name      - Delete plugin
PUT    /api/plugins/:name/toggle - Toggle plugin
```

### Statistics APIs

```
GET    /api/stats/cost         - Get cost statistics
GET    /api/stats/cost/today   - Get today's cost
GET    /api/stats/cost/week    - Get this week's cost
GET    /api/stats/cost/month   - Get this month's cost
```

## Configuration

### Environment Variables

```bash
# Server port
PORT=8081

# Logging level
RUST_LOG=info

# CORS allowed origins
CORS_ORIGIN=http://localhost:5173
```

### Cargo.toml

```toml
[package]
name = "ccr-ui-backend"
version = "0.1.0"
edition = "2024"

[dependencies]
axum = "0.7"
tokio = { version = "1.42", features = ["full"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["cors", "trace", "compression"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.9"
anyhow = "1.0"
thiserror = "2.0"
tracing = "0.1"
tracing-subscriber = "0.3"
reqwest = { version = "0.12", features = ["json"] }
```

## Development

### Build

```bash
# Debug build
cargo build

# Release build
cargo build --release
```

### Run

```bash
# Run with default settings
cargo run

# Run with custom port
cargo run -- --port 8082

# Run with logging
RUST_LOG=debug cargo run
```

### Test

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture
```

### Lint & Format

```bash
# Check code style
cargo clippy

# Format code
cargo fmt

# Check formatting
cargo fmt -- --check
```

## Performance

### Benchmarking

```bash
# Use cargo-bench
cargo bench

# Or use external tools
wrk -t12 -c400 -d30s http://localhost:8081/api/system/info
```

### Optimization Tips

1. **Release Build**: Always use `--release` for production
2. **Connection Pooling**: Reuse HTTP clients
3. **Async I/O**: Use Tokio for all I/O operations
4. **Caching**: Cache frequently accessed data
5. **Compression**: Enable response compression

## Security

### Authentication

Currently, CCR UI backend doesn't implement authentication. Consider adding:

- JWT tokens
- API keys
- OAuth 2.0

### CORS

Configure CORS properly:

```rust
let cors = CorsLayer::new()
    .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
    .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
    .allow_headers(Any);
```

### Input Validation

Always validate user input:

```rust
#[derive(Debug, Deserialize)]
pub struct CreateConfigRequest {
    #[serde(deserialize_with = "validate_name")]
    pub name: String,
    pub base_url: Url,
}

fn validate_name<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let name = String::deserialize(deserializer)?;
    if name.is_empty() || name.len() > 100 {
        return Err(serde::de::Error::custom("Invalid name length"));
    }
    Ok(name)
}
```

## Deployment

### Build for Production

```bash
cargo build --release
```

Executable location: `target/release/ccr-ui-backend`

### Run as Service

```bash
# Using systemd
sudo systemctl start ccr-ui-backend

# Using Docker
docker run -p 8081:8081 ccr-ui-backend
```

### Monitoring

Use tracing for structured logging:

```rust
#[instrument]
async fn process_request() {
    tracing::info!("Processing request");
    // ...
}
```

## Further Reading

- [Development Guide](/en/backend/development)
- [API Documentation](/en/backend/api)
- [Deployment Guide](/en/backend/deployment)
- [Error Handling](/en/backend/error-handling)

