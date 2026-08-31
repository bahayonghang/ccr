# Development Resource Contracts

> Keep local Vite and Vitest workloads bounded without changing production behavior.

## Scenario: bounded frontend development resources

### 1. Scope / Trigger

Apply this contract when changing `vite.config.ts`, `vitest.smoke.config.ts`, the
`jsdom` dependency or browser-runtime test shims, `scripts/dev-*`, Vite measurement
scripts, watcher paths, warmup targets, cache cleanup, or child-process lifecycle
behavior.

The contract prevents Vite from watching Rust build output, avoids duplicate startup transforms, guarantees task-owned process cleanup, preserves dependency caches, and leaves CPU capacity for the IDE and Rust toolchain.

### 2. Signatures

- Watcher exclusions: `server.watch.ignored: string[]` in `vite.config.ts`.
- Warmup manifest: `scripts/dev-warm-targets.json` with `{ healthPath: string, clientFiles: string[] }`.
- Process cleanup:

  ```ts
  terminateProcessTree(
    child: ProcessTreeChild,
    options?: ProcessTreeOptions,
  ): Promise<void>
  ```

- Smoke worker policy:

  ```ts
  resolveSmokeMaxWorkers(
    env?: NodeJS.ProcessEnv,
    available?: number,
  ): number
  ```

- jsdom runtime boundary:
  `tests/quality/jsdom-runtime-contract.smoke.test.ts`.

