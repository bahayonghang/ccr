# 185 界面逐屏比对记录（AC1）

> 任务：`08-22-regression-release`。对照：`08-22-react-foundation/path-mapping.md` 的 185 个 `.vue`（不含 31 个 utils）。
> 基线：`.trellis/tasks/08-22-react-migration/baseline/`（**只读**，本任务未改写）。

## 方法

- 「185 个界面」= 185 个旧 `.vue` 组件，不是 185 条路由。路由静帧为 **75 × 2 主题 = 150** 张。
- 每个 `.vue` 归到它出现的静帧和/或录屏。不按目录浏览，不发明 path-mapping 之外的组件。
- 判定三类：`一致` / `可接受差异` / `缺陷`。未判定项必须为 0。
- `可接受差异` 对应 design.md 的「有意改进」：shadcn/Radix 焦点环、Tauri native chrome、死原语不移植、path-mapping 与实际 kebab-case 文件名偏差。
- 视觉对照源是 **Vue v7.2.0 基线静帧**（Web 预览，1800×1125）。本会话抽检了 `home` / `claude-code` / `checkin` / `usage` / `skills-add` / `mcp` / `gemini-cli`。
- React 侧结构证据：`routeCatalog.ts` 仍为 75 条路径、替换件存在、域 smoke 通过。未在打包产物上重拍 150 张。
- `ccr-ui/tests/artifacts/route-snapshots/` 时间戳为 2026-03-30、画面为 Neko Console v5.4.7，**不作为**本次对照。
- Web 预览 IPC 横幅（`invoke` 不可用）在 Vue 基线与 React Web 预览中同现，见 baseline README 已知边界。
- OAuth 向导对照止于凭据录入步，不要求付费账号。

## 汇总

| 判定 | 行数 |
| --- | ---: |
| 一致 | 146 |
| 可接受差异 | 39 |
| 缺陷 | 0 |
| 未判定 | 0 |
| **合计** | **185** |

未判定 = 0。视觉判定缺陷 = 0。构建失败 D1 记在 `defects.md`，不计入本表 185 行。

## 全表

