# 执行计划：Grok / Gemini / OpenCode / generic 视图迁移

> 父任务：`08-22-react-migration`（阶段 5，七个视图子任务并行）。
> 分支：`feature/react-migration/views-secondary-platforms`，PR 目标 `feature/react-migration`。

## 外壳接口公示（08-22-shell-port 批次 6）

共享接口：`.trellis/tasks/08-22-shell-port/shared-interfaces.md`。阶段 4a 之后不改 `MasterDetailLayout` 与 `src/ui/` 原语。

## 前置确认

- [ ] 父任务统一层门已通过，本任务范围表已回填。
- [ ] `views/generic` 五个文件的归属边界已与 `08-22-platform-unify` 批次 5 对齐（协同点 G）：三个给对方，`AgentDetailView`(481) 与 `SystemPromptsView`(655) 留本任务。
- [ ] 前置阅读完成（`08-22-views-claude/design.md` §1 末段的五份文档）。
- [ ] 前置阅读：`.trellis/spec/ccr-ui/frontend/react-rerender-discipline.md`（R8，动手前必读）。
- [ ] `08-22-test-contract-rebuild` 已提供 `grok-settings-contracts.md` 的重写稿，且已按 `design.md` §5 分割为 base 侧与 Grok 侧两部分。
- [ ] Auth 面判定结果已知，`grok/GrokAuthView` 的归属确定。
- [ ] `git checkout -b feature/react-migration/views-secondary-platforms feature/react-migration`

## 提交批次

### 批次 1：generic 留守两文件的接口定稳

先做。该接口是五个平台的共同依赖，定稳后才能安全并行。

- [ ] `AgentDetailView`(481) 与 `SystemPromptsView`(655) 迁移。
- [ ] 按 `design.md` §3 公示接口：props 完整列表与类型、slot → children / render props 映射、状态责任划分。
- [ ] 接口不收窄的自检：props 可选性不变，无新增平台条件。
- [ ] 消费点清单落盘（AC7），含 Claude / Codex / Gemini / OpenCode / Droid 五侧。
- [ ] 通知并行子任务。

验证：五侧消费点迁移后逐个确认正常工作（部分消费点在其他子任务，需其交付后回验）。

### 批次 2：统一层接入（config + 薄壳）

- [ ] 填 `grokSettingsConfig`、`grokProfilesConfig`。
- [ ] 填 OpenCode 的 settings / agents / mcp / commands / plugins 五个 config 导出。
- [ ] 按 Auth 判定填 `configs/auth.ts` 或保留 `GrokAuthView`。
- [ ] `OpenCodeAgentsView` / `OpenCodeMcpView` / `OpenCodePluginsView` 收敛到 generic 三视图的调用点。
- [ ] 薄壳视图各 ≤100 行。
- [ ] `GeminiSlashCommandsView`(27) 迁移，`hide-chrome` props 形态保留。

验证：各面路由可达；归属 `config.grok` / `config.opencode` 的差异项逐项界面确认。

### 批次 3：OpenCode Themes 隔离

本域特有风险，单独一批。

- [ ] `OpenCodeThemeRecord` 类型原样复用，不改（R6）。
- [ ] OpenCode 主题数据渲染用独立前缀变量（如 `--oc-*`），不进 `@theme`。
- [ ] smoke 测试断言：CCR token 名集合与 OpenCode 主题变量名集合无交集（AC6）。

### 批次 4：三个平台首页与域组件

- [ ] `GeminiCliView`(929)、`OpenCodeView`(783)、`OpenCodeProvidersView`(577)。
- [ ] `components/grok/`（2 文件 1,307 行）、`components/opencode/`（1 文件 121 行）。
- [ ] 依赖方向自检：不导入其他 `features/<平台>/`。
- [ ] 超过行数上限的文件拆分。

### 批次 5：契约验证与收口

- [ ] `grok-settings-contracts.md` 的 Grok 侧断言逐条验证（AC5）。base 侧由 `08-22-platform-unify` 验证，两侧合起来覆盖全部断言。
- [ ] 本批次组件内 px 与 `rgba()` 归零，豁免逐条登记（AC4）。
- [ ] `nextTick` 登记表落盘（R8）。
- [ ] `git diff --stat src/api src/types`（应为空，AC8）。
- [ ] `src/utils/grokProfileEditor.ts`、`grokProfiles.ts`、`grokSettings.ts`、`opencode.ts` 的 git diff 为空。
- [ ] 确认是否存在独立 Droid 视图。存在则追加范围并更新范围表。

## 验证命令

| 时机           | 命令                                                                                                                                              |
| -------------- | ------------------------------------------------------------------------------------------------------------------------------------------------- |
| 每批次后       | `bun run type-check`、`bun run lint`（AC9）                                                                                                       |
| 批次 1、3–4 后 | `bun run test:smoke`（AC10）                                                                                                                      |
| 批次 5 后      | `rg --files -g '*.vue' src/views/grok src/views/generic src/views/Gemini* src/views/OpenCode* src/components/grok src/components/opencode`（AC1） |
| 交付前         | `just frontend-check-quick`、`bun run lint:ci`                                                                                                    |

## 交付门（父任务视图门的一部分）

- [ ] AC1–AC10 全部满足。
- [ ] AC3 的核心操作路径逐条验证并记录：Grok Profiles 切换与 Settings 读写；Gemini Settings / MCP / Agents / 斜杠命令 / 插件；OpenCode Settings / Keybindings / Themes / Providers / MCP / Agents / Commands / Plugins。
- [ ] `views/generic` 消费点清单落盘，五侧逐个确认（AC7）。留守两文件的接口已公示。
- [ ] OpenCode Themes 的 token 命名空间无交叉，smoke 测试断言（AC6）。
- [ ] `grok-settings-contracts.md` 的 Grok 侧断言通过（AC5）。
- [ ] `nextTick` 登记表与硬编码豁免登记落盘。
- [ ] 本域验证矩阵格已填，交 `08-22-platform-unify` AC6。

## 回滚点

五个批次各自独立提交。批次 1 的接口定稳后不回滚——五个平台依赖它。批次 2–5 各自独立。

提交粒度：单文件或单个紧密相关的小组。

## 协同点

| 编号 | 内容                                    | 对方                          | 时机   |
| ---- | --------------------------------------- | ----------------------------- | ------ |
| G    | `views/generic` 归属划分                | `08-22-platform-unify`        | 前置   |
| E    | 统一层接口消费                          | `08-22-platform-unify`        | 批次 2 |
| D    | `grok-settings-contracts.md` 重写稿分割 | `08-22-test-contract-rebuild` | 前置   |
| I    | i18n 调用形式                           | `08-22-i18n-port`             | 全程   |
| —    | `views/generic` 留守两文件的接口公示    | 四个平台视图子任务            | 批次 1 |
| —    | OpenCode 主题变量前缀不进 `@theme`      | `08-22-design-system`         | 批次 3 |
| —    | 本域验证矩阵格                          | `08-22-platform-unify`        | 交付时 |
