# 路径映射表（AC9）

> 任务：`08-22-react-foundation` 批次 6。依据：本任务 `design.md` §8、父任务 `design.md` §2（features/ 按域聚合）、父任务 `prd.md` 任务地图、`08-22-platform-unify` `implement.md` 批次 1。
>
> - 生成日期：2026-08-23
> - 方法：脚本枚举 `ccr-ui/src` 下全部 `.vue` 文件与 `ccr-ui/src/utils/` 全部文件后按规则分类落盘，非人工逐行抄录。
> - 规模：185 行 `.vue` + 31 行 `utils` = **216 行**，无空缺、无重复；旧路径集合与实际文件集合经脚本比对完全一致。
> - 规则：文件基名不变，仅扩展名 `.vue` → `.tsx` 与目录变化；目录按父任务 `design.md` §2 的 `features/<domain>/` 聚合。
> - **表随实现更新**：本表按当前测量填写。若后续子任务实际落位与表不一致，改表不改文件。

## 与源文档的偏差登记

`design.md` §8 与本任务 `implement.md` 批次 6 写「移交 `08-22-platform-unify` 的 20 个文件」，但三处权威枚举——platform-unify `prd.md` Scope 表、其 `design.md` §4 收敛方案表、其 `implement.md` 批次 1 普查清单——列出的均为 **18 个文件**，且行数分项和恰为文档声称的总量 15,672（3,923 + 3,325 + 2,298 + 2,090 + 1,734 + 1,580 + 722 = 15,672）。「20」为文档算术误差。本表以 platform-unify 权威清单为准，标注收敛方式的行数为 **18**（另有 3 个 `views/generic/` base 本体视图归 platform-unify 所有，见对应小节）。该偏差已同步至本任务 `implement.md` 批次 6。

## 目录判定补充说明（表内未展开的判断）

- `src/App.vue` 不迁移，由 `src/shell/App.tsx` 替代删除。
- `components/common/` 与根级跨域共享件落 `src/ui/`；其中全局外壳件（Toast、背景、环境指示、导航、版本/更新）落 `src/shell/`。
- `views/generic/AgentDetailView`、`SystemPromptsView` 按 platform-unify 协同点 G 由 `08-22-views-secondary-platforms` 保留，落 `features/platform/`（跨平台泛化视图无更贴切的既有域）。
- tray 三视图为 Codex 托管窗口，落 `features/codex/tray/`；editor 两件为配置源编辑器，落 `features/configs/`；Monitoring / SSH / WSL 为运行环境工具，落 `features/sync/`。三者迁移归属均为 `08-22-views-sync-tools`（父任务任务地图第 12 行明列 editor / Monitoring / SSH / WSL / tray）。
- utils 判定以 `utils-disposition.md` 实测为准（较本任务 `design.md` §7 的 prd 预期清单有偏差登记）：12 个需接线文件移出 `src/utils/`（11 个入 `src/shell/utils/` 归 `08-22-shell-port`；`apexChartsCore.ts` 入 `src/features/usage/` 归 `08-22-views-usage`），其余 19 个原样复用（同路径，归 `08-22-react-foundation` 复用验证）。


### 外壳（App / layout / 窗口与主题引导相关根组件）→ `src/shell/`（14 行）

| 旧路径 | 新路径 | 归属子任务 |
| --- | --- | --- |
| `src/App.vue` | `删除（由 src/shell/App.tsx 替代）` | `08-22-shell-port` |
| `src/components/MainLayout.vue` | `src/shell/MainLayout.tsx` | `08-22-shell-port` |
| `src/components/layout/Titlebar.vue` | `src/shell/Titlebar.tsx` | `08-22-shell-port` |
| `src/components/common/AnimatedBackground.vue` | `src/shell/AnimatedBackground.tsx` | `08-22-shell-port` |
| `src/components/common/StageBackground.vue` | `src/shell/StageBackground.tsx` | `08-22-shell-port` |
| `src/components/common/ToastContainer.vue` | `src/shell/ToastContainer.tsx` | `08-22-shell-port` |
| `src/components/common/GlobalConfirmDialog.vue` | `src/shell/GlobalConfirmDialog.tsx` | `08-22-shell-port` |
| `src/components/BackendStatusBanner.vue` | `src/shell/BackendStatusBanner.tsx` | `08-22-shell-port` |
| `src/components/EnvironmentBadge.vue` | `src/shell/EnvironmentBadge.tsx` | `08-22-shell-port` |
| `src/components/EnvironmentSwitcher.vue` | `src/shell/EnvironmentSwitcher.tsx` | `08-22-shell-port` |
| `src/components/ModuleSubnav.vue` | `src/shell/ModuleSubnav.tsx` | `08-22-shell-port` |
| `src/components/ThemeToggle.vue` | `src/shell/ThemeToggle.tsx` | `08-22-shell-port` |
| `src/components/UpdateModal.vue` | `src/shell/UpdateModal.tsx` | `08-22-shell-port` |
| `src/components/VersionManager.vue` | `src/shell/VersionManager.tsx` | `08-22-shell-port` |

