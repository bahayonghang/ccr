# ccr-ui 前端框架迁移 Vue 3 → React 与依赖全量升级

> 父任务。持有需求集合、任务地图与跨子任务验收标准。本任务本身不承担实现工作。

## Goal

将 `ccr-ui/src` 从 Vue 3 全量迁移到 React，同步将前端与 `ccr-ui/src-tauri` 依赖升级到最新兼容版，并在 React 侧重建设计体系，降低后续修改样式的成本。

## 决策依据

用户目标为「降低开发者改样式的成本」。评估给出两条路径：

| 方案 | 内容                                                              | 工程日  |
| ---- | ----------------------------------------------------------------- | ------- |
| A    | 保留 Vue，接入 Reka UI + shadcn-vue，升级 Tailwind v4，收口 token | 47–56   |
| B    | 迁移到 React，并在 React 侧完成方案 A 的全部内容                  | 220–229 |

用户在了解 4.2–4.5 倍成本差与迁移期风险后选定方案 B。本任务按方案 B 执行。

选定方案 B 后追加的三项范围（均由用户确认）：

| 追加项                     | 内容                                                                  | 净增工程日 |
| -------------------------- | --------------------------------------------------------------------- | ---------- |
| workspace Cargo 依赖升级   | 13 个 crate 的共享依赖，含 `ts-rs` 11 → 12                            | 5–8        |
| 架构约定、质量门与性能基线 | 分层强制、规模与复杂度 lint、覆盖率门、五项性能基线、React 重渲染纪律 | 8–12       |
| 跨平台功能面统一           | 15,672 行重复实现收敛为 6,000–7,500 行统一层                          | 8–14       |

修正后总量 **241–263 工程日**。

跨平台统一的净增工程日低于最初 25–35 日的估计。原因：`BaseSlashCommands` + 平台 config + 薄壳的模式已在仓库内验证，不需要设计新架构，只需扩展到剩余功能面。

## 测量基准

迁移前的仓库状态。所有子任务的规模数字以此为准。

**SFC 行数分布**（185 个 `.vue` 文件，合计 85,274 行）

| 区块         | 行数   |
| ------------ | ------ |
| `<template>` | 35,726 |
| `<script>`   | 24,235 |
| `<style>`    | 24,434 |

**其他资产**

| 项目                                                       | 数量                                                   |
| ---------------------------------------------------------- | ------------------------------------------------------ |
| `src/*.ts` 文件 / 行数                                     | 415 / 51,132                                           |
| `src/types/generated`（ts-rs 产物）                        | 204 文件 / 904 行                                      |
| `src/api` / `src/utils` / `src/composables` / `src/stores` | 57 / 31 / 35 / 10 文件                                 |
| composables 行数 / stores 行数                             | 6,894 / 2,531                                          |
| `src/styles/*.css` 行数 / `tokens.css` 变量数              | 4,026 / 448                                            |
| 路由条数                                                   | 75                                                     |
| i18n key 数 / 词条文件体积                                 | 4,164 / 417.7 KB                                       |
| smoke 测试数（挂载组件 / 读源码文本）                      | 122（63 / 19）                                         |
| 前端契约文档                                               | 16 份                                                  |
| Tauri IPC 命令                                             | 334（base）/ 342（含 Windows 专属 8 条）/ 271（typed） |

**Vue 特有 API 使用点**

| API                    | 出现次数 |
| ---------------------- | -------- |
| `v-model`              | 353      |
| slot                   | 87       |
| `nextTick`             | 52       |
| `defineAsyncComponent` | 22       |
| `v-html`               | 14       |
| `Transition`           | 12       |
| `Teleport`             | 8        |
| `defineExpose`         | 8        |
| `KeepAlive`            | 4        |
| `provide` / `inject`   | 1 / 1    |

**样式硬编码值**

| 类型               | `.vue`                  | `.css` |
| ------------------ | ----------------------- | ------ |
| px 字面量          | 1,639                   | 290    |
| `rgba()` / `rgb()` | 932                     | —      |
| hex 颜色           | 20                      | 102    |
| `var(--)` 引用     | 4,097                   | —      |
| `@apply`           | 648（集中在 25 个文件） | 2      |

**弹层现状**：33 个文件引用 `BaseModal`，13 个文件自行实现 `fixed inset-0` 弹层。`Dropdown` / `Tooltip` / `Popover` / `Tabs` / `Accordion` / `Combobox` 命名文件各 0 个，对应交互由手写 div 承担。

