# 执行计划：架构约定、质量门与性能基线

> 父任务：`08-22-react-migration`（阶段 2，本任务先于 `08-22-design-system`）。
> 分支：`feature/react-migration/arch-quality-perf`，PR 目标 `feature/react-migration`。
>
> **本任务是硬门**。规则若在阶段 5 之后才落地，七个视图子任务已产出的约 78,000 行需返工（父任务 `implement.md` §4 约束门）。

## 前置确认

- [x] 父任务基座门已通过（`08-22-react-foundation` AC1–AC9 与 `08-22-dep-upgrade` AC1–AC10 全部满足）。
- [x] `08-22-react-foundation` 的 `path-mapping.md` 已落盘，作为目录分层规则的输入。
- [x] `git checkout -b feature/react-migration/arch-quality-perf feature/react-migration`

## 批次 1：分布测量

阈值取值的前置。无此批次则第 3 节的方法无输入。

- [x] 测量当前 185 个 `.vue` 与已迁移 `.tsx` 的行数分布，产出 P50 / P75 / P90 / P95 / max。
- [x] 测量圈复杂度、最大嵌套深度、最大参数个数分布（用 ESLint 规则以 warning 级别跑一遍取数据，不提交该配置）。
- [x] 测量组件内样式行数分布（139 个带局部样式的组件，24,434 行）。
- [x] 按 `design.md` §3.1 第 2 步排除将被统一层接管的 20 个文件（清单见 `platform-unify/implement.md` 批次 1），产出排除后的暂定分布。不做「统一后分布」的推算——总行数区间无法唯一推出文件行数分布（`design.md` §3.1 末段）。

产物：`distribution.md`。（已提交 1f602bde，测量脚本 `ccr-ui/scripts/measure-distribution.mjs` 可复测）

## 批次 2：分层与边界规则

- [x] 装 `eslint-plugin-boundaries`（或按 `design.md` §11 的判据选定的等价物），按 `design.md` §1 的依赖图声明 element types 与 rules。
- [x] 门面边界规则用核心 `no-restricted-imports`，既有导入点列白名单。该规则只管**消费侧**。
- [x] 定义面不新造机制：确认既有 `ccr-ui/tests/api-facade-boundary.smoke.test.ts` 的 `freezes legacy direct invoke calls in tauri.ts` 用例保留（9 条允许命令集合），并在 `layering-contracts.md` 中写明该分工。
- [x] 循环依赖检查落为独立脚本，加入 `package.json` 的 `check:cycles`，纳入 `just frontend-check`。
- [x] 构造 4 个违规用例（`design.md` §2）：3 个 lint 夹具 + 1 个向 `tauri.ts` 加 `invoke()` 的临时改动（跑一次 smoke 确认变红后还原，不提交）（AC2）。
- [x] 构造 1 个循环用例，断言脚本报错（AC3）。
- [x] 夹具目录从正常 lint 范围排除。
- [x] 记录 `bun run lint:ci` 加规则前后的单次耗时。超过 2 倍则按 `design.md` §1 的兜底判据把开销最大的规则移入 CI 专用。

验证：`bun run lint:ci` 退出码 0（AC1）；`bun run check:cycles` 退出码 0。

### 批次 2 验证证据（2026-08-23，分支 `react-migration/react-foundation`，未提交）

**规则形态与修复**：

