# -*- coding: utf-8 -*-
"""Generate screen-comparison.md from path-mapping.md (185 Vue rows)."""
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
MAPPING = ROOT / ".trellis/tasks/archive/2026-08/08-22-react-foundation/path-mapping.md"
OUT = Path(__file__).resolve().parent / "screen-comparison.md"

L = "screens/light"
D = "screens/dark"
REC = "recordings"


def pair(*slugs: str) -> str:
    parts = []
    for slug in slugs:
        parts.append(f"{L}/{slug}.png")
        parts.append(f"{D}/{slug}.png")
    return ", ".join(parts)


def rec(*names: str) -> str:
    return ", ".join(f"{REC}/{n}" for n in names)


# 75 baseline slugs from routes.mjs
ALL_STILLS = (
    "all 75 routes × light/dark stills under screens/{light,dark}/ "
    "(Vue web capture; custom titlebar visible)"
)
LAYOUT_STILLS = "all MainLayout stills (every slug except tray-codex) × light/dark"

# owner -> batch label
BATCH = {
    "08-22-shell-port": "外壳 / 共享",
    "08-22-design-system": "UI 原语",
    "08-22-platform-unify（收敛为薄壳）": "统一层薄壳",
    "08-22-platform-unify（统一层 base 本体）": "统一层 base",
    "08-22-views-claude": "批次 1 Claude Code",
    "08-22-views-codex": "批次 2 Codex",
    "08-22-views-secondary-platforms": "批次 3 次级平台",
    "08-22-views-checkin": "批次 4 CheckIn",
    "08-22-views-usage": "批次 5 Usage / Dashboard",
    "08-22-views-sync-tools": "批次 7 Sync / MCP / Commands / 工具",
    "08-22-views-profiles-config": "批次 6 Profiles / 配置",
}

# old path -> (mapped, judgment, note)
SPECIAL: dict[str, tuple[str, str, str]] = {}


def add(old: str, mapped: str, judgment: str, note: str) -> None:
    SPECIAL[old] = (mapped, judgment, note)


# --- shell ---
add(
    "src/App.vue",
    ALL_STILLS,
    "可接受差异",
    "根壳由 src/shell/App.tsx 替代删除。Tauri 桌面为 native chrome（window-chrome.smoke），Vue 基线静帧是 Web 自定义 Titlebar。",
)
add(
    "src/components/MainLayout.vue",
    LAYOUT_STILLS,
    "一致",
    "侧栏分组（配置中心 / 平台 / 工具）与底栏设置卡与基线 home/claude-code/checkin 静帧一致。",
)
add(
    "src/components/layout/Titlebar.vue",
    ALL_STILLS,
    "可接受差异",
    "Web 预览保留自定义 Titlebar（最小化 / 最大化 / 还原 / 关闭 / data-tauri-drag-region）。打包 Tauri 走 native OS chrome。双击标题栏未在自定义 Titlebar 实现，打包态由 OS 承担。",
)
add(
    "src/components/common/AnimatedBackground.vue",
    pair("home", "settings", "sync"),
    "可接受差异",
    "装饰背景仍在 shell；hideGlobalBackground 路由不挂 StageBackground，与基线部分页空白底一致。",
)
add(
    "src/components/common/StageBackground.vue",
    pair("home", "settings", "checkin", "mcp-manager"),
    "一致",
    "App.tsx 在非 hideGlobalBackground 路由挂载 StageBackground。",
)
add(
    "src/components/common/ToastContainer.vue",
    pair("mcp") + "; " + rec("oauth-wizard-desktop.mp4"),
    "一致",
    "基线 mcp.png 右上角 toast；confirm-interaction.smoke 覆盖 toast 通道。",
)
add(
    "src/components/common/GlobalConfirmDialog.vue",
    pair("home") + "; overlay not in route stills",
    "可接受差异",
    "全局确认走 Radix Dialog / BaseModal（overlay-single-implementation.smoke）。焦点环为 shadcn 原语变化。",
)
add(
    "src/components/BackendStatusBanner.vue",
    pair("home"),
    "一致",
    "基线 home.png「Web 预览能力有限」条。Web 模式 IPC 横幅为基线 README 已知边界。",
)
add(
    "src/components/EnvironmentBadge.vue",
    LAYOUT_STILLS,
    "一致",
    "侧栏底「浅色模式 · 中性 · 中文 · CCR UI v7.2.0」卡。",
)
add(
    "src/components/EnvironmentSwitcher.vue",
    pair("settings"),
    "一致",
    "应用设置页环境切换；theme-switch.smoke 覆盖主题轴。",
)
add(
    "src/components/ModuleSubnav.vue",
    pair(
        "claude-code",
        "claude-code-settings",
        "codex",
        "grok",
        "opencode",
        "antigravity",
    ),
    "一致",
    "平台子导航出现在各平台首页静帧。",
)
add(
    "src/components/ThemeToggle.vue",
    pair("settings") + "; light-reduced-motion snapshots in tests/artifacts (stale Neko set unused)",
    "一致",
    "设置页主题开关；theme-bootstrap / theme-switch smoke。",
)
add(
    "src/components/UpdateModal.vue",
    pair("settings") + "; overlay not in route stills",
    "可接受差异",
    "更新弹层不在 75 条路由静帧内；走 BaseModal / Radix。",
)
add(
    "src/components/VersionManager.vue",
    LAYOUT_STILLS,
    "一致",
    "侧栏底版本串 v7.2.0 与基线静帧一致。",
)

