# React Rerender Discipline

> View-layer re-render constraints for the React migration: four lint-enforced rules and three review-gate rules.

---

## Scenario: view-layer re-render constraints

### 1. Scope / Trigger

- Applies to all new/migrated React view code in `ccr-ui/src/features/**` and `ccr-ui/src/views/**` (and any store/composable subscription code in `src/**`).
- Why this exists (see `perf-baseline.md`): Vue's fine-grained reactivity re-renders only components whose reactive deps changed; React re-renders from the top of the component tree on state change, so the same data flow can re-render unrelated subtrees. The 353 `v-model` landing zones concentrated in config forms (`perf-baseline.md` §1) are the highest-risk surface.
- The four lint-enforced entries are `error` level (design §6: "可 lint 的四项落为 error 规则"); the three review-gate entries are checked in code review, not by tooling.
- Contract authors: the seven view subtasks must read this document before starting work (R8).

### 2. Lint-Enforced Discipline (error-level)

| # | Discipline | Rule ID | Scope / config block |
| --- | --- | --- | --- |
| 1 | Form inputs use react-hook-form uncontrolled registration (`useForm`/`register`); controlled `<input value + onChange>` without `defaultValue` is banned | `no-restricted-syntax` (selector `JSXOpeningElement[name.name='input']:has(value):has(onChange):not(:has(defaultValue))`) | `src/features/**/*.{tsx,jsx}` + `src/views/**/*.{tsx,jsx}` (`app/rerender-views`) |
| 2 | List-item components use `memo`; no inline object/function props to list items | `react/jsx-no-bind` | `src/features/**/*.{tsx,jsx}` + `src/views/**/*.{tsx,jsx}` (`app/rerender-views`) |
| 3 | Zustand subscriptions always pass a selector; bare whole-store subscription banned | `no-restricted-syntax` (selector `CallExpression[callee.name=/^use[A-Z]\w*Store$/][arguments.length=0]`) | `src/**/*.{ts,tsx,mts}` (`app/rerender-store-subscription`) |
| 4 | List keys never use array index | `react/no-array-index-key` | `**/*.{tsx,jsx}` (`app/rerender-jsx-keys`) |

Notes:
- Entries 1–2 deliberately exclude `src/ui/` and `src/shell/`: primitives and shell glue may legitimately receive inline handlers and implement controlled primitives (scope decision recorded in the config comment and `layering-contracts.md` §5).
- Entry 3 targets Zustand stores. Current `src/` hits are legacy Pinia (Vue) store calls — registered as file-level exemptions in the batch-4 block, assigned to `08-22-state-logic-port` / view subtasks; removed when rewritten with selectors.

### 3. Review-Gate Discipline (not lintable)

| # | Discipline | Review check |
| --- | --- | --- |
| 5 | Context split by change frequency | High-frequency values must not share a Provider with low-frequency values; split by change cadence so a high-frequency update only re-renders its consumers |
| 6 | `useMemo`/`useCallback` only across `memo` boundaries or for expensive computes | Every `useMemo`/`useCallback` either feeds a `memo` boundary or computes something measurably expensive; blind memoization is a code smell |
| 7 | Log-stream / chart data updates via ref + batched commits | Log feeds (500-row cap) and chart series updates must accumulate via ref and commit in batches (e.g. `flushSync` or rAF-throttled commit), not one `setState` per entry |

### 4. Pointers

- `perf-baseline.md` (`.trellis/tasks/08-22-arch-quality-perf/`): the five measured scenarios (large-form input, virtual list scroll, log stream, chart update, route switch) and why they are the regression risk; Vue baseline numbers for comparison.
- `thresholds.md` (same directory): size/complexity caps (max-lines 500, complexity 16, max-depth 2, max-params 3, component style 412) keep components small enough that re-render scope stays reviewable.
- Phase-7 re-measurement: `08-22-regression-release` step 7 re-runs the same framework-agnostic scripts (`ccr-ui/scripts/perf/`, `--base-url`/`--cdp-url` redirection) to compare against `perf-baseline.md`; the three review-gate entries are the primary suspected regression drivers.

### 5. Good/Base/Bad Cases

- Good: a config form uses `useForm` + `register`; a 10k-row list item is `memo`ized with stable props; a store consumer reads `useUsageStore((s) => s.total)`.
- Base: `src/ui/` primitive receives an inline click handler from its parent (allowed outside list-item scope).
- Bad: `<input value={form.values.x} onChange={...}>` without `defaultValue`; a list item recreated each render via `onClick={() => ...}`; `useUsageStore()` with no selector; `items.map((item, i) => <li key={i}>)`; a log feed doing `setEntries([...entries, entry])` per entry.

### 6. Tests Required

- `cd ccr-ui && bun run lint:ci` → exit 0 (entries 1–4 enforced).
- Red-proof for entries 1–4 is documented in `08-22-arch-quality-perf/implement.md` batch-4 evidence (temporary scratch file → 7 errors, reverted).
- Entries 5–7 are review items on each view subtask PR, cross-checked in the phase-7 re-measurement.

---

Sibling contract: `layering-contracts.md` covers dependency direction and component layering; this document covers re-render discipline only.
