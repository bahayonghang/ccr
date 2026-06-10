# ccr-db Backend Spec Index

> Desktop/check-in/usage SQLite storage and data services.

## Guidelines Index

| Guide | Description | Status |
|-------|-------------|--------|
| [Backend Guidelines](./backend-guidelines.md) | SQLite pools, migrations, repository boundaries, errors, logs, tests, and verification | Complete |
| [Test Fixtures](./test-fixtures.md) | ccr-db test environment fixtures | Complete |

## Pre-Development Checklist

- Read [Backend Guidelines](./backend-guidelines.md) before changing database pools, schema, migrations, repositories, usage import, or monitoring persistence.
- Read [Test Fixtures](./test-fixtures.md) before adding tests that initialize global database state or mutate OpenCode/CCR env paths.