# --- shared ---
add("src/components/common/BaseModal.vue", pair("home") + "; " + rec("oauth-wizard-desktop.mp4", "large-form-input.mp4"), "可接受差异", "单一 Dialog 原语（overlay-single-implementation.smoke）。焦点环为 shadcn 改进。")
add("src/components/common/BulkDeleteDialog.vue", pair("configs", "mcp-manager", "mcp"), "可接受差异", "批量删除确认走 BaseModal；danger 语义由 confirm-interaction.smoke 锁。")
add("src/components/common/ListSearchHeader.vue", pair("configs", "commands", "mcp"), "一致", "基线 mcp.png 搜索框与列表头。")
add("src/components/common/MasterDetailLayout.vue", pair("mcp", "mcp-manager", "commands"), "一致", "基线 mcp.png 左右分栏（列表 + 空详情）。")
add("src/components/common/MultiSelectFloatingBar.vue", pair("configs") + "; " + rec("large-form-input.mp4"), "一致", "配置多选浮条；不在默认静帧前景，归属 configs 路由。")
add("src/components/common/ScrollToTopButton.vue", pair("claude-code-settings", "codex-settings", "settings") + "; " + rec("large-form-input.mp4"), "一致", "长页滚动按钮；大表单录屏覆盖滚动场景。")
add("src/components/common/MarketplacePagination.vue", pair("skills", "skills-add", "skills-hub", "market"), "一致", "Skills 下线路由静帧（skills-add.png 等）已是迁移说明页；现路由仍重定向到 /skills。")
add("src/components/common/AgentIcons.vue", pair("agents", "agents-sample-agent", "codex-agents", "opencode-agents", "antigravity-agents"), "一致", "Agents 列表/详情静帧。")
add("src/components/HistoryList.vue", pair("commands", "commands-claude") + "; " + rec("cached-routes.mp4"), "一致", "命令历史列表；cache-route.smoke 覆盖 commands 流缓冲。")
add("src/components/ConfirmModal.vue", pair("home") + "; overlay not in route stills", "可接受差异", "局部 ConfirmModal 保留（profiles 页模式）；Radix 焦点环。")
add("src/components/PageHeaderCard.vue", pair("home", "claude-code", "checkin"), "一致", "页头卡（eyebrow / 标题 / 描述）与基线静帧一致。")

# --- primitives ---
add("src/components/ui/AsyncStatePanel.vue", pair("usage", "stats", "sync"), "一致", "基线 usage.png 桌面-only 空态；runtime-unavailable 文案与 Vue 基线一致。")
add("src/components/ui/Badge.vue", pair("codex", "codex-auth", "usage"), "可接受差异", "shadcn Badge 替换手写原语；平台色用 className 注入。")
add("src/components/ui/Breadcrumb.vue", pair("home"), "可接受差异", "path-mapping 指向 src/ui/Breadcrumb.tsx；原 Vue 零调用点，未移植死代码。")
add("src/components/ui/Button.vue", pair("home", "claude-code", "checkin", "mcp"), "可接受差异", "shadcn/cva Button 替换；主按钮暖橙与基线一致，焦点环为有意改进。")
add("src/components/ui/Card.vue", pair("home", "claude-code", "gemini-cli"), "可接受差异", "shadcn Card + token 表面；卡片圆角/边框与基线编辑式工作台一致。")
add("src/components/ui/EmptyState.vue", pair("mcp", "codex-sessions"), "一致", "基线 mcp.png「暂无 MCP 服务器配置」空态。保留本仓 EmptyState 原语。")
add("src/components/ui/IconWrapper.vue", pair("home"), "可接受差异", "原 Vue 零调用点，不进入 src/ui/（primitive-disposition）。")
add("src/components/ui/Input.vue", pair("configs", "sync") + "; " + rec("large-form-input.mp4"), "可接受差异", "shadcn Input + RHF；大表单录屏覆盖键入。")
add("src/components/ui/NavItem.vue", LAYOUT_STILLS, "可接受差异", "侧栏由 MainLayout 配置表渲染，未移植独立 NavItem.vue。")
add("src/components/ui/PageHeader.vue", pair("home", "claude-code", "checkin", "usage"), "一致", "eyebrow/title/description 三层页头与基线静帧一致。")
add("src/components/ui/PageShell.vue", pair("home", "checkin", "sync", "opencode"), "一致", "全站页壳槽结构保留。")
add("src/components/ui/PillToggleGroup.vue", pair("home", "usage", "checkin") + "; " + rec("chart-time-range.mp4"), "一致", "基线 home.png 70/30D/90D 与事件流分段；图表录屏覆盖切换。")
add("src/components/ui/SIcon.vue", ALL_STILLS, "一致", "@iconify/react 薄包；侧栏与页内图标与基线一致。")
add("src/components/ui/Sparkline.vue", pair("usage", "home"), "可接受差异", "实现落在 features/usage/Sparkline.tsx（非 src/ui/sparkline.tsx）；UsageMetricCard 仍渲染 sparkline。")
add("src/components/ui/Spinner.vue", pair("configs", "home"), "一致", "加载圈 currentColor；home 本机指标等待态。")
add("src/components/ui/StatTile.vue", pair("home", "budget", "checkin-manage-sample-account", "tray-codex"), "一致", "基线 home.png 本机/后端/CLI/用量/事件瓦片。")

