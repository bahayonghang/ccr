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

- [ ] `tsconfig` 加 `"jsx": "react-jsx"`，`include` 去 `.vue`。
- [ ] `package.json` 的 `type-check` 改 `tsc --noEmit`，移除 `vue-tsc`。
- [ ] `eslint.config.js`（现 133 行）换 React 规则集，Vue 系 4 个插件移除，`.vue` 加入 ignore。
- [ ] `.stylelintrc.json` 移除 `stylelint-config-recommended-vue`，`postcss.config.js` 移除 `postcss-html`。
- [ ] 保留 `@typescript-eslint/no-explicit-any: error`。新增规则归 `08-22-arch-quality-perf`，本批次不加。

验证：`bun run type-check`（AC3）、`bun run lint`（AC4）、`bun run lint:style` 退出码 0。

### 批次 4：测试环境

- [ ] `vitest.smoke.config.ts` 按 `design.md` §5 改写，`vitest.shims.d.ts` 适配。
- [ ] 写 1 个 React smoke 测试，断言最小页面渲染出 IPC 返回值（用 mock 的 `invoke`）。
- [ ] StrictMode 下订阅数不翻倍的断言写入该测试或独立测试。mock 的 `listen` 必须**延迟 resolve**（`await` 一个可控 deferred 后再返回 unlisten），并覆盖「resolve 发生在卸载之后」的时序。同步 resolve 的 mock 会让 `08-22-state-logic-port` AC5 要防的泄漏形态无法暴露（其 `design.md` §7）。

验证：`bun run test:smoke`（AC5）退出码 0。

### 批次 5：资产复用验证与判定清单

- [ ] `git diff --stat src/api src/types` 确认为空（AC6）。非空项登记为独立缺陷，不在本任务修改。
- [ ] `src/utils` 31 个文件逐个判定，`utils-disposition.md` 落盘（AC7）。
- [ ] `src-tauri` 下 `cargo check`（AC8）。

### 批次 6：路径映射表

- [ ] 按 `design.md` §8 的格式产出 `path-mapping.md`，216 行，无空缺（AC9）。
- [ ] 20 个移交 `08-22-platform-unify` 的文件标注收敛方式。
- [ ] 表内每个新路径的归属子任务与父任务 `prd.md` 的 18 子任务范围表一致。

### 批次 7：zod 试点

- [ ] 按 `design.md` §9 完成试点，结论落盘。

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

- [ ] AC1–AC9 全部满足。
- [ ] `utils-disposition.md`、`path-mapping.md`、插件选择测量数据、zod 试点结论四项落盘。
- [ ] `package.json` 无 `vue-tsc` 与 4 个 Vue 系 ESLint 插件（AC3、AC4 的检查项）。
- [ ] 与 `08-22-dep-upgrade` 共同满足父任务的基座门（父任务 `implement.md` §4）。

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
