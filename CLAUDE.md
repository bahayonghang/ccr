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

## Design Context

### Users

Independent developers who manage multiple AI CLI tools (Claude Code, Codex, Gemini, Qwen, iFlow, Droid, OpenCode) in their daily workflow. They switch configurations frequently, need quick access to status and settings, and value both efficiency and a pleasant visual experience. The tool runs as a Tauri v2 desktop application on their development machine.

### Brand Personality

**Cute + Professional + Refined** — A cat-girl (Nekomata) themed interface that is playful on the surface but deeply serious about engineering quality. The brand conveys warmth and approachability without sacrificing precision or trust. It should feel like a capable companion, not a toy.

### Aesthetic Direction

- **Visual Tone**: Kawaii meets Apple Liquid Glass — soft glassmorphism with cat-themed decorative accents (ears, paws, tails, bells). Lush gradients (pink-to-purple) with generous use of blur, glow, and translucency.
- **Primary Reference**: Apple Liquid Glass design language (semi-transparent surfaces, frosted blur, light refraction, layered depth).
- **Anti-References**: Enterprise dashboards with flat gray palettes; overly gamified UIs with excessive particle effects; brutalist or stark minimalist designs that lack warmth.
- **Theme**: Dual-mode — Kawaii Light (sakura pink `#FFF5F7` base) and Deep Purple Night (`#1A0A20` base). Both themes share the same pink-purple accent palette but adjust intensity and contrast accordingly.
- **Typography**: MapleBright (locally hosted woff2 subsets) as the primary typeface for both sans-serif and monospace contexts, with weights compressed to 400/500 for a clean, uniform aesthetic.
- **Iconography**: Lucide Vue Next for UI icons; custom SVG cat-themed decorations (ears, paws, whiskers) for brand personality elements.

### Color System

| Role | Light Mode | Dark Mode |
|------|-----------|-----------|
| **Accent Primary (Neko Pink)** | `#F472B6` | `#F9A8D4` |
| **Accent Secondary (Lavender)** | `#A78BFA` | `#C4B5FD` |
| **Background Base** | `#FFF5F7` | `#1A0A20` |
| **Text Primary** | `#2D1B30` | `#FDF2F8` |
| **Success** | `#34D399` | `#6EE7B7` |
| **Warning** | `#FBBF24` | `#FCD34D` |
| **Danger** | `#FB7185` | `#FDA4AF` |
| **Info** | `#C084FC` | `#D8B4FE` |

Platform-specific colors: Claude `#FF6B35`, Codex `#10B981`, Gemini `#4285F4`, Qwen `#00B5E2`, iFlow `#FAAD14`.

### Design Principles

1. **Kawaii with Substance** — Every decorative element (cat ears, paw prints, glow effects) must serve a UX purpose: guiding attention, indicating state, or providing delight during routine tasks. Never add decoration that obscures function.

2. **Glass-First Surfaces** — Prefer glassmorphism (`backdrop-filter: blur`) over opaque backgrounds for layered UI elements. Use the established token hierarchy (`glass-surface` → `glass-effect` → `glass-elevated` → `glass-modal` → `liquid-glass`) to communicate depth and importance.

3. **Token-Driven Consistency** — All colors, spacing, radii, shadows, and animations must reference CSS custom properties from `tokens.css` / `theme.css`. Never hardcode values in components. Tailwind classes should bridge to these tokens via `tailwind.config.ts`.

4. **Graceful Motion** — Animations should feel organic and spring-like (`ease-spring`, `ease-out-back`). Respect `prefers-reduced-motion` by disabling backdrop-filter and simplifying transitions. Neko-themed animations (breathe, ear-wiggle, paw-bounce) are reserved for personality moments, not core interactions.

5. **Dual-Theme Parity** — Every UI element must look intentional in both light and dark themes. Dark mode is not an afterthought — it uses brighter accent variants and stronger glow effects to maintain visual richness against deep purple backgrounds.