**i18n 可移植性**：词条未使用 linked message（`@:`），未使用复数管道（`|`）。词条数据可直接移植，仅调用点需要转换。

**两项计数的取数方式**（避免沿用过时数字）：

- IPC 命令数取自 `ccr-ui/src/api/generated/command-manifest.json`（`schema_version` 2）的 `base_command_count` / `windows_command_count` / `typed_command_count`。该文件是 Rust 测试 `commands::handler_registry::tests::command_inventory_document_matches_registry` 比对的生成产物之一，随注册表变化。先前工件记录的「141+」已过时。
- i18n key 数取两个 locale 的**叶子 key** 计数（`zh-CN.ts` 与 `en-US.ts` 递归展开后各 4,164）。`zh-CN.keys.txt` 的 4,261 行是源码行数，不是 key 数。

## 跨平台重复

同类功能在多个平台各自独立实现的现状。这是当前代码库最大的架构问题，也是「改一处行为要改多处」的直接来源。

| 功能面        | 独立实现文件数 | 行数   |
| ------------- | -------------- | ------ |
| Settings      | 5              | 5,322  |
| Profiles      | 3              | 3,325  |
| MCP           | 4              | 2,664  |
| Commands      | 5              | 2,364  |
| Agents        | 3              | 2,305  |
| Auth          | 3              | 2,298  |
| Plugins       | 3              | 1,089  |
| Providers     | 1              | 577    |
| SlashCommands | 3              | 274    |
| 合计          | 29             | 20,218 |

20,218 行占 `views/` 总量 47,035 行的 43%。

**已有的统一模式**：仓库内已存在可用且已验证的模式，SlashCommands 三个平台已完成统一。

```
src/components/BaseSlashCommands.vue   507 行   统一实现
src/configs/slashCommands.ts                    每平台一个 config 对象
src/views/SlashCommandsView.vue         18 行   薄壳
src/views/GeminiSlashCommandsView.vue   27 行   薄壳（+ hide-chrome props）
```

`src/views/generic/` 另有 5 个部分抽象：`AgentsView`（725）、`AgentDetailView`（481）、`SystemPromptsView`（655）、`PlatformMcpView`（407）、`PlatformPluginsView`（367）。

**实际需统一的量**：从 20,218 行中扣除已统一的 SlashCommands（274）、单一实现的 Providers（577）、应用级设置 `AppSettingsView`（1,399）、跨平台 MCP 管理器 `McpManagerView`（523）与 generic 层自身（1,499），剩余 **15,672 行 / 20 个文件** 需收敛。预估统一后 6,000–7,500 行。详见子任务 `08-22-platform-unify`。

**质量门缺口**：`eslint.config.js`（133 行）只有 `@typescript-eslint/no-explicit-any: error`，无文件规模、复杂度、嵌套深度、导入边界规则。`vitest.smoke.config.ts` 无覆盖率阈值，`@vitest/coverage-v8` 已在依赖中但未设门。`src/api/tauri.ts` 的冻结门面边界只有文档与 smoke 测试保护，无静态强制。

**性能现有基础设施**：`scripts/check-bundle-budget.mjs`、`scripts/measure-vite-route.mjs`、`scripts/warm-vite-deps.mjs`、`src/utils/perfTelemetry.ts`、三层 CSS 加载策略（`shell-critical.css` / `deferred-decorations.css` / `deferred-interactive.css`）、`corePlugins.preflight: false`。启动性能在本仓库已是显式关注项，迁移不得回退。

## Requirements

### R1 框架迁移

- R1.1 `ccr-ui/src` 下全部 185 个 `.vue` 组件迁移为 React 组件，Vue 运行时依赖从 `package.json` 移除。
- R1.2 353 处 `v-model` 展开为受控属性与回调对；87 处 slot 转为 children 或 render props。
- R1.3 `defineAsyncComponent`（22）、`Teleport`（8）、`Transition`（12）、`KeepAlive`（4）、`defineExpose`（8）改用 React 等价机制。`nextTick`（52 处）逐点判定，无等价物的改为显式副作用时序。
- R1.4 `src/api`（57 文件）、`src/types`（231 文件，含 204 个 ts-rs 产物）、`src/utils` 中的纯逻辑模块保持原样复用，不做重写。
- R1.5 `ccr-ui/src-tauri` 的 IPC 命令签名与 Tauri Event 名称不变（334 base / 342 含 Windows）。

### R2 依赖升级

