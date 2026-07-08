# 技术设计:使用统计页性能与排版优化

## 1. Tab 缓存结构

```vue
<KeepAlive :max="4">
  <component :is="activeTabComponent" v-bind="activeTabProps" />
</KeepAlive>
```

- 7 个 tab 组件已是独立 SFC,改为 `shallowRef` 组件映射表 + computed activeTabComponent;defineAsyncComponent 懒加载保持。
- 各 tab 组件内图表用 `onActivated/onDeactivated` 暂停/恢复(ApexCharts 无需显式 pause,但要避免 deactivated 期间响应 options 变化 → 在 tab 组件内对 props 做 activated 门控,或接受 KeepAlive 默认行为并实测)。
- props 透传:现有 UsageDashboardView 给每个 tab 传 8~20 个 props;改为提供 `provide/inject` 的 usage 上下文对象(由 useUsageDashboardState 返回值构成),tab 组件注入取用——同时解决 props 面爆炸。此改动限制在 usage 目录内。

## 2. Options 记忆化

- 新建 `src/views/usage/chartOptions.ts`:导出 `buildTrendOptions(theme, locale)` / `buildPieOptions(...)` 等工厂,静态骨架为模块级冻结对象,工厂只注入主题色与格式化函数。
- composable 内:`const trendOptions = computed(() => buildTrendOptions(themeKey.value, locale.value))`——依赖只剩 theme/locale;series 单独 computed。数据刷新 → 只有 series 引用变化 → ApexCharts 走 updateSeries 快路径。
- 图表颜色统一走 `styles/chart-colors.css` 既有 CSS 变量读取(getComputedStyle 一次缓存,theme 变化时失效)。

## 3. Composable 拆分(纯搬移)

```
views/usage/
  useUsageDashboardState.ts   # 组合根,≤400 行
  state/useUsageFilters.ts    # platform/range/tab + onFilterChange
  state/useUsageCharts.ts     # trend/pie series+options
  state/useUsageLogs.ts       # logs 分页/筛选/修复
  state/useUsageMeta.ts       # cockpit/meta/empty 文案
  chartOptions.ts
```

导出签名保持不变(UsageDashboardView 解构不动),内部 re-export。

## 4. Sparkline 合一

- 目标实现:`components/ui/Sparkline.vue`(从 profiles/Sparkline.vue 出发,54 行 SVG polyline);补 props:`values, width, height, stroke(默认 currentColor), fill(可选渐变), ariaLabel`。
- 迁移引用:usage/UsageMetricCard、UsageOpsCockpit(用 SparkLine.vue 的)、dashboard/DashboardUsageMovement(如引用 UsageSparkline)、profiles/ProfilesStatStrip;删除两份旧实现。
- 若 usage/SparkLine.vue 有 tooltip/渐变等超集能力,先盘点使用面:未用到的能力不迁移。

## 5. 第一屏重排(截图复核后:cockpit 降级为横幅)

```
[toolbar(平台/窗口/导入 + "数据源"popover 收纳 meta chips)]
[⚠ 过期横幅(条件渲染):Usage 数据 4 天未同步 · [刷新 usage] [查看诊断]]
[费用结论卡 7col            | 指标 2×2 5col        ]
[  $26,114.04 +48% 环比     |  tokens/请求/        ]
[  TokenBreakdownStrip 内嵌 |  活跃天数/模型数      ]
[tab 胶囊条]
[tab 内容]
```

- UsageOpsCockpit 组件重构为两部分:`UsageStaleBanner`(横幅,仅 stale 时渲染)+ 诊断抽屉内容(来源健康/快照缓存/深钻维度/运维告警迁入现有 openDiagnostics 抽屉);L/M/D 缩写在横幅与结论区不出现,抽屉内用全称(在档/缺失/已删)。
- degraded 徽章:tooltip 或行内附一句解释 + "刷新"动作;运维告警区无内容时不渲染。
- 结论卡数据全部来自现有 summaryCards/opsCockpit presentation,不新增后端调用。
- formatTokens/formatCost 在 format 工具层升级:≥1B 用 B 单位、千分位、保持 tabular-nums。
- <1280px:结论卡与指标格纵向堆叠;<900px 指标格 2×1。
- ambient 层删除;`usage-page` 背景交给全局 base(StageBackground 已提供氛围)。

## 6. 权衡

- 不换图表库(ECharts 迁移成本大);ApexCharts 在记忆化 + KeepAlive 后瓶颈可控,若实测仍卡再立后续任务。
- provide/inject 只在 usage 域内使用,不上升为全局模式,避免破坏其他页面 props 显式传递的可读性。

## 7. 回滚

commit:①composable 拆分(无行为变化) ②options 记忆化 ③KeepAlive ④sparkline 合一 ⑤排版重构;①-④ 各自可独立 revert,⑤ 依赖 ①。
