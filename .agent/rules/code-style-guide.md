---
trigger: always_on
glob: "**/*.{rs,ts,js,vue,toml}"
description: "Core code style principles and language-specific standards (Rust/Vue) following Antigravity best practices."
---

# Code Style & Standards

## 1. Core Principles (The Antigravity Way)
- **KISS (Keep It Simple, Stupid):** Prefer simple, readable solutions over complex abstractions. Code should be self-explanatory.
- **DRY (Don't Repeat Yourself):** Abstract repeated logic into functions or components.
- **YAGNI (You Aren't Gonna Need It):** Do not implement features or abstractions for hypothetical future use cases.
- **SOLID:** Adhere to SOLID principles, especially Single Responsibility.

## 2. Rust Standards
- **Formatting:** Strictly follow `rustfmt` and `clippy`.
- **Error Handling:**
  - ❌ Avoid `unwrap()` and `expect()` in production/library code.
  - ✅ Use `Result<T, E>` and propagate errors with `?`.
  - ✅ Use `unwrap_or_else` or `match` for graceful degradation.
- **Concurrency:** Use `tokio` idioms. Prefer message passing (`mpsc`) over shared state (`Mutex`) where possible ("Do not communicate by sharing memory; instead, share memory by communicating").
- **Testing:** Unit tests must be co-located in the same file (bottom module).

## 3. Frontend Standards (Vue 3 + TypeScript)
- **Composition API:** Always use `<script setup lang="ts">`.
- **Styling:**
  - ✅ Use **TailwindCSS** utility classes first.
  - ❌ Avoid raw CSS/SCSS unless for complex animations or legacy overrides.
- **Components:**
  - Naming: `PascalCase` (e.g., `SettingsPanel.vue`).
  - Structure: `template` -> `script` -> `style`.
- **State Management:** Use `pinia` for global state; `ref`/`reactive` for local state.

## 4. Comments & Documentation
- **Why vs. What:** Comments should explain *why* a decision was made, not *what* the code does.
- **Public API:** All public structs/functions in Rust must have documentation comments (`///`).