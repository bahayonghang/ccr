# CCR Core Module (src/)

[Root Directory](../CLAUDE.md) > **src**

## Change Log (Changelog)
- **2025-10-22 00:04:36 CST**: Initial module documentation created

## Module Responsibilities

The `src/` module is the core CLI application of CCR, implementing the main configuration management logic. It provides:

1. **CLI Interface**: Complete command-line interface with 13+ commands
2. **Service Layer**: Business logic orchestration (6 services)
3. **Manager Layer**: Data access and persistence (3 managers)
4. **Web API**: Lightweight Axum-based REST API (14 endpoints)
5. **TUI**: Interactive terminal user interface
6. **Core Infrastructure**: Error handling, file locking, atomic writes, logging

This module is designed as both a standalone binary (`ccr`) and a library that can be used programmatically.

## Entry and Startup

### Main Entry Point

**File**: `/home/lyh/Documents/Github/ccr/src/main.rs`

**Startup Flow**:
```rust
fn main() {
    // 1. Initialize logger
    init_logger();

    // 2. Parse CLI arguments (Clap)
    let cli = Cli::parse();

    // 3. Route to command handler
    let result = match cli.command {
        Commands::List => commands::list_command(),
        Commands::Switch { config_name } => commands::switch_command(&config_name),
        // ... other commands
    };

    // 4. Handle errors and exit
    if let Err(e) = result {
        eprintln!("{}", e.user_message());
        std::process::exit(e.exit_code());
    }
}
```

### Library Entry Point

**File**: `/home/lyh/Documents/Github/ccr/src/lib.rs`

**Public API Exports**:
- Core types: `CcrError`, `Result`, `ColorOutput`, `LockManager`
- Managers: `ConfigManager`, `SettingsManager`, `HistoryManager`
- Services: `ConfigService`, `SettingsService`, `HistoryService`, `BackupService`
- Models: `CcsConfig`, `ClaudeSettings`, `ConfigSection`, `HistoryEntry`

### Initialization Sequence

1. **Logger**: Env-based logger (controlled by `CCR_LOG_LEVEL`)
2. **Config Loading**: Read `~/.ccs_config.toml` on demand
3. **Lock Acquisition**: Automatic file locking for concurrent operations
4. **Settings Validation**: Check `~/.claude/settings.json` structure

## External Interfaces

### CLI Commands

#### Configuration Management
- `ccr init [--force]` - Initialize config file
- `ccr list` - List all configurations (table format)
- `ccr current` - Show current config and environment
- `ccr switch <name>` - Switch to configuration
- `ccr add` - Add new configuration (interactive)
- `ccr delete <name> [--force]` - Delete configuration
- `ccr optimize` - Sort configs alphabetically

#### Operations and History
- `ccr validate` - Validate all configs and settings
- `ccr history [-l N] [-t TYPE]` - View operation history
- `ccr export [-o FILE] [--no-secrets]` - Export configuration
- `ccr import <FILE> [--merge] [--force]` - Import configuration
- `ccr clean [-d DAYS] [--dry-run]` - Clean old backups

#### Cloud Sync
- `ccr sync config` - Configure WebDAV sync
- `ccr sync status` - Check sync status
- `ccr sync push [--force]` - Upload to cloud
- `ccr sync pull [--force]` - Download from cloud

#### User Interfaces
- `ccr tui [--yes]` - Launch terminal UI
- `ccr web [-p PORT]` - Launch web API server (default: 8080)
- `ccr ui [-p PORT] [--backend-port PORT]` - Launch full UI app

#### Utilities
- `ccr update [--check]` - Update from GitHub
- `ccr version` - Show version and features

### Web API Endpoints (Port 8080)

#### Configuration Management
```
GET    /api/configs           - List all configurations
POST   /api/switch            - Switch configuration
POST   /api/config            - Create new config section
POST   /api/config/{name}     - Update config section
DELETE /api/config/{name}     - Delete config section
```

#### Settings and History
```
GET    /api/settings          - Get current Claude settings
GET    /api/settings/backups  - List backup files
POST   /api/settings/restore  - Restore from backup
GET    /api/history           - View operation history
```

#### Operations
```
POST   /api/validate          - Validate configs
POST   /api/export            - Export configuration
POST   /api/import            - Import configuration
POST   /api/clean             - Clean old backups
GET    /api/system            - Get system information
```

### TUI Keyboard Shortcuts

- `1-4` / `Tab` / `Shift+Tab` - Switch tabs
- `↑↓` / `j`/`k` - Navigate lists
- `Enter` - Switch to selected config
- `d` - Delete config (requires YOLO mode)
- `Y` - Toggle YOLO mode (auto-confirm)
- `q` / `Ctrl+C` - Quit

## Key Dependencies and Configuration

### Rust Dependencies (Cargo.toml)