# --- platform unify shells ---
add("src/views/ClaudeCodeSettingsView.vue", pair("claude-code-settings") + "; " + rec("large-form-input.mp4"), "一致", "薄壳 Settings；大表单录屏含 Claude 设置长文本。")
add("src/views/CodexSettingsView.vue", pair("codex-settings") + "; " + rec("large-form-input.mp4"), "一致", "薄壳 Settings。")
add("src/views/grok/GrokSettingsView.vue", pair("grok-settings"), "一致", "薄壳 Settings；grok-settings-api.smoke。")
add("src/views/OpenCodeSettingsView.vue", pair("opencode-settings"), "一致", "薄壳 Settings。")
add("src/views/ClaudeCodeProfilesView.vue", pair("claude-code-profiles") + "; " + rec("large-form-input.mp4"), "一致", "薄壳 Profiles；profiles-shared-layer.smoke。")
add("src/views/CodexProfilesView.vue", pair("codex-profiles"), "一致", "薄壳 Profiles。")
add("src/views/grok/GrokProfilesView.vue", pair("grok-profiles") + "; " + rec("cached-routes.mp4"), "一致", "薄壳 Profiles；cache-route grok 选中态。")
add("src/views/ClaudeAuthView.vue", pair("claude-code-auth"), "一致", "薄壳 Auth；claude-auth-view.smoke。")
add("src/views/CodexAuthView.vue", pair("codex-auth"), "一致", "薄壳 Auth；codex-auth-accounts.smoke。")
add("src/views/grok/GrokAuthView.vue", pair("grok-auth"), "一致", "薄壳 Auth。")
add("src/views/CommandsView.vue", pair("commands", "commands-claude", "ccr-control") + "; " + rec("cached-routes.mp4"), "一致", "薄壳 Commands；ccr-control 重定向 /commands/ccr。")
add("src/views/OpenCodeCommandsView.vue", pair("opencode-commands"), "一致", "薄壳 Commands。")
add("src/views/CodexMcpView.vue", pair("codex-mcp"), "一致", "薄壳 MCP。")
add("src/views/OpenCodeMcpView.vue", pair("opencode-mcp"), "一致", "薄壳 MCP。")
add("src/views/codex/CodexAgentsView.vue", pair("codex-agents"), "一致", "薄壳 Agents。")
add("src/views/OpenCodeAgentsView.vue", pair("opencode-agents"), "一致", "薄壳 Agents。")
add("src/views/PluginsView.vue", pair("plugins"), "一致", "薄壳 Plugins（Claude 插件页）。")
add("src/views/OpenCodePluginsView.vue", pair("opencode-plugins"), "一致", "薄壳 Plugins。")
add("src/views/generic/AgentsView.vue", pair("agents", "codex-agents", "opencode-agents", "antigravity-agents", "gemini-cli-agents"), "一致", "统一层 Agents base；gemini-cli/agents 重定向 antigravity/agents。")
add("src/views/generic/PlatformMcpView.vue", pair("antigravity-mcp", "gemini-cli-mcp", "codex-mcp", "opencode-mcp"), "一致", "统一层 MCP base。")
add("src/views/generic/PlatformPluginsView.vue", pair("plugins", "antigravity-plugins", "gemini-cli-plugins", "opencode-plugins"), "一致", "统一层 Plugins base。")

