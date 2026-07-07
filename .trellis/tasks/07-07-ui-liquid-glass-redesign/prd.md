# ccr-ui 液态玻璃视觉与交互重构总纲

## Goal

以一次性的深度审计为基础,分阶段重构 ccr-ui 的视觉与交互:修复亮色主题"泛白、层次糊"的问题,建立一套**性能受控的液态玻璃材质体系**,统一 Claude/Codex Profiles 两页的交互模型,并优化使用统计页的性能与排版。最终全站回到唯一的视觉主线:Anthropic-like 编辑式工作台 + 暖中性表面 + 克制而真实的玻璃深度。

## 背景:代码审计结论(2026-07-07)

### A. 配色泛白的根源(tokens.css)

1. 亮色 clay 底色三层亮度几乎相同:`--color-bg-base #f4ede3` / `--color-bg-elevated #fbf6ee` / `--color-bg-surface #fffaf3`,层与层之间肉眼难以区分。
2. 边框透明度过低:`--color-border-subtle` 8% / `default` 13% / `strong` 21%,卡片边界近乎不可见。
3. 阴影过弱:`--shadow-sm` 5% / `--shadow-md` 7% alpha,暖色底上几乎不可感知,elevation 三档失效。
4. 玻璃令牌"名不副实":`--glass-blur-sm/md/lg/xl` 仅 blur(2px)~blur(6px) + saturate(104%~110%),低于产生玻璃质感的下限(业界实践 8~16px + saturate 140%+);渲染结果是"白雾片"而不是玻璃,这是"泛白"观感的直接来源之一。
5. 大量表面用 86%~96% 的半透明白叠在米白底上(`--glass-bg-*`、`--home-surface-card` 92%),半透明没有带来材质感,只稀释了对比。

### B. 液态玻璃性能约束(网络调研结论)

- `backdrop-filter` 每一处都强制 GPU 拷贝背景缓冲、模糊、回贴;滚动容器上或大面积使用会持续重绘。
- 实践准则:同屏玻璃面板 ≤3 块;blur 控制在 8~16px 并用 saturate 补偿;玻璃元素加 `contain`/独立合成层;禁止嵌套玻璃;必须提供 `prefers-reduced-transparency` 与低端设备降级。
- 当前代码现状:backdrop-filter/blur 令牌引用散布 31 个文件共 75 处,方向刚好相反——"处处轻微模糊"而不是"少数关键表面真实玻璃"。

### C. Profiles 两页交互割裂(用户截图问题 #2)

1. 确认交互不一致:Claude 页用原生 `confirm()/alert()`(ClaudeCodeProfilesView.vue:1112/1145/1157/1165/1177),Codex 页用 `ConfirmModal` 组件。
2. 快捷键语义冲突:Codex 页 ⌘K 打开命令面板,Claude 页 ⌘K 聚焦搜索框;Codex 有 ⌘1-9 快速切换、QuickRail、CommandPalette,Claude 全没有。
3. Codex 统计条展示硬编码假数据:`:total-spark="[3,5,4,6,7,8,7]"` / `:recent-spark="[2,4,3,5,4,6,5]"`(CodexProfilesView.vue:58-59),以真实数据的样貌呈现假趋势。
4. 主色轴不一致:Claude 页 `--cp-accent` 取 accent-secondary(沙色),Codex 页取 accent-primary(陶土色),同族页面双色轴。
5. Codex 卡片视图为强制单列(`cp-grid` 单列注释),宽屏浪费;Claude 卡片视图同样单列。
6. "最近写入"提示取的是列表加载时间而非真实写入时间(两页同病)。
7. Claude 编辑 modal 20+ 字段 4 分区,分区滚动同步用未节流的 @scroll 处理器;首屏无引导层级,新手无从下手。

### D. 使用统计页(用户截图问题 #4)

1. ApexCharts 已 defineAsyncComponent 懒加载(好),但 tab 用 v-if/v-else 切换,每次切 tab 图表实例全量销毁重建;无 keep-alive。
2. `useUsageDashboardState.ts` 单文件 1005 行、30 个 computed,趋势/饼图 options 对象在每次响应式变化时整体重建,触发 ApexCharts 全量重渲染。
3. 项目内存在 3 份重复的 sparkline 实现(usage/SparkLine.vue 262 行、usage/UsageSparkline.vue 149 行、profiles/Sparkline.vue 54 行)。
4. 排版:顶部 cockpit + 4 张指标卡 + token 分解条 + meta chips 全部并列,信息优先级平坦,重点(成本/趋势异常)不突出。

### E. 全站一致性

1. 原生 `confirm()/alert()` 残留 8+ 文件:ClaudeCodeProfilesView、ClaudeAuthView、McpManagerView、AgentDetailView、BaseSlashCommands、McpPresetsPanel、usePlatformMcp、usePlatformPlugins。
2. 长列表几乎无虚拟化(仅 HistoryList),无限动画散布 18 个文件 25 处。
3. 新旧页面 DNA 割裂:Dashboard/Profiles 已是"编辑式工作台",而 McpManager、AgentDetail、ClaudeAuth 等仍是旧代。

### F. 截图复核结论(2026-07-07,深色模式实截 4 张)

