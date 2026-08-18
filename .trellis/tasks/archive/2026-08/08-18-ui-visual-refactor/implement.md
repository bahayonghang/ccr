# 实施计划

8 个 Wave：基座 → 原语 → 样板 → 分页族 → 清扫 → 总验。勾销表：`views-inventory.md`（51 个视图）。

## 交付形态与冻结

本任务保持单目录，不拆子任务。若后续要拆，按下面映射开 child，父任务只做集成验收：

| 可拆 child | 内容 |
|---|---|
| A 基座 | Wave 0 + DESIGN.md / theme-token-contracts 修订 |
| B 原语与 Shell | Wave 1–2 |
| C Dashboard | Wave 3 |
| D 平台族 | Wave 4 |
| E 功能族 + 清扫 | Wave 5–6 |
| F 总验 | Wave 7 |

**冻结**：功能分支（建议 `feature/ui-visual-refactor`）从 `dev` 拉出。Wave 0–6 的中间提交只留在该分支。未完成 Wave 7 之前，不合并 `main`，不打 release tag。`base_branch` 为 `dev`。

每个 Wave 出口：验证命令绿 + 该 Wave 的 `evidence/` 截图（web preview，预载三 key 并断言 dataset）。

## Wave 0 — Token 基座与主题收敛

**文件**：`src/styles/tokens.css`、`theme.css`、`base.css`、`backgrounds.css`、`home.css`、`utilities.css`、`animations.css`、`tailwind.config.ts`、`src/utils/themeBootstrap.ts`、`index.html` IIFE、`src/stores/shellPreferences.ts`、`ccr-ui/DESIGN.md`、`.trellis/spec/ccr-ui/frontend/theme-token-contracts.md`、`src/i18n/locales/zh-CN.ts`、`en-US.ts`、`bootMessages.ts`。

1. 删除 `--inner-glow` / `--glass-inner-glow` 及 tailwind surface 插件注入。收紧暗色阴影。`--glow-*` 只留 focus 环。mocha / flavor 作用域里的 `prefers-reduced-transparency` 重置迁到 `neutral` / `clay`。
2. 按 `prd.md` R1 改 FlavorMode / AccentMode 与两张迁移表。删除 `slate → sky`。IIFE 与 TS 行为字节等价。删除 `isCatppuccinFlavor` 与 latte/mocha 段。
3. 删 `premium.*`、`.text-gradient-*`、aurora/mesh、pulse-glow / tag-glow / text-glow。
4. `--tracking-normal` = 0。字重 token 改为 400 / 500 / 600 / 700。
5. 改写四个 theme smoke 的断言域（mocha 块改为不得存在；accent 列表只剩 `clay`；组合 4 组）。阈值常量不改。
6. 修订 DESIGN.md（`design.md` §0 对照表）。修订 `theme-token-contracts.md` 值域与迁移表。
7. 删除三份 i18n 里的 catppuccin / mauve / sage / sky 文案。跑 `bun run test:i18n`。

**验证**：

```
cd ccr-ui
bun run type-check
bun run lint
bun run test:i18n
bunx vitest run --config vitest.smoke.config.ts tests/theme-bootstrap.smoke.test.ts tests/app-settings.smoke.test.ts tests/apple-glass-surface-contract.smoke.test.ts tests/theme-contrast-contract.smoke.test.ts
```

**回滚点**：本 Wave 单独 revert 会留下已写回的 `neutral`/`clay`。不要把本 Wave 单独合入 `main`。

## Wave 1 — 共享原语

**文件**：`src/components/ui/*`、`PageHeaderCard.vue`、`opencode/OpenCodePageShell.vue`；新建 `PageHeader` / `StatTile` / `PillToggleGroup` / `PageShell`。

1. 先按 `design.md` §1 改 `OpenCodePageShell`（清掉现有字面 rgb），再抽 `PageShell`。
2. `Card` 去 gloss / 废弃 glow。`Button` 去 hover 光晕与全量 backdrop-filter；主按钮实心 accent、pill、44px。`Input` 圆角 8–10px。`Badge` 增加方角档。
3. 新建 PageHeader（含 `:lang` eyebrow）、StatTile、PillToggleGroup。
4. 删除 `ui/StatCard`、`ConfigItem.vue`。`Sparkline`、`ListSearchHeader`、`ConfigCard` **留用**，本 Wave 只改表面，ConfigCard 色值留到 Wave 6。

**验证**：`cd ccr-ui && bun run test` + web preview 原语走查（evidence：`primitives-zh-CN-dark-neutral.png`）。

## Wave 2 — Shell 与设置

**文件**：`MainLayout.vue`、`layout/Titlebar.vue`、`App.vue`、`AppSettingsView.vue`、`config/mainLayoutShell.ts`（如需）。

1. chrome 去内高光。导航激活态去竖条与渐变底，改 tonal。
2. 设置 dock 收敛。去掉 accent 选择入口。
3. `AppSettingsView` 主题区只留 light/dark + neutral/clay。色板预览白名单保留。

**前后结构**：设置页从「flavor 三卡 + accent 四卡」改为「flavor 双卡，无 accent 区」。

**验证**：theme smoke 四件套 + `test:i18n` + 设置页截图 zh/en × light/dark。

