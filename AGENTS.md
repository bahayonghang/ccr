# AGENTS.md

This file provides guidance to neovate when working with code in this repository.

<!-- OPENSPEC:START -->
# OpenSpec Instructions

These instructions are for AI assistants working in this project.

Always open `@/openspec/AGENTS.md` when the request:
- Mentions planning or proposals (words like proposal, spec, change, plan)
- Introduces new capabilities, breaking changes, architecture shifts, or big performance/security work
- Sounds ambiguous and you need the authoritative spec before coding

Use `@/openspec/AGENTS.md` to learn:
- How to create and apply change proposals
- Spec format and conventions
- Project structure and guidelines

Keep this managed block so 'openspec update' can refresh the instructions.

<!-- OPENSPEC:END -->

## Repository Guidelines

## Project Structure & Module Organization
CCR is a Cargo workspace: CLI and shared library code live in `src/`, integration suites in `tests/`, and shared manifests in the root `Cargo.toml`. Reference docs stay in `docs/`, runnable walkthroughs in `examples/`, and protocol specs under `openspec/`. `ccr-ui/backend/` hosts the Axum API server, `ccr-ui/frontend/` contains the Vue 3 + TypeScript client, and automation scripts sit in `scripts/`.

## Development Commands

### Core Build/Compilation Commands
- `cargo build --workspace` or `just build` - Compiles workspace crates in debug mode
- `just release` - Performs optimized release build
- `cargo build --no-default-features` - CLI only build (saves ~75% build time)
- `cargo build --features web` - CLI + Web API build
- `cargo build --features tui` - CLI + TUI build
- `cargo build --all-features` - Complete build with all features

### Testing Commands
- `just test` - Runs test suite in proper order (serial platform tests first)
- `cargo test --test platform_integration_tests -- --test-threads=1` - Platform integration tests (must run serially)
- `cargo test --lib` - Unit tests only
- `cargo test --workspace` - Test all workspace members
- `cargo test --all-features` - Full test suite
- `just test-all` - Complete test suite including ignored tests

### Linting/Formatting Commands
- `just lint` - Run format and clippy checks
- `cargo fmt` - Format code
- `cargo fmt --all --check` - Check formatting without modifying
- `cargo clippy --all-targets -- -D warnings` - Lint with warnings as errors
- `just clippy` - Run clippy in CCR UI backend
- `just ci` - Complete CI pipeline (format + clippy + test + build)

### Development Server Commands
- `just dev` - Development check and test cycle
- `just watch` - Monitor file changes and auto-rebuild
- `just run -- <args>` - Run debug version with arguments
- `just run-release -- <args>` - Run release version with arguments
- `cd ccr-ui && just s` - Start CCR UI dev environment (Vue frontend + Rust backend)
- `just quick-start` - One command setup (check prereqs + install + start)

### Package Management Commands
- `just install` - Install to local (~/.cargo/bin)
- `just reinstall` - Force reinstall
- `just uninstall` - Remove installed binary
- `just update-deps` - Check dependency updates
- `just clean` - Clean build artifacts
- `cd ccr-ui && just install` - Install all UI dependencies

### Database/Migration Commands
- `ccr migrate` - Migrate from legacy to unified mode
- `ccr migrate --check` - Check if migration needed
- `ccr migrate --platform <name>` - Migrate specific platform

### Deployment/Release Commands
- `just release` - Build optimized release binary
- `just ci` - Complete CI build pipeline
- `just prepare-release` - Prepare for release (clean + build)
- `cargo install --git https://github.com/bahayonghang/ccr ccr` - Install from GitHub

## Code Architecture & Patterns

### Architecture Overview
CCR follows a strict **layered architecture**:
```
CLI/Web Layer → Services → Managers → Core/Utils
```

**Cargo Workspace Architecture**:
- Workspace Root: Centralized dependency management with `[workspace.dependencies]`
- CCR Main Crate: CLI tool + reusable library
- CCR-UI Backend: Axum web server using CCR as a library
- Shared Dependencies: 15+ core libraries (serde, tokio, axum, chrono, etc.)

### Key Components
- **Service Layer**: 4 services (Config, Settings, History, Backup) with 26+ methods
- **Manager Layer**: 3 managers (Config, Settings, History) for data access & file operations
- **Web Module**: Axum-based server with 14+ RESTful API endpoints
- **Core Infrastructure**: Atomic writer, file locking, error handling, logging

### Configuration Modes
CCR supports two configuration modes:
- **Legacy Mode**: Single platform using `~/.ccs_config.toml` (backward compatible with CCS)
- **Unified Mode**: Multi-platform using `~/.ccr/` directory structure (default)

### Platform Abstraction
- **Platform Trait**: Defines platform interface for Claude, Codex, Gemini, etc.
- **Factory Pattern**: `create_platform()` function creates platform instances
- **Platform Registry**: Manages all available platforms with detection logic

### Data Flow Patterns
- **Atomic Operations**: Temp file + rename pattern for safe file writes
- **File Locking**: Prevents corruption during concurrent operations
- **Audit Trail**: Every operation logged with UUID, timestamp, masked data
- **Auto Backup**: Automatic backups before destructive operations

### Build Performance Optimizations
- **Development Profile**: `opt-level = 1` with `debug = 1` for faster builds
- **Dependency Optimization**: `[profile.dev.package."*"] opt-level = 2` for dependencies
- **Incremental Compilation**: Enabled by default

## Technology Stack & Dependencies

### Core Dependencies
- **Rust Edition 2024**: Modern Rust features and syntax
- **Tokio**: Async runtime with multi-threaded features
- **Axum**: Web framework for API endpoints
- **Serde**: Serialization with derive features
- **Clap**: CLI argument parsing with derive
- **Comfy-table**: Beautiful terminal table UI

### Web/Backend Stack
- **Axum**: High-performance async web framework
- **Tower**: HTTP service framework with middleware
- **Tokio**: Async runtime for concurrent operations
- **Tracing**: Structured logging and diagnostics

### Frontend Stack (CCR UI)
- **Vue.js 3.5**: Modern JavaScript framework with Composition API
- **TypeScript**: Type-safe development
- **Tailwind CSS**: Utility-first styling framework
- **Vite**: Fast development build tool
- **Pinia**: State management
- **Axios**: HTTP client

### Testing & Quality
- **Cargo Test**: Built-in test framework
- **Clippy**: Rust linter with strict warning policies
- **Cargo Fmt**: Code formatting with rustfmt
- **Tarpaulin**: Test coverage (target: 95%+)

### Platform Support
- **Multi-Platform**: Claude Code, Codex (GitHub Copilot), Gemini CLI, Qwen, iFlow
- **Cross-OS**: Linux, macOS, Windows support via conditional builds
- **WebDAV Sync**: Cloud sync with Nutstore, Nextcloud, ownCloud support

### Key Design Patterns
- **Service Layer Pattern**: Business logic encapsulation
- **Manager Pattern**: Data access abstraction
- **Factory Pattern**: Platform instance creation
- **Builder Pattern**: Configuration construction
- **Singleton Pattern**: Global managers and state
- **Async/Await**: Non-blocking I/O operations
- **Option/Result**: Error handling and nullable values
