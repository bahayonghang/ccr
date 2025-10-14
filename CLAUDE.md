# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

CCR (Claude Code Configuration Switcher) is a Rust-powered configuration management tool for Claude Code with complete audit trails, atomic operations, and multi-interface support (CLI + Web UI + Desktop App).

**Version**: 1.1.5
**Rust Edition**: 2024 (requires Rust 1.85+)
**License**: MIT

## Common Development Commands

### Core Development Workflow

```bash
# Quick development cycle
just dev              # Check + test
just watch            # Auto-rebuild on file changes (requires cargo-watch)
just ci               # Full CI: format check + clippy + test + build

# Building
just build            # Debug build
just release          # Release build (optimized with LTO)

# Testing
just test             # Run tests
just test-all         # Include ignored tests

# Code Quality
just fmt              # Format code
just fmt-check        # Check formatting (CI mode)
just clippy           # Lint with Clippy (warnings as errors)
just lint             # Format + Clippy

# Installation
just install          # Install to ~/.cargo/bin
just reinstall        # Force reinstall

# Documentation
just doc              # Build docs
just doc-open         # Build and open docs in browser

# Cleanup
just clean            # Remove build artifacts
```

### Running CCR Commands

```bash
# Run with arguments
just run -- <command> [args]
just run -- --help
just run -- list
just run -- switch anthropic

# Run release version
just run-release -- <command> [args]
```

### CCR UI (Full-Stack Web App)

```bash
cd ccr-ui

# Simplified commands (recommended)
just s              # Start development (backend + frontend)
just i              # Install dependencies
just b              # Build production
just c              # Check code
just t              # Run tests
just f              # Format code

# Or full commands
just dev            # Start development
just build          # Build production
just test           # Run tests

# Manual control
just dev-backend    # Start backend only (port 8081)
just dev-frontend   # Start frontend only (port 5173)
```

## Architecture

CCR follows a **strict layered architecture** with clear separation of concerns:

```
CLI Layer (main.rs + commands/)    → User Interface (13 commands)
Web Layer (web/)                   → HTTP API (11 endpoints)
Service Layer (services/)          → Business Logic (4 services, 26 methods)
Manager Layer (managers/)          → Data Access (3 managers)
Core Layer (core/)                 → Infrastructure (atomic writes, locks, logging)
Utils Layer (utils/)               → Utilities (validation, masking)
```

### Layer Dependency Rules

- **CLI/Web Layers** → call Service Layer only
- **Service Layer** → calls Manager Layer + Core utilities
- **Manager Layer** → uses Core infrastructure
- **Core Layer** → provides primitives (no upward dependencies)
- **Utils Layer** → standalone utilities

**CRITICAL**: NEVER bypass layers (e.g., Commands directly accessing Managers).

### Project Structure

```
src/
├── main.rs              # CLI entry point
├── lib.rs               # Library exports
├── commands/            # CLI commands (13 files)
│   ├── init.rs          # Initialize config
│   ├── list.rs          # List configs
│   ├── current.rs       # Show current config
│   ├── switch.rs        # Switch config (5-step atomic operation)
│   ├── validate.rs      # Validate configs
│   ├── optimize.rs      # Sort configs alphabetically
│   ├── history_cmd.rs   # Show operation history
│   ├── clean.rs         # Clean old backups
│   ├── export.rs        # Export configs
│   ├── import.rs        # Import configs
│   └── update.rs        # Self-update from GitHub
├── services/            # Business logic (4 services)
│   ├── config_service.rs     # Config CRUD, validation
│   ├── settings_service.rs   # Settings file management
│   ├── history_service.rs    # Operation history
│   └── backup_service.rs     # Backup management
├── managers/            # Data access (3 managers)
│   ├── config.rs        # ~/.ccs_config.toml management
│   ├── settings.rs      # ~/.claude/settings.json management
│   └── history.rs       # ~/.claude/ccr_history.json management
├── core/                # Infrastructure
│   ├── error.rs         # Error types + exit codes
│   ├── atomic_writer.rs # Atomic file writes (temp + rename)
│   ├── lock.rs          # File locking for multi-process safety
│   ├── logging.rs       # Colored console output
│   └── file_manager.rs  # File manager trait
├── utils/               # Utilities
│   ├── validation.rs    # Validation trait + implementations
│   └── mask.rs          # API token masking
└── web/                 # Web server (embedded tiny_http)
    ├── mod.rs           # Server entry
    ├── routes.rs        # Route definitions
    ├── handlers.rs      # API handlers (11 endpoints)
    ├── models.rs        # Request/response models
    └── system_info_cache.rs  # System info caching

ccr-ui/                  # Full-stack web application
├── backend/             # Actix Web server
│   ├── src/
│   │   ├── main.rs      # Server entry
│   │   ├── executor/    # CCR CLI subprocess executor
│   │   ├── handlers/    # API route handlers
│   │   └── models/      # Request/response types
│   └── Cargo.toml
└── frontend/            # React + TypeScript
    ├── src/
    │   ├── App.tsx
    │   ├── pages/       # Page components
    │   ├── components/  # Reusable components
    │   ├── api/         # API client
    │   └── types/       # TypeScript definitions
    └── package.json

tests/                   # Integration tests
docs/                    # VitePress documentation
examples/                # Example configs
```

## Critical Files & Paths

CCR manages these files in the user's home directory:

