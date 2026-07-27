# Implementation plan

## 1. Establish focused regression guards

- [x] Add Node-environment smoke tests for resolved watcher exclusions and preservation of source/public watching.
- [x] Add manifest tests proving warm targets exist, are unique and match root-route startup intent.
- [x] Add worker-budget tests for local, CI, explicit and invalid values.
- [x] Add process-tree helper tests covering normal exit, repeated shutdown and timeout/fallback paths without launching Vite.

Validation:

```powershell
cd ccr-ui
bunx vitest run --config vitest.smoke.config.ts tests/dev-tooling-resource.smoke.test.ts
```

## 2. Exclude generated and high-churn directories

- [x] Add Vite watcher ignores for `src-tauri/target`, `ref` and `logs`.
- [x] Keep existing filesystem allowlist and public/source HMR behavior unchanged.
- [x] Confirm resolved Vite config merges these entries with Vite defaults.

Rollback point: revert only the `server.watch.ignored` block if a legitimate source path is excluded.

## 3. Remove duplicate warmup work

- [x] Change the warm-target manifest to `healthPath` plus a minimal root-route `clientFiles` list.
- [x] Remove manual module fetches, duplicate route probes and concurrency handling from the wrapper.
- [x] Add a bounded one-shot health request and stable readiness output.
- [x] Update `measure-vite-route.mjs` to wait for the new readiness signal.

Validation:

```powershell
cd ccr-ui
$env:PORT='15174'
bun run measure:vite-route -- --route=/
Remove-Item Env:PORT
```

## 4. Unify process-tree shutdown

- [x] Extract the shared cross-platform termination helper.
- [x] Route signal, error, timeout, child-exit and measurement-finally paths through it.
- [x] Prove Windows uses `taskkill /T` and that cleanup is idempotent.
- [x] Ensure shutdown does not report an ordinary Ctrl+C as a product failure.

Rollback point: keep the existing measurement-script `taskkill /T` behavior available until the shared helper passes focused tests.

## 5. Preserve caches and cap test workers

- [x] Stop deleting `node_modules/.vite` during routine Windows web startup.
- [x] Add explicit `CCR_DEV_RESET_VITE_CACHE=1` recovery behavior.
- [x] Add the local/CI/override worker budget to `vitest.smoke.config.ts`.
- [x] Preserve the existing `ccr-ui/package.json` version 7.0.0 user change; avoid touching the file unless a script entry is demonstrably necessary.

## 6. Run resource and lifecycle acceptance

- [x] Ensure the currently running legacy Vite process is stopped before measuring; do not compare against its polluted state.
- [x] Run the focused resource verifier on a free port.
- [x] Exercise controlled writes only inside a newly created probe directory under `src-tauri/target` and clean it in `finally`.
- [x] Check AC6 CPU, memory and handle-growth budgets.
- [x] Stop the parent and prove all task-owned PIDs and the listener disappear within five seconds.
- [x] Repeat one normal restart and prove the Vite cache survives.

## 7. Complete project verification

Run from the repository root unless noted:

```powershell
cd ccr-ui
bun run type-check
bun run lint
bun run test:smoke
cd ..
just frontend-check-quick
git diff --check
git status --short
```

Inspect the final diff and confirm no unrelated dirty file, especially the existing `ccr-ui/package.json` version change, was overwritten or swept into the task.

## 8. Update executable guidance

- [x] Add a concise ccr-ui frontend spec covering generated-directory watcher exclusions, single-owner warmup, process-tree cleanup and local test worker budgets.
- [x] Link the new spec from `.trellis/spec/ccr-ui/frontend/index.md`.
- [x] Re-run the focused tests after the spec update and prepare the task for finish-work.