- `eslint.config.js`：`boundaryElements`（`ui-primitive` / `shell` / `feature`+domain capture / `legacy-feature` / `store` / `composable` / `api` / `utils` / `types` / `shared`）+ `boundaryPolicies` 具名导出；`app/arch-boundaries` 块以 `boundaries/dependencies: error` 强制分层；`no-restricted-imports` 冻结 `src/api/tauri.ts` 消费侧；`tests/api-facade-coverage.smoke.test.ts` 白名单。
- 修复 `shared → shell` 目标遗漏：`src/main.tsx` 导入 `./shell/*` 曾命中 `boundaries/dependencies`（shell/shared 粘合层应可依赖一切内部层），在 shared/shell 放行策略的 anyOf 中加入 `'shell'`。仅此一处策略放宽，其余禁令未动。
- `check-arch-boundaries.mjs`：自检临时配置 `.eslint.arch-selfcheck.mjs` 补齐 `import/resolver.typescript`（`alwaysTryTypes: true, project: './tsconfig.json'`，与主配置一致）——extensionless 相对导入此前无法解析导致 3 个夹具静默通过；夹具根 `reverse-dep.ts` 以 `mode:'file'` 单文件映射为 `utils` 元素（配合 `boundaries/legacy-warnings: false` 抑制弃用告警）。
- `reverse-dep.ts` 夹具导入路径修正：`'../../stores/fixtureStore'`（不存在）→ `'./store'`（夹具 store `tests/fixtures/arch-violations/store/index.ts`）。
- `check-cycles.mjs`：dpdm 4.3.0 的 `parseDependencyTree` 返回 `DependencyTree` 而非 `OutputResult`（无 `.circulars` 字段），循环清单改用 `parseCircular(tree, skipDynamicImports=true)`（等价 CLI `--skip-dynamic-imports circular` 语义）。
- `tsconfig.json`：`exclude` 加入 `tests/fixtures/arch-violations`——夹具故意违反依赖方向且类型上不成立（互递归函数隐式 any、store 导出名不匹配），只供定向自检，不让 `tsc --noEmit`（`tests/**/*.ts` 的 include 会扫到）拦截常规 `frontend-check-quick`。

**Definition-of-done 命令与退出码**：

| 命令 | 退出码 | 结果 |
| --- | --- | --- |
| `cd ccr-ui && bun run lint:ci` | 0 | eslint + stylelint 全绿 |
| `cd ccr-ui && bun run check:arch-boundaries` | 0 | 4 个夹具全部 PASS（跨层/跨域/反向依赖 boundaries/dependencies + 门面绕过 no-restricted-imports） |
| `cd ccr-ui && bun run check:cycles` | 0 | 217 个文件，无循环依赖 |
| `cd ccr-ui && bun ./scripts/check-cycles.mjs --self-check` | 0 | 恰好检出 1 个 2 节点循环（cycle-a <-> cycle-b） |
| `just frontend-check-quick` | 0 | 类型 + lint + smoke 全绿 |

**AC2 用例 4（定义面冻结，红→绿）**：

- 临时改动：向 `src/api/tauri.ts` 追加 `export const fixtureNewFacadeCommand = async <T = UnknownRecord>(): Promise<T> => { return invoke('some_new_command_for_fixture') }`。
- 红跑：`bun run test:smoke` 中 `tests/api-facade-boundary.smoke.test.ts > API facade boundary > freezes legacy direct invoke calls in tauri.ts` **FAIL**（`AssertionError: expected [ 'update_config', …(9) ] to deeply equal [ 'update_config', …(8) ]`，received 新增 `"some_new_command_for_fixture"`，断言位于 `tests/api-facade-boundary.smoke.test.ts:104`）。全量 smoke 退出码 1。
- `git checkout -- ccr-ui/src/api/tauri.ts` 还原后复跑该文件：4 tests 全通过，退出码 0。`git diff ccr-ui/src/api/tauri.ts` 为空。

**lint 耗时（design.md §1 兜底判据，`time bunx eslint . --quiet` 3 次均值）**：

| 配置 | 3 次耗时 | 均值 |
| --- | --- | --- |
| 当前配置（含 arch-boundaries + no-restricted-imports） | 4294 / 4167 / 4413 ms | ≈ 4,291 ms |
| HEAD 配置（不含两项，`git stash push -- ccr-ui/eslint.config.js` 临时切换） | 3116 / 2979 / 3004 ms | ≈ 3,033 ms |