### 跨域共享组件（原 components/common/ 与根级共享件）→ `src/ui/`（11 行）

| 旧路径 | 新路径 | 归属子任务 |
| --- | --- | --- |
| `src/components/common/BaseModal.vue` | `src/ui/BaseModal.tsx` | `08-22-shell-port` |
| `src/components/common/BulkDeleteDialog.vue` | `src/ui/BulkDeleteDialog.tsx` | `08-22-shell-port` |
| `src/components/common/ListSearchHeader.vue` | `src/ui/ListSearchHeader.tsx` | `08-22-shell-port` |
| `src/components/common/MasterDetailLayout.vue` | `src/ui/MasterDetailLayout.tsx` | `08-22-shell-port` |
| `src/components/common/MultiSelectFloatingBar.vue` | `src/ui/MultiSelectFloatingBar.tsx` | `08-22-shell-port` |
| `src/components/common/ScrollToTopButton.vue` | `src/ui/ScrollToTopButton.tsx` | `08-22-shell-port` |
| `src/components/common/MarketplacePagination.vue` | `src/ui/MarketplacePagination.tsx` | `08-22-shell-port` |
| `src/components/common/AgentIcons.vue` | `src/ui/AgentIcons.tsx` | `08-22-shell-port` |
| `src/components/HistoryList.vue` | `src/ui/HistoryList.tsx` | `08-22-shell-port` |
| `src/components/ConfirmModal.vue` | `src/ui/ConfirmModal.tsx` | `08-22-shell-port` |
| `src/components/PageHeaderCard.vue` | `src/ui/PageHeaderCard.tsx` | `08-22-shell-port` |

### UI 原语 16 个（原 components/ui/）→ `src/ui/`（16 行）

| 旧路径 | 新路径 | 归属子任务 |
| --- | --- | --- |
| `src/components/ui/AsyncStatePanel.vue` | `src/ui/AsyncStatePanel.tsx` | `08-22-design-system` |
| `src/components/ui/Badge.vue` | `src/ui/Badge.tsx` | `08-22-design-system` |
| `src/components/ui/Breadcrumb.vue` | `src/ui/Breadcrumb.tsx` | `08-22-design-system` |
| `src/components/ui/Button.vue` | `src/ui/Button.tsx` | `08-22-design-system` |
| `src/components/ui/Card.vue` | `src/ui/Card.tsx` | `08-22-design-system` |
| `src/components/ui/EmptyState.vue` | `src/ui/EmptyState.tsx` | `08-22-design-system` |
| `src/components/ui/IconWrapper.vue` | `src/ui/IconWrapper.tsx` | `08-22-design-system` |
| `src/components/ui/Input.vue` | `src/ui/Input.tsx` | `08-22-design-system` |
| `src/components/ui/NavItem.vue` | `src/ui/NavItem.tsx` | `08-22-design-system` |
| `src/components/ui/PageHeader.vue` | `src/ui/PageHeader.tsx` | `08-22-design-system` |
| `src/components/ui/PageShell.vue` | `src/ui/PageShell.tsx` | `08-22-design-system` |
| `src/components/ui/PillToggleGroup.vue` | `src/ui/PillToggleGroup.tsx` | `08-22-design-system` |
| `src/components/ui/SIcon.vue` | `src/ui/SIcon.tsx` | `08-22-design-system` |
| `src/components/ui/Sparkline.vue` | `src/ui/Sparkline.tsx` | `08-22-design-system` |
| `src/components/ui/Spinner.vue` | `src/ui/Spinner.tsx` | `08-22-design-system` |
| `src/components/ui/StatTile.vue` | `src/ui/StatTile.tsx` | `08-22-design-system` |

### 移交 08-22-platform-unify 的收敛文件（18 个，标注收敛方式）（18 行）

