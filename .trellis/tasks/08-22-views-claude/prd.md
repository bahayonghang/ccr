# Claude Code 视图迁移

> 父任务：`08-22-react-migration`

## Goal

将 Claude Code 平台的全部视图与组件从 Vue 迁移到 React，约 10,989 行。

## Scope

> **范围变更（跨平台统一决策后）**：以下 4 个文件、4,004 行移交子任务 `08-22-platform-unify` 的统一层，本任务改为提供 Claude 平台的 config 与薄壳视图：`ClaudeCodeSettingsView.vue`（1,325）、`ClaudeAuthView.vue`（1,179）、`ClaudeCodeProfilesView.vue`（1,074）、`PluginsView.vue`（426）。`SlashCommandsView.vue`（18）已是薄壳，无需处理。本任务剩余约 6,985 行。精确切分由 `08-22-platform-unify` 的差异普查（R1）确定后回填本表。

| 文件 / 目录 | 行数 |
|---|---|
| `src/views/ClaudeCodeSettingsView.vue` | 1,325 |
| `src/views/ClaudeAuthView.vue` | 1,179 |
| `src/views/ClaudeCodeProfilesView.vue` | 1,074 |
| `src/views/HooksView.vue` | 920 |
| `src/views/ClaudeCodeView.vue` | 745 |
| `src/views/OutputStylesView.vue` | 558 |
| `src/views/PluginsView.vue` | 426 |
| `src/views/SkillsMigrationView.vue` | 392 |
| `src/views/StatuslineView.vue` | 230 |
| `src/views/SlashCommandsView.vue` | 18 |
| `src/components/claude/`（3 文件） | 1,869 |
| `src/components/claude-observer/`（7 文件） | 2,253 |
| 合计 | 10,989 |

覆盖的功能面：MCP 服务器、Agents、斜杠命令、插件、Settings、Hooks、Auth、Output Styles、Statusline、Skills 迁移、Claude 观测。

## Requirements

- R1 上表全部文件迁移为 React 组件，对应 `.vue` 文件删除。
- R2 本批次内的 `v-model` 展开为受控属性与回调对，slot 转为 children 或 render props。
- R3 消费 `08-22-design-system` 产出的原语与 token，本批次不新增硬编码样式值。
- R4 IPC 调用点沿用 `src/api` 现有 wrapper，不新增或修改 wrapper。
- R5 `src/components/claude-observer/` 的观测数据流沿用 `claudeObserver` store 与 Tauri Event 订阅，事件名不变。
- R6 落在本批次的 `nextTick` 调用逐点登记与改写。
- R7 页面内的确认与批量操作行为遵循 `confirm-interaction-contracts.md`。
- R8 `development-resource-contracts.md` 覆盖的资源管理行为不变。

## Acceptance Criteria

- [ ] AC1 上表 22 个文件全部迁移，`rg --files -g '*.vue' src/views/Claude* src/views/Hooks* src/views/OutputStyles* src/views/Plugins* src/views/SkillsMigration* src/views/Statusline* src/views/SlashCommands* src/components/claude src/components/claude-observer` 无匹配。
- [ ] AC2 10 个视图的路由可达，页面渲染无报错。
- [ ] AC3 每个视图的核心操作路径手动验证通过并记录：Settings 读写、Profiles 切换、Auth 登录、Hooks 增删、插件安装、Skills 迁移、Statusline 配置、Output Styles 切换、斜杠命令增删、观测数据刷新。
- [ ] AC4 本批次组件内 px 字面量与 `rgba()` 数量为 0（登记豁免除外）。
- [ ] AC5 `src/api` 的 git diff 为空。
- [ ] AC6 Claude 观测的 Tauri Event 订阅在页面卸载后正确解绑。
- [ ] AC7 `nextTick` 登记表落盘，本批次内调用点全部有改写说明。
- [ ] AC8 `bun run type-check` 与 `bun run lint` 退出码 0。
- [ ] AC9 本批次相关的 smoke 测试通过（`claude-auth-view`、`claude-code-view`、`claude-observer-tabs` 等）。

## 前置与后续

- 前置：`08-22-shell-port`。
- 可与 `08-22-views-codex`、`08-22-views-secondary-platforms`、`08-22-views-checkin`、`08-22-views-usage`、`08-22-views-profiles-config`、`08-22-views-sync-tools` 并行。
- i18n 调用点在本批次内同步转换，运行时切换与收尾校验属 `08-22-i18n-port`。

## Out of Scope

- 新增功能与信息架构调整。
- `src/api` 与 `src/types` 的修改。
- `src-tauri` 侧改动。
- 共享原语与 token 的形态决策（属 `08-22-design-system`）。

## Notes

- `ClaudeAuthView.vue`（1,179 行）含 OAuth 流程与 WebView 交互，迁移后需重点验证。
- `src/components/claude-observer/TokenDetailTab.vue` 与主题 token 存在耦合，迁移时同步核对。
