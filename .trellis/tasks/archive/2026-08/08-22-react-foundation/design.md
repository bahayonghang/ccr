# 技术设计：React 基座与构建管线

> 父任务：`08-22-react-migration`。选型见父任务 `design.md` §1，目录结构见 §2。本文件只写本子任务的局部设计。

## 1. 入口形状

现状：`src/main.ts` 挂载 `src/App.vue`，注册 router、pinia、i18n 三个插件。

目标：

```
src/main.tsx          创建 root，装配 Provider
src/shell/App.tsx     顶层组件，暂为最小页面
src/shell/router.tsx  createBrowserRouter，暂只 1 条路由
src/shell/queryClient.ts  QueryClient 实例与默认选项
```

Provider 嵌套顺序：

```
<StrictMode>
  <QueryClientProvider client={queryClient}>
    <RouterProvider router={router} />
  </QueryClientProvider>
</StrictMode>
```

Zustand 不需要 Provider，store 为模块级单例（`08-22-state-logic-port` 使用该形态）。i18n Provider 由 `08-22-i18n-port` 在本层之外补入，本任务的最小页面使用硬编码中文文案。

## 2. StrictMode 决策

**开启 StrictMode。**

代价：开发模式下 `useEffect` 与 `useState` 初始化函数双调用，Tauri `listen()` 订阅若不幂等会出现双订阅。

理由：双调用暴露的正是 `08-22-state-logic-port` R4 与 AC5 要检查的订阅泄漏。在基座阶段开启，问题在写代码时暴露；在回归阶段才开启，问题在 185 个界面里暴露。

约束：本任务需在最小页面内放一个带 `listen()` 订阅的示例组件，验证 StrictMode 下订阅数不翻倍。该示例作为后续子任务的订阅写法参照。

## 3. 构建配置映射

`vite.config.ts` 现 90 行，逐项映射：

| 现状                                                                                 | 目标                                                                                                                                                                                                                                                                                                                                                         |
| ------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `plugins: [vue()]`                                                                   | `@vitejs/plugin-react`。SWC 变体（`plugin-react-swc`）在本任务内测量冷启动与 HMR 耗时后二选一，测量数据落盘                                                                                                                                                                                                                                                  |
| `alias['@'] → ./src`                                                                 | 不变                                                                                                                                                                                                                                                                                                                                                         |
| `alias['vue-i18n']` 的 dev / build 双入口（避免 runtime compiler 与桌面壳 CSP 冲突） | 删除。i18next 无该问题，`08-22-i18n-port` 确认无等价需求                                                                                                                                                                                                                                                                                                     |
| `manualChunks` 10 组                                                                 | 逐组改写：`vue-vendor` → `react-vendor`（react / react-dom / react-router）、`ui-vendor` → `@iconify/react`、`i18n-vendor` → i18next 系、`virtual-vendor` → `@tanstack/react-virtual`、`charts-vendor` → `apexcharts/core` + `react-apexcharts`。新增 `query-vendor`、`form-vendor`、`motion-vendor` 的必要性由 `08-22-arch-quality-perf` 的 bundle 预算判定 |
| `chunkSizeWarningLimit: 500`                                                         | 保留，取值复核归 `08-22-arch-quality-perf`                                                                                                                                                                                                                                                                                                                   |
| `server.port: 15173` / `strictPort`                                                  | 不变                                                                                                                                                                                                                                                                                                                                                         |
| `server.fs.allow` 放行 `crates/ccr-checkin/data`                                     | 不变。`providers-catalog.json` 为前后端共享数据源                                                                                                                                                                                                                                                                                                            |
| `server.warmup.clientFiles` 读 `dev-warm-targets.json`                               | 目标文件列表在目录重组后失效，需重新生成。该文件的生成方式在本任务内确认                                                                                                                                                                                                                                                                                     |
| `optimizeDeps.noDiscovery: true` + 15 项 `include`                                   | 保留 `noDiscovery`。`include` 清单按新依赖重写，Vue 系 5 项删除                                                                                                                                                                                                                                                                                              |

`vite` 7.3.5 → 8.2.2 为跨主版本升级，与插件切换同一提交内完成（`08-22-dep-upgrade` 的第一段）。

## 4. 类型检查与 lint

