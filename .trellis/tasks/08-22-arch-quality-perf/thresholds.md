# 规模与复杂度阈值记录（批次 3，暂定值）

> 任务：`08-22-arch-quality-perf` 批次 3。取值方法见 `design.md` §3.1（第一段，阶段 2）。
> 本文件记录：暂定取值与推导、反馈轮计数与决策、超限文件清单与逐文件处置（AC4、AC11）。
> 本文为**暂定段**；最终值待阶段 4 冻结（批次 3b，协同点 N），在本文追加第二段数据。
> 日期：2026-08-23，分支 `react-migration/react-foundation`，未提交。

## 1. 暂定取值（design.md §3.1 第 3 步）

输入为 `distribution.md` 的活文件集分布（217 个 `src/**/*.{ts,tsx}`，排除 `src/types/generated`，排除 21 个统一层接管文件——该排除在活文件集上是空操作，见 `distribution.md` 适配说明）。

| 指标 | 分布 P90 | 取整规则 | 暂定值 |
| --- | --- | --- | --- |
| 单文件行数（物理行） | 414 | 向上取整到 100 的倍数 | **500** |
| 圈复杂度（文件内最差函数） | 16 | 向上取整到整数 | **16** |
| 最大嵌套深度 | 3 | 向上取整到整数 | 3 → **2**（反馈轮下调，见 §2） |
| 最大参数个数 | 4 | 向上取整到整数 | 4 → **3**（反馈轮下调，见 §2） |
| 组件内样式行数 | 412 | 绝对值 | **412**（见 §4） |

行数口径：物理行（`skipBlankLines: false`、`skipComments: false`），与批次 1 测量口径一致。ESLint `max-lines` 对末尾无换行的文件计数比 `split('\n')` 少 1（如 `commandCapabilities.ts` 测量为 6,058、ESLint 报 6,057），阈值判断不受影响，豁免注释以 ESLint 上报值为准。

复杂度口径：ESLint 核心 `complexity`，默认 variant，与批次 1 临时测量配置（`max: 0` 取数）语义一致。

## 2. 反馈轮（implement.md 批次 3，design.md §3.2 第 3 步，只做一轮）

以最终规则形态（error 级，`max-lines=500` / `complexity=16` / `max-depth=3` / `max-params=4`）定向 lint 全部 217 个文件，计数如下：

| 指标 | 超限文件数 | 占比 | 判定（>15% 上调 / <3% 下调） | 调整 |
| --- | --- | --- | --- | --- |
| 行数 500 | 19 | 8.8% | 在 [3%, 15%] 带内 | 不动 |
| 圈复杂度 16 | 13 | 6.0% | 在 [3%, 15%] 带内 | 不动 |
| 嵌套深度 3 | 2 | 0.9% | **< 3%** | **下调一档至 2** |
| 参数个数 4 | 6 | 2.8% | **< 3%** | **下调一档至 3** |

**决策**：行数与圈复杂度保留 P90 值；嵌套深度 3→2、参数个数 4→3。调整后四项占比落在 [3%, 15%] 带内（复核：深度 2 → 13 文件 6.0%；参数 3 → 19 文件 8.8%）。下调方向与 design.md §3.2 语义一致——阈值取到分布 P90 后实际只拦截尾部 0.9%/2.8% 文件，说明 P90 对这两项过松，下调一档使规则真正有约束力。反馈只做一轮，不做进一步调整。

**最终生效暂定阈值**（本批次 lint 采用）：

| 规则 | 生效值 | 超限文件数 | 占比 |
| --- | --- | --- | --- |
| `max-lines` | 500 | 19 | 8.8% |
| `complexity` | 16 | 13 | 6.0% |
| `max-depth` | 2 | 13 | 6.0% |
| `max-params` | 3 | 19 | 8.8% |
| 合计（去重） | — | **49** | — |

## 3. 超限文件清单与处置（AC11）

无全局豁免。豁免均为逐文件、逐规则登记（`eslint.config.js` 的逐文件覆盖块，各带内联注释），源文件不含 `eslint-disable`。处置分两类：**注册豁免**（纯数据表 / 生成物 / 冻结门面，登记在册，阶段 4 冻结时复核）与**归迁移批次**（在对应子任务中拆分/改写，届时移除此块并恢复规则）。共 49 个文件，17 个注册豁免 + 32 个批次归属，**无未分配项**。

### 3.1 注册豁免（17 个）

