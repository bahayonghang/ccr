# 执行计划：跨平台功能面统一

> 父任务：`08-22-react-migration`（阶段 4）。
> 分支：`feature/react-migration/platform-unify`，PR 目标 `feature/react-migration`。
>
> 差异普查（批次 1）可在父任务约束门通过后即启动，不必等 `08-22-shell-port` 完成。统一层接口必须在阶段 5 开始前定稳。

## 前置确认

- [x] 父任务约束门已通过（`08-22-arch-quality-perf` 与 `08-22-design-system` 的 AC 全满足）。
- [x] 批次 2 起需 `08-22-shell-port` 已交付（`shared-interfaces.md` 已公示）。该任务已归档。descriptor 已扩展 `surfaces`；`routeCatalog` 仍用 `genericPlatformDescriptors` 生成 gemini 三条路由，75 条路径不变。归档任务无需改代码。
- [x] **批次 4 起需父任务阶段 4a 共享层前置门已通过**：`components/profiles/` 10 文件已迁为 React 且 `profiles-shared-interfaces.md` 已落盘（协同点 F）。
  - 通知（`08-22-views-profiles-config` 批次 1）：React 模块在 `ccr-ui/src/components/profiles/*.tsx`，接口见 `.trellis/tasks/08-22-views-profiles-config/profiles-shared-interfaces.md`。`BaseProfiles` 经 `features/platform/profiles/shared.ts` 再导出（boundaries 豁免）。
- [x] **批次 5 起需** `components/mcp/` 4 文件已迁为 React 且 `mcp-shared-interfaces.md` 已落盘（协同点 F2）。2026-08-24：面板迁到 `ccr-ui/src/features/platform/mcp/`，`features/mcp` 再导出。接口：`.trellis/tasks/08-22-views-sync-tools/mcp-shared-interfaces.md`。
- [x] 读 `src/components/BaseSlashCommands.vue`（507 行）与 `src/configs/slashCommands.ts`（192 行）——参照实现。
- [x] 留在 `react-migration/react-foundation`（主会话指令：不新开 `feature/react-migration/platform-unify`）。

## 批次 1：差异普查（可提前启动）

按 `design.md` §1 的六个维度，对 7 个功能面的每个平台文件逐项抽取。

- [x] Settings：`ClaudeCodeSettingsView`(1,325)、`grok/GrokSettingsView`(1,245)、`CodexSettingsView`(1,023)、`OpenCodeSettingsView`(330)。
- [x] Profiles：`CodexProfilesView`(1,173)、`grok/GrokProfilesView`(1,078)、`ClaudeCodeProfilesView`(1,074)。
- [x] Auth：`ClaudeAuthView`(1,179)、`CodexAuthView`(958)、`grok/GrokAuthView`(161)。
- [x] Commands：`CommandsView`(1,744)、`OpenCodeCommandsView`(346)。
- [x] MCP：`CodexMcpView`(1,301)、`OpenCodeMcpView`(433)，对照 `generic/PlatformMcpView`(407)。
- [x] Agents：`codex/CodexAgentsView`(1,138)、`OpenCodeAgentsView`(442)，对照 `generic/AgentsView`(725)。
- [x] Plugins：`PluginsView`(426)、`OpenCodePluginsView`(296)，对照 `generic/PlatformPluginsView`(367)。
- [x] 集合比较，输出共有项与平台独有项，每项定「归属」列。
- [x] 项的拆分按 `design.md` §5.1 的原子化规则，不按执行者习惯（七个面均适用）。
- [x] `diff-matrix.md` 落盘，无未确认项（AC1）。

普查在 Vue 源码上进行——此时这些文件尚未迁移。普查产出与框架无关。

## 批次 2：Auth 面判定

- [x] 按 `design.md` §5.2 计两个量：实际（假设全统一时的）分支点数，以及各差异项的理论分支点数之和。
- [x] 汇总评审门的三项输入（`design.md` §5.3）：比值、保留重复的成本（不统一时需在 N 个平台各改一次的行为条目数）、统一后的条件密度（分支点数 / base 行数，与 `BaseSlashCommands` 参照对比）。
- [x] 取「全统一 / 部分统一 / 不统一」之一。判定为部分统一时，子集须覆盖 ≥2 个平台且 ≥3 个差异项，明确其边界。
- [x] 判定过程的三项输入与结论一并写入 `diff-matrix.md`（AC2 的记录依据要求）。结论：部分统一（session / refresh / auth-off / local-only / confirm-off）。
- [x] 保留的文件从 AC3 的行数对比基线中排除。

