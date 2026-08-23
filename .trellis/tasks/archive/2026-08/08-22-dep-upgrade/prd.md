# 依赖全量升级到最新兼容版

> 父任务：`08-22-react-migration`

## Goal

将 `ccr-ui` 的 npm 依赖与 `ccr-ui/src-tauri` 的 Rust 依赖升级到最新兼容版，完成 Vue 系依赖到 React 系等价物的替换，并复核 9 项 `overrides` pin。

## Scope

### 需替换的 Vue 绑定依赖

选型已在父任务 `design.md` 第 1 节定稳。版本为 2026-08-22 查询的最新版。

| 现依赖 | 当前版本 | 替换为 | 目标版本 |
|---|---|---|---|
| `vue` | ^3.5.26 | `react` + `react-dom` | 19.2.8 |
| `vue-router` | ^4.6.4 | `react-router` | 8.3.0 |
| `pinia` | ^2.3.1 | `zustand` | 5.0.15 |
| `vue-i18n` | ^9.14.5 | `react-i18next` + `i18next` | 17.0.12 / 26.4.0 |
| `@iconify/vue` | ^5.0.0 | `@iconify/react` | 6.0.2 |
| `@tanstack/vue-virtual` | ^3.13.18 | `@tanstack/react-virtual` | 3.14.10 |
| `vue3-apexcharts` | ^1.10.0 | `react-apexcharts` | 2.1.1 |
| `@vitejs/plugin-vue` | ^6.0.7 | `@vitejs/plugin-react`（或 SWC 变体） | 最新 |
| `vue-tsc` | ^2.2.12 | `typescript` 自带 `tsc` | — |
| `eslint-plugin-vue` | ^10.6.2 | React ESLint 插件集（含 `react-hooks`） | 最新 |
| `vue-eslint-parser` | ^10.4.1 | 移除 | — |
| `@vue/eslint-config-typescript` | ^14.7.0 | `typescript-eslint`（已在依赖中） | — |
| `@intlify/eslint-plugin-vue-i18n` | ^4.5.1 | i18n lint 等价方案（见 `08-22-i18n-port` R9） | — |
| `stylelint-config-recommended-vue` | ^1.6.1 | 移除 | — |
| `postcss-html` | ^1.8.1 | 逐项判定是否仍需要 | — |
| `@vue/test-utils`（devDep 未显式列出） | — | `@testing-library/react` | 16.3.2 |

### 需新增的依赖

| 依赖 | 版本 | 用途 | 决策位置 |
|---|---|---|---|
| `@tanstack/react-query` | 5.101.4 | IPC 命令（334 base / 342 含 Windows）的服务端数据层 | design.md §4 |
| `react-hook-form` | 7.86.0 | 353 处 `v-model` 的转换目标 | design.md §7 |
| `zod` | 4.4.3 | 表单校验 schema | design.md §7 |
| `@hookform/resolvers` | 5.9.1 | zod 接入 react-hook-form | design.md §7 |
| `motion` | 13.1.1 | 进出场动画，替代 12 处 Vue `Transition` | design.md §9 |
| `@uiw/react-codemirror` | 4.25.11 | CodeMirror 6 的 React 桥接 | design.md §10 |

`zod` 与 `motion` 均增加 bundle 体积。`08-22-arch-quality-perf` 的 bundle 预算需为两者显式预留额度（见父任务 design.md §12.2）。

**@uiw/react-codemirror 的依赖冲突前置核对**：该包自带一组 `@codemirror/*` 依赖版本范围，与现有 9 个直接依赖可能冲突。CodeMirror 6 对 `@codemirror/state` 的多实例敏感，若产生重复实例需通过 `overrides` 收敛到单一版本。该核对是本任务的交付项，结论供 `08-22-views-sync-tools` 使用。


### 需升级并保留的框架无关依赖

`@codemirror/*`（9 包）、`apexcharts` ^5.3.6、`dompurify` ^3.4.13、`fuse.js` ^7.3.0、`ansi_up` ^6.0.6、`@iconify-json/solar` ^1.2.5、`@tauri-apps/api` 2.11.0、`@tauri-apps/cli` 2.11.2、`typescript` ^5.9.3、`vitest` ^4.1.10、`@vitest/coverage-v8` ^4.1.10、`playwright` ^1.60.0、`jsdom` ^26.1.0、`postcss` ^8.5.23、`autoprefixer` ^10.5.0、`stylelint` ^17.12.0、`stylelint-config-standard` ^40.0.0、`@types/node` ^22.19.19、`globals` ^16.5.0、`eslint` ^9.39.4、`@eslint/js` ^9.39.4、`@typescript-eslint/parser` ^8.60.1、`typescript-eslint` ^8.60.1。

