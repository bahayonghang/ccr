# Usage 子页表格 — 执行计划

## Ordered checklist

1. **Ledger 壳**  
   新增 `ccr-ui/src/features/usage/components/UsageLedger.tsx` 与 `styles/usage-ledger.css`。导出 `UsageLedger` + `memo` 的 `UsageLedgerRow`。`role="table"`，表头 `role="row"` / `columnheader`，数据行 `role="row"` / `cell`。列轨道通过 CSS 变量传入。先写失败测试再接线（步骤 7 的红样可与本步并行：先断言 Models 还没有独立 Cost 表头）。

2. **DTO → 行模型**  
   在 `UsageTableTabs.tsx`（或同目录小型 `usageLedgerRows.ts`，若 tab 文件会超过现有复杂度）把 model/project/provider stats 收成 `UsageLedgerRowData[]`。`useMemo` 依赖 `stats` 与 `formatCost`/`formatTokens`/`t`。Share 分母为本表 cost 合计；合计为 0 则 share 文案 `0%`。Pricing 映射到已有 `usage.dashboard.table.status*`；未知 status 显示原值，不用新 token。

3. **接 Models / Projects / Providers**  
   三个 tab 改为：标题 + `UsageLedger` 或空态。删除 `article.models-tab__row` 这类无 CSS 的节点。Projects 主名 `shortenPath`，`secondary`/`title` 为完整路径。Providers 名称走 `usageSourceFallbackLabel`。

4. **清死 CSS**  
   `usage-models-tab.css`：删除 `min-width: 106rem` 与只对 `thead`/`tbody` 生效的规则。`usage-providers-tab.css` 同样。`usage-projects-tab.css`：若不再用 `__item` 卡片，删除未引用规则。保留 empty / title。

5. **Tokens 日表（R5）与日柱（R11）**  
   `UsageTokensTab.tsx`：图表 `article` 后增加 ledger。行来自已有 `rows`。柱状图改接到 `buildDailyBarChartOptions` / `toDailyBarPoints`；options 只依赖 theme、locale、`rows.length`、stacked。空态：无 trends 时图与表都走现有 empty，不单独空表。`UsageCostTab` 日柱同样接入该工厂。`useUsageViewStore` 初态/重置带 30 日窗口（R12）。

6. **Overview / Cost 排行标记**  
   `UsageOverviewTab.tsx`：按 `design.md` 补 `rank-index`、`rank-row`、`detail`、`rank-bar`。`UsageCostTab.tsx`：补 `cost-tab__rank`、`ranking-row`、bar、`share_cost` 百分比。不要再让 `rank-item` 只有一个子节点。

7. **测试**  
   新增 `ccr-ui/tests/usage/usage-table-layout.smoke.test.tsx`（夹具：长 `project_path`、空 provider、多模型费用合计、空 stats、一条 trend）。收紧 `usage-tabs.smoke.test.tsx`：stub 补齐 `request_count` / `pricing_status` / `cost_with_cache_usd`，断言表头文本。行组件不得使用 array index 当 key。

8. **浏览器走查**  
   `cd ccr-ui && bun run dev:web -- --host 127.0.0.1 --strictPort`，`http://127.0.0.1:5173/` → Usage。桌面宽度，dark × clay。过 Models / Projects / Providers / Tokens / Overview 排行 / Cost 排行。确认无粘连、排行能读最后一段路径、窄宽（约 900px）表壳可横滑。

## Validation commands

```bash
cd ccr-ui && bunx vitest run --config vitest.smoke.config.ts tests/usage/usage-tabs.smoke.test.tsx tests/usage/usage-table-layout.smoke.test.tsx tests/usage/usage-daily-bar-chart.smoke.test.ts tests/state/state-store-actions.smoke.test.ts
cd ccr-ui && bun run type-check
cd ccr-ui && bun run lint:ci
```

改 i18n 时加 `cd ccr-ui && bun run test:i18n`。日柱改动后跑 `tests/usage/usage-daily-bar-chart.smoke.test.ts` 与 `tests/usage/usage-chart-stability.smoke.test.tsx`。

## Risky files

| 文件 | 风险 |
|---|---|
| `UsageTokensTab.tsx` / `UsageCostTab.tsx` | 日柱必须走 datetime 工厂；options 不得把 trends 数组放进依赖 |
| `UsageTableTabs.tsx` | 无 CSS 的 `__row` 与 106rem 类名；删除时确认无其它引用 |
| `usage-overview-tab.css` / `usage-cost-tab.css` | 标记修好后不要再改网格列数，除非走查仍挤 |
| `UsageLogsTab.tsx` | 不要为了复用去改虚拟列表 |

## Rollback points

- Ledger 接入前：仅新文件，可删。
- 三表接入后：回退 `UsageTableTabs.tsx` + CSS 删除。
- 排行接入后：回退两个 tab 的 list 标记。
- 测试失败：先修断言/标记，不要放宽 AC 去删列。

## Follow-up before `task.py start`

- `implement.jsonl` / `check.jsonl` 已含真实 spec/research 条目（非 seed）。
- 规划摘要已给用户，且用户明确批准后再 `task.py start`。
- 不在本任务改 DTO / Tauri / llmusage。
