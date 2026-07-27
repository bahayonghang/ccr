# Design: bounded ccr-ui development resources

## Architecture and boundaries

The fix stays inside `ccr-ui` development tooling. Production bundles, Tauri commands and application runtime behavior remain unchanged.

```text
just ui-dev-web
  -> dev-web-windows.ps1 (port ownership + optional explicit cache reset)
    -> dev-web-warm-start.mjs (parent lifecycle + one health probe)
      -> Vite (native dependency optimization + native clientFiles warmup)
        -> Chokidar (source/config/public only; Rust target/ref/logs ignored)
```

Smoke tests use a separate bounded worker policy and do not attach to the live Vite process.

## D1. Watcher boundary

Add `server.watch.ignored` entries in `vite.config.ts` for:

- `**/src-tauri/target/**`
- `**/ref/**`
- `**/logs/**`

Vite merges user ignores with its default `.git`, `node_modules`, cache and output ignores. Keep `public/fonts` watched because its 134 files are not material and developers may need asset refresh. Keep `src-tauri/src` visible to normal tooling; only generated build output is excluded.

## D2. Warmup ownership

Retain `dev-warm-targets.json` as the single declarative list consumed only by Vite's `server.warmup.clientFiles`. Replace route/request arrays with a single `healthPath` used by the wrapper.

Reduce `clientFiles` to actual root-route startup modules. The list must include `main.ts`, `App.vue`, router, `MainLayout.vue`, `DashboardView.vue` and directly required shell/bootstrap modules. Remove `AppSettingsView.vue` and other secondary route targets.

The wrapper waits for Vite's ready line, sends one `GET healthPath` with an abort timeout, drains the body and prints a stable ready message. It does not fetch module URLs or issue a second route-probe round.

## D3. Shared process-tree termination

Extract a small ESM helper under `ccr-ui/scripts/` that accepts a spawned child and:

- is idempotent;
- on Windows executes `taskkill.exe /PID <pid> /T /F` and waits for child exit;
- on Unix first sends SIGTERM, waits for a bounded grace period, then uses SIGKILL;
- treats an already-exited process as success;
- surfaces unexpected termination errors without masking the original failure.

Both `dev-web-warm-start.mjs` and `measure-vite-route.mjs` use this helper. Signal handlers initiate the async shutdown once. Startup failure, health timeout, uncaught error and measurement `finally` paths all converge on the same cleanup.

## D4. Cache behavior

Remove unconditional deletion of `node_modules/.vite` from `dev-web-windows.ps1`. Support `CCR_DEV_RESET_VITE_CACHE=1` for explicit recovery and log when the cache is removed. Normal cleanup remains limited to verified CCR-owned processes and PID/port metadata.

## D5. Smoke-test resource budget

In `vitest.smoke.config.ts`, compute `maxWorkers` as:

- explicit positive integer `CCR_TEST_WORKERS`, capped at `availableParallelism()`;
- otherwise `min(2, availableParallelism())` locally;
- otherwise `min(4, availableParallelism())` in CI.

Keep file parallelism enabled within that ceiling. Do not serialize the entire suite unless a focused test proves shared-state contamination.

Pure development-tool contracts run in Node environment. Tests verify target-list existence/uniqueness, resolved watcher ignores, worker-budget parsing and process-tree helper behavior using controlled fake children or short-lived fixture processes. Real Vite startup belongs to the explicit resource verification command, not the default smoke suite.

## D6. Resource verification

Extend the existing measurement tooling or add a focused script that:

1. starts the wrapper on a verified free port;
2. waits for the health-ready signal;
3. records the Vite PID baseline;
4. creates and updates a task-owned probe directory under `src-tauri/target`;
5. samples CPU, working set, private bytes and handles for 60 seconds;
6. removes the probe directory in `finally`;
7. terminates the parent and proves the process tree and port are gone.

The verifier fails when AC6 thresholds are exceeded and prints JSON evidence for later comparison. It never deletes pre-existing build output.

## Compatibility and trade-offs

- Secondary routes may incur their first transform on first navigation because warmup prioritizes the actual root route. This is intentional: steady system responsiveness takes priority over speculative route latency.
- Keeping two local Vitest workers may lengthen the full smoke suite compared with 24-way scheduling, but preserves resources for the IDE, Vite and Rust compiler.
- Ignoring Rust target output removes meaningless HMR events; no frontend source dependency should originate from that directory.
- Dependency upgrades are deferred until the local boundary fix is measured. This avoids combining behavioral fixes with lockfile churn.

## Rollback

Each control is independently reversible: watcher ignores, warm-target reduction, cache reset behavior, process helper and test worker ceiling. If a validation regression appears, revert only the responsible control while retaining the confirmed `src-tauri/target` watcher exclusion unless evidence disproves it.