- `~/.ccs_config.toml` - Main configuration (shared with CCS shell version)
- `~/.claude/settings.json` - Claude Code settings (CCR writes to this)
- `~/.claude/backups/` - Automatic backups with timestamps
- `~/.claude/ccr_history.json` - Operation audit trail
- `~/.claude/.locks/` - File lock directory (auto-cleanup)

## Key Design Patterns

1. **Atomic Operations**: All file writes use `tempfile + rename` pattern via `AtomicFileWriter`
2. **File Locking**: Multi-process safety via `LockManager` (prevents concurrent writes)
3. **Audit Trail**: Every operation logged in `ccr_history.json` with UUID, timestamp, actor
4. **Backup Strategy**: Automatic backup before destructive operations
5. **Desensitization**: API tokens masked in display/logs via `mask` utility

## Development Guidelines

### Code Quality Standards

**Always follow these Rust patterns:**

1. **Error Handling**: Use `?` operator, avoid explicit match for simple propagation
2. **Option Handling**: Prefer combinator methods (`.map()`, `.unwrap_or_default()`)
3. **String Parameters**: Use `&str` for function parameters, not `String`
4. **Iterators**: Use iterator methods instead of manual loops
5. **Paths**: Use `PathBuf` for owned paths, never string concatenation
6. **Logging**: Use `ColorOutput::success/info/error/warning` for user messages
7. **File Operations**: ALWAYS use `AtomicFileWriter::write_atomic`, never `std::fs::write`
8. **Concurrency**: ALWAYS use `LockManager` for file access

### Service Layer Pattern

When adding new features:

```rust
use crate::managers::{ConfigManager, SettingsManager};
use crate::core::error::{CcrError, Result};

pub struct MyService {
    config_manager: ConfigManager,
    settings_manager: SettingsManager,
}

impl MyService {
    pub fn default() -> Result<Self> {
        Ok(Self {
            config_manager: ConfigManager::default()?,
            settings_manager: SettingsManager::default()?,
        })
    }

    pub fn business_operation(&self, param: &str) -> Result<ReturnType> {
        // 1. Validate input
        // 2. Load data via managers
        // 3. Apply business logic
        // 4. Save via managers
        // 5. Record history
        Ok(result)
    }
}
```

### Testing Requirements

**Always use isolated test environments:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_operation_success() {
        // Arrange: Use tempfile for isolation
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join(".ccs_config.toml");

        // Act: Execute operation
        let result = my_function(&config_path);

        // Assert: Verify expectations
        assert!(result.is_ok());
    }
}
```

**Rules:**
- ALWAYS use `tempfile::TempDir` for file system tests
- NEVER test with real user files (`~/.claude/`, `~/.ccs_config.toml`)
- TEST both success and error paths
- NAME tests: `test_<function>_<scenario>_<expected>`

### Adding New Commands

When adding a new CLI command:

1. Create `src/commands/my_command.rs`
2. Implement command logic using Service layer
3. Add to `src/commands/mod.rs` exports
4. Add command definition in `src/main.rs` CLI parser
5. Add tests in the same file under `#[cfg(test)]`
6. Update documentation

### Adding Web API Endpoints

When adding a new API endpoint:

1. Add request/response models in `src/web/models.rs`
2. Implement handler in `src/web/handlers.rs` using Service layer
3. Add route in `src/web/routes.rs`
4. Test with `ccr web` and manual HTTP requests

## Troubleshooting

### Common Issues

**Build Failures:**
- Ensure Rust 1.85+ is installed: `rustc --version`
- Update toolchain: `rustup update stable`

**Test Failures:**
- Clean and rebuild: `just clean && just test`
- Run single test: `cargo test test_name -- --nocapture`

**Lock Timeout Errors:**
- Check for zombie processes: `ps aux | grep ccr`
- Clean locks manually: `rm -rf ~/.claude/.locks/*`

**Permission Denied:**
- Fix settings.json: `chmod 600 ~/.claude/settings.json`
- Fix config: `chmod 644 ~/.ccs_config.toml`

### Debug Logging

Enable verbose logging:

```bash
export CCR_LOG_LEVEL=debug  # trace|debug|info|warn|error
ccr switch anthropic
```

## CI/CD

The project uses GitHub Actions. Before committing:

```bash
just ci  # Run full CI pipeline locally
```

This runs:
1. `cargo fmt --check` - Format validation
2. `cargo clippy -- -D warnings` - Lint with warnings as errors
3. `cargo test` - All tests
4. `cargo build --release` - Release build

## Performance Notes

- Release builds use `opt-level = 3`, `lto = true`, `strip = true`
- Parallel processing with `rayon` for operations on multiple configs
- Async web server with `axum` + `tokio` for CCR UI backend
- System info caching for frequently accessed data

## Related Documentation

- Main README: `README.md` (English) / `README_CN.md` (Chinese)
- CCR UI Guide: `ccr-ui/README.md`
- Architecture Details: `ccr-ui/ARCHITECTURE.md`
- VitePress Docs: `docs/` (run with `cd docs && npm run dev`)

## Important Reminders

1. **Never bypass the layered architecture** - always use Services → Managers → Core
2. **Always use atomic file operations** via `AtomicFileWriter`
3. **Always use file locking** via `LockManager` for concurrent safety
4. **Always mask sensitive data** (API tokens) in logs and display
5. **Always write tests** for new features using `tempfile::TempDir`
6. **Always run `just ci`** before committing
7. **Read existing code** before modifying - use Read tool to understand context
