# 执行计划：React 基座与构建管线

> 父任务：`08-22-react-migration`（阶段 1，与 `08-22-dep-upgrade` 交织执行）。
> 分支：`feature/react-migration/react-foundation`，PR 目标 `feature/react-migration`。

## 前置确认

- [x] 父任务的基线采集门已通过（视觉、性能、测试三类基线已落盘）。基线未采集则本任务不启动。
- [x] `git checkout -b feature/react-migration/react-foundation feature/react-migration`（实际分支名 `react-migration/react-foundation`：`feature/react-migration/<slug>` 与既有分支 `feature/react-migration` 为冲突 ref，无法并存，见父任务 §11 偏差记录）。

## 提交批次

本任务与 `08-22-dep-upgrade` 交织：依赖安装由 dep-upgrade 的第一段提交完成，本任务的批次 1 依赖该提交。

### 批次 1：入口与 Provider 装配

- [x] 建 `src/main.tsx`、`src/shell/App.tsx`、`src/shell/router.tsx`、`src/shell/queryClient.ts`。
- [x] 按 `design.md` §1 的顺序装配 Provider，开启 StrictMode。
- [x] `index.html` 的入口脚本改指 `main.tsx`。
- [x] 删除 `src/main.ts`。`src/App.vue` 暂留，由 `08-22-shell-port` 删除。
- [x] 最小页面内容：一个按钮调用 `src/api` 下的一个 wrapper 并显示返回值（`systemApi.getVersion`，即 `check_version`），一个带 `listen()` 订阅的示例组件（`shell/useTauriListen.ts`，dispose 取消协议防 StrictMode 双订阅）。

验证（两个环境分开做）：

- [x] `bun run dev`（纯 Vite）打开页面，渲染成功（h1「CCR UI — React 基座」+ IPC 按钮 + 事件计数卡均渲染；仅 Tauri 运行时缺失的 `listen()` unhandled rejection，属 AC1 明示不计入项）（AC1 ✅ 2026-08-23 主线程浏览器实测）。
- [x] `bun run tauri:dev`（桌面运行时）打开页面，按钮触发真实 IPC 并显示返回值（AC2 ✅ 2026-08-23 主线程经 WebView2 CDP 实测：`check_version` 返回 `{"current":"7.2.0","latest":null,"update_available":false}` 渲染于页面）。

### 批次 2：构建配置

- [x] `vite.config.ts` 按 `design.md` §3 逐项改写（vite 8/rolldown 下 manualChunks 需函数形态；alias 双入口 vue-i18n 删除；warmup/fs/port 保留）。
- [x] 测量 `@vitejs/plugin-react` 与 SWC 变体的冷启动与 HMR 耗时，二选一，数据落盘（`plugin-selection.md`：中位数差异小于组内波动，维持 plugin-react，SWC 卸载）。
- [x] 确认 `dev-warm-targets.json` 的生成方式（手写清单，无生成脚本，4 处消费/校验方已登记于 plugin-selection.md §dev-warm-targets）。
- [x] `optimizeDeps.include` 清单重写（含 `react-dom/client` 子路径——缺它会在 noDiscovery 下以裸 CJS 直出导致 `createRoot` 具名导入失败，2026-08-23 修复）。

- [x] 验证：`bun run build` 成功（exit 0，182 modules，react-vendor 278.6 kB + index 167.2 kB）；`bun run dev` 冷启动耗时已记录于 `plugin-selection.md`（ready 中位数 4526ms / page 145ms / HMR 530ms）。

### 批次 3：类型检查与 lint

- [x] `tsconfig` 加 `"jsx": "react-jsx"`，`include` 去 `.vue`（`src/vite-env.d.ts` 增设 `*.vue` ambient shim 兜住 legacy `.ts` 的 `.vue` 导入）。
- [x] `package.json` 的 `type-check` 改 `tsc --noEmit`，`vue-tsc` 不在依赖（补装 `@types/react`/`@types/react-dom`，批次 1 遗留缺口）。
- [x] `eslint.config.js` 换 React 规则集（plugin-react + plugin-react-hooks 仅注册，规则启用归 `08-22-arch-quality-perf`），Vue 系 4 插件移除，`**/*.vue` 加入 ignore。
- [x] `.stylelintrc.json` 移除 `stylelint-config-recommended-vue` 与 postcss-html 接线；`lint:style` glob 收窄为 css。
- [x] 保留 `@typescript-eslint/no-explicit-any: error`，未新增其他规则。

- [x] 验证通过（2026-08-23）：`bun run type-check`、`bun run lint`、`bun run lint:style` 均 exit 0。

### 批次 4：测试环境