| 文件 | 违规指标（实测） | 处置与理由 |
| --- | --- | --- |
| `src/api/generated/commandCapabilities.ts` | max-lines 6,057 | 生成数据表（handler_registry.rs 生成，`do not edit`），零逻辑，拆分无收益 |
| `src/i18n/locales/en-US.ts` | max-lines 5,456 | 翻译数据表，零逻辑 |
| `src/i18n/locales/zh-CN.ts` | max-lines 5,300 | 翻译数据表，零逻辑 |
| `src/i18n/bootMessages.ts` | max-lines 1,203 | 文案数据表，零逻辑 |
| `src/types/checkin.ts` | max-lines 667 | 类型数据表，零逻辑 |
| `src/types/codex.ts` | max-lines 503 | 类型数据表，零逻辑 |
| `src/api/tauri.ts` | max-lines 736 | 冻结门面（constraint C5，只读），定义侧由 `api-facade-boundary.smoke.test.ts` 冻结，不可拆分 |
| `src/api/domains/codex.ts` | max-lines 952 | 域门面：60 个 export 均为对 generated/invoke 的薄类型封装（typed wrapper facade），拆分无收益；API 层迁移期原样保留（state-logic-port Out of Scope） |
| `src/api/domains/claude.ts` | max-lines 624 | 域门面，与 `codex.ts` 同型（typed wrapper facade），API 层迁移期原样保留 |
| `src/api/generated/codex.ts` | max-params 4 | ts-rs 生成绑定，命令签名与后端一致，不可改 |
| `src/api/generated/systemPrompts.ts` | max-params 4 | ts-rs 生成绑定，不可改 |
| `src/api/generated/uiState.ts` | max-params 4 | ts-rs 生成绑定，不可改 |
| `src/api/generated/usageV2.ts` | max-params 7 | ts-rs 生成绑定，不可改 |
| `src/api/domains/environment.ts` | max-params 5 | 域门面薄封装，参数镜像后端命令签名，API 层迁移期原样保留 |
| `src/api/domains/sync.ts` | max-params 6 | 域门面薄封装，参数镜像后端命令签名，API 层迁移期原样保留 |
| `src/api/domains/systemPrompts.ts` | max-params 4 | 域门面薄封装，参数镜像后端命令签名，API 层迁移期原样保留 |
| `src/api/domains/unifiedMcp.ts` | max-params 4 | 域门面薄封装，参数镜像后端命令签名，API 层迁移期原样保留 |

### 3.2 归迁移批次（32 个）

| 文件 | 违规指标（实测） | 归属批次 |
| --- | --- | --- |
| `src/stores/usage.ts` | max-lines 991、complexity 27、max-depth 3 | `08-22-state-logic-port`（store 重写为 Zustand/TanStack Query 时拆分） |
| `src/composables/useCodexDashboard.ts` | max-lines 657、complexity 27 | `08-22-state-logic-port` |
| `src/composables/useGrokDashboard.ts` | max-lines 580、complexity 18、max-depth 3 | `08-22-state-logic-port` |
| `src/composables/useUnifiedMcp.ts` | max-lines 534、complexity 17 | `08-22-state-logic-port` |
| `src/composables/useMonitoringFeed.ts` | complexity 19 | `08-22-state-logic-port` |
| `src/composables/useStream.ts` | max-depth 4 | `08-22-state-logic-port` |
| `src/composables/useAgents.ts` | max-depth 3 | `08-22-state-logic-port` |
| `src/composables/usePolledData.ts` | max-depth 3 | `08-22-state-logic-port` |
| `src/composables/useProfilesFilter.ts` | max-depth 3 | `08-22-state-logic-port` |
| `src/composables/useProfilesInsights.ts` | max-depth 3 | `08-22-state-logic-port` |
| `src/stores/homeUsageOverview.ts` | complexity 23 | `08-22-state-logic-port` |
| `src/stores/usageDashboardPayload.ts` | max-params 4 | `08-22-state-logic-port` |
| `src/views/dashboard/dashboardPresentation.ts` | max-lines 663、complexity 27、max-params 5 | `08-22-views-usage` |
| `src/views/usage/usageOpsCockpit.ts` | max-lines 516、complexity 51、max-params 4 | `08-22-views-usage` |
| `src/views/platform-usage/platformUsagePresentation.ts` | complexity 18、max-params 5 | `08-22-views-usage` |
| `src/views/usage/usageChartOptions.ts` | max-params 4 | `08-22-views-usage` |
| `src/views/usage/usageOverviewInsights.ts` | max-params 4 | `08-22-views-usage` |
| `src/views/usage/usageSummaryCards.ts` | max-params 5 | `08-22-views-usage` |
| `src/views/checkin/composables/useCheckinState.ts` | max-lines 569 | `08-22-views-checkin` |
| `src/views/checkin/composables/balanceRefreshQueue.ts` | max-depth 3 | `08-22-views-checkin` |
| `src/views/checkin/composables/checkinJobRuntime.ts` | max-params 4、max-depth 3 | `08-22-views-checkin` |
| `src/views/checkin/composables/checkinWafRecovery.ts` | max-params 4、max-depth 4 | `08-22-views-checkin` |
| `src/utils/claudeProfiles.ts` | max-lines 521 | `08-22-views-profiles-config`（其关联资产清单含 `claudeProfiles.ts`） |
| `src/utils/providerTemplates.ts` | max-lines 513、complexity 26 | `08-22-views-profiles-config` |
| `src/utils/claudeProfileEditor.ts` | complexity 25 | `08-22-views-profiles-config` |
| `src/configs/providersCatalog.ts` | complexity 20 | `08-22-views-profiles-config`（provider 模板消费链） |
| `src/utils/grokProfileEditor.ts` | complexity 20 | `08-22-views-secondary-platforms`（消费方 GrokProfilesView/GrokProfileEditorModal） |
| `src/router/index.ts` | max-lines 594 | `08-22-shell-port`（Scope 明确含 `src/router/index.ts`，75 条路由迁移时改写） |
| `src/utils/logger.ts` | max-depth 3 | `08-22-shell-port`（通用日志收口，`no-console` 依赖） |
| `src/utils/errorHandler.ts` | max-depth 3 | `08-22-shell-port`（通用错误处理工具） |
| `src/utils/logRedact.ts` | max-params 4、max-depth 3 | `08-22-shell-port`（凭据脱敏通用工具，logger 依赖） |
| `src/i18n/formatMessage.ts` | max-params 4 | `08-22-i18n-port`（i18n 运行时迁移） |