**CLI & Serialization**:
- `clap` 4.5.50 - Command-line parsing with derive macros
- `serde` 1.0.228 - Serialization framework
- `serde_json` 1.0.145 - JSON support
- `toml` 0.9.8 - TOML parsing
- `indexmap` 2.12 - Ordered maps for config

**Error Handling**:
- `anyhow` 1.0.100 - Flexible error handling
- `thiserror` 2.0.17 - Custom error derive macros

**File System**:
- `dirs` 6.0.0 - Cross-platform user directories
- `fs4` 0.13.1 - File locking
- `tempfile` 3.23 - Atomic file operations
- `filetime` 0.2 - File timestamp manipulation

**Async Web**:
- `tokio` 1.48 - Async runtime
- `axum` 0.8.6 - Web framework
- `tower-http` 0.6.6 - CORS middleware

**Terminal UI**:
- `ratatui` 0.29 - TUI framework
- `crossterm` 0.29 - Terminal control
- `comfy-table` 7.2 - Table formatting

**Logging & Output**:
- `log` 0.4.28 - Logging facade
- `env_logger` 0.11.8 - Environment-based logger
- `colored` 3.0.0 - Colored terminal output

**Utilities**:
- `chrono` 0.4.42 - Date/time handling
- `uuid` 1.18.1 - UUID generation
- `whoami` 1.6 - Current user identification
- `sysinfo` 0.37.2 - System information
- `rayon` 1.11 - Parallel processing
- `reqwest_dav` 0.2 - WebDAV client

### Environment Variables

**Logging**:
- `CCR_LOG_LEVEL` - Log level (trace|debug|info|warn|error)
- `RUST_LOG` - Alternative log control

**Managed in settings.json**:
- `ANTHROPIC_BASE_URL` - API endpoint
- `ANTHROPIC_AUTH_TOKEN` - Auth token (masked in logs)
- `ANTHROPIC_MODEL` - Default model
- `ANTHROPIC_SMALL_FAST_MODEL` - Fast model (optional)

### Configuration Files

**Main Config**: `~/.ccs_config.toml`
```toml
default_config = "anthropic"
current_config = "anthropic"

[settings]
skip_confirmation = false
auto_backup = true
backup_retention_days = 7

[settings.sync]
enabled = false
webdav_url = ""
username = ""
password = ""
remote_path = "/ccr/.ccs_config.toml"
auto_sync = false

[anthropic]
description = "Anthropic Official API"
base_url = "https://api.anthropic.com"
auth_token = "sk-ant-xxx"
model = "claude-sonnet-4-5-20250929"
```

**Settings Target**: `~/.claude/settings.json`
```json
{
  "env": {
    "ANTHROPIC_BASE_URL": "https://api.anthropic.com",
    "ANTHROPIC_AUTH_TOKEN": "sk-ant-xxx",
    "ANTHROPIC_MODEL": "claude-sonnet-4-5-20250929"
  }
}
```

## Data Models

### Core Types

**ConfigSection** (`src/managers/config.rs`):
```rust
pub struct ConfigSection {
    pub description: Option<String>,
    pub base_url: String,
    pub auth_token: String,
    pub model: String,
    pub small_fast_model: Option<String>,
}
```

**ClaudeSettings** (`src/managers/settings.rs`):
```rust
pub struct ClaudeSettings {
    pub env: HashMap<String, String>,
}
```

**HistoryEntry** (`src/managers/history.rs`):
```rust
pub struct HistoryEntry {
    pub id: String,              // UUID
    pub timestamp: DateTime<Utc>,
    pub operation_type: OperationType,
    pub actor: String,           // System username
    pub details: OperationDetails,
    pub result: OperationResult,
}
```

### Error Types

**CcrError** (`src/core/error.rs`):
```rust
pub enum CcrError {
    ConfigNotFound(String),
    ConfigFileError(String),
    SettingsError(String),
    ValidationError(Vec<String>),
    LockError(String),
    IoError(String),
    // ... more variants
}
```

Each error maps to a specific exit code:
- `ConfigNotFound` → Exit code 2
- `ValidationError` → Exit code 3
- `LockError` → Exit code 4
- Fatal errors → Exit code 1

## Testing and Quality

### Test Files

Located in `/home/lyh/Documents/Github/ccr/tests/`:

1. **integration_test.rs** - Core integration tests
2. **manager_tests.rs** - Manager layer tests
3. **service_workflow_tests.rs** - Service layer tests
4. **concurrent_tests.rs** - Concurrency and locking tests
5. **end_to_end_tests.rs** - Full workflow tests
6. **add_delete_test.rs** - Config CRUD operations

### Test Coverage

- **Target**: 95%+ overall coverage
- **Unit Tests**: Embedded in modules with `#[cfg(test)]`
- **Integration Tests**: Separate test files simulating real usage
- **Concurrent Tests**: Multi-threaded scenarios to verify locking

### Running Tests