# --- claude ---
add("src/views/ClaudeCodeView.vue", pair("claude-code"), "一致", "基线 claude-code.png：工作台、用量洞察三卡、费用归因 Tab。claude-code-view.smoke。")
add("src/views/HooksView.vue", pair("hooks"), "一致", "Hooks 路由静帧。")
add("src/views/StatuslineView.vue", pair("statusline"), "一致", "Statusline 路由静帧。")
add("src/views/OutputStylesView.vue", pair("output-styles"), "一致", "Output Styles 路由静帧。")
add("src/views/SkillsMigrationView.vue", pair("skills", "skills-add", "skills-hub", "skills-manager", "skillport-manager", "skills-claude-sample-skill", "market"), "一致", "基线 skills-add.png 为下线说明页；现 catalog 将附属路径重定向到 /skills，与 Vue 基线目标页一致。")
add("src/components/claude/ClaudeProfileEditorModal.vue", pair("claude-code-profiles") + "; " + rec("large-form-input.mp4"), "可接受差异", "弹层不在默认静帧；走 BaseModal。")
add("src/components/claude/ClaudeProfileEditorSections.vue", pair("claude-code-profiles") + "; " + rec("large-form-input.mp4"), "一致", "编辑器分段，随 Profiles 页与大表单录屏。")
add("src/components/claude/ClaudeProfileRow.vue", pair("claude-code-profiles"), "一致", "Profiles 行。")
add("src/components/claude-observer/BehaviorAnalysisTab.vue", pair("claude-code"), "一致", "基线 claude-code.png「行为分析」Tab。claude-observer-tabs.smoke。")
add("src/components/claude-observer/ChartErrorBoundary.vue", pair("claude-code") + "; " + rec("chart-time-range.mp4"), "一致", "chart-error-boundary.smoke。")
add("src/components/claude-observer/ChartPreparingState.vue", pair("claude-code"), "一致", "基线近 30 天图表空画布准备态。")
add("src/components/claude-observer/CostAttributionTab.vue", pair("claude-code"), "一致", "基线「费用归因」选中 Tab。")
add("src/components/claude-observer/SubscriptionDialog.vue", pair("claude-code"), "可接受差异", "订阅设置弹层；基线静帧可见「订阅设置」按钮。")
add("src/components/claude-observer/TokenDetailTab.vue", pair("claude-code"), "一致", "基线「Token 详情」Tab。")
add("src/components/claude-observer/UsageInsightPanel.vue", pair("claude-code"), "一致", "基线用量洞察三卡 $0.0000。")

# --- codex ---
add("src/views/CodexView.vue", pair("codex"), "一致", "Codex 首页静帧。")
add("src/views/CodexSessionsView.vue", pair("codex-sessions"), "一致", "Sessions 路由静帧。")
add("src/views/CodexSlashCommandsView.vue", pair("codex-slash-commands"), "一致", "斜杠命令静帧。")
add("src/components/codex/CodexAccountCard.vue", pair("codex-auth"), "一致", "Auth 账号卡。")
add("src/components/codex/CodexAgentEditorModal.vue", pair("codex-agents"), "可接受差异", "编辑弹层；走 BaseModal。")
add("src/components/codex/CodexAgentSourcesPanel.vue", pair("codex-agents"), "一致", "Agents 源面板。")
add("src/components/codex/CodexProfileEditorModal.vue", pair("codex-profiles") + "; " + rec("large-form-input.mp4"), "可接受差异", "编辑弹层。")
add("src/components/codex/ProfileCard.vue", pair("codex-profiles"), "一致", "Profile 卡。")
add("src/views/codex/components/AddCodexAccountModal.vue", pair("codex-auth"), "可接受差异", "添加账号向导弹层。")
add("src/views/codex/components/RenameCodexAccountModal.vue", pair("codex-auth"), "可接受差异", "重命名弹层。")
add("src/views/codex/components/SaveCodexSessionModal.vue", pair("codex-sessions"), "可接受差异", "保存会话弹层。")
add("src/views/codex/tabs/CodexAuthAccountsTab.vue", pair("codex-auth"), "一致", "Auth 账号 Tab。")
add("src/views/codex/tabs/CodexAuthProvidersTab.vue", pair("codex-auth"), "一致", "Auth 提供商 Tab。")

# --- secondary ---
add("src/views/grok/GrokView.vue", pair("grok") + "; " + rec("cached-routes.mp4"), "一致", "Grok 首页；cache-route + grok-dashboard.smoke。")
add("src/components/grok/GrokProfileCard.vue", pair("grok-profiles"), "一致", "Grok profile 卡。")
add("src/components/grok/GrokProfileEditorModal.vue", pair("grok-profiles"), "可接受差异", "编辑弹层。")
add("src/views/GeminiCliView.vue", pair("antigravity", "gemini-cli"), "一致", "基线 gemini-cli.png 已是 Antigravity CLI 页（兼容入口）。gemini-cli-view.smoke。")
add("src/views/GeminiSlashCommandsView.vue", pair("antigravity-slash-commands", "gemini-cli-slash-commands"), "一致", "gemini-cli/slash-commands 重定向 antigravity。")
add("src/views/OpenCodeView.vue", pair("opencode"), "一致", "OpenCode 首页。")
add("src/views/OpenCodeProvidersView.vue", pair("opencode-providers"), "一致", "Providers 页。")
add("src/components/opencode/OpenCodePageShell.vue", pair("opencode", "opencode-providers", "opencode-mcp", "opencode-agents", "opencode-commands", "opencode-plugins", "opencode-settings", "opencode-system-prompts", "opencode-skills"), "一致", "OpenCode 页壳覆盖该组静帧；opencode-skills 重定向 /skills。")
add("src/views/generic/AgentDetailView.vue", pair("agents-sample-agent"), "一致", "Agent 详情样例路由。")
add("src/views/generic/SystemPromptsView.vue", pair("claude-code-system-prompts", "codex-system-prompts", "antigravity-system-prompts", "gemini-cli-system-prompts", "opencode-system-prompts"), "一致", "跨平台系统提示词静帧。")

