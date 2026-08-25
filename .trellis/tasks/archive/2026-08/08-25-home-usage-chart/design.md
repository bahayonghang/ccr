# 技术设计：用量与成本图表区

## 1. 改动范围

| 文件 | 改动 |
|---|---|
| `ccr-ui/src/features/usage/dashboard/DashboardUsageMovement.tsx` | 呈现层改写；条件挂载成本子组件 |
| `ccr-ui/src/features/usage/dashboard/DashboardCostMetric.tsx` | 新增：唯一调用 `useUsageSummary` 的组件 |
| `ccr-ui/src/features/usage/styles/dashboard-usage-movement.css` | 版式重写 |
| `ccr-ui/tests/dashboard-usage-movement.smoke.test.tsx` | 新增或扩展：派生函数、成本三态、区间构造 |

props 契约不变：`overview` / `loading` / `error` / `activeDays` / `onChangeDays` / `className`。
不改 `DashboardView.tsx`、`dashboard-view.css`（归 `08-25-home-runtime-layout`）。
不改 `ccr-ui/src/features/usage/queries.ts`、不改 `src/api/`。

测试文件落点先查 `rg -l 'DashboardUsageMovement' ccr-ui/tests`，有则扩展，无则新建，落点写进本表。

## 2. 版式

对应设计稿 526–582 行：

```
[标题行]  用量与成本 + 「仅统计本地用量归档 · 更新于 {last_updated}」   |  7D/30D/90D 分段控件
[指标行]  请求(hero) · TOKEN · 成本 · 会话 ······ 平台图例
[图表]    flex:1，按天堆叠柱，两条虚线网格，底边框
[轴与出口] 日期刻度(mono 11px)                    |  「打开完整报表 →」
```

指标行的四项对应：

| 指标 | 来源 | 字号档 |
|---|---|---|
| 请求 | `overview.summary.total_requests` | hero 档（全页唯一） |
| TOKEN | `overview.summary.total_tokens` | 次级数据档 |
| 成本 | `DashboardCostMetric` → `total_cost_usd` | 次级数据档 |
| 会话 | `overview.summary.total_sessions` | 次级数据档 |

**字号档位不在本文件写死令牌名。** 令牌子任务的名称增量审计默认结论是复用既有
`--text-*` 档位而非新增 `--text-data-lg` / `--text-data-md`。开工前读
`08-25-design-token-consolidation/research/token-name-delta.md` 取实际档位名，
按其结论填入 CSS。若该文件判定确需新增，则用新增名称。

## 3. 堆叠柱的数据派生

输入 `overview.series: HomeOverviewSeriesItem[]`，每项形如
`{ date, claude, codex, antigravity, opencode }`，四个平台各有 `{ sessions, requests, tokens }`。

派生步骤：

1. 对每个 `series` 项，取四平台的 `requests`，得到该天的四段值。
2. 全序列的日总和最大值作为柱高基准（`maxDailyTotal`）。
3. 每根柱的整体高度 = `dayTotal / maxDailyTotal`；柱内各段高度 = `platformRequests / dayTotal`。
4. `maxDailyTotal === 0` 时走空态，不渲染零高柱。

段色取平台色令牌（映射见 `08-25-home-runtime-layout/design.md` §5，含 `antigravity → --color-platform-gemini` 的名称漂移）。

图例只列出在当前区间内有非零值的平台，避免图例与图形不符。

## 4. 成本接入

### 4.1 区间对齐

`useUsageSummary(platform?, startDate?, endDate?)` 的 `queryKey` 是
`usageKeys.summary(platform, startDate, endDate)`，**随三个参数变化**。
因此显式传区间既能对齐口径，又能在切换 7D/30D/90D 时自动重取。无参调用取的是另一区间语义，不能用。

区间按后端 `local_usage_date_window(days)` 的同一口径构造：

```ts
// 与 usage.rs 的 local_usage_date_window 对齐：end = 本地今天，start = end - (days - 1)
function homeDateWindow(days: number): { startDate: string; endDate: string } {
  const safeDays = Math.max(1, days)
  const end = new Date()
  const start = new Date(end)
  start.setDate(end.getDate() - (safeDays - 1))
  return { startDate: formatLocalDate(start), endDate: formatLocalDate(end) }
}
```

`formatLocalDate` 必须按本地日期格式化（`YYYY-MM-DD`），不能用 `toISOString()`——
那会转成 UTC，在非零时区跨日时产生偏移一天的区间。

这个函数与其对 `activeDays` 的响应是 AC4 的断言对象，写成可导出的纯函数。

