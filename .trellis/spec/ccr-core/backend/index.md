# ccr-core Backend Spec Index

> Shared infrastructure primitives used across CCR crates.

## Guidelines Index

| Guide | Description | Status |
|-------|-------------|--------|
| [Backend Guidelines](./backend-guidelines.md) | Shared infrastructure boundaries, errors, logging, env fixtures, and verification | Complete |
| [Atomic Writer](./atomic-writer.md) | Crash-safe file replacement contracts | Complete |
| [Test Fixtures](./test-fixtures.md) | ccr-core process environment test fixtures | Complete |

## Pre-Development Checklist

- Read [Backend Guidelines](./backend-guidelines.md) before changing shared primitives, error types, logging setup, masking helpers, or file I/O helpers.
- Read [Atomic Writer](./atomic-writer.md) before changing atomic file writes, Windows replacement behavior, or config/auth persistence helpers.
- Read [Test Fixtures](./test-fixtures.md) before adding tests that mutate `CCR_LOG_LEVEL`, `RUST_LOG`, or process env.