| 旧路径 | 新路径 | 归属子任务 |
| --- | --- | --- |
| `src/views/ClaudeCodeSettingsView.vue` | `src/features/platform/settings/ClaudeCodeSettingsView.tsx` | `08-22-platform-unify（收敛为薄壳）` |
| `src/views/CodexSettingsView.vue` | `src/features/platform/settings/CodexSettingsView.tsx` | `08-22-platform-unify（收敛为薄壳）` |
| `src/views/grok/GrokSettingsView.vue` | `src/features/platform/settings/GrokSettingsView.tsx` | `08-22-platform-unify（收敛为薄壳）` |
| `src/views/OpenCodeSettingsView.vue` | `src/features/platform/settings/OpenCodeSettingsView.tsx` | `08-22-platform-unify（收敛为薄壳）` |
| `src/views/ClaudeCodeProfilesView.vue` | `src/features/platform/profiles/ClaudeCodeProfilesView.tsx` | `08-22-platform-unify（收敛为薄壳）` |
| `src/views/CodexProfilesView.vue` | `src/features/platform/profiles/CodexProfilesView.tsx` | `08-22-platform-unify（收敛为薄壳）` |
| `src/views/grok/GrokProfilesView.vue` | `src/features/platform/profiles/GrokProfilesView.tsx` | `08-22-platform-unify（收敛为薄壳）` |
| `src/views/ClaudeAuthView.vue` | `src/features/platform/auth/ClaudeAuthView.tsx` | `08-22-platform-unify（收敛为薄壳）` |
| `src/views/CodexAuthView.vue` | `src/features/platform/auth/CodexAuthView.tsx` | `08-22-platform-unify（收敛为薄壳）` |
| `src/views/grok/GrokAuthView.vue` | `src/features/platform/auth/GrokAuthView.tsx` | `08-22-platform-unify（收敛为薄壳）` |
| `src/views/CommandsView.vue` | `src/features/platform/commands/CommandsView.tsx` | `08-22-platform-unify（收敛为薄壳）` |
| `src/views/OpenCodeCommandsView.vue` | `src/features/platform/commands/OpenCodeCommandsView.tsx` | `08-22-platform-unify（收敛为薄壳）` |
| `src/views/CodexMcpView.vue` | `src/features/platform/mcp/CodexMcpView.tsx` | `08-22-platform-unify（收敛为薄壳）` |
| `src/views/OpenCodeMcpView.vue` | `src/features/platform/mcp/OpenCodeMcpView.tsx` | `08-22-platform-unify（收敛为薄壳）` |
| `src/views/codex/CodexAgentsView.vue` | `src/features/platform/agents/CodexAgentsView.tsx` | `08-22-platform-unify（收敛为薄壳）` |
| `src/views/OpenCodeAgentsView.vue` | `src/features/platform/agents/OpenCodeAgentsView.tsx` | `08-22-platform-unify（收敛为薄壳）` |
| `src/views/PluginsView.vue` | `src/features/platform/plugins/PluginsView.tsx` | `08-22-platform-unify（收敛为薄壳）` |
| `src/views/OpenCodePluginsView.vue` | `src/features/platform/plugins/OpenCodePluginsView.tsx` | `08-22-platform-unify（收敛为薄壳）` |

### 统一层 base 本体（原 views/generic/ 三视图，协同点 G 移交）（3 行）

| 旧路径 | 新路径 | 归属子任务 |
| --- | --- | --- |
| `src/views/generic/AgentsView.vue` | `src/features/platform/agents/AgentsView.tsx` | `08-22-platform-unify（统一层 base 本体）` |
| `src/views/generic/PlatformMcpView.vue` | `src/features/platform/mcp/PlatformMcpView.tsx` | `08-22-platform-unify（统一层 base 本体）` |
| `src/views/generic/PlatformPluginsView.vue` | `src/features/platform/plugins/PlatformPluginsView.tsx` | `08-22-platform-unify（统一层 base 本体）` |

### Claude 域 → `src/features/claude/`（15 行）