### 4.2 延迟发起：条件挂载，不改 hook

`useUsageSummary` 没有 `enabled` 参数。给它加参数会波及所有既有调用方，不在本任务范围。
改为把 hook 调用隔离到一个子组件，由父组件控制挂载时机：

```tsx
// DashboardUsageMovement.tsx
const [costReady, setCostReady] = useState(false)
useEffect(() => scheduleWhenIdle(() => setCostReady(true)), [])
...
{costReady ? <DashboardCostMetric days={activeDays} /> : <CostPlaceholder />}
```

```tsx
// DashboardCostMetric.tsx —— 本任务唯一调用 useUsageSummary 的地方
const { startDate, endDate } = homeDateWindow(days)
const query = useUsageSummary(undefined, startDate, endDate)
```

条件渲染子组件即条件调用 hook，是 React 允许的形式；在父组件内用条件调用 hook 不是。

`scheduleWhenIdle` 是 `DashboardView` 已有的模式，沿用同一工具，不新造调度。

### 4.3 三态呈现

| 情形 | 呈现 |
|---|---|
| 未挂载 / `isLoading` / `isError` / `data == null` / 非 Tauri 运行时 | `—` |
| 有数据且 `total_cost_usd === 0` | `$0.00` |
| 有数据且 `total_cost_usd > 0` | 格式化金额 |

`—` 与 `$0.00` 必须可区分（AC6）。不新增 `src/api/` 封装，不新增 IPC 命令。

## 5. 空态、loading、error

| 状态                              | 呈现                                                    |
| --------------------------------- | ------------------------------------------------------- |
| `loading`                         | 骨架：指标行占位 + 图表区占位条，保留卡片边框与标题     |
| `error`                           | 卡内错误文案 + 重试入口，保留标题行                     |
| `overview == null` 或 `series` 空 | 空态文案 + 说明为何为空（可用 `overview.empty_reason`） |
| 非 Tauri 运行时                   | 按 AC5 的诚实要求显示不可用，不伪造数据                 |

三种状态都不得出现空白卡（R6）。

## 6. 可访问性

- 图表容器给 `role="img"` + `aria-label`，文案含区间、总请求数与参与平台。
- 每根柱给 `title`，内容为日期与各平台请求数，使灰度下也能取得归属信息（AC7）。
- 分段控件 7D/30D/90D 用 `role="radiogroup"` + `aria-checked`，键盘可达。
- `prefers-reduced-motion: reduce` 下关闭柱状入场动画（AC8）。

## 7. 不引入图表库

堆叠柱用嵌套 flex + 百分比高度实现，与设计稿 568–574 行的做法一致。设计稿已验证该结构可行，且不新增运行时依赖（Out of Scope）。

## 8. 测试

| 断言对象 | 内容 |
|---|---|
| `homeDateWindow(days)` | `days = 7` 时 `endDate` 为今天、`startDate` 为今天减 6 天；`days = 30` / `90` 同理；跨时区不产生偏移一天（用固定时钟测）。这是 AC4 |
| 堆叠柱派生纯函数 | 正常输入按天分层；`maxDailyTotal === 0` 走空态；图例只含有非零值的平台 |
| 成本三态 | 未挂载 / query 无数据 → `—`；`total_cost_usd === 0` → `$0.00`；`> 0` → 格式化金额。这是 AC6 |
| 延迟挂载 | 首次渲染时 `DashboardCostMetric` 未挂载（AC5） |

派生函数与 `homeDateWindow` 都写成可导出的纯函数，测试直接调用，不经组件渲染。

## 9. 回滚

```bash
git checkout -- ccr-ui/src/features/usage/dashboard/DashboardUsageMovement.tsx \
  ccr-ui/src/features/usage/styles/dashboard-usage-movement.css ccr-ui/tests/
rm -f ccr-ui/src/features/usage/dashboard/DashboardCostMetric.tsx
```

新增文件需显式删除，`git checkout` 不会移除未跟踪文件。不影响其他子任务。

## 开工记录

- 测试落点：`rg -l DashboardUsageMovement ccr-ui/tests` 无命中，新建 `ccr-ui/tests/dashboard-usage-movement.smoke.test.tsx`。
- 数据字号按 `token-name-delta.md` A3：hero `--text-2xl`，次级 `--text-xl`，不新增 `--text-data-*`。
- 平台色：claude → `--color-platform-claude`；codex → `--color-platform-codex`；antigravity → `--color-platform-gemini`；opencode → `--color-platform-opencode`。
