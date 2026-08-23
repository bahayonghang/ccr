# views/generic 消费点清单（AC7）

## SystemPromptsView

公示接口：`SystemPromptsView({ config, t? })`，`config` 为 `systemPromptsConfigs[platform]`。

| 消费方 | 接线 |
| --- | --- |
| Claude | 既有 `features/claude/ClaudeSystemPromptsView.tsx`（本任务不改） |
| Codex | 既有 `features/codex/CodexSystemPromptsView.tsx`（本任务不改） |
| Gemini | `features/gemini/GeminiSystemPromptsView.tsx` → `systemPromptsConfigs.gemini` |
| OpenCode | `features/opencode/OpenCodeSystemPromptsView.tsx` → `systemPromptsConfigs.opencode` |
| Droid | 无专属视图；可复用同一 `SystemPromptsView` + 新 config，props 仍可选 |

平台差异走 `config.features`（`hierarchyNote` / `geminiNote` / `showRules` / `limitHint`），不在 Base 里比较平台名。

## AgentDetailView

公示接口：无平台 props。路由 `/agents/:name`，API 仍为 Claude `getAgent` / `updateAgent` / `deleteAgent` / `toggleAgent`。

| 消费方 | 接线 |
| --- | --- |
| Claude `/agents/:name` | `geminiRouteLoaders['agent-detail']` → `AgentDetailView` |
| 其余平台 | 不使用该详情路由；各自 Agents 面走 `BaseAgents` |

## Agents 列表 `/agents`

`AgentsHomeView` = `BaseAgents` + `claudeAgentsConfig`。loader id `agents`。
