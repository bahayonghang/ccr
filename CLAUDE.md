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

# Test (use --test-threads=1 to avoid concurrent conflicts)
cargo test --workspace --all-features -- --test-threads=1

# Single test file
cargo test --test manager_tests

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

### Frontend (ccr-ui/frontend)

```bash
cd ccr-ui/frontend
npm install                    # Install dependencies
npm run dev                    # Dev server (localhost:3000)
npm run build                  # Production build
npm run type-check             # TypeScript check
npm run lint                   # ESLint check
```

### UI Backend (ccr-ui/backend)

```bash
cd ccr-ui/backend
cargo run                      # Start backend (127.0.0.1:8081)
```

## Architecture

### Workspace Structure

```
ccr/
├── src/                    # Core CLI crate (ccr binary + library)
├── ccr-types/              # Shared type definitions crate
├── ccr-ui/                 # Full-stack Web/Desktop App
│   ├── backend/            # Axum REST API server
│   └── frontend/           # Vue 3 + Vite + Tauri
├── tests/                  # Integration tests
├── docs/                   # VitePress documentation
└── justfile                # Task runner
```

### Core CLI Architecture (src/)

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
- Edition 2024 (requires Rust 1.88+)
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
- `src/CLAUDE.md` - Core CLI module details
- `ccr-ui/CLAUDE.md` - UI module details
- `AGENTS.md` - OpenSpec instructions for proposals/specs

## Testing

Integration tests in `/tests/`:
- `manager_tests.rs` - Manager layer tests
- `service_workflow_tests.rs` - Service layer tests
- `concurrent_tests.rs` - Concurrency & locking tests
- `end_to_end_tests.rs` - Full workflow tests

Run specific test: `cargo test --test <file_stem>`
Run by name: `cargo test <keyword>`