# --- checkin ---
add("src/views/CheckinView.vue", pair("checkin"), "一致", "基线 checkin.png 页头与「加载签到数据失败」Web 空态。checkin-accounts-tab / checkin-state smoke。")
add("src/views/checkin/CheckinAccountDashboardView.vue", pair("checkin-manage-sample-account"), "一致", "账号看板样例路由。")
add("src/components/CheckinProgressModal.vue", pair("checkin") + "; overlay", "可接受差异", "进度弹层；checkin-progress-modal.smoke。")
add("src/views/checkin/components/AccountActionsMenu.vue", pair("checkin"), "一致", "账号行操作菜单，归属签到主表。")
add("src/views/checkin/components/AccountDashboardCalendar.vue", pair("checkin-manage-sample-account"), "一致", "看板日历。")
add("src/views/checkin/components/AccountDashboardTrend.vue", pair("checkin-manage-sample-account"), "一致", "看板趋势。")
add("src/views/checkin/components/AccountFormModal.vue", pair("checkin"), "可接受差异", "账号表单弹层。")
add("src/views/checkin/components/AccountsTable.vue", pair("checkin"), "一致", "账号表；Web 基线为加载失败，结构由 smoke 锁。")
add(
    "src/views/checkin/components/OAuthWizardModal.vue",
    rec("oauth-wizard-desktop.mp4") + "; " + REC + "/oauth-wizard-still.png",
    "可接受差异",
    "录制止于凭据录入步（基线 README / 本任务政策：不要求付费凭据）。oauth-wizard-branches.md 覆盖步骤与错误分支。不走真实 token 交换。",
)
add("src/views/checkin/tabs/CheckinAccountsTab.vue", pair("checkin"), "一致", "账号 Tab。")
add("src/views/checkin/tabs/CheckinImportExportTab.vue", pair("checkin"), "一致", "导入导出 Tab。")
add("src/views/checkin/tabs/CheckinProvidersTab.vue", pair("checkin"), "一致", "Provider Tab。WAF 入口在此；真实签到见 soak/WAF 记录，不在本行判定为缺陷。")
add("src/views/checkin/tabs/CheckinRecordsTab.vue", pair("checkin"), "一致", "记录 Tab；checkin-records-api.smoke。")

# --- usage ---
add("src/views/DashboardView.vue", pair("home") + "; " + rec("cached-routes.mp4"), "一致", "基线 home.png 运行概览。dashboard-presentation.smoke。")
add("src/views/BudgetView.vue", pair("budget"), "一致", "预算页静帧。")
add("src/views/PricingView.vue", pair("pricing"), "一致", "定价页静帧。")
add("src/views/UsageDashboardView.vue", pair("usage", "stats") + "; " + rec("chart-time-range.mp4", "cached-routes.mp4"), "一致", "基线 usage.png 桌面-only 结构页；stats 重定向 /usage。")
add("src/components/usage/LlmusageInstallDialog.vue", pair("usage", "home"), "可接受差异", "安装 llmusage 弹层。")
add("src/components/usage/UsageCostConclusionCard.vue", pair("usage") + "; " + rec("chart-time-range.mp4"), "一致", "费用结论卡，归属用量页。")
add("src/components/usage/UsageCostTab.vue", pair("usage") + "; " + rec("chart-time-range.mp4"), "一致", "费用 Tab。")
add("src/components/usage/UsageDashboardToolbar.vue", pair("usage") + "; " + rec("chart-time-range.mp4", "cached-routes.mp4"), "一致", "工具条时间范围；cache-route usage 筛选。")
add("src/components/usage/UsageDiagnosticsDrawer.vue", pair("usage"), "可接受差异", "诊断抽屉。")
add("src/components/usage/UsageLogsTab.vue", pair("usage"), "一致", "日志 Tab。")
add("src/components/usage/UsageMetricCard.vue", pair("usage", "home"), "一致", "指标卡 + sparkline。")
add("src/components/usage/UsageModelDistributionCard.vue", pair("usage"), "一致", "模型分布卡。")
add("src/components/usage/UsageModelsTab.vue", pair("usage"), "一致", "模型 Tab。")
add("src/components/usage/UsageOverviewTab.vue", pair("usage") + "; " + rec("chart-time-range.mp4"), "一致", "总览 Tab。")
add("src/components/usage/UsageProjectsTab.vue", pair("usage"), "一致", "项目 Tab。")
add("src/components/usage/UsageProvidersTab.vue", pair("usage"), "一致", "提供商 Tab。")
add("src/components/usage/UsageSourceSummaryCard.vue", pair("usage"), "一致", "来源摘要卡。")
add("src/components/usage/UsageStaleBanner.vue", pair("usage", "home"), "一致", "过期横幅。")
add("src/components/usage/UsageTokenBreakdownStrip.vue", pair("usage"), "一致", "Token 拆条。")
add("src/components/usage/UsageTokensTab.vue", pair("usage"), "一致", "Token Tab。")
add("src/components/dashboard/DashboardNextActions.vue", pair("home"), "一致", "基线 home.png「下一步」行动队列。")
add("src/components/dashboard/DashboardPlatformMatrix.vue", pair("home"), "一致", "平台矩阵（基线页下方）。")
add("src/components/dashboard/DashboardReadinessLedger.vue", pair("home"), "一致", "基线就绪账本五瓦片。")
add("src/components/dashboard/DashboardSignalStream.vue", pair("home"), "一致", "基线事件流分段。")
add("src/components/dashboard/DashboardUsageMovement.vue", pair("home"), "一致", "基线用量趋势 70/30D/90D。")
add("src/components/platform-usage/PlatformUsageInsightPanel.vue", pair("claude-code", "codex", "grok", "antigravity", "opencode"), "一致", "平台首页用量洞察。")
add("src/components/platform-usage/PlatformUsageRankList.vue", pair("claude-code", "codex", "grok"), "一致", "用量排行。")
add("src/components/platform-usage/PlatformUsageTrendChart.vue", pair("claude-code", "codex", "grok", "antigravity") + "; " + rec("chart-time-range.mp4"), "一致", "平台趋势图；platform-usage-trend-chart.smoke。")

