# 设计：Overview 铺满 + StatTile 徽章

## 1. 边界

| 层 | 改 | 不改 |
|---|---|---|
| 壳 | `PageShell.vue` 去掉 1480 居中；`DashboardView.vue` 去掉 1440 居中 | `MainLayout` 滚动区内边距、侧栏宽度、`AppSettingsView` / `profiles-page.css` 的独立上限 |
| 原语 | `StatTile` 增加可选 `tone` 徽章壳 | 不新建 `StatBadge` / `MetricChip`；不改 `Badge.vue` 公共 API |
| Dashboard | `DashboardReadinessLedger` 传入 `metric.tone`；`DashboardUsageMovement` 摘要传 `tone="neutral"` | `dashboardPresentation.ts` 的判定与 `tone` 赋值 |
| 合同 | `dashboard-presentation-contracts.md`；必要时 `DESIGN.md` 一句修订 | flavor / accent 值域、对比度阈值、glass 预算 |
| 测试 | `ui-primitives.smoke.test.ts` 补 tone 分支 | 不改 presentation smoke 的判定期望，除非断言撞上新 DOM |

数据流：`buildStatusMetrics()` 已产出 `tone` → Ledger 透传 → `StatTile` 渲染壳。本任务补的是最后一跳。

## 2. 宽度

`.page-shell__inner` 删除 `max-width: 1480px` 与 `margin-inline: auto`，改为 `width: 100%; min-width: 0`。内边距、gap、header/subnav/content 槽不变。

`.dashboard-workbench` 删除 `width: min(100%, 1440px)` 与 `margin: 0 auto`，改为 `width: 100%; min-width: 0`。12 列栅格与 `<=1180` 回退不变。

`content-scroll-area` 的 `p-4 sm:p-6` 与 `PageShell` 的 `1rem 1.25rem` / `1.5rem` 叠在一起，PageShell 页左右约 48px。这是已有页边，不是 390px 居中空底。本任务不拆双层 padding。

页内更窄的阅读列（`PricingView` 54rem、说明 `max-w-[48rem]`）保留。它们约束的是正文，不是工作台。

## 3. StatTile 徽章

```
[label]
[  ●  35.5% / 76.2%  ]   ← 仅 value 进壳
[hint]
```

- 新 prop：`tone?: DashboardTone`。缺省 = 现网裸瓦片。
- 壳 class：`stat-tile__value` 在有 tone 时加 `stat-tile__value--badge` 与 `data-tone`。
- 颜色：数字 `var(--color-text-primary)`；壳背景 `rgb(var(--color-*-rgb) / 10%)`，边 `rgb(var(--color-*-rgb) / 18%)`。映射：

  | tone | token |
  |---|---|
  | neutral | `--color-bg-overlay` + `--color-border-subtle` |
  | success | `--color-success-rgb` |
  | warning | `--color-warning-rgb` |
  | danger | `--color-danger-rgb` |
  | accent | `--color-accent-primary-rgb`，透明度 10/18，与现网 Ready 徽章一致 |

- 圆角 `var(--radius-md)`（8px）。卡 12px + 内边距 16px，内壳 8px，同心。
- 可选 6px 圆点，颜色跟 tone；`neutral` 用 `--color-text-muted`。
- `font-variant-numeric: tabular-nums` 留在数值上。
- `display: inline-flex`；瓦片本身仍是 column，壳不 `width: 100%`。
- 字面 hex/rgb 禁止。语义色只进背景/边/点，不进化字。

不复用 `Badge.vue`：Badge 的字号档（xs–lg）和「字跟 variant 出色」会把数字涂成语义色，违反 R2。StatTile 自己画壳，token 与 Badge 的 10/18 配方对齐。

## 4. 就绪条栅格

就绪条在 `width >= 1680` 保持左右分栏 + 五列等分。更窄时整卡改为上下排列，指标用 `auto-fit minmax(9.5rem, 1fr)`，避免右侧 710px 里硬塞五列把「Web 预览」挤断或把第 5 项掉到下一行。

徽章数值 `white-space: nowrap`，不在壳内拆 CJK。

## 5. 兼容

- 未传 `tone` 的 20+ 个 `StatTile` 调用点 DOM/样式与改前一致。
- `data-theme` / `data-flavor` / `data-accent` 不合层。徽章只读已有语义 token。
- 回滚：还原 `PageShell.vue`、`DashboardView.vue`、`StatTile.vue`、两个 Dashboard 子组件、合同与 smoke。用户磁盘配置不受影响。

## 6. 取舍

| 选项 | 选择 | 代价 |
|---|---|---|
| 只放 Overview vs 连 PageShell | 连 PageShell | 40+ 页变宽；页内阅读列仍窄 |
| 默认全站徽章 vs 按 tone 开启 | 按 tone 开启 | 其他页数字暂不升级 |
| 复用 Badge vs StatTile 自绘壳 | 自绘壳 | 两处 10/18 配方要靠合同对齐 |
| 拆双层 padding | 不拆 | PageShell 页左右约 48px 页边留下 |

## 7. 风险

- 宽屏上五指标等分后，短值（`0`、`3/3`）左侧会空。壳跟数字走，空的是列，不是假卡。
- `accent` 用量归档在就绪条上会出现 clay 浅壳。这是已有 tone 赋值，不是新涂色。
- `ui-primitives` 的「bare tile」断言必须改成「默认 bare；带 tone 有壳」。
