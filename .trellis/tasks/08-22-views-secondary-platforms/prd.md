# Grok / Gemini / OpenCode / generic 视图迁移

> 父任务：`08-22-react-migration`

## Goal

将 Grok、Gemini CLI、OpenCode 三个平台的视图与 `views/generic` 通用视图从 Vue 迁移到 React，约 11,669 行。

## Scope

> **范围变更（跨平台统一决策后）**：以下文件移交子任务 `08-22-platform-unify` 的统一层，本任务改为提供 Grok / Gemini / OpenCode 三个平台的 config 与薄壳视图。
>
> 平台重复实现 4,331 行：`grok/GrokSettingsView.vue`（1,245）、`grok/GrokProfilesView.vue`（1,078）、`OpenCodeAgentsView.vue`（442）、`OpenCodeMcpView.vue`（433）、`OpenCodeCommandsView.vue`（346）、`OpenCodeSettingsView.vue`（330）、`OpenCodePluginsView.vue`（296）、`grok/GrokAuthView.vue`（161）。
>
> generic 层 1,499 行并入统一层：`generic/AgentsView.vue`（725）、`generic/PlatformMcpView.vue`（407）、`generic/PlatformPluginsView.vue`（367）。`generic/AgentDetailView.vue`（481）与 `generic/SystemPromptsView.vue`（655）留在本任务。
>
> 本任务剩余约 5,839 行。精确切分由 `08-22-platform-unify` 的差异普查（R1）确定后回填本表。

| 文件 / 目录 | 行数 |
|---|---|
| `src/views/grok/`（4 文件，含 `GrokSettingsView.vue` 1,245、`GrokProfilesView.vue` 1,078） | 3,443 |
| `src/components/grok/`（2 文件） | 1,307 |
| `src/views/generic/`（5 文件） | 2,635 |
| `src/views/GeminiCliView.vue` | 929 |
| `src/views/GeminiSlashCommandsView.vue` | 27 |
| `src/views/OpenCodeView.vue` | 783 |
| `src/views/OpenCodeProvidersView.vue` | 577 |
| `src/views/OpenCodeAgentsView.vue` | 442 |
| `src/views/OpenCodeMcpView.vue` | 433 |
| `src/views/OpenCodeCommandsView.vue` | 346 |
| `src/views/OpenCodeSettingsView.vue` | 330 |
| `src/views/OpenCodePluginsView.vue` | 296 |
| `src/components/opencode/`（1 文件） | 121 |
| 合计 | 11,669 |

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

- [ ] AC1 上表 20 个文件全部迁移，`rg --files -g '*.vue' src/views/grok src/views/generic src/views/Gemini* src/views/OpenCode* src/components/grok src/components/opencode` 无匹配。
- [ ] AC2 全部视图的路由可达，页面渲染无报错。
- [ ] AC3 每个平台的核心操作路径手动验证通过并记录：Grok Profiles 切换与 Settings 读写、Gemini Settings / MCP / Agents / 斜杠命令 / 插件、OpenCode Settings / Keybindings / Themes / Providers / MCP / Agents / Commands / Plugins。
- [ ] AC4 本批次组件内 px 字面量与 `rgba()` 数量为 0（登记豁免除外）。
- [ ] AC5 `grok-settings-contracts.md` 定义的行为验证通过。
- [ ] AC6 OpenCode Themes 的 token 命名空间与 CCR 主题 token 无交叉，由 smoke 测试断言。
- [ ] AC7 `views/generic` 的复用点清单落盘，迁移后各消费方均正常工作。
- [ ] AC8 `src/api` 的 git diff 为空。
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
- Droid 平台。`ccr-ui/CLAUDE.md` 列出 Droid 支持，但 `src/views` 下无 Droid 专属视图，其功能面由 `views/generic` 承载。若迁移中发现独立 Droid 视图，追加到本任务范围并更新本表。

## Notes

- `src/utils/grokProfileEditor.ts`、`grokProfiles.ts`、`grokSettings.ts`、`opencode.ts` 为纯逻辑，由 `08-22-react-foundation` 判定为原样复用，本任务只改调用点。
- `views/generic` 的 5 个视图是跨平台复用点，其接口变更会波及 Claude / Codex / Gemini / OpenCode / Droid 五个平台的视图，接口需在本任务早期定稳并通知并行子任务。
