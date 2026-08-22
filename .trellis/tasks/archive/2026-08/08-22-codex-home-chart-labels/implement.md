# Implement: Codex 首页趋势图坐标标签与 KPI 图标优化

## Checklist

1. 从 `usageChartOptions.ts` 导出 `parseUtcDate`。现有 `formatTrendTooltipLabel` 继续走同一函数。补一条 smoke：`'2026-07-22'` → `Date.UTC(2026, 6, 22)`。
2. 新增 `ccr-ui/src/views/platform-usage/platformUsageTrendChart.ts`：输入 `DailyTrend[]` + metric，输出 datetime series 与 `tickAmount`。series join key 记忆化留在 Vue computed 里，纯函数只做转换。
3. 改 `PlatformUsageTrendChart.vue`：datetime 轴、`trim: false`、`formatTrendAxisLabel`、`getTrendTickAmount`、`buildChartAnimations`、resize 两旗。`useI18n().locale` 注入 formatter。高度仍为 `286`。
4. 给步骤 2 的纯函数加 smoke（可放 `tests/platform-usage-presentation.smoke.test.ts` 或新 `tests/platform-usage-trend-chart.smoke.test.ts`）：30 点 tickAmount=6；timestamp 对齐 UTC 日；辅助模块源码不含 `trim: true`。
5. 改 `PlatformUsageInsightPanel.vue` KPI 图标：按 `card.id` 设 `--kpi-icon-rgb`，浅底 + 描边 + `w-5 h-5`。按钮/notice 只核对对齐，不改语义色规则。
6. 改 `CodexView.vue`：页头四个图标进 `Button` leading 槽；readiness 与 management 的 `SIcon` 改为 `w-5 h-5`。
7. 验证：`cd ccr-ui && bun run type-check`；`bun run lint`；`bunx vitest run --config vitest.smoke.config.ts tests/usage-chart-diagnostics.smoke.test.ts tests/platform-usage-presentation.smoke.test.ts tests/apexcharts-style-contract.smoke.test.ts` 以及新的 trend-chart smoke。
8. 浏览器或 Tauri：Codex 首页 Cost / Tokens / Requests，zh-CN 与 en-US，dark。截图进 `evidence/`。扫 Antigravity 与 OpenCode 首页确认面板未回归。

## Risky files

- `PlatformUsageTrendChart.vue`：options 引用一变会重建 canvas。先抽纯函数再改 Vue。
- `usageChartOptions.ts`：只导出，不改 `formatTrendAxisLabel` 行为。
- `CodexView.vue`：页头槽位改错会让按钮文字与图标间距叠两层 `mr-2`。改完看间距只有 leading 槽那一档。

## Rollback

`git checkout --` 上述文件与测试。无持久化副作用。

## Validation commands

```
cd ccr-ui && bun run type-check
cd ccr-ui && bun run lint
cd ccr-ui && bunx vitest run --config vitest.smoke.config.ts tests/usage-chart-diagnostics.smoke.test.ts tests/platform-usage-presentation.smoke.test.ts tests/apexcharts-style-contract.smoke.test.ts
```

新 smoke：`ccr-ui/tests/platform-usage-trend-chart.smoke.test.ts`。验证命令补上该文件。

```
cd ccr-ui && bunx vitest run --config vitest.smoke.config.ts tests/platform-usage-trend-chart.smoke.test.ts tests/usage-chart-diagnostics.smoke.test.ts tests/platform-usage-presentation.smoke.test.ts tests/apexcharts-style-contract.smoke.test.ts
```

## Before start

- 规划已收敛：D1 locale short，D2 整个 Codex 首页已有图标，D3 KPI 分卡语义色。
- 实现前再读 `usage-chart-stability-contracts.md` 与 `theme-token-contracts.md`。
- 用户批准本规划摘要后才运行 `task.py start`。
