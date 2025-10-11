# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

CCR (Claude Code Configuration Switcher) is a Rust CLI tool that manages Claude Code configurations by directly manipulating `~/.claude/settings.json`. It provides configuration switching, audit trails, automatic backups, file locking, and a web interface.

## Development Commands

### Building & Running

```bash
# Quick development cycle
cargo check              # Fast type checking (recommended during dev)
cargo build              # Debug build
cargo build --release    # Optimized release build

# Run with arguments
cargo run -- <command>
cargo run -- --help
cargo run -- switch anthropic

# Using justfile (if just is installed)
just build               # Debug build
just release             # Release build
just run -- <args>       # Run debug build
just run-release -- <args>  # Run release build
```

### Testing & Code Quality

```bash
# Run all tests
cargo test
just test

# Code quality checks
cargo clippy             # Linting
just clippy              # Clippy with warnings as errors
cargo fmt                # Format code
just fmt
```

### Installation

```bash
# Install to ~/.cargo/bin
cargo install --path . --locked
just install

# Reinstall (force)
just reinstall

# Uninstall
cargo uninstall ccr
just uninstall
```

### Documentation

```bash
cargo doc --no-deps      # Generate docs
cargo doc --no-deps --open  # Generate and open in browser
just doc-open
```

## Architecture

CCR follows a strict **layered architecture** with clear separation of concerns:

```
┌─────────────────────────────────────┐
│   CLI Layer (main.rs + commands/)   │  ← User Interface
├─────────────────────────────────────┤
│   Web Layer (web/)                  │  ← Web Interface  
├─────────────────────────────────────┤
│   Service Layer (services/)         │  ← Business Logic
├─────────────────────────────────────┤
│   Manager Layer (managers/)         │  ← Data Access
├─────────────────────────────────────┤
│   Core Layer (core/)                │  ← Infrastructure
├─────────────────────────────────────┤
│   Utils Layer (utils/)              │  ← Utilities
└─────────────────────────────────────┘
```

### Directory Structure

```
src/
├── main.rs                          # CLI entry point
├── lib.rs                           # Library entry point
│
├── commands/                        # 🎯 CLI Layer
│   ├── mod.rs
│   ├── clean.rs, current.rs, export.rs
│   ├── history_cmd.rs, import.rs, init.rs
│   ├── list.rs, optimize.rs, switch.rs
│   ├── update.rs, validate.rs
│   └── [13 command files total]
│
├── web/                             # 🌐 Web Layer
│   ├── mod.rs
│   ├── handlers.rs                  # Request handlers
│   ├── models.rs                    # API data models
│   ├── routes.rs                    # Route definitions
│   ├── server.rs                    # HTTP server
│   └── system_info_cache.rs         # System info caching
│
├── services/                        # 🎯 Service Layer
│   ├── mod.rs
│   ├── backup_service.rs            # Backup operations
│   ├── config_service.rs            # Configuration CRUD
│   ├── history_service.rs           # History tracking
│   └── settings_service.rs          # Settings management
│
├── managers/                        # 📁 Manager Layer
│   ├── mod.rs
│   ├── config.rs                    # ~/.ccs_config.toml manager
│   ├── history.rs                   # Operation history manager
│   └── settings.rs                  # ~/.claude/settings.json manager
│
├── core/                            # 🏗️ Core Layer
│   ├── mod.rs
│   ├── atomic_writer.rs             # Atomic file operations
│   ├── error.rs                     # Error types & codes
│   ├── file_manager.rs              # File manager trait
│   ├── lock.rs                      # File locking mechanism
│   └── logging.rs                   # Colored output & logging
│
└── utils/                           # 🛠️ Utils Layer
    ├── mod.rs
    ├── mask.rs                      # Sensitive data masking
    └── validation.rs                # Validation trait
```

### Layer Breakdown

#### 🎯 CLI Layer (`main.rs` + `commands/`)
- **`main.rs`**: CLI entry point using `clap` for argument parsing
- **`commands/`**: Command implementations (13 commands total)
  - Each command in its own file
  - Calls Service layer for business logic
  - Handles user interaction and output formatting

#### 🌐 Web Layer (`web/`)
- **`server.rs`**: HTTP server using `tiny_http`
- **`handlers.rs`**: Request handlers for 11 API endpoints
- **`models.rs`**: API data models (request/response)
- **`routes.rs`**: Route definitions
- **`system_info_cache.rs`**: System info caching for performance

#### 🎯 Service Layer (`services/`)
- **`config_service.rs`**: Configuration business logic (CRUD, validation, import/export)
- **`settings_service.rs`**: Settings management (apply config, backup/restore)
- **`history_service.rs`**: Operation history tracking and querying
- **`backup_service.rs`**: Backup cleanup and scanning

#### 📁 Manager Layer (`managers/`)
- **`config.rs`**: ConfigManager - manages `~/.ccs_config.toml`
- **`settings.rs`**: SettingsManager - manages `~/.claude/settings.json`
- **`history.rs`**: HistoryManager - manages `~/.claude/ccr_history.json`

#### 🏗️ Core Layer (`core/`)
- **`atomic_writer.rs`**: Atomic file write operations
- **`error.rs`**: Error types with exit codes (13 error types)
- **`file_manager.rs`**: File manager trait definition
- **`lock.rs`**: File locking mechanism with timeout protection
- **`logging.rs`**: Colored terminal output utilities