| 旧路径 | 新路径 | 归属子任务 |
| --- | --- | --- |
| `src/views/ClaudeCodeView.vue` | `src/features/claude/ClaudeCodeView.tsx` | `08-22-views-claude` |
| `src/views/HooksView.vue` | `src/features/claude/HooksView.tsx` | `08-22-views-claude` |
| `src/views/StatuslineView.vue` | `src/features/claude/StatuslineView.tsx` | `08-22-views-claude` |
| `src/views/OutputStylesView.vue` | `src/features/claude/OutputStylesView.tsx` | `08-22-views-claude` |
| `src/views/SkillsMigrationView.vue` | `src/features/claude/SkillsMigrationView.tsx` | `08-22-views-claude` |
| `src/components/claude/ClaudeProfileEditorModal.vue` | `src/features/claude/ClaudeProfileEditorModal.tsx` | `08-22-views-claude` |
| `src/components/claude/ClaudeProfileEditorSections.vue` | `src/features/claude/ClaudeProfileEditorSections.tsx` | `08-22-views-claude` |
| `src/components/claude/ClaudeProfileRow.vue` | `src/features/claude/ClaudeProfileRow.tsx` | `08-22-views-claude` |
| `src/components/claude-observer/BehaviorAnalysisTab.vue` | `src/features/claude/observer/BehaviorAnalysisTab.tsx` | `08-22-views-claude` |
| `src/components/claude-observer/ChartErrorBoundary.vue` | `src/features/claude/observer/ChartErrorBoundary.tsx` | `08-22-views-claude` |
| `src/components/claude-observer/ChartPreparingState.vue` | `src/features/claude/observer/ChartPreparingState.tsx` | `08-22-views-claude` |
| `src/components/claude-observer/CostAttributionTab.vue` | `src/features/claude/observer/CostAttributionTab.tsx` | `08-22-views-claude` |
| `src/components/claude-observer/SubscriptionDialog.vue` | `src/features/claude/observer/SubscriptionDialog.tsx` | `08-22-views-claude` |
| `src/components/claude-observer/TokenDetailTab.vue` | `src/features/claude/observer/TokenDetailTab.tsx` | `08-22-views-claude` |
| `src/components/claude-observer/UsageInsightPanel.vue` | `src/features/claude/observer/UsageInsightPanel.tsx` | `08-22-views-claude` |

### Codex 域 → `src/features/codex/`（13 行）

| 旧路径 | 新路径 | 归属子任务 |
| --- | --- | --- |
| `src/views/CodexView.vue` | `src/features/codex/CodexView.tsx` | `08-22-views-codex` |
| `src/views/CodexSessionsView.vue` | `src/features/codex/CodexSessionsView.tsx` | `08-22-views-codex` |
| `src/views/CodexSlashCommandsView.vue` | `src/features/codex/CodexSlashCommandsView.tsx` | `08-22-views-codex` |
| `src/components/codex/CodexAccountCard.vue` | `src/features/codex/CodexAccountCard.tsx` | `08-22-views-codex` |
| `src/components/codex/CodexAgentEditorModal.vue` | `src/features/codex/CodexAgentEditorModal.tsx` | `08-22-views-codex` |
| `src/components/codex/CodexAgentSourcesPanel.vue` | `src/features/codex/CodexAgentSourcesPanel.tsx` | `08-22-views-codex` |
| `src/components/codex/CodexProfileEditorModal.vue` | `src/features/codex/CodexProfileEditorModal.tsx` | `08-22-views-codex` |
| `src/components/codex/ProfileCard.vue` | `src/features/codex/ProfileCard.tsx` | `08-22-views-codex` |
| `src/views/codex/components/AddCodexAccountModal.vue` | `src/features/codex/AddCodexAccountModal.tsx` | `08-22-views-codex` |
| `src/views/codex/components/RenameCodexAccountModal.vue` | `src/features/codex/RenameCodexAccountModal.tsx` | `08-22-views-codex` |
| `src/views/codex/components/SaveCodexSessionModal.vue` | `src/features/codex/SaveCodexSessionModal.tsx` | `08-22-views-codex` |
| `src/views/codex/tabs/CodexAuthAccountsTab.vue` | `src/features/codex/CodexAuthAccountsTab.tsx` | `08-22-views-codex` |
| `src/views/codex/tabs/CodexAuthProvidersTab.vue` | `src/features/codex/CodexAuthProvidersTab.tsx` | `08-22-views-codex` |

### Grok / Gemini / OpenCode / generic 保留视图 → 各自 features/（10 行）

| 旧路径 | 新路径 | 归属子任务 |
| --- | --- | --- |
| `src/views/grok/GrokView.vue` | `src/features/grok/GrokView.tsx` | `08-22-views-secondary-platforms` |
| `src/components/grok/GrokProfileCard.vue` | `src/features/grok/GrokProfileCard.tsx` | `08-22-views-secondary-platforms` |
| `src/components/grok/GrokProfileEditorModal.vue` | `src/features/grok/GrokProfileEditorModal.tsx` | `08-22-views-secondary-platforms` |
| `src/views/GeminiCliView.vue` | `src/features/gemini/GeminiCliView.tsx` | `08-22-views-secondary-platforms` |
| `src/views/GeminiSlashCommandsView.vue` | `src/features/gemini/GeminiSlashCommandsView.tsx` | `08-22-views-secondary-platforms` |
| `src/views/OpenCodeView.vue` | `src/features/opencode/OpenCodeView.tsx` | `08-22-views-secondary-platforms` |
| `src/views/OpenCodeProvidersView.vue` | `src/features/opencode/OpenCodeProvidersView.tsx` | `08-22-views-secondary-platforms` |
| `src/components/opencode/OpenCodePageShell.vue` | `src/features/opencode/OpenCodePageShell.tsx` | `08-22-views-secondary-platforms` |
| `src/views/generic/AgentDetailView.vue` | `src/features/platform/AgentDetailView.tsx` | `08-22-views-secondary-platforms` |
| `src/views/generic/SystemPromptsView.vue` | `src/features/platform/SystemPromptsView.tsx` | `08-22-views-secondary-platforms` |

