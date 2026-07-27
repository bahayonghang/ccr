# Research: Vite development resource growth

## Live evidence (2026-07-27)

| Signal | Evidence |
| --- | --- |
| Process tree | `dev-web-warm-start.mjs` PID 65844 -> Vite PID 68472 |
| Runtime | Vite had run for 24.22 hours |
| CPU | About 11.5% of total CPU, or roughly 1.8 of 24 logical processors |
| Memory | 2.94 GB working set; 3.93 GB private bytes |
| Handles | 203,734 -> 204,087 over five seconds |
| Network | One listening socket; no connection volume explaining the handle count |

The parent warm-start process used about 40 MB and 194 handles. The load was isolated to the Vite child.

## Repository evidence

- `ccr-ui/src-tauri/target`: about 190,544 files (`debug` 118,664; `release` 63,016; `llvm-cov-target` 8,997).
- `ccr-ui/ref`: about 1,439 files.
- `ccr-ui/public/fonts`: 134 files and not a dominant source; keep public asset watching intact.
- `ccr-ui/vite.config.ts:47-62`: server configuration has warmup but no watcher exclusions.
- Installed Vite 7.3.6 resolves watcher options with defaults for `.git`, `node_modules`, `test-results`, Vite cache and build output only. It then calls Chokidar on the whole project root.
- `ccr-ui/scripts/dev-web-warm-start.mjs:33-56,92-139`: the wrapper loads all warm targets and fetches them even though Vite already owns the same `clientFiles` list.
- `ccr-ui/scripts/dev-web-warm-start.mjs:63-72`: shutdown sends a signal only to the direct Vite child.
- `ccr-ui/scripts/measure-vite-route.mjs:268-278`: the diagnostic path already proves `taskkill /T` is the established Windows tree-cleanup mechanism.
- `ccr-ui/scripts/dev-web-windows.ps1:26-32`: routine startup deletes `node_modules/.vite`.
- `ccr-ui/vitest.smoke.config.ts:11-20`: 103 smoke files are collected without a worker ceiling.

## Root-cause ranking

1. **Confirmed dominant cause:** Vite watches the Rust build output nested under its project root. The file count closely matches the abnormal process handle count, and build output changes continuously during Rust development.
2. **Confirmed lifecycle defect:** the normal wrapper cannot guarantee Windows descendant cleanup, allowing an orphaned Vite server to keep accumulating resources.
3. **Confirmed repeat-work defect:** native and manual warmup transform overlapping modules and repeat route probes.
4. **Confirmed restart penalty:** routine cache deletion forces optimizeDeps work on every Windows web-dev start.
5. **Development pressure multiplier:** unconstrained Vitest workers compete with Vite, Rust, IDEs and language servers on a 24-logical-processor workstation.
6. **Secondary cost:** Tailwind/PostCSS cold transformation remains expensive, but it does not explain a 204,000-handle process by itself.

## External reference

- Vite issue #22672: Windows watcher CPU/memory runaway after a watched OneDrive project directory is deleted. The issue is not a direct reproduction of this repository, but confirms that watcher event storms can produce the same failure shape.

## Design consequence

Fix local watcher scope first and validate resource slopes. Do not treat a dependency upgrade or an automatic restart watchdog as the primary solution.
