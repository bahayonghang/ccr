# 泛白问题根因诊断（2026-07-27 审计）

> 来源：三张暗色（Catppuccin Macchiato）截图 + 全仓代码审计。所有结论附文件与行号。
> 本文件是三个子任务的共享证据库，实施时按引用查阅。

## 用户可感症状

暗色模式下整个 UI 呈"奶雾感"：文本发灰、卡片边界模糊、按钮发白发灰、所有页面像蒙了一层雾。

## 根因清单（按视觉影响排序）

### R1. 确定性 bug：无暗色守卫的白色背景

- `ccr-ui/src/views/generic/PlatformMcpView.vue:410,415` 与 `PlatformPluginsView.vue:371,376`：`onCardHover` 用 JS 无条件写入 `rgba(255,255,255,0.9/0.7)` 内联背景，hover 一次后卡片在暗色下永久发白。
- `ccr-ui/src/views/OutputStylesView.vue:253,339`：`bg-white` 无 `dark:` 守卫，暗色下纯白按钮。

### R2. 壳层 chrome 玻璃常驻双大面

- `MainLayout.vue:443-467` 侧栏 + 顶栏使用 `--surface-shell-*` → `--material-glass-chrome-*`：暗色 `bg-elevated/78%` + `blur(10px) saturate(150%)`（`tokens.css:562`），每页透出背景光晕。

### R3. 内容区 247 处模板 alpha 表面

- `bg-bg-{base,elevated,surface,overlay}/N` 共 247 处 / 43 文件。重灾区：`CodexAgentSourcesPanel.vue` 19、`MonitoringView.vue` 18、`OpenCodeAgentsView.vue` 16、`OpenCodeSettingsView.vue` 15、`OpenCodeMcpView.vue` 13、`CodexAgentsView.vue` 12、`OpenCodeProvidersView.vue` 12、`OpenCodePluginsView.vue` 11、`AgentsView.vue` 11。

### R4. 全局 StageBackground 底雾

- `App.vue:2` 全局挂载 `StageBackground.vue`：暗色下 34% premium-blue halo（blur 88px、opacity 0.54）+ 16% premium-pink 顶部洗色带 + 4.6% 噪点（`StageBackground.vue:67-85`）。全屏常驻，被 R2/R3 所有半透明面放大。
- `AnimatedBackground.vue`（挂载于 `ClaudeCodeView.vue:3`、`ConfigsView.vue:3`、`OpenCodePageShell.vue:3`）同类问题：72% premium-blue halo。

### R5. stage 文本令牌半透明

- 暗色 `--color-stage-text-muted` 76%、`quiet` 62%、`secondary` 90%、`primary` 98%（`tokens.css:202-205`），经 `theme.css:54-57` 桥接后 134 处 / 10 文件（ClaudeCodeView 24、CodexView 22、GeminiCliView 17、ClaudeAuthView 14 等）。
- 直接写死的 `color: rgb(var(--color-text-*-rgb) / N%)` 另有 11 处（Titlebar.vue 387/397/430/438 等）。

### R6. 约 30 处卡片 inset 白高光

- `inset 0 1px 0 rgb(255 255 255 / 4%–20%)` 遍布所有卡片族，暗色下每张卡顶部一条"奶白边"。最亮：`UsageMetricCard.vue:97` 12%、`ConfigFilters.vue:174` 20%。

### R7. 模态 floating 玻璃

- `--material-glass-floating-*` 暗色 66% + `blur(16px) saturate(170%)`（`tokens.css:558`）；`BaseModal.vue:231` 用 `bg-white/80 dark:bg-bg-elevated/90 backdrop-blur-xl backdrop-saturate-150`。

### R8. 粉彩 accent + 白字按钮

- 暗色 accent 全为浅粉彩（clay `#e79a77`；Catppuccin 映射 lavender/rosewater/mauve…），`utilities.css:290,313,324` 的 `.btn-primary/.btn-danger/.btn-success` 用 `color: white`，另有 `text-white` 62 处 / 27 文件 → 强调按钮发灰发白。

### R9. Catppuccin 映射反转了 elevation 几何

- clay 暗色：base `#17120f` < elevated `#221b18` < surface `#2a221e`（elevated 更亮，正确）。
- Catppuccin 映射（`tokens.css:1022-1029`）：`--color-bg-elevated: var(--ctp-mantle)` 比 base 更暗 → 暗色下"抬升面"反而下沉，层次浑浊。

### R10. 全局字体平滑

- `base.css:19-20` 全局 `-webkit-font-smoothing: antialiased` + `-moz-osx-font-smoothing: grayscale`，暗底上字形变细，文本更显灰。

### R11. 死代码背景系统

- `backgrounds.css` 的 `.premium-background` / `.premium-bg-orb` / `.orb-*` / `.premium-bg-pattern`（含 `mix-blend-mode: screen/overlay`、blur 90–100px）全仓无消费者；`tokens.css` 的 `--stage-bg-mesh/aurora/orb/grid-*` 令牌同样无人使用。存在被误挂载即翻倍雾感的隐患。

## 健康部分（保留）

- `--surface-card-bg` / `--surface-workspace-bg`（98% 近不透明）语义契约方向正确，需提到 100%。
- `theme.css` legacy bridge、`data-theme`/`data-flavor`/`data-accent` 三轴独立架构、index.html 首帧 IIFE 预注水。
- 玻璃三档预算制度（≤3 个、不嵌套、不进滚动区）——预算不变，但 chrome 档改不透明。

## 既有契约约束（spec 摘要）

- `.trellis/spec/ccr-ui/frontend/theme-token-contracts.md`：flavor 用语义重映射而非第二套组件语言；`data-theme`/`data-flavor`/`data-accent` 保持独立；高优先级覆盖用 `html:root[data-resolved-flavor='x']` 并配 smoke 锁定；reduced-transparency 重置块必须覆盖 flavor 作用域；替换默认 clay 需任务显式声明（本父任务已显式声明：用户选定中性高对比新默认）。
- 守卫测试：`theme-bootstrap.smoke.test.ts`（逐字锁定 index.html IIFE）、`app-settings.smoke.test.ts`（锁定 testid 与 localStorage 行为）、`apple-glass-surface-contract.smoke.test.ts`（材质令牌 + 字体栈扫描）、`font-preferences.smoke.test.ts`、`main-layout-theme-stage.smoke.test.ts`（settings dock）。
