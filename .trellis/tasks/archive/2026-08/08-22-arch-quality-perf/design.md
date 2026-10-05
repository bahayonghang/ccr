# 技术设计：架构约定、质量门与性能基线

> 父任务：`08-22-react-migration`。本任务产出规则、工具与文档，不产出业务代码。本文件写强制手段的选型、阈值的取值方法与性能测量方案。

## 1. 分层与依赖方向的强制手段

目标依赖图（父任务 `design.md` §2）：

```
features/*  →  features/platform  →  ui  →  styles
     ↓                ↓
    api  →  types
     ↑
  config / configs
```

禁止项：`ui/` 导入 `features/` 或 `api/`；`features/<a>/` 导入 `features/<b>/`；任何位置反向导入。

三类违规的强制手段分开选，因为强制的粒度不同：

| 违规类型                                       | 手段                                                    | 依据                                                                                                                       |
| ---------------------------------------------- | ------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------- |
| 跨层导入、反向依赖、跨域直连                   | `eslint-plugin-boundaries` 的 element types + rules     | 该插件按目录声明层级类型，规则表达为「哪类可以导入哪类」，与本仓的 `features/` / `ui/` / `api/` / `types/` 分层同构        |
| 门面绕过（消费侧直接 import `src/api/tauri.ts`） | ESLint 核心 `no-restricted-imports` 的 patterns         | 无需新依赖。规则形态为：除 `src/api/index.ts` 外禁止导入 `src/api/tauri.ts`。冻结门面的既有导入点为白名单                 |
| 门面绕过（定义侧向 `tauri.ts` 加新 wrapper）   | 既有 smoke 测试的冻结用例，不是 lint                    | 见下方「定义面的冻结手段」                                                                                                 |
| 循环依赖                                       | 独立 CI 脚本（`dpdm` 或等价物），不用 `import/no-cycle` | `no-cycle` 需构建完整模块图，在 185+ 文件规模下 lint 单次耗时上升明显。作为独立脚本可只在 CI 与 `just frontend-check` 中跑 |

**定义面的冻结手段（不能用 `no-restricted-imports`）。** 该规则限制的是消费方的 import 路径，不检查 `tauri.ts` 内部声明了什么。且 `src/api/index.ts:8` 有 `export * from './tauri'`，开发者把新 wrapper 加进 `tauri.ts` 后经允许的 `@/api` 消费，import 规则全绿。

仓库已有正确的机制，本任务保留并复用，不重造：`ccr-ui/tests/api-facade-boundary.smoke.test.ts` 的 `freezes legacy direct invoke calls in tauri.ts` 用例断言 `tauri.ts` 内 `invoke()` 的命令名序列**恰好等于** `ALLOWED_TAURI_FACADE_COMMANDS`（当前 9 条：`update_config`、`list_mcp_presets`、`get_mcp_preset`、`install_mcp_preset`、`install_mcp_preset_single`、`list_source_mcp_servers`、`sync_mcp_server`、`sync_all_mcp_servers`、`health_check`）。新增 wrapper 会使该断言失败。

同文件另有两个用例需一并保留：`keeps invoke() usage inside the API layer across all of src/`（`INVOKE_ALLOWED_PATHS` 白名单）与 `keeps manifest-typed commands behind generated clients`。

**该文件的遍历后缀集合必须扩到 `.tsx`。** 现为 `/\.(ts|mts|vue)$/`。迁移后组件是 `.tsx`，不改则三个用例对全部 React 组件失效且静默通过。该动作归 `08-22-test-contract-rebuild` 批次 1，本任务在 AC2 的用例中依赖它。

`src/api/tauri.ts` 的冻结边界因此有三道：消费侧 import 规则（本任务新增）、定义面冻结测试（既有，保留）、`api-facade-boundary.md` 契约文档（重写后保留）。

**选型的兜底判据**：`bun run lint:ci` 的单次耗时上限。加入全部规则后若耗时超过升级前的 2 倍，把开销最大的规则移出 `lint` 移入 `lint:ci`（CI 专用）。耗时数据落盘。

## 2. 组件分层

原语（`ui/`）→ 复合组件 → 域组件 → 页面。

