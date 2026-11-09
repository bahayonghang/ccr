# 技术设计：首页 1b 栅格与平台卡

## 1. 改动范围

| 文件 | 改动 |
|---|---|
| `ccr-ui/src/shell/MainLayoutNav.tsx` | 分组标题 mono 标签样式；平台条目加平台色识别块 |
| `ccr-ui/src/features/usage/dashboard/DashboardView.tsx` | 栅格重排；就绪 pill 与主行动落在区块标题行；移除 readiness ledger 引用 |
| `ccr-ui/src/features/usage/styles/dashboard-view.css` | 新栅格与三档响应式 |
| `ccr-ui/src/features/usage/dashboard/DashboardPlatformMatrix.tsx` | 原地改写为四张卡 |
| `ccr-ui/src/features/usage/styles/dashboard-platform-matrix.css` | 卡片样式 |
| `ccr-ui/src/views/dashboard/dashboardPresentation.ts` | 加 `sparkline` / `trackingHealth` 可选字段与派生逻辑 |
| `ccr-ui/tests/dashboard-presentation.smoke.test.ts` | sparkline 派生断言（R10） |
| `ccr-ui/tests/dashboard-platform-matrix.smoke.test.tsx` | 占位分支 DOM 断言 |
| `ccr-ui/tests/helpers/usageFixtures.ts` | `makeSourceHealth` fixture |

条件删除（视 §7 结论）：
`ccr-ui/src/features/usage/dashboard/DashboardReadinessLedger.tsx`、
`ccr-ui/src/features/usage/styles/dashboard-readiness-ledger.css`。
这两个文件计入 change list 与回滚清单，不算范围外改动。

不改：`DashboardUsageMovement`、`DashboardNextActions`、`DashboardSignalStream` 的内部（分别归其他子任务）。

## 2. 栅格

设计稿 `1b` 内容区（`padding: 24px`，纵向 `gap: 16px`）：

```
[区块标题行：运行时 + 说明 + 就绪 pill + 主行动 + 「全部平台 →」]
[平台卡阵列：grid-template-columns: repeat(4, 1fr); gap: 12px]        ← flex: none
[下半区：grid-template-columns: 1.85fr 1fr; gap: 16px]                ← flex: 1
   左：用量与成本（home-usage-chart）
   右：grid-template-rows: auto 1fr; gap: 16px
       上：下一步（home-side-rail）
       下：事件流（home-side-rail）
```

响应式三档，用 CSS Media Queries Level 4 区间语法加 px 字面量，与仓库既有写法一致
（例：`ccr-ui/src/styles/components/profiles-page.css:184` `@media (width >= 1280px)`）：

| 档 | 条件 | 平台卡 | 下半区 |
|---|---|---|---|
| 宽桌面 | `@media (width >= 1440px)` | 4 列 | 1.85fr / 1fr |
| 常规桌面 | 默认（无媒体查询） | 4 列 | 1.85fr / 1fr |
| 窄窗口 | `@media (width <= 1024px)` | 2 列 | 单列堆叠 |

**不引用 `--breakpoint-*`。** 这些变量位于 `tokens.css` 中标注「仅用于参考」的 `:root` 块，
不在 `core.css` 的 `@theme` 中，既不生成 Tailwind 变体，也不能用于 `@media` 条件。

AC8 的验收视口固定为 1440px、1280px、1024px 三个宽度。

## 3. chrome 层：先核查，可能零改动

设计稿 `1c` 要求侧栏/顶栏是与卡片可区分的实色层。令牌层现状已满足：

```
--surface-shell-bg  →  --material-glass-chrome-bg  →  --color-bg-elevated   （blur: none）
--surface-card-bg   →  --color-bg-surface
```

实施第一步核查：

```bash
rg -n 'sidebar-glass|topbar-glass' ccr-ui/src/shell/ ccr-ui/src/styles/
rg -n 'surface-shell|material-glass-chrome' ccr-ui/src/shell/ ccr-ui/src/styles/
```

**核查结论 A（2026-08-25）**：`ccr-ui/src/shell/shell.css` 的 `.sidebar-glass` / `.topbar-glass` 已消费 `--surface-shell-bg` / `--surface-shell-blur` / `--surface-shell-border` / `--surface-shell-shadow`。chrome 层零改动，`shell.css` 与 `MainLayoutChrome.tsx` 移出 change list，AC1 只验证视觉结果。不改 `--material-glass-chrome-bg`。

- **结论 B（绕过语义别名，自带取值）**：把该类改为消费 `--surface-shell-bg` / `--surface-shell-border`。
  这是把组件接回既有语义层，不是新建层。本任务未走结论 B。

