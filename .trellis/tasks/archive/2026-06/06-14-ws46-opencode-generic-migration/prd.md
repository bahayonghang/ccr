# WS4.6 OpenCode 双页 generic 迁移 — 决断：放弃迁移（WONTFIX）

## 结论

**不把 `OpenCodeMcpView` / `OpenCodeCommandsView` 迁入现有 generic 基建。** 维持两页独立实现。

## 调研结论（2026-06-14，逐文件实测）

PRD WS4.6 的前提是「generic 基建就绪，只需接 descriptor」。实测该前提不成立，方向恰好相反：

### generic 基建本身是债务

`views/generic/PlatformMcpView.vue` + `composables/usePlatformMcp.ts` + `config/platformDescriptors.ts`：

- **旧玻璃设计语言**：`glass-effect`、`linear-gradient(135deg,…)` 按钮、`text-white`、
  `onCardHover` 内裸 `rgba(255,255,255,.9)` / `rgba(99,102,241,…)`、`box-shadow: 0 0 20px var(--glow-primary)`。
  这正是 WS6 正在全局删除的语言——generic 视图自身尚未迁移。
- **gemini 专属数据模型**：`PlatformMcpServer` 为 gemini 形（`command: string`、`args`、`env`、`trust`、`includeTools`）；
  descriptor 类型签名硬编码 `id: 'gemini'`、`rootPath: 'antigravity'`。

### OpenCode 两页已是目标态

`OpenCodeMcpView`(424) / `OpenCodeCommandsView`(340)：

- **已在新设计系统**：`OpenCodePageShell` + `Card`/`Button`/`BaseModal` + 语义 token + `copyText`/`getErrorMessage` 工具。
- **承载 OpenCode 专属能力**，generic/BaseSlashCommands 无法表达：
  - MCP：`local/remote` 类型、`enabled` 开关、CLI handoff（`opencode mcp auth/debug/logout`）、env/headers JSON 编辑器、command 以数组存储（`splitCommandInput`）。
  - Commands：`agent`/`model`/`subtask`/`template`/`scope` frontmatter 模型、built-in 命令覆盖语义说明面板。
- `BaseSlashCommands` 围绕 claude/gemini/codex 的斜杠命令语义构建，与 OpenCode frontmatter 模型不同构。

## 为什么放弃

强行迁移会用**两个干净、现代、紧凑（各 ~400 行）、功能完整**的平台专属视图，
换取一个**旧玻璃语言、gemini 形、功能更弱**的壳层 —— 视觉与功能双回归，违背项目自身设计方向。
WS4.6 想消除的重复，主要存在于「gemini-generic ↔ 其他平台」之间；OpenCode 两页本就小而清晰，
强迁入的成本/收益为负。

## 后续（如未来仍想收敛 4 套 MCP → 1 套）

正确顺序是**反向重建**：先把 generic 基建用新设计系统重写（或直接以 OpenCode 的 PageShell 模式为模板，
descriptor 泛化掉 `id:'gemini'`/`rootPath` 硬编码、数据模型抽象出 `enabled`/`type`/CLI handoff 等差异点），
再把 gemini 迁进去。这是另立的较大重构，不在本任务范围。

## 验收处置

- AC#8 中「OpenCodeMcpView + OpenCodeCommands 完成 generic 迁移」一项标注为 **不适用 / 主动放弃**，附本决断。
- CodexMcp/CodexAgents 的 generic 迁移同理顺延（同样会撞 generic 基建债务）。