- R2.1 `ccr-ui/package.json` 的 dependencies 与 devDependencies 升级到最新兼容版，Vue 系依赖替换为 React 系等价物。
- R2.2 Tailwind CSS 从 3.4.19 升到 v4，采用 CSS-first `@theme` 配置模型。
- R2.3 `ccr-ui/src-tauri` 的 Rust 依赖升级到最新兼容版。
- R2.3b 根 workspace 的共享 Cargo 依赖升级到最新兼容版，覆盖 13 个 crate。含 `ts-rs` 11 → 12 及随之重新生成的 204 个 TypeScript 类型绑定。
- R2.4 `package.json` 的 9 项 `overrides` pin（`fast-uri`、`flatted`、`js-yaml`、`nanoid`、`picomatch`、`postcss`、`rollup`、`esbuild`、`ws`）逐项复核，升级后仍需要的保留，已被上游修复的移除。
- R2.5 `just audit` 与 `bun run audit:dependencies` 无新增高危项。

### R3 设计体系重建

- R3.1 `tokens.css` 的 448 个变量按两层结构落位：可切换的语义变量留普通 CSS 变量（挂在 `[data-theme]` / `[data-flavor]` / `[data-accent]` 选择器下），Tailwind namespace 映射进 `@theme inline`，全主题同值的常量 token 进 `@theme`。工具类可直接引用，且运行时主题切换生效。变量名集合不变。
- R3.2 接入 shadcn/ui 原语，至少覆盖 Dialog、Popover、DropdownMenu、Tooltip、Tabs、Combobox、Select、Switch、Checkbox。
- R3.3 33 个 `BaseModal` 调用点与 13 个自实现弹层统一收口到同一 Dialog 原语。
- R3.4 2,571 处组件内硬编码样式值（1,639 px 字面量 + 932 `rgba()`）收回 token 层。
- R3.5 迁移后单个组件的局部样式行数不超过其模板行数，样式决策以 token 与原语变体表达。

### R4 主题配置域

- R4.1 保留 `data-theme` / `data-flavor` / `data-accent` 三层模型的语义。
- R4.2 `data-flavor` 与 `data-accent` 的值域可扩展，`themeBootstrap` 支持自定义 accent 输入。
- R4.3 明暗两套主题均满足现有对比度要求，reduced motion 降级路径保留。

### R5 质量门

- R5.1 122 个 smoke 测试重写并全部通过。19 个读源码文本的测试按新目录结构调整断言。
- R5.2 `.trellis/spec/ccr-ui/frontend/` 的契约文档重写，含 31.5 KB 的 `theme-token-contracts.md`。基线 16 份，迁移后 19 份。
- R5.3 `just ci` 全流程通过。
- R5.4 185 个界面逐屏完成行为与视觉比对，差异逐条记录并判定。

### R6 架构

- R6.1 定稳分层与依赖方向：视图 → 域逻辑 → API → 类型。禁止反向依赖，由 ESLint 导入规则强制。
- R6.2 `src/api/tauri.ts` 冻结门面边界由 lint 规则强制，新 wrapper 只能落 `src/api/domains/<domain>.ts`。当前该约定只有文档与 smoke 测试保护，无静态强制。
- R6.3 无循环依赖，由检查工具在 CI 中强制。
- R6.4 跨平台共享层的边界明确：`views/generic`（当前 5 文件 2,635 行）与平台专属视图的职责划分写入契约，共享层接口变更需通知全部消费方。
- R6.5 组件分层明确：原语（`src/ui/`，原 `src/components/ui/`）→ 复合组件 → 域组件 → 页面。原语不依赖域逻辑与 store。
- R6.6 状态归属明确：服务端数据、UI 瞬态、跨页面共享三类状态各有唯一承载位置，不混用。
- R6.7 迁移后目录结构在父任务 `design.md` 中定稳，全部 7 个视图子任务遵循同一结构。

### R7 质量门

- R7.1 单文件行数上限由 lint 强制。当前最大 `.vue` 为 1,744 行（`CommandsView.vue`），39 个根级视图平均 739 行。上限值在子任务 2c 中确定。
- R7.2 圈复杂度、嵌套深度、参数个数上限由 lint 强制。当前 `eslint.config.js`（133 行）无任何规模与复杂度规则。
- R7.3 `@typescript-eslint/no-explicit-any: error` 保留。新增 `no-unsafe-*` 系列规则，禁止隐式 any 逃逸。
- R7.4 React hooks 规则强制：`react-hooks/rules-of-hooks`、`react-hooks/exhaustive-deps` 为 error。后者直接对应 `computed` → `useMemo` 的依赖遗漏风险。
- R7.5 测试覆盖率门建立。当前 `vitest.smoke.config.ts` 无覆盖率阈值，`@vitest/coverage-v8` 已在依赖中但未设门。阈值在子任务 2c 中确定。
- R7.6 组件内样式行数上限由 lint 或检查脚本强制。当前 139 / 185 个组件带局部样式，合计 24,434 行，其中最大者超过其模板行数。
- R7.7 lint 规则以 error 级别加入 `bun run lint:ci`，不使用 warning 级别的软约束。

