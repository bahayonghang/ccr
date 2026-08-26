# Dashboard Presentation Contracts

> Executable contracts for `ccr-ui/src/views/dashboard/dashboardPresentation.ts` and the `DashboardView.tsx` children it feeds (`DashboardNextActions`, `DashboardUsageMovement`, `DashboardSignalStream`, `DashboardPlatformMatrix`) under `ccr-ui/src/features/usage/dashboard/`. Readiness copy, pill, and reason checklist render in `DashboardView` itself (`08-25-home-runtime-layout` removed `DashboardReadinessLedger`).

---

## Scenario: Signal severity gating (core vs. frontend-log noise)

### 1. Scope / Trigger

- Trigger: changing `countSignals`, `buildReadiness`, or `buildActions` in `dashboardPresentation.ts`, or adding a new aggregate health/alert indicator anywhere on the Dashboard that's driven by `MonitoringEntry[]`.
- Introduced by `07-07-ui-shell-home` to fix a screenshot-confirmed bug: a single frontend retry-log error (e.g. `logger.error('Failed to save Claude profile:', ...)` in `ClaudeCodeProfilesView`) was simultaneously flipping the readiness card to "attention", turning the signals tile red, and injecting an "open monitoring" action — three amplifications of one piece of noise.

### 2. Signatures

- `isCoreSignal(entry: MonitoringEntry): boolean` — `entry.channel` is not in `{frontend, runtime}` (`dashboardPresentation.ts`).
- `countSignals(logs: MonitoringEntry[]): DashboardSignalCounts` — filters through `isCoreSignal` before computing `errors`/`warnings`/`total`.
- `channel: 'frontend'` comes from `normalizeLoggerEntry`. `channel: 'runtime'` comes from the Tauri tracing bridge. Neither drives readiness. Domain events stay `checkin`, `usage`, `environment`, `sync`, `task`, `app`, `system`.

### 3. Contracts