原语层的硬约束：不得导入 `features/`、`api/`、任何 store。原语只接受 props 与 children。该约束由第 1 节的 `boundaries` 规则覆盖，无需额外规则。

违规用例（AC2）四个，各构造一个最小文件放在测试夹具目录，断言检查报错：

1. `src/ui/Button.tsx` 导入 `src/features/claude/...` → lint 报错。
2. `src/features/claude/X.tsx` 导入 `src/features/codex/Y.tsx` → lint 报错。
3. `src/features/claude/X.tsx` 直接从 `src/api/tauri.ts` 导入 → lint 报错（消费侧）。
4. 向 `src/api/tauri.ts` 添加一个新的 `invoke('some_new_command')` → **既有 smoke 测试** `freezes legacy direct invoke calls in tauri.ts` 报错（定义面）。用例 3 通过不能替代用例 4：经 `@/api`（`export * from './tauri'`）消费时 import 规则不触发。

用例 1–3 的夹具目录需从正常 lint 范围排除，只在规则自检时定向 lint。用例 4 的验证方式是临时改 `tauri.ts` 跑一次 `bun run test:smoke` 确认变红，然后还原，结果记录，不提交该临时改动。

## 3. 规模与复杂度阈值的取值方法

不使用无根据的整数（R2）。**取值分两段**，因为最终分布依赖 `08-22-platform-unify` 的实际产出，而本任务在其之前执行。

### 3.1 第一段：暂定阈值（本任务，阶段 2）

1. **测量现状分布**。按文件统计行数、圈复杂度、最大嵌套深度、最大参数个数、组件内样式行数五项，产出分位数（P50 / P75 / P90 / P95 / max）。
2. **排除将被统一层接管的 20 个文件**（15,672 行，清单见 `platform-unify/implement.md` 批次 1）。这 20 个文件在阶段 4 后不再以现形态存在，把它们计入分布会抬高阈值。排除后的分布即为暂定分布。
3. **暂定阈值取该分布的 P90**，行数向上取整到 100 的倍数，其余四项向上取整到整数。
4. 记录超限清单与所属处理批次（AC11）。

**为什么不在本段直接模拟统一后的分布**：`platform-unify` 只承诺总行数落在 6,000–7,500 区间，未承诺文件数与每文件行数；Auth 面还可能部分保留或不统一（其 R6）。总行数区间无法唯一推出文件行数分布，因此本段不做该推算——不同执行者从同一输入会得到不同的 P90。

### 3.2 第二段：冻结阈值（阶段 4 → 5 门，协同点 N）

`platform-unify` 批次 6 完成后，统一层的实际文件集合与行数已知：

1. 用实际文件集合替换步骤 2 中被排除的 20 个文件条目，重算分布。
2. 重取 P90，同样的取整规则。与暂定值不同时，以本段的值为最终值。
3. 超限清单重出。若超限文件数超过总数的 15%，阈值上调一档并重新记录；若低于 3%，阈值下调一档。**该反馈只做一轮**，避免为凑数字反复调整。
4. 暂定值与最终值都记录在 `thresholds.md`，含两段的分布数据（AC4）。

阶段 2 到阶段 4 之间的开发按暂定值 lint。冻结时若阈值下调，超出的文件进超限清单并分配处理批次，不做全局豁免（R12）。

现状锚点：最大 `.vue` 为 1,744 行；39 个根级视图平均 739 行；组件内样式 24,434 行分布在 139 个组件（均值 176 行）。

组件内样式行数上限的额外约束（父任务 `design.md` §6）：单组件的局部样式行数不超过其 JSX 行数。该约束用检查脚本实现，不用 ESLint（需同时读 `.tsx` 与配对的 `.module.css`）。

## 4. 覆盖率门

现状：`just frontend-coverage` 已有 lines ≥70% 阈值，写在 `justfile` 的命令行参数里，未纳入 `just ci`。

本任务的两项动作：

1. 阈值从 justfile 参数移入 `vitest.smoke.config.ts` 的 `coverage.thresholds`。移入后 `bun run test:smoke --coverage` 直接生效，不依赖 justfile 传参。
2. `frontend-coverage` 纳入 `just ci`。插入位置为 `frontend-check` 之后。