比值 ≈ 1.41×，**未超过 2× 阈值**，无需将任一规则移出 `lint` 移入 CI 专用。耗时增量主要来自 `boundaries/dependencies` 的模块解析。

**环境与脚本注册**：

- `ccr-ui/package.json` 新增 `check:cycles`、`check:arch-boundaries`（置于 `check:bundle-budget` 之后）。
- 根 `justfile`：`frontend-check` 依赖链加入 `frontend-check-cycles`、`frontend-check-arch-boundaries`（独立 recipe，`cd ccr-ui && bun run …`，Windows pwsh 下可跑；`just --list` 可见、`just -n frontend-check` 可展开）。注意：`just --fmt --check` 在改动前即因历史漂移失败，本任务未触碰既有格式问题。
- `ccr-ui/.gitignore` 追加 `.eslint.arch-selfcheck.mjs`（脚本 finally 已清理，git status 无残留）。

## 批次 3：规模与复杂度规则（暂定阈值）

- [x] 按 `design.md` §3.1 第 3 步取五项**暂定阈值**。
- [x] 一轮反馈调整（超限文件数 >15% 上调一档，<3% 下调一档），只做一轮。
- [x] 五项规则以 error 级别加入 `eslint.config.js`。组件内样式行数用检查脚本（需同时读 `.tsx` 与 `.module.css`）。
- [x] `thresholds.md` 落盘，含暂定取值、依据与「最终值待阶段 4 冻结」的标注（AC4）。
- [x] 超限文件清单落盘，每项标注所属处理批次（归到七个视图子任务之一），无全局豁免（AC11）。

验证：`bun run lint:ci` 退出码 0。超限文件在此时应报错——把它们加入各视图子任务的处理批次，不加 `eslint-disable`。

### 批次 3 验证证据（2026-08-23，分支 `react-migration/react-foundation`，未提交）

**阈值取值与反馈轮（完整推导见 `thresholds.md`）**：

- 暂定值按 P90（`distribution.md` 活文件集 217 个）：行数 414→500（向上取整到 100）、复杂度 16、深度 3、参数 4、组件样式 412。
- 反馈轮（以最终规则形态定向 lint 217 个文件，只做一轮）：

| 指标 | 超限 | 占比 | 判定 | 结果 |
| --- | --- | --- | --- | --- |
| 行数 500 | 19 | 8.8% | 在 [3%,15%] 带内 | 保留 |
| 复杂度 16 | 13 | 6.0% | 在 [3%,15%] 带内 | 保留 |
| 深度 3 | 2 | 0.9% | **< 3%** | **下调至 2** |
| 参数 4 | 6 | 2.8% | **< 3%** | **下调至 3** |

- 生效暂定阈值：`max-lines=500`、`complexity=16`、`max-depth=2`、`max-params=3`、组件样式 412 + JSX 比例约束。调整后占比均落回 [3%,15%] 带内（深度 6.0%、参数 8.8%）。两项下调与预期「不改动」不符，按真实计数执行 design.md §3.2 的反馈规则（记录在 `thresholds.md` §2）。

**规则形态与豁免机制**：

- `eslint.config.js` 新增 `app/threshold-rules` 块：四项 error 级规则，作用域 `src/**/*.{ts,tsx,mts}`（tests/、scripts/ 不在测量集；`src/types/generated/**` 与 `**/*.vue` 已在全局 ignore）。
- 超限文件逐文件登记豁免：`app/threshold-rules` 之后 49 个「文件 × 规则」覆盖块，各带内联注释（文件、违规指标+实测值、处置）。无全局豁免、源文件无 `eslint-disable`（R12、AC11）。49 = 17 注册豁免（纯数据表/生成物/冻结门面）+ 32 归迁移批次（state-logic-port 12、views-usage 6、views-checkin 4、views-profiles-config 4、shell-port 4、views-secondary-platforms 1、i18n-port 1），无未分配项。
- `ccr-ui/scripts/check-component-style-lines.mjs`：`.tsx` + `.module.css` 配对的样式行数检查（绝对上限 412 + 样式 ≤ JSX 比例约束），不查 `.vue`（已退出管线）。`package.json` 新增 `check:style-lines` 并追加进 `lint:ci`。