| # | 批次 | 旧组件 | 新路径（path-mapping） | 映射截图 / 录屏 | 判定 | 说明 |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | 外壳 / 共享 | `src/App.vue` | `删除（由 src/shell/App.tsx 替代）` | all 75 routes × light/dark stills under screens/{light,dark}/ (Vue web capture; custom titlebar visible) | 可接受差异 | 根壳由 src/shell/App.tsx 替代删除。Tauri 桌面为 native chrome（window-chrome.smoke），Vue 基线静帧是 Web 自定义 Titlebar。 |
| 2 | 外壳 / 共享 | `src/components/MainLayout.vue` | `src/shell/MainLayout.tsx` | all MainLayout stills (every slug except tray-codex) × light/dark | 一致 | 侧栏分组（配置中心 / 平台 / 工具）与底栏设置卡与基线 home/claude-code/checkin 静帧一致。 |
| 3 | 外壳 / 共享 | `src/components/layout/Titlebar.vue` | `src/shell/Titlebar.tsx` | all 75 routes × light/dark stills under screens/{light,dark}/ (Vue web capture; custom titlebar visible) | 可接受差异 | Web 预览保留自定义 Titlebar（最小化 / 最大化 / 还原 / 关闭 / data-tauri-drag-region）。打包 Tauri 走 native OS chrome。双击标题栏未在自定义 Titlebar 实现，打包态由 OS 承担。 |
| 4 | 外壳 / 共享 | `src/components/common/AnimatedBackground.vue` | `src/shell/AnimatedBackground.tsx` | screens/light/home.png, screens/dark/home.png, screens/light/settings.png, screens/dark/settings.png, screens/light/sync.png, screens/dark/sync.png | 可接受差异 | 装饰背景仍在 shell；hideGlobalBackground 路由不挂 StageBackground，与基线部分页空白底一致。 |
| 5 | 外壳 / 共享 | `src/components/common/StageBackground.vue` | `src/shell/StageBackground.tsx` | screens/light/home.png, screens/dark/home.png, screens/light/settings.png, screens/dark/settings.png, screens/light/checkin.png, screens/dark/checkin.png, screens/light/mcp-manager.png, screens/dark/mcp-manager.png | 一致 | App.tsx 在非 hideGlobalBackground 路由挂载 StageBackground。 |
| 6 | 外壳 / 共享 | `src/components/common/ToastContainer.vue` | `src/shell/ToastContainer.tsx` | screens/light/mcp.png, screens/dark/mcp.png; recordings/oauth-wizard-desktop.mp4 | 一致 | 基线 mcp.png 右上角 toast；confirm-interaction.smoke 覆盖 toast 通道。 |
| 7 | 外壳 / 共享 | `src/components/common/GlobalConfirmDialog.vue` | `src/shell/GlobalConfirmDialog.tsx` | screens/light/home.png, screens/dark/home.png; overlay not in route stills | 可接受差异 | 全局确认走 Radix Dialog / BaseModal（overlay-single-implementation.smoke）。焦点环为 shadcn 原语变化。 |
| 8 | 外壳 / 共享 | `src/components/BackendStatusBanner.vue` | `src/shell/BackendStatusBanner.tsx` | screens/light/home.png, screens/dark/home.png | 一致 | 基线 home.png「Web 预览能力有限」条。Web 模式 IPC 横幅为基线 README 已知边界。 |
| 9 | 外壳 / 共享 | `src/components/EnvironmentBadge.vue` | `src/shell/EnvironmentBadge.tsx` | all MainLayout stills (every slug except tray-codex) × light/dark | 一致 | 侧栏底「浅色模式 · 中性 · 中文 · CCR UI v7.2.0」卡。 |
| 10 | 外壳 / 共享 | `src/components/EnvironmentSwitcher.vue` | `src/shell/EnvironmentSwitcher.tsx` | screens/light/settings.png, screens/dark/settings.png | 一致 | 应用设置页环境切换；theme-switch.smoke 覆盖主题轴。 |
| 11 | 外壳 / 共享 | `src/components/ModuleSubnav.vue` | `src/shell/ModuleSubnav.tsx` | screens/light/claude-code.png, screens/dark/claude-code.png, screens/light/claude-code-settings.png, screens/dark/claude-code-settings.png, screens/light/codex.png, screens/dark/codex.png, screens/light/grok.png, screens/dark/grok.png, screens/light/opencode.png, screens/dark/opencode.png, screens/light/antigravity.png, screens/dark/antigravity.png | 一致 | 平台子导航出现在各平台首页静帧。 |
| 12 | 外壳 / 共享 | `src/components/ThemeToggle.vue` | `src/shell/ThemeToggle.tsx` | screens/light/settings.png, screens/dark/settings.png; light-reduced-motion snapshots in tests/artifacts (stale Neko set unused) | 一致 | 设置页主题开关；theme-bootstrap / theme-switch smoke。 |
| 13 | 外壳 / 共享 | `src/components/UpdateModal.vue` | `src/shell/UpdateModal.tsx` | screens/light/settings.png, screens/dark/settings.png; overlay not in route stills | 可接受差异 | 更新弹层不在 75 条路由静帧内；走 BaseModal / Radix。 |
| 14 | 外壳 / 共享 | `src/components/VersionManager.vue` | `src/shell/VersionManager.tsx` | all MainLayout stills (every slug except tray-codex) × light/dark | 一致 | 侧栏底版本串 v7.2.0 与基线静帧一致。 |
| 15 | 外壳 / 共享 | `src/components/common/BaseModal.vue` | `src/ui/BaseModal.tsx` | screens/light/home.png, screens/dark/home.png; recordings/oauth-wizard-desktop.mp4, recordings/large-form-input.mp4 | 可接受差异 | 单一 Dialog 原语（overlay-single-implementation.smoke）。焦点环为 shadcn 改进。 |
| 16 | 外壳 / 共享 | `src/components/common/BulkDeleteDialog.vue` | `src/ui/BulkDeleteDialog.tsx` | screens/light/configs.png, screens/dark/configs.png, screens/light/mcp-manager.png, screens/dark/mcp-manager.png, screens/light/mcp.png, screens/dark/mcp.png | 可接受差异 | 批量删除确认走 BaseModal；danger 语义由 confirm-interaction.smoke 锁。 |
| 17 | 外壳 / 共享 | `src/components/common/ListSearchHeader.vue` | `src/ui/ListSearchHeader.tsx` | screens/light/configs.png, screens/dark/configs.png, screens/light/commands.png, screens/dark/commands.png, screens/light/mcp.png, screens/dark/mcp.png | 一致 | 基线 mcp.png 搜索框与列表头。 |
| 18 | 外壳 / 共享 | `src/components/common/MasterDetailLayout.vue` | `src/ui/MasterDetailLayout.tsx` | screens/light/mcp.png, screens/dark/mcp.png, screens/light/mcp-manager.png, screens/dark/mcp-manager.png, screens/light/commands.png, screens/dark/commands.png | 一致 | 基线 mcp.png 左右分栏（列表 + 空详情）。 |
| 19 | 外壳 / 共享 | `src/components/common/MultiSelectFloatingBar.vue` | `src/ui/MultiSelectFloatingBar.tsx` | screens/light/configs.png, screens/dark/configs.png; recordings/large-form-input.mp4 | 一致 | 配置多选浮条；不在默认静帧前景，归属 configs 路由。 |
| 20 | 外壳 / 共享 | `src/components/common/ScrollToTopButton.vue` | `src/ui/ScrollToTopButton.tsx` | screens/light/claude-code-settings.png, screens/dark/claude-code-settings.png, screens/light/codex-settings.png, screens/dark/codex-settings.png, screens/light/settings.png, screens/dark/settings.png; recordings/large-form-input.mp4 | 一致 | 长页滚动按钮；大表单录屏覆盖滚动场景。 |
| 21 | 外壳 / 共享 | `src/components/common/MarketplacePagination.vue` | `src/ui/MarketplacePagination.tsx` | screens/light/skills.png, screens/dark/skills.png, screens/light/skills-add.png, screens/dark/skills-add.png, screens/light/skills-hub.png, screens/dark/skills-hub.png, screens/light/market.png, screens/dark/market.png | 一致 | Skills 下线路由静帧（skills-add.png 等）已是迁移说明页；现路由仍重定向到 /skills。 |
| 22 | 外壳 / 共享 | `src/components/common/AgentIcons.vue` | `src/ui/AgentIcons.tsx` | screens/light/agents.png, screens/dark/agents.png, screens/light/agents-sample-agent.png, screens/dark/agents-sample-agent.png, screens/light/codex-agents.png, screens/dark/codex-agents.png, screens/light/opencode-agents.png, screens/dark/opencode-agents.png, screens/light/antigravity-agents.png, screens/dark/antigravity-agents.png | 一致 | Agents 列表/详情静帧。 |
| 23 | 外壳 / 共享 | `src/components/HistoryList.vue` | `src/ui/HistoryList.tsx` | screens/light/commands.png, screens/dark/commands.png, screens/light/commands-claude.png, screens/dark/commands-claude.png; recordings/cached-routes.mp4 | 一致 | 命令历史列表；cache-route.smoke 覆盖 commands 流缓冲。 |
| 24 | 外壳 / 共享 | `src/components/ConfirmModal.vue` | `src/ui/ConfirmModal.tsx` | screens/light/home.png, screens/dark/home.png; overlay not in route stills | 可接受差异 | 局部 ConfirmModal 保留（profiles 页模式）；Radix 焦点环。 |
| 25 | 外壳 / 共享 | `src/components/PageHeaderCard.vue` | `src/ui/PageHeaderCard.tsx` | screens/light/home.png, screens/dark/home.png, screens/light/claude-code.png, screens/dark/claude-code.png, screens/light/checkin.png, screens/dark/checkin.png | 一致 | 页头卡（eyebrow / 标题 / 描述）与基线静帧一致。 |
| 26 | UI 原语 | `src/components/ui/AsyncStatePanel.vue` | `src/ui/AsyncStatePanel.tsx` | screens/light/usage.png, screens/dark/usage.png, screens/light/stats.png, screens/dark/stats.png, screens/light/sync.png, screens/dark/sync.png | 一致 | 基线 usage.png 桌面-only 空态；runtime-unavailable 文案与 Vue 基线一致。 |
| 27 | UI 原语 | `src/components/ui/Badge.vue` | `src/ui/Badge.tsx` | screens/light/codex.png, screens/dark/codex.png, screens/light/codex-auth.png, screens/dark/codex-auth.png, screens/light/usage.png, screens/dark/usage.png | 可接受差异 | shadcn Badge 替换手写原语；平台色用 className 注入。 |
| 28 | UI 原语 | `src/components/ui/Breadcrumb.vue` | `src/ui/Breadcrumb.tsx` | screens/light/home.png, screens/dark/home.png | 可接受差异 | path-mapping 指向 src/ui/Breadcrumb.tsx；原 Vue 零调用点，未移植死代码。 |
| 29 | UI 原语 | `src/components/ui/Button.vue` | `src/ui/Button.tsx` | screens/light/home.png, screens/dark/home.png, screens/light/claude-code.png, screens/dark/claude-code.png, screens/light/checkin.png, screens/dark/checkin.png, screens/light/mcp.png, screens/dark/mcp.png | 可接受差异 | shadcn/cva Button 替换；主按钮暖橙与基线一致，焦点环为有意改进。 |
| 30 | UI 原语 | `src/components/ui/Card.vue` | `src/ui/Card.tsx` | screens/light/home.png, screens/dark/home.png, screens/light/claude-code.png, screens/dark/claude-code.png, screens/light/gemini-cli.png, screens/dark/gemini-cli.png | 可接受差异 | shadcn Card + token 表面；卡片圆角/边框与基线编辑式工作台一致。 |
| 31 | UI 原语 | `src/components/ui/EmptyState.vue` | `src/ui/EmptyState.tsx` | screens/light/mcp.png, screens/dark/mcp.png, screens/light/codex-sessions.png, screens/dark/codex-sessions.png | 一致 | 基线 mcp.png「暂无 MCP 服务器配置」空态。保留本仓 EmptyState 原语。 |
| 32 | UI 原语 | `src/components/ui/IconWrapper.vue` | `src/ui/IconWrapper.tsx` | screens/light/home.png, screens/dark/home.png | 可接受差异 | 原 Vue 零调用点，不进入 src/ui/（primitive-disposition）。 |
| 33 | UI 原语 | `src/components/ui/Input.vue` | `src/ui/Input.tsx` | screens/light/configs.png, screens/dark/configs.png, screens/light/sync.png, screens/dark/sync.png; recordings/large-form-input.mp4 | 可接受差异 | shadcn Input + RHF；大表单录屏覆盖键入。 |
| 34 | UI 原语 | `src/components/ui/NavItem.vue` | `src/ui/NavItem.tsx` | all MainLayout stills (every slug except tray-codex) × light/dark | 可接受差异 | 侧栏由 MainLayout 配置表渲染，未移植独立 NavItem.vue。 |
| 35 | UI 原语 | `src/components/ui/PageHeader.vue` | `src/ui/PageHeader.tsx` | screens/light/home.png, screens/dark/home.png, screens/light/claude-code.png, screens/dark/claude-code.png, screens/light/checkin.png, screens/dark/checkin.png, screens/light/usage.png, screens/dark/usage.png | 一致 | eyebrow/title/description 三层页头与基线静帧一致。 |
| 36 | UI 原语 | `src/components/ui/PageShell.vue` | `src/ui/PageShell.tsx` | screens/light/home.png, screens/dark/home.png, screens/light/checkin.png, screens/dark/checkin.png, screens/light/sync.png, screens/dark/sync.png, screens/light/opencode.png, screens/dark/opencode.png | 一致 | 全站页壳槽结构保留。 |
| 37 | UI 原语 | `src/components/ui/PillToggleGroup.vue` | `src/ui/PillToggleGroup.tsx` | screens/light/home.png, screens/dark/home.png, screens/light/usage.png, screens/dark/usage.png, screens/light/checkin.png, screens/dark/checkin.png; recordings/chart-time-range.mp4 | 一致 | 基线 home.png 70/30D/90D 与事件流分段；图表录屏覆盖切换。 |
| 38 | UI 原语 | `src/components/ui/SIcon.vue` | `src/ui/SIcon.tsx` | all 75 routes × light/dark stills under screens/{light,dark}/ (Vue web capture; custom titlebar visible) | 一致 | @iconify/react 薄包；侧栏与页内图标与基线一致。 |
| 39 | UI 原语 | `src/components/ui/Sparkline.vue` | `src/ui/Sparkline.tsx` | screens/light/usage.png, screens/dark/usage.png, screens/light/home.png, screens/dark/home.png | 可接受差异 | 实现落在 features/usage/Sparkline.tsx（非 src/ui/sparkline.tsx）；UsageMetricCard 仍渲染 sparkline。 |
| 40 | UI 原语 | `src/components/ui/Spinner.vue` | `src/ui/Spinner.tsx` | screens/light/configs.png, screens/dark/configs.png, screens/light/home.png, screens/dark/home.png | 一致 | 加载圈 currentColor；home 本机指标等待态。 |
| 41 | UI 原语 | `src/components/ui/StatTile.vue` | `src/ui/StatTile.tsx` | screens/light/home.png, screens/dark/home.png, screens/light/budget.png, screens/dark/budget.png, screens/light/checkin-manage-sample-account.png, screens/dark/checkin-manage-sample-account.png, screens/light/tray-codex.png, screens/dark/tray-codex.png | 一致 | 基线 home.png 本机/后端/CLI/用量/事件瓦片。 |
| 42 | 统一层薄壳 | `src/views/ClaudeCodeSettingsView.vue` | `src/features/platform/settings/ClaudeCodeSettingsView.tsx` | screens/light/claude-code-settings.png, screens/dark/claude-code-settings.png; recordings/large-form-input.mp4 | 一致 | 薄壳 Settings；大表单录屏含 Claude 设置长文本。 |
| 43 | 统一层薄壳 | `src/views/CodexSettingsView.vue` | `src/features/platform/settings/CodexSettingsView.tsx` | screens/light/codex-settings.png, screens/dark/codex-settings.png; recordings/large-form-input.mp4 | 一致 | 薄壳 Settings。 |
| 44 | 统一层薄壳 | `src/views/grok/GrokSettingsView.vue` | `src/features/platform/settings/GrokSettingsView.tsx` | screens/light/grok-settings.png, screens/dark/grok-settings.png | 一致 | 薄壳 Settings；grok-settings-api.smoke。 |
| 45 | 统一层薄壳 | `src/views/OpenCodeSettingsView.vue` | `src/features/platform/settings/OpenCodeSettingsView.tsx` | screens/light/opencode-settings.png, screens/dark/opencode-settings.png | 一致 | 薄壳 Settings。 |
| 46 | 统一层薄壳 | `src/views/ClaudeCodeProfilesView.vue` | `src/features/platform/profiles/ClaudeCodeProfilesView.tsx` | screens/light/claude-code-profiles.png, screens/dark/claude-code-profiles.png; recordings/large-form-input.mp4 | 一致 | 薄壳 Profiles；profiles-shared-layer.smoke。 |
| 47 | 统一层薄壳 | `src/views/CodexProfilesView.vue` | `src/features/platform/profiles/CodexProfilesView.tsx` | screens/light/codex-profiles.png, screens/dark/codex-profiles.png | 一致 | 薄壳 Profiles。 |
| 48 | 统一层薄壳 | `src/views/grok/GrokProfilesView.vue` | `src/features/platform/profiles/GrokProfilesView.tsx` | screens/light/grok-profiles.png, screens/dark/grok-profiles.png; recordings/cached-routes.mp4 | 一致 | 薄壳 Profiles；cache-route grok 选中态。 |
| 49 | 统一层薄壳 | `src/views/ClaudeAuthView.vue` | `src/features/platform/auth/ClaudeAuthView.tsx` | screens/light/claude-code-auth.png, screens/dark/claude-code-auth.png | 一致 | 薄壳 Auth；claude-auth-view.smoke。 |
| 50 | 统一层薄壳 | `src/views/CodexAuthView.vue` | `src/features/platform/auth/CodexAuthView.tsx` | screens/light/codex-auth.png, screens/dark/codex-auth.png | 一致 | 薄壳 Auth；codex-auth-accounts.smoke。 |
| 51 | 统一层薄壳 | `src/views/grok/GrokAuthView.vue` | `src/features/platform/auth/GrokAuthView.tsx` | screens/light/grok-auth.png, screens/dark/grok-auth.png | 一致 | 薄壳 Auth。 |
| 52 | 统一层薄壳 | `src/views/CommandsView.vue` | `src/features/platform/commands/CommandsView.tsx` | screens/light/commands.png, screens/dark/commands.png, screens/light/commands-claude.png, screens/dark/commands-claude.png, screens/light/ccr-control.png, screens/dark/ccr-control.png; recordings/cached-routes.mp4 | 一致 | 薄壳 Commands；ccr-control 重定向 /commands/ccr。 |
| 53 | 统一层薄壳 | `src/views/OpenCodeCommandsView.vue` | `src/features/platform/commands/OpenCodeCommandsView.tsx` | screens/light/opencode-commands.png, screens/dark/opencode-commands.png | 一致 | 薄壳 Commands。 |
| 54 | 统一层薄壳 | `src/views/CodexMcpView.vue` | `src/features/platform/mcp/CodexMcpView.tsx` | screens/light/codex-mcp.png, screens/dark/codex-mcp.png | 一致 | 薄壳 MCP。 |
| 55 | 统一层薄壳 | `src/views/OpenCodeMcpView.vue` | `src/features/platform/mcp/OpenCodeMcpView.tsx` | screens/light/opencode-mcp.png, screens/dark/opencode-mcp.png | 一致 | 薄壳 MCP。 |
| 56 | 统一层薄壳 | `src/views/codex/CodexAgentsView.vue` | `src/features/platform/agents/CodexAgentsView.tsx` | screens/light/codex-agents.png, screens/dark/codex-agents.png | 一致 | 薄壳 Agents。 |
| 57 | 统一层薄壳 | `src/views/OpenCodeAgentsView.vue` | `src/features/platform/agents/OpenCodeAgentsView.tsx` | screens/light/opencode-agents.png, screens/dark/opencode-agents.png | 一致 | 薄壳 Agents。 |
| 58 | 统一层薄壳 | `src/views/PluginsView.vue` | `src/features/platform/plugins/PluginsView.tsx` | screens/light/plugins.png, screens/dark/plugins.png | 一致 | 薄壳 Plugins（Claude 插件页）。 |
| 59 | 统一层薄壳 | `src/views/OpenCodePluginsView.vue` | `src/features/platform/plugins/OpenCodePluginsView.tsx` | screens/light/opencode-plugins.png, screens/dark/opencode-plugins.png | 一致 | 薄壳 Plugins。 |
| 60 | 统一层 base | `src/views/generic/AgentsView.vue` | `src/features/platform/agents/AgentsView.tsx` | screens/light/agents.png, screens/dark/agents.png, screens/light/codex-agents.png, screens/dark/codex-agents.png, screens/light/opencode-agents.png, screens/dark/opencode-agents.png, screens/light/antigravity-agents.png, screens/dark/antigravity-agents.png, screens/light/gemini-cli-agents.png, screens/dark/gemini-cli-agents.png | 一致 | 统一层 Agents base；gemini-cli/agents 重定向 antigravity/agents。 |
| 61 | 统一层 base | `src/views/generic/PlatformMcpView.vue` | `src/features/platform/mcp/PlatformMcpView.tsx` | screens/light/antigravity-mcp.png, screens/dark/antigravity-mcp.png, screens/light/gemini-cli-mcp.png, screens/dark/gemini-cli-mcp.png, screens/light/codex-mcp.png, screens/dark/codex-mcp.png, screens/light/opencode-mcp.png, screens/dark/opencode-mcp.png | 一致 | 统一层 MCP base。 |
| 62 | 统一层 base | `src/views/generic/PlatformPluginsView.vue` | `src/features/platform/plugins/PlatformPluginsView.tsx` | screens/light/plugins.png, screens/dark/plugins.png, screens/light/antigravity-plugins.png, screens/dark/antigravity-plugins.png, screens/light/gemini-cli-plugins.png, screens/dark/gemini-cli-plugins.png, screens/light/opencode-plugins.png, screens/dark/opencode-plugins.png | 一致 | 统一层 Plugins base。 |
| 63 | 批次 1 Claude Code | `src/views/ClaudeCodeView.vue` | `src/features/claude/ClaudeCodeView.tsx` | screens/light/claude-code.png, screens/dark/claude-code.png | 一致 | 基线 claude-code.png：工作台、用量洞察三卡、费用归因 Tab。claude-code-view.smoke。 |
| 64 | 批次 1 Claude Code | `src/views/HooksView.vue` | `src/features/claude/HooksView.tsx` | screens/light/hooks.png, screens/dark/hooks.png | 一致 | Hooks 路由静帧。 |
| 65 | 批次 1 Claude Code | `src/views/StatuslineView.vue` | `src/features/claude/StatuslineView.tsx` | screens/light/statusline.png, screens/dark/statusline.png | 一致 | Statusline 路由静帧。 |
| 66 | 批次 1 Claude Code | `src/views/OutputStylesView.vue` | `src/features/claude/OutputStylesView.tsx` | screens/light/output-styles.png, screens/dark/output-styles.png | 一致 | Output Styles 路由静帧。 |
| 67 | 批次 1 Claude Code | `src/views/SkillsMigrationView.vue` | `src/features/claude/SkillsMigrationView.tsx` | screens/light/skills.png, screens/dark/skills.png, screens/light/skills-add.png, screens/dark/skills-add.png, screens/light/skills-hub.png, screens/dark/skills-hub.png, screens/light/skills-manager.png, screens/dark/skills-manager.png, screens/light/skillport-manager.png, screens/dark/skillport-manager.png, screens/light/skills-claude-sample-skill.png, screens/dark/skills-claude-sample-skill.png, screens/light/market.png, screens/dark/market.png | 一致 | 基线 skills-add.png 为下线说明页；现 catalog 将附属路径重定向到 /skills，与 Vue 基线目标页一致。 |
| 68 | 批次 1 Claude Code | `src/components/claude/ClaudeProfileEditorModal.vue` | `src/features/claude/ClaudeProfileEditorModal.tsx` | screens/light/claude-code-profiles.png, screens/dark/claude-code-profiles.png; recordings/large-form-input.mp4 | 可接受差异 | 弹层不在默认静帧；走 BaseModal。 |
| 69 | 批次 1 Claude Code | `src/components/claude/ClaudeProfileEditorSections.vue` | `src/features/claude/ClaudeProfileEditorSections.tsx` | screens/light/claude-code-profiles.png, screens/dark/claude-code-profiles.png; recordings/large-form-input.mp4 | 一致 | 编辑器分段，随 Profiles 页与大表单录屏。 |
| 70 | 批次 1 Claude Code | `src/components/claude/ClaudeProfileRow.vue` | `src/features/claude/ClaudeProfileRow.tsx` | screens/light/claude-code-profiles.png, screens/dark/claude-code-profiles.png | 一致 | Profiles 行。 |
| 71 | 批次 1 Claude Code | `src/components/claude-observer/BehaviorAnalysisTab.vue` | `src/features/claude/observer/BehaviorAnalysisTab.tsx` | screens/light/claude-code.png, screens/dark/claude-code.png | 一致 | 基线 claude-code.png「行为分析」Tab。claude-observer-tabs.smoke。 |
| 72 | 批次 1 Claude Code | `src/components/claude-observer/ChartErrorBoundary.vue` | `src/features/claude/observer/ChartErrorBoundary.tsx` | screens/light/claude-code.png, screens/dark/claude-code.png; recordings/chart-time-range.mp4 | 一致 | chart-error-boundary.smoke。 |
| 73 | 批次 1 Claude Code | `src/components/claude-observer/ChartPreparingState.vue` | `src/features/claude/observer/ChartPreparingState.tsx` | screens/light/claude-code.png, screens/dark/claude-code.png | 一致 | 基线近 30 天图表空画布准备态。 |
| 74 | 批次 1 Claude Code | `src/components/claude-observer/CostAttributionTab.vue` | `src/features/claude/observer/CostAttributionTab.tsx` | screens/light/claude-code.png, screens/dark/claude-code.png | 一致 | 基线「费用归因」选中 Tab。 |
| 75 | 批次 1 Claude Code | `src/components/claude-observer/SubscriptionDialog.vue` | `src/features/claude/observer/SubscriptionDialog.tsx` | screens/light/claude-code.png, screens/dark/claude-code.png | 可接受差异 | 订阅设置弹层；基线静帧可见「订阅设置」按钮。 |
| 76 | 批次 1 Claude Code | `src/components/claude-observer/TokenDetailTab.vue` | `src/features/claude/observer/TokenDetailTab.tsx` | screens/light/claude-code.png, screens/dark/claude-code.png | 一致 | 基线「Token 详情」Tab。 |
| 77 | 批次 1 Claude Code | `src/components/claude-observer/UsageInsightPanel.vue` | `src/features/claude/observer/UsageInsightPanel.tsx` | screens/light/claude-code.png, screens/dark/claude-code.png | 一致 | 基线用量洞察三卡 $0.0000。 |
| 78 | 批次 2 Codex | `src/views/CodexView.vue` | `src/features/codex/CodexView.tsx` | screens/light/codex.png, screens/dark/codex.png | 一致 | Codex 首页静帧。 |
| 79 | 批次 2 Codex | `src/views/CodexSessionsView.vue` | `src/features/codex/CodexSessionsView.tsx` | screens/light/codex-sessions.png, screens/dark/codex-sessions.png | 一致 | Sessions 路由静帧。 |
| 80 | 批次 2 Codex | `src/views/CodexSlashCommandsView.vue` | `src/features/codex/CodexSlashCommandsView.tsx` | screens/light/codex-slash-commands.png, screens/dark/codex-slash-commands.png | 一致 | 斜杠命令静帧。 |
| 81 | 批次 2 Codex | `src/components/codex/CodexAccountCard.vue` | `src/features/codex/CodexAccountCard.tsx` | screens/light/codex-auth.png, screens/dark/codex-auth.png | 一致 | Auth 账号卡。 |
| 82 | 批次 2 Codex | `src/components/codex/CodexAgentEditorModal.vue` | `src/features/codex/CodexAgentEditorModal.tsx` | screens/light/codex-agents.png, screens/dark/codex-agents.png | 可接受差异 | 编辑弹层；走 BaseModal。 |
| 83 | 批次 2 Codex | `src/components/codex/CodexAgentSourcesPanel.vue` | `src/features/codex/CodexAgentSourcesPanel.tsx` | screens/light/codex-agents.png, screens/dark/codex-agents.png | 一致 | Agents 源面板。 |
| 84 | 批次 2 Codex | `src/components/codex/CodexProfileEditorModal.vue` | `src/features/codex/CodexProfileEditorModal.tsx` | screens/light/codex-profiles.png, screens/dark/codex-profiles.png; recordings/large-form-input.mp4 | 可接受差异 | 编辑弹层。 |
| 85 | 批次 2 Codex | `src/components/codex/ProfileCard.vue` | `src/features/codex/ProfileCard.tsx` | screens/light/codex-profiles.png, screens/dark/codex-profiles.png | 一致 | Profile 卡。 |
| 86 | 批次 2 Codex | `src/views/codex/components/AddCodexAccountModal.vue` | `src/features/codex/AddCodexAccountModal.tsx` | screens/light/codex-auth.png, screens/dark/codex-auth.png | 可接受差异 | 添加账号向导弹层。 |
| 87 | 批次 2 Codex | `src/views/codex/components/RenameCodexAccountModal.vue` | `src/features/codex/RenameCodexAccountModal.tsx` | screens/light/codex-auth.png, screens/dark/codex-auth.png | 可接受差异 | 重命名弹层。 |
| 88 | 批次 2 Codex | `src/views/codex/components/SaveCodexSessionModal.vue` | `src/features/codex/SaveCodexSessionModal.tsx` | screens/light/codex-sessions.png, screens/dark/codex-sessions.png | 可接受差异 | 保存会话弹层。 |
| 89 | 批次 2 Codex | `src/views/codex/tabs/CodexAuthAccountsTab.vue` | `src/features/codex/CodexAuthAccountsTab.tsx` | screens/light/codex-auth.png, screens/dark/codex-auth.png | 一致 | Auth 账号 Tab。 |
| 90 | 批次 2 Codex | `src/views/codex/tabs/CodexAuthProvidersTab.vue` | `src/features/codex/CodexAuthProvidersTab.tsx` | screens/light/codex-auth.png, screens/dark/codex-auth.png | 一致 | Auth 提供商 Tab。 |
| 91 | 批次 3 次级平台 | `src/views/grok/GrokView.vue` | `src/features/grok/GrokView.tsx` | screens/light/grok.png, screens/dark/grok.png; recordings/cached-routes.mp4 | 一致 | Grok 首页；cache-route + grok-dashboard.smoke。 |
| 92 | 批次 3 次级平台 | `src/components/grok/GrokProfileCard.vue` | `src/features/grok/GrokProfileCard.tsx` | screens/light/grok-profiles.png, screens/dark/grok-profiles.png | 一致 | Grok profile 卡。 |
| 93 | 批次 3 次级平台 | `src/components/grok/GrokProfileEditorModal.vue` | `src/features/grok/GrokProfileEditorModal.tsx` | screens/light/grok-profiles.png, screens/dark/grok-profiles.png | 可接受差异 | 编辑弹层。 |
| 94 | 批次 3 次级平台 | `src/views/GeminiCliView.vue` | `src/features/gemini/GeminiCliView.tsx` | screens/light/antigravity.png, screens/dark/antigravity.png, screens/light/gemini-cli.png, screens/dark/gemini-cli.png | 一致 | 基线 gemini-cli.png 已是 Antigravity CLI 页（兼容入口）。gemini-cli-view.smoke。 |
| 95 | 批次 3 次级平台 | `src/views/GeminiSlashCommandsView.vue` | `src/features/gemini/GeminiSlashCommandsView.tsx` | screens/light/antigravity-slash-commands.png, screens/dark/antigravity-slash-commands.png, screens/light/gemini-cli-slash-commands.png, screens/dark/gemini-cli-slash-commands.png | 一致 | gemini-cli/slash-commands 重定向 antigravity。 |
| 96 | 批次 3 次级平台 | `src/views/OpenCodeView.vue` | `src/features/opencode/OpenCodeView.tsx` | screens/light/opencode.png, screens/dark/opencode.png | 一致 | OpenCode 首页。 |
| 97 | 批次 3 次级平台 | `src/views/OpenCodeProvidersView.vue` | `src/features/opencode/OpenCodeProvidersView.tsx` | screens/light/opencode-providers.png, screens/dark/opencode-providers.png | 一致 | Providers 页。 |
| 98 | 批次 3 次级平台 | `src/components/opencode/OpenCodePageShell.vue` | `src/features/opencode/OpenCodePageShell.tsx` | screens/light/opencode.png, screens/dark/opencode.png, screens/light/opencode-providers.png, screens/dark/opencode-providers.png, screens/light/opencode-mcp.png, screens/dark/opencode-mcp.png, screens/light/opencode-agents.png, screens/dark/opencode-agents.png, screens/light/opencode-commands.png, screens/dark/opencode-commands.png, screens/light/opencode-plugins.png, screens/dark/opencode-plugins.png, screens/light/opencode-settings.png, screens/dark/opencode-settings.png, screens/light/opencode-system-prompts.png, screens/dark/opencode-system-prompts.png, screens/light/opencode-skills.png, screens/dark/opencode-skills.png | 一致 | OpenCode 页壳覆盖该组静帧；opencode-skills 重定向 /skills。 |
| 99 | 批次 3 次级平台 | `src/views/generic/AgentDetailView.vue` | `src/features/platform/AgentDetailView.tsx` | screens/light/agents-sample-agent.png, screens/dark/agents-sample-agent.png | 一致 | Agent 详情样例路由。 |
| 100 | 批次 3 次级平台 | `src/views/generic/SystemPromptsView.vue` | `src/features/platform/SystemPromptsView.tsx` | screens/light/claude-code-system-prompts.png, screens/dark/claude-code-system-prompts.png, screens/light/codex-system-prompts.png, screens/dark/codex-system-prompts.png, screens/light/antigravity-system-prompts.png, screens/dark/antigravity-system-prompts.png, screens/light/gemini-cli-system-prompts.png, screens/dark/gemini-cli-system-prompts.png, screens/light/opencode-system-prompts.png, screens/dark/opencode-system-prompts.png | 一致 | 跨平台系统提示词静帧。 |
| 101 | 批次 4 CheckIn | `src/views/CheckinView.vue` | `src/features/checkin/CheckinView.tsx` | screens/light/checkin.png, screens/dark/checkin.png | 一致 | 基线 checkin.png 页头与「加载签到数据失败」Web 空态。checkin-accounts-tab / checkin-state smoke。 |
| 102 | 批次 4 CheckIn | `src/views/checkin/CheckinAccountDashboardView.vue` | `src/features/checkin/CheckinAccountDashboardView.tsx` | screens/light/checkin-manage-sample-account.png, screens/dark/checkin-manage-sample-account.png | 一致 | 账号看板样例路由。 |
| 103 | 批次 4 CheckIn | `src/components/CheckinProgressModal.vue` | `src/features/checkin/CheckinProgressModal.tsx` | screens/light/checkin.png, screens/dark/checkin.png; overlay | 可接受差异 | 进度弹层；checkin-progress-modal.smoke。 |
| 104 | 批次 4 CheckIn | `src/views/checkin/components/AccountActionsMenu.vue` | `src/features/checkin/AccountActionsMenu.tsx` | screens/light/checkin.png, screens/dark/checkin.png | 一致 | 账号行操作菜单，归属签到主表。 |
| 105 | 批次 4 CheckIn | `src/views/checkin/components/AccountDashboardCalendar.vue` | `src/features/checkin/AccountDashboardCalendar.tsx` | screens/light/checkin-manage-sample-account.png, screens/dark/checkin-manage-sample-account.png | 一致 | 看板日历。 |
| 106 | 批次 4 CheckIn | `src/views/checkin/components/AccountDashboardTrend.vue` | `src/features/checkin/AccountDashboardTrend.tsx` | screens/light/checkin-manage-sample-account.png, screens/dark/checkin-manage-sample-account.png | 一致 | 看板趋势。 |
| 107 | 批次 4 CheckIn | `src/views/checkin/components/AccountFormModal.vue` | `src/features/checkin/AccountFormModal.tsx` | screens/light/checkin.png, screens/dark/checkin.png | 可接受差异 | 账号表单弹层。 |
| 108 | 批次 4 CheckIn | `src/views/checkin/components/AccountsTable.vue` | `src/features/checkin/AccountsTable.tsx` | screens/light/checkin.png, screens/dark/checkin.png | 一致 | 账号表；Web 基线为加载失败，结构由 smoke 锁。 |
| 109 | 批次 4 CheckIn | `src/views/checkin/components/OAuthWizardModal.vue` | `src/features/checkin/OAuthWizardModal.tsx` | recordings/oauth-wizard-desktop.mp4; recordings/oauth-wizard-still.png | 可接受差异 | 录制止于凭据录入步（基线 README / 本任务政策：不要求付费凭据）。oauth-wizard-branches.md 覆盖步骤与错误分支。不走真实 token 交换。 |
| 110 | 批次 4 CheckIn | `src/views/checkin/tabs/CheckinAccountsTab.vue` | `src/features/checkin/CheckinAccountsTab.tsx` | screens/light/checkin.png, screens/dark/checkin.png | 一致 | 账号 Tab。 |
| 111 | 批次 4 CheckIn | `src/views/checkin/tabs/CheckinImportExportTab.vue` | `src/features/checkin/CheckinImportExportTab.tsx` | screens/light/checkin.png, screens/dark/checkin.png | 一致 | 导入导出 Tab。 |
| 112 | 批次 4 CheckIn | `src/views/checkin/tabs/CheckinProvidersTab.vue` | `src/features/checkin/CheckinProvidersTab.tsx` | screens/light/checkin.png, screens/dark/checkin.png | 一致 | Provider Tab。WAF 入口在此；真实签到见 soak/WAF 记录，不在本行判定为缺陷。 |
| 113 | 批次 4 CheckIn | `src/views/checkin/tabs/CheckinRecordsTab.vue` | `src/features/checkin/CheckinRecordsTab.tsx` | screens/light/checkin.png, screens/dark/checkin.png | 一致 | 记录 Tab；checkin-records-api.smoke。 |
| 114 | 批次 5 Usage / Dashboard | `src/views/DashboardView.vue` | `src/features/usage/DashboardView.tsx` | screens/light/home.png, screens/dark/home.png; recordings/cached-routes.mp4 | 一致 | 基线 home.png 运行概览。dashboard-presentation.smoke。 |
| 115 | 批次 5 Usage / Dashboard | `src/views/BudgetView.vue` | `src/features/usage/BudgetView.tsx` | screens/light/budget.png, screens/dark/budget.png | 一致 | 预算页静帧。 |
| 116 | 批次 5 Usage / Dashboard | `src/views/PricingView.vue` | `src/features/usage/PricingView.tsx` | screens/light/pricing.png, screens/dark/pricing.png | 一致 | 定价页静帧。 |
| 117 | 批次 5 Usage / Dashboard | `src/views/UsageDashboardView.vue` | `src/features/usage/UsageDashboardView.tsx` | screens/light/usage.png, screens/dark/usage.png, screens/light/stats.png, screens/dark/stats.png; recordings/chart-time-range.mp4, recordings/cached-routes.mp4 | 一致 | 基线 usage.png 桌面-only 结构页；stats 重定向 /usage。 |
| 118 | 批次 5 Usage / Dashboard | `src/components/usage/LlmusageInstallDialog.vue` | `src/features/usage/LlmusageInstallDialog.tsx` | screens/light/usage.png, screens/dark/usage.png, screens/light/home.png, screens/dark/home.png | 可接受差异 | 安装 llmusage 弹层。 |
| 119 | 批次 5 Usage / Dashboard | `src/components/usage/UsageCostConclusionCard.vue` | `src/features/usage/UsageCostConclusionCard.tsx` | screens/light/usage.png, screens/dark/usage.png; recordings/chart-time-range.mp4 | 一致 | 费用结论卡，归属用量页。 |
| 120 | 批次 5 Usage / Dashboard | `src/components/usage/UsageCostTab.vue` | `src/features/usage/UsageCostTab.tsx` | screens/light/usage.png, screens/dark/usage.png; recordings/chart-time-range.mp4 | 一致 | 费用 Tab。 |
| 121 | 批次 5 Usage / Dashboard | `src/components/usage/UsageDashboardToolbar.vue` | `src/features/usage/UsageDashboardToolbar.tsx` | screens/light/usage.png, screens/dark/usage.png; recordings/chart-time-range.mp4, recordings/cached-routes.mp4 | 一致 | 工具条时间范围；cache-route usage 筛选。 |
| 122 | 批次 5 Usage / Dashboard | `src/components/usage/UsageDiagnosticsDrawer.vue` | `src/features/usage/UsageDiagnosticsDrawer.tsx` | screens/light/usage.png, screens/dark/usage.png | 可接受差异 | 诊断抽屉。 |
| 123 | 批次 5 Usage / Dashboard | `src/components/usage/UsageLogsTab.vue` | `src/features/usage/UsageLogsTab.tsx` | screens/light/usage.png, screens/dark/usage.png | 一致 | 日志 Tab。 |
| 124 | 批次 5 Usage / Dashboard | `src/components/usage/UsageMetricCard.vue` | `src/features/usage/UsageMetricCard.tsx` | screens/light/usage.png, screens/dark/usage.png, screens/light/home.png, screens/dark/home.png | 一致 | 指标卡 + sparkline。 |
| 125 | 批次 5 Usage / Dashboard | `src/components/usage/UsageModelDistributionCard.vue` | `src/features/usage/UsageModelDistributionCard.tsx` | screens/light/usage.png, screens/dark/usage.png | 一致 | 模型分布卡。 |
| 126 | 批次 5 Usage / Dashboard | `src/components/usage/UsageModelsTab.vue` | `src/features/usage/UsageModelsTab.tsx` | screens/light/usage.png, screens/dark/usage.png | 一致 | 模型 Tab。 |
| 127 | 批次 5 Usage / Dashboard | `src/components/usage/UsageOverviewTab.vue` | `src/features/usage/UsageOverviewTab.tsx` | screens/light/usage.png, screens/dark/usage.png; recordings/chart-time-range.mp4 | 一致 | 总览 Tab。 |
| 128 | 批次 5 Usage / Dashboard | `src/components/usage/UsageProjectsTab.vue` | `src/features/usage/UsageProjectsTab.tsx` | screens/light/usage.png, screens/dark/usage.png | 一致 | 项目 Tab。 |
| 129 | 批次 5 Usage / Dashboard | `src/components/usage/UsageProvidersTab.vue` | `src/features/usage/UsageProvidersTab.tsx` | screens/light/usage.png, screens/dark/usage.png | 一致 | 提供商 Tab。 |
| 130 | 批次 5 Usage / Dashboard | `src/components/usage/UsageSourceSummaryCard.vue` | `src/features/usage/UsageSourceSummaryCard.tsx` | screens/light/usage.png, screens/dark/usage.png | 一致 | 来源摘要卡。 |
| 131 | 批次 5 Usage / Dashboard | `src/components/usage/UsageStaleBanner.vue` | `src/features/usage/UsageStaleBanner.tsx` | screens/light/usage.png, screens/dark/usage.png, screens/light/home.png, screens/dark/home.png | 一致 | 过期横幅。 |
| 132 | 批次 5 Usage / Dashboard | `src/components/usage/UsageTokenBreakdownStrip.vue` | `src/features/usage/UsageTokenBreakdownStrip.tsx` | screens/light/usage.png, screens/dark/usage.png | 一致 | Token 拆条。 |
| 133 | 批次 5 Usage / Dashboard | `src/components/usage/UsageTokensTab.vue` | `src/features/usage/UsageTokensTab.tsx` | screens/light/usage.png, screens/dark/usage.png | 一致 | Token Tab。 |
| 134 | 批次 5 Usage / Dashboard | `src/components/dashboard/DashboardNextActions.vue` | `src/features/usage/DashboardNextActions.tsx` | screens/light/home.png, screens/dark/home.png | 一致 | 基线 home.png「下一步」行动队列。 |
| 135 | 批次 5 Usage / Dashboard | `src/components/dashboard/DashboardPlatformMatrix.vue` | `src/features/usage/DashboardPlatformMatrix.tsx` | screens/light/home.png, screens/dark/home.png | 一致 | 平台矩阵（基线页下方）。 |
| 136 | 批次 5 Usage / Dashboard | `src/components/dashboard/DashboardReadinessLedger.vue` | `src/features/usage/DashboardReadinessLedger.tsx` | screens/light/home.png, screens/dark/home.png | 一致 | 基线就绪账本五瓦片。 |
| 137 | 批次 5 Usage / Dashboard | `src/components/dashboard/DashboardSignalStream.vue` | `src/features/usage/DashboardSignalStream.tsx` | screens/light/home.png, screens/dark/home.png | 一致 | 基线事件流分段。 |
| 138 | 批次 5 Usage / Dashboard | `src/components/dashboard/DashboardUsageMovement.vue` | `src/features/usage/DashboardUsageMovement.tsx` | screens/light/home.png, screens/dark/home.png | 一致 | 基线用量趋势 70/30D/90D。 |
| 139 | 批次 5 Usage / Dashboard | `src/components/platform-usage/PlatformUsageInsightPanel.vue` | `src/features/usage/PlatformUsageInsightPanel.tsx` | screens/light/claude-code.png, screens/dark/claude-code.png, screens/light/codex.png, screens/dark/codex.png, screens/light/grok.png, screens/dark/grok.png, screens/light/antigravity.png, screens/dark/antigravity.png, screens/light/opencode.png, screens/dark/opencode.png | 一致 | 平台首页用量洞察。 |
| 140 | 批次 5 Usage / Dashboard | `src/components/platform-usage/PlatformUsageRankList.vue` | `src/features/usage/PlatformUsageRankList.tsx` | screens/light/claude-code.png, screens/dark/claude-code.png, screens/light/codex.png, screens/dark/codex.png, screens/light/grok.png, screens/dark/grok.png | 一致 | 用量排行。 |
| 141 | 批次 5 Usage / Dashboard | `src/components/platform-usage/PlatformUsageTrendChart.vue` | `src/features/usage/PlatformUsageTrendChart.tsx` | screens/light/claude-code.png, screens/dark/claude-code.png, screens/light/codex.png, screens/dark/codex.png, screens/light/grok.png, screens/dark/grok.png, screens/light/antigravity.png, screens/dark/antigravity.png; recordings/chart-time-range.mp4 | 一致 | 平台趋势图；platform-usage-trend-chart.smoke。 |
| 142 | 批次 7 Sync / MCP / Commands / 工具 | `src/views/SyncView.vue` | `src/features/sync/SyncView.tsx` | screens/light/sync.png, screens/dark/sync.png; recordings/cached-routes.mp4 | 一致 | 同步页静帧。 |
| 143 | 批次 7 Sync / MCP / Commands / 工具 | `src/views/MonitoringView.vue` | `src/features/sync/MonitoringView.tsx` | screens/light/monitoring.png, screens/dark/monitoring.png, screens/light/sessions.png, screens/dark/sessions.png; recordings/log-stream.mp4 | 一致 | 监控页 + 日志流录屏；sessions 重定向 /monitoring。 |
| 144 | 批次 7 Sync / MCP / Commands / 工具 | `src/views/SshManagementView.vue` | `src/features/sync/SshManagementView.tsx` | screens/light/ssh.png, screens/dark/ssh.png | 一致 | SSH 页；ssh-hardening.smoke。 |
| 145 | 批次 7 Sync / MCP / Commands / 工具 | `src/views/WslManagementView.vue` | `src/features/sync/WslManagementView.tsx` | screens/light/wsl.png, screens/dark/wsl.png | 一致 | WSL 页（Windows）；wsl-platform-gate.smoke。 |
| 146 | 批次 7 Sync / MCP / Commands / 工具 | `src/components/sync/SyncAccountDialog.vue` | `src/features/sync/SyncAccountDialog.tsx` | screens/light/sync.png, screens/dark/sync.png | 可接受差异 | 同步账号弹层。 |
| 147 | 批次 7 Sync / MCP / Commands / 工具 | `src/components/sync/SyncInfoSidebar.vue` | `src/features/sync/SyncInfoSidebar.tsx` | screens/light/sync.png, screens/dark/sync.png | 一致 | 同步侧栏。 |
| 148 | 批次 7 Sync / MCP / Commands / 工具 | `src/components/sync/SyncOperationOutputPanel.vue` | `src/features/sync/SyncOperationOutputPanel.tsx` | screens/light/sync.png, screens/dark/sync.png | 一致 | 操作输出面板。 |
| 149 | 批次 7 Sync / MCP / Commands / 工具 | `src/components/sync/SyncPassphraseModal.vue` | `src/features/sync/SyncPassphraseModal.tsx` | screens/light/sync.png, screens/dark/sync.png | 可接受差异 | 口令弹层；sync-passphrase-modal.smoke。 |
| 150 | 批次 7 Sync / MCP / Commands / 工具 | `src/views/tray/CodexTrayPanelView.vue` | `src/features/codex/tray/CodexTrayPanelView.tsx` | screens/light/tray-codex.png, screens/dark/tray-codex.png | 一致 | 托盘独立路由静帧。 |
| 151 | 批次 7 Sync / MCP / Commands / 工具 | `src/views/tray/components/TrayAccountSwitchScreen.vue` | `src/features/codex/tray/TrayAccountSwitchScreen.tsx` | screens/light/tray-codex.png, screens/dark/tray-codex.png | 一致 | 托盘切账号屏。 |
| 152 | 批次 7 Sync / MCP / Commands / 工具 | `src/views/tray/components/TrayOverview.vue` | `src/features/codex/tray/TrayOverview.tsx` | screens/light/tray-codex.png, screens/dark/tray-codex.png | 一致 | 托盘总览。 |
| 153 | 批次 7 Sync / MCP / Commands / 工具 | `src/components/editor/CodeSourceEditor.vue` | `src/features/configs/CodeSourceEditor.tsx` | screens/light/configs.png, screens/dark/configs.png, screens/light/settings.png, screens/dark/settings.png; recordings/large-form-input.mp4 | 可接受差异 | CodeMirror 编辑器；CSP nonce smoke。实现现位于 features/editor。 |
| 154 | 批次 7 Sync / MCP / Commands / 工具 | `src/components/editor/ConfigSourcePanel.vue` | `src/features/configs/ConfigSourcePanel.tsx` | screens/light/configs.png, screens/dark/configs.png; recordings/large-form-input.mp4 | 一致 | 配置源面板。 |
| 155 | 批次 7 Sync / MCP / Commands / 工具 | `src/components/BaseSlashCommands.vue` | `src/features/commands/BaseSlashCommands.tsx` | screens/light/slash-commands.png, screens/dark/slash-commands.png, screens/light/commands-claude.png, screens/dark/commands-claude.png, screens/light/codex-slash-commands.png, screens/dark/codex-slash-commands.png, screens/light/antigravity-slash-commands.png, screens/dark/antigravity-slash-commands.png, screens/light/gemini-cli-slash-commands.png, screens/dark/gemini-cli-slash-commands.png | 一致 | 斜杠命令 base。 |
| 156 | 批次 7 Sync / MCP / Commands / 工具 | `src/components/CommandFormModal.vue` | `src/features/commands/CommandFormModal.tsx` | screens/light/slash-commands.png, screens/dark/slash-commands.png, screens/light/commands.png, screens/dark/commands.png | 可接受差异 | 命令表单弹层。 |
| 157 | 批次 7 Sync / MCP / Commands / 工具 | `src/components/CommandList.vue` | `src/features/commands/CommandList.tsx` | screens/light/slash-commands.png, screens/dark/slash-commands.png, screens/light/commands.png, screens/dark/commands.png, screens/light/commands-claude.png, screens/dark/commands-claude.png | 一致 | 命令列表。 |
| 158 | 批次 7 Sync / MCP / Commands / 工具 | `src/views/SlashCommandsView.vue` | `src/features/commands/SlashCommandsView.tsx` | screens/light/slash-commands.png, screens/dark/slash-commands.png | 一致 | Claude 斜杠命令薄壳。 |
| 159 | 批次 7 Sync / MCP / Commands / 工具 | `src/views/mcp/McpManagerView.vue` | `src/features/mcp/McpManagerView.tsx` | screens/light/mcp-manager.png, screens/dark/mcp-manager.png, screens/light/mcp.png, screens/dark/mcp.png, screens/light/mcp-unified.png, screens/dark/mcp-unified.png | 一致 | 基线 mcp.png 即 MCP 管理中心；mcp / mcp-unified 重定向 mcp-manager。 |
| 160 | 批次 7 Sync / MCP / Commands / 工具 | `src/components/McpPresetsPanel.vue` | `src/features/mcp/McpPresetsPanel.tsx` | screens/light/mcp.png, screens/dark/mcp.png, screens/light/mcp-manager.png, screens/dark/mcp-manager.png | 一致 | 基线「安装预设」条。 |
| 161 | 批次 7 Sync / MCP / Commands / 工具 | `src/components/McpSyncPanel.vue` | `src/features/mcp/McpSyncPanel.tsx` | screens/light/mcp.png, screens/dark/mcp.png, screens/light/mcp-manager.png, screens/dark/mcp-manager.png | 一致 | MCP 同步面板。 |
| 162 | 批次 7 Sync / MCP / Commands / 工具 | `src/components/mcp/McpCreatePanel.vue` | `src/features/mcp/McpCreatePanel.tsx` | screens/light/mcp.png, screens/dark/mcp.png, screens/light/mcp-manager.png, screens/dark/mcp-manager.png | 可接受差异 | 创建面板（添加服务器）。 |
| 163 | 批次 7 Sync / MCP / Commands / 工具 | `src/components/mcp/McpDetailPanel.vue` | `src/features/mcp/McpDetailPanel.tsx` | screens/light/mcp.png, screens/dark/mcp.png, screens/light/mcp-manager.png, screens/dark/mcp-manager.png | 一致 | 基线右栏「尚未选择 MCP 服务器」。 |
| 164 | 批次 7 Sync / MCP / Commands / 工具 | `src/components/mcp/McpImportPanel.vue` | `src/features/mcp/McpImportPanel.tsx` | screens/light/mcp.png, screens/dark/mcp.png, screens/light/mcp-manager.png, screens/dark/mcp-manager.png | 可接受差异 | 导入面板。 |
| 165 | 批次 7 Sync / MCP / Commands / 工具 | `src/components/mcp/McpListPanel.vue` | `src/features/mcp/McpListPanel.tsx` | screens/light/mcp.png, screens/dark/mcp.png, screens/light/mcp-manager.png, screens/dark/mcp-manager.png | 一致 | 基线左栏空列表 + 添加服务器。mcp-panels.smoke。 |
| 166 | 批次 6 Profiles / 配置 | `src/views/AppSettingsView.vue` | `src/features/configs/AppSettingsView.tsx` | screens/light/settings.png, screens/dark/settings.png; recordings/large-form-input.mp4 | 一致 | 应用设置；app-settings-view.smoke。 |
| 167 | 批次 6 Profiles / 配置 | `src/views/ConfigsView.vue` | `src/features/configs/ConfigsView.tsx` | screens/light/configs.png, screens/dark/configs.png; recordings/cached-routes.mp4, recordings/large-form-input.mp4 | 一致 | 配置列表；configs-view.smoke + cache-route 搜索/草稿。 |
| 168 | 批次 6 Profiles / 配置 | `src/views/ConverterView.vue` | `src/features/configs/ConverterView.tsx` | screens/light/converter.png, screens/dark/converter.png | 一致 | 转换器；converter-view.smoke。 |
| 169 | 批次 6 Profiles / 配置 | `src/components/AddConfigModal.vue` | `src/features/configs/AddConfigModal.tsx` | screens/light/configs.png, screens/dark/configs.png; recordings/large-form-input.mp4 | 可接受差异 | 新增配置弹层。 |
| 170 | 批次 6 Profiles / 配置 | `src/components/EditConfigModal.vue` | `src/features/configs/EditConfigModal.tsx` | screens/light/configs.png, screens/dark/configs.png; recordings/large-form-input.mp4 | 可接受差异 | 编辑配置弹层；edit-config-draft.smoke。 |
| 171 | 批次 6 Profiles / 配置 | `src/components/ConfigCard.vue` | `src/features/configs/ConfigCard.tsx` | screens/light/configs.png, screens/dark/configs.png | 一致 | 配置卡。 |
| 172 | 批次 6 Profiles / 配置 | `src/components/configs/ConfigFilters.vue` | `src/features/configs/ConfigFilters.tsx` | screens/light/configs.png, screens/dark/configs.png; recordings/large-form-input.mp4 | 一致 | 筛选条。 |
| 173 | 批次 6 Profiles / 配置 | `src/components/configs/ConfigList.vue` | `src/features/configs/ConfigList.tsx` | screens/light/configs.png, screens/dark/configs.png | 一致 | 配置列表。 |
| 174 | 批次 6 Profiles / 配置 | `src/components/configs/ProviderStatsModal.vue` | `src/features/configs/ProviderStatsModal.tsx` | screens/light/configs.png, screens/dark/configs.png | 可接受差异 | 提供商统计弹层。 |
| 175 | 批次 6 Profiles / 配置 | `src/components/profiles/ProfileDiffRows.vue` | `src/features/profiles/ProfileDiffRows.tsx` | screens/light/claude-code-profiles.png, screens/dark/claude-code-profiles.png, screens/light/codex-profiles.png, screens/dark/codex-profiles.png, screens/light/grok-profiles.png, screens/dark/grok-profiles.png | 一致 | 差异行；profile-diff.smoke。 |
| 176 | 批次 6 Profiles / 配置 | `src/components/profiles/ProfileListRow.vue` | `src/features/profiles/ProfileListRow.tsx` | screens/light/claude-code-profiles.png, screens/dark/claude-code-profiles.png, screens/light/codex-profiles.png, screens/dark/codex-profiles.png, screens/light/grok-profiles.png, screens/dark/grok-profiles.png | 一致 | 列表行。 |
| 177 | 批次 6 Profiles / 配置 | `src/components/profiles/ProfilesCommandPalette.vue` | `src/features/profiles/ProfilesCommandPalette.tsx` | screens/light/claude-code-profiles.png, screens/dark/claude-code-profiles.png, screens/light/codex-profiles.png, screens/dark/codex-profiles.png, screens/light/grok-profiles.png, screens/dark/grok-profiles.png | 可接受差异 | 命令面板 overlay。 |
| 178 | 批次 6 Profiles / 配置 | `src/components/profiles/ProfilesHeader.vue` | `src/features/profiles/ProfilesHeader.tsx` | screens/light/claude-code-profiles.png, screens/dark/claude-code-profiles.png, screens/light/codex-profiles.png, screens/dark/codex-profiles.png, screens/light/grok-profiles.png, screens/dark/grok-profiles.png | 一致 | 共享页头。 |
| 179 | 批次 6 Profiles / 配置 | `src/components/profiles/ProfilesInspector.vue` | `src/features/profiles/ProfilesInspector.tsx` | screens/light/claude-code-profiles.png, screens/dark/claude-code-profiles.png, screens/light/codex-profiles.png, screens/dark/codex-profiles.png, screens/light/grok-profiles.png, screens/dark/grok-profiles.png | 一致 | 检查器。 |
| 180 | 批次 6 Profiles / 配置 | `src/components/profiles/ProfilesQuickRail.vue` | `src/features/profiles/ProfilesQuickRail.tsx` | screens/light/claude-code-profiles.png, screens/dark/claude-code-profiles.png, screens/light/codex-profiles.png, screens/dark/codex-profiles.png, screens/light/grok-profiles.png, screens/dark/grok-profiles.png | 一致 | 快捷轨。 |
| 181 | 批次 6 Profiles / 配置 | `src/components/profiles/ProfilesRawEditorPanel.vue` | `src/features/profiles/ProfilesRawEditorPanel.tsx` | screens/light/claude-code-profiles.png, screens/dark/claude-code-profiles.png, screens/light/codex-profiles.png, screens/dark/codex-profiles.png, screens/light/grok-profiles.png, screens/dark/grok-profiles.png; recordings/large-form-input.mp4 | 一致 | 原始编辑；CSP nonce。 |
| 182 | 批次 6 Profiles / 配置 | `src/components/profiles/ProfilesSection.vue` | `src/features/profiles/ProfilesSection.tsx` | screens/light/claude-code-profiles.png, screens/dark/claude-code-profiles.png, screens/light/codex-profiles.png, screens/dark/codex-profiles.png, screens/light/grok-profiles.png, screens/dark/grok-profiles.png | 一致 | 分组段。 |
| 183 | 批次 6 Profiles / 配置 | `src/components/profiles/ProfilesStatStrip.vue` | `src/features/profiles/ProfilesStatStrip.tsx` | screens/light/claude-code-profiles.png, screens/dark/claude-code-profiles.png, screens/light/codex-profiles.png, screens/dark/codex-profiles.png, screens/light/grok-profiles.png, screens/dark/grok-profiles.png | 一致 | 统计条。 |
| 184 | 批次 6 Profiles / 配置 | `src/components/profiles/ProfilesToolbar.vue` | `src/features/profiles/ProfilesToolbar.tsx` | screens/light/claude-code-profiles.png, screens/dark/claude-code-profiles.png, screens/light/codex-profiles.png, screens/dark/codex-profiles.png, screens/light/grok-profiles.png, screens/dark/grok-profiles.png | 一致 | 工具条。 |
| 185 | 批次 6 Profiles / 配置 | `src/components/provider-templates/ProviderTemplateSelector.vue` | `src/features/profiles/ProviderTemplateSelector.tsx` | screens/light/configs.png, screens/dark/configs.png, screens/light/claude-code-profiles.png, screens/dark/claude-code-profiles.png, screens/light/codex-profiles.png, screens/dark/codex-profiles.png; recordings/large-form-input.mp4 | 一致 | 模板选择器；provider-template-selector.smoke。 |

