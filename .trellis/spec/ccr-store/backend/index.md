# ccr-store Backend Spec Index

> CLI-side history, cost, budget, pricing, and session persistence.

## Guidelines Index

| Guide | Description | Status |
|-------|-------------|--------|
| [Backend Guidelines](./backend-guidelines.md) | Store crate boundaries, SQLite session patterns, logging, tests, and verification | Complete |
| [Recorded Profile Activity](./profile-activity.md) | Pure path resolution, strict batched read-only activation history, availability/errors and fixtures | Complete |

## Pre-Development Checklist

- Read [Backend Guidelines](./backend-guidelines.md) before changing history, budget/cost/pricing managers, session indexing, or CLI-side SQLite storage.
- Read [Recorded Profile Activity](./profile-activity.md) before changing Profile last-use/history display projections or their CLI facade.
