# AG03 observation surface audit

2026-10-26. Scope: approved airy-glass CSS migration only. AG01 owns tokens/shell/theme and AG02 owns shared primitives, editors and operation-resource pages. Native hook pointed to AG01, so the explicit AG03 dispatch artifacts and manifest were loaded directly.

| Routes | Display owners | Existing state/behavior evidence |
|---|---|---|
| `/` and lower Insights | `features/usage/dashboard`, `features/usage/styles/dashboard-*`, `styles/components/home.css` | dashboard presentation/matrix/movement/signals/Insights smoke; unknown `—`, neutral comparison, bounded bars and exact table retained |
| `/claude-code`, `/codex`, `/grok`, `/antigravity`, `/opencode` | platform view classes plus shared `PlatformUsageInsightPanel`, trend and rank CSS | platform/usage smoke; local-only/unavailable/loading/error roles retained |
| `/usage` | `features/usage/components`, `features/usage/styles/usage-*` | usage tab/controller/ledger tests; visited tabs stay mounted and options/series untouched |
| `/budget`, `/pricing` | `features/usage/budget`, `pricing` and their CSS | existing frontend checks; pricing destructive and feedback colors use semantic roles |
| `/monitoring` | `features/monitoring` | monitoring sanitization and shell bridge tests; actual 500-log scroll before workload captured by AG01 |
| `/agent-sessions`, `/codex/sessions` | `features/agent-sessions`, `features/codex/CodexSessionsView.tsx` | local-only eight-family list/transcript smoke; estimates 132/148 and overscan 8/6 preserved |
| `/commands/:client?` | `features/commands/styles/commands-view.css` | lifecycle/reducer smoke; start/cancel/reconcile/listener/history logic untouched |
| `/checkin`, `/checkin/manage/:accountId` | `features/checkin/styles` | queue/event waiting/progress/account/cookie smoke; success/already/failed/skipped and balance/unknown states retained |
| `/tray/codex` | `features/tray/styles/tray.css` | compact solid root and shell retained; native popup/DPI/focus remain separate acceptance |

## Required corrections beyond inherited tokens

- Existing neutral-alpha fills on reading/field/sticky surfaces cannot inherit opacity from the newly solid token values: `rgb(var(--color-bg-*-rgb) / N%)` explicitly reintroduces transparency. Replace these declarations with the same existing solid role. Do not repaint charts or change row geometry.
- `home.css` still defines card/card-hover as 98% and sunk as 56%/48%; make those existing local aliases solid.
- Platform KPI cost panel adds a radial glow over a translucent fill, contrary to the new solid reading-surface contract. Remove the extra gradient and the redundant shared `backdrop-filter` declaration. Preserve layout and per-platform identity.
- Pricing feedback contains raw warm success/error RGB values and `.dark` overrides. Resolve these through existing status/tint roles so the actual theme/flavor, rather than an unrelated class, controls them.
- Existing native inputs and selects in command/pricing/session filters use weak alpha boundaries; use the existing interactive boundary/focus role without changing dimensions or event wiring.

Tinted state chips, hover washes on an already solid reading surface, skeleton/series opacity, separators and chart encodings are not additional glass layers; keep their functional meaning. A static scan alone is not runtime visual evidence.

## Acceptance limits

Web previews cannot prove real native IPC, system window/DPI/tray behavior, or populated Usage/Session backend states. AG04 must record those independently. Primary/secondary/muted token floors do not establish every pre-existing semantic success/info text consumer meets 4.5:1; measure those actual rendered states separately.
