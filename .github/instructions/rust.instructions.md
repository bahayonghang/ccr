---
applyTo: "crates/**/*.rs,ccr-ui/src-tauri/**/*.rs"
description: "Rust workspace conventions for CCR crates and the Tauri backend"
---

# Rust Instructions

- Use `Result`-based error handling and avoid `unwrap` or `expect` in production code.
- Keep internal implementation comments in Chinese and public API docs in English.
- Reuse existing services, managers, and models before introducing new abstractions.
- Preserve masking, backup, file-lock, and atomic-write behavior when touching config flows.
- Add or update regression tests when behavior changes.
- Verify with the narrowest relevant `cargo` or `just` command before finishing.
