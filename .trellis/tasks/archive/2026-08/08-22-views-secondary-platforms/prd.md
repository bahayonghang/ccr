# Grok / Gemini / OpenCode / generic 视图迁移

> 父任务：`08-22-react-migration`

## Goal

将 Grok、Gemini CLI、OpenCode 三个平台的视图与 `views/generic` 通用视图从 Vue 迁移到 React，约 11,669 行。

## Scope

> **范围回填（`08-22-platform-unify` 普查后，2026-08-24）**：Grok Settings/Profiles/Auth、OpenCode Settings/MCP/Agents/Commands/Plugins、generic `AgentsView` / `PlatformMcpView` / `PlatformPluginsView` 已由统一层提供 config + React 薄壳。本任务改为删除对应 `.vue` 并接路由。`generic/AgentDetailView.vue`（481）与 `generic/SystemPromptsView.vue`（655）留在本任务（协同点 G）。`OpenCodeProvidersView.vue` 为单一实现，不进统一层。

| 文件 / 目录 | 行数 | 处置 |
|---|---|---|
| `src/views/grok/GrokView.vue` | 959 | 本任务 |
| `src/components/grok/`（2 文件） | 1,307 | 本任务（Profile 编辑器） |
| `src/views/generic/AgentDetailView.vue` | 481 | 本任务 |
| `src/views/generic/SystemPromptsView.vue` | 655 | 本任务 |
| `src/views/GeminiCliView.vue` | 929 | 本任务 |
| `src/views/GeminiSlashCommandsView.vue` | 27 | 已是薄壳，改接到 React |
| `src/views/OpenCodeView.vue` | 783 | 本任务 |
| `src/views/OpenCodeProvidersView.vue` | 577 | 本任务（单一实现） |
| `src/components/opencode/`（1 文件） | 121 | 本任务 |
| Grok Settings/Profiles/Auth + OpenCode 五面 + generic 三面 | 5,830 | 统一层已提供薄壳，本任务只删 Vue + 接路由 |
| 合计（仍迁实现） | 5,839 | |

覆盖的功能面：Grok Profiles 与 Settings、Gemini CLI 的 Settings / MCP / Agents / 斜杠命令 / 插件、OpenCode 的 Settings / Keybindings / Themes / Providers / MCP / Agents / Commands / Plugins、`views/generic` 的跨平台通用视图。

## Requirements

- R1 上表全部文件迁移为 React 组件，对应 `.vue` 文件删除。
- R2 本批次内的 `v-model` 展开为受控属性与回调对，slot 转为 children 或 render props。
- R3 消费 `08-22-design-system` 产出的原语与 token，本批次不新增硬编码样式值。
- R4 IPC 调用点沿用 `src/api` 现有 wrapper，不新增或修改 wrapper。
- R5 `grok-settings-contracts.md` 定义的行为在迁移后成立。
- R6 OpenCode Themes 功能与 CCR 自身主题体系相互独立，迁移后不得混淆两者的 token 命名空间。`OpenCodeThemeRecord` 类型（`src/types/generated/opencode/`）为 ts-rs 产物，原样复用。
- R7 `views/generic` 的 5 个通用视图在迁移后仍可被多平台复用，接口不收窄。
- R8 落在本批次的 `nextTick` 调用逐点登记与改写。

## Acceptance Criteria

- [x] AC1 上表 20 个文件全部迁移，`rg --files -g '*.vue' src/views/grok src/views/generic src/views/Gemini* src/views/OpenCode* src/components/grok src/components/opencode` 无匹配。
- [x] AC2 全部视图的路由可达，页面渲染无报错。
- [x] AC3 每个平台的核心操作路径手动验证通过并记录：Grok Profiles 切换与 Settings 读写、Gemini Settings / MCP / Agents / 斜杠命令 / 插件、OpenCode Settings / Keybindings / Themes / Providers / MCP / Agents / Commands / Plugins。
- [x] AC4 本批次组件内 px 字面量与 `rgba()` 数量为 0（登记豁免除外）。
- [x] AC5 `grok-settings-contracts.md` 定义的行为验证通过。
- [x] AC6 OpenCode Themes 的 token 命名空间与 CCR 主题 token 无交叉，由 smoke 测试断言。
- [x] AC7 `views/generic` 的复用点清单落盘，迁移后各消费方均正常工作。
- [x] AC8 `src/api` 的 git diff 为空。
- [x] AC9 `bun run type-check` 与 `bun run lint` 退出码 0。
- [x] AC10 本批次相关的 smoke 测试通过。

## 前置与后续

- 前置：`08-22-shell-port`。
- 可与其余六个视图子任务并行。
- i18n 调用点在本批次内同步转换，运行时切换与收尾校验属 `08-22-i18n-port`。

## Out of Scope

- 新增功能与信息架构调整。
- `src/api` 与 `src/types` 的修改。
- `src-tauri` 侧改动。
- Droid 平台。`ccr-ui/CLAUDE.md` 列出 Droid 支持，但 `src/views` 下无 Droid 专属视图，其功能面由 `views/generic` 承载。若迁移中发现独立 Droid 视图，追加到本任务范围并更新本表。

## Notes

- `src/utils/grokProfileEditor.ts`、`grokProfiles.ts`、`grokSettings.ts`、`opencode.ts` 为纯逻辑，由 `08-22-react-foundation` 判定为原样复用，本任务只改调用点。
- `views/generic` 的 5 个视图是跨平台复用点，其接口变更会波及 Claude / Codex / Gemini / OpenCode / Droid 五个平台的视图，接口需在本任务早期定稳并通知并行子任务。
