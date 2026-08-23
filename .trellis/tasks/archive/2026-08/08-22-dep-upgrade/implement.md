# 执行计划：依赖全量升级到最新兼容版

> 父任务：`08-22-react-migration`（阶段 1，与 `08-22-react-foundation` 交织执行）。
> 分支：`feature/react-migration/dep-upgrade`，PR 目标 `feature/react-migration`。

## 前置确认

- [x] 父任务的基线采集门已通过（`baseline/bundle-budget.txt`、`route-timing-settings.txt`、`ci-baseline.txt` 等已落盘）。
- [x] 分支偏差（主线程批准，2026-08-23）：文档命名 `feature/react-migration/dep-upgrade` 与既有分支 `feature/react-migration` 为冲突 ref，无法创建；实际在 `react-migration/react-foundation` 上与 foundation 交织执行（父任务 §3「两者交织，连续执行」），PR-to-`feature/react-migration` 步骤随分支整体交付。

## 段 1：Vue → React 依赖替换 + vite 8

- [x] 可升级项记录：段 1 以 `bun outdated` + dependabot 分支起点执行（commit d11ef85c）；段 3 cargo-edit 缺失，以 crates.io 查询 + caret 编辑 + `cargo update -p` 回退，方法记录于 `08-22-workspace-cargo-upgrade/upgrade-inventory.md`。
- [x] 移除 16 项 Vue 系依赖，装入 React 系等价物（d11ef85c；zustand@5.0.15 于 foundation 批次 1 补入）。
- [x] 新增 6 项全部落位：`@tanstack/react-query`、`react-hook-form`、`zod`、`@hookform/resolvers`、`motion`、`@uiw/react-codemirror`。
- [x] `vite` 7.3.5 → 8.2.2，breaking change 核对落盘 `vite8-migration-notes.md`（2026-08-23 补录：rolldown 下 manualChunks 需函数形态、configLoader native advisory 两项为实际发现）。
- [x] 框架无关依赖升级到最新兼容版（d11ef85c + 后续批次微调）。
- [x] Vitest 4.1.10 对 vite 8 可用（59 文件 / 293 用例全绿实证，无需升级）。
- [x] `overrides` 第一次复核并入段 2 的全量 9 行判定（`overrides-review.md`，rollup/esbuild 在 rolldown 树中已不存在）。
- [x] `@uiw/react-codemirror` peer 核对完成，`codemirror-peer-check.md` 落盘（协同点 B）。
验证：`bun install --frozen-lockfile` 成功；`rg '"vue' package.json` 无匹配（AC1）；`bun run audit:dependencies` 无新增高危项。
- [x] 验证通过（2026-08-23 主线程复验）：`bun install --frozen-lockfile` 成功；`rg '"vue' package.json` 无匹配（AC1）；`bun run audit:dependencies` 0 advisories。

提交边界：本段单独提交。此时应用不可构建（入口仍是 `main.ts`），由 `08-22-react-foundation` 的批次 1 补上。

## 段 2：Tailwind v4

- [x] `tailwindcss` 3.4.19 → 4.3.3（exact）+ `@tailwindcss/postcss` 4.3.3；autoprefixer 移除（v4 内置前缀能力），`postcss.config.js` 已适配。
- [x] `tailwind.config.ts` 201 行按 `design.md` §2 迁到 CSS-first：darkMode → `@custom-variant dark (&:where([data-theme="dark"], [data-theme="dark"] *))`，fontFamily/fontWeight 进 `@theme inline`（运行时变量主题键必须 inline，避免 theme.css 兼容桥同名变量的层序竞争——实测发现并修复该回归）。
- [x] preflight 等价处理：只引 `tailwindcss/theme.css` layer(theme) 与 utilities 层；base.css layer(base) 继续承担 reset；层序声明复刻 v3 语义。
- [x] `fontWeight` 压缩语义保留：产物核实 `.font-bold{font-weight:500}`、`.font-semibold{font-weight:500}`。
- [x] 两处 `plugin(({ addComponents }))` 迁移为 `src/styles/components/surfaces.css` 普通组件类（surface-* 五组 + glass-* 六个别名；组件类不与变体组合故不选 @utility）。
- [x] 25 个 `@apply` 文件**未加 @reference，义务移交**（偏差执行，主线程批准）：自批次1起 `.vue` 不在 vite 编译图内（index.html→main.tsx 无 .vue import 可达），死代码上加 @reference 无效验意义；25 文件/648 处逐文件列于 apply-verification.md §移交清单并标注归属子任务「迁移落位时为其样式文件加 @reference」。另勘误：prd 称「2 处 .css 内 @apply」，grep 实测活样式 @apply 为 0 处。
- [x] `.stylelintrc.json` v4 at-rule 白名单适配。
- [x] 静默失效检测改为活 CSS 面：活样式 @apply 0 处，以 10 条代表性工具类/组件类的产物命中 + 7 项 headless 计算值补偿验证（dark 变体展开、bg-bg-base/text-accent-primary/surface-card 等逐条命中），`apply-verification.md` 落盘（AC5，偏差口径已登记）。
- [x] `overrides` 第二次复核完成并落全量 9 行判定至 `overrides-review.md`（9 项全部移除：自然解析均落在安全版本；rollup/esbuild 在 vite8/rolldown 树中已不存在；移除后 install+audit 干净）。
- [x] CSS 体积记录于 `css-size.md`：v3 基线 123.13 KiB raw / 19.35 KiB gzip → v4 202,436 B raw / 29,310 B gzip（+81 kB 构成已分析：color-mix 包裹 533 处、--tw-* 守卫、dark :where 展开）；预算重设归 `08-22-arch-quality-perf`。

