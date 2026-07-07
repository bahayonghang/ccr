# 使用统计页性能与排版优化

## Goal

优化 UsageDashboardView 的两个维度:(1) 性能——消除 tab 切换时的图表全量销毁重建、options 对象反复重建导致的 ApexCharts 重渲染;(2) 排版——把平坦并列的信息块重排为"结论优先"的层级,落地新材质体系。

## 现状问题清单(代码证据)

| # | 问题 | 位置 |
|---|---|---|
| U1 | 7 个 tab 用 template v-if/v-else 切换,切 tab 即销毁/重建整棵子树与图表实例 | UsageDashboardView.vue:128-239 |
| U2 | `useUsageDashboardState.ts` 1005 行、30 个 computed;trendOptions/pieOptions 等图表 options 每次响应式变化整体重建 → ApexCharts 全量 re-render | src/views/usage/useUsageDashboardState.ts |
| U3 | 3 份重复 sparkline 实现 | usage/SparkLine.vue(262)、usage/UsageSparkline.vue(149)、profiles/Sparkline.vue(54) |
| U4 | 信息层级平坦:OpsCockpit + 4 指标卡 + TokenBreakdownStrip + meta chips 并列,无主次 | UsageDashboardView.vue:41-81 |
| U5 | 页面级 ambient 径向渐变常驻(装饰),叠加在滚动区上 | UsageDashboardView.vue:372-380 |
| U6 | logs 表分页已有,但筛选变化时无 loading 骨架,表格跳变 | usage/UsageLogsTab.vue |
| U7 | 运维驾驶舱占满首屏:过期告警大标题 + 6 个运维 tile("30s TTL"/"7 个维度"/L·M·D 密码式指标),结论数字(总费用/总 tokens)沉到折叠线下 | 截图4 |
| U8 | 数字格式未本地化:12527.4M(应 12.53B)、$26114.04(无千分位) | 截图4 指标卡 |
| U9 | "运维告警"空面板整块占位;三个来源全部 degraded 徽章但无解释与修复动作 | 截图4 来源健康区 |

## Requirements

### R1 性能

- tab 内容改 `<KeepAlive>` + 动态组件(或 v-show 关键 tab):已访问过的 tab 二次进入不重建图表实例;首次进入仍懒加载(defineAsyncComponent 保持)。
- 图表 options 记忆化:静态部分(axis 样式、grid、tooltip 主题)提为模块级常量,动态部分(series、categories、颜色)最小化 diff;确保 locale/theme 变化才重建 options 引用,数据刷新只更新 series。
- `useUsageDashboardState.ts` 拆分:按 tab 域拆为 useUsageTrendCharts / useUsagePieCharts / useUsageLogs 等子 composable(纯移动,不改逻辑),主 composable 组合导出,单文件 ≤400 行。
- sparkline 三合一:保留一个实现(以 profiles/Sparkline.vue 的轻量 SVG 为基,补齐 usage 需要的 props),其余删除并迁移引用。
- 切换平台/时间窗时,后端请求已在 store 层;确保 UI 侧无级联重复请求(onFilterChange 只触发一次加载)。

### R2 排版(截图复核后强化:结论优先)

- **指标卡上移为首屏第一行**(U7 修复的核心):总费用(视觉最重、含环比)+ 总 tokens + 总请求 + 活跃天数;TokenBreakdownStrip 并入费用/tokens 结论卡下沿。
- **运维驾驶舱降级为一条状态横幅**:数据过期 → warning 横幅(一句话 + 相对时间 + "刷新 usage"按钮);数据健康 → 不渲染;来源健康/快照缓存/深钻维度等诊断细节全部收进"查看诊断"抽屉(现有 openDiagnostics 入口)。
- **术语人话化**:"L 2,053 · M 2,829 · D 0"改"在档 2,053 · 缺失 2,829 · 已删 0";"30s TTL"/"7 个维度"类开发者措辞不出现在页面主层;degraded 徽章附一句解释与修复动作(如"归档 4 天未同步,点击刷新")。
- **空面板不渲染**:"运维告警"无内容时整块隐藏(U9)。
- **数字格式化**(U8):formatTokens ≥1B 显示 B 单位(12.53B);formatCost 千分位($26,114.04);统一在 format 层处理,全站受益。
- meta chips 降为工具栏右侧"数据源"popover,不再单独占一行。
- tab 切换器沿用胶囊样式但贴新令牌(不透明表面 + 选中态 accent 边框);ambient 装饰层移除。
- logs 表:筛选/翻页时行区域显示骨架行,表头 sticky。

### R3 材质与降级

- 页面所有卡片用不透明 card 档;无新增 backdrop-filter(玻璃预算留给外壳与模态)。
- 图表动画尊重 prefers-reduced-motion(ApexCharts `animations.enabled` 跟随)。

## Out of Scope

- llmusage 同步/导入流程、store 数据层与 IPC(仅消费);诊断对话框逻辑。
- Claude Observer 页(claude-observer/* 组件)——清扫任务处理。

## Acceptance Criteria

- [ ] tab 在 overview↔tokens↔cost 间来回切换:第二次进入 tab 无图表重建(Performance 面板对比前后,记录耗时数据)。
- [ ] 数据窗口切换(7d→30d)时图表只更新 series,不整图闪烁重挂载。
- [ ] `useUsageDashboardState.ts` ≤400 行,拆分后 type-check 通过且行为不变。
- [ ] 仓库内 sparkline 实现只剩 1 份,`rg -l "SparkLine|UsageSparkline"` 无旧引用。
- [ ] 第一屏截图(1080p):第一行即为结论指标卡;数据过期时仅一条 warning 横幅,健康时无运维元素;诊断细节在抽屉内可达。
- [ ] 12527.4M → 12.53B 格式;费用千分位;L/M/D 缩写在主层不出现。
- [ ] 空"运维告警"不渲染;degraded 徽章有解释与修复动作。
- [ ] prefers-reduced-motion 下图表无入场动画。
- [ ] `bun run type-check && bun run lint` + frontend-check-quick 通过。

## Dependencies

- 依赖 07-07-ui-glass-tokens(card 档不透明令牌)。
