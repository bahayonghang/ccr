# 组件表面迁移：半透明面/文本透明度/inset 高光按新契约收敛

## Goal

把组件层绕过表面契约的三类写法（模板 `bg-bg-*/N` alpha 表面、直写半透明文本、inset 白高光）迁移到子任务 A 建立的不透明契约上；壳层与按钮文字色接入新语义。

## Background

- 父任务：`07-28-ccr-ui-visual-redesign`；依赖 `07-28-color-system-rebuild` 完成（新令牌值与契约已就绪后才能批量迁移）。
- 证据：`../07-28-ccr-ui-visual-redesign/research/diagnosis.md` R2/R3/R5/R6/R7/R8。
- 子任务 A 已在令牌层修复 stage 文本/表面（134 处 stage 文本消费、26 处 stage-soft 消费无需逐文件改）；本任务处理**直写写法**与**壳层/按钮**。

## Requirements

### R1. 模板 alpha 表面迁移（247 处 / 43 文件）

- `bg-bg-{base,elevated,surface,overlay}/N` → 不透明语义类。默认映射：`/N ≤ 70` 用低一档实心令牌，`> 70` 用同档实心令牌；真正的遮罩/ scrim 场景改用专用 scrim 令牌（若无则在 tokens.css 增补一个，不走 bg 令牌 alpha）。
- 重灾区优先：`CodexAgentSourcesPanel.vue`(19)、`MonitoringView.vue`(18)、`OpenCodeAgentsView.vue`(16)、`OpenCodeSettingsView.vue`(15)、`OpenCodeMcpView.vue`(13)、`CodexAgentsView.vue`(12)、`OpenCodeProvidersView.vue`(12)、`OpenCodePluginsView.vue`(11)、`AgentsView.vue`(11)。
- hover/disabled 态的 alpha 属于交互反馈，保留但须引用令牌 alpha（如 `rgb(var(--color-bg-overlay-rgb) / 60%)`），不直接写 `bg-bg-*/N`。

### R2. 直写半透明文本（11 处）

- `Titlebar.vue:387,397,430,438`、`McpCreatePanel.vue:416`、`ListSearchHeader.vue:87`、`DashboardNextActions.vue:234,238`、`SyncView.vue:718,956`（实为 border）→ 改实心令牌（`--color-text-secondary/muted/ghost`）。
- 存量 `text-text-*/N` Tailwind 透明度类（5 处 / 4 文件）同改。

### R3. inset 白高光收敛（~30 处）

- 暗色下卡片顶部 `inset 0 1px 0 rgb(255 255 255 / N%)` 全部移除或改 `var(--inner-glow)`（A 已将其降至 ≤3%）。亮色下 > 46% 的高光同步收敛进令牌。
- 优先：`UsageMetricCard.vue:87,97`、`ConfigFilters.vue:174`、`UsageDashboardToolbar.vue:200,262`、`CodexProfileEditorModal.vue:796-834`、`ClaudeCodeProfilesView.vue:1387,1446,1596`、`TrayOverview.vue:307`、`CodexTrayPanelView.vue:157`、`PlatformUsageInsightPanel.vue:247`、`UsageDashboardView.vue:369`。

### R4. 壳层与模态接入新契约

- `MainLayout.vue:443-467` `.sidebar-glass` / `.topbar-glass`：确认渲染结果为不透明（A 改令牌后验证计算样式，必要时把类名从 glass 工具类切到语义 surface 类）。
- `BaseModal.vue:231`：`bg-white/80 dark:bg-bg-elevated/90 backdrop-blur-xl backdrop-saturate-150` → `--surface-modal-*` 契约。
- usage 仪表盘 `glass-panel`（~20 处）：改不透明 card 语义；`UsageDashboardView.vue:364-369` 的渐变覆盖 + 6% 白 inset 移除。
- `ClaudeProfileRow.vue` 的 `backdrop-blur-xl` 评估移除（滚动列表内禁玻璃）。

### R5. 按钮与 text-white 收敛

- `utilities.css:290,313,324` `.btn-primary/.btn-danger/.btn-success`：`color: white` → `var(--color-accent-primary-contrast)`（danger/success 新增对应 contrast 令牌或在 A 增补）。
- 62 处 `text-white` / 27 文件逐一审计：accent 实心底上的改 contrast 令牌；非 accent 底上的（bug）改 `--color-text-*`；mask 用途保留。
- `chart-colors.css` 五个图表色按新 palette 校准（暗色可读性）。

### R6. 不引入回归

- 不新增任何 `backdrop-filter`；不新增 `255 255 255` 无守卫引用；不改动业务逻辑与组件 props。

## Acceptance Criteria

- [ ] AC1: `rg "bg-bg-(base|elevated|surface|overlay)/" ccr-ui/src` 仅剩 scrim/交互态白名单（在 implement.md 登记），其余为零。
- [ ] AC2: `rg "rgb\(var\(--color-text-[a-z]+-rgb\) /" ccr-ui/src` 无结果；`rg "inset 0 1px 0 rgb\(255 255 255" ccr-ui/src` 无结果（亮色 >46% 的也收敛）。
- [ ] AC3: `rg "color: white" ccr-ui/src/styles` 无结果；`text-white` 审计完毕，非 accent 底上的全部改令牌。
- [ ] AC4: 侧栏/顶栏计算样式 `backdrop-filter: none` 且背景不透明（Playwright 或手动 DevTools 证据）；`rg "backdrop-blur" ccr-ui/src` 仅剩 modal/浮层白名单。
- [ ] AC5: `bun run type-check && bun run lint` 通过；全量 smoke 通过；`just frontend-check-quick` 通过。
- [ ] AC6: 视觉核验：dark+neutral 下 Monitoring / CodexAgents / Usage / Profiles 路由截图，卡片边界清晰、无奶白边、无雾感（证据存父任务 research/）。

## Out Of Scope

- 令牌值本身调整（子任务 A）；`/settings` 重设计（子任务 C）。
- 组件结构/props 重构、业务逻辑变更。
- `CodexSettingsView.vue:899-900` 已知的 phantom token 写法（`var(--platform-codex-rgb, 245 158 11)`）——顺手修复允许，但不扩展为全仓 phantom 审计。