### R8 性能

- R8.1 React 组件级重渲染不得导致交互回退。以下场景逐项测量：配置大表单输入（353 处 `v-model` 的主要落点）、10,000 行虚拟列表滚动、实时日志流、图表数据更新、路由切换。
- R8.2 memo 与状态切分纪律写入约定并落为 lint 规则，避免在视图迁移阶段逐个补救。
- R8.3 启动耗时与首屏渲染耗时不高于迁移前基线。`perfTelemetry.ts` 的采集能力保留。
- R8.4 bundle 体积由 `check:bundle-budget` 约束。基线按 React 产物重设时须记录对比数据与重设依据。
- R8.5 路由级代码分割保留。当前 22 处 `defineAsyncComponent` 对应的懒加载边界在 React 侧等价保留，首屏加载模块集合不扩大。
- R8.6 关键 CSS 分离保留。当前 `shell-critical.css`、`deferred-decorations.css`、`deferred-interactive.css` 三层加载策略与 `corePlugins.preflight: false` 的启动优化意图在 Tailwind v4 下等价保留。
- R8.7 长时间运行无内存增长与监听器累积。

### R9 跨平台统一

- R9.1 Settings / Profiles / Auth / MCP / Agents / Plugins / Commands 七个功能面的重复实现收敛为统一层，覆盖 20 个文件、15,672 行。
- R9.2 统一层采用仓库已有模式：base 组件承载共性，平台 config 对象承载差异，视图层为薄壳。不引入新的抽象机制。
- R9.3 base 组件内禁止平台名称条件分支（形如 `if (platform === 'codex')`）。
- R9.4 差异普查先行，产出「平台 × 功能面」差异矩阵。矩阵的每一项在统一后可追溯到 config 字段或 props，无静默丢失。
- R9.5 统一后单个平台的视图文件行数不超过 100 行。
- R9.6 Auth 面的统一范围由差异普查决定。三种认证流程（Claude OAuth、Codex OAuth、Grok token）存在实质差异，若统一会引入超过差异项数的条件复杂度，该面可部分统一或不统一，判定需记录依据。
- R9.7 路由路径与页面划分不变。统一是视图实现层的收敛。

## Constraints

- C1 Vue 与 React 无法在同一组件树内共存。迁移期存在不可发版窗口，预计约 75 工程日（模板与脚本转换阶段）。本任务接受该窗口，不并行维护双壳。
- C2 迁移期 122 个 smoke 测试与 16 份契约全部失效。受影响范围为**前端到 IPC 命令的接线**：命令**清单**由 `just tauri-command-inventory-check`（Rust 测试 `commands::handler_registry::tests::command_inventory_document_matches_registry`）保护，该保护独立于前端测试，迁移期不失效。子任务 `test-contract-rebuild` 需尽早交付最小测试集以覆盖接线部分。
  另有一处易被忽略的保护缺口：`ccr-ui/tests/api-facade-boundary.smoke.test.ts` 遍历 `.ts` / `.mts` / `.vue`。迁移后组件为 `.tsx`，不扩后缀集合则该测试对 React 组件全面失效且静默通过。
- C3 `ccr-ui/src-tauri` 的 Rust 代码不做功能改动。若迁移过程需要新增 IPC 命令，走单独任务。
- C4 品牌与视觉方向遵循 `ccr-ui/CLAUDE.md` 的 Design Context：Anthropic-like 编辑式工作台、暖中性色表面、克制材质深度。禁止引入 `Neko` / `anime` / `purple-tech` / `guofeng` 分支。
- C5 `ccr-ui/src/api/tauri.ts` 是冻结的兼容门面。迁移不改变该门面的边界约定，新 wrapper 仍放 `src/api/domains/<domain>.ts`。
- C6 版本号来源仍是根 `Cargo.toml` 的 `workspace.package.version`。若迁移期间需要改版本，走 `just version-sync`。
- C7 分支 `feature/react-migration`，PR 目标分支 `dev`。