取值复核：按迁移前 122 个 smoke 测试的实际覆盖数据复核 70% 是否仍合适。复核只在数据显著偏离时调整（实际覆盖远高于 70% 则上调，接近 70% 则保留），调整需给出依据（R5）。

`lines` 之外是否增加 `functions` / `branches` / `statements` 阈值：默认不增加。理由是 122 个测试为等价重建（`08-22-test-contract-rebuild` Out of Scope 明确不扩大覆盖面），新增维度阈值会在不扩大覆盖的前提下变成阻塞项。

## 5. 状态三分类判定表

父任务 `design.md` §4 已给出 10 个 store 的处理，本任务把它扩成完整判定表并加入 35 个 composable。

判定表的列：名称、行数、类别（服务端数据 / 跨页面共享 / 组件本地 / 纯变换）、承载位置、依据。

判据：

| 类别       | 判据                                                                |
| ---------- | ------------------------------------------------------------------- |
| 服务端数据 | 数据来自 IPC 命令或 Tauri Event，有新鲜度概念 → TanStack Query      |
| 跨页面共享 | 多个路由读写同一份状态，非服务端数据 → Zustand                      |
| 组件本地   | 单个组件或单个表单内的瞬态 → `useState` / react-hook-form           |
| 纯变换     | 无状态，输入到输出的映射 → `utils/`，不进状态层                     |

45 项（10 store + 35 composable）全部归类，无未判定项（AC6）。表落盘为 `state-disposition.md`，供 `08-22-state-logic-port` 直接使用。

## 6. React 重渲染纪律

契约文档 `react-rerender-discipline.md` 的条目，标注可否 lint：

| 条目                                                              | 可 lint                                                                   |
| ----------------------------------------------------------------- | ------------------------------------------------------------------------- |
| 表单输入用 react-hook-form 非受控注册，不用 `useState` 逐字段受控 | 部分：可禁止在标记为表单的目录下对 `<input>` 同时写 `value` 与 `onChange` |
| 列表项组件用 `memo`，且 props 不传内联对象与内联函数              | 可：`react/jsx-no-bind` 与自定义规则组合                                  |
| Zustand 订阅必须用选择器，禁止整 store 订阅                       | 可：`no-restricted-syntax` 匹配无参数的 store 调用                        |
| Context 按变更频率拆分，高频值不与低频值同 Provider               | 否，review 门                                                             |
| `useMemo` / `useCallback` 只用于跨 `memo` 边界传递或昂贵计算      | 否，review 门                                                             |
| 日志流与图表数据更新走 ref + 批量提交，不逐条 setState            | 否，review 门                                                             |
| `key` 不用数组索引                                                | 可：`react/no-array-index-key`                                            |

可 lint 的四项落为 error 规则。不可 lint 的三项写入契约，作为七个视图子任务的 review 检查项。

契约在 `08-22-shell-port` 完成前定稳，七个视图子任务在动手前阅读（R8）。

## 7. 性能测量方案

五个场景的测量方式。驱动工具为 `playwright`（已在 devDependencies，当前未构成 e2e 套件；本任务只作为测量驱动使用，不建立 e2e 套件）。

| 场景                 | 指标                                           | 方法                                                                                                                                                                     |
| -------------------- | ---------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| 1 大表单输入延迟     | 按键到绘制的 P50 / P95                         | 在 `AppSettingsView`、`ClaudeCodeSettingsView`、`CodexSettingsView` 三页各选一个字段，连续键入 200 字符，用 `performance.measure` 记录每次 `input` 到下一帧 paint 的间隔 |
| 2 虚拟列表滚动       | 帧率、掉帧数                                   | 10,000 行数据，程序化滚动固定距离，用 `requestAnimationFrame` 时间戳序列算帧间隔分布                                                                                     |
| 3 日志流             | 帧率、`performance.memory.usedJSHeapSize` 增长 | `MonitoringView` 持续注入日志 5 分钟，每 10 秒采样                                                                                                                       |
| 4 图表更新与主题切换 | 单次渲染耗时                                   | 时间范围切换 20 次、明暗切换 20 次，每次记录 `performance.measure`                                                                                                       |
| 5 路由切换           | 单次切换耗时                                   | 复用 `measure-vite-route.mjs` 与 `perfTelemetry.ts` 的 `recordRouteTiming`。75 条路由按域采样，每域取 2–3 条                                                             |

