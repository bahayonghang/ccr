# CCR - Claude Code Configuration Switcher (Gemini Context)

## Project Overview
**CCR** is a high-performance, multi-platform configuration management tool written in **Rust**. It allows users to manage and switch configurations for various AI CLI tools (Claude, Codex, Gemini, Qwen) seamlessly.

The project is a **monorepo** consisting of:
- **Core CLI**: The main Rust application (`src/`) providing CLI commands, TUI, and logic.
- **CCR UI**: A full-stack application (`ccr-ui/`) with a Rust backend (`axum`) and a Vue 3 frontend, capable of running as a web server or a desktop app (via **Tauri**).

## Tech Stack
- **Core Language**: Rust (Edition 2024)
- **Frontend**: Vue 3, TypeScript, Vite, Pinia, Tailwind CSS
- **Backend (UI)**: Rust (Axum, Tokio)
- **Desktop Wrapper**: Tauri 2.0
- **TUI Framework**: Ratatui
- **Build System**: Cargo, Just, NPM

## Key Directories
- `src/`: Core CLI source code.
  - `commands/`: CLI command implementations.
  - `managers/`: Data management logic.
  - `platforms/`: Platform-specific logic (Claude, Codex, etc.).
- `ccr-ui/`: UI sub-project.
  - `backend/`: Axum server source.
  - `frontend/`: Vue 3 application source.
  - `src-tauri/`: Tauri configuration and rust source.
- `tests/`: Integration tests.
- `scripts/`: Utility scripts (version sync, git hooks).

## Development Workflow

### Build & Run
The project uses `just` as a command runner.

| Task | Command | Description |
|------|---------|-------------|
| **Build (Debug)** | `just build` | Builds the CLI in debug mode. |
| **Build (Release)** | `just release` | Builds with optimizations (`target/release/ccr`). |
| **Run CLI** | `just run -- <args>` | Runs the CLI (e.g., `just run -- --help`). |
| **Run UI (Dev)** | `cd ccr-ui && just s` | Starts Frontend (5173) and Backend (8081). |
| **Run TUI** | `cargo run --features tui -- tui` | Runs the Terminal UI. |

### Testing & Quality
| Task | Command | Description |
|------|---------|-------------|
| **Test** | `just test` | Runs unit and integration tests. |
| **Lint** | `just lint` | Runs `cargo fmt` and `cargo clippy`. |
| **CI** | `just ci` | Full check: fmt, clippy, test, release build. |

**Note:** Platform integration tests (`platform_integration_tests.rs`) must run serially (`--test-threads=1`). The `just test` command handles this automatically.

### Architecture Highlights
- **Atomic Writes**: Configuration files are updated using atomic writes and file locking (`fs4`) to prevent data corruption.
- **Multi-Platform**: Supports unified config storage (`~/.ccr/`) or legacy single-file modes.
- **WebDAV Sync**: Built-in synchronization logic for config sharing across devices.

## Important Conventions
- **Rust Style**: Strict adherence to `rustfmt` and `clippy` rules.
- **Error Handling**: Use `anyhow` and `thiserror` for structured error management.
- **Logging**: `env_logger` is used. Control verbosity via `CCR_LOG_LEVEL` (trace, debug, info, warn, error).
- **Dependency Management**: Use `cargo add/remove`. `Cargo.lock` should be committed.

## User Context (Session)
- **Operating System**: linux
- **Shell**: bash
- **Working Directory**: `/home/lyh/Documents/Github/ccr`