**Definition-of-done 命令与退出码**：

| 命令 | 退出码 | 结果 |
| --- | --- | --- |
| `cd ccr-ui && bun run lint:ci` | 0 | eslint + stylelint + check:style-lines 全绿 |
| `cd ccr-ui && bun run check:style-lines` | 0 | 0 个 `.module.css`，零违规（基线，批次 3b 补测） |
| `cd ccr-ui && bunx eslint src/stores/usage.ts` | 0 | 豁免生效：max-lines/complexity/max-depth 不报，其余规则正常应用 |
| `just frontend-check-quick` | 0 | 类型 + lint + 59 文件 293 smoke 全绿 |

**合成违规证明（规则确实生效）**：

- 临时 `src/__scratch_proof.ts`（600 行 filler + 高复杂度 + 嵌套 + 6 参数函数），`bunx eslint src/__scratch_proof.ts` 报 6 个 error（max-lines 608、complexity 24、max-depth 3/4/5、max-params 6），退出码 1。删除后 `git status` 无残留。
- `check:style-lines` 反向验证：临时 `src/__scratch__/Scratch.module.css`（452 行）+ `Scratch.tsx`，脚本报绝对上限与比例约束两项违规，退出码 1；删除后无残留。

### 批次 3b：阈值冻结（阶段 4 → 5 门，不在阶段 2 执行）

本子批次在 `08-22-platform-unify` 批次 6 完成后执行（协同点 N）。

- [ ] 用统一层的实际文件集合替换批次 1 中排除的 20 个文件条目，重算分布。
- [ ] 按 `design.md` §3.2 重取 P90，写入 `thresholds.md` 的第二段。与暂定值不同时以本段为最终值。
- [ ] 超限清单重出，新增超限项分配处理批次。
- [ ] `eslint.config.js` 的阈值更新为最终值。

验证：`bun run lint:ci` 退出码 0。

## 批次 4：类型与 hooks 规则

- [ ] `react-hooks/rules-of-hooks` 与 `react-hooks/exhaustive-deps` 设为 error（R3）。
- [ ] `@typescript-eslint/no-unsafe-*` 系列启用（R4）。
- [ ] `no-explicit-any: error` 保留。
- [ ] `design.md` §6 的四条可 lint 的重渲染规则落为 error。

验证：`bun run lint:ci` 退出码 0（AC1）。

## 批次 5：覆盖率门

- [ ] 阈值从 justfile 参数移入 `vitest.smoke.config.ts` 的 `coverage.thresholds`。
- [ ] 按 `design.md` §4 复核 70% 取值，调整需给依据。
- [ ] `frontend-coverage` 纳入 `just ci`（插在 `frontend-check` 之后）。
- [ ] 不新增 `functions` / `branches` / `statements` 阈值（依据见 `design.md` §4 末段）。

验证：`just frontend-coverage` 退出码 0（AC5）；`just ci` 的步骤数从 13 变 14，父任务 `implement.md` §2.1 与 `prd.md` AC3 需同步更新。

## 批次 6：状态判定表

- [ ] 按 `design.md` §5 的判据，10 个 store + 35 个 composable 共 45 项逐个归类。
- [ ] `state-disposition.md` 落盘，无未判定项（AC6）。
- [ ] 交付给 `08-22-state-logic-port`。

## 批次 7：性能测量与基线

- [ ] 按 `design.md` §7 写五个测量脚本，落在 `ccr-ui/scripts/perf/`。
- [ ] 每个脚本连续跑 3 次，相对标准差不超过 15%。超过则改进方法后重测。
- [ ] 采集 React 侧基线，`perf-baseline.md` 落盘（AC7）。
- [ ] 与父任务基线采集门在 `dev` 上采集的数据对齐（同一测量方法，只依赖 DOM 与 `performance` API）。

