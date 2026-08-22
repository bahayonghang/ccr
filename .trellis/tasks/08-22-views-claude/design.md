# 技术设计：Claude Code 视图迁移

> 父任务：`08-22-react-migration`。共性转换规则见本文件第 1 节，其余为本域特有设计。

## 1. 本批次共性转换

七个视图子任务共用的转换规则，各子任务不重复决策：

| Vue                  | React                                                              | 来源                                         |
| -------------------- | ------------------------------------------------------------------ | -------------------------------------------- |
| `v-model="x"`        | `value={x} onChange={...}`，表单内改 react-hook-form 的 `register` | 父任务 `design.md` §7                        |
| 具名 slot            | props 传 ReactNode                                                 | `08-22-shell-port` 的 `shared-interfaces.md` |
| 默认 slot            | `children`                                                         | 同上                                         |
| 作用域 slot          | render props                                                       | 同上                                         |
| `v-if` / `v-for`     | 条件表达式 / `map`，`key` 不用数组索引                             | `react-rerender-discipline.md`               |
| `<style scoped>`     | Tailwind 工具类为主，残余进 `.module.css`                          | `08-22-design-system` §4 的判定标准          |
| px / `rgba()` 字面量 | 查 `hardcode-mapping.md` 换 token                                  | `08-22-design-system` §5                     |
| `nextTick`           | 按意图查替代表                                                     | `08-22-state-logic-port` §6.4                |
| `$t('key')`          | `t('key')`（`useTranslation`）                                     | `08-22-i18n-port` 的调用形式约定             |

前置阅读（动手前）：`react-rerender-discipline.md`、`shared-interfaces.md`、`hardcode-mapping.md`、`primitive-disposition.md`、本域的契约重写稿（协同点 D）。

## 2. 本域范围与统一层切分

范围变更（PRD Scope）：4 个文件、4,004 行移交 `08-22-platform-unify`。本任务对这 4 个面改为提供 Claude 平台的 config 与薄壳：

| 文件                         | 行数  | 本任务的新工作                                                                    |
| ---------------------------- | ----- | --------------------------------------------------------------------------------- |
| `ClaudeCodeSettingsView.vue` | 1,325 | 填 `configs/settings.ts` 的 `claudeSettingsConfig` + 薄壳                         |
| `ClaudeAuthView.vue`         | 1,179 | 按 Auth 面判定（`platform-unify` 批次 2）执行。判定为不统一时本任务完整迁移该文件 |
| `ClaudeCodeProfilesView.vue` | 1,074 | 填 `claudeProfilesConfig` + 薄壳                                                  |
| `PluginsView.vue`            | 426   | 收敛到 `generic/PlatformPluginsView`，本任务提供调用点                            |

`SlashCommandsView.vue`（18 行）已是薄壳，只做框架转换。

本任务自有迁移范围约 6,985 行：`HooksView`(920)、`ClaudeCodeView`(745)、`OutputStylesView`(558)、`SkillsMigrationView`(392)、`StatuslineView`(230)、`components/claude/`(3 文件 1,869)、`components/claude-observer/`(7 文件 2,253)。

精确切分在 `platform-unify` 批次 8 回填。

## 3. claude-observer 的事件流（本域主要风险）

`src/components/claude-observer/`（7 文件 2,253 行）消费 `claudeObserver` 的数据。

父任务 `design.md` §4 的处理：事件流数据 → TanStack Query（配合 Event 订阅失效），UI 态 → Zustand。`08-22-state-logic-port` 已完成 store 侧拆分与事件桥接层。

本任务的接线要求：

- 数据读取走 `claudeObserverKeys` 的 Query hook，不自行 `listen()`。事件订阅集中在 `shell/eventBridge.ts`（`08-22-state-logic-port` §3）。
- 若某个观测面需要组件级订阅（桥接层无法覆盖），该订阅的建立与解绑在 `useEffect` 内，StrictMode 下不双订阅。卸载后解绑由 AC6 验证。
- 事件名不变（R5）。
- `TokenDetailTab.vue` 与主题 token 存在耦合（PRD Notes），迁移时按 `token-classification.md` 核对其消费的变量名仍存在。

事件名与 Query key 的对应关系由 `08-22-state-logic-port` 批次 7 通知本任务。

## 4. HooksView 的表单

`HooksView`(920 行) 有 15 处 `v-model`，是本域 `v-model` 密度最高的文件（全仓密度最高的是 `CodexSettingsView` 33 处、`ClaudeCodeSettingsView` 32 处，两者已移交统一层）。

处理：react-hook-form 的非受控注册。Hook 配置为动态数组（增删 hook 条目），用 `useFieldArray`。校验用 zod schema + `@hookform/resolvers`。

非受控注册使输入不触发父组件重渲染，直接服务 `08-22-arch-quality-perf` 场景 1 的输入延迟指标。

## 5. 其余视图的要点

| 视图                        | 要点                                                                           |
| --------------------------- | ------------------------------------------------------------------------------ |
| `ClaudeCodeView`(745)       | 平台首页，聚合入口。消费 `StatTile` 等原语，按 `primitive-disposition.md` 适配 |
| `OutputStylesView`(558)     | 样式切换。与 CCR 自身主题体系无关，token 命名空间不得交叉                      |
| `SkillsMigrationView`(392)  | 迁移向导，多步流程。步骤状态属组件本地态，用 `useState`                        |
| `StatuslineView`(230)       | 配置表单，同第 4 节处理                                                        |
| `components/claude/`(1,869) | 3 个文件，域组件。依赖方向：只可导入 `features/platform`、`ui`、`api`、`types` |

## 6. 契约

| 契约                                          | 处理                                                                                  |
| --------------------------------------------- | ------------------------------------------------------------------------------------- |
| `development-resource-contracts.md`（5.3 KB） | 覆盖的资源管理行为不变（R8）。重写稿由 `08-22-test-contract-rebuild` 提供（协同点 D） |
| `confirm-interaction-contracts.md`（3.6 KB）  | 确认与批量操作行为遵循（R7）。底座由 `08-22-shell-port` 提供                          |

## 7. 不变量

- IPC 调用点沿用 `src/api` 现有 wrapper，不新增不修改（R4）。`git diff --stat src/api` 须为空（AC5）。
- `src/types` 不改。
- `src-tauri` 不改。

## 8. 未决项

- Auth 面的统一判定结果决定 `ClaudeAuthView`(1,179 行) 是否留在本任务（第 2 节）。
- 本任务的精确文件清单待 `platform-unify` 批次 8 回填。
- `claude-observer` 是否需要组件级订阅（第 3 节第 2 条），按桥接层的覆盖范围在实施时确定。