```bash
# All tests
cargo test

# Specific category
cargo test --test concurrent_tests

# With output
cargo test -- --nocapture

# Single test
cargo test test_switch_config

# Coverage report (requires cargo-tarpaulin)
cargo tarpaulin --out Html
```

### Quality Checks

```bash
# Linting
cargo clippy --all-targets --all-features

# Formatting
cargo fmt --check

# Security audit
cargo audit

# Dependency tree
cargo tree
```

## Architecture Layers

### 1. CLI/Web Layer

**Responsibilities**:
- Parse command-line arguments (Clap)
- Route to appropriate command handler
- Handle user input/output
- Launch web server or TUI

**Key Files**:
- `src/main.rs` - CLI entry point
- `src/commands/*.rs` - 13+ command implementations
- `src/web/*.rs` - Web server and API routes
- `src/tui/*.rs` - Terminal UI

### 2. Service Layer

**Responsibilities**:
- Orchestrate business logic
- Coordinate multiple managers
- Provide transaction-like operations
- Ensure consistency

**Key Files**:
- `src/services/config_service.rs` - Config operations
- `src/services/settings_service.rs` - Settings management
- `src/services/history_service.rs` - Audit logging
- `src/services/backup_service.rs` - Backup operations
- `src/services/sync_service.rs` - WebDAV sync
- `src/services/ui_service.rs` - UI launcher

### 3. Manager Layer

**Responsibilities**:
- Data access and persistence
- File I/O operations
- Data validation
- Query and update operations

**Key Files**:
- `src/managers/config.rs` - Config file management
- `src/managers/settings.rs` - Settings file management
- `src/managers/history.rs` - History file management

### 4. Core/Utils Layer

**Responsibilities**:
- Error types and handling
- File locking mechanism
- Atomic file writer
- Logging and colored output
- Validation utilities
- Sensitive data masking

**Key Files**:
- `src/core/error.rs` - Custom error types
- `src/core/lock.rs` - File locking
- `src/core/atomic_writer.rs` - Atomic file operations
- `src/core/logging.rs` - Colored output
- `src/utils/validation.rs` - Validation helpers
- `src/utils/mask.rs` - Sensitive data masking

## Frequently Asked Questions (FAQ)

### Q: How does CCR ensure concurrent safety?

A: CCR uses file-based locking (`fs4` crate) with automatic lock acquisition and release. Each operation acquires a lock before modifying files, preventing corruption from simultaneous access.

### Q: What happens if CCR crashes during a config switch?

A: All file writes use atomic operations (write to temp file → rename). If the process crashes, the original file remains intact. Plus, automatic backups are created before destructive operations.

### Q: How are API keys protected?

A: API keys are masked in all output (logs, history, display) using pattern matching. The masking logic is in `src/utils/mask.rs`.

### Q: Can I use CCR and CCS together?

A: Yes! Both tools share the same `~/.ccs_config.toml` file and can coexist. They use different locking mechanisms so won't interfere.

### Q: Where are backups stored?

A: Automatic backups are in `~/.claude/backups/` with timestamped names like `settings_20250101_120000.json.bak`.

### Q: How do I enable debug logging?

A: Set environment variable: `export CCR_LOG_LEVEL=debug` before running CCR.

### Q: What's the difference between `ccr web` and `ccr ui`?

A:
- `ccr web` - Lightweight API server (14 endpoints, port 8080) for programmatic access
- `ccr ui` - Full-stack web app (129 endpoints backend + Vue frontend, ports 38081/3000) for visual management

## Related File List

### Source Code
- `/home/lyh/Documents/Github/ccr/src/main.rs` - CLI entry point
- `/home/lyh/Documents/Github/ccr/src/lib.rs` - Library exports
- `/home/lyh/Documents/Github/ccr/src/commands/` - CLI commands (13 files)
- `/home/lyh/Documents/Github/ccr/src/services/` - Service layer (6 files)
- `/home/lyh/Documents/Github/ccr/src/managers/` - Manager layer (3 files)
- `/home/lyh/Documents/Github/ccr/src/core/` - Core infrastructure (5 files)
- `/home/lyh/Documents/Github/ccr/src/web/` - Web server (4 files)
- `/home/lyh/Documents/Github/ccr/src/tui/` - Terminal UI (5 files)
- `/home/lyh/Documents/Github/ccr/src/utils/` - Utilities (3 files)

### Configuration
- `/home/lyh/Documents/Github/ccr/Cargo.toml` - Rust dependencies
- `/home/lyh/Documents/Github/ccr/.gitignore` - Git ignore rules

### Tests
- `/home/lyh/Documents/Github/ccr/tests/*.rs` - Integration tests (6 files)

### Documentation
- `/home/lyh/Documents/Github/ccr/README.md` - Project readme
- `/home/lyh/Documents/Github/ccr/README_CN.md` - Chinese readme