| 项目                 | 现状                                                                                      | 目标                                                                                                           |
| -------------------- | ----------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------- |
| `type-check`         | `vue-tsc --noEmit`                                                                        | `tsc --noEmit`                                                                                                 |
| `tsconfig`           | Vue SFC 支持                                                                              | `"jsx": "react-jsx"`；`include` 去掉 `**/*.vue`                                                                |
| lint parser          | `vue-eslint-parser`                                                                       | `@typescript-eslint/parser`（已在依赖中）                                                                      |
| lint 插件            | `eslint-plugin-vue` + `@vue/eslint-config-typescript` + `@intlify/eslint-plugin-vue-i18n` | `eslint-plugin-react-hooks` + `eslint-plugin-react`（i18n 静态检查的等价方案归 `08-22-i18n-port` R9）          |
| stylelint            | `stylelint-config-recommended-vue` + `postcss-html`                                       | 两者移除。`postcss-html` 只服务 SFC 内 `<style>`，无 `.vue` 后不需要                                           |
| 未迁移的 `.vue` 文件 | 在 lint 与 typecheck 范围内                                                               | 显式排除。阶段 1–5 期间 `.vue` 与 `.tsx` 并存，`.vue` 不再进任何检查管线，也不再被 vite 编译（无引用即不编译） |

`package.json` 的 `type-check`、`lint`、`lint:style`、`test:smoke` 四个 script 名不变，实现替换。命令名是 `justfile` 与 CI 的接口。

## 5. 测试环境

`vitest.smoke.config.ts` 改动：

- `environment: 'jsdom'` 保留（`jsdom` ^26.1.0 不变）。
- `plugins` 换 React 插件。
- `setupFiles` 增加 `@testing-library/react` 的 `cleanup`（`afterEach`）。
- `globals` 与 `vitest.shims.d.ts` 按 React 测试工具的类型声明调整。

本任务只交付 1 个 React smoke 测试（AC5），断言最小页面渲染出 IPC 返回值。122 个测试的重写归 `08-22-test-contract-rebuild`。

## 6. 三层资产复用验证方法

| 资产                       | 验证方法                                                                         |
| -------------------------- | -------------------------------------------------------------------------------- |
| `src/api/**`（57 文件）    | 在最小页面内导入并调用一个 wrapper，观察返回值。`git diff --stat src/api` 须为空 |
| `src/types/**`（231 文件） | `tsc --noEmit` 通过即证明可导入。`git diff --stat src/types` 须为空              |
| `src/utils/**`（31 文件）  | 逐个判定，方法见第 7 节                                                          |

`src/api/tauri.ts` 的冻结门面约定不变：新 wrapper 只落 `src/api/domains/<domain>.ts`。本任务不新增 wrapper。

## 7. `src/utils` 判定方法

判定依据为该文件是否导入 `vue` 或依赖 Vue 运行时。

```
rg -l "from 'vue'|from \"vue\"" src/utils
```

已知需接线的 11 个：`windowChrome`、`tauriWindow`、`nativeWindowAppearance`、`themeBootstrap`、`fontPreferences`、`startupRecovery`、`perfTelemetry`、`runtimeState`、`tauriRuntime`、`errorHandler`、`logger`。这 11 个的接线归 `08-22-shell-port`。

判定清单落盘为 `utils-disposition.md`，两列：文件名、判定（原样复用 / 需接线）。31 行，无空缺。

## 8. 路径映射表（AC9）

格式：

| 旧路径                            | 新路径                                     | 归属子任务          |
| --------------------------------- | ------------------------------------------ | ------------------- |
| `src/views/CodexSettingsView.vue` | `src/features/codex/CodexSettingsView.tsx` | `08-22-views-codex` |

规则：

- 文件基名不变，仅扩展名与目录变化。
- 覆盖 185 个 `.vue` 与 31 个 `utils` 文件，合计 216 行。
- 移交 `08-22-platform-unify` 的 20 个文件，新路径填统一层路径，并标注「收敛为薄壳」或「删除」。
- 落盘为 `path-mapping.md`。该表是 `08-22-regression-release` 逐屏比对的对照依据（父任务 `design.md` §12.1），也是七个视图子任务确认自身文件落位的唯一来源。

映射表在本任务产出时按当前测量填写。若后续子任务实际落位与表不一致，改表不改文件，表随实现更新。

## 9. zod 试点范围

试点对象：本任务在最小页面调用的那一个 IPC wrapper。

试点内容：为其返回值写一个 zod schema，与 ts-rs 生成的类型做 `z.infer` 一致性检查，测量 schema 手写成本与 bundle 增量。

试点产出一个结论：是否值得推广到新增 wrapper。不回填 57 个既有 wrapper（父任务 `design.md` §15）。结论落盘，供 `08-22-arch-quality-perf` 的 bundle 预算引用。

## 10. 未决项

- `@vitejs/plugin-react` 与 SWC 变体的二选一，由第 3 节的测量决定。
- `dev-warm-targets.json` 的生成方式需在本任务内确认；若无生成脚本，`server.warmup` 配置改为手写清单或移除。原因未查明前不删除该配置。
- `manualChunks` 是否新增 `query-vendor` / `form-vendor` / `motion-vendor`，等 `08-22-arch-quality-perf` 的预算数据。
