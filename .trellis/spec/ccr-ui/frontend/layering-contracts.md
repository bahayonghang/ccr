# Layering Contracts

> Enforced dependency direction, component layering, and facade-boundary division of labor for the React migration.

---

## Scenario: dependency graph enforcement

### 1. Scope / Trigger

- Trigger: adding, moving, or re-exporting a module under `ccr-ui/src/**`.
- Applies to every `import`/`export` in `ccr-ui/src/**/*.{ts,tsx,mts}`.
- Excluded by design: `src/types/generated/**` (ts-rs generated bindings, drift-checked by `just tauri-bindings-check`, not by lint), `**/*.vue` (unmigrated, exit the lint pipeline until phases 4–5), `tests/fixtures/**` (self-check fixtures, not part of normal lint).

### 2. Signatures

- **Single source of truth**: `boundaryElements` and `boundaryPolicies` named exports in `ccr-ui/eslint.config.js`. Both the `app/arch-boundaries` lint block and `scripts/check-arch-boundaries.mjs` consume these exports, so the graph cannot drift between lint and the fixture self-check.
- Element map (from `boundaryElements`):
  - `ui-primitive` → `src/ui`, `src/components/ui`
  - `shell` → `src/shell`
  - `feature` → `src/features/<domain>` (domain captured)
  - `legacy-feature` → `src/views`, `src/components`
  - `store` → `src/stores`
  - `composable` → `src/composables`
  - `api` → `src/api`
  - `utils` → `src/utils`
  - `types` → `src/types`
  - `shared` → `src` (catch-all glue: main / router / i18n / configs)

### 3. Contracts

Dependency graph (target state):

```
features/*  →  features/platform  →  ui  →  styles
     ↓                ↓
    api  →  types
     ↑
  config / configs
```

Forbidden edges:
- `ui` (`ui-primitive`) importing `features/` or `api/` or any store.
- `features/<a>/` importing `features/<b>/` for a ≠ b and b ≠ platform (feature cross-domain direct).
- Any reverse dependency (a lower layer importing a higher layer).

Allowed edges (mirrors `boundaryPolicies`):
- `ui-primitive` → `types` / `utils` / `shared` only.
- `shell` / `shared` → any internal layer (glue may depend on everything).
- `feature` → `store`, `api`, `types`, `utils`, `ui-primitive`, `shared`, same-domain `feature`, `feature/platform`.
- `legacy-feature` → any internal layer (migration-period policy, see §6).
- `store` → `api`, `types`, `utils`, `shared`, `store`.
- `composable` → `api`, `types`, `utils`, `shared`, `store`, `composable`.
- `api` → `api`, `types`, `utils`, `shared`.
- `utils` / `types` → `types`, `utils`, `shared` (bottom of the graph).

### 4. Validation & Error Matrix

| Violation | Mechanism | Rule ID / command | Level |
| --- | --- | --- | --- |
| Cross-layer import, reverse dependency, feature cross-domain direct | `eslint-plugin-boundaries` | `boundaries/dependencies` in `app/arch-boundaries` | error |
| Facade consumer-side bypass (direct `src/api/tauri.ts` import) | ESLint core | `no-restricted-imports` in `app/custom-rules`, sole whitelist `app/facade-coverage-test-whitelist` (`tests/api-facade-coverage.smoke.test.ts`) | error |
| Facade definition-side (new wrapper added to `tauri.ts`) | smoke freeze test (not lint) | `freezes legacy direct invoke calls in tauri.ts` in `tests/api-facade-boundary.smoke.test.ts` | test fail |
| Circular imports | dpdm standalone script | `bun run check:cycles` (CI + `just frontend-check`) | exit ≠ 0 |
| Boundary rule self-check | fixture lint | `bun run check:arch-boundaries` (4 fixtures) | exit ≠ 0 |

