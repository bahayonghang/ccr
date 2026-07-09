# ccr-cli Backend Spec Index

> CLI/application domain crate.

## Guidelines Index

| Guide | Description | Status |
|-------|-------------|--------|
| [Backend Guidelines](./backend-guidelines.md) | CLI command boundaries, output/logging rules, errors, tests, and verification | Complete |
| [Test Fixtures](./test-fixtures.md) | Process-wide env and filesystem fixtures for CLI tests | Complete |

## Pre-Development Checklist

- Read [Backend Guidelines](./backend-guidelines.md) before changing command definitions, command handlers, CLI services, CLI managers, or command output.
- Read [Test Fixtures](./test-fixtures.md) before adding tests that mutate process env or home-directory paths.