判定必须在批次 4 之前完成——Auth 是否统一决定 `src/configs/auth.ts` 是否存在。

## 批次 3：两层 config 契约

- [x] 按 `design.md` §2 扩展 `src/config/platformDescriptors.ts`（现 50 行）。`platformCapabilities.ts`（74 行）不动。
- [x] 从差异矩阵的「归属 = config.*」项推出各 `<Surface>Config` 接口的字段集合。
- [x] 字段设计遵守三条原则：平台独有用可选、取值不同用必填、不设 `platform` 标识字段。
- [x] 建 `src/configs/settings.ts`、`profiles.ts`、`commands.ts`，以及按批次 2 判定的 `auth.ts`。`slashCommands.ts` 不动。
- [x] 通知 `08-22-shell-port` 替换路由生成的输入（其 `design.md` §2 已预留形状）。该任务已归档：见本文件前置确认第二条。
- [x] 契约文档 `platform-surface-contracts.md` 落盘到 `.trellis/spec/ccr-ui/frontend/`，并登记到 `08-22-test-contract-rebuild` 的范围表（使其成为第 19 份）（AC9）。文件名已定，不再待议。

**本批次是阶段 4 → 5 门的准出项**。接口定稳后阶段 5 期间不改——变更成本随五个并行子任务推进而上升（PRD Notes）。

验证：`bun run type-check`；75 条路由路径不变，路由清单比对通过（AC8）。

## 批次 4：新建 base 组件（Settings / Profiles / Commands / Auth）

按功能面分四次提交。

- [x] `Base Settings`：承载差异矩阵中「归属 = base」的项，4 个平台 config 填「归属 = config.*」的项。
- [x] `Base Profiles`：复用 `components/profiles/`（10 文件 4,040 行）共享层。**前置**：该共享层已由 `08-22-views-profiles-config` 批次 1 迁为 React 且接口已公示（阶段 4a、协同点 F）。本任务不改造该共享层，也不代为迁移。
- [x] `Base Commands`。
- [x] `Base Auth`：按批次 2 判定执行（全统一 / 部分统一 / 不统一）。
- [x] 每个 base 组件的 lint 检查：无平台名称条件分支（AC4）。
- [x] `no-restricted-syntax` 的匹配模式提交给 `08-22-arch-quality-perf` 的规则集，或直接加入 `eslint.config.js`。

## 批次 5：收敛到 `generic/`（MCP / Agents / Plugins）

每个面两步，先补齐后接入（R7 的顺序要求）。

**前置**：`components/mcp/`（4 文件 2,064 行）已由 `08-22-views-sync-tools` 批次 3 前半迁为 React 且接口已公示（阶段 4a、协同点 F2）。**已交付**：面板在 `ccr-ui/src/features/mcp/`，接口见对方 `mcp-shared-interfaces.md`。`t` 由父级注入；`McpCreatePanel` 必填 `formApi`。

- [x] `PlatformMcpView`：从差异矩阵推出缺失能力 → 补齐 → 接入 Codex 与 OpenCode 调用点。
- [x] `AgentsView`：同上。
- [x] `PlatformPluginsView`：同上。
- [x] 与 `08-22-views-secondary-platforms` 划清 `views/generic` 的归属（协同点 G）：本任务接收 `AgentsView`、`PlatformMcpView`、`PlatformPluginsView`；对方保留 `AgentDetailView`(481)、`SystemPromptsView`(655)。

先接入后补齐会导致接入期间该平台功能缺失，因此两步顺序不可颠倒。

## 批次 6：薄壳视图

- [x] 每个平台每个面一个薄壳，形如现有 `SlashCommandsView.vue`（18 行）：`PageShell` + `PageHeader` + `<BaseX config={...} />`。
- [x] 行数不超过 100 行（R5、AC7）。超出项说明原因。
- [x] 20 个原重复实现文件全部处理：或收敛为薄壳，或按 R6 判定保留（AC2）。

验证：`bun run type-check`、`bun run lint:ci`（AC11）。

