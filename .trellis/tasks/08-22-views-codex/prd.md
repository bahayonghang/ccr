# Codex 视图迁移

> 父任务：`08-22-react-migration`

## Goal

将 Codex 平台的全部视图与组件从 Vue 迁移到 React，约 13,226 行。本批次是七个视图子任务中规模最大的一批。

## Scope

> **范围回填（`08-22-platform-unify` 普查后，2026-08-24）**：Settings / Profiles / MCP / Agents 已由统一层提供 config + React 薄壳。本任务改为删除对应 `.vue` 并把路由接到薄壳。Auth 判定为部分统一：session/auth-off 在 `BaseAuth`，`CodexAuthView.vue` 与 `AddCodexAccountModal.vue` 的 OAuth/配额/Provider 向导保留在本任务。`CodexSlashCommandsView.vue`（229）按现有薄壳模式接到 React。

| 文件 / 目录 | 行数 | 处置 |
|---|---|---|
| `src/views/codex/`（除 `CodexAgentsView.vue`：含 `AddCodexAccountModal.vue` 1,179） | 2,440 | 本任务（OAuth 向导等） |
| `src/views/CodexAuthView.vue` | 958 | 本任务迁 React（OAuth 保留） |
| `src/views/CodexSessionsView.vue` | 883 | 本任务 |
| `src/views/CodexView.vue` | 880 | 本任务 |
| `src/views/CodexSlashCommandsView.vue` | 229 | 薄壳接到 React |
| `src/components/codex/`（5 文件） | 3,201 | 本任务 |
| `CodexMcpView` / `CodexProfilesView` / `CodexAgentsView` / `CodexSettingsView` | 4,635 | 统一层已提供薄壳，本任务只删 Vue + 接路由 |
| 合计（仍迁实现） | 8,591 | |

覆盖的功能面：Profiles、MCP、Agents、斜杠命令、插件、Auth、Sessions、Settings。

## Requirements

- R1 上表全部文件迁移为 React 组件，对应 `.vue` 文件删除。
- R2 本批次内的 `v-model` 展开为受控属性与回调对，slot 转为 children 或 render props。
- R3 消费 `08-22-design-system` 产出的原语与 token，本批次不新增硬编码样式值。
- R4 IPC 调用点沿用 `src/api` 现有 wrapper，不新增或修改 wrapper。
- R5 `src/styles/codex-auth-shared.css`（14.8 KB）的样式在 token 体系下重新落位，由 `08-22-design-system` 定义的分层承载。
- R6 Codex auth off 契约保留（见 git 历史 `feab8669 docs(规范): 写入 auth off 契约`，对应规范文档需在迁移后仍成立）。
- R7 落在本批次的 `nextTick` 调用逐点登记与改写。
- R8 页面内的确认与批量操作行为遵循 `confirm-interaction-contracts.md`。

## Acceptance Criteria

- [ ] AC1 上表 18 个文件全部迁移，`rg --files -g '*.vue' src/views/codex src/views/Codex* src/components/codex` 无匹配。
- [ ] AC2 7 个根级视图与 `views/codex/` 下视图的路由可达，页面渲染无报错。
- [ ] AC3 每个视图的核心操作路径手动验证通过并记录：Profiles 增删改切换、MCP 服务器管理、Agents 管理、斜杠命令增删、Auth 登录与 auth off、Sessions 浏览、Settings 读写、账号添加向导。
- [ ] AC4 本批次组件内 px 字面量与 `rgba()` 数量为 0（登记豁免除外）。
- [ ] AC5 `codex-auth-shared.css` 的 14.8 KB 样式全部落位，无遗留未归类规则。
- [ ] AC6 Codex auth off 行为验证通过。
- [ ] AC7 `src/api` 的 git diff 为空。
- [ ] AC8 `nextTick` 登记表落盘，本批次内调用点全部有改写说明。
- [ ] AC9 `bun run type-check` 与 `bun run lint` 退出码 0。
- [ ] AC10 本批次相关的 smoke 测试通过。

## 前置与后续

- 前置：`08-22-shell-port`。
- 可与其余六个视图子任务并行。
- i18n 调用点在本批次内同步转换，运行时切换与收尾校验属 `08-22-i18n-port`。

## Out of Scope

- 新增功能与信息架构调整。
- `src/api` 与 `src/types` 的修改。
- `src-tauri` 侧改动。
- 共享原语与 token 的形态决策（属 `08-22-design-system`）。

## Notes

- `AddCodexAccountModal.vue`（1,179 行）与 `CodexAuthView.vue`（958 行）含 OAuth 与凭据处理，迁移时需保留脱敏行为（`logRedact.ts`）。
- `src/utils/codexProfileEditor.ts`、`codexProfiles.ts`、`codexHelpers.ts` 为纯逻辑，由 `08-22-react-foundation` 判定为原样复用，本任务只改调用点。
- 本批次规模最大，建议在 `implement.md` 中拆为 3 个提交批次：`views/codex/` 目录、7 个根级视图、`components/codex/`。
