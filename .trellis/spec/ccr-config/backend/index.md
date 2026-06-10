# ccr-config Backend Spec Index

> Platform/profile configuration contracts and registry helpers.

## Guidelines Index

| Guide | Description | Status |
|-------|-------------|--------|
| [Backend Guidelines](./backend-guidelines.md) | Config crate boundaries, TOML persistence, errors, logs, and verification | Complete |
| [Test Fixtures](./test-fixtures.md) | Process-wide CCR env fixtures for config tests | Complete |

## Pre-Development Checklist

- Read [Backend Guidelines](./backend-guidelines.md) before changing `Platform`, `PlatformPaths`, profile TOML helpers, config managers, or config services.
- Read [Test Fixtures](./test-fixtures.md) before adding tests that mutate `CCR_ROOT`, `CCR_LOCK_DIR`, or home-directory config resolution.
