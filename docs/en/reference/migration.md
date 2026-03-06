# Repository Layout Migration Guide

This page maps the legacy repository layout to the current Rust workspace layout.

## Path Mapping

| Legacy path | Current path | Notes |
|---|---|---|
| `src/` | `crates/ccr/src/` | Main CLI, TUI, Web API, and service layers |
| `tests/` | `crates/ccr/tests/` | CLI integration tests |
| `build.rs` | `crates/ccr/build.rs` | Main package build script |
| `ccr-db/` | `crates/ccr-db/` | Database crate |
| `ccr-types/` | `crates/ccr-types/` | Shared types crate |
| `ccr-ui/backend/` | `ccr-ui/src-tauri/` | Tauri desktop shell |
| `ccr-ui/frontend/` | `ccr-ui/src/` | Vue frontend source |
| No unified artifact directory | `outputs/` | Final collected artifacts without changing native build outputs |

## Command Mapping

| Legacy command | Current command |
|---|---|
| `cargo install --path .` | `cargo install --path crates/ccr` |
| `cargo run -- ...` | `cargo run -p ccr -- ...` |
| `cargo build --release` | `cargo build -p ccr --release` |
| `cd ccr-ui/backend && cargo build --release` | `cd ccr-ui/src-tauri && cargo build --release` |
| `cd ccr-ui/frontend && bun run build` | `cd ccr-ui && bun run build` |

## Build Outputs

- Native CLI binaries still build into `target/`.
- Frontend static assets still build into `ccr-ui/dist/`.
- Tauri desktop artifacts still build into `ccr-ui/src-tauri/target/`.
- Use root `just outputs-collect` to copy final deliverables into `outputs/`.

## See Also

- [Quick Start](/en/guide/quick-start)
- [Architecture](/en/reference/architecture)
- [Command Reference](/en/reference/commands/)
