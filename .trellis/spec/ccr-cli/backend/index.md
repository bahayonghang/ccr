# ccr-cli Backend Spec Index

> CLI/application domain crate.

## Guidelines Index

| Guide | Description | Status |
|-------|-------------|--------|
| [Backend Guidelines](./backend-guidelines.md) | CLI command boundaries, output/logging rules, errors, tests, and verification | Complete |
| [Test Fixtures](./test-fixtures.md) | Process-wide env and filesystem fixtures for CLI tests | Complete |
| [Profile Initialization](./profile-init.md) | Claude/Codex/Grok profile scaffolding, templates, guarded creation, and registry registration | Complete |
| [Grok Profile Runtime](./grok-profile-runtime.md) | Grok profile validation, runtime switching, restoration, CAS, and secret boundaries | Complete |

## Pre-Development Checklist

- Read [Backend Guidelines](./backend-guidelines.md) before changing command definitions, command handlers, CLI services, CLI managers, or command output.
- Read [Test Fixtures](./test-fixtures.md) before adding tests that mutate process env or home-directory paths.
- Read [Profile Initialization](./profile-init.md) before changing profile init commands, embedded examples, or platform registry bootstrap.
- Read [Grok Profile Runtime](./grok-profile-runtime.md) before changing Grok profile validation, runtime switching, restoration, deletion, or credential display.
