# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

CCR (Claude Code Configuration Switcher) is a high-performance, multi-platform configuration management tool written in Rust. It provides unified management for AI CLI tools including Claude Code, Codex, Gemini, Qwen, and iFlow.

## Build & Development Commands

### Rust (Workspace)

```bash
# Build
cargo build                    # Debug build
cargo build --release          # Release build (LTO enabled)

# Install CLI from a local checkout
cargo install --path crates/ccr

# Test (use --test-threads=1 to avoid concurrent conflicts)
cargo test --workspace --all-features -- --test-threads=1

# Single test file
cargo test -p ccr --test managers

# Lint (CI standard)
cargo fmt --all                # Format code
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Strict lint (CI enforced - warns on unwrap usage)
cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::unwrap_used
```

### Just Task Runner (Recommended)

```bash
just build          # Debug build
just release        # Release build
just test           # Run all tests
just lint           # Format + Clippy
just lint-strict    # Strict Clippy (no unwrap)
just ci             # Full CI pipeline
just install        # Install to ~/.cargo/bin
```

### Frontend (ccr-ui)

```bash
cd ccr-ui
npm install                    # Install dependencies
npm run dev                    # Dev server (localhost:3000)
npm run build                  # Production build
npm run type-check             # TypeScript check
npm run lint                   # ESLint check
```

### Desktop Shell (ccr-ui/src-tauri)

```bash
cd ccr-ui
npm run tauri:dev              # Start the full desktop development flow
npm run tauri:check            # Check the Tauri/Rust shell
```

## Architecture

### Workspace Structure

```
ccr/
├── Cargo.toml              # Workspace manifest + shared dependencies
├── crates/
│   ├── ccr/                # Core CLI crate (ccr binary + library)
│   │   ├── src/            # CLI, services, managers, sync, web, tui
│   │   └── tests/          # Integration tests for the CLI crate
│   ├── ccr-db/             # Database-facing services and models
│   └── ccr-types/          # Shared type definitions crate
├── ccr-ui/                 # Full-stack Web/Desktop App
│   ├── src/                # Vue 3 application
│   └── src-tauri/          # Tauri desktop shell
├── docs/                   # VitePress documentation
├── scripts/                # Repo automation helpers
├── examples/               # Sample configurations
├── outputs/                # Collected/generated artifacts (when present)
└── justfile                # Task runner
```

### Core CLI Architecture (`crates/ccr/src/`)

Layered architecture with strict separation of concerns:

```
CLI/Web Layer (commands/, web/, tui/)
    ↓
Service Layer (services/) - Business logic orchestration
    ↓
Manager Layer (managers/) - Data access & persistence
    ↓
Core Layer (core/) - Error handling, file locking, atomic writes
```

**Key Design Principles:**
- Atomic file operations (tempfile + rename via `atomic_writer.rs`)
- File locking prevents concurrent corruption (`fs4` crate)
- Full audit trail (UUID, timestamp, operator)
- Auto-backup before destructive operations

### Feature Flags

```toml
[features]
default = ["web", "tui"]
tui = ["dep:crossterm", "dep:ratatui"]      # Terminal UI
web = ["dep:axum", "dep:tower", ...]        # Web API server
```

## Code Style

### Rust
- Edition 2024 (the installable CLI crate currently requires Rust 1.90+)
- Format: `cargo fmt` (default rustfmt)
- Naming: `snake_case` modules/functions, `PascalCase` types
- Error handling: Custom `CcrError` type with `thiserror`
- Comments: Chinese for internal logic, English for public API docs
- Tests: Use `#[allow(clippy::unwrap_used)]` in test modules

### TypeScript/Vue (Frontend)
- 2-space indentation
- Format: Prettier (`.prettierrc`)
- Lint: ESLint
- Components: `<script setup>` Composition API
- Styling: Tailwind CSS

## Commit Convention

Conventional Commits with optional emoji prefixes:

```bash
feat(core): add platform command
fix(backend): fix config parsing error
refactor(ui): restructure component hierarchy
docs: update installation guide
chore: update dependencies
```

## Module Documentation

Detailed module-specific guidance is available in:
- `crates/ccr/src/CLAUDE.md` - Core CLI module details
- `ccr-ui/CLAUDE.md` - UI module details
- `AGENTS.md` - OpenSpec instructions for proposals/specs

## Testing

Integration tests in `crates/ccr/tests/`:
- `commands.rs` - Command behavior coverage
- `managers.rs` - Manager layer tests
- `platforms.rs` - Platform integration coverage
- `workflows.rs` - End-to-end workflow checks

Run specific test: `cargo test -p ccr --test <file_stem>`
Run by name: `cargo test <keyword>`
