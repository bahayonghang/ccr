# ccr-codex Backend Spec Index

> Dedicated Codex and OpenCode domain crate.

## Guidelines Index

| Guide | Description | Status |
|-------|-------------|--------|
| [Backend Guidelines](./backend-guidelines.md) | Codex/OpenCode domain boundaries, auth safety, errors, logs, tests, and verification | Complete |
| [Codex Session Recovery](./codex-session-recovery.md) | sync-history visibility repair and recoverable session trash contracts | Complete |
| [Test Fixtures](./test-fixtures.md) | Process-wide Codex and CCR env fixtures for tests | Complete |

## Pre-Development Checklist

- Read [Backend Guidelines](./backend-guidelines.md) before changing Codex/OpenCode auth, runtime, quota, session, usage, or history services.
- Read [Codex Session Recovery](./codex-session-recovery.md) before changing `CodexHistorySyncService`, `CodexSessionTrashService`, or `ccr codex sessions` / `ccr codex sync-history` behavior.
- Read [Test Fixtures](./test-fixtures.md) before adding tests that resolve Codex home, OpenCode home, or CCR env paths.