注意：场景 1、3、4 依赖的视图此时尚未迁移（阶段 5 才迁）。本批次的做法是先在 Vue 版本上跑通脚本并采数据，React 侧的对应数值在 `08-22-regression-release` 补测。脚本的框架无关性是前提。

## 批次 8：预算与分割约定

- [ ] 按 `design.md` §8 重设 `check-bundle-budget.mjs` 配置。
- [ ] `motion` 与 `zod` 单列两行，记录实际增量与预留值（R9.1）。
- [ ] 判定 `manualChunks` 是否新增三组，结论通知 `08-22-react-foundation` 或直接改 `vite.config.ts`。
- [ ] `bundle-budget.md` 落盘（AC9）。
- [ ] 按 `design.md` §9 产出 `code-splitting.md`（AC10）。

验证：`bun run check:bundle-budget` 退出码 0。

## 批次 9：契约文档与登记

- [ ] `react-rerender-discipline.md` 与 `layering-contracts.md` 写入 `.trellis/spec/ccr-ui/frontend/`。
- [ ] 登记到 `08-22-test-contract-rebuild` 的范围表，16 份变 18 份（AC12）。
- [ ] 通知七个视图子任务：动手前需阅读 `react-rerender-discipline.md`（R8）。

## 验证命令

| 时机        | 命令                                                  |
| ----------- | ----------------------------------------------------- |
| 批次 2–4 后 | `bun run lint:ci`、`bun run check:cycles`             |
| 批次 5 后   | `just frontend-coverage`                              |
| 批次 8 后   | `bun run check:bundle-budget`                         |
| 交付前      | `just frontend-check-quick`、`just frontend-coverage` |

## 交付门（父任务约束门的一半）

- [ ] AC1–AC12 全部满足。
- [ ] 全部新增规则为 error 级别，无 warning 级别软约束（R2 末段）。
- [ ] 无全局豁免。豁免逐文件登记（R12）。
- [ ] 七份记录落盘：`distribution.md`、`thresholds.md`、`state-disposition.md`、`perf-baseline.md`、`bundle-budget.md`、`code-splitting.md`、超限文件清单。
- [ ] 两份契约进 spec 目录并完成登记。
- [ ] `just ci` 步骤数变更已同步到父任务文档。

## 回滚点

| 批次    | 回滚方式                                                           |
| ------- | ------------------------------------------------------------------ |
| 1、6、7 | 只产出文档与脚本，revert 无副作用                                  |
| 2–5     | 每批次单独提交。规则回滚即回到无约束状态，不影响已写代码的可运行性 |
| 8       | 预算配置回滚                                                       |

规则一旦被七个视图子任务消费，回滚的代价是那些子任务的代码不再受约束，但代码本身仍可运行。因此回滚安全，代价在于返工风险回归。

## 协同点

| 编号 | 内容                                            | 对方                          | 时机      |
| ---- | ----------------------------------------------- | ----------------------------- | --------- |
| L    | 性能基线供最终对比                              | `08-22-regression-release`    | 批次 7 后 |
| —    | `state-disposition.md` 是对方的直接输入         | `08-22-state-logic-port`      | 批次 6 后 |
| —    | `react-rerender-discipline.md` 需在动手前被阅读 | 七个视图子任务                | 批次 9 后 |
| —    | `manualChunks` 分组结论                         | `08-22-react-foundation`      | 批次 8    |
| —    | 新增两份契约进范围表（使其从 16 份变 18 份）    | `08-22-test-contract-rebuild` | 批次 9    |
| N    | 阈值冻结：对方批次 6 完成后本任务执行批次 3b    | `08-22-platform-unify`        | 批次 3b   |
| —    | 组件内样式行数上限约束其产出                    | `08-22-design-system`         | 批次 3 后 |
