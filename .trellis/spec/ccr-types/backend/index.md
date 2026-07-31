# ccr-types Backend Spec Index

> Shared serializable contracts for the CCR ecosystem.

## Guidelines Index

| Guide | Description | Status |
|-------|-------------|--------|
| [Backend Guidelines](./backend-guidelines.md) | Shared type-contract boundaries, serde compatibility, tests, and verification | Complete |
| [Claude Auth Runtime Diagnosis](../../ccr-cli/backend/claude-auth-runtime.md) | Authoritative cross-layer source ordering and confidence contract for shared Claude Auth DTOs | Complete |

## Pre-Development Checklist

- Read [Backend Guidelines](./backend-guidelines.md) before changing shared serialized structs, enums, serde defaults, aliases, or public type re-exports.
- Read [Claude Auth Runtime Diagnosis](../../ccr-cli/backend/claude-auth-runtime.md) before changing the shared Claude auth diagnosis/action DTO shape or enum values.
