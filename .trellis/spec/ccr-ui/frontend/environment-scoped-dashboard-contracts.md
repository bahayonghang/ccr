# Environment-Scoped Dashboard Contracts

> Keep Local-only dashboard data bound to the active execution environment while preserving useful stale data for ordinary refresh failures.

## Scenario: Local-only dashboard refresh and cache invalidation

### 1. Scope / Trigger

- Trigger: adding a dashboard or composable that combines environment-scoped Tauri data with CLI version detection and module-level TTL caches.
- Applies to `useGrokDashboard` and future Local-only platform dashboards.
- The contract prevents cached local data from appearing in a remote environment and prevents version results from being combined with an unsupported overview.

### 2. Signatures

```ts
refresh(force?: boolean): Promise<void>

type LocalOnlyOverviewResponse<T> =
  | ({ status: 'ok' } & T)
  | { status: 'unsupported_environment'; env_type: string }
```

- Environment identity comes from `getCurrentEnvironment(): Promise<EnvironmentInfo>`.
- CLI version detection uses `getCliVersion({ tool, timeoutMs, force })` only after a supported overview exists.

### 3. Contracts

- Every refresh establishes the active environment before reading overview or version data. Inflight environment, overview, and version requests may be deduplicated independently.
- Environment detection failure is fail-closed for the mounted view: clear its rendered overview, set the initial/load error, and issue no overview or version request.
- A detected non-Local environment sets the Local-only state, clears the mounted overview, and issues no platform overview or version request.
- When the Local environment ID changes, invalidate all module-level overview and version caches before loading data for the new environment.
- A backend `unsupported_environment` response is authoritative even after a frontend Local result. Clear both module-level caches, clear the mounted overview, set the Local-only state from the envelope, and skip version detection.
- Version detection runs only after a Local overview succeeds. Overview and version TTLs are independent.
- An ordinary forced overview refresh failure preserves a previously rendered overview and reports `refreshError`. An initial overview failure reports `loadError` and does not run version detection.
- A forced version refresh failure preserves the last confirmed cached version and reports `refreshError`; without a confirmed version, render the version error state.

### 4. Validation & Error Matrix

| Condition | Required behavior |
| --- | --- |
| Environment lookup rejects | Clear mounted overview; `loadError`; no overview/version call |
| Environment is WSL/SSH | Local-only state; clear mounted overview; no overview/version call |
| Local environment ID changes | Clear overview and version caches before loading |
| Overview returns `unsupported_environment` | Clear all dashboard caches; Local-only state; no version call |
| Initial overview request rejects | No overview; `loadError`; no version call |
| Forced overview refresh rejects with stale overview | Preserve overview; `refreshError` |
| Forced version refresh rejects with cached version | Preserve confirmed version; `refreshError` |

### 5. Good / Base / Bad Cases

- Good: a cached Local overview is hidden immediately when environment detection fails, and no backend data is fetched until the environment is known.
- Base: Local environment -> overview succeeds -> version detection runs -> both values are reused within their independent TTLs.
- Bad: render the shared overview before checking the environment, or call version detection in parallel with an overview that can return `unsupported_environment`.

### 6. Tests Required

- Mock a non-Local environment and assert that neither the overview nor version API is called.
- Seed successful caches, then return `unsupported_environment`; assert the mounted overview is null, the version call count does not increase, and a subsequent refresh cannot restore cached local data.
- Seed a successful overview/version, reject forced overview refresh, and assert stale overview plus `refreshError`.
- Seed a successful version, reject forced version refresh, and assert the confirmed version remains visible plus `refreshError`.
- Reject the environment lookup after seeding a shared overview; assert the mounted view clears it and does not call overview/version APIs.
- Run `cd ccr-ui && bunx vitest run --config vitest.smoke.config.ts tests/grok-dashboard.smoke.test.ts` and `just frontend-check-quick`.

### 7. Wrong vs Correct

#### Wrong

```ts
const [overview, version] = await Promise.all([
  loadOverview(),
  getCliVersion({ tool: 'grok' }),
])
```

This can combine remote version state with Local-only configuration data and can retain stale data after an authoritative unsupported response.

#### Correct

```ts
const environment = await loadEnvironment()
if (environment.env_type !== 'local') return showLocalOnly(environment.env_type)

const overview = await loadOverview()
if (overview.status === 'unsupported_environment') {
  resetSharedCaches()
  return showLocalOnly(overview.envType)
}

await loadVersion()
```