两种结论都**不**新增 chrome 颜色令牌，**不**改 `--material-glass-chrome-bg` 的定义或其
`prefers-reduced-transparency` 回退目标——`apple-glass-surface-contract.smoke.test.ts` 对这两处有断言。

侧栏底部的 settings dock 保持既有结构与 `data-testid="settings-dock-link"`，只换表面色与圆角令牌。

## 4. 顶栏：不新增元素

`MainLayoutTopbar` 位于 shell 层，不持有 dashboard presentation。
把 readiness 提升到顶栏需要引入 shell 层 store 与跨层依赖，父任务 D5 决定不做。

因此顶栏保持现状：左侧 `TitleTrail` 面包屑，右侧 `EnvironmentSwitcher`。
设计稿画在顶栏的就绪 pill 与主行动按钮，落在 `DashboardView` 的区块标题行——
视觉高度与顶栏相邻，满足「同屏可见」，且不破坏分层。

顶栏 `PROFILE / default` 下拉不实施（父任务 D4：应用无全局 profile 概念）。

## 5. 平台卡

`DashboardPlatformMatrix` props 不变：`rows` / `installedCliCount` / `runtimeCliCount` / `className`。

单卡结构（对应设计稿 424–520 行）：

```
[3px 平台色条]                      background: var(--color-platform-{key})
[平台名 15/600] [版本 mono 11px]  [状态 chip]
[sparkline，高 38px]                峰值柱用平台色，其余用中性色
[分隔线]
[请求 label+值 mono]  [TOKEN label+值 mono，右对齐]
```

平台色令牌映射：

| `platformKey` | 令牌 |
|---|---|
| `claude-code` | `--color-platform-claude` |
| `codex` | `--color-platform-codex` |
| `antigravity` | `--color-platform-gemini`（名称漂移，父任务已记录，本任务不重命名） |
| `opencode` | `--color-platform-opencode`（令牌层子任务新增） |

圆角按令牌子任务的四档约定：chip 用 `--radius-md`，卡片用 `--radius-2xl`，pill 用 `--radius-full`。

## 6. sparkline 派生

在 `dashboardPresentation.ts` 中，`DashboardPlatformRow` 增补：

```ts
sparkline?: number[]
trackingHealth?: 'live' | 'degraded' | 'missing'
```

`sparkline` 是 AC5 要求的唯一展示数据字段。`trackingHealth` 是 R7/AC6 占位分支所需的可选派生：平台卡 props 不能追加 `source_health`，而全零 `sparkline` 又不能当未跟踪判据。既有字段与既有消费者保持不变。

派生 sparkline：遍历 `input.overview.series`，按 `usageKey` 取对应平台的 `requests`，按日期顺序输出。
`usageKey` 已存在于 `DashboardPlatformSource`，与 `HomeOverviewSeriesItem` 的字段名对照：

| `usageKey` | `HomeOverviewSeriesItem` 字段 |
|---|---|
| `claude` | `claude` |
| `codex` | `codex` |
| `gemini` | `antigravity` |
| `opencode` | `opencode` |

`overview` 为 null 或 `series` 为空时不写 `sparkline`（保持 undefined）。

**注意**：`series` 非空时，未跟踪平台同样得到全零数组，`sparkline` 会被写成 `[0,0,...]`。
因此 `sparkline === undefined` **不是**未跟踪的判据，见 §7。

这是纯加字段，`buildDashboardPresentation` 的既有输出与消费者不受影响。

## 7. 未跟踪判据与 readiness 落位

### 7.1 未跟踪判据

后端 `usage.rs` 的 `empty_home_platform_map()` 为每个首页平台插入零值统计，
`build_home_date_range_from()` 逐日补齐。未跟踪平台与「已跟踪但本区间零用量」在 `series` 上完全一样。
`DashboardPlatformRow.state` 表示 CLI 安装/运行状态，与用量跟踪无关。

判据改用 `overview.archive.source_health: Array<UsageSourceHealth>`：

```ts
type UsageSourceHealth = { source: string, state: "live" | "degraded" | "missing", ... }
```

实施第一步验证 `source` 的取值域：

```bash
rg -n 'UsageSourceHealth \{' -A 8 ccr-ui/src-tauri/src/services/usage.rs
```

定位 759–807 行附近的构造点，读出 `source` 的实际赋值来源。

**核查结论（2026-08-25，可对应）**：`usage.rs` 把 `diagnostics.by_source` 的 `source.source`（llmusage `SourceKind`）写入 `UsageSourceHealth.source: String`。同文件快照测试断言 `"codex"`；`SourceKind::as_str()` 与 `home_usage_platforms()` 使用同一套 id。首页四卡对照：

