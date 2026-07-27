# ccr-sync Backend Spec Index

> WebDAV sync domain and sync folder registry.

## Guidelines Index

| Guide | Description | Status |
|-------|-------------|--------|
| [Backend Guidelines](./backend-guidelines.md) | Sync crate boundaries, WebDAV error mapping, filters, logging, and verification | Complete |
| [Test Fixtures](./test-fixtures.md) | Process-wide CCR and sync env fixtures for tests | Complete |
| [WebDAV Hardening Contracts](./webdav-hardening-contracts.md) | Path, budget, transaction, transport, and sensitive envelope contracts | Complete |

## Pre-Development Checklist

- Read [Backend Guidelines](./backend-guidelines.md) before changing WebDAV transfer, sync folders, content selection, or path expansion.
- Read [Test Fixtures](./test-fixtures.md) before adding tests that mutate `CCR_ROOT`, `CCR_SYNC_FOLDERS_CONFIG`, or `CCR_SYNC_CONFIG_PATH`.
- Read [WebDAV Hardening Contracts](./webdav-hardening-contracts.md) before changing WebDAV href parsing, transfer limits, pull replacement, transport policy, or sensitive asset encryption.