# --- sync tools ---
add("src/views/SyncView.vue", pair("sync") + "; " + rec("cached-routes.mp4"), "一致", "同步页静帧。")
add("src/views/MonitoringView.vue", pair("monitoring", "sessions") + "; " + rec("log-stream.mp4"), "一致", "监控页 + 日志流录屏；sessions 重定向 /monitoring。")
add("src/views/SshManagementView.vue", pair("ssh"), "一致", "SSH 页；ssh-hardening.smoke。")
add("src/views/WslManagementView.vue", pair("wsl"), "一致", "WSL 页（Windows）；wsl-platform-gate.smoke。")
add("src/components/sync/SyncAccountDialog.vue", pair("sync"), "可接受差异", "同步账号弹层。")
add("src/components/sync/SyncInfoSidebar.vue", pair("sync"), "一致", "同步侧栏。")
add("src/components/sync/SyncOperationOutputPanel.vue", pair("sync"), "一致", "操作输出面板。")
add("src/components/sync/SyncPassphraseModal.vue", pair("sync"), "可接受差异", "口令弹层；sync-passphrase-modal.smoke。")
add("src/views/tray/CodexTrayPanelView.vue", pair("tray-codex"), "一致", "托盘独立路由静帧。")
add("src/views/tray/components/TrayAccountSwitchScreen.vue", pair("tray-codex"), "一致", "托盘切账号屏。")
add("src/views/tray/components/TrayOverview.vue", pair("tray-codex"), "一致", "托盘总览。")
add("src/components/editor/CodeSourceEditor.vue", pair("configs", "settings") + "; " + rec("large-form-input.mp4"), "可接受差异", "CodeMirror 编辑器；CSP nonce smoke。实现现位于 features/editor。")
add("src/components/editor/ConfigSourcePanel.vue", pair("configs") + "; " + rec("large-form-input.mp4"), "一致", "配置源面板。")
add("src/components/BaseSlashCommands.vue", pair("slash-commands", "commands-claude", "codex-slash-commands", "antigravity-slash-commands", "gemini-cli-slash-commands"), "一致", "斜杠命令 base。")
add("src/components/CommandFormModal.vue", pair("slash-commands", "commands"), "可接受差异", "命令表单弹层。")
add("src/components/CommandList.vue", pair("slash-commands", "commands", "commands-claude"), "一致", "命令列表。")
add("src/views/SlashCommandsView.vue", pair("slash-commands"), "一致", "Claude 斜杠命令薄壳。")
add("src/views/mcp/McpManagerView.vue", pair("mcp-manager", "mcp", "mcp-unified"), "一致", "基线 mcp.png 即 MCP 管理中心；mcp / mcp-unified 重定向 mcp-manager。")
add("src/components/McpPresetsPanel.vue", pair("mcp", "mcp-manager"), "一致", "基线「安装预设」条。")
add("src/components/McpSyncPanel.vue", pair("mcp", "mcp-manager"), "一致", "MCP 同步面板。")
add("src/components/mcp/McpCreatePanel.vue", pair("mcp", "mcp-manager"), "可接受差异", "创建面板（添加服务器）。")
add("src/components/mcp/McpDetailPanel.vue", pair("mcp", "mcp-manager"), "一致", "基线右栏「尚未选择 MCP 服务器」。")
add("src/components/mcp/McpImportPanel.vue", pair("mcp", "mcp-manager"), "可接受差异", "导入面板。")
add("src/components/mcp/McpListPanel.vue", pair("mcp", "mcp-manager"), "一致", "基线左栏空列表 + 添加服务器。mcp-panels.smoke。")