#### 🛠️ Utils Layer (`utils/`)
- **`mask.rs`**: Sensitive data masking functions
- **`validation.rs`**: Validation trait for data structures

### Key Design Patterns

1. **Atomic Operations**: All file writes use temp file + rename to prevent partial updates
2. **File Locking**: Ensures multi-process safety when modifying settings
3. **Audit Trail**: Every operation is logged with timestamp and actor
4. **Backup Strategy**: Automatic backup before destructive operations
5. **Desensitization**: API tokens are masked in display/logs (shows first/last chars only)

### Critical File Paths

```
~/.ccs_config.toml          # Main configuration (shared with CCS shell version)
~/.claude/settings.json     # Claude Code settings (managed by CCR)
~/.claude/backups/          # Automatic backups with timestamps
~/.claude/ccr_history.json  # Operation audit trail
~/.claude/.locks/           # File lock directory
```

## Development Guidelines

### Adding New Commands

1. Create new module in `src/commands/<name>.rs`
2. Implement command function with signature: `fn command() -> Result<()>`
3. Use Service layer for business logic:
   ```rust
   use crate::services::ConfigService;
   
   pub fn my_command() -> Result<()> {
       let service = ConfigService::default()?;
       let result = service.some_operation()?;
       // Display result using ColorOutput
       Ok(())
   }
   ```
4. Export in `src/commands/mod.rs`
5. Add command variant in `main.rs` `Commands` enum with clap attributes
6. Route command in `main.rs` match statement

### Using Service Layer

The Service layer encapsulates business logic. Always prefer Service methods over direct Manager access:

```rust
// ✅ Good: Use Service layer
use crate::services::ConfigService;

let config_service = ConfigService::default()?;
let configs = config_service.list_configs()?;

// ❌ Bad: Direct Manager access (bypass business logic)
use crate::managers::config::ConfigManager;

let manager = ConfigManager::default()?;
let config = manager.load()?;
```

Available Services:
- **ConfigService** (`services/config_service.rs`): Configuration CRUD operations
- **SettingsService** (`services/settings_service.rs`): Settings file management
- **HistoryService** (`services/history_service.rs`): Operation history tracking
- **BackupService** (`services/backup_service.rs`): Backup cleanup operations

### Error Handling

- Use `CcrError` types from `core::error`
- Fatal errors return exit code 1, non-fatal return 0
- Provide user-friendly messages via `user_message()` method
- Use `ColorOutput` from `core::logging` for consistent error display

```rust
use crate::core::error::{CcrError, Result};
use crate::core::logging::ColorOutput;

pub fn my_command() -> Result<()> {
    // Your logic
    Ok(())
}
```

### File Operations

- **Read before write**: Always read existing file before modifications to ensure Edit tool compatibility
- **Use managers**: Always use Manager layer for file operations:
  - `SettingsManager` (`managers::settings`) for `~/.claude/settings.json`
  - `ConfigManager` (`managers::config`) for `~/.ccs_config.toml`
  - `HistoryManager` (`managers::history`) for `~/.claude/ccr_history.json`
- **Atomic writes**: Managers use `core::atomic_writer` internally for atomic updates
- **File locking**: Managers use `core::lock` for concurrent safety

```rust
use crate::managers::settings::SettingsManager;
use crate::core::lock::LockManager;

let lock_manager = LockManager::default()?;
let settings_manager = SettingsManager::default()?;

// Lock is automatically acquired when needed
settings_manager.save_atomic(&settings)?;
```

### Testing Approach

- Unit tests for individual modules
- Integration tests for command workflows
- Mock file system operations where appropriate
- Test error cases and edge conditions

## Common Tasks

### Running a Single Test

```bash
cargo test test_name
cargo test managers::config::tests::test_validation
cargo test services::config_service::tests
cargo test --test integration_test
```

### Debugging File Operations

Set log level to see detailed operation logs:
```bash
export CCR_LOG_LEVEL=debug  # trace, debug, info, warn, error
ccr switch anthropic
```

### Testing Web Interface Locally

```bash
cargo run -- web --port 8080
# Opens browser automatically to http://localhost:8080
```

### Release Process

The project uses optimized release profile (Cargo.toml):
- LTO enabled for smaller binary size
- Symbols stripped for production
- Single codegen unit for maximum optimization

```bash
cargo build --release
ls -lh target/release/ccr  # Check binary size
```

## Dependencies

Key dependencies and their purposes:
- `clap`: CLI argument parsing with derive macros
- `serde`/`toml`/`serde_json`: Configuration serialization
- `anyhow`/`thiserror`: Error handling
- `fs4`: Cross-platform file locking
- `tempfile`: Safe temporary file creation for atomic writes
- `colored`: Terminal color output
- `tiny_http`: Embedded web server
- `whoami`: Current user identification for audit trail
- `uuid`: Unique operation IDs for history tracking

## Compatibility Notes

CCR is designed to be fully compatible with CCS (shell implementation):
- Shares same configuration file format (`~/.ccs_config.toml`)
- Can coexist and alternate usage between CCR and CCS
- Configuration changes made by either tool are immediately visible to the other