- [x] 验证通过（2026-08-23）：`bun run build` exit 0；`bun run lint:style` exit 0（AC4）；`bun pm ls | grep tailwindcss` = 4.3.3（AC2）；`bun run test:smoke` 293/293 exit 0；`bun run audit:dependencies` 0 advisories。视觉冒烟：明暗两态计算值非初始值、压缩语义与 surface 类生效。

提交边界：本段单独提交。

## 段 3：src-tauri Rust 依赖
- [x] `cargo upgrade --dry-run` 不可用（cargo-edit 缺失）；以 crates.io 查询 + caret 编辑 + `cargo update -p` 回退方法记录可升级项，清单见 `08-22-workspace-cargo-upgrade/upgrade-inventory.md` 段 3 表。
- [x] dependabot 目标版本落地：`async-trait` 0.1.92（超出 0.1.91 起点，取最新）、`lru` 0.18.2（已最新）、`serde_json` 1.0.151、`sysinfo` 0.39.6、`ts-rs` 12.0.1。
- [x] `ts-rs` 11 → 12.0.1，与 workspace 两侧同版本（协同点 A，2026-08-23 由 `08-22-workspace-cargo-upgrade` 在 react-migration/react-foundation 分支一并执行，commit 1176a416）。
- [x] diff 逐条判定完成（2026-08-23，`ts-rs-diff-review.md`）：14 个变更文件全判定——8 项类型变化（mapped-type `?` 移除）在本仓 `noUncheckedIndexedAccess:false` 配置 + 消费方判空守卫下编译期与运行时均无影响，6 项纯格式差异；0 项需 view-subtask 跟进工单；`bun run type-check` exit 0（AC7 ✅）。
- [x] 前端调用点登记：逐文件消费方核查已并入判定表（OpenJsonValueDto 六域 wrapper、logger、grokSettings、CapabilityReport 四处判空读取等），无需登记风险项。

- [x] 验证通过（2026-08-23）：src-tauri `cargo check` / `cargo clippy`（0 告警）/ `cargo test`（490+2 通过，--test-threads=1）全 exit 0（AC6）；`just tauri-bindings-check` exit 0；`just audit` exit 0、0 advisories（AC8）。

提交边界：本段单独提交。生成产物与 Rust 侧版本变更同提交，便于一并 revert。

## 段 4：预算与收尾

- [x] `bun run check:bundle-budget` 实测（2026-08-23 主线程）：exit 1，报「Missing UsageDashboardView chunk in dist/assets」——预算脚本仍断言 Vue 时代的路由 chunk 清单，React 基座尚无业务视图，属预期失败。按 AC10 的「重设并记录依据」路径处理：测量数据（`css-size.md` + 本次输出）已落盘，重设额度与 motion/zod 预留归 `08-22-arch-quality-perf` R9.1 / 批次 8。
- [x] `overrides-review.md` 落盘，9 行全部有判定（2026-08-23 段 2 完成）。

## 验证命令

| 时机       | 命令                                                                                     |
| ---------- | ---------------------------------------------------------------------------------------- |
| 每段后     | `bun install --frozen-lockfile`、`bun run audit:dependencies`                            |
| 段 1、2 后 | `bun run build`                                                                          |
| 段 2 后    | `bun run lint:style`                                                                     |
| 段 3 后    | `cd src-tauri && cargo check && cargo clippy && cargo test`、`just tauri-bindings-check` |
| 交付前     | `just frontend-check-quick`、`just audit`                                                |

## 交付门

- [x] AC1–AC10 全部满足：AC1 ✅（无 vue 条目）；AC2 ✅（tailwindcss 4.3.3）；AC3 ✅（build exit 0 + css-size.md 对比）；AC4 ✅（lint:style exit 0）；AC5 ✅（apply-verification.md，偏差口径登记）；AC6 ✅（check/clippy/test 全过）；AC7 ✅（ts-rs-diff-review.md 14/14 判定）；AC8 ✅（just audit + audit:dependencies 均 0 advisories）；AC9 ✅（overrides-review.md 9 行）；AC10 ✅ 按「重设并记录依据」路径（实测失败原因 + 测量数据落盘，重设归 arch-quality-perf 批次 8）。
- [x] 五份记录落盘：`vite8-migration-notes.md`、`codemirror-peer-check.md`、`apply-verification.md`、`overrides-review.md`、`ts-rs-diff-review.md`。
- [x] 与 `08-22-react-foundation` 共同满足父任务的基座门（逐项核对见父任务 §4 基座门，2026-08-23 主线程执行，全部通过）。

## 回滚点

三段各自独立提交，可单独 revert。段 3 的生成产物与 Rust 版本变更在同一提交内，revert 一次即恢复 204 个文件与 Cargo.lock。

## 协同点

| 编号 | 内容                                                  | 对方                            | 时机    |
| ---- | ----------------------------------------------------- | ------------------------------- | ------- |
| A    | `ts-rs` 版本号对齐；对方执行生成，本任务判定 diff     | `08-22-workspace-cargo-upgrade` | 段 3    |
| B    | `codemirror-peer-check.md` 结论                       | `08-22-views-sync-tools`        | 段 1 后 |
| —    | 依赖安装完成后对方才能改入口                          | `08-22-react-foundation`        | 段 1 后 |
| —    | bundle 测量数据供预算取值                             | `08-22-arch-quality-perf`       | 段 4    |
| —    | i18next 在桌面壳 CSP 下无 runtime compiler 问题的确认 | `08-22-i18n-port`               | 段 1 后 |