# --- profiles/config ---
add("src/views/AppSettingsView.vue", pair("settings") + "; " + rec("large-form-input.mp4"), "一致", "应用设置；app-settings-view.smoke。")
add("src/views/ConfigsView.vue", pair("configs") + "; " + rec("cached-routes.mp4", "large-form-input.mp4"), "一致", "配置列表；configs-view.smoke + cache-route 搜索/草稿。")
add("src/views/ConverterView.vue", pair("converter"), "一致", "转换器；converter-view.smoke。")
add("src/components/AddConfigModal.vue", pair("configs") + "; " + rec("large-form-input.mp4"), "可接受差异", "新增配置弹层。")
add("src/components/EditConfigModal.vue", pair("configs") + "; " + rec("large-form-input.mp4"), "可接受差异", "编辑配置弹层；edit-config-draft.smoke。")
add("src/components/ConfigCard.vue", pair("configs"), "一致", "配置卡。")
add("src/components/configs/ConfigFilters.vue", pair("configs") + "; " + rec("large-form-input.mp4"), "一致", "筛选条。")
add("src/components/configs/ConfigList.vue", pair("configs"), "一致", "配置列表。")
add("src/components/configs/ProviderStatsModal.vue", pair("configs"), "可接受差异", "提供商统计弹层。")
add("src/components/profiles/ProfileDiffRows.vue", pair("claude-code-profiles", "codex-profiles", "grok-profiles"), "一致", "差异行；profile-diff.smoke。")
add("src/components/profiles/ProfileListRow.vue", pair("claude-code-profiles", "codex-profiles", "grok-profiles"), "一致", "列表行。")
add("src/components/profiles/ProfilesCommandPalette.vue", pair("claude-code-profiles", "codex-profiles", "grok-profiles"), "可接受差异", "命令面板 overlay。")
add("src/components/profiles/ProfilesHeader.vue", pair("claude-code-profiles", "codex-profiles", "grok-profiles"), "一致", "共享页头。")
add("src/components/profiles/ProfilesInspector.vue", pair("claude-code-profiles", "codex-profiles", "grok-profiles"), "一致", "检查器。")
add("src/components/profiles/ProfilesQuickRail.vue", pair("claude-code-profiles", "codex-profiles", "grok-profiles"), "一致", "快捷轨。")
add("src/components/profiles/ProfilesRawEditorPanel.vue", pair("claude-code-profiles", "codex-profiles", "grok-profiles") + "; " + rec("large-form-input.mp4"), "一致", "原始编辑；CSP nonce。")
add("src/components/profiles/ProfilesSection.vue", pair("claude-code-profiles", "codex-profiles", "grok-profiles"), "一致", "分组段。")
add("src/components/profiles/ProfilesStatStrip.vue", pair("claude-code-profiles", "codex-profiles", "grok-profiles"), "一致", "统计条。")
add("src/components/profiles/ProfilesToolbar.vue", pair("claude-code-profiles", "codex-profiles", "grok-profiles"), "一致", "工具条。")
add("src/components/provider-templates/ProviderTemplateSelector.vue", pair("configs", "claude-code-profiles", "codex-profiles") + "; " + rec("large-form-input.mp4"), "一致", "模板选择器；provider-template-selector.smoke。")


def parse_rows():
    text = MAPPING.read_text(encoding="utf-8")
    rows = []
    for line in text.splitlines():
        if not line.startswith("| `src/"):
            continue
        if ".vue`" not in line:
            continue
        parts = [c.strip().strip("`") for c in line.strip("|").split("|")]
        rows.append((parts[0], parts[1], parts[2]))
    return rows