### CheckIn 域 → `src/features/checkin/`（13 行）

| 旧路径 | 新路径 | 归属子任务 |
| --- | --- | --- |
| `src/views/CheckinView.vue` | `src/features/checkin/CheckinView.tsx` | `08-22-views-checkin` |
| `src/views/checkin/CheckinAccountDashboardView.vue` | `src/features/checkin/CheckinAccountDashboardView.tsx` | `08-22-views-checkin` |
| `src/components/CheckinProgressModal.vue` | `src/features/checkin/CheckinProgressModal.tsx` | `08-22-views-checkin` |
| `src/views/checkin/components/AccountActionsMenu.vue` | `src/features/checkin/AccountActionsMenu.tsx` | `08-22-views-checkin` |
| `src/views/checkin/components/AccountDashboardCalendar.vue` | `src/features/checkin/AccountDashboardCalendar.tsx` | `08-22-views-checkin` |
| `src/views/checkin/components/AccountDashboardTrend.vue` | `src/features/checkin/AccountDashboardTrend.tsx` | `08-22-views-checkin` |
| `src/views/checkin/components/AccountFormModal.vue` | `src/features/checkin/AccountFormModal.tsx` | `08-22-views-checkin` |
| `src/views/checkin/components/AccountsTable.vue` | `src/features/checkin/AccountsTable.tsx` | `08-22-views-checkin` |
| `src/views/checkin/components/OAuthWizardModal.vue` | `src/features/checkin/OAuthWizardModal.tsx` | `08-22-views-checkin` |
| `src/views/checkin/tabs/CheckinAccountsTab.vue` | `src/features/checkin/CheckinAccountsTab.tsx` | `08-22-views-checkin` |
| `src/views/checkin/tabs/CheckinImportExportTab.vue` | `src/features/checkin/CheckinImportExportTab.tsx` | `08-22-views-checkin` |
| `src/views/checkin/tabs/CheckinProvidersTab.vue` | `src/features/checkin/CheckinProvidersTab.tsx` | `08-22-views-checkin` |
| `src/views/checkin/tabs/CheckinRecordsTab.vue` | `src/features/checkin/CheckinRecordsTab.tsx` | `08-22-views-checkin` |

### Usage 域 → `src/features/usage/`（28 行）