`vite` ^7.3.5 → 8.2.2 为跨主版本升级，breaking change 需逐项核对，与 React 插件切换同步进行。

### Tailwind CSS 3.4.19 → v4

目标版本 4.3.3。

- 配置模型从 `tailwind.config.ts`（201 行）改为 CSS-first `@theme`。
- `corePlugins.preflight: false` 的等价处理（当前自带 reset 在 `src/styles/base.css`）。
- 1 处自定义 `plugin(({ addComponents }) => ...)`（`tailwind.config.ts:139`、`:180`）迁移。
- `fontFamily` / `fontWeight` 的 theme 覆盖迁移。`fontWeight` 当前将 8 个档位压缩到 400/500 两值，此语义需保留。
- 648 处 `@apply`（集中在 25 个文件）+ 2 处 `.css` 内 `@apply`：v4 下在组件级样式文件中使用 `@apply` 需 `@reference`，逐文件处理。
- `postcss.config.js` 与 `.stylelintrc.json` 适配。

### overrides pin 复核

`fast-uri` 3.1.5、`flatted` 3.4.2、`js-yaml` 4.3.1、`nanoid` 3.3.18、`picomatch` 4.0.4、`postcss` `$postcss`、`rollup` 4.61.0、`esbuild` 0.28.1、`ws` 8.21.0。逐项判定：升级后仍需要的保留并记录原因，上游已修复的移除。

### src-tauri Rust 依赖

`ccr-ui/src-tauri/Cargo.toml` 的依赖升级到最新兼容版。origin 上有对应 dependabot 分支：`async-trait` 0.1.91、`lru` 0.18.1、`serde_json` 1.0.151、`sysinfo` 0.39.6、`ts-rs` 12.0.1。

`ts-rs` 升级影响 `src/types/generated` 下 204 个文件的生成结果，需重新生成并比对差异。

## Requirements

- R1 `package.json` 无任何 Vue 系依赖条目。
- R2 Tailwind 主版本为 4，配置采用 CSS-first `@theme` 模型。
- R3 25 个使用 `@apply` 的文件在 v4 下样式生效，由 stylelint 规则或测试断言保护。
- R4 `tailwind.config.ts` 的 `fontWeight` 压缩语义（8 档位映射到 400/500）在 v4 模型下保留。
- R5 9 项 `overrides` 逐项给出保留或移除的判定与依据。
- R6 `src-tauri` Rust 依赖升级后 `cargo check`、`cargo clippy`、`cargo test` 通过。
- R7 `ts-rs` 升级后重新生成的 204 个类型文件与升级前逐个比对，差异逐条判定。
- R8 `just audit` 与 `bun run audit:dependencies` 无新增高危项。

## Acceptance Criteria

- [ ] AC1 `rg '"vue' ccr-ui/package.json` 无匹配。
- [ ] AC2 `bun pm ls | rg tailwindcss` 显示 4.x。
- [ ] AC3 `bun run build` 成功，产物中 Tailwind 生成的 CSS 体积记录并与升级前对比。
- [ ] AC4 `bun run lint:style` 退出码 0。
- [ ] AC5 25 个 `@apply` 文件的样式生效验证记录落盘，无静默失效项。
- [ ] AC6 `cd src-tauri && cargo check && cargo clippy && cargo test` 全部通过。
- [ ] AC7 `src/types/generated` 重新生成后的 diff 逐条判定，判定记录落盘。
- [ ] AC8 `just audit` 退出码 0，`bun run audit:dependencies` 无新增高危项。
- [ ] AC9 `overrides` 复核表落盘，9 项全部有判定。
- [ ] AC10 `bun run check:bundle-budget` 通过，或预算基线按新框架重设并记录依据。

## 前置与后续

- 前置：`08-22-react-foundation`（交织执行，两者需连续完成）。
- 后续：`08-22-design-system`。

## Out of Scope

- 根 workspace 下 12 个 Rust crate 的 Cargo 依赖升级。范围待确认，见父任务 `prd.md` 的 Q1。
- `ccr-vscode` 依赖升级。
- `docs/` VitePress 依赖升级。
- token 迁移与 shadcn/ui 接入（属 `08-22-design-system`）。

## Notes

- Tailwind v4 的 `@theme inline` 模型将变量与工具类合并，具体映射由 `08-22-design-system` 完成。本任务只完成版本升级与配置模型切换，保证现有样式不回退。
- 升级顺序建议：先 Vue→React 依赖替换，再 Tailwind v4，最后 src-tauri Rust。每步单独提交以便回滚。