- Windows acceptance probe:

  ```powershell
  pwsh -File scripts/verify-vite-resources.ps1 `
    [-Port <1..65535>] [-SoakSeconds <10..600>] `
    [-MaxHandleGrowth <n>] [-MaxPrivateGrowthMB <n>] `
    [-MaxMachineCpuPercent <n>]
  ```

### 3. Contracts

- Vite must ignore `**/src-tauri/target/**`, `**/ref/**`, and `**/logs/**` while keeping `src/**`, configuration files, the shared provider catalog allowlist, and `public/**` watched.
- `clientFiles` belongs only to Vite's native `server.warmup.clientFiles`. The wrapper may issue one bounded request to `healthPath`; it must not fetch module targets or secondary routes.
- Windows cleanup uses `taskkill /PID <pid> /T /F`. Unix cleanup sends `SIGTERM`, waits for a bounded grace period, then escalates to `SIGKILL`. Repeated cleanup calls for the same child return the same promise.
- Routine startup preserves `node_modules/.vite`. `CCR_DEV_RESET_VITE_CACHE=1` is the only supported automatic reset switch.
- `CCR_DEV_HEALTH_TIMEOUT_MS` is a positive duration capped at 300,000 ms; invalid values fall back to 60,000 ms.
- Local smoke tests use at most two workers by default. CI uses at most four and never exceeds `availableParallelism()`. `CCR_TEST_WORKERS` accepts positive decimal integers only and is capped at available parallelism.
- The resolved jsdom release must support the canonical hosted Node runtime. A
  successful run on a newer local Node major is not evidence for the hosted Node
  lane.
- Tests use jsdom's native `PointerEvent`; do not install a global or per-file
  `PointerEvent` polyfill that would hide `HTMLElement.click()` or pointer-event
  behavior from the selected jsdom release.
- Keep capability shims narrow: the shared in-memory `localStorage` and
  `matchMedia` setup provide deterministic test isolation, while
  `ResizeObserver` and `scrollIntoView` stubs belong only in tests that exercise
  consumers of APIs jsdom does not implement. Do not promote those layout-only
  stubs to a fake browser-wide implementation.
- Live resource probes create only a unique `.ccr-vite-resource-probe-<GUID>` directory under `src-tauri/target` and remove it in `finally`; they never delete existing target output.

### 4. Validation & Error Matrix

| Condition | Required behavior |
| --- | --- |
| Generated Rust output changes | No Vite HMR event or sustained handle growth |
| Health request exceeds timeout or returns non-2xx | Report the health failure, terminate the Vite tree, exit non-zero |
| Vite child exits unexpectedly | Propagate its failure exit status and do not leave descendants |
| Measurement and cleanup both fail | Preserve the measurement error; log cleanup failure separately |
| `taskkill` reports an error after the child already exited | Treat cleanup as successful |
| Process remains after bounded cleanup | Reject with the target PID in the error |
| `CCR_TEST_WORKERS` is zero, fractional, or non-numeric | Use the normal local/CI default |
| jsdom supplies native `PointerEvent` | Exercise it directly; a test shim must not replace it |
| A component needs an unimplemented layout API | Stub only the missing capability at that test's boundary |
| Resource threshold is exceeded | Emit JSON measurements, clean probe/process artifacts, exit non-zero |

### 5. Good / Base / Bad Cases

- Good: Rust builds update thousands of files under `src-tauri/target`; Vite handles remain stable and source HMR still works.
- Base: Vite starts normally, native warmup handles the root route, one health request succeeds, and Ctrl+C removes the wrapper, Vite child, descendants, and listener.
- Bad: a launcher deletes `.vite` on every start, manually fetches every warm target, watches `src-tauri/target`, or kills only the direct Windows child.

### 6. Tests Required

- Resolve the real Vite config and assert all three ignored patterns are present while `src/main.ts` and a public asset remain accepted by the watcher filter.
- Parse `dev-warm-targets.json`; assert targets exist, are unique, include the root route, and exclude lazy settings modules.
- Assert the wrapper source contains one bounded fetch and no `clientFiles` or legacy probe arrays.
- Unit-test local, CI, explicit, capped, and invalid worker values.
- Assert the selected jsdom runtime provides native `PointerEvent`, dispatches a
  `PointerEvent` from `HTMLElement.click()`, preserves localhost secure-cookie
  behavior, and implements the expected CSSOM behavior without pretending to
  provide layout-only APIs.
- Search the smoke suite after a jsdom upgrade and remove obsolete
  `PointerEvent` shims. Retain `ResizeObserver`, `scrollIntoView`, `matchMedia`,
  and storage shims only where their consumer or deterministic-isolation reason
  still exists.
- Unit-test Windows full-tree cleanup, already-exited idempotence, and Unix SIGKILL escalation.
- Run `scripts/verify-vite-resources.ps1` for explicit Windows acceptance and assert resource thresholds plus parent/listener cleanup.
- Required commands:

  ```powershell
  cd ccr-ui
  bunx vitest run --config vitest.smoke.config.ts tests/quality/jsdom-runtime-contract.smoke.test.ts
  bunx vitest run --config vitest.smoke.config.ts tests/quality/dev-tooling-resource.smoke.test.ts
  bun run type-check
  bun run lint
  cd ..
  just frontend-check-quick
  ```

### 7. Wrong vs Correct

Wrong: routine startup destroys the cache and leaves Vite watching all nested build output.

```powershell
Remove-Item -Recurse -Force node_modules/.vite
```

```ts
server: { watch: {} }
```

Correct: cache reset is explicit and generated/high-churn paths are excluded without disabling normal HMR.

```powershell
if ($env:CCR_DEV_RESET_VITE_CACHE -eq '1') {
    Remove-Item -Recurse -Force node_modules/.vite
}
```

```ts
server: {
  watch: {
    ignored: [
      '**/src-tauri/target/**',
      '**/ref/**',
      '**/logs/**',
    ],
  },
}
```


---

## Common Mistake: Playwright `addInitScript` drops extra args in Tauri web-preview mocks

**Symptom**: A web-preview mock of `window.__TAURI_INTERNALS__` partially works —
`list_sync_assets` returns fixtures but `sync_status` resolves `undefined`, so
the Sync page silently renders the "not configured" branch and gating checks
look broken.

**Cause**: `page.addInitScript(fn, arg)` accepts exactly ONE argument. A call
like `page.addInitScript(mock, assets, status)` silently drops `status`.

**Fix**: pass a single fixture object.

```ts
// Wrong
await page.addInitScript(mock, assets, status)
// Correct
await page.addInitScript(mock, { assets, status })
```

**Prevention**: reference pattern lives in `ccr-ui/.tmp/sync-preview-check.cjs`.
When verifying dark theme in web preview, the app boots light by default — set
`document.documentElement.setAttribute('data-theme', 'dark')` after load
instead of relying on mocked preference commands.
