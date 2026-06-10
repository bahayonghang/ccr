# ccr Backend Spec Index

> Root binary/library facade for the Rust workspace.

## Guidelines Index

| Guide | Description | Status |
|-------|-------------|--------|
| [Backend Guidelines](./backend-guidelines.md) | Root CLI/library facade boundaries, logging, errors, and verification | Complete |
| [Desktop Command Policy](./desktop-command-policy.md) | Request-level validation for desktop command passthrough | Complete |
| [Dependency Governance](./dependency-governance.md) | Root/Tauri dependency drift gates | Complete |
| [Public API Boundary](./public-api-boundary.md) | Stable prelude and root re-export compatibility guards | Complete |
| [Tauri Handler Registry](./tauri-handler-registry.md) | Domain command registry for the desktop invoke handler | Complete |
| [Test Fixtures](./test-fixtures.md) | Root `ccr` integration test environment fixtures | Complete |

## Pre-Development Checklist

- Read [Backend Guidelines](./backend-guidelines.md) before changing `crates/ccr/src/main.rs`, `crates/ccr/src/lib.rs`, root re-exports, features, or binary dispatch.
- Read [Public API Boundary](./public-api-boundary.md) before changing public re-exports or `prelude`.
- Read [Test Fixtures](./test-fixtures.md) before adding root integration tests.
- Read [Dependency Governance](./dependency-governance.md) before changing workspace/Tauri dependency versions.
- Read [Desktop Command Policy](./desktop-command-policy.md) and [Tauri Handler Registry](./tauri-handler-registry.md) before changing desktop command passthrough or handler registration.