| `usageKey` | `source_health.source` / series 字段 |
|---|---|
| `claude` | `claude` |
| `codex` | `codex` |
| `gemini` | `antigravity` |
| `opencode` | `opencode` |

因此走占位分支：`state === "missing"` → 占位；`degraded` → 状态 chip 降级但仍显示数据。匹配时同时接受 `source === usageKey`，以免历史 gemini 字面量漏判。

- **不可对应**：不实现占位态，在本文件记录原因，界面上不显示误导性的 0
  （改为显示「—」或省略该行，二选一并记录）。**不得用零值冒充占位**（父任务 D8）。本任务未走此分支。

占位分支的呈现：sparkline 位置改为虚线占位，底部两项数据替换为一行说明 + 配置入口。

### 7.2 readiness ledger 逐项落位表

`DashboardReadinessLedger` 消费的每一项都必须有结论。实施时逐行填「迁移到 X」或「删除，理由 Y」，
不留「待定」（AC3）。

| 来源字段 | 当前呈现 | 落位结论 |
|---|---|---|
| `readiness.status` | 卡片 `data-status` 与状态点 | 迁移到区块标题行就绪 pill 的 `data-status`（已有 `dashboard-header__badge`） |
| `readiness.labelKey` | eyebrow 文案 | 迁移到就绪 pill 文案 |
| `readiness.titleKey` | 卡片标题 | 迁移到区块标题行 pill 旁的可见标题（`dashboard-header__readiness-title`） |
| `readiness.descriptionKey` | 卡片描述 | 迁移到 `PageHeader` 描述，替换静态 `dashboard.description` |
| `readiness.reasons[]` | 逐条 ok/not-ok 列表 | 迁移到标题行下方的紧凑 checklist（pill 放不下逐条原因）。保留去句号展示。失败条数写入 pill 数字，与 `reasons.filter(r => !r.ok).length` 一致 |
| `statusMetrics[]` | `StatTile` 阵列 | 删除。1b 首屏是平台运行时卡，不是主机台账；CLI/后端已出现在平台卡与 `reasons[]`；CPU/内存不在 1b；`home-usage-chart` 指标行是 requests/tokens/cost/sessions，不承接本组。`buildDashboardPresentation` 仍产出该数组，界面不再消费，便于回退 |

已知约束：

- 就绪 pill 只能承载 `status` 与一个聚合数字，承载不了 `reasons[]` 与三个文案键。
- `statusMetrics` 含 CPU、内存、后端、CLI 计数等，与用量指标行不是同一组数据；
  `home-usage-chart` 的指标行是 requests / tokens / cost / sessions，**不承接** `statusMetrics`。
  这条推翻了上一版设计中「`statusMetrics` 由用量指标行承载」的说法。
- 若某项确实无处可去，合法结论是「删除」，但必须写明为什么这条信息在新 IA 下不再需要。

本子任务不删除 `buildDashboardPresentation` 的 `readiness` / `statusMetrics` 输出，
即使界面不再消费——保留输出可让落位决策后续可逆。

## 8. 测试

| 测试文件 | 断言 |
|---|---|
| `ccr-ui/tests/dashboard-presentation.smoke.test.ts` | sparkline 派生：正常输入长度等于 `series` 长度且顺序与日期一致；`overview == null` 时为 `undefined`；`series` 为空时为 `undefined`；`usageKey: 'gemini'` 取到 `antigravity` 字段 |
| `ccr-ui/tests/dashboard-platform-matrix.smoke.test.tsx`（新建） | 占位分支：构造 `source_health` 中 `state: "missing"` 的 fixture 触发占位；构造全零 `series` **不**触发占位（AC6 的关键区分） |

组件测试落点：`ccr-ui/tests/` 无既有 `DashboardPlatformMatrix` 覆盖，新建 `dashboard-platform-matrix.smoke.test.tsx`。

fixture 辅助：`ccr-ui/tests/helpers/usageFixtures` 已有 `makeArchiveDiagnostics`，
`source_health` 的 fixture 构造复用它。

## 9. 回滚

```bash
git checkout -- ccr-ui/src/shell/ \
  ccr-ui/src/features/usage/dashboard/DashboardView.tsx \
  ccr-ui/src/features/usage/dashboard/DashboardPlatformMatrix.tsx \
  ccr-ui/src/features/usage/styles/ \
  ccr-ui/src/views/dashboard/dashboardPresentation.ts \
  ccr-ui/tests/
```

若 §7.2 结论为删除 ledger 文件，回滚需 `git checkout -- <两个文件路径>` 恢复。
`dashboardPresentation.ts` 的加字段向后兼容，即使单独保留也不会破坏其他子任务。
</content>
