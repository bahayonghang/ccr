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

CCR follows a **layered architecture** with clear separation of concerns:

```
┌─────────────────────────────────────┐
│   CLI Layer (main.rs + commands/)   │  ← User Interface
├─────────────────────────────────────┤
│   Web Layer (web/)                  │  ← Web Interface  
├─────────────────────────────────────┤
│   Service Layer (services/)         │  ← Business Logic
├─────────────────────────────────────┤
│   Manager Layer (config, settings)  │  ← Data Access
├─────────────────────────────────────┤
│   Core Layer (core, utils, lock)    │  ← Infrastructure
└─────────────────────────────────────┘
```

### Layer Breakdown

#### 🎯 CLI Layer
- **`main.rs`**: CLI entry point using `clap` for argument parsing
- **`commands/`**: Command implementations (list, switch, current, validate, etc.)

#### 🌐 Web Layer
- **`web/server.rs`**: HTTP server using `tiny_http`
- **`web/handlers.rs`**: Request handlers
- **`web/models.rs`**: API data models
- **`web/routes.rs`**: Route definitions

#### 🎯 Service Layer (Business Logic)
- **`services/config_service.rs`**: Configuration business logic
- **`services/settings_service.rs`**: Settings business logic
- **`services/history_service.rs`**: History recording business logic
- **`services/backup_service.rs`**: Backup management business logic

#### 📁 Manager Layer (Data Access)
- **`config.rs`**: Configuration file management (`~/.ccs_config.toml`)
- **`settings.rs`**: Claude Code settings manager (`~/.claude/settings.json`)
- **`history.rs`**: Operation audit trail (`~/.claude/ccr_history.json`)

#### 🏗️ Core Layer (Infrastructure)
- **`core/atomic_writer.rs`**: Atomic file write operations
- **`core/file_manager.rs`**: File manager trait
- **`utils/mask.rs`**: Sensitive data masking utilities
- **`utils/validation.rs`**: Validation trait
- **`lock.rs`**: File locking abstraction
- **`logging.rs`**: Colored terminal output
- **`error.rs`**: Error types with exit codes

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
let config_service = ConfigService::default()?;
let configs = config_service.list_configs()?;

// ❌ Bad: Direct Manager access (bypass business logic)
let manager = ConfigManager::default()?;
let config = manager.load()?;
```

Available Services:
- **ConfigService**: Configuration CRUD operations
- **SettingsService**: Settings file management
- **HistoryService**: Operation history tracking
- **BackupService**: Backup cleanup operations

### Error Handling

- Use `CcrError` types from `error.rs`
- Fatal errors return exit code 1, non-fatal return 0
- Provide user-friendly messages via `user_message()` method
- Use `ColorOutput` for consistent error display

### File Operations

- **Read before write**: Always read existing file before modifications to ensure Edit tool compatibility
- **Use settings manager**: Always use `SettingsManager` methods for `settings.json` operations to ensure locking/backup
- **Use config manager**: Use `ConfigManager` for `~/.ccs_config.toml` operations to ensure validation
- **Atomic writes**: Use `tempfile` + `fs::rename` for atomic updates

### Testing Approach

- Unit tests for individual modules
- Integration tests for command workflows
- Mock file system operations where appropriate
- Test error cases and edge conditions

## Common Tasks

### Running a Single Test

```bash
cargo test test_name
cargo test config::tests::test_validation
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