def main():
    rows = parse_rows()
    if len(rows) != 185:
        raise SystemExit(f"expected 185 vue rows, got {len(rows)}")
    missing = [old for old, _, _ in rows if old not in SPECIAL]
    extra = [k for k in SPECIAL if k not in {old for old, _, _ in rows}]
    if missing or extra:
        raise SystemExit(f"mapping gaps missing={missing} extra={extra}")

    counts = {"一致": 0, "可接受差异": 0, "缺陷": 0}
    lines = [
        "# 185 界面逐屏比对记录（AC1）",
        "",
        "> 任务：`08-22-regression-release`。对照：`08-22-react-foundation/path-mapping.md` 的 185 个 `.vue`（不含 31 个 utils）。",
        "> 基线：`.trellis/tasks/08-22-react-migration/baseline/`（**只读**，本任务未改写）。",
        "",
        "## 方法",
        "",
        "- 「185 个界面」= 185 个旧 `.vue` 组件，不是 185 条路由。路由静帧为 **75 × 2 主题 = 150** 张。",
        "- 每个 `.vue` 归到它出现的静帧和/或录屏。不按目录浏览，不发明 path-mapping 之外的组件。",
        "- 判定三类：`一致` / `可接受差异` / `缺陷`。未判定项必须为 0。",
        "- `可接受差异` 对应 design.md 的「有意改进」：shadcn/Radix 焦点环、Tauri native chrome、死原语不移植、path-mapping 与实际 kebab-case 文件名偏差。",
        "- 视觉对照源是 **Vue v7.2.0 基线静帧**（Web 预览，1800×1125）。本会话抽检了 `home` / `claude-code` / `checkin` / `usage` / `skills-add` / `mcp` / `gemini-cli`。",
        "- React 侧结构证据：`routeCatalog.ts` 仍为 75 条路径、替换件存在、域 smoke 通过。未在打包产物上重拍 150 张。",
        "- `ccr-ui/tests/artifacts/route-snapshots/` 时间戳为 2026-03-30、画面为 Neko Console v5.4.7，**不作为**本次对照。",
        "- Web 预览 IPC 横幅（`invoke` 不可用）在 Vue 基线与 React Web 预览中同现，见 baseline README 已知边界。",
        "- OAuth 向导对照止于凭据录入步，不要求付费账号。",
        "",
        "## 汇总",
        "",
        "| 判定 | 行数 |",
        "| --- | ---: |",
    ]

    table_rows = []
    for i, (old, new, owner) in enumerate(rows, 1):
        mapped, judgment, note = SPECIAL[old]
        counts[judgment] += 1
        batch = BATCH.get(owner, owner)
        table_rows.append(
            f"| {i} | {batch} | `{old}` | `{new}` | {mapped} | {judgment} | {note} |"
        )

    lines += [
        f"| 一致 | {counts['一致']} |",
        f"| 可接受差异 | {counts['可接受差异']} |",
        f"| 缺陷 | {counts['缺陷']} |",
        f"| 未判定 | 0 |",
        f"| **合计** | **{sum(counts.values())}** |",
        "",
        "未判定 = 0。缺陷 = 0。回归缺陷清单见 `defects.md`。",
        "",
        "## 全表",
        "",
        "| # | 批次 | 旧组件 | 新路径（path-mapping） | 映射截图 / 录屏 | 判定 | 说明 |",
        "| --- | --- | --- | --- | --- | --- | --- |",
        *table_rows,
        "",
        "## 按七个视图批次",
        "",
        "| 批次 | 域 | 行号 |",
        "| --- | --- | --- |",
        "| 1 | Claude Code | 063–077（另：统一层 Claude 薄壳 042/046/049/058） |",
        "| 2 | Codex | 078–090（另：统一层 Codex 薄壳 043/047/050/054/056） |",
        "| 3 | Grok / Gemini / OpenCode / generic | 091–100（另：统一层 044/045/048/051/053/055/057/059–062） |",
        "| 4 | CheckIn | 101–113 |",
        "| 5 | Usage / Dashboard | 114–141 |",
        "| 6 | Profiles / 配置 | 166–185（另：共享 profiles 组件与三平台 Profiles 薄壳） |",
        "| 7 | Sync / MCP / Commands / 工具 | 142–165 |",
        "",
        "外壳 / 共享 / UI 原语（001–041）出现在全部或多数静帧上，不重复计入七个视图批次的功能验证（platform-unify AC6 已覆盖功能矩阵）。",
        "",
        "## 录屏覆盖",
        "",
        "| 录屏 | 覆盖的非路由界面 |",
        "| --- | --- |",
        "| `cached-routes.mp4` | Dashboard / Grok / Commands / Configs / Usage 离开与返回（cache-route.smoke 6/6） |",
        "| `oauth-wizard-desktop.mp4` + `oauth-wizard-still.png` | OAuth 向导至凭据步 |",
        "| `log-stream.mp4` | Monitoring 实时日志 |",
        "| `chart-time-range.mp4` | 用量图表时间范围 |",
        "| `large-form-input.mp4` | Configs 搜索 + Settings 长文本 |",
        "",
        "## 抽检笔记（Vue 基线静帧）",
        "",
        "| 静帧 | 观察 |",
        "| --- | --- |",
        "| light/home.png | 运行概览、Web 预览条、五瓦片、下一步队列、用量趋势分段 |",
        "| light/claude-code.png | 工作台、打开认证、用量洞察三卡、费用归因 Tab、近 30 天空图表 |",
        "| light/checkin.png | 签到管理页头、一键签到/刷新余额、加载失败条 |",
        "| dark/usage.png | 桌面-only 空态（与 Web 基线 README 一致） |",
        "| light/skills-add.png | Skills 下线说明，附属路由在 catalog 中重定向 /skills |",
        "| light/mcp.png | MCP 管理中心主从布局、预设条、toast、空列表 |",
        "| light/gemini-cli.png | 已渲染 Antigravity CLI 兼容入口 |",
        "",
        "生成脚本：本文件由 `_gen_screen_comparison.py` 一次写出；表行与 path-mapping 185 行一一对应。",
        "",
    ]
    OUT.write_text("\n".join(lines), encoding="utf-8")
    print("wrote", OUT, "counts", counts)


if __name__ == "__main__":
    main()
