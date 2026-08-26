# Development Resource Contracts

> Keep local Vite and Vitest workloads bounded without changing production behavior.

## Scenario: bounded frontend development resources

### 1. Scope / Trigger

Apply this contract when changing `vite.config.ts`, `vitest.smoke.config.ts`, `scripts/dev-*`, Vite measurement scripts, watcher paths, warmup targets, cache cleanup, or child-process lifecycle behavior.

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
- Unit-test Windows full-tree cleanup, already-exited idempotence, and Unix SIGKILL escalation.
- Run `scripts/verify-vite-resources.ps1` for explicit Windows acceptance and assert resource thresholds plus parent/listener cleanup.
- Required commands:

  ```powershell
  cd ccr-ui
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
