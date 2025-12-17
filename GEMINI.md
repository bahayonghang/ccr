# CCR (Claude Code Configuration Switcher) - Project Context

## 1. Project Overview
**CCR** is a high-performance, multi-platform configuration management tool written in **Rust** (Edition 2024). It allows users to manage and switch configurations for AI CLI tools like **Claude Code**, **Codex**, **Gemini CLI**, and **Qwen**.

**Key Features:**
*   **Multi-Platform:** Unified management for Claude, Codex, Gemini, Qwen.
*   **Interfaces:** CLI (primary), TUI (Ratatui), Web API (Axum), and a GUI (Vue 3 + Tauri).
*   **Safety:** Atomic writes, file locking (`fs4`), auto-backups, and sensitive data masking.
*   **Sync:** WebDAV support for multi-folder synchronization.

## 2. Architecture & Directory Structure

### Root Directory
*   `src/`: Core Rust codebase (CLI, TUI, Web API logic).
    *   `commands/`: CLI command implementations (`check`, `sync`, `ui`, etc.).
    *   `services/`: Business logic layer.
    *   `managers/`: Data access/management.
    *   `platforms/`: Platform-specific adapters.
    *   `tui/`: Terminal User Interface (Ratatui).
    *   `web/`: Embedded Web API endpoints.
*   `tests/`: Integration tests.
*   `docs/`: VitePress documentation site.
*   `ccr-ui/`: Full-stack UI application (Workspace Member).
    *   `backend/`: Rust Axum server (separate binary/crate).
    *   `frontend/`: Vue 3 + Vite + TailwindCSS + Tauri.

### Key Files
*   `Cargo.toml`: Workspace configuration.
*   `justfile`: Task runner configuration (Build, Test, Lint, Run).
*   `CLAUDE.md`: Detailed architectural guidelines and command references.
*   `AGENTS.md`: Specific instructions for AI agents.

## 3. Technology Stack

*   **Core Language:** Rust (Edition 2024)
*   **Web Framework:** Axum (Backend API)
*   **TUI Framework:** Ratatui
*   **Frontend:** Vue 3, TypeScript, Vite, TailwindCSS
*   **Desktop:** Tauri (v2)
*   **Package Managers:** `cargo` (Rust), `bun` (Frontend), `uv` (Python scripting if needed)

## 4. Development Workflow

The project uses `just` for task automation. **Always use `just` commands when available.**

### Build & Run
*   `just build`: Build debug version.
*   `just release`: Build release version.
*   `just run -- <args>`: Run the CLI (e.g., `just run -- --help`).
*   `just s`: Start UI development (Frontend + Backend) inside `ccr-ui/`.

### Testing & Quality
*   `just check`: Quick syntax/type check.
*   `just test`: Run all workspace tests (serially).
*   `just lint`: Run `fmt` and `clippy` (warnings as errors).
*   `just ci`: Full CI pipeline (Sync Version -> Lint -> Test -> Release Build -> Audit).
*   `just frontend-check`: Full frontend validation (Typecheck + Lint + Build + Docs).

### Versioning
*   `just version-sync`: Syncs version from root `Cargo.toml` to UI components.

## 5. Coding Conventions & Standards

*   **Rust Style:** Follow `rustfmt` and `clippy` strictly.
*   **Error Handling:**
    *   ❌ **Avoid** `unwrap()` and `expect()` in production code.
    *   ✅ **Use** `Result<T, E>` and propagate errors with `?`.
    *   Exceptions: Tests and top-level `main` initialization.
*   **Locking:** Use `unwrap_or_else` for `RwLock`/`Mutex` poisoning.
*   **Commits:** Follow **Conventional Commits** (e.g., `feat(ui): add settings page`, `fix(sync): resolve webdav conflict`).
*   **Tests:** Aim for >95% coverage. New features *must* have tests.

## 6. Agent Instructions (from AGENTS.md)

*   **Plan First:** For complex tasks, create a `Plan Artifact` before implementing.
*   **Tool Usage:**
    *   Use `codebase_investigator` for exploring unfamiliar code.
    *   Use `write_todos` for complex task tracking.
    *   Use `uv` for any Python operations (Global Rule).
*   **Security:** Never modify `.env` files or commit secrets.
*   **Documentation:** Read `AGENTS.md` and `openspec/AGENTS.md` if planning architecture changes.

## 7. Useful References
*   **Legacy Mode:** `~/.ccs_config.toml` (Single file).
*   **Unified Mode:** `~/.ccr/` (Multi-platform, recommended).
*   **Logs:** `~/.ccr/logs/` (if enabled via `CCR_LOG_LEVEL=debug`).
