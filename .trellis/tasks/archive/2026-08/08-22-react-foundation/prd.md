# React 基座与构建管线

> 父任务：`08-22-react-migration`

## Goal

在 `ccr-ui` 中建立可运行的 React 基座与配套构建管线，并验证框架无关的三层资产可原样复用，为后续 12 个迁移子任务提供落地平面。

## Scope

**建立**

- Vite + React 应用入口，替换 `src/main.ts` 与 `src/App.vue` 的 Vue 挂载路径。
- 路由库与状态库接入（选型在 `design.md` 中决定）。
- ESLint / Prettier / Stylelint 配置从 Vue 规则集切到 React 规则集。
- 类型检查从 `vue-tsc --noEmit` 切到 `tsc --noEmit`。
- Vitest 配置（`vitest.smoke.config.ts`）适配 React 测试环境。
- `package.json` scripts 的 `type-check`、`lint`、`lint:style`、`test:smoke` 保持命令名不变，实现替换。

**复用验证**

| 资产 | 数量 | 验证内容 |
|---|---|---|
| `src/api/**` | 57 文件 | 纯 `invoke()` wrapper，无框架依赖，可原样导入 |
| `src/types/**` | 231 文件（含 204 个 ts-rs 产物） | 纯类型声明，可原样导入 |
| `src/utils/**` | 31 文件 | 逐个判定：纯逻辑模块原样复用，含 Vue 响应式引用的模块登记到 `08-22-shell-port` |

**框架无关依赖保留清单**

`@codemirror/*`（9 包）、`apexcharts`、`dompurify`、`fuse.js`、`ansi_up`、`@tauri-apps/api`、`tailwindcss`。

## Requirements

- R1 React 基座可运行。两项分开验证：纯 Web 预览下最小页面渲染成功；桌面运行时下成功调用至少一个 `src/api` 下的 IPC wrapper 并显示返回值。
- R2 `bun run type-check` 在基座范围内通过，`vue-tsc` 从 devDependencies 移除。
- R3 `bun run lint` 使用 React 规则集通过，Vue 系 ESLint 插件（`eslint-plugin-vue`、`vue-eslint-parser`、`@vue/eslint-config-typescript`、`@intlify/eslint-plugin-vue-i18n`）移除。
- R4 `bun run test:smoke` 可在 React 环境下运行至少一个新写的 smoke 测试。
- R5 `src/api`、`src/types` 内容不做修改，仅验证可导入。修改需求登记为独立缺陷。
- R6 `src/utils` 的 31 个文件完成逐个判定，产出「原样复用 / 需接线」两个清单。
- R7 `src/api/tauri.ts` 冻结门面的边界约定不变（见 `.trellis/spec/ccr-ui/frontend/api-facade-boundary.md`）。

## Acceptance Criteria

- [ ] AC1 `bun run dev`（纯 Vite，无 Tauri 壳）启动后页面渲染成功，控制台无报错。依赖 `invoke()` 的部分在此环境下不可用，其报错不计入本项。
- [ ] AC2 在**桌面运行时**下（`bun run tauri:dev`，或 `just dev` / `bun run build:desktop` 产物），页面上至少一个按钮触发 `src/api` 下的真实 IPC 调用并显示返回值。`package.json` 的 `dev` 脚本是 `vite`，不提供 Tauri IPC 环境，因此 AC1 的环境不能用于本项。smoke 测试中的 `invoke` mock 也不能替代本项。
- [ ] AC3 `bun run type-check` 退出码 0，`package.json` 无 `vue-tsc`。
- [ ] AC4 `bun run lint` 退出码 0，`package.json` 无 Vue 系 ESLint 插件。
- [ ] AC5 `bun run test:smoke` 至少 1 个 React smoke 测试通过。
- [ ] AC6 `src/api` 与 `src/types` 的 git diff 为空。
- [ ] AC7 `src/utils` 判定清单落盘到本任务目录，31 个文件全部归类，无未判定项。
- [ ] AC8 `cargo check` 在 `src-tauri` 下通过（确认基座切换未破坏 Tauri 侧构建配置）。
- [ ] AC9 旧路径 → 新路径映射表落盘，覆盖 185 个 `.vue` 文件与 31 个 `utils` 文件，无未映射项。

## 前置与后续

- 前置：无。本任务是迁移的第一步。
- 与 `08-22-dep-upgrade` 交织：React 版本与 Tailwind v4 的选择同时决定基座形态，两个任务需连续执行。
- 后续：`08-22-design-system`。

## Out of Scope

- 任何业务视图或组件的迁移。
- Tailwind v4 升级本身（属 `08-22-dep-upgrade` 与 `08-22-design-system`）。
- i18n 运行时接入（属 `08-22-i18n-port`）。
- 路由表填充（属 `08-22-shell-port`，本任务只接入路由库并验证一条路由可用）。

## Notes

选型已在父任务 `design.md` 第 1 节定稳，本任务不再重新决策：

| 领域 | 选定 | 版本 |
|---|---|---|
| UI 运行时 | React | 19.2.8 |
| 构建 | Vite | 8.2.2 |
| 路由 | React Router | 8.3.0 |
| 状态 | Zustand | 5.0.15 |
| 服务端数据 | TanStack Query | 5.101.4 |
| 表单 / 校验 | react-hook-form + zod + @hookform/resolvers | 7.86.0 / 4.4.3 / 5.9.1 |
| 测试 | @testing-library/react | 16.3.2 |

**目录结构：全量按域聚合 `features/`**（父任务 `design.md` §2）。分层与依赖方向由 `08-22-arch-quality-perf` 落为 ESLint 规则。

**本任务追加交付项：旧路径 → 新路径映射表。** 因目录结构为全量重组，185 个文件同时更换框架与位置，AC11 的逐屏比对与 `git log --follow` 追溯能力下降。缓解措施（父任务 `design.md` §12.1）：

- 本任务产出完整映射表并落盘，供 `08-22-regression-release` 作为前置输入。
- 文件基名保持不变（`CodexSettingsView.vue` → `CodexSettingsView.tsx`），便于按名检索。
- 每个文件的移动与改写在同一提交内完成，提交粒度为单文件或单个紧密相关的小组。

`src/utils` 的 31 个文件判定清单（R6）与 `08-22-shell-port` 的接线范围直接衔接。已知需接线的 11 个文件：`windowChrome`、`tauriWindow`、`nativeWindowAppearance`、`themeBootstrap`、`fontPreferences`、`startupRecovery`、`perfTelemetry`、`runtimeState`、`tauriRuntime`、`errorHandler`、`logger`。

zod 用于 IPC 返回值运行时校验的范围在本任务试点后确定，不预先承诺（父任务 `design.md` §15）。现状为 ts-rs 只提供编译期类型，无运行时校验。试点不回填 57 个既有 wrapper。