1. **字体体系单一**:`--font-sans/--font-brand/--font-mono` 全部指向 MapleBright,大号 CJK 标题呈终端等宽观感,层级只能靠字号硬撑;仅 mocha flavor 有 brand/mono 分离覆盖(SF Pro Display / Cascadia),其余 flavor 缺失。→ 07-07-ui-glass-tokens R5
2. **深色主题同样层次糊**:面板与底色亮度差过小、边框几乎不可见,与亮色泛白同根源。→ 07-07-ui-glass-tokens R1
3. **首页双 Hero + 文案重复**:页头"运行概览"之下"处理阻塞项"又是巨型 display 标题;"用量趋势"eyebrow 与标题文字完全重复,数据来源说明句出现两次。→ 07-07-ui-shell-home R2
4. **告警语义过载**:3 条前端日志错误(Failed to save Claude prof... ×3)同时驱动红色阻塞卡、红色事件 tile、行动队列 01 项;事件流排序混乱且重复条目不聚合;行动队列 01-04 编号是装饰,不对应快捷键。→ 07-07-ui-shell-home R2/R4
5. **Profiles 双页信息设计问题**:Codex 卡片把只读 base_url/model 渲染成输入框样式(假可编辑);Claude 页 19 行重复大号"应用此 Profile"按钮;当前 profile 未在列表置顶;base_url 空间充足仍被截断;右栏分布渲染 0 值条目;两页页头命名不一致;假 sparkline 截图可见。→ 07-07-ui-profiles-unify P9-P13
6. **Usage 首屏被运维驾驶舱占满**:过期告警大标题 + "30s TTL"/"7 个维度"/L·M·D 密码式指标堆满第一屏,结论数字(总费用/总 tokens)沉到折叠线下;空"运维告警"面板占位;数字格式未本地化(12527.4M 应为 12.53B,$26114.04 无千分位)。→ 07-07-ui-usage-dashboard U7-U9

## Requirements

- R1 修复亮色主题泛白:表面分层、边框、阴影、文字对比全面重标定,亮/暗两套主题都达到高对比(WCAG AA 正文,标题 AAA 目标)。
- R2 建立"真实但受控"的液态玻璃材质体系:玻璃只用于少数悬浮层(侧栏、顶栏、模态、浮动面板),blur 8~16px + saturate 补偿,同屏 ≤3 块,提供 reduced-transparency/低端降级;普通卡片回归不透明分层表面。
- R3 统一 Profiles 双页交互模型(确认对话框、快捷键、命令面板、卡片栅格、accent 轴),移除假数据。
- R4 使用统计页:图表渲染开销可控(tab 缓存、options 记忆化)、排版重点分明。
- R5 清除全站原生 confirm/alert,旧页面逐步对齐编辑式 DNA。
- R6 所有动效兼容 prefers-reduced-motion;所有玻璃兼容 prefers-reduced-transparency。
- R7 遵守 theme-token-contracts:保持 data-theme / data-flavor / data-accent 三层正交,clay 仍为默认 flavor,Catppuccin 语义重映射不变。
- R8 字体三轨分离:brand(标题)/sans(正文)/mono(数值与代码)三条字体轨分离且**全 flavor 生效**;CJK 大标题不得呈等宽渲染观感(承接 F1)。

## 约束

- 品牌红线(ccr-ui/CLAUDE.md):禁止紫色科技感/anime/guofeng;暖中性 + 编辑式排版是唯一主线;液态玻璃是"材质深度",不是新视觉分支。
- 液态玻璃为 Web 近似实现(backdrop-filter + 分层边框 + 高光),不得宣称为 Apple 官方实现。
- 主题烟雾测试契约必须同步维护:apple-glass-surface-contract / theme-bootstrap / app-settings smoke tests。
- 秘密掩码、备份、原子写等持久化行为不受影响(本次纯前端视觉/交互)。

## 任务地图(子任务)

| 顺序 | 任务 | 范围 | 依赖 |
|---|---|---|---|
| 1 | 07-07-ui-glass-tokens | tokens.css/home.css 材质与对比度体系 | 无(其余全部依赖它) |
| 2 | 07-07-ui-shell-home | MainLayout 外壳 + Dashboard 首页落地新材质 | 1 |
| 3 | 07-07-ui-profiles-unify | Claude/Codex Profiles 交互与视觉统一 | 1 |
| 4 | 07-07-ui-usage-dashboard | 使用统计页性能与排版 | 1 |
| 5 | 07-07-ui-consistency-sweep | 原生对话框清除 + 旧页面对齐 | 1(3 完成后收益最大) |

## Acceptance Criteria(跨子任务)

- [ ] 亮色 clay 下,base/elevated/surface 三层在截图中肉眼可分层;卡片边界清晰;阴影可感知。
- [ ] 同屏启用 backdrop-filter 的元素 ≤3 个(DevTools Layers 面板验证);滚动 60fps 无掉帧(Performance 面板抽查 Dashboard/Profiles/Usage 三页)。
- [ ] prefers-reduced-transparency 下所有玻璃面板回退为不透明表面;prefers-reduced-motion 下无常驻动画。
- [ ] Claude/Codex Profiles 两页:同一套确认对话框、同一套快捷键语义、无硬编码假 sparkline。
- [ ] 使用统计页 tab 来回切换不再全量重建图表(实测切换耗时下降,记录前后数据)。
- [ ] `rg "\\b(confirm|alert)\\(" ccr-ui/src` 仅剩注释/降级路径命中。
- [ ] `cd ccr-ui && bun run type-check && bun run lint` 通过;三个主题 smoke 测试通过。
- [ ] 亮/暗 × clay/paper/graphite/mocha 关键页面截图验收(记录 dataset 值)。
- [ ] 页面 display 级大标题以比例字体渲染,数值/代码区保持等宽(F1);首页只有一个 display 级大标题且无重复文案(F3)。
- [ ] Usage 页 1080p 首屏第一视觉为结论指标卡,过期告警收敛为一条横幅(F6)。

## Notes

- 液态玻璃性能实践参考:LogRocket《How to create Liquid Glass effects with CSS and SVG》、Lucky Graphics《High-Performance Refractive UI》、Developer Playground《Glassmorphism Implementation Guide》。
- 本父任务不直接承载实现;最终做跨子任务集成验收。