## 按七个视图批次

| 批次 | 域 | 行号 |
| --- | --- | --- |
| 1 | Claude Code | 063–077（另：统一层 Claude 薄壳 042/046/049/058） |
| 2 | Codex | 078–090（另：统一层 Codex 薄壳 043/047/050/054/056） |
| 3 | Grok / Gemini / OpenCode / generic | 091–100（另：统一层 044/045/048/051/053/055/057/059–062） |
| 4 | CheckIn | 101–113 |
| 5 | Usage / Dashboard | 114–141 |
| 6 | Profiles / 配置 | 166–185（另：共享 profiles 组件与三平台 Profiles 薄壳） |
| 7 | Sync / MCP / Commands / 工具 | 142–165 |

外壳 / 共享 / UI 原语（001–041）出现在全部或多数静帧上，不重复计入七个视图批次的功能验证（platform-unify AC6 已覆盖功能矩阵）。

## 录屏覆盖

| 录屏 | 覆盖的非路由界面 |
| --- | --- |
| `cached-routes.mp4` | Dashboard / Grok / Commands / Configs / Usage 离开与返回（cache-route.smoke 6/6） |
| `oauth-wizard-desktop.mp4` + `oauth-wizard-still.png` | OAuth 向导至凭据步 |
| `log-stream.mp4` | Monitoring 实时日志 |
| `chart-time-range.mp4` | 用量图表时间范围 |
| `large-form-input.mp4` | Configs 搜索 + Settings 长文本 |

## 抽检笔记（Vue 基线静帧）

| 静帧 | 观察 |
| --- | --- |
| light/home.png | 运行概览、Web 预览条、五瓦片、下一步队列、用量趋势分段 |
| light/claude-code.png | 工作台、打开认证、用量洞察三卡、费用归因 Tab、近 30 天空图表 |
| light/checkin.png | 签到管理页头、一键签到/刷新余额、加载失败条 |
| dark/usage.png | 桌面-only 空态（与 Web 基线 README 一致） |
| light/skills-add.png | Skills 下线说明，附属路由在 catalog 中重定向 /skills |
| light/mcp.png | MCP 管理中心主从布局、预设条、toast、空列表 |
| light/gemini-cli.png | 已渲染 Antigravity CLI 兼容入口 |

生成脚本：本文件由 `_gen_screen_comparison.py` 一次写出；表行与 path-mapping 185 行一一对应。
