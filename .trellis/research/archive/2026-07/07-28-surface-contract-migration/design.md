# 组件表面迁移 — 技术设计

> 依赖：`07-28-color-system-rebuild` 完成后的新令牌。策略：**映射机械、白名单登记、逐批验证**。

## 1. 迁移映射表

| 现状写法 | 目标写法 | 备注 |
|---|---|---|
| `bg-bg-base/N`、`bg-bg-elevated/N`（N≤70） | `bg-bg-base` / `bg-bg-elevated`（实心） | 半透明想表达的"凹陷层"用低一档实心 |
| `bg-bg-surface/N`、`bg-bg-overlay/N`（N>70） | `bg-bg-surface` / `bg-bg-overlay`（实心） | |
| 真遮罩/scrim（modal 背后、抽屉背后） | 新增 `--color-scrim`（暗色 `rgb(0 0 0 / 56%)`，亮色 `rgb(25 27 32 / 32%)`），子任务 A 增补；若 A 未增补则本任务在 tokens.css 补 | 唯一允许的常驻半透明 |
| hover/active 的 `bg-bg-*/N` | `rgb(var(--color-bg-overlay-rgb) / 60%)` 形式 | 交互反馈保留 alpha |
| `color: rgb(var(--color-text-*-rgb) / N%)` | `var(--color-text-secondary/muted/ghost)` | 按语义选档 |
| `inset 0 1px 0 rgb(255 255 255 / N%)` | 删除或 `box-shadow: var(--inner-glow)` | 暗色一律删；亮色 >46% 收敛 |
| `.btn-* color: white` | `var(--color-accent-*-contrast)` | A 未提供 danger/success contrast 时本任务增补 |
| `bg-white/80 dark:bg-bg-elevated/90 backdrop-blur-xl`（BaseModal） | `--surface-modal-*` 四件套 | |
| `glass-panel`（usage 仪表盘） | card 语义（`--surface-card-*`） | |

## 2. 批次划分（每批独立验证）

- B1 重灾区 9 文件（CodexAgentSourcesPanel / Monitoring / OpenCode×5 / CodexAgents / Agents）。
- B2 其余 34 文件的 alpha 表面。
- B3 文本透明度 + inset 高光 + text-white。
- B4 壳层/模态/glass-panel/图表色 + 计算样式验证。

## 3. 白名单机制

`implement.md` 维护白名单登记表（文件:行 → 理由）。AC 的 rg 扫描 = 全量 − 白名单。白名单条目必须满足：遮罩 scrim、交互态 alpha、accent 上 contrast 文字、mask 图像。

## 4. 验证设计

- 每批后：`bun run type-check && bun run lint` + 相关 smoke。
- B4 后：Playwright 断言 `.sidebar-glass`/`.topbar-glass` 计算样式（`backdrop-filter: none`、背景 alpha=1）。
- 视觉抽查 4 条路由截图。

## 5. 风险与回滚

- 风险：`bg-bg-*/N` 语义误读（把"刻意透视"改成实心后视觉变闷）——逐批截图对比，拿不准的保留并登记白名单。
- 回滚：按批次 git 还原；批次间无依赖。
