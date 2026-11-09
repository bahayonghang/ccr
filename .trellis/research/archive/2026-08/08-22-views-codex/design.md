# 技术设计：Codex 视图迁移

> 父任务：`08-22-react-migration`。本批次是七个视图子任务中规模最大的一批（13,226 行，移交统一层后剩约 7,633 行）。

## 1. 本批次共性转换

与 `08-22-views-claude` 的 `design.md` §1 同表，不重复。前置阅读同。

## 2. 本域范围与统一层切分

范围变更（PRD Scope）：5 个文件、5,593 行移交 `08-22-platform-unify`：

| 文件                        | 行数  | 本任务的新工作                                                            |
| --------------------------- | ----- | ------------------------------------------------------------------------- |
| `CodexMcpView.vue`          | 1,301 | 收敛到 `generic/PlatformMcpView`，提供调用点                              |
| `CodexProfilesView.vue`     | 1,173 | 填 `codexProfilesConfig` + 薄壳                                           |
| `codex/CodexAgentsView.vue` | 1,138 | 收敛到 `generic/AgentsView`，提供调用点                                   |
| `CodexSettingsView.vue`     | 1,023 | 填 `codexSettingsConfig` + 薄壳。该文件有 33 处 `v-model`，是全仓密度最高 |
| `CodexAuthView.vue`         | 958   | 按 Auth 面判定执行                                                        |

`CodexSlashCommandsView.vue`（229 行）按现有薄壳模式处理——它已是薄壳形态但比另两个平台的薄壳长（18 / 27 行），迁移时核对超出部分是 Codex 独有差异还是可下沉到 `BaseSlashCommands` 的共性。

本任务自有迁移范围约 7,633 行：`views/codex/`（6 文件 3,578，扣除 `CodexAgentsView` 1,138 后为 2,440，含 `AddCodexAccountModal` 1,179）、`CodexSessionsView`(883)、`CodexView`(880)、`components/codex/`（5 文件 3,201）、`CodexSlashCommandsView`(229)。

精确切分在 `platform-unify` 批次 8 回填。

## 3. `AddCodexAccountModal`（1,179 行，本域最大文件）

含 OAuth 流程与凭据处理（PRD Notes）。

设计要点：

- 多步向导的步骤状态属组件本地态，用 `useState` 或 `useReducer`。步骤间的表单数据用 react-hook-form 的单个 form 实例跨步骤共享，不每步一个 form。
- 凭据字段的脱敏：界面显示掩码，日志经 `logRedact.ts`。迁移后不得出现明文（与 `08-22-views-profiles-config` R7 同类要求）。
- OAuth 回调的等待：现状机制需在迁移时确认（Tauri Event、轮询、或 WebView 导航监听）。等待逻辑的超时与取消分支逐个保留。
- 该文件规模超过 `08-22-arch-quality-perf` 的文件行数上限（阈值按其 §3 取值，1,179 行大概率超限）。因此迁移时需拆分。拆分边界按向导步骤划分，每步一个组件，共享 form context。

## 4. Codex auth off 契约

R6：Codex auth off 契约保留（对应 git 历史 `feab8669 docs(规范): 写入 auth off 契约`）。

处理：迁移前读该规范文档，抽出其断言项，逐条在迁移后验证（AC6）。断言项清单落盘。

该契约的规范文档位置需在实施时定位（`.trellis/spec/` 下或 `docs/` 下）。

## 5. `codex-auth-shared.css`（14.8 KB）

R5：该文件的样式在 token 体系下重新落位，由 `08-22-design-system` 定义的分层承载。

判定（按 `08-22-design-system` §3 的标准）：其规则若只服务 Codex Auth 单一路由，进 `features/codex/` 下的 `.module.css`；若被 Codex 的多个面共享，进 `styles/components/`。

AC5 要求 14.8 KB 样式全部落位，无遗留未归类规则。核对方式：逐个选择器确认其新位置，产出落位清单。

Auth 面若判定为统一，该样式的一部分需下沉到统一层的 base 组件；判定为不统一则全部留在本域。

## 6. `CodexSessionsView`(883) 与 `CodexView`(880)

| 视图                | 要点                                                                                                                                       |
| ------------------- | ------------------------------------------------------------------------------------------------------------------------------------------ |
| `CodexSessionsView` | 会话列表浏览。若为长列表，用 `@tanstack/react-virtual` 3.14.10。虚拟滚动的替换在 `08-22-views-usage` R9 落地，本域若也需要则复用其接线形态 |
| `CodexView`         | 平台首页，聚合入口。消费原语，按 `primitive-disposition.md` 适配                                                                           |

## 7. `components/codex/`（5 文件 3,201 行）

域组件层。依赖方向自检：只可导入 `features/platform`、`ui`、`api`、`types`，不可导入其他 `features/<平台>/`。

均值 640 行/文件，大概率有文件超过行数上限，需在迁移时拆分。拆分不改变对外接口。

## 8. 框架无关资产

`src/utils/codexProfileEditor.ts`、`codexProfiles.ts`、`codexHelpers.ts` 由 `08-22-react-foundation` 判定为原样复用，本任务只改调用点（PRD Notes）。

若发现这三个文件需要修改，登记为独立缺陷，不在本任务改（与 `src/api` 同类约束）。

## 9. 不变量

- IPC 调用点沿用现有 wrapper（R4）。`git diff --stat src/api` 须为空（AC7）。
- `src/types` 与 `src-tauri` 不改。

## 10. 未决项

- Auth 面判定结果决定 `CodexAuthView`(958) 与 `codex-auth-shared.css` 的归属。
- OAuth 回调的等待机制（第 3 节第 3 条），实施时确认。
- Codex auth off 契约文档的位置（第 4 节）。
- `CodexSlashCommandsView` 超出另两个平台薄壳的 202 行是差异还是可下沉的共性（第 2 节）。
- 本任务的精确文件清单待 `platform-unify` 批次 8 回填。
