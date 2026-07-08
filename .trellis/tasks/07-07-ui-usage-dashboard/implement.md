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
6. [x] 第一屏重排(design.md §5):指标卡上移首行、cockpit 拆为 StaleBanner + 诊断抽屉、L/M/D 人话化、degraded 解释与动作、空告警隐藏、meta popover、去 ambient。
   - 验证:1920/1280/900px 三档截图;stale 与健康两种状态截图;i18n 齐全。
   - 已完成(未提交):UsageOpsCockpit.vue(449行)删除,拆为 UsageStaleBanner.vue(仅
     state≠'ready' 时渲染的单行横幅)+ UsageDiagnosticsDrawer.vue(包 BaseModal,装健康格
     +来源明细+告警,alerts 为空时整节不渲染)+ UsageCostConclusionCard.vue(费用结论卡,
     slot 内嵌 TokenBreakdownStrip)。UsageDashboardView.vue 首屏改
     `.usage-hero-row`(7fr/5fr grid):左结论卡,右 `.usage-metric-grid` 装剩余 3 张
     UsageMetricCard(tokens/activeDays/requests,cost 已挪进结论卡)。断点:
     <1280px hero-row 转 1fr 纵向堆叠,<900px metric-grid 转 1fr 单列。
     L/M/D 人话化:usageOverviewInsights.ts 新增 formatSourceCounts()(三个独立
     词条 liveLabel/missingLabel/deletedLabel 拼接,不用 ICU 占位符),替换
     usageOpsCockpit.ts 两处 + buildDashboardMetaItems 的 archive chip 一处裸
     `L/M/D` 拼接;抽屉与 toolbar 数据源 popover 内实测均为 "Live 1 · Missing 1 ·
     Deleted 0"/"在档 2,053 · 缺失 2,829 · 已删 0"。degraded/missing 来源卡新增
     `hint` 字段(i18n sourceStateHints.degraded/missing)+ 行内"刷新 usage"按钮,
     emit `refresh` 冒泡到 doImport。meta chips 迁入 UsageDashboardToolbar.vue 新增
     `metaItems` prop 驱动的"数据源"popover(点击外部/Escape 关闭,模式参考
     EnvironmentSwitcher.vue 但按钮语义改 role="group"+aria-label,非 dialog)。
     `.usage-page__ambient` 径向渐变层整段删除。openDiagnostics 因唯一调用方改为
     直接开抽屉而从 useUsageDashboardState.ts 移除;handleOpsPrimaryAction 的
     'diagnostics' 分支(needs_session_index 等状态下跳 Logs tab 走修复流程)保持不动。
     新增 i18n key(zh/en 成对):ops.health.liveLabel/missingLabel/deletedLabel、
     ops.sourceStateHints.degraded/missing、ops.drawerTitle、ops.healthGridTitle、
     toolbar.dataSource、cards.periodOverPeriod。
   - 验证记录:`just frontend-check-quick` 全绿(type-check/lint:ci/test:i18n 23/23/
     test:smoke 372/372,含新增 usage-stale-banner.smoke.test.ts +
     usage-diagnostics-drawer.smoke.test.ts,删除并拆分原 usage-ops-cockpit.smoke.test.ts)。
     用 `.trellis/tasks/07-07-ui-usage-dashboard/research/perf-harness/tauri-shim.js`
     注入 Playwright,`ccr-ui-web`(15173)起服务,实测 1920/1280/1200/900/850px ×
     stale/健康 两态:健康态横幅不渲染、结论卡+指标格并排正确、<1280 纵向堆叠、
     <900 指标格转单列均在 1200/850 px 处实测确认(1280/900 整数边界处仍是断点未触发前
     的状态,符合 `width < N` 语义);stale 态横幅一句话+相对时间+两按钮、点开诊断抽屉见
     人话化健康格与 Codex(degraded)/Antigravity(missing)/Claude(live)三来源卡、
     Antigravity 无告警数据时"运维告警"整节不渲染(非空列表兜底文案)。截图存档于
     `research/layout-item6/`。frontend-quality-reviewer 复核后修了两处:toolbar
     popover 的 `role="dialog"` 缺 aria-modal/labelledby 改为 `role="group"` +
     aria-label;UsageCostConclusionCard 补全遗漏的 `.usage-cost-conclusion--sand`
     tone 规则(此前靠基类默认值巧合渲染正确)。追加修复:手动点"刷新 usage"此前在浏览器
     测试桩下报 `TypeError: Cannot read properties of null (reading 'snapshot')`,
     溯源到 `stores/usage.ts:800`(`start_usage_import_job_v2` 桩未覆盖导致
     `response` 为 null;修完后暴露第二处同源缺口:`ensureImportJobListeners` 紧接着调
     `get_usage_import_job_status_v2` 读 `.status`,同样未打桩)——两处均为测试桩缺口,
     `stores/usage.ts` 本身本轮未改动。已在 `research/perf-harness/tauri-shim.js` 补
     `buildImportJobSnapshot()` 统一构造 `status:'finished'` 快照,两条命令共用;
     Playwright 复测点击链路(打开抽屉→点 Codex 卡"刷新 usage")0 console error,
     两条命令均按预期参数被调用。另,复核发现 `usage.dashboard.ops.sourcesHint` 仍是未人话化的
     `'live / missing / deleted'` 裸英文,已在抽屉的"来源健康"小节标题旁展示,与本项
     "人话化"精神相悖但属既有代码、本项未触碰,留作后续小任务。
