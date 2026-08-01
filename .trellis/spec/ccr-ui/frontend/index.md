# ccr-ui Frontend Development Guidelines

> Frontend contracts for the CCR desktop UI.

---

## Guidelines Index

| Guide                                                                     | Description                                                                                                    | Status   |
| ------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------- | -------- |
| [API Facade Boundary](./api-facade-boundary.md)                           | Domain-first API wrappers and legacy Tauri facade guardrails                                                   | Complete |
| [Check-in UX Concurrency Contracts](./checkin-ux-contracts.md)            | Balance refresh per-origin queue/throttle, event-based job waiting, 4-state display, toast-only errors         | Complete |
| [Provider Template Contracts](./provider-template-contracts.md)           | Non-secret global provider templates, platform overrides, and saved-provider separation                        | Complete |
| [Theme Token Contracts](./theme-token-contracts.md)                       | Theme/flavor/accent/font token layering, font-preference fallback override, and visual verification guardrails | Complete |
| [Dashboard Presentation Contracts](./dashboard-presentation-contracts.md) | Signal severity gating, readiness reason shape, first-run heuristic, compact-card empty states                 | Complete |
| [Usage Chart Stability Contracts](./usage-chart-stability-contracts.md)   | ApexCharts options/series reference discipline, redraw freeze flags, KeepAlive interplay                       | Complete |
| [Confirm Interaction Contracts](./confirm-interaction-contracts.md)       | requestConfirm gate pattern, danger/warning semantics, no native dialogs, composable boundary                  | Complete |
| [Raw Config Editor Contracts](./raw-config-editor-contracts.md)           | Local-only plaintext source editing, validation, versioned saves, and shared editor behavior                   | Complete |
| [Brand Asset Pipeline Contract](./brand-asset-pipeline.md)                | Brand SVG ownership, Cairo/Pillow rendering, generated outputs, and cross-surface verification                 | Complete |
| [Sync Security Contracts](./sync-security-contracts.md)                   | Typed sync IPC, truth table, canonical WebDAV config, and operation passphrase lifecycle                       | Complete |
| [Development Resource Contracts](./development-resource-contracts.md)     | Vite watcher scope, warmup ownership, process cleanup, cache preservation, and smoke worker budgets            | Complete |
| [Profiles Page Contracts](./profiles-page-contracts.md)                   | Codex form-derived auth fields and `env_key` serialization, two-page shared skeleton, `--cp-*`/`pe-*` boundary | Complete |
| [Environment-Scoped Dashboard Contracts](./environment-scoped-dashboard-contracts.md) | Local-only environment gates, cache invalidation, and stale refresh behavior                                  | Complete |

## Pre-Development Checklist

- Read [API Facade Boundary](./api-facade-boundary.md) before adding or changing frontend API wrappers.
- Read [Check-in UX Concurrency Contracts](./checkin-ux-contracts.md) before changing check-in batch refresh, job waiting, result display, or error surfacing.
- Read [Provider Template Contracts](./provider-template-contracts.md) before adding or changing Claude Code, Codex, or OpenCode provider template flows.
- Read [Theme Token Contracts](./theme-token-contracts.md) before changing `ccr-ui/src/styles/tokens.css`, flavor overrides, font tracks / `fontPreferences.ts` / font-preference overrides, or theme smoke contracts.
- Read [Dashboard Presentation Contracts](./dashboard-presentation-contracts.md) before changing `dashboardPresentation.ts` or its five Dashboard child components.
- Read [Usage Chart Stability Contracts](./usage-chart-stability-contracts.md) before adding or changing any ApexCharts usage chart, its options/series wiring, or the usage tab KeepAlive structure.
- Read [Confirm Interaction Contracts](./confirm-interaction-contracts.md) before adding any confirmation dialog, destructive-action flow, or user-facing alert/toast.
- Read [Raw Config Editor Contracts](./raw-config-editor-contracts.md) before adding raw config, prompt, or profile source editing.
- Read [Brand Asset Pipeline Contract](./brand-asset-pipeline.md) before changing `branding/`, `generate_icons.py`, or generated UI/Tauri/docs/VS Code brand assets.
- Read [Sync Security Contracts](./sync-security-contracts.md) before changing fixed sync assets, sync IPC payloads, WebDAV configuration ownership, or the sensitive passphrase flow.
- Read [Development Resource Contracts](./development-resource-contracts.md) before changing Vite/Vitest development startup, warmup, watcher, cache, or process-lifecycle tooling.
- Read [Profiles Page Contracts](./profiles-page-contracts.md) before changing either Profiles page, the shared `components/profiles/*` family, a profile card / editor modal, or `utils/{claude,codex}Profile*.ts`.
- Read [Environment-Scoped Dashboard Contracts](./environment-scoped-dashboard-contracts.md) before adding or changing a Local-only dashboard with environment-scoped caches or CLI version detection.

## Quality Check

- Run the focused smoke guard when touching `src/api/tauri.ts` or `src/api/domains/*`:
  - `cd ccr-ui && bun run test:smoke -- tests/api-facade-boundary.smoke.test.ts`
- Run `cd ccr-ui && bun run type-check` and `cd ccr-ui && bun run lint` for frontend API changes.
- Run the backend and frontend focused checks from [Raw Config Editor Contracts](./raw-config-editor-contracts.md) when changing source editors or raw-file commands.
- Run `cd ccr-ui && bun run test:smoke -- tests/provider-templates.smoke.test.ts` when changing provider template data, custom template persistence, selectors, or platform mappers.
- Run the targeted theme smoke checks when changing theme/flavor/accent tokens:
  - `cd ccr-ui && bunx vitest run --config vitest.smoke.config.ts tests/apple-glass-surface-contract.smoke.test.ts tests/theme-bootstrap.smoke.test.ts tests/app-settings.smoke.test.ts tests/theme-contrast-contract.smoke.test.ts`
- Additionally run the font-preference guard and i18n compile when changing font tracks, `fontPreferences.ts`, or the font controls/copy:
  - `cd ccr-ui && bunx vitest run --config vitest.smoke.config.ts tests/font-preferences.smoke.test.ts` and `cd ccr-ui && bun run test:i18n`
- Run `cd ccr-ui && bun run icons:generate && bun run icons:ensure && bun run build` for brand-source or renderer changes, then verify deterministic hashes and generated image formats per the brand asset contract.
- Run the focused Tauri sync tests plus frontend type-check, lint, and smoke tests from [Sync Security Contracts](./sync-security-contracts.md) for sync contract changes.
- Run `cd ccr-ui && bunx vitest run --config vitest.smoke.config.ts tests/dev-tooling-resource.smoke.test.ts` for development resource tooling changes.
- Run the focused Profiles smoke set from [Profiles Page Contracts](./profiles-page-contracts.md) when changing Profiles pages, shared profile components, or profile form serialization.
- Run the focused dashboard smoke tests from [Environment-Scoped Dashboard Contracts](./environment-scoped-dashboard-contracts.md) when changing Local-only dashboard refresh ordering or cache behavior.
