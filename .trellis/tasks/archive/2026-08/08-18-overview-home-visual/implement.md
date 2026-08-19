# 实施清单：Overview 铺满 + StatTile 徽章

## 顺序

1. **宽度**
   - `ccr-ui/src/components/ui/PageShell.vue`：删除 `.page-shell__inner` 的 `max-width: 1480px` 与 `margin-inline: auto`，改为 `width: 100%; min-width: 0`。
   - `ccr-ui/src/views/DashboardView.vue`：删除 `.dashboard-workbench` 的 `width: min(100%, 1440px)` 与 `margin: 0 auto`，改为 `width: 100%; min-width: 0`。
   - 全局搜 `1480px` / Dashboard 的 `1440px`，确认没有第二处壳上限。不要动 `AppSettingsView`、`profiles-page.css`、桌面默认窗宽。

2. **StatTile**
   - `ccr-ui/src/components/ui/StatTile.vue`：增加可选 `tone`；有值时给 `.stat-tile__value` 加徽章壳与 `data-tone`。
   - 数字保持主文本色 + `tabular-nums`。壳用 token 的 10/18 配方。禁止 `ui-card`、禁止字面色。
   - 导出类型与 `DashboardTone` 对齐；可从 `dashboardPresentation` 复用类型，或在 `StatTile` 内写同一字面联合，避免 UI 原语反向依赖 view。推荐：`StatTile` 自备联合，Ledger 传入的 `metric.tone` 结构兼容。

3. **接线**
   - `DashboardReadinessLedger.vue`：`:tone="metric.tone"`。
   - `DashboardUsageMovement.vue`：四个摘要 `:tone="'neutral'"`。
   - `width >= 1680` 时就绪条指标改为五列等分；更窄用 `auto-fit minmax(9.5rem, 1fr)`。

4. **合同**
   - `dashboard-presentation-contracts.md` 增补：tone 只驱动壳。
   - 若 `ccr-ui/DESIGN.md` 仍写「禁语义色装饰数字」，改成禁止把数字本身涂成语义色；浅壳允许。

5. **测试**
   - `ui-primitives.smoke.test.ts`：保留无 tone 裸瓦片；新增有 tone 的壳断言（`data-tone`、无 `ui-card`、源码含 `tabular-nums`）。
   - 跑 `dashboard-presentation.smoke.test.ts`，确认判定未动。

6. **验证**
   - `cd ccr-ui && bun run type-check`
   - `cd ccr-ui && bun run lint`
   - `cd ccr-ui && bunx vitest run --config vitest.smoke.config.ts tests/ui-primitives.smoke.test.ts tests/dashboard-presentation.smoke.test.ts`
   - web preview：`cd ccr-ui && bun run dev:web -- --host 127.0.0.1 --strictPort`
   - 预载 `ccr-theme` / `ccr-flavor` / `ccr-accent`，断言 dataset 后再截图。
   - 截图写入本任务 `evidence/`。

## 风险点

| 文件 | 风险 | 回滚 |
|---|---|---|
| `PageShell.vue` | 40+ 页同时变宽 | 恢复 1480 + `margin-inline: auto` |
| `StatTile.vue` | 默认外观被徽章污染 | `tone` 必须可选且默认关闭 |
| `DashboardReadinessLedger.vue` | 宽屏五列把长数字挤换行 | 壳 `inline-flex` + `minmax(0,1fr)` |
| `DESIGN.md` / spec | 与旧「禁装饰数字」字面冲突 | 按 R4 改一句，阈值不降 |

## `task.py start` 前

- [x] `prd.md` 已收敛，无阻塞 Open Questions
- [x] `design.md` / `implement.md` 已写
- [ ] 用户批准本规划摘要
- [ ] 批准后再 `python ./.trellis/scripts/task.py start 08-18-overview-home-visual`

## 启动后检查

- 改 `StatTile` 前先 `rg "StatTile" ccr-ui/src`，确认未传 `tone` 的调用点不会吃到新壳。
- 改 `PageShell` 前 `rg "1480px|max-width: 1480" ccr-ui`。
- 截图前确认 `data-theme` / `data-flavor` / `data-resolved-flavor` / `data-accent`。