6b. [x] formatTokens/formatCost 格式化升级(≥1B 用 B、千分位)。
   - 验证:12527.4M → 12.53B;$26,114.04;既有单测/快照更新。
   - 已完成(commit 03ff641d):smoke 9/9 通过,首屏实测 12.53B / $26,114.04。
7. [x] logs 骨架行 + sticky 表头;图表动画接 prefers-reduced-motion。
   - 验证:reduced-motion 模拟下无入场动画。
   - 已完成(未提交):
     **7a** UsageLogsTab.vue:loading 态由单行"加载中"文字改为骨架行——复用
     `diagnostics-tab__row--item` 网格(6 列,后 3 列 is-right),每格一个静态灰块
     `diagnostics-tab__skeleton`(无 shimmer,天然兼容 reduced-motion),整块
     aria-hidden。行数 `Math.min(ctx.logsPageSize, 12)`(context 新增
     `logsPageSize: toRef(store,'logsPageSize')`;分页大小 50,但滚动容器
     max-height 内只见约 12 行,多渲染无意义)。sticky 表头:滚动容器从
     `__body`(max-height 32rem/overflow auto)上移到 `__ledger`(max-height
     35rem/overflow auto,原 overflow:hidden),`__header` 加 position:sticky
     top:0 z-index:1(背景本就是 92% 不透明 elevated 色);横向滚动时表头与行
     同容器滚动保持列对齐,__body 自身的滚动样式删除。
     **7b** 选项 B(已与用户确认):usageChartOptions.ts 模块级
     `prefersReducedMotion` ref + `matchMedia('(prefers-reduced-motion: reduce)')`
     change 监听;`buildChartAnimations()` 返回 `{ enabled: !ref }`,TREND/PIE
     两个冻结 base 中删除硬编码 `animations:{enabled:false}`,工厂里
     `{ ...BASE, animations: buildChartAnimations() }` 注入。options 工厂在
     useUsageCharts 的 computed 内被调用,ref 变化经依赖追踪自动重建 options,
     无需组件侧接线;记忆化不受影响(动画开关不依赖数据)。
   - 验证记录:`just frontend-check-quick` 全绿(type-check/lint:ci/test:i18n
     23/23/test:smoke 372/372)。Playwright + tauri-shim 实测(vite 15173):
     ① `emulateMedia({reducedMotion:'reduce'})` 下 matchMedia=true,趋势线
     path `d` 在渲染后 500ms 内完全静止(无入场动画);② no-preference 下
     6 次 120ms 间隔采样得 6 个不同 path(入场动画正常播放)。诊断 tab:
     表头 computed position=sticky/z-index=1,ledger scrollTop=400 前后表头
     相对容器 offset 恒为 1px(sticky 生效);强制 `logsLoading=true` 后渲染
     12 骨架行 × 6 格 = 72 个占位块、表头仍在,复位后恢复 20 条真实行。
     全程 0 console error。
8. [ ] `bun run type-check && bun run lint` + `just frontend-check-quick`。
9. [ ] 前后性能数据对比写入 research/,截图入 research/;review gate。

## Rollback

按 design.md §7 commit 划分独立 revert;性能不达标时保留 ①② 仅回滚 ③(KeepAlive 内存不可控时)。