Facade division of labor (explicit; do not merge the two sides):
- **Lint freezes only the consumer side.** `no-restricted-imports` with patterns `['**/api/tauri', '**/api/tauri.*']` forbids importing `src/api/tauri.ts` directly. The sole whitelist file is `tests/api-facade-coverage.smoke.test.ts` (the facade-coverage assertion itself), registered as `app/facade-coverage-test-whitelist`. Inside `src/api/**`, relative imports are naturally unaffected; every other consumer must import from `@/api` or `@/api/domains/<domain>`.
- **Lint cannot freeze the definition side.** `src/api/index.ts` re-exports `tauri.ts` (`export * from './tauri'`), so a new wrapper added to `tauri.ts` and consumed through `@/api` passes every import rule. The definition side is frozen by the existing smoke test `freezes legacy direct invoke calls in tauri.ts`, which asserts the `invoke()` command sequence in `tauri.ts` equals exactly the 9-command `ALLOWED_TAURI_FACADE_COMMANDS` allowlist. Adding a command there fails the smoke test.
- The sibling contract `api-facade-boundary.md` covers the facade's behavioral contracts (domain wrappers, generated clients, manifest-typed commands, `INVOKE_ALLOWED_PATHS`). This document covers only the enforcement graph and layering; see the sibling for facade API semantics.

### 5. Component layering

Primitive (`src/ui/`) → composite → domain component → page.

- `src/ui/` primitives accept props/children only. Hard constraints: no imports from `features/`, `api/`, or any store. Enforced by the `ui-primitive` policy above (`types` / `utils` / `shared` only).
- `src/ui/` and `src/shell/` may legitimately receive inline handlers / implement controlled primitives outside list scenarios — the rerender-view lint scope (`src/features/**`, `src/views/**`) deliberately excludes them (see `react-rerender-discipline.md` §2).

### 6. Legacy migration-period note

- `legacy-feature` (`src/views` + `src/components`, the `.vue`-era code still being migrated) carries a permissive policy (any internal layer) until phases 4–5 complete. `.vue` files themselves are ignored entirely; the element type exists so TypeScript files in these directories do not trip the graph during the migration window.
- The permissive policy is an intentional, time-boxed exception, not a gap: it is removed as each view subtask finishes and its files leave `legacy-feature`.

### 7. Exemptions

- No global exemptions. No `eslint-disable` comments in source.
- Per-file, per-rule exemptions are registered as inline-commented override blocks in `eslint.config.js` and enumerated in `.trellis/tasks/08-22-arch-quality-perf/thresholds.md` §3 (49 file × rule entries: 17 registered exemptions + 32 assigned to migration batches; the batch-4 hooks/rerender exemption block adds 9 more). Each block carries the file, the violated metric + measured value, and the disposition (registered or migration-batch owner).
- Removing a block when the disposition completes restores the rule. No exemption is global.

### 8. Good/Base/Bad Cases

- Good: `src/features/claude/X.tsx` imports `@/stores` and `@/api`; `src/ui/Button.tsx` imports only `@/types` and `@/utils`.
- Base: `src/shell/*` glue imports across all layers; `legacy-feature` files keep pre-migration imports until rewritten.
- Bad: `src/ui/Button.tsx` imports `@/features/claude/...`; `src/features/claude/X.tsx` imports `src/features/codex/Y.tsx`; any module imports `src/api/tauri.ts` directly; a new `invoke('new_command')` added to `tauri.ts`.

### 9. Tests Required

- `cd ccr-ui && bun run check:arch-boundaries` → exit 0 (4 fixtures PASS: cross-layer, cross-domain, reverse-dep, facade bypass).
- `cd ccr-ui && bun run check:cycles` → exit 0 (217 files, no cycles).
- `cd ccr-ui && bun ./scripts/check-cycles.mjs --self-check` → exit 0 (detects exactly the 1 cycle-a ↔ cycle-b fixture).
- `cd ccr-ui && bun run lint:ci` → exit 0.
- `cd ccr-ui && bun run test:smoke -- tests/api-facade-boundary.smoke.test.ts` → definition-side freeze.