## Acceptance Criteria

跨子任务的整体验收。子任务各自的验收标准见其 `prd.md`。

- [x] AC1 `ccr-ui/package.json` 无 `vue`、`vue-router`、`pinia`、`vue-i18n`、`@iconify/vue`、`@tanstack/vue-virtual`、`vue3-apexcharts`、`@vitejs/plugin-vue`、`vue-tsc`、`eslint-plugin-vue`、`vue-eslint-parser`、`@vue/eslint-config-typescript`、`@intlify/eslint-plugin-vue-i18n`、`stylelint-config-recommended-vue` 条目。
- [x] AC2 `ccr-ui/src` 下 `.vue` 文件数为 0。
- [x] AC3 `just ci` 退出码 0，且其实际 recipe 依赖清单与 `justfile` 一致。迁移前基线为 13 步（version-sync → version-check → fmt → fmt-check → lint-strict → check-workspace → test → release → audit → ci-governance-check → tauri-bindings-check → frontend-check → vscode-ci）；`08-22-arch-quality-perf` 已纳入 `frontend-coverage`（2026-08-23，插在 `frontend-check` 之后），现为 14 步。核对以实际清单为准，不以本文件的数字为准。（根 `CLAUDE.md` 记录的 10 步描述已过时。）
- [x] AC4 122 个 smoke 测试全部通过，覆盖范围不低于迁移前（按被测组件与被测契约条目计数）。
- [x] AC5 Tailwind 版本为 v4。448 个变量分两集合落盘：稳定语义变量集合（普通 CSS 变量，随 `data-*` 切换）与 Tailwind namespace 映射集合（`@theme inline`）+ 常量 token（`@theme`）。工具类可引用，运行时切换生效，变量名集合不变。
- [x] AC6 组件内硬编码 px 字面量数量从 1,639 降到 0，`rgba()` 从 932 降到 0（图表与画布等确需字面量的场景逐个登记豁免）。证据：`ccr-ui/tests/hardcode-px-rgba.smoke.test.ts`（残留 31 条 == 豁免清单）。
- [x] AC7 `BaseModal` 与 13 个自实现弹层收口为单一 Dialog 原语，弹层的焦点陷阱、Esc 关闭、滚动锁定行为只有一处实现。
- [x] AC8 IPC 命令与全部 Tauri Event 名称在迁移前后一致，由 `api-facade-coverage` 类测试断言。命令名的数据源为 `ccr-ui/src/api/generated/command-manifest.json`；事件名的数据源为统一前端事件 inventory（全局桥接层 + 已声明的组件级局部事件）。
- [ ] AC9 Tauri 打包产物可启动，CSP、窗口 chrome、WAF WebView bypass、启动恢复四项行为验证通过。已有：打包启动截图、CSP 未放宽、chrome 六项、杀进程后可再启动。WAF 真实签到未做（凭据未提供）。四项合取，本条不勾选。`waiver-waf-ac6-ac9.md` 只记录跳过，不是关闭本条的授权。
- [x] AC10 前端契约文档重写完成，无残留 Vue 文件路径与 SFC 模式引用。基线 16 份，迁移后 19 份（`08-22-arch-quality-perf` 新增 `react-rerender-discipline.md`、`layering-contracts.md`；`08-22-platform-unify` 新增 `platform-surface-contracts.md`）。
- [x] AC11 185 个界面的逐屏比对记录归档，未判定项为 0。
- [x] AC12 `just audit` 与 `bun run audit:dependencies` 无新增高危项。
- [x] AC13 workspace 13 个 crate 的共享 Cargo 依赖升级清单落盘，无未判定项。`ts-rs` 重新生成的 204 个类型文件 diff 逐条判定。
- [x] AC14 分层依赖方向、门面边界、循环依赖三项由 lint 或检查工具以 error 级别强制，`bun run lint:ci` 退出码 0。
- [x] AC15 文件行数、圈复杂度、嵌套深度、组件内样式行数四项上限由 lint 强制。全仓无超限文件。
- [x] AC16 `react-hooks/rules-of-hooks` 与 `react-hooks/exhaustive-deps` 为 error 且无豁免注释残留。
- [x] AC17 测试覆盖率门建立并通过，阈值与依据落盘。
- [x] AC18 五项性能场景（大表单输入、10,000 行列表滚动、实时日志流、图表更新、路由切换）的迁移前后测量数据落盘，无回退项。回退项需有优化后的复测数据。React 数值：`08-22-regression-release/perf-react-after.md`。列表仍为 500 行替代口径（与 Vue 相同）。图表范围 P50 高于 Vue、P95 低于 Vue，不单开优化。
- [x] AC19 启动耗时、首屏渲染耗时、bundle 体积三项不高于迁移前基线，或超出项有重设依据与对比数据。DCL 不高于 Vue。FCP `/` 48 vs 28（打包 vs tauri dev）。index gzip 72.92 vs 45.41，预算重设见 `08-22-regression-release/bundle-reset.md`。
- [x] AC20 路由级代码分割与三层 CSS 加载策略在 React 侧等价保留，首屏加载模块集合未扩大。
- [x] AC21 七个功能面的差异矩阵落盘，无未确认项。20 个重复实现文件全部处理，统一后总行数与 15,672 行基线的对比数据落盘。
- [x] AC22 base 组件内无平台名称条件分支，由检查规则断言。统一后各平台视图文件行数不超过 100 行。
- [x] AC23 「平台 × 功能面」验证矩阵无未验证格。修改一处共性行为后全部消费平台同时生效，由一个跨平台用例证明。