## 4. 组件内样式行数

**绝对上限 412**：来源为 139 个历史 `.vue` 组件局部样式行数分布的 P90（`distribution.md`，P50 131 / P75 238 / P90 412 / max 806）。当前 `.tsx` 活文件集无组件级局部样式（无 `.module.css`），约束对现状零违例；基线在首个带样式组件落地后于批次 3b 补测。

**比例约束**（父任务 `design.md` §6）：单组件局部样式行数 ≤ 其 JSX 行数。实现为 `ccr-ui/scripts/check-component-style-lines.mjs`（`bun run check:style-lines`），不用 ESLint（需同时读 `.tsx` 与配对的 `.module.css`）。

脚本口径（记录在脚本头，作为约定基准）：

- 扫描 `src/**/*.module.css`，与消费组件配对：同目录同名 `.tsx` 优先（`Foo.module.css` ↔ `Foo.tsx`），否则解析同目录 `.tsx` 的 `import ... from './*.module.css'`。
- `styleLines` = `.module.css` 物理行数（`split('\n').length`，与分布测量口径一致）。
- `jsxLines` = 消费 `.tsx` 的非空、非纯注释源码行数（JSX 行数的简单代理；此口径记录在脚本头作为约定基准）。
- 两个约束：`styleLines ≤ 412`；`styleLines ≤ jsxLines`。任一违反输出违规清单，退出码 1。
- `.vue` 的 `<style>` 块不检查：`.vue` 已整体退出 lint 管线，阶段 4–5 离开树（implement.md 批次 3）。

检查已接入 `package.json` 的 `check:style-lines`，并追加进 `lint:ci`（保持 AC1 的 `bun run lint:ci` 表面）。

## 5. 生效位置

- `eslint.config.js` `app/threshold-rules` 块：`max-lines` / `complexity` / `max-depth` / `max-params` 四项 error 级，作用域 `src/**/*.{ts,tsx,mts}`。
- `eslint.config.js` 逐文件豁免块（49 个文件，各带处置注释）。
- `ccr-ui/scripts/check-component-style-lines.mjs` + `package.json` 的 `check:style-lines` 与 `lint:ci` 接线。

## 6. 最终值冻结（阶段 4 → 5 门，批次 3b，协同点 N）

**最终值待阶段 4 冻结（批次 3b，协同点 N）**。届时：

1. 用统一层（`08-22-platform-unify` 批次 6）的实际文件集合替换批次 1 排除的 21 个条目，重算分布（`scripts/measure-distribution.mjs` 已保留排除逻辑）。
2. 重取 P90，按同样的取整规则与反馈轮规则得到最终值；与暂定值不同时以最终值为准。
3. 超限清单重出；新增项分配处理批次。
4. 本文追加第二段数据，`eslint.config.js` 阈值更新。