- [x] `vitest.smoke.config.ts` 按 `design.md` §5 改写（plugin-react + jsdom + cleanup setup）；`vitest.shims.d.ts` 删除（仅指向未安装包的三斜线引用，无引用方）。
- [x] React smoke 测试：`tests/react-shell.smoke.test.tsx`（mock invoke → 断言 `check_version` 返回值渲染）。
- [x] `tests/use-tauri-listen.smoke.test.tsx`：延迟 resolve 的 listen mock，覆盖「resolve 先于卸载」与「resolve 发生在卸载之后」两时序，断言 unlisten 恰好配对（TPR-05，供 `08-22-state-logic-port` AC5 参照）。

- [x] 验证通过（2026-08-23）：`bun run test:smoke` exit 0，59 文件 / 293 用例全绿。

### 批次 5：资产复用验证与判定清单

- [x] `git diff --stat src/api src/types` 确认为空（AC6 ✅ 主线程复验 0 行）。
- [x] `src/utils` 31 个文件逐个判定，`utils-disposition.md` 落盘（AC7）：原样复用 19 / 需接线 12（Tauri 运行时 8 + Vue 耦合 4：apexChartsCore、claude/codex/grokProfiles）。
  - 勘误：prd Notes 的 11 项需接线预期经实测修正——errorHandler / runtimeState / fontPreferences 三项纯逻辑改判原样复用，apexChartsCore 补入需接线；claude/codex/grokProfiles 三项为 Vue 耦合（prd 清单本就含 vue 导入判定，映射表归 `08-22-views-profiles-config` 随共享层迁移重写）。详见 utils-disposition.md 偏差登记。
- [x] `src-tauri` 下 `cargo check`（AC8）exit 0（36s）。

### 批次 6：路径映射表

- [x] 按 `design.md` §8 的格式产出 `path-mapping.md`，216 行（185 vue + 31 utils），脚本比对无空缺无重复（AC9）。
- [x] 移交 `08-22-platform-unify` 的文件标注收敛方式：18 个（收敛为薄壳）+ 3 个 views/generic base 本体（统一层 base，协同点 G）。
  - 勘误：「20 个移交文件」实为 18（文档算术误差，以 platform-unify 权威清单为准，行数分项和恰为 15,672；登记于 path-mapping.md 头部）。
- [x] 表内每个新路径的归属子任务与父任务 `prd.md` 的 18 子任务范围表一致（脚本校验全部落在 18 个 slug 内）。

### 批次 7：zod 试点

- [x] 按 `design.md` §9 完成试点，`zod-pilot.md` 落盘：`src/schemas/versionInfo.ts` + `tests/zod-pilot.smoke.test.ts`（保留供 state-logic-port 参照）；编译期 Equal 断言通过；bundle 增量 gzip +15.6 KiB（zod 核心一次性成本）；结论：推广到新增 wrapper，不回填 57 个既有 wrapper。

## 验证命令

| 时机      | 命令                                             |
| --------- | ------------------------------------------------ |
| 每批次后  | `bun run type-check`、`bun run lint`             |
| 批次 1 后 | `bun run dev`（AC1）、`bun run tauri:dev`（AC2） |
| 批次 2 后 | `bun run build`                                  |
| 批次 4 后 | `bun run test:smoke`                             |
| 批次 5 后 | `cd src-tauri && cargo check`                    |
| 交付前    | `just frontend-check-quick`、`just tauri-check`  |

## 交付门

- [x] AC1–AC9 全部满足（AC1/AC2 见批次 1 实测记录；AC3–AC9 见批次 3–6）。
- [x] `utils-disposition.md`、`path-mapping.md`、插件选择测量数据（`plugin-selection.md`）、zod 试点结论（`zod-pilot.md`）四项落盘。
- [x] `package.json` 无 `vue-tsc` 与 4 个 Vue 系 ESLint 插件。
- [x] 与 `08-22-dep-upgrade` 共同满足父任务的基座门（2026-08-23 主线程逐项核对通过：frontend-typecheck/lint/tauri-check exit 0，无 Vue 系依赖，codemirror peer 结论与 Tailwind v4 处理落盘）。

## 回滚点

| 批次 | 回滚方式                                                                 |
| ---- | ------------------------------------------------------------------------ |
| 1–2  | 单独 revert。`src/main.ts` 与 `vite.config.ts` 恢复即回到 Vue 可运行状态 |
| 3–4  | 单独 revert。检查管线配置与业务代码无耦合                                |
| 5–7  | 只产出文档与判定，revert 不影响代码                                      |

## 协同点

| 编号 | 内容                                                          | 对方                       |
| ---- | ------------------------------------------------------------- | -------------------------- |
| J    | `path-mapping.md` 是逐屏比对的对照依据                        | `08-22-regression-release` |
| —    | 依赖安装、Tailwind v4、vite 8 由对方提交，本任务批次 1–2 依赖 | `08-22-dep-upgrade`        |
| —    | `utils-disposition.md` 的「需接线」11 项是对方的范围输入      | `08-22-shell-port`         |
| —    | zod 试点结论供 bundle 预算引用                                | `08-22-arch-quality-perf`  |