## 任务地图

18 个子任务。规模列为迁移涉及的 `.vue` 行数或对应资产量。

| 序  | 子任务                            | 范围                                                                                                              | 规模                    |
| --- | --------------------------------- | ----------------------------------------------------------------------------------------------------------------- | ----------------------- |
| 1   | `08-22-react-foundation`          | Vite + React + Router + 状态库基座，lint / type-check / test / build 管线切换，`api` / `types` / `utils` 复用验证 | 319 文件复用验证        |
| 2   | `08-22-dep-upgrade`               | npm 依赖全量升级、`src-tauri` Rust 依赖升级、`overrides` 复核、audit                                              | 40 个 npm 包            |
| 2b  | `08-22-workspace-cargo-upgrade`   | 根 workspace 共享 Cargo 依赖升级，`ts-rs` 11 → 12 与 204 个类型绑定重新生成                                       | 13 个 crate             |
| 2c  | `08-22-arch-quality-perf`         | 分层与依赖方向强制、规模与复杂度 lint 约束、覆盖率门、五项性能基线、React 重渲染纪律                              | 规则与工具产出          |
| 3   | `08-22-design-system`             | Tailwind v4 `@theme`、448 token 迁移、shadcn/ui 原语、2,571 处硬编码收口、主题配置域扩展                          | 28,460 行 CSS           |
| 4   | `08-22-state-logic-port`          | 10 个 store 与 35 个 composable 迁移                                                                              | 9,425 行                |
| 5   | `08-22-shell-port`                | App 外壳、`layout` / `common` / `ui` 与外壳级根组件、75 条路由、窗口与主题引导接线                                | 6,877 行                |
| 5b  | `08-22-platform-unify`            | 七个功能面差异普查与统一层：Settings / Profiles / Auth / MCP / Agents / Plugins / Commands                        | 15,672 → 6,000–7,500 行 |
| 6   | `08-22-views-claude`              | Claude Code 视图与组件，含 `claude-observer`                                                                      | 10,989 行               |
| 7   | `08-22-views-codex`               | Codex 视图与组件，含 Auth 与 Sessions                                                                             | 13,226 行               |
| 8   | `08-22-views-secondary-platforms` | Grok、Gemini CLI、OpenCode、`views/generic`                                                                       | 11,669 行               |
| 9   | `08-22-views-checkin`             | CheckIn 视图与组件，含 OAuth 向导与 WAF 流程                                                                      | 8,607 行                |
| 10  | `08-22-views-usage`               | Dashboard、Usage、Budget、Pricing，含 ApexCharts 桥接重写                                                         | 11,995 行               |
| 11  | `08-22-views-profiles-config`     | `profiles`、`provider-templates`、Configs、AppSettings、Converter                                                 | 9,882 行                |
| 12  | `08-22-views-sync-tools`          | Sync、MCP、Commands、editor、Monitoring、SSH、WSL、tray                                                           | 12,055 行               |
| 13  | `08-22-i18n-port`                 | 4,164 个 key 的调用点转 react-i18next，词条数据复用                                                               | 417.7 KB 词条           |
| 14  | `08-22-test-contract-rebuild`     | 122 个 smoke 测试重写、契约文档重写（基线 16 份 → 最终 19 份）                                                    | —                       |
| 15  | `08-22-regression-release`        | 185 界面逐屏回归、Tauri 打包与四项行为验证、`just ci` 全绿                                                        | —                       |