| 旧路径 | 新路径 | 归属子任务 |
| --- | --- | --- |
| `src/views/DashboardView.vue` | `src/features/usage/DashboardView.tsx` | `08-22-views-usage` |
| `src/views/BudgetView.vue` | `src/features/usage/BudgetView.tsx` | `08-22-views-usage` |
| `src/views/PricingView.vue` | `src/features/usage/PricingView.tsx` | `08-22-views-usage` |
| `src/views/UsageDashboardView.vue` | `src/features/usage/UsageDashboardView.tsx` | `08-22-views-usage` |
| `src/components/usage/LlmusageInstallDialog.vue` | `src/features/usage/LlmusageInstallDialog.tsx` | `08-22-views-usage` |
| `src/components/usage/UsageCostConclusionCard.vue` | `src/features/usage/UsageCostConclusionCard.tsx` | `08-22-views-usage` |
| `src/components/usage/UsageCostTab.vue` | `src/features/usage/UsageCostTab.tsx` | `08-22-views-usage` |
| `src/components/usage/UsageDashboardToolbar.vue` | `src/features/usage/UsageDashboardToolbar.tsx` | `08-22-views-usage` |
| `src/components/usage/UsageDiagnosticsDrawer.vue` | `src/features/usage/UsageDiagnosticsDrawer.tsx` | `08-22-views-usage` |
| `src/components/usage/UsageLogsTab.vue` | `src/features/usage/UsageLogsTab.tsx` | `08-22-views-usage` |
| `src/components/usage/UsageMetricCard.vue` | `src/features/usage/UsageMetricCard.tsx` | `08-22-views-usage` |
| `src/components/usage/UsageModelDistributionCard.vue` | `src/features/usage/UsageModelDistributionCard.tsx` | `08-22-views-usage` |
| `src/components/usage/UsageModelsTab.vue` | `src/features/usage/UsageModelsTab.tsx` | `08-22-views-usage` |
| `src/components/usage/UsageOverviewTab.vue` | `src/features/usage/UsageOverviewTab.tsx` | `08-22-views-usage` |
| `src/components/usage/UsageProjectsTab.vue` | `src/features/usage/UsageProjectsTab.tsx` | `08-22-views-usage` |
| `src/components/usage/UsageProvidersTab.vue` | `src/features/usage/UsageProvidersTab.tsx` | `08-22-views-usage` |
| `src/components/usage/UsageSourceSummaryCard.vue` | `src/features/usage/UsageSourceSummaryCard.tsx` | `08-22-views-usage` |
| `src/components/usage/UsageStaleBanner.vue` | `src/features/usage/UsageStaleBanner.tsx` | `08-22-views-usage` |
| `src/components/usage/UsageTokenBreakdownStrip.vue` | `src/features/usage/UsageTokenBreakdownStrip.tsx` | `08-22-views-usage` |
| `src/components/usage/UsageTokensTab.vue` | `src/features/usage/UsageTokensTab.tsx` | `08-22-views-usage` |
| `src/components/dashboard/DashboardNextActions.vue` | `src/features/usage/DashboardNextActions.tsx` | `08-22-views-usage` |
| `src/components/dashboard/DashboardPlatformMatrix.vue` | `src/features/usage/DashboardPlatformMatrix.tsx` | `08-22-views-usage` |
| `src/components/dashboard/DashboardReadinessLedger.vue` | `src/features/usage/DashboardReadinessLedger.tsx` | `08-22-views-usage` |
| `src/components/dashboard/DashboardSignalStream.vue` | `src/features/usage/DashboardSignalStream.tsx` | `08-22-views-usage` |
| `src/components/dashboard/DashboardUsageMovement.vue` | `src/features/usage/DashboardUsageMovement.tsx` | `08-22-views-usage` |
| `src/components/platform-usage/PlatformUsageInsightPanel.vue` | `src/features/usage/PlatformUsageInsightPanel.tsx` | `08-22-views-usage` |
| `src/components/platform-usage/PlatformUsageRankList.vue` | `src/features/usage/PlatformUsageRankList.tsx` | `08-22-views-usage` |
| `src/components/platform-usage/PlatformUsageTrendChart.vue` | `src/features/usage/PlatformUsageTrendChart.tsx` | `08-22-views-usage` |

### Sync / MCP / Commands / editor / Monitoring / SSH / WSL / tray（24 行）

