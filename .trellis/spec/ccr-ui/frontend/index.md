# ccr-ui Frontend Development Guidelines

> Frontend contracts for the CCR desktop UI.

---

## Guidelines Index

| Guide                                                                     | Description                                                                                            | Status   |
| ------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------ | -------- |
| [API Facade Boundary](./api-facade-boundary.md)                           | Domain-first API wrappers and legacy Tauri facade guardrails                                           | Complete |
| [Check-in UX Concurrency Contracts](./checkin-ux-contracts.md)            | Balance refresh per-origin queue/throttle, event-based job waiting, 4-state display, toast-only errors | Complete |
| [Provider Template Contracts](./provider-template-contracts.md)           | Non-secret global provider templates, platform overrides, and saved-provider separation                | Complete |
| [Theme Token Contracts](./theme-token-contracts.md)                       | Theme/flavor/accent token layering and visual verification guardrails                                  | Complete |
| [Dashboard Presentation Contracts](./dashboard-presentation-contracts.md) | Signal severity gating, readiness reason shape, first-run heuristic, compact-card empty states         | Complete |
| [Usage Chart Stability Contracts](./usage-chart-stability-contracts.md)   | ApexCharts options/series reference discipline, redraw freeze flags, KeepAlive interplay               | Complete |

## Pre-Development Checklist

- Read [API Facade Boundary](./api-facade-boundary.md) before adding or changing frontend API wrappers.
- Read [Check-in UX Concurrency Contracts](./checkin-ux-contracts.md) before changing check-in batch refresh, job waiting, result display, or error surfacing.
- Read [Provider Template Contracts](./provider-template-contracts.md) before adding or changing Claude Code, Codex, or OpenCode provider template flows.
- Read [Theme Token Contracts](./theme-token-contracts.md) before changing `ccr-ui/src/styles/tokens.css`, flavor overrides, or theme smoke contracts.
- Read [Dashboard Presentation Contracts](./dashboard-presentation-contracts.md) before changing `dashboardPresentation.ts` or its five Dashboard child components.
- Read [Usage Chart Stability Contracts](./usage-chart-stability-contracts.md) before adding or changing any ApexCharts usage chart, its options/series wiring, or the usage tab KeepAlive structure.

## Quality Check

- Run the focused smoke guard when touching `src/api/tauri.ts` or `src/api/domains/*`:
  - `cd ccr-ui && bun run test:smoke -- tests/api-facade-boundary.smoke.test.ts`
- Run `cd ccr-ui && bun run type-check` and `cd ccr-ui && bun run lint` for frontend API changes.
- Run `cd ccr-ui && bun run test:smoke -- tests/provider-templates.smoke.test.ts` when changing provider template data, custom template persistence, selectors, or platform mappers.
- Run the targeted theme smoke checks when changing theme/flavor/accent tokens:
  - `cd ccr-ui && bunx vitest run --config vitest.smoke.config.ts tests/apple-glass-surface-contract.smoke.test.ts tests/theme-bootstrap.smoke.test.ts tests/app-settings.smoke.test.ts`