**覆盖校验**：子任务 5–12 的 `.vue` 行数合计 85,300，与全仓 `.vue` 总量 85,274 的差值来自跨域共享组件在两处登记。185 个 `.vue` 文件全部有归属，无遗漏。

### 执行顺序

父子结构不是依赖系统。以下顺序写入各子任务的 `prd.md` 与 `implement.md`，实施时按此推进。

```
1 react-foundation  →  2 dep-upgrade  →  2c arch-quality-perf  →  3 design-system
                                                                        ↓
                                              4 state-logic-port  →  5 shell-port
                                                                        ↓
              阶段 4a 共享层前置：11 批次 1（components/profiles/ 10 文件）
                                12 批次 3 前半（components/mcp/ 4 文件）
                                                                        ↓
                                                    5b platform-unify（差异普查可提前到 2c 之后启动）
                                                                        ↓
        ┌──────────┬──────────┬──────────┬──────────┬──────────┐
        6 claude   7 codex    8 secondary 9 checkin  10 usage   11 profiles-config（批次 2 起）  12 sync-tools（其余批次）
        └──────────┴──────────┴──────────┴──────────┴──────────┘
                                              ↓
                       13 i18n-port（可与 6–12 并行推进）
                                              ↓
                       14 test-contract-rebuild
                                              ↓
                       15 regression-release

2b workspace-cargo-upgrade（旁路，与 1–5 任一阶段并行，不占关键路径）
```

约束说明：

- 子任务 1 与 2 交织，`dep-upgrade` 的 Tailwind v4 与 React 版本选择决定基座形态，两者需连续执行。
- 子任务 2c 必须在 6–12 之前完成。规则若在视图迁移之后才落地，7 个子任务已产出的约 78,000 行代码需要返工。
- 子任务 3 必须在 6–12 之前完成原语与 token 层，避免视图迁移时重复决策样式。
- 子任务 6–12 之间无相互依赖，可并行。**两处例外构成阶段 4a**：`components/profiles/`（11 的批次 1）与 `components/mcp/`（12 的批次 3 前半）是 5b 的 `BaseProfiles` 与 `PlatformMcpView` 的复用对象。React base 组件无法复用尚未迁移的 Vue 组件，因此这两批必须早于 5b 的对应批次。两者只做框架迁移与接口公示，不改造接口。
- 子任务 13 的调用点转换与视图迁移在同一批文件上发生，实施时按视图批次同步转换，`i18n-port` 负责运行时切换与收尾校验。
- 子任务 14 需在 5 完成后开始交付最小测试集，不等到 12 结束（见 C2）。
- 9 个业务域根级组件按域归入视图子任务，不属 5：`BaseSlashCommands`、`McpPresetsPanel`、`McpSyncPanel`、`CommandFormModal`、`CommandList` 归 12；`EditConfigModal`、`AddConfigModal`、`ConfigCard` 归 11；`CheckinProgressModal` 归 9。
- `MonitoringView.vue`（699 行）按其依赖（`ansiRenderer.ts`、`logRedact.ts`、`app-log` 事件）归 12，不归 10。
- 子任务 2b 与前端迁移无技术依赖，可与 1–5 任一阶段并行，不占关键路径。它与 2 在 `ts-rs` 升级上有一个协同点：2b 执行 Rust 侧升级并重新生成 204 个类型文件，2 执行前端侧的 diff 判定。两个任务的 `implement.md` 需写入同一个协同检查点。
- 子任务 5b 必须在 6、7、8、11、12 之前完成统一层接口。这五个子任务的范围因 5b 而缩减，其 `prd.md` 范围表需在 5b 的差异普查（R1）完成后调整。9 与 10 不受 5b 影响。**反向依赖两处**：5b 批次 4 依赖 11 批次 1 的 `components/profiles/`，5b 批次 5 依赖 12 批次 3 前半的 `components/mcp/`，见阶段 4a。
- 子任务 5b 的差异普查可在 2c 之后即启动，不必等 5 完成。普查产出的差异矩阵是 5b 实现与全部平台验证的共同输入。
- `AppSettingsView.vue`（1,399 行）为应用级设置，不属 5b 的统一范围，仍归 11。

### Phase 1 收尾前置动作

以下动作必须在 `08-22-react-foundation` 开始实现之前完成，否则迁移后无法回溯原始状态：