| 旧路径 | 新路径 | 归属子任务 |
| --- | --- | --- |
| `src/views/SyncView.vue` | `src/features/sync/SyncView.tsx` | `08-22-views-sync-tools` |
| `src/views/MonitoringView.vue` | `src/features/sync/MonitoringView.tsx` | `08-22-views-sync-tools` |
| `src/views/SshManagementView.vue` | `src/features/sync/SshManagementView.tsx` | `08-22-views-sync-tools` |
| `src/views/WslManagementView.vue` | `src/features/sync/WslManagementView.tsx` | `08-22-views-sync-tools` |
| `src/components/sync/SyncAccountDialog.vue` | `src/features/sync/SyncAccountDialog.tsx` | `08-22-views-sync-tools` |
| `src/components/sync/SyncInfoSidebar.vue` | `src/features/sync/SyncInfoSidebar.tsx` | `08-22-views-sync-tools` |
| `src/components/sync/SyncOperationOutputPanel.vue` | `src/features/sync/SyncOperationOutputPanel.tsx` | `08-22-views-sync-tools` |
| `src/components/sync/SyncPassphraseModal.vue` | `src/features/sync/SyncPassphraseModal.tsx` | `08-22-views-sync-tools` |
| `src/views/tray/CodexTrayPanelView.vue` | `src/features/codex/tray/CodexTrayPanelView.tsx` | `08-22-views-sync-tools` |
| `src/views/tray/components/TrayAccountSwitchScreen.vue` | `src/features/codex/tray/TrayAccountSwitchScreen.tsx` | `08-22-views-sync-tools` |
| `src/views/tray/components/TrayOverview.vue` | `src/features/codex/tray/TrayOverview.tsx` | `08-22-views-sync-tools` |
| `src/components/editor/CodeSourceEditor.vue` | `src/features/configs/CodeSourceEditor.tsx` | `08-22-views-sync-tools` |
| `src/components/editor/ConfigSourcePanel.vue` | `src/features/configs/ConfigSourcePanel.tsx` | `08-22-views-sync-tools` |
| `src/components/BaseSlashCommands.vue` | `src/features/commands/BaseSlashCommands.tsx` | `08-22-views-sync-tools` |
| `src/components/CommandFormModal.vue` | `src/features/commands/CommandFormModal.tsx` | `08-22-views-sync-tools` |
| `src/components/CommandList.vue` | `src/features/commands/CommandList.tsx` | `08-22-views-sync-tools` |
| `src/views/SlashCommandsView.vue` | `src/features/commands/SlashCommandsView.tsx` | `08-22-views-sync-tools` |
| `src/views/mcp/McpManagerView.vue` | `src/features/mcp/McpManagerView.tsx` | `08-22-views-sync-tools` |
| `src/components/McpPresetsPanel.vue` | `src/features/mcp/McpPresetsPanel.tsx` | `08-22-views-sync-tools` |
| `src/components/McpSyncPanel.vue` | `src/features/mcp/McpSyncPanel.tsx` | `08-22-views-sync-tools` |
| `src/components/mcp/McpCreatePanel.vue` | `src/features/mcp/McpCreatePanel.tsx` | `08-22-views-sync-tools` |
| `src/components/mcp/McpDetailPanel.vue` | `src/features/mcp/McpDetailPanel.tsx` | `08-22-views-sync-tools` |
| `src/components/mcp/McpImportPanel.vue` | `src/features/mcp/McpImportPanel.tsx` | `08-22-views-sync-tools` |
| `src/components/mcp/McpListPanel.vue` | `src/features/mcp/McpListPanel.tsx` | `08-22-views-sync-tools` |

### Profiles / Configs 域（20 行）

| 旧路径 | 新路径 | 归属子任务 |
| --- | --- | --- |
| `src/views/AppSettingsView.vue` | `src/features/configs/AppSettingsView.tsx` | `08-22-views-profiles-config` |
| `src/views/ConfigsView.vue` | `src/features/configs/ConfigsView.tsx` | `08-22-views-profiles-config` |
| `src/views/ConverterView.vue` | `src/features/configs/ConverterView.tsx` | `08-22-views-profiles-config` |
| `src/components/AddConfigModal.vue` | `src/features/configs/AddConfigModal.tsx` | `08-22-views-profiles-config` |
| `src/components/EditConfigModal.vue` | `src/features/configs/EditConfigModal.tsx` | `08-22-views-profiles-config` |
| `src/components/ConfigCard.vue` | `src/features/configs/ConfigCard.tsx` | `08-22-views-profiles-config` |
| `src/components/configs/ConfigFilters.vue` | `src/features/configs/ConfigFilters.tsx` | `08-22-views-profiles-config` |
| `src/components/configs/ConfigList.vue` | `src/features/configs/ConfigList.tsx` | `08-22-views-profiles-config` |
| `src/components/configs/ProviderStatsModal.vue` | `src/features/configs/ProviderStatsModal.tsx` | `08-22-views-profiles-config` |
| `src/components/profiles/ProfileDiffRows.vue` | `src/features/profiles/ProfileDiffRows.tsx` | `08-22-views-profiles-config` |
| `src/components/profiles/ProfileListRow.vue` | `src/features/profiles/ProfileListRow.tsx` | `08-22-views-profiles-config` |
| `src/components/profiles/ProfilesCommandPalette.vue` | `src/features/profiles/ProfilesCommandPalette.tsx` | `08-22-views-profiles-config` |
| `src/components/profiles/ProfilesHeader.vue` | `src/features/profiles/ProfilesHeader.tsx` | `08-22-views-profiles-config` |
| `src/components/profiles/ProfilesInspector.vue` | `src/features/profiles/ProfilesInspector.tsx` | `08-22-views-profiles-config` |
| `src/components/profiles/ProfilesQuickRail.vue` | `src/features/profiles/ProfilesQuickRail.tsx` | `08-22-views-profiles-config` |
| `src/components/profiles/ProfilesRawEditorPanel.vue` | `src/features/profiles/ProfilesRawEditorPanel.tsx` | `08-22-views-profiles-config` |
| `src/components/profiles/ProfilesSection.vue` | `src/features/profiles/ProfilesSection.tsx` | `08-22-views-profiles-config` |
| `src/components/profiles/ProfilesStatStrip.vue` | `src/features/profiles/ProfilesStatStrip.tsx` | `08-22-views-profiles-config` |
| `src/components/profiles/ProfilesToolbar.vue` | `src/features/profiles/ProfilesToolbar.tsx` | `08-22-views-profiles-config` |
| `src/components/provider-templates/ProviderTemplateSelector.vue` | `src/features/profiles/ProviderTemplateSelector.tsx` | `08-22-views-profiles-config` |

