# Dashboard Presentation Contracts

> Executable contracts for `ccr-ui/src/views/dashboard/dashboardPresentation.ts` and the five `DashboardView.vue` child components it feeds (`DashboardReadinessLedger`, `DashboardNextActions`, `DashboardUsageMovement`, `DashboardSignalStream`, `DashboardPlatformMatrix`).

---

## Scenario: Signal severity gating (core vs. frontend-log noise)

### 1. Scope / Trigger

- Trigger: changing `countSignals`, `buildReadiness`, or `buildActions` in `dashboardPresentation.ts`, or adding a new aggregate health/alert indicator anywhere on the Dashboard that's driven by `MonitoringEntry[]`.
- Introduced by `07-07-ui-shell-home` to fix a screenshot-confirmed bug: a single frontend retry-log error (e.g. `logger.error('Failed to save Claude profile:', ...)` in `ClaudeCodeProfilesView.vue`) was simultaneously flipping the readiness card to "attention", turning the signals tile red, and injecting an "open monitoring" action — three amplifications of one piece of noise.

### 2. Signatures

- `isCoreSignal(entry: MonitoringEntry): boolean` — `entry.channel` is not in `{frontend, runtime}` (`dashboardPresentation.ts`).
- `countSignals(logs: MonitoringEntry[]): DashboardSignalCounts` — filters through `isCoreSignal` before computing `errors`/`warnings`/`total`.
- `channel: 'frontend'` comes from `normalizeLoggerEntry`. `channel: 'runtime'` comes from the Tauri tracing bridge. Neither drives readiness. Domain events stay `checkin`, `usage`, `environment`, `sync`, `task`, `app`, `system`.

### 3. Contracts

