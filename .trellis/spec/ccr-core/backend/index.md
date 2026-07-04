# ccr-core Backend Spec Index

> Shared infrastructure primitives used across CCR crates.

## Guidelines Index

| Guide                                         | Description                                                                       | Status   |
| --------------------------------------------- | --------------------------------------------------------------------------------- | -------- |
| [Backend Guidelines](./backend-guidelines.md) | Shared infrastructure boundaries, errors, logging, env fixtures, and verification | Complete |
| [Atomic Writer](./atomic-writer.md)           | Crash-safe file replacement and guarded write (lock/backup/fsync/0o600) contracts | Complete |
| [Test Fixtures](./test-fixtures.md)           | ccr-core process environment test fixtures                                        | Complete |

## Pre-Development Checklist

- Read [Backend Guidelines](./backend-guidelines.md) before changing shared primitives, error types, logging setup, the `Secret` credential type, masking helpers, or file I/O helpers. Credential fields must be `Secret`, never bare `String`; `rg 'expose_plaintext'` lists every plaintext-on-disk field.
- Read [Atomic Writer](./atomic-writer.md) before changing atomic file writes, Windows replacement behavior, guarded write (lock/backup/secret) policy, or config/auth persistence helpers.
- Read [Test Fixtures](./test-fixtures.md) before adding tests that mutate `CCR_LOG_LEVEL`, `RUST_LOG`, or process env.