### src/utils 31 文件（19 原样复用 + 12 需接线）（31 行）

| 旧路径 | 新路径 | 归属子任务 |
| --- | --- | --- |
| `src/utils/ansiRenderer.ts` | `src/utils/ansiRenderer.ts` | `08-22-react-foundation` |
| `src/utils/apexChartsCore.ts` | `src/features/usage/apexChartsCore.ts` | `08-22-views-usage`（迁移时重写为 react-apexcharts 入口） |
| `src/utils/claudeProfileEditor.ts` | `src/utils/claudeProfileEditor.ts` | `08-22-react-foundation` |
| `src/utils/claudeProfileFields.ts` | `src/utils/claudeProfileFields.ts` | `08-22-react-foundation` |
| `src/utils/claudeProfiles.ts` | `src/features/profiles/claudeProfiles.ts` | `08-22-views-profiles-config`（随共享层迁移重写描述符导入） |
| `src/utils/clipboard.ts` | `src/utils/clipboard.ts` | `08-22-react-foundation` |
| `src/utils/codexProfiles.ts` | `src/features/profiles/codexProfiles.ts` | `08-22-views-profiles-config`（随共享层迁移重写描述符导入） |
| `src/utils/codexHelpers.ts` | `src/utils/codexHelpers.ts` | `08-22-react-foundation` |
| `src/utils/codexProfileEditor.ts` | `src/utils/codexProfileEditor.ts` | `08-22-react-foundation` |
| `src/utils/download.ts` | `src/utils/download.ts` | `08-22-react-foundation` |
| `src/utils/errorHandler.ts` | `src/utils/errorHandler.ts` | `08-22-react-foundation` |
| `src/utils/fontPreferences.ts` | `src/utils/fontPreferences.ts` | `08-22-react-foundation` |
| `src/utils/grokProfileEditor.ts` | `src/utils/grokProfileEditor.ts` | `08-22-react-foundation` |
| `src/utils/grokProfiles.ts` | `src/features/profiles/grokProfiles.ts` | `08-22-views-profiles-config`（随共享层迁移重写描述符导入） |
| `src/utils/grokSettings.ts` | `src/utils/grokSettings.ts` | `08-22-react-foundation` |
| `src/utils/logRedact.ts` | `src/utils/logRedact.ts` | `08-22-react-foundation` |
| `src/utils/logger.ts` | `src/shell/utils/logger.ts` | `08-22-shell-port` |
| `src/utils/nativeWindowAppearance.ts` | `src/shell/utils/nativeWindowAppearance.ts` | `08-22-shell-port` |
| `src/utils/opencode.ts` | `src/utils/opencode.ts` | `08-22-react-foundation` |
| `src/utils/perfTelemetry.ts` | `src/shell/utils/perfTelemetry.ts` | `08-22-shell-port` |
| `src/utils/profileDiff.ts` | `src/utils/profileDiff.ts` | `08-22-react-foundation` |
| `src/utils/providerTemplates.ts` | `src/utils/providerTemplates.ts` | `08-22-react-foundation` |
| `src/utils/runtimeState.ts` | `src/utils/runtimeState.ts` | `08-22-react-foundation` |
| `src/utils/sanitize.ts` | `src/utils/sanitize.ts` | `08-22-react-foundation` |
| `src/utils/scheduling.ts` | `src/utils/scheduling.ts` | `08-22-react-foundation` |
| `src/utils/startupRecovery.ts` | `src/shell/utils/startupRecovery.ts` | `08-22-shell-port` |
| `src/utils/tauriRuntime.ts` | `src/shell/utils/tauriRuntime.ts` | `08-22-shell-port` |
| `src/utils/tauriWindow.ts` | `src/shell/utils/tauriWindow.ts` | `08-22-shell-port` |
| `src/utils/text.ts` | `src/utils/text.ts` | `08-22-react-foundation` |
| `src/utils/themeBootstrap.ts` | `src/shell/utils/themeBootstrap.ts` | `08-22-shell-port` |
| `src/utils/windowChrome.ts` | `src/shell/utils/windowChrome.ts` | `08-22-shell-port` |
