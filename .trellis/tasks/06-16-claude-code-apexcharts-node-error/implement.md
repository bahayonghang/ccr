# 实施计划 · ApexCharts node 报错

## 步骤

1. **新增 `ccr-ui/src/components/claude-observer/ChartErrorBoundary.vue`**
   - `onErrorCaptured` 接住子树异常 → 自愈（rAF 重挂，`reloadKey++`）/ 超限降级 `ChartPreparingState` → `return false`。
   - 作用域插槽暴露 `reloadKey`；props：`label?`、`maxRetries?=2`。
   - 用 `logger.warn` 记录被接住的异常（便于排查，不丢信息）。
   - verify：组件能编译、被边界包裹的 throwing 子组件不向上冒泡。

2. **改写 `ccr-ui/src/components/claude-observer/apexChart.ts`**
   - `ApexChartAsync` → `defineComponent({ inheritAttrs:false })`，渲染
     `ChartErrorBoundary` + 作用域插槽中的异步真图，`key=reloadKey` 透传 `...attrs`。
   - 保留 `vue3-apexcharts` 异步加载 + `ChartPreparingState` loading 态。
   - verify：三 Tab 用法不变，`bun run type-check` 通过。

3. **扩展 `ccr-ui/tests/claude-observer-tabs.smoke.test.ts`（或新增 `chart-error-boundary.smoke.test.ts`）**
   - 用例 A：throwing 子组件 → 边界显示降级态、`app.config.errorHandler` 未被调用（断言无冒泡）。
   - 用例 B：首挂抛错、重挂成功的子组件 → 边界自愈后渲染出子内容。
   - verify：`bunx vitest run --config vitest.smoke.config.ts` 全绿。

4. **验证**
   - `bun run type-check`
   - `bun run lint`
   - `bunx vitest run --config vitest.smoke.config.ts tests/claude-observer-tabs.smoke.test.ts`
   - （可选）Tauri dev 打开 `/claude-code` 人工确认无吐司 + 出图。

## 验收映射（对应 prd.md Acceptance Criteria）

- 无 `应用错误 ... 'node'` 吐司 ← 步骤 1 `return false` + 步骤 3 用例 A
- 面积图正常出图 ← 步骤 1 自愈重挂 + 步骤 2 `:key` 干净重挂
- 三 Tab 不抛错 ← 步骤 2 收口共享封装
- `/usage` 无回归 ← 不动其封装
- 测试/类型/lint 全绿 ← 步骤 4

## 不做（防 scope creep）

- 不改三个 Tab 的 options（不动 `redrawOnParentResize`/`animations`），记为可选后续。
- 不升级/降级 apexcharts 版本。
- 不新增 i18n key（降级态复用既有准备态文案）。