三项产物指标：启动耗时、首屏渲染耗时、bundle 体积。前两项来自 `perfTelemetry.ts`，第三项来自 `check-bundle-budget.mjs`。

**可重复性要求**（R7、AC7）：同一脚本连续跑 3 次，指标的相对标准差不超过 15%。超过则该场景的测量方法需改进（增加采样次数或固定更多变量）后重测。

测量脚本落在 `ccr-ui/scripts/perf/` 下，每个场景一个文件。基线数据落盘为 `perf-baseline.md`。

本任务的基线在 React 基座之上采集，与父任务基线采集门在 `dev`（Vue 版本）上采集的数据构成对比的两端。两端的采集方法必须相同，因此测量脚本需能同时在 Vue 版本与 React 版本上运行——脚本只依赖 DOM 与 `performance` API，不依赖框架。

## 8. bundle 预算

预算取值方法：

1. 记录 Vue 版本的产物体积（父任务基线采集门已有）。
2. 记录 React 基座 + 依赖替换后的体积（`08-22-dep-upgrade` 段 4 提供）。
3. 为 `motion` 13.1.1 与 `zod` 4.4.3 显式预留额度（R9.1）。预留方式为在预算表中单列两行，记录各自的实际增量与预留值。
4. 预算超出不构成回退这两项选型的理由（父任务 `design.md` §12.2），但超出量需落盘。

`check-bundle-budget.mjs` 的配置按新框架重设。`manualChunks` 分组是否新增 `query-vendor` / `form-vendor` / `motion-vendor` 由本任务的预算数据判定，结论通知 `08-22-react-foundation`（若其已交付，则本任务直接改 `vite.config.ts`）。

预算表落盘为 `bundle-budget.md`（AC9）。

## 9. 代码分割与 CSS 分层的等价方案

| 现状                                                                                | React 侧等价                                                                                                                                               |
| ----------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 22 处 `defineAsyncComponent` 路由级懒加载                                           | `React.lazy` + `Suspense`，或 React Router 的路由级 `lazy`。二选一由 `08-22-shell-port` 落地，本任务只定约定：懒加载边界与路由边界一致，不在路由内部再分割 |
| 三层 CSS 加载（`shell-critical` / `deferred-decorations` / `deferred-interactive`） | 三层语义保留。Tailwind v4 下的落地方式由 `08-22-design-system` 决定，本任务只约定：首屏 CSS 只含 `shell-critical` 层，其余两层延迟加载                     |
| `corePlugins.preflight: false` + 自带 reset                                         | 由 `08-22-dep-upgrade` 段 2 完成等价处理，本任务核对首屏 CSS 体积未因 v4 上升                                                                              |

约定落盘为 `code-splitting.md`（AC10）。

## 10. 契约文档登记

本任务新增的契约文档：`react-rerender-discipline.md`、`layering-contracts.md`（第 1–2 节的规则说明）、`thresholds.md`、`perf-baseline.md`、`bundle-budget.md`、`code-splitting.md`、`state-disposition.md`。

其中前两份进 `.trellis/spec/ccr-ui/frontend/`，作为长期契约，需登记到 `08-22-test-contract-rebuild` 的契约范围表（基线 16 份 → 18 份；`08-22-platform-unify` 的 `platform-surface-contracts.md` 为第 19 份）（R11、AC12）。其余五份为本任务的测量与判定记录，留在任务目录。

## 11. 未决项

- `eslint-plugin-boundaries` 与 `dpdm` 的具体版本，以及是否有更契合本仓布局的替代物，在实施时按第 1 节的耗时判据确定。
- 五项阈值的暂定数值，按第 3.1 节的方法在测量后确定；最终值按第 3.2 节在阶段 4 后冻结。
- 覆盖率阈值是否从 70% 调整，按第 4 节的复核结果确定。
