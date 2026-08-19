# ccr-core Backend Spec Index

> Shared infrastructure primitives used across CCR crates.

## Guidelines Index

| Guide                                         | Description                                                                       | Status   |
| --------------------------------------------- | --------------------------------------------------------------------------------- | -------- |
| [Backend Guidelines](./backend-guidelines.md) | Shared infrastructure boundaries, errors, logging, env fixtures, and verification | Complete |
| [Logging Contracts](./logging-contracts.md) | Daily file names, write-boundary redaction, bridge queue | Complete |
| [CcrError Freeze](./ccr-error-freeze.md)      | ADR: CcrError frozen at 25 variants; new domain errors live in owning crates      | Complete |
| [Atomic Writer](./atomic-writer.md)           | Crash-safe file replacement and guarded write (lock/backup/fsync/0o600) contracts | Complete |
| [Test Fixtures](./test-fixtures.md)           | ccr-core process environment test fixtures                                        | Complete |
| [Managed Process Tree](./managed-process.md)  | Cross-platform process-tree ownership, termination, and reap contracts             | Complete |

## Pre-Development Checklist

- Read [Backend Guidelines](./backend-guidelines.md) before changing shared primitives, error types, logging setup, the `Secret` credential type, masking helpers, or file I/O helpers. Credential fields must be `Secret`, never bare `String`; `rg 'expose_plaintext'` lists every plaintext-on-disk field.
- Read [Logging Contracts](./logging-contracts.md) before changing `init_logger`, log file names, `log_redact`, or the bridge queue.
- Read [CcrError Freeze](./ccr-error-freeze.md) before touching `CcrError` variants or choosing an error type for a new module — the enum is frozen; new domain errors belong in the owning crate.
- Read [Atomic Writer](./atomic-writer.md) before changing atomic file writes, Windows replacement behavior, guarded write (lock/backup/secret) policy, or config/auth persistence helpers.
- Read [Test Fixtures](./test-fixtures.md) before adding tests that mutate `CCR_LOG_LEVEL`, `RUST_LOG`, or process env.
- Read [Managed Process Tree](./managed-process.md) before changing child spawn, process-group/Job Object setup, cancellation, timeout escalation, or reap behavior.
