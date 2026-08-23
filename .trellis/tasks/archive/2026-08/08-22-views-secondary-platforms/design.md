# 技术设计：Grok / Gemini / OpenCode / generic 视图迁移

> 父任务：`08-22-react-migration`。本域覆盖三个平台与 `views/generic` 通用层，是跨平台复用点最集中的一批。

## 1. 本批次共性转换

与 `08-22-views-claude` 的 `design.md` §1 同表，不重复。前置阅读同。

## 2. 本域范围与统一层切分

范围变更（PRD Scope）：

**平台重复实现 4,331 行移交统一层**：`grok/GrokSettingsView`(1,245)、`grok/GrokProfilesView`(1,078)、`OpenCodeAgentsView`(442)、`OpenCodeMcpView`(433)、`OpenCodeCommandsView`(346)、`OpenCodeSettingsView`(330)、`OpenCodePluginsView`(296)、`grok/GrokAuthView`(161)。本任务改为提供三个平台的 config 与薄壳。

**generic 层 1,499 行并入统一层**：`generic/AgentsView`(725)、`generic/PlatformMcpView`(407)、`generic/PlatformPluginsView`(367)。

**留在本任务的 generic**：`generic/AgentDetailView`(481)、`generic/SystemPromptsView`(655)。

本任务剩余约 5,839 行。精确切分在 `platform-unify` 批次 8 回填。

`views/generic` 的归属划分是协同点 G，需与 `08-22-platform-unify` 批次 5 显式对齐——三个文件给对方、两个文件留本任务，边界不可含混。

## 3. `views/generic` 留守两个文件的接口约束

`AgentDetailView`(481) 与 `SystemPromptsView`(655) 是跨平台复用点，其接口变更会波及 Claude / Codex / Gemini / OpenCode / Droid 五个平台的视图（PRD Notes）。

R7：迁移后仍可被多平台复用，接口不收窄。

「接口不收窄」的可操作含义：

- props 的可选性不变（原为可选的不变必填）。
- 平台无关的行为不加平台条件。
- 消费点清单落盘（AC7），迁移后逐个确认正常工作。

接口需在本任务早期定稳并通知并行子任务（PRD Notes）。定稳的公示内容与 `08-22-shell-port` 的 `shared-interfaces.md` 同格式：props 完整列表与类型、slot → children / render props 映射、状态责任划分。

## 4. OpenCode Themes 的 token 命名空间隔离（本域特有风险）

R6：OpenCode Themes 功能与 CCR 自身主题体系相互独立，迁移后不得混淆两者的 token 命名空间。

风险来源：`08-22-design-system` 把 CCR 的 448 个 token 迁到 Tailwind `@theme`，工具类前缀（`--color-*` 等）进入全局。OpenCode 的主题数据是被管理的外部对象，其颜色值不是 CCR 的 token。

隔离方式：

- OpenCode 主题数据的渲染用内联样式或 CSS 变量，变量名带独立前缀（如 `--oc-*`），不进 `@theme`。
- `OpenCodeThemeRecord` 类型（`src/types/generated/opencode/`）为 ts-rs 产物，原样复用，不改。
- AC6 由 smoke 测试断言：CCR 的 token 名集合与 OpenCode 主题渲染用的变量名集合无交集。

## 5. Grok 的契约

`grok-settings-contracts.md`（5.5 KB）定义的行为在迁移后成立（R5、AC5）。

`GrokSettingsView`(1,245) 移交统一层后，该契约的断言分两部分：共性部分在 `BaseSettings` 中成立，Grok 独有部分在 `grokSettingsConfig` 中成立。契约重写时需按此分割，重写稿由 `08-22-test-contract-rebuild` 提供（协同点 D）。

契约验证的责任：本任务负责 Grok 侧的验证，`08-22-platform-unify` 负责 base 侧。两者的验证点不重叠，合起来覆盖契约全部断言。

## 6. 其余视图的要点

| 视图                      | 行数  | 要点                                                                                      |
| ------------------------- | ----- | ----------------------------------------------------------------------------------------- |
| `GeminiCliView`           | 929   | Gemini 平台首页，聚合 Settings / MCP / Agents / 斜杠命令 / 插件入口                       |
| `GeminiSlashCommandsView` | 27    | 已是薄壳，传 `geminiConfig` + `hide-chrome` prop。`hide-chrome` 的 props 形态在统一层保留 |
| `OpenCodeView`            | 783   | OpenCode 平台首页                                                                         |
| `OpenCodeProvidersView`   | 577   | 单一实现，无重复，不进统一层（`platform-unify` 的「不在统一范围」）                       |
| `components/grok/`        | 1,307 | 2 文件，域组件                                                                            |
| `components/opencode/`    | 121   | 1 文件                                                                                    |

## 7. 框架无关资产

`src/utils/grokProfileEditor.ts`、`grokProfiles.ts`、`grokSettings.ts`、`opencode.ts` 由 `08-22-react-foundation` 判定为原样复用，本任务只改调用点（PRD Notes）。需修改则登记为独立缺陷。

## 8. Droid 平台

`ccr-ui/CLAUDE.md` 列出 Droid 支持（Settings / MCP / Agents / Plugins / Models / Profiles），但 `src/views` 下无 Droid 专属视图，其功能面由 `views/generic` 承载（PRD Out of Scope）。

因此第 3 节的「接口不收窄」约束对 Droid 同样适用——Droid 没有专属视图，完全依赖 generic 层的通用性。若迁移中发现独立 Droid 视图，追加到本任务范围并更新范围表。

## 9. 不变量

- IPC 调用点沿用现有 wrapper（R4）。`git diff --stat src/api` 须为空（AC8）。
- `src/types` 与 `src-tauri` 不改。
- `OpenCodeThemeRecord` 等 ts-rs 产物不改。

## 10. 未决项

- Auth 面判定结果决定 `grok/GrokAuthView`(161) 的归属。
- `views/generic` 五个文件的归属边界待与 `platform-unify` 批次 5 对齐（协同点 G）。
- Droid 是否存在独立视图（第 8 节末段）。
- 本任务的精确文件清单待 `platform-unify` 批次 8 回填。