- `signalCounts` (and therefore the readiness "attention" branch, the signals status tile's tone, and the `open-monitoring` action) must only be driven by non-diagnostic channels.
- `DashboardSignalStream.vue` must keep rendering **all** entries including `frontend` and `runtime`.
- Genuine backend/checkin/sync-channel errors still drive all three surfaces.

### 4. Validation & Error Matrix

- New frontend `logger.error(...)` or bridged `runtime` warn/error -> event stream only; must not flip readiness/tile/action.
- A new core channel from a domain backend event is counted unless added to `DIAGNOSTIC_CHANNELS`.

### 5. Good/Base/Bad Cases

- Good: a `logger.error()` call in a component only ever reaches the dashboard through the signal stream list, never through `signalCounts`.
- Bad: adding a second, separate frontend-error counter that bypasses `countSignals` for a "new" indicator — re-introduces the triple-amplification bug through a side door.

### 6. Tests Required

- `ccr-ui/tests/dashboard-presentation.smoke.test.ts` — extend the existing `logs`-based test case (`createLog` helper) if adding a new channel or counting path; the current suite's `createLog` defaults to `channel: 'usage'` (a core channel) specifically so the frontend-exclusion logic isn't accidentally exercised by unrelated tests.

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

- Trigger: adding, removing, or reordering a reason in `buildReadiness()`, or changing how `DashboardReadinessLedger.vue` renders the reason list.

### 2. Signatures

- `DashboardReadinessReason = { key: string; ok: boolean }` (`dashboardPresentation.ts`).
- `DashboardReadiness.reasons: DashboardReadinessReason[]` (renamed from the pre-`07-07-ui-shell-home` `reasonKeys: string[]`).
- Consumed by `DashboardReadinessLedger.vue`: `reason.ok` picks `SIcon` name (`Check` vs `AlertTriangle`) and the icon's color class; `stripTrailingPeriod()` strips a trailing `。`/`.` from the translated string so rows read as a checklist, not sentences.

### 3. Contracts

- Every new reason pushed into the `reasons` array in `buildReadiness()` must set `ok` to the actual boolean outcome it represents (not always `false`, not a placeholder) — the ledger's icon/color is meaningless otherwise.
- Locale strings for `dashboard.readiness.reasons.*` may keep their trailing period (existing zh-CN/en-US strings do); do not strip it in the locale file — `stripTrailingPeriod()` in the component handles display, keeping the string reusable elsewhere as a full sentence if needed later.

### 4. Validation & Error Matrix

- Adding a reason without a paired `ok` boolean -> TypeScript error (`DashboardReadinessReason` requires both fields) — this is intentionally not optional.
- Renaming/removing a reason key -> update `ccr-ui/tests/dashboard-presentation.smoke.test.ts`'s assertion (`presentation.readiness.reasons.map(r => r.key)).toContain(...)`).

### 5. Good/Base/Bad Cases

- Good: `{ key: 'dashboard.readiness.reasons.usageReady', ok: true }` for the success branch, `{ key: '...usageError', ok: false }` for the failure branch of the same conceptual check.
- Bad: `{ key: reasonKey, ok: false }` hardcoded regardless of which branch produced `reasonKey` — silently shows an alert icon on a "things are fine" reason.

---

## Scenario: First-run / empty-state detection needs a real usage signal, not just CLI count

### 1. Scope / Trigger

- Trigger: any "is this a fresh install / has the user configured anything yet" check on the Dashboard (currently `DashboardPresentation.isFirstRun`, consumed by `DashboardNextActions.vue`'s `showOnboarding` prop).

### 2. Signatures

- `isFirstRun: input.isNativeRuntime && input.cliVersionsLoaded && !input.usageLoading && installedCliCount === 0 && (!input.overview || input.overview.summary.total_requests === 0)`.

### 3. Contracts

- `installedCliCount` only counts `isRuntimeCli: true` platforms (`claude-code`, `codex`, `antigravity` per `DashboardView.vue`'s `platforms` computed) — `opencode` is `mode: 'managed', isRuntimeCli: false` and is **never** counted, regardless of how actively it's used. Do not use `installedCliCount === 0` alone as a "nothing configured" signal; a managed-only (OpenCode) user will always read as 0.
- Pair any CLI-install-based "empty" check with a usage-based fallback (`overview.summary.total_requests === 0` or equivalent) so a user who has real activity through a managed platform isn't permanently misidentified as first-run.
- Gate on both `cliVersionsLoaded` and `!usageLoading` before evaluating — otherwise the flag can flip `true` for one tick while usage is still in flight (even for a returning user with history), then flip back once the overview loads.
- There is no dedicated "profile count" signal in `DashboardPresentationInput` today. If a future task adds one (e.g. via a new IPC call), prefer it over this heuristic and update this contract.

### 4. Validation & Error Matrix

- New managed-mode platform added to `DashboardView.vue`'s `platforms` array -> re-check whether `isFirstRun`'s usage-fallback still covers it (it will, as long as that platform's activity flows into `overview.summary.total_requests`).
- `isFirstRun` used to gate anything more disruptive than a card's onboarding content (e.g. a modal or redirect) -> reconsider; this is a soft heuristic, not a guaranteed "zero configuration" proof.

### 5. Good/Base/Bad Cases

- Good: OpenCode-only user with real usage history -> `overview.summary.total_requests > 0` -> `isFirstRun` false -> normal action queue shown.
- Bad: gating first-run purely on `installedCliCount === 0` -> permanently true for OpenCode-only users no matter how long they've used the app.

---

## Scenario: Compact card empty/onboarding states should not import `EmptyState.vue`

### 1. Scope / Trigger

- Trigger: adding an empty-state or onboarding affordance inside a Dashboard grid card (roughly 4–8 grid columns wide, sharing row height with a sibling card via `align-items: stretch`).

### 2. Contracts

- `ccr-ui/src/components/ui/EmptyState.vue` has `min-h-[300px]` and full-page/section-level padding (`p-12`) — designed for a whole view's empty state, not a card slot that shares height with a sibling action/readiness card.
- For a card-scoped empty/onboarding state, replicate `EmptyState.vue`'s visual language inline (icon circle, title, description, optional numbered steps) sized to the card's existing padding/gap tokens (`--home-card-pad`, `--home-text-*`), rather than importing the component. See `DashboardNextActions.vue`'s `dashboard-actions__onboarding` block for the pattern.

### 3. Good/Base/Bad Cases

- Good: `DashboardNextActions.vue` renders its 3-step onboarding list inline, reusing `.dashboard-action`-adjacent styling at the card's own scale.
- Bad: `<EmptyState v-if="showOnboarding" .../>` inside a `dashboard-grid__actions` slot — forces the card (and, via `align-items: stretch`, its sibling) to at least 300px+ regardless of the grid's actual space budget.