- 从 `dev` 分支构建当前产物，采集 185 个界面在明暗两套主题下的截图与关键交互录屏，作为 `08-22-regression-release` 的比对基线。
- 记录当前启动耗时、首屏渲染耗时与 bundle 体积基线。
- 记录当前 `bun run test:smoke` 的 122 项通过清单与被测组件清单，作为 `08-22-test-contract-rebuild` 的覆盖范围比对基准。

## Out of Scope

- `ccr-ui/src-tauri` 的功能改动与新增 IPC 命令。
- `crates/` 下 12 个 Rust crate 的功能改动。
- `ccr-vscode` 扩展。
- `docs/` VitePress 站点的框架变更。
- 前端信息架构（IA）重做。`views/` 下的页面划分与 75 条路由结构保持现状。跨平台功能面统一（R9）是视图实现层的收敛，不属 IA 重做。
- 新增业务功能。

## Open Questions

无。Q1（workspace Cargo 依赖是否纳入升级范围）已于 2026-08-22 由用户确认纳入，对应 R2.3b 与子任务 `08-22-workspace-cargo-upgrade`。

## Risks

| 风险                                                              | 影响                                           | 缓解                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| ----------------------------------------------------------------- | ---------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 迁移期约 75 工程日无法发版                                        | 期间无法交付其他前端需求                       | 已由 C1 接受。紧急修复走 `dev` 分支，迁移分支定期 rebase                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| 122 测试与 16 契约同时失效                                        | IPC 行为回归无保护                             | 子任务 14 在 5 完成后先交付 `api-facade-coverage` 与 IPC 名称断言，并把 `api-facade-boundary` 测试的遍历后缀集合扩到 `.tsx`                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| `nextTick` 52 处无等价物                                          | 依赖 DOM 更新时序的交互出现偶发失败            | 逐点登记，转换时记录原始时序意图，回归阶段重点复验                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| CodeMirror 6 桥接重写                                             | 原始配置编辑器行为回退                         | `raw-config-editor-contracts.md` 先重写为可执行断言，再改实现                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| ApexCharts 桥接重写                                               | 图表样式与稳定性回退                           | `usage-chart-stability-contracts.md` 与 `apexcharts-style-contract` 测试先行                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| Tailwind v4 的 `@apply` 需 `@reference`                           | 25 个文件的样式静默失效                        | 升级时一次性处理该 25 个文件，由 stylelint 规则兜底                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| WAF WebView bypass 依赖 WebView 行为                              | 迁移后 CheckIn 流程失效                        | 子任务 9 与 15 各自验证，保留 Tauri 侧实现不变                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| React 组件级重渲染替代 Vue 细粒度响应式                           | 大表单、长列表、日志流、图表出现掉帧与输入延迟 | 子任务 2c 定义 memo 与状态切分纪律并落为 lint 规则，子任务 15 逐项测量对比                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| 加密依赖升级（`aes-gcm` / `argon2` / `blake3` / `sha2` / `rand`） | 已加密的凭据数据无法读取                       | 子任务 2b 按三种真实持久格式分别固化旧版本产物再读回。格式与位置：Codex auth 导出（`crates/ccr-codex/src/services/codex_auth_crypto.rs`，Argon2id + salt + KDF 参数 + AAD）、Sync 信封 V2（`crates/ccr-sync/src/sync/envelope.rs`，magic + version + kdf_params + salt + nonce + metadata AAD，另有 PlaintextV1 读路径）、CheckIn 凭据（`crates/ccr-checkin/src/core/crypto.rs`，独立随机 key 文件 + `nonce \|\| ciphertext`，无 KDF）。`crates/ccr-db` 只用 `sha2`（导入去重），blake3 在 `ccr-core` / `ccr-skills` / `ccr-store` / `ccr-cli` 作内容哈希，其变化影响缓存与索引而非解密 |
| 跨平台重复实现 20,218 行照搬迁移                                  | 迁移后改一处行为仍需改 3–5 处，原始目标未达成  | 见「跨平台重复」一节的决策记录                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| 无文件规模、复杂度与导入边界的 lint 约束                          | React 侧重现 1,744 行单文件与 139 处分散样式   | 子任务 2c 在视图迁移前落地 lint 约束                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |

## Notes

- 子任务的 `design.md` 与 `implement.md` 在各自的 Phase 1.1 中编写，本父任务不预先代写。
- 本任务为复杂任务，父任务自身在 `task.py start` 之前需完成 `design.md`（整体架构决策、状态库与路由选型、目录结构、迁移期分支策略）与 `implement.md`（子任务推进顺序、评审门、回滚点）。