## 批次 7：追溯表与验证矩阵

- [x] 追溯表：差异矩阵每一项 → 统一后的 config 字段名或 props 名（AC5）。
- [x] 验证矩阵：按「平台 × 功能面」组织，每格列核心操作路径清单与验证结果。无未验证格（AC6）。
- [x] 统一后总行数统计，与 15,672 行基线对比（AC3）。计数范围见 `design.md` §7。
- [x] 跨平台验证用例：改一处共性行为，断言全部消费平台同时生效（AC10）。
- [x] 把统一层的实际文件集合与行数交 `08-22-arch-quality-perf`，触发其批次 3b 的阈值冻结（协同点 N）。

## 批次 8：范围回填

阶段 4 → 5 门的准出项。

- [x] 按 `design.md` §8 回填五个子任务的范围表：`08-22-views-claude`、`views-codex`、`views-secondary-platforms`、`views-profiles-config`、`views-sync-tools`。
- [x] 回填内容为各子任务的行数与文件清单，并明确其工作从「迁移 N 行」变为「填 config + 写薄壳」。
- [x] `08-22-views-checkin` 与 `08-22-views-usage` 不受影响，不回填。

## 验证命令

| 时机        | 命令                                      |
| ----------- | ----------------------------------------- |
| 批次 3 后   | `bun run type-check`、路由清单比对        |
| 批次 4–6 后 | `bun run type-check`、`bun run lint:ci`   |
| 批次 7 后   | `bun run test:smoke`（AC10 的跨平台用例） |
| 交付前      | `just frontend-check-quick`               |

## 交付门（父任务统一层门）

- [x] AC1（差异矩阵）先行落盘 —— 该项是门的第一条准出条件，早于其余 AC。
- [x] 统一层接口契约文档落盘（AC9）。
- [x] 五个受影响子任务的范围表已回填（批次 8）。
- [x] AC2–AC11 全部满足。
- [x] 三张表落盘：`diff-matrix.md`、追溯表、验证矩阵。三者共用同一份项清单。
- [x] base 组件内无平台名称条件分支，检查规则可运行（AC4）。
- [x] 行数对比数据落盘（AC3）。

## 回滚点

| 批次     | 回滚方式                                                                 |
| -------- | ------------------------------------------------------------------------ |
| 1–2、7–8 | 只产出文档与判定，revert 无代码影响                                      |
| 3        | config 契约。回滚会波及批次 4–6 与五个视图子任务，代价最高。定稳后不回滚 |
| 4        | 按功能面分四次提交，可按面回退                                           |
| 5        | 按功能面分三次提交，每面「补齐」与「接入」可分两次                       |
| 6        | 按平台分次提交                                                           |

批次 3 定稳后不回滚：接口变更成本随五个并行子任务推进而上升。若必须变更，需同时通知五个子任务并评估其返工量。

## 协同点

| 编号 | 内容                                                                             | 对方                                                                                                    | 时机           |
| ---- | -------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------- | -------------- |
| E    | 统一层接口定稳，五个视图子任务消费                                               | `views-claude`、`views-codex`、`views-secondary-platforms`、`views-profiles-config`、`views-sync-tools` | 批次 3         |
| F    | `components/profiles/` 迁移 + 接口公示，**须早于本任务批次 4**                   | `08-22-views-profiles-config`                                                                           | 阶段 4a（前置） |
| F2   | `components/mcp/` 迁移 + 接口公示，**须早于本任务批次 5**                        | `08-22-views-sync-tools`                                                                                | 阶段 4a（前置） |
| G    | `views/generic` 的归属划分                                                       | `08-22-views-secondary-platforms`                                                                       | 批次 5         |
| N    | 统一层实际文件集合与行数，触发对方的阈值冻结                                     | `08-22-arch-quality-perf`                                                                               | 批次 7         |
| —    | descriptor 扩展后替换路由生成的输入                                              | `08-22-shell-port`                                                                                      | 批次 3         |
| —    | `platform-surface-contracts.md` 登记为第 19 份契约                               | `08-22-test-contract-rebuild`                                                                           | 批次 3         |
| —    | 平台条件分支的 lint 匹配模式                                                     | `08-22-arch-quality-perf`                                                                               | 批次 4         |
