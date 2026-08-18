# 51 视图勾销表

盘点日期：2026-08-18。来源：`ccr-ui/src/views/**/*View.vue`（排除 `TrayOverview.vue` 文件名误匹配）。

原语列：`H` = PageHeader，`S` = StatTile，`P` = PillToggleGroup，`Sh` = PageShell。`—` = 本页不适用，禁止为凑数而插入。

路由 redirect（`ccr-control`、`sessions`、`market`、`skills-manager`、`opencode-skills` 等）不在本表。

## Wave 2 — 设置（1）

| # | 组件 | H | S | P | Sh | 完成 |
|---|---|---|---|---|---|---|
| 1 | `AppSettingsView` | H | — | — | — | [x] |

## Wave 3 — Dashboard（1）

| # | 组件 | H | S | P | Sh | 完成 |
|---|---|---|---|---|---|---|
| 2 | `DashboardView` | H | S | P | — | [x] |

## Wave 4 — 平台族（29）

### 主页（5）

| # | 组件 | H | S | P | Sh | 完成 |
|---|---|---|---|---|---|---|
| 3 | `ClaudeCodeView` | H | S | — | Sh | [x] |
| 4 | `CodexView` | H | S | — | Sh | [x] |
| 5 | `grok/GrokView` | H | S | — | Sh | [x] |
| 6 | `GeminiCliView` | H | S | — | Sh | [x] |
| 7 | `OpenCodeView` | H | S | — | Sh | [x] |

### Claude 子页（3）

| # | 组件 | H | S | P | Sh | 完成 |
|---|---|---|---|---|---|---|
| 8 | `ClaudeCodeSettingsView` | H | — | — | Sh | [x] |
| 9 | `ClaudeCodeProfilesView` | H | S | — | Sh | [x] |
| 10 | `ClaudeAuthView` | H | — | — | Sh | [x] |

### Codex 子页（7）

| # | 组件 | H | S | P | Sh | 完成 |
|---|---|---|---|---|---|---|
| 11 | `CodexMcpView` | H | — | — | Sh | [x] |
| 12 | `CodexProfilesView` | H | S | — | Sh | [x] |
| 13 | `codex/CodexAgentsView` | H | — | — | Sh | [x] |
| 14 | `CodexSessionsView` | H | — | P | Sh | [x] |
| 15 | `CodexSlashCommandsView` | H | — | — | Sh | [x] |
| 16 | `CodexAuthView` | H | — | — | Sh | [x] |
| 17 | `CodexSettingsView` | H | — | — | Sh | [x] |

### Grok 子页（2）

| # | 组件 | H | S | P | Sh | 完成 |
|---|---|---|---|---|---|---|
| 18 | `grok/GrokProfilesView` | H | S | — | Sh | [x] |
| 19 | `grok/GrokSettingsView` | H | — | — | Sh | [x] |

### Antigravity / Gemini 子页（3）

| # | 组件 | H | S | P | Sh | 完成 |
|---|---|---|---|---|---|---|
| 20 | `GeminiSlashCommandsView` | H | — | — | Sh | [x] |
| 21 | `generic/PlatformMcpView` | H | — | — | Sh | [x] |
| 22 | `generic/PlatformPluginsView` | H | — | — | Sh | [x] |

### OpenCode 子页（6）

| # | 组件 | H | S | P | Sh | 完成 |
|---|---|---|---|---|---|---|
| 23 | `OpenCodeProvidersView` | H | — | — | Sh | [x] |
| 24 | `OpenCodeMcpView` | H | — | — | Sh | [x] |
| 25 | `OpenCodeAgentsView` | H | — | — | Sh | [x] |
| 26 | `OpenCodeCommandsView` | H | — | — | Sh | [x] |
| 27 | `OpenCodePluginsView` | H | — | — | Sh | [x] |
| 28 | `OpenCodeSettingsView` | H | — | — | Sh | [x] |

### 跨平台复用（3）

| # | 组件 | H | S | P | Sh | 完成 |
|---|---|---|---|---|---|---|
| 29 | `generic/SystemPromptsView` | H | — | — | Sh | [x] |
| 30 | `generic/AgentsView` | H | — | — | Sh | [x] |
| 31 | `generic/AgentDetailView` | H | — | — | Sh | [x] |

## Wave 5 — 功能族（20）

| # | 组件 | H | S | P | Sh | 完成 |
|---|---|---|---|---|---|---|
| 32 | `CommandsView` | H | — | P | — | [x] |
| 33 | `ConverterView` | H | — | — | — | [x] |
| 34 | `SyncView` | H | — | — | — | [x] |
| 35 | `ConfigsView` | H | — | — | — | [x] |
| 36 | `BudgetView` | H | S | — | — | [x] |
| 37 | `PricingView` | H | — | — | — | [x] |
| 38 | `UsageDashboardView` | H | S | P | — | [x] |
| 39 | `MonitoringView` | H | — | P | — | [x] |
| 40 | `mcp/McpManagerView` | H | — | P | — | [x] |
| 41 | `SlashCommandsView` | H | — | — | — | [x] |
| 42 | `SkillsMigrationView` | H | — | — | — | [x] |
| 43 | `PluginsView` | H | — | — | — | [x] |
| 44 | `HooksView` | H | — | — | — | [x] |
| 45 | `OutputStylesView` | H | — | — | — | [x] |
| 46 | `StatuslineView` | H | — | — | — | [x] |
| 47 | `CheckinView` | H | S | P | — | [x] |
| 48 | `checkin/CheckinAccountDashboardView` | H | S | P | — | [x] |
| 49 | `WslManagementView` | H | — | — | — | [x] |
| 50 | `SshManagementView` | H | — | — | — | [x] |
| 51 | `tray/CodexTrayPanelView` | H | S | — | — | [x] |

合计：1 + 1 + 29 + 20 = 51。

## 非视图但必须清扫的表面

这些不是 `*View.vue`，不计入 51，但 R7 / Wave 6 必须处理：

- `CodexAgentEditorModal`、`ConfirmModal`、`BulkDeleteDialog`、`AccountFormModal`、`OAuthWizardModal`
- `ConfigCard`、`CommandList`、`BaseSlashCommands`、`AgentIcons`
- `McpDetailPanel`、`McpListPanel`、`McpCreatePanel`
- `components/dashboard/*`、`components/usage/*`、`tray/components/TrayOverview`
- `OpenCodePageShell`（Wave 1 先改，再抽 PageShell）
