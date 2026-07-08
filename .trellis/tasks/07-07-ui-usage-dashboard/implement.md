# 执行计划:使用统计页性能与排版优化

## Checklist

1. [x] 基线性能录制:tab 切换 ×3、窗口切换 ×2 的 Performance trace + 耗时记录,存 research/。
   - 已完成(2026-07-08):research/perf-harness/(tauri-shim.js 注入 harness + notes) + research/baseline/(baseline-perf.json、stale/healthy 首屏截图)。实测 12/12 次 tab 再进入图表全量重建、窗口切换图表重挂载,U1/U2 实锤;20 次往返无泄漏。
2. [x] composable 拆分(纯搬移,签名不变)。
   - 验证:`bun run type-check`;页面行为无差异;主文件 ≤400 行。
   - 已完成(commit 5b12c732):1005→390 行,state/{filters,charts,logs,meta} 四个子 composable;vue-tsc/eslint 通过;harness 7 tab 回归通过。
3. [x] chartOptions.ts 工厂 + options 记忆化;series 独立 computed。
   - 验证:窗口切换时图表不闪烁重挂载(录屏对比);theme/locale 切换正常重建。
   - 已完成(commit 6aa0947b):usageChartOptions.ts 工厂化,options 依赖收敛至 theme/locale/轴形状,labels/tickAmount join-key 记忆化。注:窗口切换不重挂载需配合第 4 步加载语义修正(loading v-if 拆树是另一半根因)。
4. [x] KeepAlive + 动态组件 tab;provide/inject usage 上下文。
   - 验证:二次进入 tab 无重建(对比第 1 步基线,记录数据);内存无异常增长(切 20 次)。
   - 已完成(未提交):usageDashboardContext.ts provide/inject + UsageDashboardView.vue 改
     KeepAlive(max=4)+ component :is="activeTabComponent"，7 个 tab 组件全部从 props 迁到
     inject。此前会话只完成 Overview/Tokens/Cost 三个，且 CostTab 自身有残留 props. 引用未替换；
     本轮补完 Providers/Models/Projects/Logs 四个并修复 CostTab 残留。models tab 用 modelToken*
     系列字段（区别于 overview 的 cost 系列 pieOptions/pieSeries）；logs tab 两处改名
     showPager->showLogsPager、repairButtonLabel->repairCodexButtonLabel。用 research/perf-harness/
     tauri-shim.js（Playwright addInitScript 注入伪造 Tauri IPC）在浏览器里跑通全部 7 个 tab 真实
     fixture 数据渲染 + 分页 + KeepAlive 状态保留验证（Tokens tab 切到"总量"模式后经 Overview 往返，
     aria-selected 仍为 true，证实同一实例未重建；max=4 的 LRU 淘汰行为符合设计预期）。6 个 smoke
     测试（cost/logs/models/overview/providers/tokens）同步从 props 挂载改为
     tests/helpers/usageDashboardContextStub.ts 的 provide 包装。type-check/lint/test:smoke
     (369/369) + just frontend-check-quick 全绿；性能对比数据留给第 9 步统一录入。
5. [x] Sparkline 三合一,删两份旧实现。
   - 验证:`rg -l "usage/SparkLine|UsageSparkline"` 零引用;涉及卡片截图正常。
   - 已完成(commit 01509644):components/ui/Sparkline.vue 唯一实现;UsageMetricCard/ProfilesStatStrip 已迁移;rg 零残留;指标卡 SVG 渲染回归通过。
6. [ ] 第一屏重排(design.md §5):指标卡上移首行、cockpit 拆为 StaleBanner + 诊断抽屉、L/M/D 人话化、degraded 解释与动作、空告警隐藏、meta popover、去 ambient。
   - 验证:1920/1280/900px 三档截图;stale 与健康两种状态截图;i18n 齐全。
6b. [x] formatTokens/formatCost 格式化升级(≥1B 用 B、千分位)。
   - 验证:12527.4M → 12.53B;$26,114.04;既有单测/快照更新。
   - 已完成(commit 03ff641d):smoke 9/9 通过,首屏实测 12.53B / $26,114.04。
7. [ ] logs 骨架行 + sticky 表头;图表动画接 prefers-reduced-motion。
   - 验证:reduced-motion 模拟下无入场动画。
8. [ ] `bun run type-check && bun run lint` + `just frontend-check-quick`。
9. [ ] 前后性能数据对比写入 research/,截图入 research/;review gate。

## Rollback

按 design.md §7 commit 划分独立 revert;性能不达标时保留 ①② 仅回滚 ③(KeepAlive 内存不可控时)。