- `signalCounts` (and therefore the readiness "attention" branch, the signals status tile's tone, and the `open-monitoring` action) must only be driven by non-diagnostic channels.
- `DashboardSignalStream` must keep rendering **all** entries including `frontend` and `runtime`.
- Genuine backend/checkin/sync-channel errors still drive all three surfaces.

### 4. Validation & Error Matrix

- New frontend `logger.error(...)` or bridged `runtime` warn/error -> event stream only; must not flip readiness/tile/action.
- A new core channel from a domain backend event is counted unless added to `DIAGNOSTIC_CHANNELS`.

### 5. Good/Base/Bad Cases

- Good: a `logger.error()` call in a component only ever reaches the dashboard through the signal stream list, never through `signalCounts`.
- Bad: adding a second, separate frontend-error counter that bypasses `countSignals` for a "new" indicator — re-introduces the triple-amplification bug through a side door.

### 6. Tests Required

- `ccr-ui/tests/dashboard/dashboard-presentation.smoke.test.ts` — extend the existing `logs`-based test case (`createLog` helper) if adding a new channel or counting path; the current suite's `createLog` defaults to `channel: 'usage'` (a core channel) specifically so the frontend-exclusion logic isn't accidentally exercised by unrelated tests.

### 7. Wrong vs Correct

#### Wrong

```ts
const countSignals = (logs: MonitoringEntry[]): DashboardSignalCounts => {
  const errors = logs.filter((e) => e.level === 'error').length // counts frontend noise too
  ...
}
```

#### Correct

```ts
const isCoreSignal = (entry: MonitoringEntry) => entry.channel !== 'frontend'
const countSignals = (logs: MonitoringEntry[]): DashboardSignalCounts => {
  const coreLogs = logs.filter(isCoreSignal)
  const errors = coreLogs.filter((e) => e.level === 'error').length
  ...
}
```

---

## Scenario: `DashboardReadiness.reasons` shape (checklist rows, not sentence pills)

### 1. Scope / Trigger

- Trigger: adding, removing, or reordering a reason in `buildReadiness()`, or changing how `DashboardView.tsx` renders the header reason checklist.

### 2. Signatures

- `DashboardReadinessReason = { key: string; ok: boolean }` (`dashboardPresentation.ts`).
- `DashboardReadiness.reasons: DashboardReadinessReason[]` (renamed from the pre-`07-07-ui-shell-home` `reasonKeys: string[]`).
- Consumed by `DashboardView.tsx` header checklist: `reason.ok` picks `SIcon` name (`Check` vs `AlertTriangle`) and the icon's color class; `stripTrailingPeriod()` strips a trailing `。`/`.` from the translated string so rows read as a checklist, not sentences.

### 3. Contracts

- Every new reason pushed into the `reasons` array in `buildReadiness()` must set `ok` to the actual boolean outcome it represents (not always `false`, not a placeholder) — the ledger's icon/color is meaningless otherwise.
- Locale strings for `dashboard.readiness.reasons.*` may keep their trailing period (existing zh-CN/en-US strings do); do not strip it in the locale file — `stripTrailingPeriod()` in the component handles display, keeping the string reusable elsewhere as a full sentence if needed later.

### 4. Validation & Error Matrix

- Adding a reason without a paired `ok` boolean -> TypeScript error (`DashboardReadinessReason` requires both fields) — this is intentionally not optional.
- Renaming/removing a reason key -> update `ccr-ui/tests/dashboard/dashboard-presentation.smoke.test.ts`'s assertion (`presentation.readiness.reasons.map(r => r.key)).toContain(...)`).

### 5. Good/Base/Bad Cases

- Good: `{ key: 'dashboard.readiness.reasons.usageReady', ok: true }` for the success branch, `{ key: '...usageError', ok: false }` for the failure branch of the same conceptual check.
- Bad: `{ key: reasonKey, ok: false }` hardcoded regardless of which branch produced `reasonKey` — silently shows an alert icon on a "things are fine" reason.

---

## Scenario: First-run / empty-state detection needs a real usage signal, not just CLI count

### 1. Scope / Trigger

- Trigger: any "is this a fresh install / has the user configured anything yet" check on the Dashboard (currently `DashboardPresentation.isFirstRun`, consumed by `DashboardNextActions.tsx`'s `showOnboarding` prop).

### 2. Signatures

- `isFirstRun: input.isNativeRuntime && input.cliVersionsLoaded && !input.usageLoading && installedCliCount === 0 && (!input.overview || input.overview.summary.total_requests === 0)`.

### 3. Contracts

- `installedCliCount` only counts `isRuntimeCli: true` platforms (`claude-code`, `codex`, `antigravity` per `DashboardView.tsx`'s `platforms` list) — `opencode` is `mode: 'managed', isRuntimeCli: false` and is **never** counted, regardless of how actively it's used. Do not use `installedCliCount === 0` alone as a "nothing configured" signal; a managed-only (OpenCode) user will always read as 0.
- Pair any CLI-install-based "empty" check with a usage-based fallback (`overview.summary.total_requests === 0` or equivalent) so a user who has real activity through a managed platform isn't permanently misidentified as first-run.
- Gate on both `cliVersionsLoaded` and `!usageLoading` before evaluating — otherwise the flag can flip `true` for one tick while usage is still in flight (even for a returning user with history), then flip back once the overview loads.
- There is no dedicated "profile count" signal in `DashboardPresentationInput` today. If a future task adds one (e.g. via a new IPC call), prefer it over this heuristic and update this contract.

### 4. Validation & Error Matrix

- New managed-mode platform added to `DashboardView.tsx`'s `platforms` array -> re-check whether `isFirstRun`'s usage-fallback still covers it (it will, as long as that platform's activity flows into `overview.summary.total_requests`).
- `isFirstRun` used to gate anything more disruptive than a card's onboarding content (e.g. a modal or redirect) -> reconsider; this is a soft heuristic, not a guaranteed "zero configuration" proof.

### 5. Good/Base/Bad Cases

- Good: OpenCode-only user with real usage history -> `overview.summary.total_requests > 0` -> `isFirstRun` false -> normal action queue shown.
- Bad: gating first-run purely on `installedCliCount === 0` -> permanently true for OpenCode-only users no matter how long they've used the app.

---

## Scenario: Compact card empty/onboarding states should not import `EmptyState`

### 1. Scope / Trigger

- Trigger: adding an empty-state or onboarding affordance inside a Dashboard grid card (roughly 4–8 grid columns wide, sharing row height with a sibling card via `align-items: stretch`).

### 2. Contracts

- `ccr-ui/src/ui/empty-state.tsx` has `min-h-[300px]` and full-page/section-level padding (`p-12`) — designed for a whole view's empty state, not a card slot that shares height with a sibling action/readiness card.
- For a card-local empty/onboarding state, replicate `EmptyState`'s visual language inline (icon circle, title, description, optional numbered steps) sized to the card's existing padding/gap tokens (`--home-card-pad`, `--home-text-*`), rather than importing the component. See `DashboardNextActions.tsx`'s `ONBOARDING_STEPS` / `dashboard-actions__onboarding` block for the pattern.

### 3. Good/Base/Bad Cases

- Good: `DashboardNextActions.tsx` renders its 3-step onboarding list inline, reusing `.dashboard-action`-adjacent styling at the card's own scale.
- Bad: `{showOnboarding ? <EmptyState .../> : null}` inside a `dashboard-grid__actions` slot — forces the card (and, via `align-items: stretch`, its sibling) to at least 300px+ regardless of the grid's actual space budget.

---

## Scenario: Status-metric `tone` drives the StatTile shell only

### 1. Scope / Trigger

- Trigger: changing `src/ui/stat-tile.tsx` badge rendering, wiring `tone` in `DashboardUsageMovement.tsx`, or changing `buildStatusMetrics()` tone assignment. Home no longer consumes `statusMetrics` (`08-25-home-runtime-layout`); `buildDashboardPresentation` still produces the array.
- Introduced by `08-18-overview-home-visual`: `DashboardStatusMetric.tone` was already computed, but the ledger dropped it and usage summary tiles stayed bare.

### 2. Signatures

- `DashboardStatusMetric.tone: DashboardTone` (`neutral | success | warning | danger | accent`) — assigned in `buildStatusMetrics()`.
- `StatTile` optional `tone?: 'neutral' | 'success' | 'warning' | 'danger' | 'accent'` — the union lives on the primitive. Do not import `DashboardTone` into `StatTile`.
- Home no longer wires `statusMetrics` into StatTile. Usage summary tiles: `tone="neutral"`.

### 3. Contracts

- `tone` drives only the value's square badge shell (`.stat-tile__value--badge` + `data-tone`): 10% fill, 18% border, optional 6px tone dot. Digits stay `--color-text-primary` + `tabular-nums`.
- Omit `tone` → bare tile (label + value + hint), no shell, no `data-tone`. Other StatTile call sites stay bare unless they already pass `tone`.
- Do not change `countSignals`, `buildReadiness`, `isFirstRun`, or `buildStatusMetrics()` tone assignment on this visual path.
- Do not wrap StatTile in `ui-card`. Do not paint digits with semantic or accent ink. Do not use a solid accent fill on the number or the whole tile.

### 4. Tests Required

- `ccr-ui/tests/ui/ui-primitives.smoke.test.tsx` — bare tile without `tone`; `tone: 'success'` has `data-tone`, the badge class, no `ui-card`, and source still contains `tabular-nums`.
- `ccr-ui/tests/dashboard/dashboard-presentation.smoke.test.ts` — existing judgment expectations stay green.
- `ccr-ui/tests/shell/react-shell.smoke.test.tsx` — root route mounts `DashboardView` (`.dashboard-view`).

---

## Scenario: Platform sparkline and trackingHealth must not treat all-zero series as untracked

### 1. Scope / Trigger

- Trigger: adding fields to `DashboardPlatformRow`, changing `buildPlatformRows()`, or changing how `DashboardPlatformMatrix` decides the untracked placeholder.

### 2. Signatures

- `DashboardPlatformRow.sparkline?: number[]` — per-day `requests` from `overview.series`, mapped by `usageKey` (`gemini` → `antigravity`).
- `DashboardPlatformRow.trackingHealth?: 'live' | 'degraded' | 'missing'` — from `overview.archive.source_health`, matching `source === usageKey` or the canonical source id (`gemini` also matches `antigravity`).

### 3. Contracts

- `overview == null` or empty `series` → omit `sparkline` (leave `undefined`). Do not invent a zero array.
- Backend home series pads every homepage platform to the selected day count, including untracked ones. **All-zero `sparkline` is not an untracked signal.**
- Untracked placeholder is `trackingHealth === 'missing'` only. `degraded` still shows data. Empty `source_health` leaves `trackingHealth` undefined and must not show the placeholder.
- When `trackingHealth === 'missing'`, do not emit `sparkline` and do not surface `0` as requests/tokens.

### 4. Tests Required

- `ccr-ui/tests/dashboard/dashboard-presentation.smoke.test.ts` — date-order sparkline; `gemini` → `antigravity`; `overview == null` / empty `series` → `undefined`; all-zero series with empty `source_health` is not missing.
- `ccr-ui/tests/dashboard/dashboard-platform-matrix.smoke.test.tsx` — `state: 'missing'` shows the placeholder; all-zero series does not.