## Wave 3 — Dashboard

**文件**：`DashboardView.vue`、`dashboard/dashboardPresentation.ts`（不改数据契约）、`components/dashboard/*`。

1. PageHeader + readiness 状态槽。删除 vw 流体标题。
2. 行动队列首行 tonal。ReadinessLedger 拆嵌套卡。
3. UsageMovement / SignalStream 接 PillToggleGroup。图表柱实心低彩度。
4. 记录前后结构：hero display + 多卡嵌套 → 一行页头 + 行动列表 + 双栏洞察。

**验证**：`tests/dashboard-presentation.smoke.test.ts`；首屏截图标注实心 accent（≤2 处）。

## Wave 4 — 平台族（29）

按 `views-inventory.md` #3–#31 勾销。profiles 三页走 `profiles-page.css` + `components/profiles/*`。Grok 设置遵守 `grok-settings-contracts.md`。

每页：适用原语 + 清页内渐变 / glow / 硬编码色。不强制把 style 块拆到 300 行以下。

**验证**：`bun run type-check && bun run lint` + 五平台主页截图。Grok 设置跑该契约所列检查。

## Wave 5 — 功能族（20）

按 `views-inventory.md` #32–#51 勾销。

重点：

- `ConverterView`：70 处 `:style` + 旧别名 `--bg-primary` 等，迁 `--color-*`。
- `UsageDashboardView` + `components/usage/*`：遵守 `usage-chart-stability-contracts`；改完跑 `apexcharts-style-contract.smoke.test.ts`。
- `CheckinView` / `CheckinAccountDashboardView`：遵守 `checkin-ux-contracts`；同步改 `apple-glass` 的 `styleLockedPaths` 期望。
- `MonitoringView`：遵守 `monitoring-log-contracts`。
- `WslManagementView` / `SshManagementView`：遵守 `environment-scoped-dashboard-contracts`。
- `tray/CodexTrayPanelView` + `TrayOverview`：去掉 22px 字面圆角。
- 源文件编辑入口遵守 `raw-config-editor-contracts`。

**验证**：同 Wave 4 + 上列契约对应测试。

## Wave 6 — 模态与硬编码色

1. 模态外壳 → `--surface-modal-*`：`CodexAgentEditorModal`、`ConfirmModal`、`BulkDeleteDialog`、`AccountFormModal`、`OAuthWizardModal`。Confirm 不改 `requestConfirm`。
2. `ConfigCard` cyan/violet → 语义 token。
3. 清 phantom-token：MCP 三面板、`McpManagerView`、`AgentIcons`、`CommandList`、`TokenDetailTab`、`BaseSlashCommands`。
4. 全站 rg（见 `prd.md` R7）。白名单外为零。`linear-gradient` 清点后写入本文件「白名单登记」小节。

**验证**：rg 报告附进 `evidence/hardcode-audit.txt`。

## Wave 7 — 全量验证

1. `cd ccr-ui && bun run type-check && bun run lint && bun run test && bun run build`；仓库根目录 `just ui-check`。
2. `views-inventory.md` 51 行全勾。`evidence/` 按 `prd.md` R8 齐套。
3. 每页核对：无内高光 / 无装饰渐变 / 无 glow / 适用原语已接 / accent 稀缺 / tabular-nums / CJK 无字距。
4. 跑 impeccable detector 一次。
5. 再读 `theme-token-contracts.md` 与 DESIGN.md，补 Wave 0 之后的遗漏，走 `trellis-update-spec`。

## 白名单登记（Wave 6）

| 文件 | 类型 | 理由 |
|---|---|---|
| `AppSettingsView.vue` 色板预览 | hex | 唯一 hex 白名单 |
| （无） | linear-gradient | 2026-08-18 清扫后 `ccr-ui/src` 内 `linear-gradient` 为 0 |

## 风险文件

| 文件 | 风险 |
|---|---|
| `themeBootstrap.ts` + `index.html` IIFE | 两表不一致 → 首绘闪烁 |
| `theme-contrast-contract.smoke.test.ts` | 误改阈值常量 |
| `apple-glass-surface-contract.smoke.test.ts` | mocha 断言与 Checkin `styleLockedPaths` |
| `OpenCodePageShell.vue` | 复制字面 rgb 进 PageShell |
| `dashboardPresentation.ts` | 误改 signal 语义 |
| `usageChartOptions.ts` / `apexChartsCore.ts` | 拆完整 CSS 加载 |
| `opencode-view.smoke.test.ts` | 误改 `catppuccin-mocha` TUI 主题名 |
| `bootMessages.ts` | 只改 locale、漏 boot 包 |

## 启动前检查

- [x] `prd.md` 已收敛；DESIGN.md 权威链已写死
- [x] accent 迁移表含 `sage` / `sky` / `slate`
- [x] 51 视图勾销表已独立成文
- [x] 原语按需接入；死组件处置与 Wave 6 不打架
- [x] `design.md` 含迁移 / 取舍 / 回滚
- [x] jsonl 含 Grok / WSL-SSH / raw-editor / 任务 design.md
- [x] 发版冻结与 `base_branch=dev` 已写
- [ ] 用户确认本规划摘要后，才可 `task.py start`
