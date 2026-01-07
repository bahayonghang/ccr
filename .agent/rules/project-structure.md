---
trigger: always_on
glob: "**/*"
description: "Detailed documentation of the project structure, architecture, and key components for CCR."
---

# Project Structure & Architecture Context

## 1. Project Overview
**CCR (Claude Code Configuration Switcher)** is a high-performance, multi-platform tool for managing AI CLI configurations (Claude, Codex, Gemini, Qwen).
- **Core Stack:** Rust (Edition 2024).
- **Interfaces:** CLI (Primary), TUI (Ratatui), Web API (Axum), Desktop GUI (Tauri + Vue 3).

## 2. Directory Map

### Root Level
- **`src/`**: The heart of the Rust application (CLI & Core Logic).
  - `commands/`: CLI command implementations (e.g., `check`, `sync`).
  - `managers/`: Data access layer (handling files, databases, state).
  - `services/`: Business logic orchestration.
  - `platforms/`: Adapters for specific AI tools (implementing common traits).
  - `tui/`: Terminal User Interface logic.
- **`tests/`**: Integration tests for the core logic.
- **`docs/`**: Project documentation (VitePress).

### UI Workspace (`ccr-ui/`)
- **`ccr-ui/backend/`**: Dedicated Rust Axum server for the UI.
- **`ccr-ui/frontend/`**: Modern Web Frontend (Vue 3 + TypeScript + TailwindCSS + Tauri).
  - `src/`: Vue components and logic.
  - `src-tauri/`: Tauri configuration and Rust bindings.

### Configuration & Tooling
- **`justfile`**: **CRITICAL**. The central task runner. Use `just` for building, testing, and running.
- **`Cargo.toml`**: Rust workspace definition.
- **`.agent/`**: AI behavior rules (this folder).
- **`GEMINI.md` / `CLAUDE.md`**: Context and architectural guidelines.

## 3. Key Architectural Concepts
- **Platform Traits:** The system is designed to be extensible. New AI tools are added by implementing standard platform traits in `src/platforms/`.
- **Atomic Persistence:** Configuration changes use atomic file writes and locking (`fs4`) to prevent corruption.
- **Workspace Separation:** The project is a Cargo Workspace. `ccr-ui` is a member, but the root crate drives the main CLI.

## 4. Development Environment
- **OS:** Windows (win32) is the current active environment.
- **Temp Dir:** `C:\Users\lyh\.gemini\tmp\...` (Use for intermediate files).
