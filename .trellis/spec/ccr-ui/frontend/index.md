# ccr-ui Frontend Development Guidelines

> Frontend contracts for the CCR desktop UI.

---

## Guidelines Index

| Guide | Description | Status |
|-------|-------------|--------|
| [API Facade Boundary](./api-facade-boundary.md) | Domain-first API wrappers and legacy Tauri facade guardrails | Complete |
| [Provider Template Contracts](./provider-template-contracts.md) | Non-secret global provider templates, platform overrides, and saved-provider separation | Complete |
| [Theme Token Contracts](./theme-token-contracts.md) | Theme/flavor/accent token layering and visual verification guardrails | Complete |

## Pre-Development Checklist

- Read [API Facade Boundary](./api-facade-boundary.md) before adding or changing frontend API wrappers.
- Read [Provider Template Contracts](./provider-template-contracts.md) before adding or changing Claude Code, Codex, or OpenCode provider template flows.
- Read [Theme Token Contracts](./theme-token-contracts.md) before changing `ccr-ui/src/styles/tokens.css`, flavor overrides, or theme smoke contracts.

## Quality Check

- Run the focused smoke guard when touching `src/api/tauri.ts` or `src/api/domains/*`:
  - `cd ccr-ui && bun run test:smoke -- tests/api-facade-boundary.smoke.test.ts`
- Run `cd ccr-ui && bun run type-check` and `cd ccr-ui && bun run lint` for frontend API changes.
- Run `cd ccr-ui && bun run test:smoke -- tests/provider-templates.smoke.test.ts` when changing provider template data, custom template persistence, selectors, or platform mappers.
- Run the targeted theme smoke checks when changing theme/flavor/accent tokens:
  - `cd ccr-ui && bunx vitest run --config vitest.smoke.config.ts tests/apple-glass-surface-contract.smoke.test.ts tests/theme-bootstrap.smoke.test.ts tests/app-settings.smoke.test.ts`
