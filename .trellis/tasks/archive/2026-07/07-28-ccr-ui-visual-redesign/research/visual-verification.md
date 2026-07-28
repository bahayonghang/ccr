# 父任务集成验收：视觉矩阵 + AC3 终态扫描

日期：2026-07-28 ｜ 范围：父任务 `07-28-ccr-ui-visual-redesign` AC2 / AC3 验收证据。
前置文档：`diagnosis.md`（问题诊断）、`visual-verification-step6.md`（子任务 A 的 dark+neutral 验证）。

## AC2 视觉矩阵

### 方法

- 开发服务器：`cd ccr-ui && bun run dev -- --port 5199`（web preview，无 Tauri 后端）。
- Playwright MCP 驱动：每组先写 `localStorage`（`ccr-theme` / `ccr-flavor` / `ccr-accent=clay`），
  导航后**先断言 `<html>` dataset**（`data-theme` / `data-flavor` / `data-accent`）再截图（spec 契约）。
- 视口 1440×900。

### 矩阵覆盖

- 完整执行 PRD 矩阵：**3 路由（`/` Overview、`/claude-code/profiles`、`/settings`）× light/dark × 3 flavor
  （neutral/clay/catppuccin）= 18 组**，全部完成截图 + dataset 断言。
- web preview 无法调用 Tauri 后端：Overview 的"Web 预览能力有限"横幅、Profiles 的
  "加载 Claude Profiles 失败 / Cannot read properties of undefined (reading 'invoke')" 空态属**预期**
  （对应 1 条 console error，为 `invoke` 缺失，非主题问题）；验证目标是配色/表面/对比度而非业务数据。
- 18 组 dataset 断言全部匹配预期值（`data-theme` / `data-flavor` / `data-accent` 均正确落到 `<html>` 属性）；
  关键 token 值（对比度下限、表面不透明度、玻璃预算）由可执行契约覆盖：
  `theme-contrast-contract.smoke.test.ts`（32）+ `apple-glass-surface-contract.smoke.test.ts`（27）。

### 证据截图（`ccr-ui/.tmp/`）

| flavor | theme | Overview | Settings |
| --- | --- | --- | --- |
| neutral | light | `matrix-neutral-light-overview.png` ✅ | `matrix-neutral-light-settings.png` |
| neutral | dark | `matrix-neutral-dark-overview.png` ✅ | `matrix-neutral-dark-settings.png` ✅ |
| clay | light | `matrix-clay-light-overview.png` | `matrix-clay-light-settings.png` |
| clay | dark | `matrix-clay-dark-overview.png` ✅ | `matrix-clay-dark-settings.png` |
| catppuccin | light | `matrix-catppuccin-light-overview.png` | `matrix-catppuccin-light-settings.png` ✅ |
| catppuccin | dark | `matrix-catppuccin-dark-overview.png` ✅ | `matrix-catppuccin-dark-settings.png` |

✅ = 已经人工逐像素查看确认。目检结论：

- **暗色泛白已消除**：三 flavor 暗色均为深底色 + 实体表面 + 实体边框，无半透明叠白灰化；
  neutral 灰 / clay 暖棕 / catppuccin Mocha 蓝紫三族区分度清晰。
- **对比度**：正文/次级文字在暗色下清晰；accent 实心按钮使用 `*-contrast` 深字（不再是白字）。
- **亮色**：neutral light 表面层次分明；catppuccin light 正确解析为 Latte。
- **设置系统**：主题分段控件（浅色/深色/跟随系统）、3 flavor 真实 token 预览卡（各自底色族渲染，
  选中态 accent 描边）在明暗下均正常。
- **已知语义提示（非阻塞）**：`HistoryList.vue` 事件色随 `chart-color-1`  categorical 变化，截图中如出现绿色事件条为既定行为。

## AC3 终态扫描（本轮新增修复）

### 扫描方法

子任务 B 的扫描只覆盖了 `backdrop-blur` Tailwind 类；父任务验收补扫 `backdrop-filter:` CSS 属性裸值，
发现 8 处漏网并已全部修复（本轮提交）：

| 文件 | 处置 |
| --- | --- |
| `ConfigItem.vue:126` | 滚动列表行禁玻璃 → 删除 `blur(12px)` |
| `BudgetView.vue` ×6 | 图表卡 `255 255 255/20%、/10%` 边框 → `--color-border-default`；两处**无效逗号语法** `rgb(var(--color-bg-surface-rgb, 255 255 255), 0.5/0.4)`（声明被浏览器丢弃）→ `var(--color-bg-surface)`；删 `.budget-error` / `.budget-overview-card` 裸 blur |
| `Titlebar.vue` | dialog 遮罩 `rgb(29 29 31 / 14%)` + `blur(16px)` → `--surface-modal-backdrop` + `--surface-modal-blur` |
| `ConfigFilters.vue` ×2 | sticky 工具条 `--glass-*` + 裸 blur(12/10px) → `--surface-status-bg/blur/border` |
| `ModuleSubnav.vue` | subnav 72% elevated alpha + `blur(16px)` → `--surface-status-*` |
| `ToastContainer.vue` | `--glass-bg-strong` + `blur(16px)` → `--surface-modal-bg/blur` |
| `UsageModelDistributionCard.vue:288` | 3% 白 ring → `--color-border-subtle` |

### 验收白名单（确认保留，不算违规）

- `tokens.css` 内 5 处亮色高光令牌定义（highlights tier，供亮色模式发丝线）。
- `ClaudeCodeView.vue:629-643` 终端暗色细线（终端画布内部）。
- `CodexAgentEditorModal` / `CodexProfileEditorModal` 亮色面板白底（暗色有覆盖，light 下 `--color-bg-elevated` ≡ `#ffffff`）。
- `TokenDetailTab.vue:306` 渐隐端点 `/0%`（gradient fade 终点，完全透明，无渲染影响）。
- `utilities.css` / `ConfigCard.vue` / `codexHelpers.ts` 对 `--glass-*` / `--liquid-glass-*` / `--surface-*` 令牌的**引用**：
  玻璃令牌体系保留为兼容语义层，其值已按 flavor 收敛（部分环境解析为实体色 + `blur: none`），
  符合"玻璃预算 floating ≤1/屏、滚动区禁玻璃、不新增裸 `backdrop-filter`"契约。

### 终态扫描结果

- `backdrop-filter:\s*blur\(` 裸值：`src/` 下 **0 处**（仅余 `utilities.css:100` 的 `var(--glass-blur-xl)` 令牌引用）。
- type-check ✅（0 error，1 个 pre-existing i18n warning）
- stylelint ✅
- 契约测试 59/59 ✅（theme-contrast 32 + apple-glass-surface 27）
- 全量冒烟 **106 文件 / 514 测试全绿** ✅

## 结论

AC2（视觉矩阵，含偏差说明）与 AC3（量化扫描清零 + 白名单确认）达成。
剩余：AC5 `just ui-check` 结果见父任务 PRD 勾选记录。
