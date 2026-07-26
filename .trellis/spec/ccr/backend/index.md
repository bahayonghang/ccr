# ccr Backend Spec Index

> Root binary/library facade for the Rust workspace.

## Guidelines Index

| Guide                                                                | Description                                                                                 | Status   |
| -------------------------------------------------------------------- | ------------------------------------------------------------------------------------------- | -------- |
| [Backend Guidelines](./backend-guidelines.md)                        | Root CLI/library facade boundaries, logging, errors, and verification                       | Complete |
| [Desktop Command Policy](./desktop-command-policy.md)                | Desktop request validation, ProcessGateway capabilities, output bounds, and lifecycle        | Complete |
| [Dependency Governance](./dependency-governance.md)                  | Root/Tauri dependency drift gates                                                           | Complete |
| [llmusage Provider Adapter Contract](./llmusage-provider-adapter.md) | Provider-scoped llmusage sync/read-only SQLite/Tauri dashboard contract                     | Complete |
| [Public API Boundary](./public-api-boundary.md)                      | Stable prelude and root re-export compatibility guards                                      | Complete |
| [SSH Trust and Transport](./ssh-security.md)                        | Validated OpenSSH arguments, app-owned trust, real handshakes, and SFTP-only file operations | Complete |
| [Tauri Handler Registry](./tauri-handler-registry.md)                | Domain command registry for the desktop invoke handler                                      | Complete |
| [Test Fixtures](./test-fixtures.md)                                  | Root `ccr` integration test environment fixtures                                            | Complete |
| [Typed IPC Bindings](./typed-ipc-bindings.md)                        | ts-rs generated TS bindings, committed artifacts, and drift guard for typed command domains | Complete |

## Pre-Development Checklist

- Read [Backend Guidelines](./backend-guidelines.md) before changing `crates/ccr/src/main.rs`, `crates/ccr/src/lib.rs`, root re-exports, features, or binary dispatch.
- Read [Public API Boundary](./public-api-boundary.md) before changing public re-exports or `prelude`.
- Read [Test Fixtures](./test-fixtures.md) before adding root integration tests.
- Read [Dependency Governance](./dependency-governance.md) before changing workspace/Tauri dependency versions.
- Read [Desktop Command Policy](./desktop-command-policy.md) and [Tauri Handler Registry](./tauri-handler-registry.md) before changing desktop command passthrough or handler registration.
- Read [SSH Trust and Transport](./ssh-security.md) before changing SSH host persistence, trust confirmation, connection state, OpenSSH arguments, or remote config file operations.
- Read [llmusage Provider Adapter Contract](./llmusage-provider-adapter.md) before changing provider-scoped usage sync, adapter filters, or dashboard payloads.
- Read [Typed IPC Bindings](./typed-ipc-bindings.md) before changing usage V2 / claude_observer wire DTOs, generated TypeScript under `ccr-ui/src/types/generated/`, or typing a new command domain.
